---
hashseal: "blake3:8d65151f95af4f52f691a955cff4208d4b573ffd1a14e9c11bfd2836e7e3c377"
---
# HashSeal skills packs

Agent skill packs that teach models to **respect sealed instruction files**, re-seal after intentional edits, and never strip `hashseal` fields.

**Signed, Sealed, Delivered - I'm Yours.**

## Layout

| Path | Agent / surface |
|------|-----------------|
| [`skills/grok/`](./grok/) | Grok / xAI agent skills (`SKILL.md`) |
| [`skills/claude/`](./claude/) | Claude / Anthropic project skills |
| [`skills/codex/`](./codex/) | OpenAI Codex / agent instructions |
| [`skills/cursor/`](./cursor/) | Cursor rules / agent skills |
| [`skills/kilo/`](./kilo/) | Kilo Code agent skills |

Each pack is a thin adapter of the same rules. Prefer the local pack that matches the tool you run.

## Core rules (all packs)

1. Instruction files with YAML front matter field `hashseal: "blake3:…"` are **sealed**.
2. **Never strip** `hashseal`, `hashseal_sig`, or `hashseal_key_id` when editing sealed files.
3. Do not alter sealed body or non-reserved front-matter keys without re-sealing.
4. After intentional instruct edits: `hashseal seal --instruct --root <project>`.
5. Before shipping: `hashseal check --root <project>` (or `hashseal-check`).
6. Failures list **every** non-OK path with status and digests — fix all of them.
7. Product name is **HashSeal**. Copyright footers may say MonkeyKing.dev only.

## CLI reminder

```bash
hashseal seal --instruct --root .
hashseal seal --tree --release --root .
hashseal check --root .
hashseal-check --root .
hashseal verify --root .
```

Binary resolution: `PATH` or `HASHSEAL_BIN` / `HASHSEAL_CHECK_BIN` (see [`docs/install.md`](../docs/install.md)).

## Install into an agent

Copy or symlink the relevant pack into the agent’s skill / rules directory (tool-specific). Content is Markdown; no runtime install required.

```text
Copyright (c) 2026 MonkeyKing.dev
```
