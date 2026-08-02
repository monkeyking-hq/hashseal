---
layout: default
title: Instruct seal format
permalink: /instruct/format/
hashseal: "blake3:6f7640fcaf0067cfd859b97b7b16f5cc4bfeb1970158d7562febd371116213cc"
---

# Instruct seal format (Markdown)

Format for the **agent instruction file integrity seal**: what was sealed is what agents still read.

## Seal field

YAML front matter:

```yaml
---
hashseal: "blake3:<hex>"
hashseal_sig: |
  -----BEGIN PGP SIGNATURE-----
  …
  -----END PGP SIGNATURE-----
---
```

Signature is optional; see [signing](../signing.md).

## Chicken-and-egg

The digest is computed over **canonical content with seal and signature fields excluded** (`hashseal`, `hashseal_sig`, `hashseal_key_id`). Updating the seal/sig does not change the hashed payload.

**YAML order when both present:** `hashseal` then `hashseal_sig` (armor).

## Canonical modes

| Mode | Hash input |
|------|------------|
| `full` (default) | Sorted front-matter keys (minus seal fields) as `key: value\n` lines, then `\n`, then LF-normalized body. If no non-seal FM keys, body only. |
| `body-only` | LF-normalized body only |

## CLI

```bash
hashseal seal --instruct
hashseal check
hashseal check --no-fail
hashseal unseal --instruct
```

## Verify UX

Failures list every path with `MISMATCH` / `MISSING_SEAL` / … and expected vs actual digests.

## Vectors

Official cases: [`verify/vectors/instruct-v1.json`](../../verify/vectors/instruct-v1.json) (FULL canonical mode). All language SDKs and core tests must agree.

```text
Copyright (c) 2026 MonkeyKing.dev
```
