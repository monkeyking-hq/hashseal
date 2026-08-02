---
layout: default
title: HashSeal
permalink: /
home: true
description: Seal and verify AI instruction files and build trees — what was sealed is what you still have.
---

<section class="hero">
  <img class="hero__mark" src="{{ '/assets/logo/mark.svg' | relative_url }}" width="88" height="88" alt="HashSeal mark — golden circlet over a blue seal" />
  <h1>What was sealed is what you still have</h1>
  <p class="hero__tagline">Signed, Sealed, Delivered - I'm Yours.</p>
  <p class="hero__lead">
    Integrity tooling for agent instruction files and source trees.
    When something drifts, verify names <strong>every</strong> non-OK path with status and digests — never a silent fail.
  </p>
  <div class="hero__actions">
    <a class="btn btn--primary" href="{{ '/install.html' | relative_url }}">Install</a>
    <a class="btn btn--ghost" href="{{ '/instruct/' | relative_url }}">Instruction seals</a>
  </div>
  <div class="hero__verified" title="Status mark used when a check passes">
    <img src="{{ '/assets/logo/verified.svg' | relative_url }}" width="20" height="20" alt="" />
    <span>Verified by HashSeal</span>
  </div>
</section>

<section class="section">
  <h2>Two Paths</h2>
  <div class="grid-2">
    <article class="card card--lead">
      <span class="lead-label">Sealed, Verifiable, Agent Instruction Files</span>
      <h3><a href="{{ '/instruct/' | relative_url }}">Agent instruction file integrity sealing</a></h3>
      <p>
        Seal <code>AGENTS.md</code>, skills, and other AI instruction files with a [BLAKE3 digest](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3 "Wikipedia link")
        with optional GPG via git’s signing credentials. The idea is simple; Check instruction file integrity before sending them to models or agents to run.  Sign your instructions so that your users can verify and trust that they are exactly what you intended to deliver.
      </p>
    </article>
    <article class="card">
      <span class="lead-label">Secure your CI and Build Chains</span>
      <h3><a href="{{ '/build/' | relative_url }}">Multi Language build system tools to sign and seal your entire build system</a></h3>
      <p>
        Tree ledgers, release bundles (<code>hashseal-bundle/</code>), and thin plugins
        so CI and multi-agent builds cannot silently rewrite what you ship.  Stop multi-agent workflows from accidentally altering build chains mid-stream. Ship builds with artifacts that can be verified by anyone.
      </p>
    </article>
  </div>
</section>

<section class="section">
  <h2>How it works</h2>
  <ol class="steps">
    <li>
      <div>
        <strong>Seal</strong>
        <span>Canonicalize content, write <code>hashseal: "blake3:…"</code> (instruct) or a tree ledger (build).</span>
      </div>
    </li>
    <li>
      <div>
        <strong>Ship or hand off</strong>
        <span>Commit, release, or pass instructions to an agent host — the seal travels with the files.</span>
      </div>
    </li>
    <li>
      <div>
        <strong>Check</strong>
        <span>Recompute digests. Failures list every path that drifted, with expected vs actual digests.</span>
      </div>
    </li>
  </ol>
</section>

<section class="section">
  <h2>Quickstart</h2>
  <pre><code>cargo build -p hashseal --release
cargo run -p hashseal -- seal --instruct --root fixtures/mvp-demo
cargo run -p hashseal -- check --root fixtures/mvp-demo</code></pre>
  <p class="muted">Same core powers the tiny <code>hashseal-check</code> binary, WASM, browser extension, and zero-dep verify SDKs.</p>
  <p><a href="{{ '/install.html' | relative_url }}">Install details →</a> · <a href="{{ '/cli.html' | relative_url }}">CLI reference →</a></p>
</section>

<section class="section">
  <h2>Docs</h2>
  <div class="grid-2">
    <article class="card">
      <h3><a href="{{ '/instruct/' | relative_url }}">Instruct</a></h3>
      <p>Format, signing, verify SDKs, IDE and browser surfaces.</p>
    </article>
    <article class="card">
      <h3><a href="{{ '/build/' | relative_url }}">Build</a></h3>
      <p>Tree seal, release bundles, plugins, packaging.</p>
    </article>
  </div>
</section>
