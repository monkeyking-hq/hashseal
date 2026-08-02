---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Codex: sealed instruct files are authoritative; re-seal
  after edits; never strip seals; name every verify failure path.
hashseal: "blake3:1fd46063c09b28c642b295a32e4fa5032f7f6f1024ebf19a57dcb39fea446c6a"
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
