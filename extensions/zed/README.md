---
hashseal: "blake3:7b37a046cdcccaeb574da2e19ee0b23a84faaa291ab5aa51a371cc8158414980"
---
# HashSeal for Zed

Thin adapter: run **`hashseal-check`** / **`hashseal`** from the project root (same idea as the VS Code extension).

**Signed, Sealed, Delivered - I'm Yours.**

## Requirements

- [Zed](https://zed.dev) with task / shell support
- HashSeal CLI on **PATH**, or env:

| Env | Purpose |
|-----|---------|
| `HASHSEAL_BIN` | Full `hashseal` CLI |
| `HASHSEAL_CHECK_BIN` | Tiny `hashseal-check` binary |

Build from monorepo:

```bash
cargo build -p hashseal --release
cargo build -p hashseal-check --release
# add target/release to PATH
```

See [`docs/install.md`](../../docs/install.md).

## Tasks (recommended)

Copy [`tasks.json`](./tasks.json) into your project’s `.zed/tasks.json` (or merge entries), then run via **Zed → Tasks**.

| Task | Command |
|------|---------|
| HashSeal: Check | `hashseal-check --root $ZED_WORKTREE_ROOT` (falls back to `hashseal check`) |
| HashSeal: Seal Instruct | `hashseal seal --instruct --root $ZED_WORKTREE_ROOT` |
| HashSeal: Seal Tree | `hashseal seal --tree --root $ZED_WORKTREE_ROOT` |
| HashSeal: Verify | `hashseal verify --root $ZED_WORKTREE_ROOT` |

Prefer **`hashseal-check`** for day-to-day workspace checks (blake3-only, tiny binary). Use full **`hashseal`** for seal / bundle / report.

## Manual shell

From the project root:

```bash
hashseal-check --root .
# or
hashseal check --root .
hashseal seal --instruct --root .
hashseal seal --tree --release --root .
hashseal verify --root .
```

On verify failure, HashSeal lists **every** non-OK path with status and digests.

## Notes

- This folder is a **stub**: no native Zed extension crate yet — tasks + docs only.
- WASM / in-editor panel is deferred; shell out to CLI.
- Not published to any extension registry from agent builds unless requested.

```text
Copyright (c) 2026 MonkeyKing.dev
```
