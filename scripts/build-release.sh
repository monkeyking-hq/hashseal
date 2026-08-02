#!/usr/bin/env bash
# Build release hashseal + hashseal-check (Unix).
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
echo ""
echo "Artifacts:"
echo "  $REL/hashseal"
echo "  $REL/hashseal-check"
echo ""
echo "Add to PATH or: export HASHSEAL_BIN=$REL/hashseal"
