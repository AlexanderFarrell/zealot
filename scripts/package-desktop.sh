#!/usr/bin/env bash
# Copies the installer artifacts produced by `tauri build` (buried under
# target/release/bundle/<type>/) into artifacts/desktop/, which is already
# gitignored. Run after `npm run tauri:desktop:build -w @zealot/desktop`.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUNDLE_DIR="$ROOT_DIR/target/release/bundle"
DEST_DIR="$ROOT_DIR/artifacts/desktop"

if [ ! -d "$BUNDLE_DIR" ]; then
    echo "No bundle output found at $BUNDLE_DIR — build the desktop app first." >&2
    exit 1
fi

mkdir -p "$DEST_DIR"

shopt -s nullglob
found=0
for f in "$BUNDLE_DIR"/dmg/*.dmg "$BUNDLE_DIR"/deb/*.deb "$BUNDLE_DIR"/appimage/*.AppImage; do
    cp -f "$f" "$DEST_DIR/"
    echo "Copied $(basename "$f") -> artifacts/desktop/"
    found=1
done

if [ "$found" -eq 0 ]; then
    echo "No installer artifacts found under $BUNDLE_DIR" >&2
    exit 1
fi
