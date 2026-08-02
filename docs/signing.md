---
hashseal: "blake3:52fdb937b7c8e46f94723ea84eab22a4aff20830d2a9c9f62452536591c1d6cc"
---
# Signing on top of sealing

**Signed, Sealed, Delivered - I'm Yours.**

HashSeal **seals** content with a digest, then optionally **signs** that digest with **GPG using the same settings as git**.

## YAML order

```yaml
---
title: agents
hashseal: "blake3:…"
hashseal_sig: |
  -----BEGIN PGP SIGNATURE-----
  …
  -----END PGP SIGNATURE-----
---
body
```

`hashseal` and `hashseal_sig` are **excluded** from the content hash.

## Signed payload

```text
HASHSEAL-GPG1
digest=blake3:<hex>
```

GPG creates a **detached ASCII-armored** signature of that UTF-8 payload.

## Git alignment

| Git config | HashSeal |
|------------|----------|
| `user.signingKey` | Default `--signing-key` |
| `gpg.program` | GPG binary (default `gpg`) |

If `git commit -S` works, `hashseal seal --instruct --sign` should work.

## CLI

```bash
hashseal seal --instruct --sign
hashseal check --require-signature
```

Local default: unsigned. Release CI: enable sign + require signature.

```text
Copyright (c) 2026 MonkeyKing.dev
```
