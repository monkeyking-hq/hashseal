---
hashseal: "blake3:0d8644af83250bf46069c8e0c3488d9d22003b42ddc942cef62f7b31be77f270"
---
# HashSeal build plugins

Thin adapters that shell to the **`hashseal` CLI**. Plugins do **not** re-implement seal/check algorithms.

Part of the **[Build tools](../build/)** product line (tree / release / CI). For instruction files, see **[Agent instruction file integrity seal](../instruct/)**.

**Signed, Sealed, Delivered - I'm Yours.**

## PATH / binary requirement

| Variable | Purpose |
|----------|---------|
| `PATH` | Must include `hashseal` (or `hashseal.exe` on Windows) |
| `HASHSEAL_BIN` | Optional absolute path override |

Build from monorepo:

```bash
cargo build -p hashseal --release
# target/release/hashseal[.exe]
```

Tiny verify-only binary (no clap/serde):

```bash
cargo build -p hashseal-check --release
```

Install story: [`docs/install.md`](../install.md).

## Matrix

| Plugin | Path | Status |
|--------|------|--------|
| npm | [`plugins/npm`](../../plugins/npm) | Skeleton — `hashseal-npm` + JS API |
| Maven | [`plugins/maven`](../../plugins/maven) | Skeleton — `seal` / `check` / `verify` goals; Central via [`java/`](../../java/) (`-Pcentral`, server `hashseal-central`) |
| Gradle | [`plugins/gradle`](../../plugins/gradle) | Skeleton — `hashsealSeal` / `Check` / `Verify` (includeBuild) |
| Cargo | [`plugins/cargo`](../../plugins/cargo) | Aliases + docs (no separate crate) |
| Python | [`plugins/python`](../../plugins/python) | Reserved / empty |
| Go | [`plugins/go`](../../plugins/go) | Reserved / empty |

## Zero-dep verify SDKs (not plugins)

For in-process digest check **without** the CLI:

| Lang | Path | Runner |
|------|------|--------|
| JS | `verify/js` | `node test/vectors.test.js` |
| Python | `verify/python` | `python test/test_vectors.py` |
| Java | `verify/java` | `javac` + `RunVectors` |
| Go | `verify/go` | `go test .` / `go run ./test/` |
| Ruby | `verify/ruby` | `ruby test/run_vectors.rb` (pure blake3) |
| .NET | `verify/dotnet` | `dotnet run --project Hashseal.Verify.Test` (pure blake3) |

Vectors: `verify/vectors/instruct-v1.json` (FULL canonical mode).

IDE: [`extensions/vscode`](../../extensions/vscode) (PATH / `HASHSEAL_BIN` / `HASHSEAL_CHECK_BIN`).

## Packaging notes

See **[docs/install.md](../install.md)**, **[docs/packaging.md](../packaging.md)**, and `.github/workflows/release-binaries.yml`.

- Agent builds **do not** publish to npm, Maven Central, or Plugin Portal unless explicitly requested.
- Prefer shipping platform binaries via GitHub Releases; plugins only resolve PATH/`HASHSEAL_BIN`.
- `hashseal-check` dependency tree must stay core → blake3 only.

```text
Copyright (c) 2026 MonkeyKing.dev
```
