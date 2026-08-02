---
hashseal: "blake3:06db60e2d8a17eed79ae50889d8dfa9e63a60859e4c015962e4712fed68aa231"
---
# Installing HashSeal binaries

**Signed, Sealed, Delivered - I'm Yours.**

Plugins (npm, Maven, Gradle, Cargo wrappers) **shell to** the native CLI. They do not embed binaries. Put `hashseal` / `hashseal-check` on **PATH**, or set **`HASHSEAL_BIN`**.

## Binaries

| Binary | Role | Dep policy |
|--------|------|------------|
| `hashseal` | Full CLI: seal / check / verify / clean, optional GPG sign | clap + serde_json OK |
| `hashseal-check` | Tiny verify-focused binary | **blake3 path only** (`hashseal-core/check`) |

On Windows the names are `hashseal.exe` and `hashseal-check.exe`.

## Build from source (all platforms)

Requires a Rust toolchain ([rustup](https://rustup.rs)).

```bash
# From monorepo root
cargo build -p hashseal --release
cargo build -p hashseal-check --release
```

Artifacts:

| OS | Path |
|----|------|
| Linux / macOS | `target/release/hashseal`, `target/release/hashseal-check` |
| Windows | `target\release\hashseal.exe`, `target\release\hashseal-check.exe` |

### Put on PATH

**Unix (bash/zsh):**

```bash
export PATH="$PWD/target/release:$PATH"
# or permanent: copy/symlink into ~/bin or /usr/local/bin
```

**PowerShell:**

```powershell
$env:PATH = "$PWD\target\release;$env:PATH"
# or set HASHSEAL_BIN:
$env:HASHSEAL_BIN = "$PWD\target\release\hashseal.exe"
```

**cmd.exe:**

```bat
set PATH=%CD%\target\release;%PATH%
set HASHSEAL_BIN=%CD%\target\release\hashseal.exe
```

### cargo install (local path)

```bash
cargo install --path rust/hashseal --locked
cargo install --path rust/hashseal-check --locked
# installs into ~/.cargo/bin (ensure that dir is on PATH)
```

## Environment variables

| Variable | Purpose |
|----------|---------|
| `PATH` | Must resolve `hashseal` (or `hashseal.exe`) for plugins and IDE commands |
| `HASHSEAL_BIN` | Absolute path override for the full CLI (plugins prefer this when set) |
| `HASHSEAL_CHECK_BIN` | Optional override for the tiny check binary (VS Code extension, scripts) |

## Cross-platform release builds

Local one-off:

```bash
# Host triple
cargo build -p hashseal --release
cargo build -p hashseal-check --release

# Example cross targets (requires appropriate linker / rustup target)
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-apple-darwin
cargo build -p hashseal --release --target x86_64-unknown-linux-gnu
```

CI skeleton: [`.github/workflows/release-binaries.yml`](../.github/workflows/release-binaries.yml) builds multi-target artifacts (no secrets required for build-only).

Future: attach archives to **GitHub Releases** (`hashseal-<version>-<target>.tar.gz` / `.zip`). Plugins continue to resolve **PATH** / **HASHSEAL_BIN** only — they do not download binaries unless you add that later.

## Verify install

```bash
hashseal --help
hashseal-check --help
hashseal check --root fixtures/mvp-demo
hashseal-check --root fixtures/mvp-demo
```

## Packaging notes

- Agent / overnight builds **do not** publish crates.io, npm, Maven Central, or Plugin Portal unless explicitly requested.
- Keep `hashseal-check` dependency tree lean: `cargo tree -p hashseal-check --edges normal` should show core → blake3 only (no clap/serde).
- See also [`docs/packaging.md`](packaging.md) and [`docs/plugins/README.md`](plugins/README.md).

```text
Copyright (c) 2026 MonkeyKing.dev
```
