---
layout: default
title: Security
permalink: /security.html
description: Report security issues for HashSeal.
---

# Security

Report vulnerabilities and security-sensitive bugs to:

**[security@hashseal.ai](mailto:security@hashseal.ai)**

Please do **not** open public issues for unfixed security problems.

## What to include

- Affected component (CLI, core, verify SDK language, extension, plugin)
- Version or commit if known
- Steps to reproduce
- Impact (integrity bypass, signature handling, path handling, etc.)

## Scope notes

HashSeal provides **content integrity, cryptographic trust, and runtime gating** for AI agent instruction files, skill packs, and optional release bundles (`instruct-v2` lossless raw front-matter digests, `signing-v2` OpenSSH `SSHSIG` and hardened GPG signatures with pinned trust roots and context `scope` binding, `.hashseal.lock`, and `SUSPICIOUS_UNICODE` smuggling detection). It does not guarantee how a model will behave or replace code review.

## Contacts

| | |
|--|--|
| Security | [security@hashseal.ai](mailto:security@hashseal.ai) |
| General | [info@hashseal.ai](mailto:info@hashseal.ai) |

```text
Copyright (c) 2026 MonkeyKing.dev
```
