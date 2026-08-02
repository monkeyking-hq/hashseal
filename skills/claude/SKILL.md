---
name: hashseal-respect-seals
description: >-
  HashSeal skill for Claude: treat sealed instruct files as authoritative,
  re-seal after edits, never strip hashseal fields, list every check failure.
hashseal: "blake3:e486bd0869e8cbb2c7b36e2ae3acf8b0e6d70f400d5f8873552e4931d2674e05"
---

# HashSeal — respect sealed instructions (Claude)

**Signed, Sealed, Delivered - I'm Yours.**

Apply when the project uses HashSeal (front-matter `hashseal`, `hashseal-bundle/`, or `.hashseal.json`).

## Rules

1. `hashseal: "blake3:…"` means the instruct file is **sealed** — treat body as authoritative.
2. **Never strip** `hashseal`, `hashseal_sig`, or `hashseal_key_id`.
3. After intentional edits to sealed instruct files: `hashseal seal --instruct --root <project>`.
4. Before shipping: `hashseal check --root <project>` or `hashseal-check --root <project>`.
5. On failure, surface **every** non-OK path with status and digests.
6. Product name is **HashSeal** (MonkeyKing.dev only in copyright footers).

## Commands

```bash
hashseal seal --instruct --root .
hashseal seal --tree --release --root .
hashseal check --root .
hashseal-check --root .
hashseal verify --root .
```

Resolve binaries via `PATH` or `HASHSEAL_BIN` / `HASHSEAL_CHECK_BIN`.

## Do not

- Silently rewrite sealed bodies without re-seal.
- Remove seal fields to “clean up” YAML.
- Summarize failures as a single exit code without naming paths.

```text
Copyright (c) 2026 MonkeyKing.dev
```
