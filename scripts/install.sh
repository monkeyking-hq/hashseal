#!/usr/bin/env sh
# HashSeal installer — downloads release archive, verifies SHA256SUMS, or builds from source.
# Copyright (c) 2026 MonkeyKing.dev
set -eu

REPO="${HASHSEAL_REPO:-https://github.com/monkeyking-hq/hashseal}"
VERSION="${HASHSEAL_VERSION:-latest}"
PREFIX="${HASHSEAL_INSTALL_DIR:-$HOME/.local/bin}"
FROM_SOURCE=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --from-source) FROM_SOURCE=1 ;;
    --version) shift; VERSION="${1:-latest}" ;;
    --prefix) shift; PREFIX="${1:-$PREFIX}" ;;
    -h|--help)
      echo "Usage: install.sh [--from-source] [--version vX.Y.Z] [--prefix DIR]"
      exit 0
      ;;
    *)
      echo "install.sh: unknown option $1" >&2
      exit 2
      ;;
  esac
  shift
done

if [ "$FROM_SOURCE" -eq 1 ]; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo "install.sh: cargo is required for --from-source" >&2
    exit 1
  fi
  cargo install --path rust/hashseal --locked
  cargo install --path rust/hashseal-check --locked
  exit 0
fi

OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS/$ARCH" in
  Linux/x86_64|Linux/amd64) TARGET="x86_64-unknown-linux-gnu" ;;
  Linux/aarch64|Linux/arm64) TARGET="aarch64-unknown-linux-gnu" ;;
  Darwin/x86_64) TARGET="x86_64-apple-darwin" ;;
  Darwin/arm64) TARGET="aarch64-apple-darwin" ;;
  *)
    echo "install.sh: unsupported platform $OS/$ARCH (try --from-source)" >&2
    exit 1
    ;;
esac

if [ "$VERSION" = "latest" ]; then
  BASE_URL="$REPO/releases/latest/download"
else
  BASE_URL="$REPO/releases/download/$VERSION"
fi

ARCHIVE="hashseal-$TARGET.tar.gz"
TMPD="$(mktemp -d)"
trap 'rm -rf "$TMPD"' EXIT

echo "Downloading $ARCHIVE and SHA256SUMS..."
curl -fsSL "$BASE_URL/$ARCHIVE" -o "$TMPD/$ARCHIVE"
curl -fsSL "$BASE_URL/SHA256SUMS" -o "$TMPD/SHA256SUMS"

(
  cd "$TMPD"
  if command -v sha256sum >/dev/null 2>&1; then
    grep " $ARCHIVE\$" SHA256SUMS | sha256sum -c -
  elif command -v shasum >/dev/null 2>&1; then
    grep " $ARCHIVE\$" SHA256SUMS | shasum -a 256 -c -
  else
    echo "install.sh: sha256sum/shasum required to verify archive" >&2
    exit 1
  fi
)

mkdir -p "$PREFIX" "$TMPD/unpack"
tar -xzf "$TMPD/$ARCHIVE" -C "$TMPD/unpack"
install -m 0755 "$TMPD/unpack/hashseal" "$PREFIX/hashseal"
install -m 0755 "$TMPD/unpack/hashseal-check" "$PREFIX/hashseal-check"
echo "Installed hashseal and hashseal-check to $PREFIX"
