#!/usr/bin/env bash
# Build release hashseal + hashseal-check (Unix) and emit SHA256SUMS.
# Copyright (c) 2026 MonkeyKing.dev
set -euo pipefail
cd "$(dirname "$0")/.."

echo "Building hashseal (release)..."
cargo build -p hashseal --release
echo "Building hashseal-check (release)..."
cargo build -p hashseal-check --release

echo "Dependency tree (hashseal-check):"
cargo tree -p hashseal-check --edges normal

REL="$(pwd)/target/release"
(
  cd "$REL"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum hashseal hashseal-check > SHA256SUMS
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 hashseal hashseal-check > SHA256SUMS
  fi
)

echo ""
echo "Artifacts:"
echo "  $REL/hashseal"
echo "  $REL/hashseal-check"
echo "  $REL/SHA256SUMS"
echo ""
echo "Add to PATH or: export HASHSEAL_BIN=$REL/hashseal"
