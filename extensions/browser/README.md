---
hashseal: "blake3:be4075435cb7db621a6bdff030099ee18552a06bf99b0c8abc9362840456588b"
---
# HashSeal browser extension (Chrome MV3)

Minimal **Manifest V3** extension: paste sealed Markdown, run a **pure JS** instruct check (same algorithm as `verify/js`), show **VALID** or a full **MISMATCH / non-OK list**.

**Signed, Sealed, Delivered - I'm Yours.**

## Install (unpacked)

1. Build / refresh the verify bundle (from monorepo root):

   ```bash
   node extensions/browser/scripts/bundle-from-verify-js.js
   ```

2. Chrome → `chrome://extensions` → enable **Developer mode** → **Load unpacked** → select this folder (`extensions/browser`).

3. Pin **HashSeal** → open the popup → paste Markdown → **Check**.

No network access, no CLI required. Digest check only (FULL canonical); GPG signature is not verified here.

## What it checks

| Scope | Supported |
|-------|-----------|
| Instruct Markdown (`hashseal` field, FULL mode) | Yes — same as `verify/js` + `instruct-v1` |
| Tree ledger / `hashseal-bundle` | No (use CLI or `verify/js` `verifyTreeInMemory` outside the popup) |
| GPG `hashseal_sig` | Not verified in-browser |

## Multi-document paste

- Single paste = one document (`pasted.md`).
- Separate docs with a line containing only `---DOC---` or `===`.
- Or use headers:

  ```text
  # file: AGENTS.md
  ---
  hashseal: "blake3:…"
  ---
  …

  # file: README.md
  ---
  …
  ```

Every non-OK path is listed with status, expected, and actual digests.

## Layout

| Path | Role |
|------|------|
| `manifest.json` | Chrome MV3 |
| `popup.html` / `popup.js` / `popup.css` | Paste + check UI |
| `lib/hashseal-verify.browser.js` | Bundled from `verify/js` **instruct** sources |
| `scripts/bundle-from-verify-js.js` | Regenerate the bundle |
| `icons/` | Toolbar icons (16 / 48 / 128) |

## Reuse of `verify/js`

The browser bundle wraps the same `check.js` + noble BLAKE3 sources used by `@hashseal/verify` for **instruct** documents. Re-run the bundle script after changing instruct verify code in `verify/js` (tree.js is not required for the popup).

## Optional: WASM / native messaging

**Pure JS is enough** for the paste-check popup (instruct FULL digest + in-bundle tree helpers).

| Path | Status |
|------|--------|
| **Pure JS (default)** | `lib/hashseal-verify.browser.js` includes `checkDocumentText`, `hashTreeFileContent`, `verifyTreeInMemory` after re-bundle |
| **WASM** | Core crate `rust/hashseal-wasm` exposes `check_text` (instruct). Build with `wasm-bindgen` when you want a native-speed path; keep pure JS as fallback so Load Unpacked works without a rustc step |
| **Native messaging** | Stub under `native-host/` — register `com.hashseal.native` host to shell out to `hashseal-check` / `hashseal` for full tree / signed verify |

Do not block packaging on WASM or native host.

## Smoke (Node)

```bash
node extensions/browser/scripts/bundle-from-verify-js.js
node -e "const a=require('./extensions/browser/lib/hashseal-verify.browser.js'); console.log(a.checkDocumentText('---\\nhashseal: \"blake3:25280e93176b8b5ae3f4c2dd4b8fef7a20c4a626ea8dfd933b0e77b3a240dccb\"\\n---\\n# Hello\\n\\nAgent rules.\\n'))"
```

Expected: `{ ok: true, status: 'valid', ... }`.

```text
Copyright (c) 2026 MonkeyKing.dev
```
