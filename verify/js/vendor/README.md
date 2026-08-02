# Vendored dependencies

## noble/

Subset of **@noble/hashes** v1.5.0 (MIT License, Paul Miller) providing pure-JS BLAKE3.

- Source: https://github.com/paulmillr/noble-hashes
- Only blake3 + its local graph is included.
- `utils.js` is patched to `require("./crypto.js")` instead of `@noble/hashes/crypto` so this tree runs without npm install.

Do not add npm dependencies under `verify/js/`.

```text
Copyright (c) 2026 MonkeyKing.dev (this README only)
```
