---
hashseal: "blake3:559bf05ca47677b4eb4d8a0550abf5a5f907b51f84036c8b4c12b867bed86ee6"
---
# Signing on top of sealing (`signing-v2`)

**Signed, Sealed, Delivered - I'm Yours.**

HashSeal seals content with BLAKE3, then signs the digest + context scope using **OpenSSH `SSHSIG`** or **hardened GPG**, aligned with your git config.

## Signed payload (`HASHSEAL-SIG-V2`)

```text
HASHSEAL-SIG-V2
digest=blake3:<hex>
scope=<repo-relative-posix-path | *>
```

- **Default (`scope=<rel-path>`)**: prevents copying a signed harmless file over `AGENTS.md` or another instruction file.
- **Portable (`--portable`, `scope=*`)**: for distributable third-party `SKILL.md` packs verified at any path.

## Backends &amp; trust roots

| Backend | Git config | Trust root |
|---|---|---|
| **SSH (`SSHSIG`)** | `gpg.format = ssh` | `.hashseal/allowed_signers` (or `--allowed-signers` / `gpg.ssh.allowedSignersFile`) |
| **Hardened GPG** | `gpg.format = openpgp` (or unset) | Isolated keyring from `.hashseal/keys/*.asc` + `--status-fd 1` + `allowed_key_ids` |

Override backend explicitly with `--sign-format ssh|gpg|auto`.

## Fail-closed verification

When `hashseal_sig` is present, `hashseal check` verifies it by default and fails closed with `UNVERIFIED_SIGNATURE` if the trust root or signing tool is missing. Pass `--digest-only` to opt out explicitly.

```bash
hashseal seal --instruct --sign --lock --root .
hashseal check --root . --require-signature
hashseal check --diff origin/main --require-signature
```

{% include demo.html name="ssh-signing" alt="HashSeal SSH signing and cross-file replay protection demo" %}

```text
Copyright (c) 2026 Nathan A. Chadwick
```
