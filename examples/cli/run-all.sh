#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Nathan A. Chadwick
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [ -z "${HASHSEAL:-}" ]; then
  if [ -x "${ROOT}/target/release/hashseal" ]; then
    export HASHSEAL="${ROOT}/target/release/hashseal"
  elif [ -x "${ROOT}/target/x86_64-unknown-linux-musl/release/hashseal" ]; then
    export HASHSEAL="${ROOT}/target/x86_64-unknown-linux-musl/release/hashseal"
  else
    export HASHSEAL="hashseal"
  fi
fi

if [ -z "${HASHSEAL_CHECK:-}" ]; then
  if [ -x "${ROOT}/target/release/hashseal-check" ]; then
    export HASHSEAL_CHECK="${ROOT}/target/release/hashseal-check"
  elif [ -x "${ROOT}/target/x86_64-unknown-linux-musl/release/hashseal-check" ]; then
    export HASHSEAL_CHECK="${ROOT}/target/x86_64-unknown-linux-musl/release/hashseal-check"
  else
    export HASHSEAL_CHECK="hashseal-check"
  fi
fi

for script in \
  "${ROOT}/examples/cli/01-overview.sh" \
  "${ROOT}/examples/cli/02-init-and-lockfile.sh" \
  "${ROOT}/examples/cli/03-tamper-and-unicode.sh" \
  "${ROOT}/examples/cli/04-ssh-signing.sh" \
  "${ROOT}/examples/cli/05-check-and-json.sh"
do
  echo "=== Running $(basename "$script") ==="
  bash "$script"
done

echo "All CLI examples passed."
