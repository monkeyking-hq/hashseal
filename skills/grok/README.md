---
hashseal: "blake3:7158998d4c60665595e31b2a02e42618f06d64193f638c1788284ecf9d9b6083"
---
# HashSeal skill — Grok

Grok / xAI oriented skill pack. Primary document: [`SKILL.md`](./SKILL.md).

**Signed, Sealed, Delivered - I'm Yours.**

Teaches agents to:

- Treat sealed instruct files as authoritative
- Re-seal after intentional edits (`hashseal seal --instruct`)
- Run `hashseal check` / `hashseal-check` before ship
- **Never strip** `hashseal` / `hashseal_sig` / `hashseal_key_id`
- List every non-OK path with digests on failure

See also monorepo [`skills/README.md`](../README.md).

```text
Copyright (c) 2026 MonkeyKing.dev
```
