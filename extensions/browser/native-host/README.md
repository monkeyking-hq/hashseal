---
hashseal: "blake3:239971296c13cf5f4f7be5797bc96db2c0ddf8dc638cd84cc77e9182a83b5f62"
---
# Optional native messaging host

**Default browser check does not need this** — pure JS in the popup is enough for instruct seals.

Wire a native host only when you want the extension to call **`hashseal-check` / `hashseal`** for tree verify or signed bundles.

## Manifest

Copy `com.hashseal.native.json` and set:

- `path` — absolute path to a host executable (script that reads Chrome native messaging frames and runs CLI)
- `allowed_origins` — your extension ID after load unpacked

Register per Chrome docs for your OS (registry on Windows, JSON under `NativeMessagingHosts` on macOS/Linux).

## Host contract (suggested)

stdin/stdout Chrome native messaging protocol. Request JSON examples:

```json
{ "cmd": "check", "root": "C:/path/to/project" }
{ "cmd": "check_file", "path": "C:/path/to/AGENTS.md" }
```

Response:

```json
{ "ok": true, "exitCode": 0, "stdout": "..." }
```

Not implemented in-repo — this folder is a **stub** so packaging can grow without redesign.

```text
Copyright (c) 2026 MonkeyKing.dev
```
