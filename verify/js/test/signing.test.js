/**
 * Run official signing-v2 vectors against checkDocumentText.
 * Usage: node test/signing.test.js
 * Zero npm deps.
 */
"use strict";

const fs = require("fs");
const path = require("path");
const { checkDocumentText } = require("../index.js");

const vecPath = path.join(__dirname, "..", "..", "vectors", "signing-v2.json");
const doc = JSON.parse(fs.readFileSync(vecPath, "utf8"));
if (doc.spec !== "signing-v2") {
  console.error(`unexpected spec ${doc.spec}`);
  process.exit(1);
}

let passed = 0;
let failed = 0;

for (const c of doc.cases) {
  try {
    const r = checkDocumentText(c.text, {
      allowedSigners: c.allowed_signers,
      path: c.path,
      requireSignature: c.require_signature,
    });
    if (r.ok !== c.expect.ok) {
      throw new Error(`ok=${r.ok} want ${c.expect.ok} (status=${r.status}, msg=${r.message})`);
    }
    if (r.status !== c.expect.status) {
      throw new Error(`status=${r.status} want ${c.expect.status} (msg=${r.message})`);
    }
    if (c.expect.signer && r.signer !== c.expect.signer) {
      throw new Error(`signer=${r.signer} want ${c.expect.signer}`);
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
