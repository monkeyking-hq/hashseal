---
hashseal: "blake3:4d7721a7e9d6bf5e246b41245565adaf24cd914d62d174887cbf7b57230bd8e2"
---
# HashSeal CLI reference

**Signed, Sealed, Delivered - I'm Yours.**

## Binaries

| Binary | Crate | Role |
|--------|--------|------|
| `hashseal` | `hashseal` | Full CLI: `init`, `seal`, `check`, `verify`, `unseal`, `clean` |
| `hashseal-check` | `hashseal-check` | Tiny instruct-only verify (`blake3` + `std` only) |

## Commands

### `hashseal init`
Bootstrap `.hashseal.json`, `.hashseal/allowed_signers`, hooks, and `.hashseal.lock`:
```bash
hashseal init [--root DIR] [--git-hooks] [--in-git-dir] [--claude-hooks] [--lock] [--sign]
```

### `hashseal seal`
```bash
hashseal seal --instruct [--sign] [--sign-format auto|ssh|gpg] [--portable] [--lock] [--unicode-policy fail|warn|off] [--root DIR]
hashseal seal --tree [--release] [--sign] [--artifact PATH]... [--root DIR]
```

### `hashseal check`
```bash
hashseal check [--root DIR] [--require-signature] [--digest-only] [--lock PATH | --no-lock] [--diff BASE_REF] [--allowed-signers PATH] [--allowed-key ID]... [--unicode-policy fail|warn|off] [--no-fail] [--format human|json] [PATH...]
```
- Auto-detects `.hashseal.lock` at `--root` (unless `--no-lock`, `--diff`, or explicit `PATH...` args are given).
- Exit codes: `0` ok, `1` findings (`MISMATCH`, `MISSING_SEAL`, `MISSING_SIGNATURE`, `BAD_SIGNATURE`, `UNTRUSTED_KEY`, `UNVERIFIED_SIGNATURE`, `SUSPICIOUS_UNICODE`, `LOCKED_FILE_REMOVED`, `UNAPPROVED_FILE`), `2` usage, `3` hard error.

### `hashseal verify`
Verify tree ledger and `hashseal-bundle/` (`MANIFEST.json`, `ledger_digest`, `--artifact` digests, and optional `MANIFEST.json.sig`):
```bash
hashseal verify [--root DIR] [--bundle DIR] [--ledger PATH] [--require-signature] [--digest-only] [--no-fail] [--format human|json]
```

### `hashseal unseal --instruct` / `hashseal clean`
```bash
hashseal unseal --instruct [--root DIR] [PATH...]
hashseal clean [--root DIR]
```

```text
Copyright (c) 2026 MonkeyKing.dev
```
