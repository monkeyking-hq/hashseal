#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Nathan A. Chadwick
# SPDX-License-Identifier: MIT
set -euo pipefail

HASHSEAL="${HASHSEAL:-hashseal}"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

cd "$TMPDIR"

cat > AGENTS.md <<'EOF'
---
globs:
  - "src/**"
tools:
  - "read"
---
# Agent Policy

Only read files under src/.
EOF

"$HASHSEAL" seal --instruct --root .
"$HASHSEAL" check --root .

# Mutate YAML front-matter policy (tools: read -> bash)
cp AGENTS.md AGENTS.md.bak
sed 's/"read"/"bash"/' AGENTS.md.bak > AGENTS.md

set +e
"$HASHSEAL" check --root .
rc_fm=$?
set -e
if [ "$rc_fm" -ne 1 ]; then
  echo "Expected exit code 1 (MISMATCH on YAML policy), got $rc_fm" >&2
  exit 1
fi

# Restore sealed file, then inject a hidden Unicode BiDi override (U+202E)
mv AGENTS.md.bak AGENTS.md
python3 - <<'PY'
with open("AGENTS.md", "a", encoding="utf-8") as f:
    f.write("Hidden\u202eoverride\n")
PY

set +e
"$HASHSEAL" check --root .
rc_unicode=$?
set -e
if [ "$rc_unicode" -ne 1 ]; then
  echo "Expected exit code 1 (SUSPICIOUS_UNICODE), got $rc_unicode" >&2
  exit 1
fi
