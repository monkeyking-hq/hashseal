---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Claude: treat sealed instruct files as authoritative,
  never overwrite human-signed seals, re-seal unsigned edits, list every check failure.
hashseal: "blake3:8a38f930f2e5fd583cb3bc459f059e318692a654cf133fbe2b5db53be7f0648a"
---

# HashSeal — respect sealed instructions (Claude)

**Signed, Sealed, Delivered - I'm Yours.**

Apply when the project uses HashSeal (front-matter `hashseal`, `.hashseal.lock`, or `.hashseal.json`).

## Rules

1. `hashseal: "blake3:…"` means the instruct file is **sealed** — treat body and front-matter policy as authoritative.
2. **Never overwrite or re-seal a human-signed file (`hashseal_sig` present)**: escalate to the human operator to review and sign with their key.
3. **Never strip** `hashseal`, `hashseal_sig`, `hashseal_sig_format`, `hashseal_signer`, `hashseal_scope`, or `hashseal_key_id`.
4. After intentional edits to **unsigned** instruct files: `hashseal seal --instruct --root <project>`.
5. Before shipping: `hashseal check --root <project>` (honors `.hashseal.lock` and checks for `SUSPICIOUS_UNICODE`). Use `hashseal check --diff origin/main --require-signature` for PR gates.
6. On failure, surface **every** non-OK path with status and digests.

## Commands

```bash
hashseal init --claude-hooks --git-hooks --lock
hashseal seal --instruct --root .
hashseal check --root .
hashseal check --diff origin/main --require-signature
hashseal-check --root .
```

Resolve binaries via `PATH` or `HASHSEAL_BIN` / `HASHSEAL_CHECK_BIN`.

```text
Copyright (c) 2026 MonkeyKing.dev
```
