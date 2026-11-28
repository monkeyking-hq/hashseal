---
hashseal: "blake3:5e305602a93ff03aaaa2fac9e4bf6cdcc1ee4bbbdf83e5dad8b50a18a07e58ea"
---
# Packaging notes

**Signed, Sealed, Delivered - I'm Yours.**

## Goals

1. Ship platform binaries (`hashseal`, `hashseal-check`) with `SHA256SUMS` verification.
2. Keep Tier-1 verify SDKs (`verify/js`, `verify/python`, `verify/go`) zero-external-dep.
3. Keep `hashseal-check` minimal: `hashseal-core/check` + `blake3` + `std` only (`cargo tree -p hashseal-check --edges normal`).

## Release & Crates.io

- **Crates.io:** `hashseal-core`, `hashseal`, `hashseal-check`, and `hashseal-wasm` are published on [crates.io](https://crates.io/crates/hashseal).
- **Gitea Actions:** [`.gitea/workflows/release-binaries.yml`](../.gitea/workflows/release-binaries.yml) builds `hashseal` + `hashseal-check`, smoke-tests `--version`, packages `.tar.gz`, and writes `SHA256SUMS`.
- **Local helper:** [`scripts/build-release.sh`](../scripts/build-release.sh) builds `--release` binaries and emits `SHA256SUMS`.

```text
Copyright (c) 2026 Nathan A. Chadwick
```
