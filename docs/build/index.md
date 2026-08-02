---
layout: default
title: Build tools
permalink: /build/
---

# Build tools

Second HashSeal product line: **seal and verify source trees and release artifacts** so multi-step and multi-agent CI cannot silently rewrite what you ship.

Instruction-file seals live under **[Agent instruction file integrity seal](../instruct/)** — same core, different object.

## What they are

| Capability | What you get |
|------------|----------------|
| **Tree seal** | Ledger of digests for configured paths |
| **Release bundle** | `hashseal-bundle/` — ledger, report, MANIFEST, optional artifact digests |
| **Verify** | Re-walk vs ledger; list every drifted path |
| **Plugins** | npm / Maven / Gradle / Cargo shells to the `hashseal` CLI |
| **Config** | JSON overlay `.hashseal.json` (no TOML crate) |

## How they work

1. Configure includes/excludes and options (example: [`config/examples/hashseal.mvp.json`](../../config/examples/hashseal.mvp.json)).
2. `hashseal seal --tree` writes the ledger; `--release` stages `hashseal-bundle/`.
3. `hashseal verify` checks the tree (and optional bundle).
4. Plugins only invoke the CLI — algorithms stay in `hashseal-core`.

```bash
hashseal seal --tree --release --root .
hashseal verify --root .
# Combined with instruction seals:
hashseal seal --instruct --tree --release --root .
```

Official tree vectors: [`verify/vectors/tree-v1.json`](../../verify/vectors/tree-v1.json).

## How to use in CI / projects

1. [Install](../install.md) `hashseal` on the runner (or set `HASHSEAL_BIN`).
2. Seal after the tree is in the state you intend to protect (or at release packaging time).
3. Verify in a later job or before publish.
4. Optionally wire a [build plugin](../plugins/) so `npm` / Maven / Gradle goals shell to the same binary.

## Stack — docs per tool

| Tool | Repo path | Documentation |
|------|-----------|----------------|
| Full CLI | `rust/hashseal` | [CLI reference](../cli.md) · [Install](../install.md) |
| Core | `rust/hashseal-core` | Algorithms for tree + instruct |
| Tiny check binary | `rust/hashseal-check` | Instruct-only; optional for gates (see install) |
| npm plugin | `plugins/npm` | [npm README](../../plugins/npm/README.md) · [plugins hub](../plugins/) |
| Maven plugin | `plugins/maven` | [Maven README](../../plugins/maven/README.md) |
| Gradle plugin | `plugins/gradle` | [Gradle README](../../plugins/gradle/README.md) |
| Cargo aliases | `plugins/cargo` | [Cargo README](../../plugins/cargo/README.md) |
| Python / Go plugin slots | `plugins/python`, `plugins/go` | Reserved |
| Packaging / GH Releases | `scripts/`, `.github/workflows/` | [Packaging](../packaging.md) |
| Demo fixture | `fixtures/mvp-demo` | [fixture README](../../fixtures/mvp-demo/README.md) |

## Plugin matrix (summary)

Plugins **require** `hashseal` on `PATH` or `HASHSEAL_BIN`. They do not embed binaries.

See [Build plugins](../plugins/) for PATH setup and status per language.

## Related

- [CLI](../cli.md) — `seal --tree`, `--release`, `verify`, `clean`
- [Signing](../signing.md) — GPG for instruct seals (tree path may grow attestation later)
- [Agent instruction file integrity seal](../instruct/) — document seals for agents

```text
Copyright (c) 2026 MonkeyKing.dev
```
