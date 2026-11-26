---
layout: default
title: Instruct seal format
permalink: /instruct/format/
hashseal: "blake3:e6a54d305b2c957836a878eae44f13acb522f83987ede375d058e1a62e06b5c1"
---

# Instruct seal format (`instruct-v2`)

Format for the **agent instruction file integrity seal**: what was sealed is what agents still read.

## Seal &amp; signature fields

```yaml
---
hashseal: "blake3:<hex>"
hashseal_sig_format: ssh
hashseal_scope: "AGENTS.md"
hashseal_sig: |
  -----BEGIN SSH SIGNATURE-----
  …
  -----END SSH SIGNATURE-----
---
```

Reserved top-level front-matter keys excluded from the hashed payload:
`hashseal` (or configured `document.field`), `hashseal_sig`, `hashseal_sig_format`, `hashseal_signer`, `hashseal_scope`, `hashseal_key_id`.

## Canonical hashing (`instruct-v2`)

1. Strip leading UTF-8 BOM (`U+FEFF`) and normalize `\r\n` / `\r` → `\n` for hashing only.
2. Remove top-level reserved keys and their indented continuation blocks; keep all remaining front-matter lines **verbatim** (preserving YAML lists, nested maps, key order, and comments).
3. Compute BLAKE3 over domain-separated bytes:
   - **`full` (default, with non-empty kept FM):** `HASHSEAL-INSTRUCT-V2\nFM\n<kept_fm_lf>\nBODY\n<body_lf>`
   - **`full` (no non-empty kept FM):** `HASHSEAL-INSTRUCT-V2\nBODY\n<body_lf>`
   - **`body-only`:** `HASHSEAL-INSTRUCT-V2-BODY\n<body_lf>`

## Unicode smuggling hygiene

By default (`unicode_policy: "fail"`), `seal` and `check` reject Unicode Tag (`U+E0000..=U+E007F`), BiDi controls (`U+202A..=U+202E`, `U+2066..=U+2069`, `U+061C`, `U+200E`, `U+200F`), zero-width formatting (`U+200B`, `U+2060..=U+2064`), and embedded mid-file BOMs (`U+FEFF` at index &gt; 0) with `SUSPICIOUS_UNICODE`.

## Vectors

Official cases: [`verify/vectors/instruct-v2.json`](../../verify/vectors/instruct-v2.json) (19 cases).

```text
Copyright (c) 2026 MonkeyKing.dev
```
