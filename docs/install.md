---
hashseal: "blake3:f46575fb7e190f82181a521fe9587d0baa261bb927866edc37ce5df5d3db7f49"
---
# Installing HashSeal binaries

**Signed, Sealed, Delivered - I'm Yours.**

## Install from crates.io (recommended)

```bash
cargo install hashseal --locked
cargo install hashseal-check --locked
```

## Installer scripts

Download prebuilt release binaries and verify `SHA256SUMS`:

```bash
# Linux / macOS
./scripts/install.sh
# Or build from source:
./scripts/install.sh --from-source
```

```powershell
# Windows (PowerShell)
pwsh scripts/install.ps1
# Or build from source:
pwsh scripts/install.ps1 -FromSource
```

## Build from source with Cargo

```bash
cargo build -p hashseal --release
cargo build -p hashseal-check --release
# or install into ~/.cargo/bin:
cargo install --path rust/hashseal --locked
cargo install --path rust/hashseal-check --locked
```

## Environment variables

| Variable | Purpose |
|----------|---------|
| `PATH` | Must resolve `hashseal` / `hashseal-check` for plugins, hooks, and IDE commands |
| `HASHSEAL_BIN` | Absolute path override for the full CLI |
| `HASHSEAL_CHECK_BIN` | Optional override for `hashseal-check` |

CI release workflow: [`.gitea/workflows/release-binaries.yml`](../.gitea/workflows/release-binaries.yml).

```text
Copyright (c) 2026 Nathan A. Chadwick
```
