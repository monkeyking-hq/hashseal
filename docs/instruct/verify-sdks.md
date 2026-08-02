---
layout: default
title: Verify SDKs
permalink: /instruct/verify-sdks/
---

# Verify SDKs (zero-dep)

In-process instruct (and tree, where implemented) check **without** the CLI. Prefer these when embedding HashSeal in hosts, tests, or language-native CI.

| Language | Path | Instruct vectors | Notes |
|----------|------|------------------|--------|
| JavaScript | `verify/js` | `node test/vectors.test.js` | Also powers browser extension bundle |
| Python | `verify/python` | `python test/test_vectors.py` | |
| Java | `verify/java` | see README | |
| Go | `verify/go` | `go test .` / `go run ./test/` | |
| Ruby | `verify/ruby` | `ruby test/run_vectors.rb` | |
| .NET | `verify/dotnet` | `dotnet run --project Hashseal.Verify.Test` | |

Shared vectors (repo root relative):

- Instruct: `verify/vectors/instruct-v1.json` (16 cases)
- Tree: `verify/vectors/tree-v1.json` (12 cases; in-memory ports)

Policy: **no package-manager blake3 dependency** for the pure paths — vendor minimal pure implementations under each `verify/*/vendor` as needed.

Also:

- **WASM:** `rust/hashseal-wasm` — `check_text` for IDE / browser bridges  
- **Tiny binary:** `hashseal-check` — same instruct check, blake3-only native binary  

Back to [Agent instruction file integrity seal](./).

```text
Copyright (c) 2026 MonkeyKing.dev
```
