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

Do **not** commit a `CNAME` until the custom domain is verified and DNS is ready — a repo `CNAME` forces Pages onto that host and can block the GitHub domain-verification TXT flow.

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
2. Site works at `https://monkeyking-hq.github.io/hashseal/` with no custom domain.
3. **Custom domain (`hashseal.ai`) — order matters:**
   1. In GitHub (org or Pages UI), start domain setup and copy the **TXT** verification record GitHub shows.
   2. Add that TXT at the DNS host; wait for verification to succeed.
   3. Add apex/www records per [GitHub Pages custom domain](https://docs.github.com/pages/configuring-a-custom-domain-for-your-github-pages-site) (A/AAAA or ALIAS/ANAME for apex; CNAME for `www` if used).
   4. In **Settings → Pages**, set Custom domain to `hashseal.ai` and enable **Enforce HTTPS** once DNS is green.
   5. Optionally commit `docs/CNAME` containing a single line `hashseal.ai` so the domain survives branch deploys (or let the Pages UI keep it).
4. When the public GitHub URL is known, set `github_url` in `_config.yml` so the header GitHub link appears.

## Private files

Strategic agent planning lives under **`.hashseal-local/planning/`** (gitignored), not in this tree.

```text
Copyright (c) 2026 MonkeyKing.dev
```
