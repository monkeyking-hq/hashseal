# HashSeal

**Signed, Sealed, Delivered - I'm Yours.**

**Website:** [hashseal.ai](https://hashseal.ai) — public docs and marketing from this repo’s [`docs/`](docs/) tree (GitHub Pages). See [`docs/index.md`](docs/index.md).

HashSeal is integrity tooling for two related problems:

1. **Agent instruction file integrity seal** — seal and verify instruction files so agents and models only run on **what you approved**.
2. **Build tools** — seal and verify source trees and release bundles so CI and multi-agent builds cannot silently rewrite what you ship.

**What was sealed is what you still have** — and verify tools always name every file that changed.

| | |
|--|--|
| License | Apache-2.0 |
| Contact | [info@hashseal.ai](mailto:info@hashseal.ai) · Security: [security@hashseal.ai](mailto:security@hashseal.ai) |
| Org (footer only) | MonkeyKing.dev |

---

## 1. Agent instruction file integrity seal

### What it is

An **integrity seal** for agent instruction files (Markdown and configured instruct formats — e.g. `AGENTS.md`, skill packs, system prompts you keep in-repo): a content digest in front matter, optional GPG signature using the same key as `git commit -S`.

Before you pass instructions to a model or agent host, you **check** that seal. If someone (or another agent) rewrote the file, check fails and lists **every** non-OK path with status and digests — never a silent exit-only fail.

This is **integrity** (the file still matches what was sealed), not a claim about model behavior or legal authorship.

### Why use it

| Risk | Without seals | With HashSeal |
|------|----------------|---------------|
| Prompt / policy drift | Silent edits to instruction files | `MISMATCH` with expected vs actual digests |
| Multi-agent tampering | One agent changes rules for the next | Verify before handoff |
| “Which AGENTS.md did we ship?” | Guesswork | Sealed digest + optional signature |
| Browser / IDE paste | Trust whatever is on screen | Client-side or CLI check before run |

### How it works

1. Content is canonicalized (see [instruct seal format](docs/instruct/format.md)).
2. A BLAKE3 digest is written into YAML front matter as `hashseal: "blake3:<hex>"`.
3. Seal fields themselves are excluded from the hash (no chicken-and-egg).
4. Optional `hashseal_sig` holds a detached-style GPG armor over the digest.
5. `check` recomputes and reports `ok` / `MISMATCH` / `MISSING_SEAL` / …

Same algorithm is implemented in:

- Full CLI: `hashseal seal --instruct` / `hashseal check`
- Tiny binary: `hashseal-check` (blake3-only deps)
- WASM: `hashseal-wasm`
- Zero-dep SDKs: `verify/js`, `verify/python`, `verify/java`, `verify/go`, `verify/ruby`, `verify/dotnet`
- Browser extension (pure JS paste check)

Official vectors: [`verify/vectors/instruct-v1.json`](verify/vectors/instruct-v1.json).

### How to use it in your projects

Commands below assume your shell’s current directory is the **project root** (or this monorepo root when developing HashSeal). Paths are repo-relative — no host-specific prefixes.

**Install / build the CLI** (package and binary are both named `hashseal`):

```bash
# Linux / macOS / Git Bash
cargo build -p hashseal --release
# → target/release/hashseal

# optional install into Cargo’s bin dir (already on PATH for many setups):
cargo install --path rust/hashseal --locked
```

```powershell
# Windows (PowerShell)
cargo build -p hashseal --release
# → target\release\hashseal.exe

cargo install --path rust/hashseal --locked
```

**Seal instruction files** in a project:

```bash
hashseal seal --instruct --root .
# optional GPG (uses git’s gpg.program + signing key):
hashseal seal --instruct --sign --root .
```

**Verify before agents or CI consume them:**

```bash
hashseal check --root .
hashseal check --require-signature   # if you seal with --sign
# tiny binary (optional):
hashseal-check --root .
```

**In-process check (no CLI)** — JS SDK example from monorepo root (Linux, macOS, or Windows):

```bash
node -e "const fs=require('fs');const {checkDocumentText}=require('./verify/js');const r=checkDocumentText(fs.readFileSync('AGENTS.md','utf8'));console.log(r);process.exit(r.ok?0:1)"
```

**Editor / browser:**

| Surface | Path | Role |
|---------|------|------|
| VS Code | [`extensions/vscode`](extensions/vscode) | Commands spawn CLI / check |
| Browser | [`extensions/browser`](extensions/browser) | Paste Markdown → pure JS check |
| Zed / Antigravity | [`extensions/zed`](extensions/zed), [`extensions/antigravity`](extensions/antigravity) | Host stubs |
| Agent skills | [`skills/`](skills) | Skill packs for common agents |

**Docs (instruction integrity):**

- [What & why](docs/instruct/index.md)
- [Seal format](docs/instruct/format.md)
- [Signing](docs/signing.md)
- [CLI reference](docs/cli.md)
- [Install](docs/install.md)
- [IDE & browser](docs/extensions/README.md)
- [Verify SDKs & vectors](docs/instruct/verify-sdks.md)

### 5-minute smoke (instruct)

From the monorepo root (works the same on Linux, macOS, and Windows once Rust/`cargo` are installed):

```bash
cargo build -p hashseal --release
cargo run -p hashseal -- seal --instruct --root fixtures/mvp-demo
cargo run -p hashseal -- check --root fixtures/mvp-demo
```

---

## 2. Build tools

### What they are

The same core and CLI also protect **trees and release artifacts** during development and CI:

- **Tree seal** — ledger of path digests for a configured include/exclude set  
- **Release bundle** — `hashseal-bundle/` (ledger + report + MANIFEST, optional extra artifacts)  
- **Plugins** — thin wrappers (npm, Maven, Gradle, Cargo aliases) that shell to `hashseal` on PATH  
- **Verify** — re-walk and compare; list every drifted path  

Purpose: stop **silent rewrites** across multi-step / multi-agent build pipelines so **what you built is what you ship**.

### How they work

1. Config (JSON): `.hashseal.json` overlay — tree includes/excludes, document flags, signing, reports. Example: [`config/examples/hashseal.mvp.json`](config/examples/hashseal.mvp.json).
2. `hashseal seal --tree` writes a ledger; `--release` stages a bundle under `hashseal-bundle/`.
3. `hashseal verify` checks the tree (and optional bundle) against the ledger.
4. Build plugins invoke the CLI; they do **not** reimplement digests.

Tree vectors: [`verify/vectors/tree-v1.json`](verify/vectors/tree-v1.json).

### How to use them

```bash
# Tree + release bundle (any OS once hashseal is on PATH)
hashseal seal --tree --release --root .
hashseal verify --root .

# Instruct + tree together
hashseal seal --instruct --tree --release --root .
```

**Put CLI on PATH** (plugins require `hashseal` resolvable from the shell). From the monorepo root after a release build:

```bash
# Linux / macOS / Git Bash
export PATH="$(pwd)/target/release:$PATH"
# or point plugins at a specific binary:
export HASHSEAL_BIN="$(pwd)/target/release/hashseal"
```

```powershell
# Windows (PowerShell)
$env:PATH = "$(Join-Path (Get-Location) 'target\release');$env:PATH"
# or point plugins at a specific binary:
$env:HASHSEAL_BIN = (Join-Path (Get-Location) 'target\release\hashseal.exe')
```

If the binary already lives on PATH (for example after `cargo install --path rust/hashseal --locked`), you can skip the `PATH` / `HASHSEAL_BIN` step.

### Stack map (docs per surface)

| Surface | Path | Docs |
|---------|------|------|
| Full CLI | `rust/hashseal` | [CLI](docs/cli.md) · [Install](docs/install.md) |
| Core library | `rust/hashseal-core` | crate README |
| Tiny check binary | `rust/hashseal-check` | optional instruct-only verify (blake3 path) |
| npm plugin | `plugins/npm` | [plugins hub](docs/build/index.md) · [npm README](plugins/npm/README.md) |
| Maven plugin | `plugins/maven` | [Maven README](plugins/maven/README.md) |
| Java verify SDK | `verify/java` | `ai.hashseal:hashseal-verify` · [Java README](verify/java/README.md) · [reactor](java/README.md) |
| Gradle | `plugins/gradle` | [Gradle README](plugins/gradle/README.md) |
| Cargo aliases | `plugins/cargo` | [Cargo README](plugins/cargo/README.md) |
| Packaging / releases | scripts + CI | [packaging](docs/packaging.md) · [install](docs/install.md) |

**Build docs hub:** [`docs/build/index.md`](docs/build/index.md)

### 5-minute smoke (build)

```bash
cargo run -p hashseal -- seal --tree --release --root fixtures/mvp-demo
cargo run -p hashseal -- verify --root fixtures/mvp-demo
```

---

## Develop (monorepo)

All paths are relative to the monorepo root. Use forward slashes in these examples; on Windows, `cargo`, `node`, and `python` accept them the same way.

```bash
# Linux / macOS / Windows (PowerShell or cmd with cargo on PATH)
cargo test --workspace
cargo build -p hashseal --release
cargo build -p hashseal-check --release
cargo tree -p hashseal-check --edges normal   # expect core → blake3 only
cargo fmt
cargo clippy --workspace -- -D warnings
```

### Git hooks (optional, recommended)

CI runs `cargo fmt --all -- --check`. Install the repo pre-commit hook so that fails locally before push when staged `.rs` files are present:

```bash
# Linux / macOS / Git Bash
./scripts/install-git-hooks.sh

# Windows (PowerShell)
pwsh scripts/install-git-hooks.ps1
```

That sets `core.hooksPath=scripts/git-hooks` for this clone only. Fix failures with `cargo fmt --all`, then re-stage and commit.

```bash
# Multi-lang instruct vectors + tree where implemented
node verify/js/test/vectors.test.js
node verify/js/test/tree-vectors.test.js
python verify/python/test/test_vectors.py
mvn -f verify/java/pom.xml test
# Go / Ruby / .NET — see verify/*/README.md
```

### Workspace layout

| Path | Role |
|------|------|
| `rust/hashseal-core` | Seal / check algorithms |
| `rust/hashseal` | Full `hashseal` CLI (package name = binary name) |
| `rust/hashseal-check` | Tiny instruct verify binary (blake3-only) |
| `rust/hashseal-wasm` | WASM verify for IDEs / browser |
| `verify/` | Zero-dep language verify SDKs + vectors |
| `java/` | Maven parent/reactor for `hashseal-verify` + Maven plugin (Central) |
| `fixtures/mvp-demo/` | Human smoke project |
| `config/examples/` | Sample `.hashseal.json` overlays |
| `plugins/` | Maven, npm, Gradle, Cargo, … |
| `extensions/` | VS Code, browser, Zed, … |
| `skills/` | Agent skill packs |
| `docs/` | GitHub Pages site + product docs |

---

## License

Apache-2.0 — see [LICENSE](LICENSE).

```text
Copyright (c) 2026 MonkeyKing.dev
```
