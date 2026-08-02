---
hashseal: "blake3:a2e8586685db1d8b747e5831cacf21229886b7f4ef2c7c660c31b538c55e482f"
---
# hashseal cargo integration

Thin **Cargo** integration that invokes the **`hashseal` CLI** (or `hashseal-check`) via aliases / PATH.

**Signed, Sealed, Delivered - I'm Yours.**

## Status

Documented skeleton (no separate `cargo-hashseal` crate yet). Prefer monorepo crates and aliases.

## PATH requirement

Same as other plugins: `hashseal` on **PATH** or `HASHSEAL_BIN`.  
Tiny verify: `hashseal-check` on PATH or `HASHSEAL_CHECK_BIN`.

```bash
cargo build -p hashseal --release
cargo build -p hashseal-check --release
# add target/release to PATH
```

See [`docs/install.md`](../../docs/install.md).

## Cargo aliases (recommended)

In the monorepo or consumer `.cargo/config.toml`:

```toml
[alias]
# When developing inside this workspace:
hashseal = ["run", "-p", "hashseal", "--"]
hashseal-check = ["run", "-p", "hashseal-check", "--"]
```

Then:

```bash
cargo hashseal -- seal --instruct --root .
cargo hashseal -- check --root .
cargo hashseal-check -- --root .
```

When the CLI is installed globally (`cargo install --path rust/hashseal`), call bare:

```bash
hashseal seal --instruct --root .
hashseal check --root .
hashseal-check --root .
```

## build.rs hook (optional pattern)

Shell out without extra crates:

```rust
// build.rs (example — not enabled by default)
fn main() {
    if std::env::var("HASHSEAL_BUILD_CHECK").ok().as_deref() != Some("1") {
        return;
    }
    let bin = std::env::var("HASHSEAL_BIN").unwrap_or_else(|_| "hashseal".into());
    let status = std::process::Command::new(&bin)
        .args(["check", "--root", "."])
        .status()
        .expect("spawn hashseal");
    if !status.success() {
        panic!("hashseal check failed (CLI lists every non-OK path)");
    }
}
```

## Packaging

- Full CLI: `hashseal` (seal, sign, bundle).
- Tiny verify: `hashseal-check` (blake3 path only — see root `AGENTS.md`).
- Do not publish a separate cargo plugin crate from agent builds unless requested.

```text
Copyright (c) 2026 MonkeyKing.dev
```
