---
name: hashseal-respect-seals
description: >-
  When working in a HashSeal-enabled repo, treat sealed instruct files as
  authoritative, never overwrite human-signed seals, re-seal unsigned edits,
  and surface every check failure path with digests.
hashseal: "blake3:e540f170b6b5638ff3be5c8c4e5b23145d875525d0844bdb87eada5c77f4086a"
---

# HashSeal — respect sealed instructions (Grok)

**Signed, Sealed, Delivered - I'm Yours.**

Use this skill whenever the workspace has HashSeal seals (`hashseal` in instruct front matter, `.hashseal.lock`, or `.hashseal.json`).

## Hard rules

1. Instruction files with YAML front matter field `hashseal: "blake3:…"` are **sealed**.
2. **Never overwrite or re-seal a human-signed file (`hashseal_sig` present)** — escalate to the human operator to review and sign.
3. **Never strip** reserved fields: `hashseal`, `hashseal_sig`, `hashseal_sig_format`, `hashseal_signer`, `hashseal_scope`, `hashseal_key_id`.
4. After intentional edits to **unsigned** instruct files: re-seal (`hashseal seal --instruct --root .`).
5. Before shipping: run `hashseal check --root .` (checks `.hashseal.lock` and `SUSPICIOUS_UNICODE`); surface **every** non-OK path.

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
