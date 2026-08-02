---
hashseal: "blake3:b5a885de3072e4d6b363d047e5a59cb8ed1b2a12e6dd9b11ba04a1ee08d1cf17"
---
# HashSeal MVP demo fixture

Minimal project used for human smoke testing of instruct + tree seal.

## Layout

```text
fixtures/mvp-demo/
  AGENTS.md          # sample instruct file (sealed in place by demo commands)
  src/hello.txt      # sample tree content
  .hashseal.json     # local overlay (instruct: AGENTS.md only)
  README.md          # this file
```

## 5-minute smoke (from monorepo root)

```bash
# Build CLI
cargo build -p hashseal --release

# Seal instruct files (writes hashseal front matter into AGENTS.md)
cargo run -p hashseal -- seal --instruct --root fixtures/mvp-demo

# Seal tree + write integrity bundle
cargo run -p hashseal -- seal --tree --release --root fixtures/mvp-demo

# Check instruct digests
cargo run -p hashseal -- check --root fixtures/mvp-demo
cargo run -p hashseal-check -- --root fixtures/mvp-demo

# Verify tree ledger (from bundle)
cargo run -p hashseal -- verify --root fixtures/mvp-demo

# JS zero-dep check (Node 16+)
node -e "const fs=require('fs');const {checkDocumentText}=require('./verify/js');const r=checkDocumentText(fs.readFileSync('fixtures/mvp-demo/AGENTS.md','utf8'));console.log(r);process.exit(r.ok?0:1)"
```

## Tamper demo

```bash
# After sealing, change a body line in AGENTS.md, then:
cargo run -p hashseal -- check --root fixtures/mvp-demo
# Expect MISMATCH with expected vs actual digests listed
```

## Optional GPG sign-on-seal

Requires `gpg` and git signing config (same key as `git commit -S`):

```bash
cargo run -p hashseal -- seal --instruct --sign --root fixtures/mvp-demo
cargo run -p hashseal -- check --require-signature --root fixtures/mvp-demo
```

## Clean artifacts

```bash
cargo run -p hashseal -- clean --root fixtures/mvp-demo
```

```text
Copyright (c) 2026 MonkeyKing.dev
```
