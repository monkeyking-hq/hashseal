/**
 * HashSeal instruct document check — FULL canonical mode (digest only).
 * Mirrors hashseal-core instruct algorithm. Zero npm dependencies.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const { blake3 } = require("./vendor/noble/blake3.js");

const SEAL_FIELD = "hashseal";
const SIG_FIELD = "hashseal_sig";
const KEY_ID_FIELD = "hashseal_key_id";

const RESERVED = new Set([SEAL_FIELD, SIG_FIELD, KEY_ID_FIELD]);

/**
 * @typedef {Object} CheckResult
 * @property {boolean} ok
 * @property {"valid"|"mismatch"|"missing_seal"|"invalid_format"} status
 * @property {string|null} algorithm
 * @property {string|null} expected  qualified digest e.g. blake3:hex
 * @property {string|null} actual
 * @property {string|null} message
 */

/**
 * Check a sealed instruct markdown document (text in memory).
 * @param {string} text
 * @param {{ field?: string }} [opts]
 * @returns {CheckResult}
 */
function checkDocumentText(text, opts) {
  const field = (opts && opts.field) || SEAL_FIELD;
  const doc = parseDocument(text);
  if (!doc.hadFrontMatter) {
    const actual = computeDigest(doc);
    return {
      ok: false,
      status: "missing_seal",
      algorithm: "blake3",
      expected: null,
      actual: actual.qualified,
      message: "missing hashseal field",
    };
  }
  const sealRaw = extractReservedField(doc.fmLines, field);
  if (sealRaw == null) {
    const actual = computeDigest(doc);
    return {
      ok: false,
      status: "missing_seal",
      algorithm: "blake3",
      expected: null,
      actual: actual.qualified,
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
      message: `invalid digest format: ${sealRaw}`,
    };
  }
  // Use algorithm from seal for hashing (blake3 only implemented here)
  if (expected.algorithm !== "blake3") {
    return {
      ok: false,
      status: "invalid_format",
      algorithm: expected.algorithm,
      expected: expected.qualified,
      actual: null,
      message: `unsupported algorithm: ${expected.algorithm}`,
    };
  }
  const actual = computeDigest(doc);
  if (actual.hex !== expected.hex || actual.algorithm !== expected.algorithm) {
    return {
      ok: false,
      status: "mismatch",
      algorithm: expected.algorithm,
      expected: expected.qualified,
      actual: actual.qualified,
      message: null,
    };
  }
  return {
    ok: true,
    status: "valid",
    algorithm: actual.algorithm,
    expected: expected.qualified,
    actual: actual.qualified,
    message: null,
  };
}

/**
 * Blake3 hex digest of UTF-8 bytes (no algorithm prefix).
 * @param {string|Uint8Array} data
 * @returns {string} lowercase hex
 */
function blake3Hex(data) {
  const bytes =
    typeof data === "string" ? new TextEncoder().encode(data) : data;
  return Buffer.from(blake3(bytes)).toString("hex");
}

/**
 * @param {string|Uint8Array} data
 * @returns {{ algorithm: string, hex: string, qualified: string }}
 */
function blake3Digest(data) {
  const hex = blake3Hex(data);
  return { algorithm: "blake3", hex, qualified: `blake3:${hex}` };
}

// --- parse / canonical (mirrors hashseal-core instruct.rs) ---

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
        let bodyStart = idx + 1 + 3;
        let body = afterOpen.slice(bodyStart);
        if (body.startsWith("\r\n")) body = body.slice(2);
        else if (body.startsWith("\n")) body = body.slice(1);
        else if (body.startsWith("\r")) body = body.slice(1);
        const fmLines = normalizeLf(fmBlock).split("\n");
        // split("\n") on trailing empty keeps last empty; LF block without trailing \n is fine
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

function isReservedKey(key) {
  return RESERVED.has(key);
}

function forEachFmEntry(lines, f) {
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const trimmed = line.trim();
    if (trimmed === "" || trimmed.startsWith("#")) {
      i += 1;
      continue;
    }
    if (line.startsWith(" ") || line.startsWith("\t")) {
      i += 1;
      continue;
    }
    const colon = trimmed.indexOf(":");
    if (colon >= 0) {
      const key = trimmed.slice(0, colon).trim();
      const rest = trimmed.slice(colon + 1).trim();
      if (isReservedKey(key)) {
        i += 1;
        while (i < lines.length) {
          const L = lines[i];
          if (L.startsWith(" ") || L.startsWith("\t")) {
            i += 1;
            continue;
          }
          if (L.trim() === "") {
            if (
              i + 1 < lines.length &&
              (lines[i + 1].startsWith(" ") || lines[i + 1].startsWith("\t"))
            ) {
              i += 1;
              continue;
            }
            break;
          }
          break;
        }
        continue;
      }
      if (rest === "|" || rest === ">" || rest === "|-" || rest === ">-") {
        let val = "";
        i += 1;
        while (
          i < lines.length &&
          (lines[i].startsWith(" ") || lines[i].startsWith("\t"))
        ) {
          if (val !== "") val += "\n";
          val += lines[i].trimStart();
          i += 1;
        }
        f(key, val);
        continue;
      }
      const val = rest.replace(/^"|"$/g, "");
      f(key, val);
    }
    i += 1;
  }
}

function fmMap(lines) {
  /** @type {Map<string, string>} */
  const map = new Map();
  forEachFmEntry(lines, (k, v) => {
    map.set(k, v);
  });
  return map;
}

function extractReservedField(lines, field) {
  let i = 0;
  while (i < lines.length) {
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
    i += 1;
  }
  return null;
}

function canonicalFmString(map) {
  const keys = Array.from(map.keys()).sort();
  let s = "";
  for (const k of keys) {
    const v = map.get(k);
    s += k;
    s += ": ";
    if (v === "" || v.includes(":") || v.includes("#") || v.includes(" ")) {
      s += '"';
      s += v.replace(/"/g, '\\"');
      s += '"';
    } else {
      s += v;
    }
    s += "\n";
  }
  return s;
}

function hashPayload(doc) {
  const bodyLf = normalizeLf(doc.bodyRaw);
  const map = fmMap(doc.fmLines);
  if (map.size === 0) {
    return new TextEncoder().encode(bodyLf);
  }
  let payload = canonicalFmString(map);
  payload += "\n";
  payload += bodyLf;
  return new TextEncoder().encode(payload);
}

function computeDigest(doc) {
  return blake3Digest(hashPayload(doc));
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
  blake3Hex,
  blake3Digest,
  SEAL_FIELD,
  // exported for unit tests / debugging
  _internal: {
    parseDocument,
    hashPayload,
    computeDigest,
    normalizeLf,
  },
};
