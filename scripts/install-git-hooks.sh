#!/usr/bin/env bash
# Point this clone at repo-managed hooks under scripts/git-hooks/.
# Copyright (c) 2026 MonkeyKing.dev
set -euo pipefail
cd "$(dirname "$0")/.."

HOOKS_PATH="scripts/git-hooks"
if [[ ! -f "$HOOKS_PATH/pre-commit" ]]; then
  echo "error: missing $HOOKS_PATH/pre-commit" >&2
  exit 1
fi

# Ensure hook is executable (POSIX clones / WSL).
chmod +x "$HOOKS_PATH/pre-commit" 2>/dev/null || true

git config core.hooksPath "$HOOKS_PATH"
echo "Configured core.hooksPath=$HOOKS_PATH"
echo "Active checks:"
echo "  pre-commit → cargo fmt --all -- --check (when .rs files are staged)"
