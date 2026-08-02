---
hashseal: "blake3:d3bff8949c85239af3e9ce7e30cdb90ecf2bb17fa37d756f666d538e53fa49e4"
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
