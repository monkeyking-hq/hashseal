#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Nathan A. Chadwick
# SPDX-License-Identifier: MIT
set -euo pipefail

HASHSEAL="${HASHSEAL:-hashseal}"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

cd "$TMPDIR"
git init -q

# Ephemeral Ed25519 signing key + allowed_signers trust root
ssh-keygen -q -t ed25519 -N "" -f "$TMPDIR/demo_key" -C "demo@hashseal.ai"
mkdir -p .hashseal
printf 'demo@hashseal.ai namespaces="hashseal" %s\n' "$(cat "$TMPDIR/demo_key.pub")" > .hashseal/allowed_signers

git config gpg.format ssh
git config user.signingkey "$TMPDIR/demo_key"

cat > AGENTS.md <<'EOF'
# Agent Instructions

Always run cargo test --workspace --release.
EOF

cat > SKILL.md <<'EOF'
# Benign Skill

Summarize git status.
EOF

"$HASHSEAL" seal --instruct --sign --lock --root .
"$HASHSEAL" check --root . --require-signature

# Cross-file signature replay attack: copy signed SKILL.md over AGENTS.md
cp SKILL.md AGENTS.md

set +e
"$HASHSEAL" check --root . --require-signature
rc_replay=$?
set -e
if [ "$rc_replay" -ne 1 ]; then
  echo "Expected exit code 1 (BAD_SIGNATURE from scope mismatch), got $rc_replay" >&2
  exit 1
fi
