#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Nathan A. Chadwick
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "=== Building static Linux binaries (x86_64-unknown-linux-musl) ==="
rustup target add x86_64-unknown-linux-musl >/dev/null 2>&1 || true
cargo build --release --locked --target x86_64-unknown-linux-musl -p hashseal -p hashseal-check

echo "=== Building VHS image with git + openssh-client ==="
docker build -q -t hashseal-vhs -f docs/tapes/Dockerfile docs/tapes

mkdir -p docs/assets/demos
for tape in docs/tapes/*.tape; do
  echo "=== Recording $tape ==="
  docker run --rm -v "${ROOT}:/vhs" -w /vhs hashseal-vhs "$tape"
done

ls -lh docs/assets/demos/
for name in overview init-and-lockfile tamper-and-unicode ssh-signing check-and-json; do
  for ext in gif webm png; do
    test -s "docs/assets/demos/${name}.${ext}"
  done
done

echo "All 5 demos (15 files) recorded successfully into docs/assets/demos/."
