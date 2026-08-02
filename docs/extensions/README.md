---
hashseal: "blake3:32acacc8f1bff2ec4f83aa724b9303bae2ecfe617eb656833f79bff6cb2b83bc"
---
# HashSeal IDE / browser extensions

| Path | Status |
|------|--------|
| [`extensions/vscode`](../../extensions/vscode) | Skeleton — spawn `hashseal` / `hashseal-check` |
| [`extensions/browser`](../../extensions/browser) | Chrome MV3 — paste Markdown, pure JS check (bundled `verify/js`) |
| [`extensions/zed`](../../extensions/zed) | Stub — Zed tasks.json + README (CLI shell-out) |
| [`extensions/antigravity`](../../extensions/antigravity) | Stub — agent host command map + README |

Install CLI: [`docs/install.md`](../install.md).

All IDE stubs prefer **`hashseal-check`** for workspace check and **`hashseal`** for seal/verify when the full CLI is required.

Browser extension needs no CLI; regenerate bundle after `verify/js` instruct changes:

```bash
node extensions/browser/scripts/bundle-from-verify-js.js
```

```text
Copyright (c) 2026 MonkeyKing.dev
```

