---
layout: default
title: Signed manifests &amp; bundles
permalink: /build/
---

# Signed manifests &amp; release bundles

Secondary HashSeal capability: **tree ledgers and verifiable release bundles** (`hashseal-bundle/`) sharing the same `hashseal-core` engine.

- **`hashseal seal --tree --release`**: writes `ledger.json`, `report.json`, and `MANIFEST.json` (with `ledger_digest`, `report_digest`, and `--artifact` digests). Add `--sign` to sign `MANIFEST.json` (`MANIFEST.json.sig`).
- **`hashseal verify`**: verifies the tree ledger, `MANIFEST.json` `ledger_digest`, every `--artifact` digest, and `MANIFEST.json.sig` (when present or `--require-signature`).

```bash
hashseal seal --tree --release --sign --artifact dist/app.tar.gz --root .
hashseal verify --root . --require-signature
```

Official tree vectors: [`verify/vectors/tree-v1.json`](../../verify/vectors/tree-v1.json) (12 cases).

```text
Copyright (c) 2026 Nathan A. Chadwick
```
