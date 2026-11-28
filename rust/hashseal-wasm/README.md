# hashseal-wasm

WebAssembly verify surface for [HashSeal](https://hashseal.ai), exposing `wasm-bindgen` exports (`check_text_json`, `version`) on top of `hashseal-core/check`.

**Signed, Sealed, Delivered - I'm Yours.**

## Install

```bash
cargo add hashseal-wasm
```

## Exports

- `check_text_json(text: &str) -> String` — verifies `instruct-v2` seal and Unicode hygiene, returning a JSON status object (`ok`, `status`, `algorithm`, `expected`, `actual`, `message`).
- `version() -> String` — returns the `hashseal-core` version string.

## Links

- Website: <https://hashseal.ai>
- Repository: <https://github.com/monkeyking-hq/hashseal>
- License: MIT

```text
Copyright (c) 2026 Nathan A. Chadwick
```
