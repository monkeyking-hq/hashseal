# HashSeal

**Signed, Sealed, Delivered - I'm Yours.**

**Website:** [hashseal.ai](https://hashseal.ai) — public docs from [`docs/`](docs/). See [`docs/index.md`](docs/index.md).

HashSeal provides **cryptographic integrity, trust, and runtime gating for AI agent instruction files and skill packs** (`AGENTS.md`, `CLAUDE.md`, `SKILL.md`, `.cursor/rules/*.mdc`, Copilot/Windsurf/Cline rules), plus optional signed release bundles.

**What was sealed is what you still have** — and verify tools always name every file that changed.

| | |
|--|--|
| License | MIT |
| Contact | [info@hashseal.ai](mailto:info@hashseal.ai) · Security: [security@hashseal.ai](mailto:security@hashseal.ai) |
| Org (footer only) | MonkeyKing.dev |

---

## Why HashSeal

| Threat | Without HashSeal | With HashSeal (`v0.6.0`) |
|---|---|---|
| Silent prompt/policy edits | Agents or PRs rewrite `AGENTS.md` / `SKILL.md` | `MISMATCH` with expected vs actual BLAKE3 digests |
| YAML policy bypass | Altering `globs:` or `tools:` lists goes unnoticed | `instruct-v2` hashes raw front matter verbatim |
| Cross-file signature replay | Signed harmless file copied over `AGENTS.md` | `signing-v2` binds `scope` (or `--portable` `*` for skill packs) |
| Deleted or unapproved files | Agent removes or drops in unapproved rules | `.hashseal.lock` catches `LOCKED_FILE_REMOVED` & `UNAPPROVED_FILE` |
| Invisible Unicode smuggling | Hidden Tag / BiDi controls inject prompts | `SUSPICIOUS_UNICODE` blocks hidden Tag/BiDi/ZWSP characters |
| Unverified signatures | Missing trust root silently passes | Fails closed (`UNVERIFIED_SIGNATURE`); SSH `allowed_signers` + GPG |

---

## Quickstart

### 1. Install

```bash
# Release archive (Linux / macOS) with SHA256SUMS verification:
./scripts/install.sh
# Or build from source:
./scripts/install.sh --from-source
```

See [docs/install.md](docs/install.md) for Windows (`scripts/install.ps1`) and `cargo install`.

### 2. Bootstrap a repository

```bash
hashseal init --claude-hooks --git-hooks --lock
```

This creates `.hashseal.json`, `.hashseal/allowed_signers`, `.githooks/pre-commit`, Claude Code `SessionStart`/`PreToolUse` hooks, seals matching instruction files, and writes `.hashseal.lock`.

### 3. Sign & verify (SSH or GPG)

```bash
# Sign instruction files + lockfile (auto-detects git gpg.format = ssh | openpgp)
hashseal seal --instruct --sign --lock --root .

# Portable signature for distributable skill packs (scope = "*")
hashseal seal --instruct --sign --portable --root skills/my-pack

# Verify seals, signatures, lockfile, and Unicode hygiene
hashseal check --root . --require-signature

# PR gate: require signatures on changed instruction files vs origin/main
hashseal check --diff origin/main --require-signature
```

---

## Architecture & Surfaces

| Surface | Path | Role |
|---|---|---|
| Core library | [`rust/hashseal-core`](rust/hashseal-core) | `instruct-v2`, `signing-v2` (SSH + GPG), Unicode scan, `.hashseal.lock`, tree/bundle |
| Full CLI | [`rust/hashseal`](rust/hashseal) | `init`, `seal`, `check`, `verify`, `unseal`, `clean` |
| Tiny check binary | [`rust/hashseal-check`](rust/hashseal-check) | Instruct check (`blake3` + `std` only) |
| WASM | [`rust/hashseal-wasm`](rust/hashseal-wasm) | `wasm-bindgen` (`check_text_json`, `version`) |
| Tier-1 Verify SDKs | [`verify/js`](verify/js), [`verify/python`](verify/python), [`verify/go`](verify/go) | Zero-dep in-process `instruct-v2` + Unicode + Ed25519 `SSHSIG` |
| Test vectors | [`verify/vectors`](verify/vectors) | `instruct-v2.json` (19), `signing-v2.json` (5), `tree-v1.json` (12) |
| CI & Git hooks | [`action/action.yml`](action/action.yml), [`.pre-commit-hooks.yaml`](.pre-commit-hooks.yaml) | PR/CI & pre-commit enforcement |
| IDE / Browser | [`extensions/vscode`](extensions/vscode), [`extensions/browser`](extensions/browser) | Inline diagnostics + status bar; MV3 paste check |
| Agent skills | [`skills/`](skills) | Claude, Codex, Cursor, Grok, Kilo |
| Build wrappers | [`plugins/npm`](plugins/npm), [`plugins/cargo`](plugins/cargo) | Thin CLI wrappers |

---

## Develop (Monorepo)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --release -- -D warnings
cargo test --workspace --release
cargo tree -p hashseal-check --edges normal   # blake3 + std only

node verify/js/test/vectors.test.js && node verify/js/test/tree-vectors.test.js && node verify/js/test/signing.test.js
python3 verify/python/test/test_vectors.py && python3 verify/python/test/test_tree_vectors.py && python3 verify/python/test/test_signing.py
(cd verify/go && go test ./...)
```

---

## Universal Code & License

Follows [Universal Code](https://github.com/monkeyking-hq/universal-code). Released under the [MIT License](LICENSE).

```text
Copyright (c) 2026 Nathan A Chadwick
```
