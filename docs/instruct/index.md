---
layout: default
title: Agent instruction file integrity seal
permalink: /instruct/
hashseal: "blake3:e50f9554c78a64559f6af91aa5fad667e72fe19defc14cb1ae748adbc9843e4a"
---

# Agent instruction file integrity seal

**Product lead for HashSeal.** Seal agent instruction files and verify them so developers and hosts only pass **approved** content to models.

## What it is

An **integrity seal** for instruction files (Markdown and configured instruct formats — `AGENTS.md`, skills, policy docs you keep next to code): a **BLAKE3 content digest** in YAML front matter. You verify that seal before agents or CI use the file. Optional GPG signature attests who sealed it (same settings as `git commit -S`).

This answers: *Is this still the instruction text I sealed?*  
It does **not** claim legal authorship, replace code review, or guarantee how a model will behave — only that the **file contents** match the seal.

## Why use it

- **Stop silent prompt edits** — multi-agent workflows and shared repos rewrite instructions; check fails with path + digests.
- **Gate model input** — fail the pipeline or refuse the agent run when seals break.
- **Same check everywhere** — CLI, tiny binary, browser extension, and zero-dep SDKs share official vectors.
- **Clear failures** — every non-OK path is listed (no silent exit-only fails).

## How it works

1. Canonicalize document content ([format details](format.md)).
2. Write `hashseal: "blake3:<hex>"` into front matter (seal fields excluded from the hash).
3. Optionally add `hashseal_sig` via `--sign`.
4. Before use: recompute digest; compare; report status per file.

```text
hashseal seal --instruct [--sign]
hashseal check [--require-signature]
```

## How to use it in your projects

### 1. Install the CLI

Package and binary are both named **`hashseal`**:

```bash
cargo build -p hashseal --release
# or cargo install --path rust/hashseal --locked
```

See [Install](../install.md).

### 2. Seal instructions

```bash
hashseal seal --instruct --root .
hashseal seal --instruct --sign --root .   # GPG via git config
```

By default this seals **agent instruction files only** (for example `AGENTS.md`, `CLAUDE.md`, Copilot/Cursor rules, and common agent skill/command directories) — not every `README.md` or docs page. Override with `document.include` / `document.exclude` in `.hashseal.json` (see [CLI config](../cli.md#default-instruct-includes)).

### 3. Check before agents / models

```bash
hashseal check --root .
# optional tiny binary (blake3-only deps):
hashseal-check --root .
```

### 4. Or check in-process (no CLI)

Use a [verify SDK](verify-sdks.md) in your language so CI or a host can validate Markdown without spawning a process.

### 5. Optional surfaces

| Surface | Use when |
|---------|----------|
| [Browser extension](../extensions/) | Paste-check instructions in the browser |
| [VS Code / IDE](../extensions/) | Seal/check from the editor |
| [Agent skills](../../skills/) | Teach agents to respect sealed files |
| [Signing](../signing.md) | Require cryptographic attestation |

## Docs in this section

| Page | Topic |
|------|--------|
| [Seal format](format.md) | Front matter, canonical modes, chicken-and-egg |
| [Verify SDKs](verify-sdks.md) | JS, Python, Java, Go, Ruby, .NET + vectors |
| [CLI](../cli.md) | `seal --instruct`, `check`, `unseal` |
| [Signing](../signing.md) | GPG / git key settings |

## Related (build line)

Tree ledgers and CI plugins live under **[Build tools](../build/)** — same core algorithms, different object (directory vs document).

```text
Copyright (c) 2026 MonkeyKing.dev
```
