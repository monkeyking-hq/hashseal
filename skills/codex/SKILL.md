---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Codex: sealed instruct files are authoritative; re-seal
  after edits; never strip seals; name every verify failure path.
hashseal: "blake3:e4b84b49fb68b9d2f64d294abfcf3081d6379a75256e705454ada6ec41c6665d"
---

# HashSeal — respect sealed instructions (Codex)

**Signed, Sealed, Delivered - I'm Yours.**

## Rules

1. Front matter `hashseal: "blake3:…"` → file is sealed; follow that body.
2. **Never strip** `hashseal`, `hashseal_sig`, `hashseal_key_id`.
3. Intentional instruct edit → `hashseal seal --instruct --root .`
4. Pre-ship → `hashseal check --root .` (or `hashseal-check`).
5. List **every** non-OK path + digests on failure.
6. Brand: **HashSeal**; MonkeyKing.dev only in copyright footers.

## Commands

```bash
hashseal seal --instruct --root .
hashseal seal --tree --release --root .
hashseal check --root .
hashseal-check --root .
hashseal verify --root .
```

`PATH` or `HASHSEAL_BIN` / `HASHSEAL_CHECK_BIN`.

```text
Copyright (c) 2026 MonkeyKing.dev
```
