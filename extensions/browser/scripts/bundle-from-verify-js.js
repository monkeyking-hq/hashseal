/**
 * Bundle verify/js (CommonJS + noble blake3) into a single browser IIFE.
 * Run from monorepo root:
 *   node extensions/browser/scripts/bundle-from-verify-js.js
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "../../..");
const jsRoot = path.join(repoRoot, "verify", "js");
const dest = path.join(
  repoRoot,
  "extensions",
  "browser",
  "lib",
  "hashseal-verify.browser.js"
);

const files = [
  "vendor/noble/crypto.js",
  "vendor/noble/_assert.js",
  "vendor/noble/_u64.js",
  "vendor/noble/utils.js",
  "vendor/noble/_blake.js",
  "vendor/noble/blake2s.js",
  "vendor/noble/blake3.js",
  "check.js",
  "tree.js",
];

function normalize(from, id) {
  if (!id.startsWith(".")) return id;
  const base = path.posix.dirname(from.replace(/\\/g, "/"));
  let p = path.posix.normalize(base + "/" + id);
  if (!p.endsWith(".js")) p += ".js";
  return p.replace(/^\.\//, "");
}

const out = [];
out.push(
  "/* Auto-generated from verify/js — do not edit by hand.",
  " * Run: node extensions/browser/scripts/bundle-from-verify-js.js",
  " * Copyright (c) 2026 MonkeyKing.dev",
  " */"
);
out.push("(function (global) {");
out.push('  "use strict";');
out.push("  // Minimal Buffer polyfill for hex (browser)");
out.push('  if (typeof Buffer === "undefined") {');
out.push("    global.Buffer = {");
out.push("      from: function (u8) {");
out.push("        return {");
out.push("          toString: function (enc) {");
out.push(
  '            if (enc !== "hex") throw new Error("Buffer polyfill only supports hex");'
);
out.push('            var hex = "";');
out.push(
  "            for (var i = 0; i < u8.length; i++) hex += u8[i].toString(16).padStart(2, \"0\");"
);
out.push("            return hex;");
out.push("          }");
out.push("        };");
out.push("      }");
out.push("    };");
out.push("  }");
out.push("  var modules = Object.create(null);");
out.push("  function require(id) {");
out.push("    var m = modules[id];");
out.push(
  '    if (!m) throw new Error("hashseal-verify browser bundle: missing module " + id);'
);
out.push("    return m.exports;");
out.push("  }");
out.push("  function define(id, factory) {");
out.push("    var module = { exports: {} };");
out.push("    modules[id] = module;");
out.push("    factory(require, module, module.exports);");
out.push("  }");

for (const f of files) {
  const abs = path.join(jsRoot, f);
  let src = fs.readFileSync(abs, "utf8");
  src = src.replace(/require\(["'](\.\.?\/[^"']+)["']\)/g, (_m, rel) => {
    const n = normalize(f, rel);
    return "require(" + JSON.stringify(n) + ")";
  });
  const id = f.replace(/\\/g, "/");
  out.push(
    "  define(" + JSON.stringify(id) + ", function (require, module, exports) {"
  );
  out.push(src);
  out.push("  });");
}

out.push('  var check = require("check.js");');
out.push('  var tree = require("tree.js");');
out.push("  var api = {");
out.push("    checkDocumentText: check.checkDocumentText,");
out.push("    blake3Hex: check.blake3Hex,");
out.push("    blake3Digest: check.blake3Digest,");
out.push("    SEAL_FIELD: check.SEAL_FIELD,");
out.push("    hashTreeFileContent: tree.hashTreeFileContent,");
out.push("    verifyTreeInMemory: tree.verifyTreeInMemory");
out.push("  };");
out.push(
  '  if (typeof module !== "undefined" && module.exports) module.exports = api;'
);
out.push("  global.HashsealVerify = api;");
out.push(
  "})(typeof globalThis !== \"undefined\" ? globalThis : typeof window !== \"undefined\" ? window : this);"
);
out.push("");

fs.mkdirSync(path.dirname(dest), { recursive: true });
fs.writeFileSync(dest, out.join("\n"));
console.log("wrote", path.relative(repoRoot, dest), "bytes", fs.statSync(dest).size);

// Smoke test (Node)
delete require.cache[require.resolve(dest)];
const api = require(dest);
const sample =
  '---\nhashseal: "blake3:25280e93176b8b5ae3f4c2dd4b8fef7a20c4a626ea8dfd933b0e77b3a240dccb"\n---\n# Hello\n\nAgent rules.\n';
const r = api.checkDocumentText(sample);
if (!r.ok || r.status !== "valid") {
  console.error("smoke failed", r);
  process.exit(1);
}
console.log("smoke ok:", r.status, r.actual);
