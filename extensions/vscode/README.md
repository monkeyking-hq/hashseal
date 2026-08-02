---
hashseal: "blake3:0ac7710593d40fb96f0e15707d91b601491b27a096628cee3886784bc993d3ac"
---
# HashSeal for VS Code

Minimal extension that runs **`hashseal`** / **`hashseal-check`** on the workspace.

**Signed, Sealed, Delivered - I'm Yours.**

## Requirements

- VS Code 1.85+
- HashSeal CLI on **PATH**, or configure:

| Setting / env | Purpose |
|---------------|---------|
| `hashseal.bin` / `HASHSEAL_BIN` | Full CLI |
| `hashseal.checkBin` / `HASHSEAL_CHECK_BIN` | Tiny check binary |
| `hashseal.preferCheckBinary` | Default `true` — workspace check uses `hashseal-check` |

Build from monorepo:

```bash
cargo build -p hashseal --release
cargo build -p hashseal-check --release
# add target/release to PATH
```

See [`docs/install.md`](../../docs/install.md).

## Commands

| Command | Action |
|---------|--------|
| **HashSeal: Check Workspace** | `hashseal-check --root <workspace>` (or `hashseal check`) |
| **HashSeal: Check Active File** | Notes active path, then workspace check (CLI lists every non-OK path) |
| **HashSeal: Seal Instruct (Workspace)** | `hashseal seal --instruct --root <workspace>` |

Output appears in the **HashSeal** output channel.

## F5 debug (Extension Development Host)

1. Open this folder (`extensions/vscode`) **or** the monorepo root in VS Code.
2. Ensure `hashseal` / `hashseal-check` are on PATH in the environment that launches VS Code.
3. Run **Developer: Install Extension from Location…** and pick `extensions/vscode`,  
   **or** create a launch config (below) and press **F5**.

### Optional `.vscode/launch.json` (in monorepo root)

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "HashSeal Extension",
      "type": "extensionHost",
      "request": "launch",
      "args": ["--extensionDevelopmentPath=${workspaceFolder}/extensions/vscode"]
    }
  ]
}
```

No `npm install` required — **zero** runtime dependencies (`extension.js` uses Node built-ins + VS Code API only).

## Notes

- WASM in-process verify is deferred; this skeleton always shells out.
- Not published to the Marketplace from agent builds unless requested.

```text
Copyright (c) 2026 MonkeyKing.dev
```
