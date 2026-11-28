# hashseal-check

Tiny verify-only [HashSeal](https://hashseal.ai) binary for `instruct-v2` seals, Unicode smuggling checks, and signatures.

**Dependency policy:** `hashseal-core` with `check` feature only (`blake3` + `std` — no `clap`, `serde`, `serde_json`, `walkdir`, or `git`).

**Signed, Sealed, Delivered - I'm Yours.**

## Install

```bash
cargo install hashseal-check
```

## Usage

```bash
hashseal-check --root .
```

## Links

- Website: <https://hashseal.ai>
- Repository: <https://github.com/monkeyking-hq/hashseal>
- License: MIT

```text
Copyright (c) 2026 Nathan A. Chadwick
```
