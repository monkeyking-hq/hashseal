/**
 * Run official instruct-v1 vectors against checkDocumentText.
 * Usage: node test/vectors.test.js
 * Zero npm deps.
 */

"use strict";

const fs = require("fs");
const path = require("path");
const { checkDocumentText, blake3Digest } = require("../index.js");

const vectorsPath = path.join(
  __dirname,
  "..",
  "..",
  "vectors",
  "instruct-v1.json"
);

const doc = JSON.parse(fs.readFileSync(vectorsPath, "utf8"));
if (doc.spec !== "instruct-v1") {
  fail(`unexpected spec ${doc.spec}`);
}

let passed = 0;
let failed = 0;

for (const c of doc.cases) {
  try {
    if (c.kind === "raw_digest") {
      const actual = blake3Digest(c.bytes_utf8).qualified;
      assertEq(actual, c.expect.digest, `${c.id} digest`);
    } else if (c.kind === "check") {
      const r = checkDocumentText(c.text);
      assertEq(r.ok, c.expect.ok, `${c.id} ok`);
      assertEq(r.status, c.expect.status, `${c.id} status`);
      if (c.expect.digest != null) {
        assertEq(r.actual, c.expect.digest, `${c.id} actual digest`);
        if (r.ok) {
          assertEq(r.expected, c.expect.digest, `${c.id} expected digest`);
        }
      }
      if (c.expect.expected != null) {
        assertEq(r.expected, c.expect.expected, `${c.id} expected`);
      }
      if (c.expect.actual != null) {
        assertEq(r.actual, c.expect.actual, `${c.id} actual`);
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
