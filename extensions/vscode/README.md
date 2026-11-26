---
hashseal: "blake3:b67e1ad0e4fd001b33faa5eacea9492a879217b5ee6d1437153211eb4cd9b81c"
---
# HashSeal for VS Code

VS Code extension (`v0.6.1`) providing inline diagnostics, status bar indicator, check-on-save, and single-file checking via the **`hashseal`** CLI.

**Signed, Sealed, Delivered - I'm Yours.**

## Features

- **`DiagnosticCollection` ("hashseal")**: maps every non-OK finding (`MISMATCH`, `SUSPICIOUS_UNICODE`, `BAD_SIGNATURE`, `UNVERIFIED_SIGNATURE`, etc.) directly onto the affected file.
- **Status bar indicator**: displays `HashSeal: N OK` or `HashSeal: N issue(s)` (click to re-run workspace check).
- **Check on save**: automatically checks instruction/Markdown files on save (`hashseal.checkOnSave: true`).
- **Single-file check**: `HashSeal: Check Active File` checks only the active document (`hashseal check --format json --root <root> <file>`).

## Configuration

| Setting / env | Default | Purpose |
|---|---|---|
| `hashseal.bin` / `HASHSEAL_BIN` | `hashseal` on `PATH` | Full CLI binary |
| `hashseal.checkBin` / `HASHSEAL_CHECK_BIN` | `hashseal-check` on `PATH` | Tiny check binary |
| `hashseal.preferCheckBinary` | `false` | Prefer plain-text `hashseal-check` over JSON diagnostics |
| `hashseal.checkOnSave` | `true` | Automatically check candidate files on save |
| `hashseal.requireSignature` | `false` | Pass `--require-signature` during checks |

```text
Copyright (c) 2026 MonkeyKing.dev
```
