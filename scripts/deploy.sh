#!/usr/bin/env bash
# Ship the package built by scripts/package.sh to another Linux machine over
# SSH and install it there.
#
# Usage:
#   scripts/deploy.sh user@host [--appimage] [--config] [--copy-local-config]
#
#   --appimage           Deploy the .AppImage instead of the .deb (default: .deb,
#                        since it declares its own runtime dependencies via apt).
#   --config             Also copy config.toml.example to the target's
#                        ~/.config/opendoor-monitor/config.toml, but only if that
#                        file doesn't already exist there (never overwrites).
#   --copy-local-config  Copy the local config.toml to the target's
#                        ~/.config/opendoor-monitor/config.toml, overwriting any
#                        existing config there.
#
# Assumes the target machine is the same CPU architecture, and (for the .deb)
# a Debian/Ubuntu-based distro with apt.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="$ROOT_DIR/release"

TARGET="${1:-}"
if [[ -z "$TARGET" ]]; then
  echo "Usage: $0 user@host [--appimage] [--config] [--copy-local-config]" >&2
  exit 1
fi
shift

USE_APPIMAGE=false
PUSH_CONFIG=false
PUSH_LOCAL_CONFIG=false
for arg in "$@"; do
  case "$arg" in
    --appimage) USE_APPIMAGE=true ;;
    --config) PUSH_CONFIG=true ;;
    --copy-local-config) PUSH_LOCAL_CONFIG=true ;;
    *) echo "Unknown option: $arg" >&2; exit 1 ;;
  esac
done

if [[ ! -d "$RELEASE_DIR" ]]; then
  echo "error: $RELEASE_DIR not found — run scripts/package.sh first" >&2
  exit 1
fi

if $USE_APPIMAGE; then
  PKG=$(find "$RELEASE_DIR" -maxdepth 1 -name '*.AppImage' | head -n1 || true)
  [[ -z "$PKG" ]] && { echo "error: no .AppImage in $RELEASE_DIR" >&2; exit 1; }
else
  PKG=$(find "$RELEASE_DIR" -maxdepth 1 -name '*.deb' | head -n1 || true)
  [[ -z "$PKG" ]] && { echo "error: no .deb in $RELEASE_DIR" >&2; exit 1; }
fi

PKG_NAME=$(basename "$PKG")
REMOTE_TMP="/tmp/$PKG_NAME"

echo "==> Copying $PKG_NAME to $TARGET:$REMOTE_TMP"
scp "$PKG" "$TARGET:$REMOTE_TMP"

if $USE_APPIMAGE; then
  echo "==> Installing AppImage on $TARGET (~/Applications)"
  ssh -t "$TARGET" "mkdir -p ~/Applications && mv '$REMOTE_TMP' ~/Applications/ && chmod +x ~/Applications/$PKG_NAME && pkill -f opendoor-monitor || true"
  LAUNCH_HINT="~/Applications/$PKG_NAME"
else
  echo "==> Installing .deb on $TARGET (sudo apt install)"
  ssh -t "$TARGET" "sudo apt-get install -y '$REMOTE_TMP' && rm -f '$REMOTE_TMP' && pkill -f opendoor-monitor || true"
  LAUNCH_HINT="opendoor-monitor (or from the applications menu)"
fi

if $PUSH_LOCAL_CONFIG; then
  echo "==> Copying local config.toml to remote (overwriting)"
  scp "$ROOT_DIR/config.toml" "$TARGET:/tmp/config.toml"
  ssh "$TARGET" '
    set -e
    mkdir -p ~/.config/opendoor-monitor
    cp /tmp/config.toml ~/.config/opendoor-monitor/config.toml
    echo "    installed config.toml"
    rm -f /tmp/config.toml
  '
elif $PUSH_CONFIG; then
  echo "==> Copying config.toml.example (only if remote has no config yet)"
  scp "$ROOT_DIR/config.toml.example" "$TARGET:/tmp/config.toml.example"
  ssh "$TARGET" '
    set -e
    mkdir -p ~/.config/opendoor-monitor
    if [[ -f ~/.config/opendoor-monitor/config.toml ]]; then
      echo "    existing config.toml left untouched"
    else
      cp /tmp/config.toml.example ~/.config/opendoor-monitor/config.toml
      echo "    installed default config.toml — edit it with your MQTT broker/topics"
    fi
    rm -f /tmp/config.toml.example
  '
fi

echo
echo "==> Done. On $TARGET, launch with: $LAUNCH_HINT"
