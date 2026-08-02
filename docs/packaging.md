---
hashseal: "blake3:8baad375ae27b2081efd38c02973a08a97cfe91e32a86a1777d65d52e79571c1"
---
# Packaging notes

**Signed, Sealed, Delivered - I'm Yours.**

## Goals

1. Ship platform binaries (`hashseal`, `hashseal-check`) that plugins and IDEs can find on PATH.
2. Keep verify libraries zero-package-deps across languages.
3. Do **not** auto-publish registries from agent builds.

## Binary matrix (intended)

| Target triple | OS | Arch | Notes |
|---------------|----|------|-------|
| `x86_64-unknown-linux-gnu` | Linux | x86_64 | Primary CI |
| `aarch64-unknown-linux-gnu` | Linux | arm64 | Optional |
| `x86_64-pc-windows-msvc` | Windows | x86_64 | Primary CI |
| `x86_64-apple-darwin` | macOS | x86_64 | Optional |
| `aarch64-apple-darwin` | macOS | arm64 | Apple Silicon |

Workflow skeleton: `.github/workflows/release-binaries.yml`.

## Suggested archive layout (future GH Releases)

```text
hashseal-0.1.0-x86_64-unknown-linux-gnu.tar.gz
  hashseal
  hashseal-check
  LICENSE
  README.txt

hashseal-0.1.0-x86_64-pc-windows-msvc.zip
  hashseal.exe
  hashseal-check.exe
  LICENSE
  README.txt
```

## Plugins vs SDKs

| Surface | Ships | Needs binary? |
|---------|-------|----------------|
| `plugins/npm`, `maven`, `gradle`, `cargo` | Thin wrappers | Yes — `hashseal` CLI |
| `verify/js|python|java|go|ruby|dotnet` | In-process digests | No |
| `extensions/vscode` | Commands spawn CLI | Yes — `hashseal` or `hashseal-check` |
| `hashseal-wasm` | WASM check | No native CLI |

## Java / Maven Central

Namespace: **`ai.hashseal`**. Reactor parent: [`java/pom.xml`](../java/pom.xml) (see [`java/README.md`](../java/README.md)).

| Artifact | Module |
|----------|--------|
| `ai.hashseal:hashseal-java-parent` | `java/` |
| `ai.hashseal:hashseal-verify` | `verify/java/` |
| `ai.hashseal:hashseal-maven-plugin` | `plugins/maven/` |

Portal server id in `settings.xml`: **`hashseal-central`**. Deploy profile: **`-Pcentral`** (sources + javadoc + GPG + `central-publishing-maven-plugin`). Versioning/tags: **`maven-release-plugin`** (`release:prepare` / `release:perform`, tag `hashseal-java-@{version}`). Operator-only — do not auto-publish from agent builds. See [`java/README.md`](../java/README.md).

## Size / deps policy

- **`hashseal-check`**: `hashseal-core` check feature + blake3 only.
- **Full `hashseal`**: clap + serde_json allowed; keep default features lean.
- Config overlays: JSON only (no TOML crate).

## Local packaging helper

```bash
# scripts/build-release.sh  (optional; see scripts/)
cargo build -p hashseal --release
cargo build -p hashseal-check --release
cargo tree -p hashseal-check --edges normal
```

```text
Copyright (c) 2026 MonkeyKing.dev
```
