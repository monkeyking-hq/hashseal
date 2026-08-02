---
hashseal: "blake3:c6e1f1abba09a75692fe38af3b91ffb182ce93a86a76239aaec2b7055a0194ed"
---
# AGENTS.md — HashSeal workspace

## Memory (read first — local only)

**`.hashseal-local/planning/MEMORY.md`** — durable snapshot of product status, next steps, GPG build-bot, toolchain notes, and stop rules. Update it when major milestones land.

Also (same directory, gitignored): `BUILD-STANDING-ORDERS.md`, `MVP-BACKLOG.md`, progress/overnight notes.

These files are **not** in git and are **not** on the public docs site. New clones will not have them until the operator restores local planning files.

## Product

Two lines, one core (see root `README.md` and `docs/index.md`):

1. **Agent instruction file integrity seal (lead)** — seal/verify instruction files so agents only see approved text.
2. **Build tools** — seal/verify trees and release bundles so CI / multi-agent builds cannot silently rewrite what you ship.

**Tagline (ASCII):** `Signed, Sealed, Delivered - I'm Yours.`

Public site: `docs/` (GitHub Pages) — `docs/instruct/`, `docs/build/`, custom layout + `docs/assets/`.

## Hard rules

1. **Independence:** Do not name, link, or copy third-party integrity products, their docs, configs, or commands.
2. **Core-first:** Integrity logic lives in `hashseal-core`. Plugins and extensions are thin adapters.
3. **Verify UX:** Failures must list **every** non-OK path with status and digests—never silent exit-only fails.
4. **TDD:** Vectors and tests first for seal/check behavior; multi-lang verify must share vectors.
5. **Branding:** Product voice is HashSeal. MonkeyKing.dev only in copyright/footer:
   `Copyright (c) 2026 MonkeyKing.dev`
6. **Contacts:** `info@hashseal.ai` (general), `security@hashseal.ai` (security).
7. **License:** Apache-2.0.
8. **CLI vocabulary:** Prefer `--instruct` (not `--docs`) for agent instruction sealing.
9. **Repo-relative paths:** Tests, scripts, and docs that load shared assets must use paths
   **relative to the monorepo root** (e.g. `verify/vectors/instruct-v1.json`, `fixtures/mvp-demo/`).
   Do **not** hardcode absolute host paths (`C:\workspaces\...`, `/home/...`).
   Resolve the root via `CARGO_MANIFEST_DIR` / `__dirname` / `__file__` (walk up or join) only as
   needed to locate that root — never bake machine-specific directories into sources or fixtures.
10. **Public vs local:** Do not commit strategic planning under `docs/` (`MEMORY`, `MVP-*`, `BUILD-*`). Do not commit AI agent session folders (`.grok/`, `.cursor/`, etc.).

## Layout

See root `README.md` and the architecture plan for monorepo map (`hashseal-core`, `hashseal` CLI, `hashseal-check`, `hashseal-wasm`, `verify/*`, `plugins/*`, `extensions/*`, `skills/*`, `docs/`).

## Dependency / binary size policy

- Prefer **std + blake3**. Avoid transitive bloat (no chrono, uuid, toml, walkdir, globset, thiserror).
- **`hashseal-check`**: `hashseal-core/check` only (blake3). Manual argv — no clap.
- **`hashseal-wasm`**: same as check.
- **`hashseal` (full CLI)**: may use clap + serde_json; keep default features lean.
- Config: **JSON** overlays (`.hashseal.json`), not a TOML crate.

## Local development

```bash
cargo test --workspace
cargo build -p hashseal-check --release
cargo tree -p hashseal-check
cargo fmt
cargo clippy --workspace -- -D warnings
```

### Git hooks

CI runs `cargo fmt --all -- --check`. After clone (once per machine):

```bash
./scripts/install-git-hooks.sh          # Unix / Git Bash
# or: pwsh scripts/install-git-hooks.ps1
```

Pre-commit runs rustfmt when staged `.rs` files exist. Hook lives at `scripts/git-hooks/pre-commit` (via `core.hooksPath`).
