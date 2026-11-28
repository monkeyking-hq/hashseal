#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Nathan A. Chadwick
# SPDX-License-Identifier: MIT
set -euo pipefail

HASHSEAL="${HASHSEAL:-hashseal}"
HASHSEAL_CHECK="${HASHSEAL_CHECK:-hashseal-check}"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

cd "$TMPDIR"

cat > AGENTS.md <<'EOF'
# Agent Instructions

Verify before running.
EOF

"$HASHSEAL" seal --instruct --root .

# Tiny zero-serde/zero-clap binary for fast runtime/CI digest checks
"$HASHSEAL_CHECK" --root . --digest-only

# Machine-readable JSON output from full CLI
"$HASHSEAL" check --root . --format json
