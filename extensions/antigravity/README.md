---
hashseal: "blake3:13d0984eff96a6b00840b19b3a9c7a5a2ab0360fb5bb3b23e33fa27061f26243"
---
# HashSeal for Antigravity

Thin adapter documentation for running **`hashseal-check`** / **`hashseal`** from an Antigravity (or compatible agent/IDE) workspace — same CLI surface as VS Code / Zed.

**Signed, Sealed, Delivered - I'm Yours.**

## Requirements

- HashSeal CLI on **PATH**, or:

| Env | Purpose |
|-----|---------|
| `HASHSEAL_BIN` | Full `hashseal` CLI |
| `HASHSEAL_CHECK_BIN` | Tiny `hashseal-check` |

```bash
cargo build -p hashseal --release
cargo build -p hashseal-check --release
```

See [`docs/install.md`](../../docs/install.md).

## Commands to wire

Prefer the tiny check binary for verify-on-open / pre-commit style hooks:

```bash
hashseal-check --root .
```

Full CLI:

```bash
hashseal check --root .
hashseal seal --instruct --root .
hashseal seal --tree --release --root .
hashseal verify --root .
```

## Suggested agent skill

Point the agent at monorepo skills:

- [`skills/grok/SKILL.md`](../../skills/grok/SKILL.md) (or claude / codex / cursor / kilo)

Rules of thumb:

1. Never strip `hashseal` / `hashseal_sig` / `hashseal_key_id` fields.
2. After intentional instruct edits, re-seal with `hashseal seal --instruct`.
3. On check failure, list **every** non-OK path (CLI already does this).

## Config snippet

Copy [`hashseal.antigravity.json`](./hashseal.antigravity.json) into the project as a reminder overlay, or merge keys into `.hashseal.json` (see [`config/examples/hashseal.mvp.json`](../../config/examples/hashseal.mvp.json)).

| Key (documentation only) | Meaning |
|--------------------------|---------|
| `preferCheckBinary` | Use `hashseal-check` when available |
| `checkOn` | Suggested triggers: `save`, `precommit` (host-defined) |

## Notes

- **Stub only** — no proprietary Antigravity SDK packaging here.
- Same independence rules as the rest of HashSeal: product voice is HashSeal; no third-party integrity product copy.

```text
Copyright (c) 2026 MonkeyKing.dev
```
