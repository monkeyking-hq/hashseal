---
hashseal: "blake3:650e113b958d57b6ad1452131b47062d5a6778f7f660addf063baf1e2da4c014"
---
# HashSeal MVP demo fixture

Minimal project used for human smoke testing of `instruct-v2` + `signing-v2` + `.hashseal.lock` + release bundle verification.

## Layout

```text
fixtures/mvp-demo/
  AGENTS.md          # sample instruct file
  src/hello.txt      # sample tree content
  .hashseal.json     # local overlay (instruct: AGENTS.md only)
  README.md          # this file
```

## 5-minute smoke (from monorepo root)

```bash
# Build release CLI binaries
cargo build -p hashseal -p hashseal-check --release

# Seal instruct files (writes hashseal front matter into AGENTS.md)
./target/release/hashseal seal --instruct --lock --root fixtures/mvp-demo

# Seal tree + write integrity bundle
./target/release/hashseal seal --tree --release --root fixtures/mvp-demo

# Check instruct digests + lockfile + Unicode hygiene
./target/release/hashseal check --root fixtures/mvp-demo
./target/release/hashseal-check --root fixtures/mvp-demo

# Verify tree ledger + bundle MANIFEST.json
./target/release/hashseal verify --root fixtures/mvp-demo

# JS zero-dep check (Node 16+)
node -e "const fs=require('fs');const {checkDocumentText}=require('./verify/js');const r=checkDocumentText(fs.readFileSync('fixtures/mvp-demo/AGENTS.md','utf8'));console.log(r);process.exit(r.ok?0:1)"
```

## Bootstrap hooks &amp; sign-on-seal (SSH or GPG)

```bash
# Initialize config, hooks, and lockfile in a project:
hashseal init --claude-hooks --git-hooks --lock --root fixtures/mvp-demo

# Sign using git's gpg.format (ssh or openpgp):
hashseal seal --instruct --sign --lock --root fixtures/mvp-demo
hashseal check --require-signature --root fixtures/mvp-demo
```

## Clean artifacts

```bash
./target/release/hashseal clean --root fixtures/mvp-demo
```

```text
Copyright (c) 2026 MonkeyKing.dev
```
