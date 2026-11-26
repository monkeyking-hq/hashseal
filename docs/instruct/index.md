---
layout: default
title: Agent instruction file integrity seal
permalink: /instruct/
hashseal: "blake3:16851fd18462f90ce70358277e68deeee04c2aa88bed701c51d69748110ab7f3"
---

# Agent instruction file integrity seal (`instruct-v2`)

Seal agent instruction files and verify them before developers, CI, or agent harnesses pass them to models.

## What it protects

- **Prompt &amp; policy integrity (`instruct-v2`)** — lossless raw front-matter + body BLAKE3 digest protects YAML lists (`globs:`, `tools:`), nested maps, and Markdown body verbatim.
- **Cryptographic trust (`signing-v2`)** — OpenSSH `SSHSIG` (`.hashseal/allowed_signers`) or hardened GPG (`.hashseal/keys/`), bound to `scope` (`<rel-path>` or `*` via `--portable`) and failing closed (`UNVERIFIED_SIGNATURE`) if unverified.
- **Presence &amp; allowlist (`.hashseal.lock`)** — detects deleted (`LOCKED_FILE_REMOVED`) and unapproved (`UNAPPROVED_FILE`) instruction files.
- **Unicode smuggling defense** — detects hidden Unicode Tag, BiDi override, zero-width, and embedded BOM characters (`SUSPICIOUS_UNICODE`).

## Quick workflow

```bash
hashseal init --claude-hooks --git-hooks --lock
hashseal seal --instruct --sign --lock --root .
hashseal check --root . --require-signature
```

## Docs in this section

| Page | Topic |
|------|--------|
| [Seal format](format.md) | `instruct-v2` raw-FM framing, reserved fields, Unicode hygiene |
| [Signing](../signing.md) | `signing-v2` (SSH `allowed_signers` + hardened GPG), `scope`, `--portable` |
| [Verify SDKs](verify-sdks.md) | Tier-1 JS, Python, Go, WASM + frozen vectors |
| [CLI](../cli.md) | `init`, `seal`, `check`, `verify`, `unseal`, `clean` |

```text
Copyright (c) 2026 MonkeyKing.dev
```
