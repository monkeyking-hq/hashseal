---
hashseal: "blake3:67b0ac5d44fd26c34408b310faa07464e31a3f6615fd71ec56350ebe8897f9a4"
---
# HashSeal CLI reference

**Signed, Sealed, Delivered - I'm Yours.**

## Binaries

| Binary | Crate | Role |
|--------|--------|------|
| `hashseal` | `hashseal` | Full CLI: `init`, `seal`, `check`, `verify`, `unseal`, `clean` |
| `hashseal-check` | `hashseal-check` | Tiny instruct-only verify (`blake3` + `std` only) |

## Config (`.hashseal.json`)

JSON only (no TOML crate). Defaults plus optional root overlay `.hashseal.json` (or `--config PATH` / `--overlay PATH`). Example: [`config/examples/hashseal.mvp.json`](../config/examples/hashseal.mvp.json).

| Key | Meaning |
|-----|---------|
| `document.enable` | Seal/check instruct files (`true`) |
| `document.include` | Globs for instruct files (`DEFAULT_DOCUMENT_INCLUDES`: `AGENTS.md`, `CLAUDE.md`, `SKILL.md`, `.cursor/rules/*.mdc`, Copilot/Windsurf/Cline/agent dirs) |
| `document.exclude` | Globs to skip (includes `target`, `node_modules`, `worktrees`, `.worktrees`) |
| `document.canonical` | `full` (`instruct-v2` lossless raw-FM) or `body-only` |
| `document.field` | Seal field name (default `hashseal`) |
| `document.auto_frontmatter` | Create front matter if absent (`true`) |
| `document.algorithm` | `blake3` (default) or `sha256` |
| `document.unicode_policy` | `fail` (default), `warn`, or `off` |
| `signing.enable` / `signing.require` | Sign on seal / require signature on check |
| `signing.format` | `auto` (default, from `git config gpg.format`), `ssh`, or `gpg` |
| `signing.signing_key` | SSH key path or GPG key id/fingerprint |
| `signing.gpg_program` | GPG binary override |
| `signing.allowed_signers` | OpenSSH `allowed_signers` path (default `.hashseal/allowed_signers`) |
| `signing.allowed_key_ids` | GPG fingerprint/key-ID allowlist |
| `enforce.on_tree_mismatch` / `on_doc_mismatch` / `on_missing_ledger` | `fail` (default) or `warn` |
| `tree.include` / `tree.exclude` / `tree.line_endings` / `tree.algorithm` / `tree.ledger` / `tree.report` | Tree ledger walk &amp; paths |
| `report.write` / `pretty` / `include_ci` / `include_hostname` / `max_findings` | JSON audit report options |

Walks always skip `worktrees` and `.worktrees` directory names so nested agent worktrees are never crawled.

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
