/**
 * HashSeal instruct document check — instruct-v2 (lossless raw-FM + Unicode + SSHSIG).
 * Mirrors hashseal-core instruct.rs & unicode.rs. Zero npm dependencies.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const { blake3 } = require("./vendor/noble/blake3.js");

const SEAL_FIELD = "hashseal";
const SIG_FIELD = "hashseal_sig";
const SIG_FORMAT_FIELD = "hashseal_sig_format";
const SIGNER_FIELD = "hashseal_signer";
const SCOPE_FIELD = "hashseal_scope";
const KEY_ID_FIELD = "hashseal_key_id";

const INSTRUCT_V2_DOMAIN = "HASHSEAL-INSTRUCT-V2";

function isReservedKey(key, sealField) {
  return (
    key === sealField ||
    key === SEAL_FIELD ||
    key === SIG_FIELD ||
    key === SIG_FORMAT_FIELD ||
    key === SIGNER_FIELD ||
    key === SCOPE_FIELD ||
    key === KEY_ID_FIELD
  );
}

/**
 * Scan text for suspicious invisible or BiDi-control Unicode characters.
 * @param {string} text
 * @returns {Array<{ line: number, col: number, codepoint: number, kind: string }>}
 */
function scanSuspiciousUnicode(text) {
  const findings = [];
  let line = 1;
  let col = 1;
  let idx = 0;

  for (const ch of text) {
    const cp = ch.codePointAt(0);
    let kind = null;
    if (cp >= 0xe0000 && cp <= 0xe007f) {
      kind = "unicode_tag";
    } else if (
      (cp >= 0x202a && cp <= 0x202e) ||
      (cp >= 0x2066 && cp <= 0x2069) ||
      cp === 0x061c ||
      cp === 0x200e ||
      cp === 0x200f
    ) {
      kind = "bidi_control";
    } else if (cp === 0x200b || cp === 0x2060 || (cp >= 0x2061 && cp <= 0x2064)) {
      kind = "zero_width";
    } else if (cp === 0xfeff && idx > 0) {
      kind = "embedded_bom";
    }
    if (kind !== null) {
      findings.push({ line, col, codepoint: cp, kind });
    }
    if (ch === "\n") {
      line += 1;
      col = 1;
    } else {
      col += 1;
    }
    idx += 1;
  }
  return findings;
}

function formatUnicodeFindings(findings) {
  if (!findings || findings.length === 0) return "";
  const f = findings[0];
  const hex = f.codepoint.toString(16).toUpperCase().padStart(4, "0");
  const head = `U+${hex} (${f.kind}) at ${f.line}:${f.col}`;
  return findings.length > 1 ? `${head} (+${findings.length - 1} more)` : head;
}

/**
 * Check a sealed instruct markdown document (text in memory).
 * @param {string} text
 * @param {{
 *   field?: string,
 *   unicodePolicy?: "fail"|"warn"|"off",
 *   requireSignature?: boolean,
 *   digestOnly?: boolean,
 *   allowedSigners?: string,
 *   path?: string,
 * }} [opts]
 */
function checkDocumentText(text, opts) {
  opts = opts || {};
  const field = opts.field || SEAL_FIELD;
  const unicodePolicy = opts.unicodePolicy || "fail";
  const requireSignature = Boolean(opts.requireSignature);
  const digestOnly = Boolean(opts.digestOnly);

  if (unicodePolicy === "fail") {
    const u = scanSuspiciousUnicode(text);
    if (u.length > 0) {
      return {
        ok: false,
        status: "suspicious_unicode",
        algorithm: null,
        expected: null,
        actual: null,
        signer: null,
        message: formatUnicodeFindings(u),
      };
    }
  }

  const doc = parseDocument(text);
  if (!doc.hadFrontMatter) {
    const actual = computeDigest(doc, field);
    return {
      ok: false,
      status: "missing_seal",
      algorithm: "blake3",
      expected: null,
      actual: actual.qualified,
      signer: null,
      message: "missing hashseal field",
    };
  }
  const sealRaw = extractReservedField(doc.fmLines, field);
  if (sealRaw == null) {
    const actual = computeDigest(doc, field);
    return {
      ok: false,
      status: "missing_seal",
      algorithm: "blake3",
      expected: null,
      actual: actual.qualified,
      signer: null,
      message: "missing hashseal field",
    };
  }
  const expected = parseDigest(sealRaw);
  if (!expected) {
    return {
      ok: false,
      status: "invalid_format",
      algorithm: null,
      expected: null,
      actual: null,
      signer: null,
      message: `invalid digest format: ${sealRaw}`,
    };
  }
  if (expected.algorithm !== "blake3") {
    return {
      ok: false,
      status: "invalid_format",
      algorithm: expected.algorithm,
      expected: expected.qualified,
      actual: null,
      signer: null,
      message: `unsupported algorithm: ${expected.algorithm}`,
    };
  }
  const actual = computeDigest(doc, field);
  if (actual.hex !== expected.hex || actual.algorithm !== expected.algorithm) {
    return {
      ok: false,
      status: "mismatch",
      algorithm: expected.algorithm,
      expected: expected.qualified,
      actual: actual.qualified,
      signer: null,
      message: null,
    };
  }

  const sig = extractReservedField(doc.fmLines, SIG_FIELD);
  if (requireSignature && sig == null) {
    return {
      ok: false,
      status: "missing_signature",
      algorithm: actual.algorithm,
      expected: expected.qualified,
      actual: null,
      signer: null,
      message: "missing hashseal_sig",
    };
  }

  if (sig != null && !digestOnly && (opts.allowedSigners != null || requireSignature)) {
    const declaredScope = extractReservedField(doc.fmLines, SCOPE_FIELD);
    const expectedScope = opts.path || null;
    if (declaredScope != null && expectedScope != null && declaredScope !== "*" && declaredScope !== expectedScope) {
      return {
        ok: false,
        status: "bad_signature",
        algorithm: actual.algorithm,
        expected: expected.qualified,
        actual: null,
        signer: null,
        message: `signature scope mismatch: signed for '${declaredScope}', checked at '${expectedScope}'`,
      };
    }
    const verifyScope = declaredScope != null ? declaredScope : (expectedScope || "*");

    if (opts.allowedSigners == null) {
      return {
        ok: false,
        status: "unverified_signature",
        algorithm: actual.algorithm,
        expected: expected.qualified,
        actual: null,
        signer: null,
        message: "missing allowedSigners for signature verification",
      };
    }

    let sshsig;
    try {
      sshsig = require("./sshsig.js");
    } catch {
      return {
        ok: false,
        status: "unverified_signature",
        algorithm: actual.algorithm,
        expected: expected.qualified,
        actual: null,
        signer: null,
        message: "signature verifier unavailable in this runtime",
      };
    }

    const sigPayload = `HASHSEAL-SIG-V2\ndigest=${actual.qualified}\nscope=${verifyScope}\n`;
    const res = sshsig.verifySshSig(sigPayload, sig, opts.allowedSigners);
    if (res.status === "good") {
      return {
        ok: true,
        status: "valid",
        algorithm: actual.algorithm,
        expected: expected.qualified,
        actual: actual.qualified,
        signer: res.signer || null,
        message: null,
      };
    }
    return {
      ok: false,
      status: res.status === "untrusted_key" ? "untrusted_key" : "bad_signature",
      algorithm: actual.algorithm,
      expected: expected.qualified,
      actual: null,
      signer: null,
      message: res.message || "signature verification failed",
    };
  }

  return {
    ok: true,
    status: "valid",
    algorithm: actual.algorithm,
    expected: expected.qualified,
    actual: actual.qualified,
    signer: null,
    message: null,
  };
}

function blake3Hex(data) {
  const bytes = typeof data === "string" ? new TextEncoder().encode(data) : data;
  return Buffer.from(blake3(bytes)).toString("hex");
}

function blake3Digest(data) {
  const hex = blake3Hex(data);
  return { algorithm: "blake3", hex, qualified: `blake3:${hex}` };
}

function stripBom(s) {
  return s.charCodeAt(0) === 0xfeff ? s.slice(1) : s;
}

function normalizeLf(s) {
  return s.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function parseDocument(text) {
  text = stripBom(text);
  if (text.startsWith("---\n") || text.startsWith("---\r\n")) {
    const afterOpen = text.startsWith("---\r\n") ? text.slice(5) : text.slice(4);
    let search = afterOpen;
    let offset = 0;
    while (true) {
      const idx = search.indexOf("\n---");
      if (idx < 0) break;
      const after = search.slice(idx + 1);
      const rest = after.slice(3);
      const closed =
        rest.length === 0 ||
        rest.startsWith("\n") ||
        rest.startsWith("\r\n") ||
        rest.startsWith("\r");
      if (closed) {
        const fmBlock = afterOpen.slice(0, offset + idx);
        const bodyStart = offset + idx + 1 + 3;
        let body = afterOpen.slice(bodyStart);
        if (body.startsWith("\r\n")) body = body.slice(2);
        else if (body.startsWith("\n")) body = body.slice(1);
        else if (body.startsWith("\r")) body = body.slice(1);
        const fmLines = normalizeLf(fmBlock).split("\n");
        return {
          fmLines,
          hadFrontMatter: true,
          bodyRaw: body,
        };
      }
      offset += idx + 1;
      search = search.slice(idx + 1);
    }
  }
  return { fmLines: [], hadFrontMatter: false, bodyRaw: text };
}

function extractReservedField(lines, field) {
  let i = 0;
  while (i < lines.length) {
    if (!lines[i].startsWith(" ") && !lines[i].startsWith("\t")) {
      const trimmed = lines[i].trim();
      const colon = trimmed.indexOf(":");
      if (colon >= 0) {
        const k = trimmed.slice(0, colon).trim();
        if (k === field) {
          const rest = trimmed.slice(colon + 1).trim();
          if (rest === "|" || rest === ">" || rest === "|-" || rest === ">-") {
            let val = "";
            i += 1;
            while (i < lines.length) {
              const L = lines[i];
              const empty = L.trim() === "";
              const indented = L.startsWith(" ") || L.startsWith("\t");
              if (
                indented ||
                (empty &&
                  i + 1 < lines.length &&
                  (lines[i + 1].startsWith(" ") || lines[i + 1].startsWith("\t")))
              ) {
                if (empty) {
                  val += "\n";
                  i += 1;
                  continue;
                }
                if (val !== "") val += "\n";
                val += L.trimStart();
                i += 1;
                continue;
              }
              break;
            }
            return val;
          }
          return rest.replace(/^"|"$/g, "");
        }
      }
    }
    i += 1;
  }
  return null;
}

function stripReservedFmLines(lines, sealField) {
  const out = [];
  let i = 0;
  while (i < lines.length) {
    const trimmed = lines[i].trim();
    const colon = trimmed.indexOf(":");
    if (
      colon >= 0 &&
      !lines[i].startsWith(" ") &&
      !lines[i].startsWith("\t") &&
      isReservedKey(trimmed.slice(0, colon).trim(), sealField)
    ) {
      i += 1;
      while (i < lines.length) {
        const L = lines[i];
        if (L.startsWith(" ") || L.startsWith("\t")) {
          i += 1;
          continue;
        }
        if (
          L.trim() === "" &&
          i + 1 < lines.length &&
          (lines[i + 1].startsWith(" ") || lines[i + 1].startsWith("\t"))
        ) {
          i += 1;
          continue;
        }
        break;
      }
      continue;
    }
    out.push(lines[i]);
    i += 1;
  }
  return out;
}

function hashPayload(doc, sealField = SEAL_FIELD) {
  const bodyLf = normalizeLf(doc.bodyRaw);
  const kept = stripReservedFmLines(doc.fmLines, sealField);
  const hasNonEmpty = kept.some((l) => l.trim() !== "");
  const payload = hasNonEmpty
    ? `${INSTRUCT_V2_DOMAIN}\nFM\n${kept.join("\n")}\nBODY\n${bodyLf}`
    : `${INSTRUCT_V2_DOMAIN}\nBODY\n${bodyLf}`;
  return new TextEncoder().encode(payload);
}

function computeDigest(doc, sealField = SEAL_FIELD) {
  return blake3Digest(hashPayload(doc, sealField));
}

function parseDigest(raw) {
  const s = String(raw).trim().replace(/^"|"$/g, "");
  const idx = s.indexOf(":");
  if (idx < 0) return null;
  const algorithm = s.slice(0, idx).toLowerCase();
  const hex = s.slice(idx + 1).trim().toLowerCase();
  if (!hex || !/^[0-9a-f]+$/.test(hex)) return null;
  return { algorithm, hex, qualified: `${algorithm}:${hex}` };
}

module.exports = {
  checkDocumentText,
  scanSuspiciousUnicode,
  blake3Hex,
  blake3Digest,
  SEAL_FIELD,
  _internal: {
    parseDocument,
    hashPayload,
    computeDigest,
    normalizeLf,
  },
};
