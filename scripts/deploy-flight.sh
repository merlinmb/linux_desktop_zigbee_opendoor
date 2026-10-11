#!/usr/bin/env bash
# Bump the patch version, build the AppImage, ship it to flight.local and
# restart the app there.
#
# Usage:
#   scripts/deploy-flight.sh [--skip-build] [--config] [--copy-local-config]
#
#   --skip-build         Deploy whatever is already in release/ instead of
#                        bumping the version and rebuilding first.
#   --config, --copy-local-config
#                        Passed through to scripts/deploy.sh.
#
# Override the target with FLIGHT_USER / FLIGHT_HOST, e.g.
#   FLIGHT_USER=pi scripts/deploy-flight.sh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="$ROOT_DIR/release"
TARGET="${FLIGHT_USER:-$USER}@${FLIGHT_HOST:-flight.local}"

SKIP_BUILD=false
DEPLOY_ARGS=()
for arg in "$@"; do
  case "$arg" in
    --skip-build) SKIP_BUILD=true ;;
    --config|--copy-local-config) DEPLOY_ARGS+=("$arg") ;;
    *) echo "Unknown option: $arg" >&2; exit 1 ;;
  esac
done

if ! $SKIP_BUILD; then
  bash "$ROOT_DIR/scripts/bump-version.sh" patch
  bash "$ROOT_DIR/scripts/package.sh"
fi

bash "$ROOT_DIR/scripts/deploy.sh" "$TARGET" --appimage "${DEPLOY_ARGS[@]}"

PKG_NAME=$(basename "$(find "$RELEASE_DIR" -maxdepth 1 -name '*.AppImage' | head -n1)")

echo "==> Restarting $PKG_NAME on $TARGET"
# [o] keeps pkill/pgrep -f from matching this remote shell's own command line.
ssh "$TARGET" "
  pkill -f '[o]pendoor-monitor' || true
  sleep 1
  DISPLAY=:0 setsid nohup ~/Applications/$PKG_NAME >/tmp/opendoor-monitor.log 2>&1 </dev/null &
  sleep 2
  if pgrep -f '[o]pendoor-monitor' >/dev/null; then
    echo '    running. Log: /tmp/opendoor-monitor.log'
  else
    echo '    failed to start — last log lines:' >&2
    tail -n 20 /tmp/opendoor-monitor.log >&2
    exit 1
  fi
"
