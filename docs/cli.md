---
hashseal: "blake3:d13425edc7f06267692cc81185456d2fd2c59843a2b49d70865d26d812b463b3"
---
# HashSeal CLI reference

**Signed, Sealed, Delivered - I'm Yours.**

## Binaries

| Binary | Crate | Role |
|--------|--------|------|
| `hashseal` | `hashseal` | Full seal / check / verify / clean / release |
| `hashseal-check` | `hashseal-check` | Tiny instruct-only verify (blake3 path) |

```bash
cargo build -p hashseal --release
cargo build -p hashseal-check --release
cargo run -p hashseal -- --help
```

## Config

JSON only (no TOML crate). Defaults plus optional root overlay:

- `.hashseal.json` in the project root
- `--config PATH` / `--overlay PATH` on seal/check/verify

Example: [`config/examples/hashseal.mvp.json`](../config/examples/hashseal.mvp.json).

Relevant keys:

| Path | Meaning |
|------|---------|
| `document.enable` | Seal/check instruct files |
| `document.include` | Globs for instruct files (see [default instruct includes](#default-instruct-includes)) |
| `document.exclude` | Globs to skip (defaults match tree excludes: `target`, `node_modules`, agent `worktrees` / `.worktrees`, …) |
| `document.canonical` | `full` (default) or `body-only` |
| `document.field` | Seal field name (default `hashseal`) |
| `tree.include` / `tree.exclude` | Tree seal walk |
| `signing.enable` / `signing.require` | GPG on seal / require on check |
| `report.write` | Write report JSON next to ledger |

### Default instruct includes

Defaults target **agent instruction surfaces**, not every Markdown file. README and general docs are **not** included unless you add them.

| Category | Examples |
|----------|----------|
| Ambient context | `AGENTS.md`, `AGENTS.local.md`, `AGENT.md`, `CLAUDE.md`, `GEMINI.md`, `QWEN.md`, `CODEX.md`, `GROK.md`, `CONVENTIONS.md` |
| Copilot | `.github/copilot-instructions.md`, `.github/instructions/**/*.md`, `.github/agents/**/*.md`, `.github/prompts/**/*.md`, `.github/skills/**/*.md` |
| Cursor / Windsurf / Cline / Continue | `.cursorrules`, `.cursor/rules/**/*.{md,mdc}`, `.cursor/skills/**/*.md`, `.windsurfrules`, `.windsurf/**/*.md`, `.clinerules`, `.clinerules/**/*.md`, `.continue/rules/**/*.md` |
| Skills entrypoint | `**/SKILL.md` (project packs and agent skill dirs) |
| Agent CLI/IDE skill & command dirs | `.agents/`, `.claude/`, `.gemini/`, `.grok/`, `.kilo/`, `.augment/`, and other common tool roots (Markdown under those trees) |

Full default list: `DEFAULT_DOCUMENT_INCLUDES` in `hashseal-core` (`rust/hashseal-core/src/config.rs`) and the example overlay [`config/examples/hashseal.mvp.json`](../config/examples/hashseal.mvp.json).

**Customize:** set `document.include` and/or `document.exclude` in `.hashseal.json` — replace includes entirely, or narrow/expand relative to your workflow. To seal *all* Markdown again: `"include": ["**/*.md"]`.

### Default walk skips (performance)

Directory basenames **`worktrees`** and **`.worktrees`** are never entered when walking (same class of hard skip as `target` / `node_modules`). That covers Claude Code’s `.claude/worktrees/<name>/` nested checkouts, root `.worktrees/`, and similar agent parallel-worktree layouts. Without this, instruct/tree scans re-crawl full nested trees and can hang for minutes.

## Commands

### `hashseal version`

Prints CLI + core version and product tagline.

### `hashseal seal`

Seal instruct files and/or a tree ledger.

```bash
hashseal seal --instruct [--sign] [--root DIR] [--config PATH]
hashseal seal --tree [--root DIR]
hashseal seal --release [--artifact PATH]...   # tree + hashseal-bundle/
hashseal seal --instruct --tree --release
```

| Flag | Effect |
|------|--------|
| `--instruct` | Seal matching instruct (Markdown) files in place |
| `--tree` | Write tree ledger |
| `--release` | Tree seal into `hashseal-bundle/` (ledger + report + MANIFEST) |
| `--sign` | GPG-sign instruct digests (`hashseal_sig`); uses git GPG settings |
| `--signing-key KEY` | Override signing key id |
| `--root DIR` | Project root (default `.`) |
| `--ledger PATH` | Ledger path override |
| `--artifact PATH` | Extra artifact digests in the release bundle |
| `--format human\|json` | Output style |

If none of `--instruct` / `--tree` / `--release` are set, behavior follows config (typically tree + instruct when document enable is true). Prefer explicit flags for scripts.

### `hashseal check`

Fast instruct-file digest check. Lists **every** non-OK path with status and digests.

```bash
hashseal check [--root DIR] [PATH...]
hashseal check --require-signature
hashseal check --no-fail
hashseal check --format json
```

| Flag | Effect |
|------|--------|
| `PATH...` | Files to check (default: walk `document.include`) |
| `--require-signature` | Fail if `hashseal_sig` missing / bad |
| `--no-fail` | Always exit 0 (still print findings) |

Exit codes: `0` ok, `1` findings, `2` usage, `3` hard error.

### `hashseal verify`

Verify a tree ledger (and optionally a release bundle).

```bash
hashseal verify [--root DIR] [--ledger PATH] [--bundle DIR]
hashseal verify --no-fail
```

### `hashseal unseal --instruct`

Strip `hashseal` / `hashseal_sig` / `hashseal_key_id` from front matter.

```bash
hashseal unseal --instruct [--root DIR] [PATH...]
```

### `hashseal clean`

Remove ledger / report / `hashseal-bundle` artifacts under `--root`.

## `hashseal-check` (tiny binary)

Manual argv only (no clap). Instruct check with the same failure listing UX.

```bash
hashseal-check [--root DIR] [--no-fail] [--require-signature] [PATH...]
hashseal-check -V
hashseal-check -h
```

Dependency policy: `hashseal-core` with `check` feature → **blake3** only (plus std).

## Instruct seal field

```yaml
---
hashseal: "blake3:<hex>"
# optional:
hashseal_sig: |
  -----BEGIN PGP SIGNATURE-----
  …
  -----END PGP SIGNATURE-----
---
```

Chicken-and-egg: `hashseal`, `hashseal_sig`, and `hashseal_key_id` are **excluded** from the hashed payload. See [document-seal.md](./document-seal.md) and [signing.md](./signing.md).

## Failure UX

On mismatch / missing seal, tools print each path:

```text
HashSeal check failed: 1 issue(s)

  MISMATCH  path/to/AGENTS.md
            expected: blake3:…
            actual:   blake3:…
```

Never silent exit-only fails for content problems.

## MVP smoke

See [`fixtures/mvp-demo/README.md`](../fixtures/mvp-demo/README.md).

```text
Copyright (c) 2026 MonkeyKing.dev
```
