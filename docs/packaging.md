---
hashseal: "blake3:acb6f939f69c04989ddcbdb23e88ae17819f8d591ffa0cd36ec07df7ed11cb34"
---
# Packaging notes

**Signed, Sealed, Delivered - I'm Yours.**

## Goals

1. Ship platform binaries (`hashseal`, `hashseal-check`) with `SHA256SUMS` verification.
2. Keep Tier-1 verify SDKs (`verify/js`, `verify/python`, `verify/go`) zero-external-dep.
3. Keep `hashseal-check` minimal: `hashseal-core/check` + `blake3` + `std` only (`cargo tree -p hashseal-check --edges normal`).

## Release workflow

- Gitea Actions: [`.gitea/workflows/release-binaries.yml`](../.gitea/workflows/release-binaries.yml) builds `hashseal` + `hashseal-check`, smoke-tests `--version`, packages `.tar.gz`, and writes `SHA256SUMS`.
- Local helper: [`scripts/build-release.sh`](../scripts/build-release.sh) builds `--release` binaries and emits `SHA256SUMS`.

```text
Copyright (c) 2026 MonkeyKing.dev
```
