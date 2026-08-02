# HashSeal public site (`docs/`)

GitHub Pages source for **[hashseal.ai](https://hashseal.ai)**.

## Layout

| Path | Role |
|------|------|
| `_layouts/default.html` | HTML shell |
| `_includes/header.html`, `footer.html` | Chrome |
| `assets/css/site.css` | Single stylesheet (no remote theme) |
| `assets/logo/` | Mark, wordmark, **Verified by HashSeal** SVGs |
| `index.md` | Marketing home |
| `instruct/`, `build/` | Product docs hubs |
| `CNAME` | `hashseal.ai` |

## Local preview

With Ruby + Bundler (optional):

```bash
# from repo root
cd docs
# if needed: gem install bundler jekyll
jekyll serve --baseurl ""
```

Or open the Markdown as source and rely on GitHub’s Pages build after push.

## Deploy

1. Repo **Settings → Pages → Build from branch**, folder **`/docs`**.
2. Custom domain: `hashseal.ai` (CNAME file is already present).
3. DNS at the registrar: point apex (and optional `www`) per GitHub Pages docs; enable HTTPS.
4. When the public GitHub URL is known, set `github_url` in `_config.yml` so the header GitHub link appears.

## Private files

Strategic agent planning lives under **`.hashseal-local/planning/`** (gitignored), not in this tree.

```text
Copyright (c) 2026 MonkeyKing.dev
```
