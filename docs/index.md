---
layout: default
title: HashSeal
permalink: /
home: true
description: Cryptographic integrity, trust, and runtime gating for AI agent instruction files and skill packs.
---

<section class="hero">
  <img class="hero__mark" src="{{ '/assets/logo/mark.svg' | relative_url }}" width="88" height="88" alt="HashSeal mark — golden circlet over a blue seal" />
  <h1>What was sealed is what you still have</h1>
  <p class="hero__tagline">Signed, Sealed, Delivered - I'm Yours.</p>
  <p class="hero__lead">
    Cryptographic integrity, trust, and runtime gating for AI agent instruction files and skill packs.
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
  <h2>Core Capabilities</h2>
  <div class="grid-2">
    <article class="card card--lead">
      <span class="lead-label">Instruct-v2 &amp; Signing-v2</span>
      <h3><a href="{{ '/instruct/' | relative_url }}">Signed, verifiable agent instructions</a></h3>
      <p>
        Seal <code>AGENTS.md</code>, <code>CLAUDE.md</code>, <code>SKILL.md</code>, and IDE rules with
        lossless raw front-matter <a href="https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3" rel="noopener noreferrer" target="_blank">BLAKE3</a>
        digests, OpenSSH <code>SSHSIG</code> or hardened GPG signatures bound to file scope, and built-in Unicode smuggling detection.
      </p>
    </article>
    <article class="card">
      <span class="lead-label">Runtime &amp; CI Enforcement</span>
      <h3><a href="{{ '/cli.html' | relative_url }}">Lockfiles, agent hooks &amp; PR gates</a></h3>
      <p>
        Bootstrap <code>.hashseal.lock</code>, git pre-commit hooks, and Claude Code hooks via
        <code>hashseal init</code>. Enforce human-signed instruction edits in PRs with
        <code>hashseal check --diff origin/main --require-signature</code>.
      </p>
    </article>
  </div>
</section>

<section class="section">
  <h2>How it works</h2>
  <ol class="steps">
    <li>
      <div>
        <strong>Bootstrap &amp; Seal</strong>
        <span>Run <code>hashseal init</code> or <code>hashseal seal --instruct --sign --lock</code>.</span>
      </div>
    </li>
    <li>
      <div>
        <strong>Gate Runtime &amp; PRs</strong>
        <span>Agent hooks, pre-commit, and CI check seals, signatures, lockfile, and Unicode before models run.</span>
      </div>
    </li>
    <li>
      <div>
        <strong>Verify Anywhere</strong>
        <span>Same frozen vectors across Rust CLI, <code>hashseal-check</code>, WASM, JS, Python, and Go.</span>
      </div>
    </li>
  </ol>
</section>

<section class="section">
  <h2>Quickstart</h2>
  <pre><code>hashseal init --claude-hooks --git-hooks --lock
hashseal seal --instruct --sign --lock --root .
hashseal check --root . --require-signature</code></pre>
  <p><a href="{{ '/install.html' | relative_url }}">Install details →</a> · <a href="{{ '/cli.html' | relative_url }}">CLI reference →</a></p>
</section>
