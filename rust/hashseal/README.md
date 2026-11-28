# hashseal

Full [HashSeal](https://hashseal.ai) CLI (`init`, `seal`, `check`, `verify`, `unseal`, `clean`) for cryptographic integrity, trust, and runtime gating of AI agent instruction files, skill packs, and release bundles.

**Signed, Sealed, Delivered - I'm Yours.**

## Install

```bash
cargo install hashseal
```

## Quickstart

```bash
# Bootstrap .hashseal.json, git hooks, Claude hooks, and .hashseal.lock
hashseal init --claude-hooks --git-hooks --lock

# Seal instruction files and lockfile
hashseal seal --instruct --lock --root .

# Verify seals, signatures, lockfile, and Unicode hygiene
hashseal check --root .
```

## Links

- Website: <https://hashseal.ai>
- Repository: <https://github.com/monkeyking-hq/hashseal>
- License: MIT

```text
Copyright (c) 2026 Nathan A. Chadwick
```
