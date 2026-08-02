---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Kilo: sealed instruct files are law; re-seal after edits;
  never strip seals; enumerate every check failure with digests.
hashseal: "blake3:cf86671ececdf527efab50f91bb32c3fefaa078a53f985972fd0ac7bc3c70387"
---

# HashSeal — respect sealed instructions (Kilo)

**Signed, Sealed, Delivered - I'm Yours.**

## Rules

1. `hashseal: "blake3:…"` marks a sealed instruct file — treat it as authoritative.
2. **Never strip** `hashseal`, `hashseal_sig`, or `hashseal_key_id`.
3. After intentional instruct edits: `hashseal seal --instruct --root .`
4. Before ship / merge claims: `hashseal check --root .` or `hashseal-check --root .`
5. On failure: list **every** non-OK path, status, and digests (no silent partial fails).
6. Product name **HashSeal**; MonkeyKing.dev only in copyright footers.

## Commands

```bash
hashseal seal --instruct --root .
hashseal seal --tree --release --root .
hashseal check --root .
hashseal-check --root .
hashseal verify --root .
```

Binaries: `PATH` or `HASHSEAL_BIN` / `HASHSEAL_CHECK_BIN`.

## Review / PR loops

When addressing review that touches sealed instruct files, re-seal and re-check so CI and local digests stay aligned.

```text
Copyright (c) 2026 MonkeyKing.dev
```