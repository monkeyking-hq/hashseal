---
hashseal: "blake3:75b4e17952ab3a55bba92f720ddd7588c22c547bb76364d3193e7cf10369a90c"
---
# @hashseal/npm-plugin

Thin **npm** wrapper that shells out to the **`hashseal` CLI**.

**Signed, Sealed, Delivered - I'm Yours.**

## PATH requirement

This package does **not** embed native binaries. You must install the HashSeal CLI and ensure it is on **`PATH`**, or set:

```bash
# absolute path to the hashseal binary
export HASHSEAL_BIN=/path/to/hashseal   # Unix
set HASHSEAL_BIN=C:\path\to\hashseal.exe  # Windows cmd
$env:HASHSEAL_BIN = "C:\path\to\hashseal.exe"  # PowerShell
```

Build from this monorepo:

```bash
cargo build -p hashseal --release
# then add target/release to PATH, or point HASHSEAL_BIN at target/release/hashseal[.exe]
```

## Usage (from monorepo, no publish)

```bash
cd plugins/npm
node bin/hashseal-npm.js seal --instruct --root ../../fixtures/mvp-demo
node bin/hashseal-npm.js check --root ../../fixtures/mvp-demo
```

Programmatic:

```js
const { check, runHashseal } = require("@hashseal/npm-plugin");
// or: require("./index.js")

const r = check({ root: "fixtures/mvp-demo" });
if (r.status !== 0) {
  process.stderr.write(r.stderr);
  process.exit(r.status || 1);
}
```

## package.json scripts

```json
{
  "scripts": {
    "hashseal:seal": "hashseal-npm seal --instruct --root .",
    "hashseal:check": "hashseal-npm check --root ."
  }
}
```

## Notes

- **Zero npm runtime dependencies.**
- Not published to the npm registry from overnight/agent builds unless you request it.
- Verify UX (naming every bad file) comes from the CLI / core — this wrapper does not swallow output when using `stdio: 'inherit'`.

```text
Copyright (c) 2026 MonkeyKing.dev
```
