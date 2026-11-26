---
hashseal: "blake3:d387659cd514ec8dfc751911bde79d4283046da9232eb887ea63859454bfe3dc"
---
# HashSeal IDE / browser extensions

| Path | Status |
|------|--------|
| [`extensions/vscode`](../../extensions/vscode) | DiagnosticsCollection, StatusBarItem, check-on-save, single-file `checkActiveFile` |
| [`extensions/browser`](../../extensions/browser) | Chrome MV3 — paste Markdown, pure JS `instruct-v2` + Unicode check (`verify/js`) |
| [`extensions/zed`](../../extensions/zed) | Zed `tasks.json` (CLI shell-out) |
| [`extensions/antigravity`](../../extensions/antigravity) | Agent host command map |

Regenerate browser bundle after `verify/js` changes:

```bash
node extensions/browser/scripts/bundle-from-verify-js.js
```

```text
Copyright (c) 2026 MonkeyKing.dev
```
