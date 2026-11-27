# Changelog

## [0.6.2] - 2026-11-26

### Fixed
- Updated `docs/_includes/footer.html` license label from `Apache-2.0` to `MIT` and updated header/footer navigation labels (`Bundles` / `Signed bundles`).
- Updated `docs/_config.yml` description and `docs/security.md` scope notes to reflect `instruct-v2`, `signing-v2` (SSH + hardened GPG), `.hashseal.lock`, and `SUSPICIOUS_UNICODE`.
- Restored `.hashseal.json` configuration reference table and worktree-skip notes in `docs/cli.md`.
- Updated `extensions/browser/README.md` smoke command to the `instruct-v2` digest and refreshed `fixtures/mvp-demo/README.md`.

## [0.6.1] - 2026-11-26

### Fixed
- Updated `verify/go/test/run_vectors.go` from `instruct-v1.json` to `instruct-v2.json`.
- Updated subdirectory READMEs (`verify/{js,python,go,vectors}/README.md`, `extensions/{vscode,browser}/README.md`, `rust/hashseal-{core,wasm}/README.md`) and test header comments to match `v0.6.x` capabilities and pruned surfaces.

## [0.6.0] - 2026-11-26

### Added
- `scripts/install.sh` and `scripts/install.ps1` installers with `SHA256SUMS` verification and `--from-source` fallback.
- `SHA256SUMS` generation and binary smoke-testing in `.gitea/workflows/release-binaries.yml` and `scripts/build-release.sh`.

### Changed
- Re-scoped `README.md`, `AGENTS.md`, `docs/`, `verify/vectors/README.md`, and `config/examples/hashseal.mvp.json` around AI agent instruction & skill pack integrity (`instruct-v2`, `signing-v2`, `.hashseal.lock`, `hashseal init`, Unicode smuggling checks, Tier-1 Rust/JS/Python/Go SDKs + WASM).
- Removed stale references to pruned Java/Ruby/.NET/Python-plugin surfaces and non-existent `.github/workflows/` paths.

## [0.5.0] - 2026-11-26

### Added
- `hashseal init` subcommand (`--git-hooks`, `--claude-hooks`, `--lock`, `--sign`) to bootstrap `.hashseal.json`, `.hashseal/allowed_signers`, git pre-commit hooks, Claude Code `SessionStart`/`PreToolUse` hooks, and `.hashseal.lock`.
- Integration test `rust/hashseal/tests/cli_init.rs`.
- Reusable `.pre-commit-hooks.yaml` and composite `action/action.yml` for PR/CI enforcement.

### Changed
- Upgraded VS Code extension (`extensions/vscode` `0.5.0`) with `DiagnosticCollection` error squiggles, `StatusBarItem` indicator, `checkOnSave`, and true single-file `checkActiveFile`.
- Updated `skills/*/SKILL.md` so agents never overwrite human-signed (`hashseal_sig`) instruction files and understand `SUSPICIOUS_UNICODE`, `.hashseal.lock`, and `--diff`.

## [0.4.0] - 2026-11-26

### Added
- Frozen `verify/vectors/signing-v2.json` (5 SSHSIG cases, cross-checked by Rust, JS, Python, and Go).
- Ported `instruct-v2`, suspicious Unicode checks, and zero-dep in-process `SSHSIG` Ed25519 verification to Tier-1 SDKs (`verify/js`, `verify/python`, `verify/go`).
- Added `wasm-bindgen` exports (`check_text_json`, `version`) to `rust/hashseal-wasm`.
- Expanded `.gitea/workflows/ci.yml` to run Rust (`--release`) plus JS, Python, and Go SDK test suites.

### Changed
- Rebuilt `extensions/browser/lib/hashseal-verify.browser.js` and updated browser extension samples to `instruct-v2`.

### Removed
- Pruned dropped surfaces: `plugins/python`, `plugins/maven`, `plugins/gradle`, `java/`, `verify/java`, `verify/ruby`, `verify/dotnet`.

## [0.3.0] - 2026-11-26

### Added
- `signing-v2` unified signing/verification module (`rust/hashseal-core/src/signing.rs`) supporting OpenSSH `SSHSIG` (`ssh-keygen -Y` + `.hashseal/allowed_signers`) and hardened GPG (`--status-fd 1`, isolated `.hashseal/keys/` keyring, `allowed_key_ids` enforcement, RAII `0o700` temp dirs).
- Context `scope` binding (`HASHSEAL-SIG-V2`, `hashseal_scope`, `--portable` for `scope=*`) to prevent cross-file signature replay.
- Fail-closed signature verification (`CheckStatus::UnverifiedSignature`, opt-out via `--digest-only`).
- Signed instruction lockfile `.hashseal.lock` (`rust/hashseal-core/src/lockfile.rs`, `seal --lock`, `check --lock` / `--no-lock`, detecting `LOCKED_FILE_REMOVED` and `UNAPPROVED_FILE`).
- Signed `hashseal-bundle/MANIFEST.json.sig` support (`sign_bundle`, `verify_bundle_with_signing`).
- PR diff gate `hashseal check --diff <base-ref>`.

## [0.2.0] - 2026-11-26

### Added
- `instruct-v2` canonicalization with lossless raw front-matter framing (`HASHSEAL-INSTRUCT-V2` domain separation), protecting YAML lists and nested maps verbatim while excluding reserved seal/signature metadata.
- Built-in suspicious Unicode smuggling detector (`rust/hashseal-core/src/unicode.rs`) flagging Unicode Tag, BiDi control, zero-width, and embedded BOM characters (`SUSPICIOUS_UNICODE`, `--unicode-policy fail|warn|off`).
- `CheckStatus::UnverifiedSignature` and `CheckStatus::SuspiciousUnicode` in core results, reports, and CLI output.
- Streaming file hashing (`Digest::compute_reader` with streaming `Sha256Hasher` and `blake3::Hasher`) for binary files and artifacts.
- `BundleManifest` `ledger_digest` / `report_digest` and `verify_bundle` verification of `MANIFEST.json`, `ledger.json`, and `--artifact` digests.
- `WalkEntry` / `walk_entries` symlink awareness in `verify_tree` (`TreeEntryStatus::BrokenSymlink`) and deterministic segment-based `glob_match`.
- Frozen `verify/vectors/instruct-v2.json` (19 cases).

### Changed
- `HashSealConfig::apply_json` now parses all documented `signing`, `enforce`, `document`, `tree`, and `report` fields.
- Fixed `is_reserved_fm_key` to respect custom `document.field`, fixed `parse_document` offset handling for embedded `---` lines in front matter, and fixed `extract_reserved_field` indentation check.
- `hashseal verify` now invokes `verify_bundle` when `--bundle` is passed or `hashseal-bundle/MANIFEST.json` is present.

### Removed
- `verify/vectors/instruct-v1.json` (replaced by `instruct-v2.json`).

## 2026-11-20

### Changed
- Migrated Gitea forge CLI instructions in `AGENTS.md` from `tea` to `gt` (`gitea-cli`, `/usr/bin/gt`, preserving the strict never-`gh` rule for Gitea; re-sealed `AGENTS.md` with `hashseal seal --instruct`).
