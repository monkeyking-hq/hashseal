---
layout: default
title: Verify SDKs
permalink: /instruct/verify-sdks/
hashseal: "blake3:9464b41d16f3989794c372adda17013c67899492454efeec4497a8b2a7bbd627"
---

# Verify SDKs (Tier-1, zero-dep)

In-process `instruct-v2` digest, Unicode smuggling, and Ed25519 `SSHSIG` verification **without** shelling out to the CLI.

| Language | Path | Test command | Capabilities |
|---|---|---|---|
| Rust | `rust/hashseal-core` | `cargo test --workspace --release` | Full `instruct-v2`, `signing-v2` (SSH + GPG), Unicode, lockfiles, bundles |
| JavaScript | `verify/js` | `node verify/js/test/vectors.test.js && node verify/js/test/signing.test.js` | `instruct-v2`, Unicode, in-process Ed25519 `SSHSIG`, tree |
| Python | `verify/python` | `python3 verify/python/test/test_vectors.py && python3 verify/python/test/test_signing.py` | `instruct-v2`, Unicode, pure-Python Ed25519 `SSHSIG`, tree |
| Go | `verify/go` | `(cd verify/go && go test ./...)` | `instruct-v2`, Unicode, stdlib Ed25519 `SSHSIG`, tree |
| WASM | `rust/hashseal-wasm` | `cargo test -p hashseal-wasm --release` | `wasm-bindgen` (`check_text_json`, `version`) |

Shared frozen vectors:
- `verify/vectors/instruct-v2.json` (19 cases)
- `verify/vectors/signing-v2.json` (5 cases)
- `verify/vectors/tree-v1.json` (12 cases)

```text
Copyright (c) 2026 Nathan A. Chadwick
```
