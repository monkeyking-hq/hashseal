# HashSeal trusted public keys

Public keys only. **Never commit private keys.**

| Identity | File | Role |
|----------|------|------|
| `build-bot@hashseal.ai` | `build-bot@hashseal.ai.pub.asc` | Passwordless CI / local build-bot signing |

## Fingerprint (build-bot)

```text
390D7CA340A36AB538E9D93733D7F44BABBA52EE
```

## Local / CI setup

Paths below are relative to the monorepo root unless noted.

1. **Import the public key** (for verify):

   ```bash
   # Linux / macOS / Git Bash
   gpg --import .hashseal/keys/build-bot@hashseal.ai.pub.asc
   ```

   ```powershell
   # Windows (PowerShell) — same relative path
   gpg --import .hashseal/keys/build-bot@hashseal.ai.pub.asc
   ```

2. **Private key** stays only in the GnuPG keyring used by **git’s** `gpg.program` (never commit it).  
   HashSeal uses the same program git uses for `git commit -S`.

   | | Typical `gpg.program` | Default keyring location |
   |--|------------------------|---------------------------|
   | Linux / macOS | `gpg` (on `PATH`) | `~/.gnupg` |
   | Windows | `gpg` on `PATH`, or e.g. `C:\Program Files\GnuPG\bin\gpg.exe` | `%APPDATA%\gnupg` |

   Check what git is using:

   ```bash
   git config --get gpg.program
   # if empty, git looks for gpg on PATH
   ```

3. **Seal with sign**:

   ```bash
   hashseal seal --instruct --sign --signing-key build-bot@hashseal.ai
   ```

```text
Copyright (c) 2026 MonkeyKing.dev
```
