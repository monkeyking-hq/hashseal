# hashseal-core

Core Rust library for [HashSeal](https://hashseal.ai) — cryptographic integrity, trust, and runtime gating for AI agent instruction files (`instruct-v2`), `signing-v2` (OpenSSH `SSHSIG` + hardened GPG), Unicode smuggling checks, `.hashseal.lock`, and tree/bundle verification.

**Signed, Sealed, Delivered - I'm Yours.**

## Install

```bash
cargo add hashseal-core
```

For minimal verify-only builds (`std` + `blake3`, no `serde`/`serde_json`):

```toml
[dependencies]
hashseal-core = { version = "0.7.1", default-features = false, features = ["check"] }
```

## Example

```rust
use hashseal_core::instruct::{check_instruct_bytes, seal_instruct_bytes, InstructOptions};

let opts = InstructOptions::default();
let (sealed, digest) = seal_instruct_bytes("# Agent Rules\n", &opts).unwrap();
let result = check_instruct_bytes(&sealed, &opts);
assert!(result.ok);
println!("Sealed: {}", digest.to_prefixed_string());
```

## Links

- Website: <https://hashseal.ai>
- Repository: <https://github.com/monkeyking-hq/hashseal>
- License: MIT

```text
Copyright (c) 2026 Nathan A. Chadwick
```
