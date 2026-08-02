/**
 * In-memory tree verify — mirrors hashseal-core tree hash + verify policy.
 * Zero npm dependencies. Used for multi-lang tree-v1 vectors without a filesystem walk.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const { blake3Digest } = require("./check.js");

const DEFAULT_TEXT_EXTENSIONS = new Set([
  "md",
  "txt",
  "toml",
  "yml",
  "yaml",
  "json",
  "rs",
  "java",
  "go",
  "py",
  "js",
  "ts",
  "tsx",
  "jsx",
  "css",
  "html",
  "xml",
  "sh",
  "ps1",
  "c",
  "h",
  "cpp",
  "cs",
  "rb",
  "svg",
]);

function normalizeLf(s) {
  return s.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function extOf(path) {
  const i = path.lastIndexOf(".");
  if (i < 0) return "";
  return path.slice(i + 1).toLowerCase();
}

/**
 * Hash one path+content with core tree policy.
 * @param {string} path posix relative path
 * @param {string} content file body (UTF-8 string; binary as latin1 if needed)
 * @param {{ lineEndingsLfText?: boolean, textExtensions?: Set<string>|string[] }} [opts]
 * @returns {{ digest: string, size: number, qualified: string, hex: string }}
 */
function hashTreeFileContent(path, content, opts) {
  const lfText = !opts || opts.lineEndingsLfText !== false;
  const textExts =
    opts && opts.textExtensions
      ? opts.textExtensions instanceof Set
        ? opts.textExtensions
        : new Set(opts.textExtensions)
      : DEFAULT_TEXT_EXTENSIONS;
  const size =
    typeof Buffer !== "undefined"
      ? Buffer.byteLength(content, "utf8")
      : new TextEncoder().encode(content).length;
  let data = content;
  if (lfText && textExts.has(extOf(path))) {
    data = normalizeLf(content.replace(/^\uFEFF/, ""));
  }
  const d = blake3Digest(data);
  return {
    digest: d.qualified,
    qualified: d.qualified,
    hex: d.hex,
    size,
  };
}

/**
 * Verify in-memory files against ledger entries (same findings as hashseal-core verify_tree).
 * @param {Record<string, string>} files path → content
 * @param {Array<{ path: string, digest: string, size?: number }>} ledgerEntries
 * @param {{ lineEndingsLfText?: boolean, textExtensions?: Set<string>|string[] }} [opts]
 * @returns {{ ok: boolean, checked: number, findings: Array<{path:string,status:string,expected:string|null,actual:string|null}> }}
 */
function verifyTreeInMemory(files, ledgerEntries, opts) {
  const current = new Map();
  const paths = Object.keys(files || {}).sort();
  for (const p of paths) {
    const h = hashTreeFileContent(p, files[p], opts);
    current.set(p, h.qualified);
  }

  const findings = [];
  const expectedPaths = new Set();
  const entries = ledgerEntries || [];

  for (const e of entries) {
    expectedPaths.add(e.path);
    const actual = current.get(e.path);
    if (actual === undefined) {
      findings.push({
        path: e.path,
        status: "removed",
        expected: e.digest,
        actual: null,
      });
    } else if (actual !== e.digest) {
      findings.push({
        path: e.path,
        status: "mismatch",
        expected: e.digest,
        actual,
      });
    }
  }

  for (const [path, digest] of current) {
    if (!expectedPaths.has(path)) {
      findings.push({
        path,
        status: "added",
        expected: null,
        actual: digest,
      });
    }
  }

  findings.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  return {
    ok: findings.length === 0,
    checked: entries.length,
    findings,
  };
}

module.exports = {
  hashTreeFileContent,
  verifyTreeInMemory,
  DEFAULT_TEXT_EXTENSIONS,
  normalizeLf,
};
