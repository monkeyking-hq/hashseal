---
hashseal: "blake3:d5bb1e48c096a689ece45da98c2feba3f70daefcf03d4ac88ff96f2f6ffb2d8b"
---
# @hashseal/verify (JavaScript)

Zero-dependency HashSeal instruct document verifier.

**Signed, Sealed, Delivered - I'm Yours.**

## Install

Copy this folder or use from the monorepo. **No npm dependencies.**

```bash
node -e "console.log(require('./index.js').checkDocumentText(fs.readFileSync('AGENTS.md','utf8')))"
```

## API

```js
const {
  checkDocumentText,
  hashTreeFileContent,
  verifyTreeInMemory,
} = require("@hashseal/verify");
// or: require("./index.js")

const result = checkDocumentText(markdownText, {
  // Optional:
  // allowedSigners: fs.readFileSync(".hashseal/allowed_signers", "utf8"),
  // path: "AGENTS.md",
  // requireSignature: true,
  // unicodePolicy: "fail", // "fail" | "warn" | "off"
});
// {
//   ok: boolean,
//   status: "valid" | "mismatch" | "missing_seal" | "invalid_format" | "suspicious_unicode" | "missing_signature" | "bad_signature" | "untrusted_key" | "unverified_signature",
//   algorithm: "blake3" | null,
//   expected: "blake3:…" | null,
//   actual: "blake3:…" | null,
//   signer: string | null,
//   message: string | null
// }

// Tree (in-memory; same digests as hashseal-core tree policy)
const h = hashTreeFileContent("src/a.txt", "hello\n");
const tree = verifyTreeInMemory(
  { "src/a.txt": "hello\n" },
  [{ path: "src/a.txt", digest: h.digest, size: h.size }]
);
// { ok, checked, findings: [{ path, status, expected, actual }] }
```

Instruct: `instruct-v2` lossless raw-FM digest + Unicode smuggling check + in-process Ed25519 `SSHSIG` verification (`sshsig.js`).  
Tree: LF policy for text extensions; every non-OK path is listed in `findings`.

## Tests

```bash
npm test
# or:
node test/vectors.test.js        # instruct-v2 (19 cases)
node test/tree-vectors.test.js   # tree-v1 (12 cases)
node test/signing.test.js        # signing-v2 (5 SSHSIG cases)
```

Uses frozen vectors at `../vectors/instruct-v2.json`, `../vectors/signing-v2.json`, and `../vectors/tree-v1.json`.

## Vendor

`vendor/noble/` is a vendored subset of [noble-hashes](https://github.com/paulmillr/noble-hashes) (MIT) for pure-JS blake3. No `node_modules` required.

```text
Copyright (c) 2026 MonkeyKing.dev
```
