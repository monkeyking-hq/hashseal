---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Kilo: sealed instruct files are law; never overwrite
  human-signed seals; re-seal unsigned edits; enumerate every check failure.
hashseal: "blake3:46aa16e70b53fe83f6f08ee3878052fbbddb262d947decd5619dbb07249d1bda"
---

# HashSeal — respect sealed instructions (Kilo)

**Signed, Sealed, Delivered - I'm Yours.**

## Rules

1. `hashseal: "blake3:…"` marks a sealed instruct file — treat it as authoritative.
2. **Never overwrite or re-seal a human-signed file (`hashseal_sig` present)** — escalate to the human operator.
3. **Never strip** `hashseal`, `hashseal_sig`, `hashseal_sig_format`, `hashseal_signer`, `hashseal_scope`, or `hashseal_key_id`.
4. After intentional edits to **unsigned** instruct files: `hashseal seal --instruct --root .`
5. Before ship / merge claims: `hashseal check --root .` (checks `.hashseal.lock` and `SUSPICIOUS_UNICODE`; use `--diff origin/main --require-signature` on PRs).
6. On failure: list **every** non-OK path, status, and digests.

## Commands

```bash
hashseal init --git-hooks --lock
hashseal seal --instruct --root .
hashseal check --root .
hashseal-check --root .
```

```text
Copyright (c) 2026 MonkeyKing.dev
```
