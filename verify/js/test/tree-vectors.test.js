/**
 * Run official tree-v1 vectors against in-memory tree verify.
 * Usage: node test/tree-vectors.test.js
 * Zero npm deps.
 */

"use strict";

const fs = require("fs");
const path = require("path");
const { hashTreeFileContent, verifyTreeInMemory } = require("../tree.js");

const vectorsPath = path.join(__dirname, "..", "..", "vectors", "tree-v1.json");

const doc = JSON.parse(fs.readFileSync(vectorsPath, "utf8"));
if (doc.spec !== "tree-v1") {
  fail(`unexpected spec ${doc.spec}`);
}

const lfText = doc.line_endings_lf_text !== false;
const textExtensions = doc.text_extensions || undefined;
const hashOpts = { lineEndingsLfText: lfText, textExtensions };

let passed = 0;
let failed = 0;

for (const c of doc.cases) {
  try {
    if (c.kind === "raw_file_digest") {
      const r = hashTreeFileContent(c.path, c.content, hashOpts);
      assertEq(r.digest, c.expect.digest, `${c.id} digest`);
      assertEq(r.size, c.expect.size, `${c.id} size`);
    } else if (c.kind === "verify_tree") {
      const r = verifyTreeInMemory(c.files || {}, c.ledger_entries || [], hashOpts);
      assertEq(r.ok, c.expect.ok, `${c.id} ok`);
      assertEq(r.checked, c.expect.checked, `${c.id} checked`);
      const want = c.expect.findings || [];
      assertEq(r.findings.length, want.length, `${c.id} findings.length`);
      for (let i = 0; i < want.length; i++) {
        const g = r.findings[i];
        const w = want[i];
        assertEq(g.path, w.path, `${c.id} finding[${i}].path`);
        assertEq(g.status, w.status, `${c.id} finding[${i}].status`);
        assertEq(g.expected, w.expected, `${c.id} finding[${i}].expected`);
        assertEq(g.actual, w.actual, `${c.id} finding[${i}].actual`);
      }
    } else {
      throw new Error(`unknown kind ${c.kind}`);
    }
    passed += 1;
    console.log(`ok  ${c.id}`);
  } catch (e) {
    failed += 1;
    console.error(`FAIL ${c.id}: ${e.message}`);
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

function assertEq(a, b, label) {
  if (a !== b) {
    throw new Error(`${label}: got ${JSON.stringify(a)} want ${JSON.stringify(b)}`);
  }
}

function fail(msg) {
  console.error(msg);
  process.exit(1);
}
