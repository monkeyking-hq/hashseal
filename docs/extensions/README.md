---
hashseal: "blake3:09edf23f7c68035cef2b716f360012c3824d89101c17d933fb480ea1d37126fe"
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
Copyright (c) 2026 Nathan A. Chadwick
```
