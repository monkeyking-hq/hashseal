---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Cursor: respect sealed AGENTS/rules files, never overwrite
  human-signed seals, re-seal unsigned edits, report full MISMATCH lists.
hashseal: "blake3:4f2563b552fbc9d855ae03889c256a39893e407ee2aa3d72ae11f0bea5920286"
---

# HashSeal — respect sealed instructions (Cursor)

**Signed, Sealed, Delivered - I'm Yours.**

Cursor often edits `AGENTS.md`, `.cursor/rules`, and similar instruct files. When HashSeal is present:

## Rules

1. Sealed instruct = authoritative (`hashseal: "blake3:…"` in YAML front matter).
2. **Never overwrite or re-seal a human-signed file (`hashseal_sig` present)**: ask the human operator to review and sign.
3. **Never strip** `hashseal`, `hashseal_sig`, `hashseal_sig_format`, `hashseal_signer`, `hashseal_scope`, or `hashseal_key_id`.
4. After intentional edits to **unsigned** instruct files, run `hashseal seal --instruct --root .`.
5. Run `hashseal check --root .` (checks `.hashseal.lock` and `SUSPICIOUS_UNICODE`) before claiming the workspace is clean.
6. Failures must list **every** bad path with status and expected/actual digests.

## Commands

```bash
hashseal init --git-hooks --lock
hashseal seal --instruct --root .
hashseal check --root .
hashseal check --diff origin/main --require-signature
hashseal-check --root .
```

```text
Copyright (c) 2026 MonkeyKing.dev
```
