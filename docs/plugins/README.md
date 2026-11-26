---
hashseal: "blake3:c72c15371232f80c5e84d18a95944d2848e6af5a8437f50d7a87a0f981196a66"
---
# HashSeal CLI wrappers &amp; integrations

Thin wrappers invoke the **`hashseal` CLI** (`PATH` or `HASHSEAL_BIN`); all integrity logic lives in `hashseal-core`.

| Integration | Path | Role |
|---|---|---|
| Git / Claude hooks | `hashseal init` | Bootstraps `.githooks/pre-commit`, `.claude/settings.json`, `.hashseal.lock` |
| pre-commit framework | [`.pre-commit-hooks.yaml`](../../.pre-commit-hooks.yaml) | `hashseal-check` and `hashseal-check-signed` hooks |
| Gitea / GitHub Action | [`action/action.yml`](../../action/action.yml) | Composite PR/CI check (`diff-base`, `require-signature`, `lock`) |
| npm wrapper | [`plugins/npm`](../../plugins/npm) | `hashseal-npm` CLI wrapper |
| Cargo aliases | [`plugins/cargo`](../../plugins/cargo) | Cargo alias config + docs |

For in-process verification without the CLI, use the Tier-1 SDKs under `verify/js`, `verify/python`, or `verify/go`.

```text
Copyright (c) 2026 MonkeyKing.dev
```
