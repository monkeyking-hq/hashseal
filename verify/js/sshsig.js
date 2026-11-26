/**
 * Pure-JS OpenSSH SSHSIG parser + Ed25519 verifier (Node built-in crypto only).
 * Mirrors hashseal-core signing.rs for `ssh-ed25519` in namespace `hashseal`.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
"use strict";

const crypto = require("crypto");

// DER SubjectPublicKeyInfo prefix for Ed25519 (RFC 8410):
// SEQUENCE { SEQUENCE { OID 1.3.101.112 }, BIT STRING (32 bytes) }
const ED25519_SPKI_PREFIX = Buffer.from("302a300506032b6570032100", "hex");

class Reader {
  constructor(buf) {
    this.buf = buf;
    this.off = 0;
  }
  remaining() {
    return this.buf.length - this.off;
  }
  readBytes(n) {
    if (this.off + n > this.buf.length) {
      throw new Error("unexpected EOF in SSHSIG");
    }
    const s = this.buf.subarray(this.off, this.off + n);
    this.off += n;
    return s;
  }
  readU32() {
    if (this.off + 4 > this.buf.length) {
      throw new Error("unexpected EOF reading u32");
    }
    const v = this.buf.readUInt32BE(this.off);
    this.off += 4;
    return v;
  }
  readString() {
    const len = this.readU32();
    return this.readBytes(len);
  }
}

function sshString(buf) {
  const b = Buffer.isBuffer(buf) ? buf : Buffer.from(buf, "utf8");
  const out = Buffer.allocUnsafe(4 + b.length);
  out.writeUInt32BE(b.length, 0);
  b.copy(out, 4);
  return out;
}

function parseSshSigArmor(armor) {
  const lines = String(armor)
    .replace(/\r\n/g, "\n")
    .split("\n")
    .map((l) => l.trim());
  const start = lines.indexOf("-----BEGIN SSH SIGNATURE-----");
  const end = lines.indexOf("-----END SSH SIGNATURE-----");
  if (start < 0 || end <= start) {
    throw new Error("invalid SSH SIGNATURE armor");
  }
  const b64 = lines.slice(start + 1, end).join("");
  if (!b64) throw new Error("empty SSH SIGNATURE armor");
  return Buffer.from(b64, "base64");
}

function parseSshSig(armor) {
  const raw = parseSshSigArmor(armor);
  const r = new Reader(raw);
  const magic = r.readBytes(6);
  if (!magic.equals(Buffer.from("SSHSIG", "ascii"))) {
    throw new Error("bad SSHSIG magic");
  }
  const version = r.readU32();
  if (version !== 1) {
    throw new Error(`unsupported SSHSIG version ${version}`);
  }
  const pubBlob = r.readString();
  const namespace = r.readString().toString("utf8");
  const reserved = r.readString();
  const hashAlg = r.readString().toString("utf8");
  const sigBlob = r.readString();
  if (r.remaining() !== 0) {
    throw new Error("trailing bytes in SSHSIG");
  }

  // Parse pubBlob: string keytype + string pubkey32
  const pr = new Reader(pubBlob);
  const pubType = pr.readString().toString("utf8");
  if (pubType !== "ssh-ed25519") {
    throw new Error(`unsupported SSH key type: ${pubType}`);
  }
  const pubKey = pr.readString();
  if (pubKey.length !== 32) {
    throw new Error("invalid Ed25519 public key length");
  }

  // Parse sigBlob: string sigtype + string sig64
  const sr = new Reader(sigBlob);
  const sigType = sr.readString().toString("utf8");
  if (sigType !== "ssh-ed25519") {
    throw new Error(`unsupported SSH sig type: ${sigType}`);
  }
  const sigBytes = sr.readString();
  if (sigBytes.length !== 64) {
    throw new Error("invalid Ed25519 signature length");
  }

  return { pubBlob, pubKey, namespace, reserved, hashAlg, sigBytes };
}

/**
 * Parse OpenSSH allowed_signers text into [{ principal, keyBlob }].
 * Supports optional options column before keytype if present, or standard:
 * `<principals> <keytype> <base64> [comment]`
 */
function parseAllowedSigners(text) {
  const entries = [];
  for (const rawLine of String(text).split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;
    const parts = line.split(/\s+/);
    if (parts.length < 3) continue;
    const principals = parts[0].split(",").map((s) => s.trim()).filter(Boolean);
    let idx = 1;
    // Skip optional options token if parts[idx] is not a key type
    if (!parts[idx].startsWith("ssh-") && !parts[idx].startsWith("ecdsa-") && !parts[idx].startsWith("sk-")) {
      idx += 1;
    }
    if (idx + 1 >= parts.length) continue;
    const keyType = parts[idx];
    const b64 = parts[idx + 1];
    if (keyType !== "ssh-ed25519") continue;
    try {
      const keyBlob = Buffer.from(b64, "base64");
      for (const p of principals) {
        entries.push({ principal: p, keyBlob });
      }
    } catch {
      // ignore malformed line
    }
  }
  return entries;
}

/**
 * Verify an OpenSSH SSHSIG over `payload` against `allowedSignersText`.
 * @param {Buffer|string} payload
 * @param {string} armor
 * @param {string} allowedSignersText
 * @param {string} [namespace="hashseal"]
 * @returns {{ status: "good"|"bad"|"untrusted_key", signer?: string, message?: string }}
 */
function verifySshSig(payload, armor, allowedSignersText, namespace = "hashseal") {
  let parsed;
  try {
    parsed = parseSshSig(armor);
  } catch (e) {
    return { status: "bad", message: e.message };
  }

  if (parsed.namespace !== namespace) {
    return {
      status: "bad",
      message: `namespace mismatch: got ${parsed.namespace}, want ${namespace}`,
    };
  }

  const signers = parseAllowedSigners(allowedSignersText);
  const matching = signers.filter((s) => s.keyBlob.equals(parsed.pubBlob));
  if (matching.length === 0) {
    return { status: "untrusted_key", message: "signer key not in allowed_signers" };
  }

  const dataBuf = Buffer.isBuffer(payload) ? payload : Buffer.from(payload, "utf8");
  let hashDigest;
  if (parsed.hashAlg === "sha512") {
    hashDigest = crypto.createHash("sha512").update(dataBuf).digest();
  } else if (parsed.hashAlg === "sha256") {
    hashDigest = crypto.createHash("sha256").update(dataBuf).digest();
  } else {
    return { status: "bad", message: `unsupported SSHSIG hash: ${parsed.hashAlg}` };
  }

  const signedData = Buffer.concat([
    Buffer.from("SSHSIG", "ascii"),
    sshString(parsed.namespace),
    sshString(parsed.reserved),
    sshString(parsed.hashAlg),
    sshString(hashDigest),
  ]);

  const spkiDer = Buffer.concat([ED25519_SPKI_PREFIX, parsed.pubKey]);
  const ok = crypto.verify(
    null,
    signedData,
    { key: spkiDer, format: "der", type: "spki" },
    parsed.sigBytes
  );
  if (!ok) {
    return { status: "bad", message: "Ed25519 SSHSIG verification failed" };
  }
  return { status: "good", signer: matching[0].principal };
}

module.exports = {
  verifySshSig,
  parseAllowedSigners,
  parseSshSig,
};
