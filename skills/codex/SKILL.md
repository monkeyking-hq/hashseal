---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Codex: sealed instruct files are authoritative; never
  overwrite human-signed seals; re-seal unsigned edits; name every verify failure path.
hashseal: "blake3:98ef6fa2b75c694afff4b2ee365f1c57455ad2ddd99417f5e1f71b06f8daf2f0"
---

# HashSeal — respect sealed instructions (Codex)

**Signed, Sealed, Delivered - I'm Yours.**

## Rules

1. Front matter `hashseal: "blake3:…"` → file is sealed; follow that body and front-matter policy.
2. **Never overwrite or re-seal a human-signed file (`hashseal_sig` present)** — escalate to the human operator.
3. **Never strip** `hashseal`, `hashseal_sig`, `hashseal_sig_format`, `hashseal_signer`, `hashseal_scope`, or `hashseal_key_id`.
4. Intentional **unsigned** instruct edit → `hashseal seal --instruct --root .`
5. Pre-ship → `hashseal check --root .` (checks `.hashseal.lock` and `SUSPICIOUS_UNICODE`; use `--diff origin/main --require-signature` on PRs).
6. List **every** non-OK path + digests on failure.

## Commands

```bash
hashseal init --git-hooks --lock
hashseal seal --instruct --root .
hashseal check --root .
hashseal-check --root .
```

`PATH` or `HASHSEAL_BIN` / `HASHSEAL_CHECK_BIN`.

```text
Copyright (c) 2026 MonkeyKing.dev
```
