---
hashseal: "blake3:4bdda8cbf431cd37e570ddd0f58cbd7f05f1513697e4b7008f199ab4322fd2b4"
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

const result = checkDocumentText(markdownText);
// {
//   ok: boolean,
//   status: "valid" | "mismatch" | "missing_seal" | "invalid_format",
//   algorithm: "blake3" | null,
//   expected: "blake3:…" | null,
//   actual: "blake3:…" | null,
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

Instruct: digest check only (FULL canonical mode). GPG is not verified here.  
Tree: LF policy for text extensions; every non-OK path is listed in `findings`.

## Tests

```bash
npm test
# or:
node test/vectors.test.js        # instruct-v1 (16 cases)
node test/tree-vectors.test.js   # tree-v1
```

Uses frozen vectors at `../vectors/instruct-v1.json` and `../vectors/tree-v1.json`.

## Vendor

`vendor/noble/` is a vendored subset of [noble-hashes](https://github.com/paulmillr/noble-hashes) (MIT) for pure-JS blake3. No `node_modules` required.

```text
Copyright (c) 2026 MonkeyKing.dev
```
