---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Cursor: respect sealed AGENTS/rules files, re-seal after
  edits, never strip hashseal fields, report full MISMATCH lists.
hashseal: "blake3:f6d47102746bd2209895d57d5781446064774ea1f5130a135762e61d4e7a1ba7"
---

# HashSeal — respect sealed instructions (Cursor)

**Signed, Sealed, Delivered - I'm Yours.**

Cursor often edits `AGENTS.md`, `.cursor/rules`, and similar instruct files. When HashSeal is present:

## Rules

1. Sealed instruct = authoritative (`hashseal: "blake3:…"` in YAML front matter).
2. **Never strip** `hashseal`, `hashseal_sig`, `hashseal_key_id` from front matter.
3. After you change sealed instruct content, run seal before ending the turn when CLI is available.
4. Run check before claiming the workspace is integrity-clean.
5. Failures must list **every** bad path with status and expected/actual digests.
6. Product: **HashSeal** (MonkeyKing.dev only in copyright footers).

## Commands

```bash
hashseal seal --instruct --root .
hashseal seal --tree --release --root .
hashseal check --root .
hashseal-check --root .
hashseal verify --root .
```

Prefer project scripts or `HASHSEAL_BIN` when the binary is not on `PATH`.

## Cursor-specific notes

- Do not “helpfully” delete HashSeal front matter when reformatting Markdown.
- If a rule file is sealed, re-seal after rule edits so agents keep matching digests.

```text
Copyright (c) 2026 MonkeyKing.dev
```
