#!/usr/bin/env bash
# Build a distributable .deb and .AppImage for this machine's architecture,
# and stage them in release/ with predictable names.
#
# Usage: scripts/package.sh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="$ROOT_DIR/release"

cd "$ROOT_DIR"

echo "==> Installing npm dependencies"
npm install

echo "==> Building app (frontend + Tauri bundles)"
npm run tauri:build

BUNDLE_DIR="src-tauri/target/release/bundle"
DEB_SRC=$(find "$BUNDLE_DIR/deb" -name '*.deb' 2>/dev/null | head -n1 || true)
APPIMAGE_SRC=$(find "$BUNDLE_DIR/appimage" -name '*.AppImage' 2>/dev/null | head -n1 || true)

if [[ -z "$DEB_SRC" && -z "$APPIMAGE_SRC" ]]; then
  echo "error: no .deb or .AppImage produced under $BUNDLE_DIR" >&2
  exit 1
fi

# Work around a Tauri v1 bundler bug: it unconditionally appends its own
# hardcoded "libwebkit2gtk-4.0-37, libgtk-3-0" to whatever `deb.depends` is
# set in tauri.conf.json, rather than replacing it. That bare (non-alternative)
# libwebkit2gtk-4.0-37 doesn't exist on Ubuntu 24.04+/Mint 22+ (they ship 4.1),
# which makes `apt install` refuse to resolve dependencies on any such target.
# Rewrite the control file's Depends field to just the list configured in
# tauri.conf.json so installation works on both older and newer distros.
if [[ -n "$DEB_SRC" ]] && command -v fakeroot >/dev/null && command -v dpkg-deb >/dev/null; then
  echo "==> Fixing up .deb dependency list (tauri-bundler duplicate/broken defaults)"
  WANTED_DEPENDS=$(python3 -c "
import json
cfg = json.load(open('src-tauri/tauri.conf.json'))
print(', '.join(cfg['tauri']['bundle']['deb']['depends']))
")
  DEB_WORK_DIR=$(mktemp -d)
  dpkg-deb -R "$DEB_SRC" "$DEB_WORK_DIR"
  sed -i "s/^Depends:.*/Depends: $WANTED_DEPENDS/" "$DEB_WORK_DIR/DEBIAN/control"
  rm -f "$DEB_SRC"
  fakeroot dpkg-deb -b "$DEB_WORK_DIR" "$DEB_SRC" >/dev/null
  rm -rf "$DEB_WORK_DIR"
  echo "    Depends: $WANTED_DEPENDS"
fi

rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

if [[ -n "$DEB_SRC" ]]; then
  cp "$DEB_SRC" "$RELEASE_DIR/"
  echo "==> Packaged: $RELEASE_DIR/$(basename "$DEB_SRC")"
fi

if [[ -n "$APPIMAGE_SRC" ]]; then
  cp "$APPIMAGE_SRC" "$RELEASE_DIR/"
  chmod +x "$RELEASE_DIR/$(basename "$APPIMAGE_SRC")"
  echo "==> Packaged: $RELEASE_DIR/$(basename "$APPIMAGE_SRC")"
fi

(cd "$RELEASE_DIR" && sha256sum ./* > SHA256SUMS)
echo "==> Checksums written to $RELEASE_DIR/SHA256SUMS"
echo
echo "Built for $(uname -m) — install target machine must be the same architecture"
echo "(and a similar Debian/Ubuntu-based distro if using the .deb)."
echo
echo "Next: scripts/deploy.sh <user@host> to ship it to another machine,"
echo "or copy $RELEASE_DIR manually."
