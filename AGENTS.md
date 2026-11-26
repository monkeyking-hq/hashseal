---
hashseal: "blake3:bf55c014b688139b1ff6e3005ea97087c9790e9a35b6ebbfdd765e3af06e3e44"
---
This project follows the Universal Code v1.0.0 — read https://github.com/monkeyking-hq/universal-code/blob/main/UC-EMBED-v1.0.0.md

# AGENTS.md — HashSeal workspace

**Working forge (Gitea):** `MonkeyKing.dev/hashseal` (`mkd-git:37374` / `stonemonkey:37373`). Use **`gt`** (`gitea-cli`, `/usr/bin/gt` — e.g. `gt pr create/list/view/checks --watch/merge --method merge --delete-branch --yes`, `gt issue list/create`, `-R MonkeyKing.dev/hashseal`, `--json`, `-q/--jq`, `--no-input`), never **`gh`**. See workspace `AGENTS.md`.

**Public GitHub mirror:** https://github.com/monkeyking-hq/hashseal (do not use `gh` for company `mkd-*` work).

## Product

**Cryptographic integrity, trust, and runtime gating for AI agent instruction files and skill packs** (`instruct-v2`, `signing-v2`, Unicode smuggling defense, `.hashseal.lock`, `hashseal init`, Tier-1 Rust/JS/Python/Go SDKs + WASM), plus optional signed release bundles.

**Tagline (ASCII):** `Signed, Sealed, Delivered - I'm Yours.`

## Hard rules

1. **Independence:** Do not name, link, or copy third-party integrity products, their docs, configs, or commands.
2. **Core-first:** Integrity logic lives in `hashseal-core`. Plugins and extensions are thin adapters.
3. **Verify UX:** Failures must list **every** non-OK path with status and digests—never silent exit-only fails.
4. **TDD:** Vectors and tests first for seal/check behavior; Tier-1 SDKs (`verify/js`, `verify/python`, `verify/go`) must share `verify/vectors/{instruct-v2,signing-v2,tree-v1}.json`.
5. **Branding:** Product voice is HashSeal. `Copyright (c) 2026 Nathan A Chadwick`
6. **Contacts:** `info@hashseal.ai` (general), `security@hashseal.ai` (security).
7. **License:** MIT.
8. **CLI vocabulary:** Prefer `--instruct` (not `--docs`) for agent instruction sealing.
9. **Repo-relative paths:** Tests, scripts, and docs must use paths relative to the monorepo root. Never hardcode absolute host paths.
10. **Public vs local:** Do not commit strategic planning under `docs/` or AI agent session folders (`.grok/`, `.cursor/`, etc.).

## Dependency / binary size policy

- Prefer **std + blake3**. Avoid transitive bloat (no chrono, uuid, toml, walkdir, globset, thiserror).
- **`hashseal-check`**: `hashseal-core/check` only (`blake3` + `std`). Manual argv — no clap, no serde.
- **`hashseal-wasm`**: `hashseal-core/check` + `wasm-bindgen`.
- **`hashseal` (full CLI)**: `clap` + `serde_json` allowed; keep default features lean.
- **Cargo profile:** Always pass `--release` for `cargo check/build/test/clippy/run` (never debug builds).

## Local development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --release -- -D warnings
cargo test --workspace --release
cargo tree -p hashseal-check --edges normal
node verify/js/test/vectors.test.js && node verify/js/test/tree-vectors.test.js && node verify/js/test/signing.test.js
python3 verify/python/test/test_vectors.py && python3 verify/python/test/test_tree_vectors.py && python3 verify/python/test/test_signing.py
(cd verify/go && go test ./...)
```

### Git hooks

```bash
./scripts/install-git-hooks.sh
```
