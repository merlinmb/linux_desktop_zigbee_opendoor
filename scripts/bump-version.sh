#!/usr/bin/env bash
# Bump version in all version files (patch increment)
# Usage: scripts/bump-version.sh [major|minor|patch]
# Default: patch (e.g., 1.0.0 -> 1.0.1)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUMP_TYPE="${1:-patch}"

# Get current version from package.json
CURRENT_VERSION=$(grep -m1 '"version"' "$ROOT_DIR/package.json" | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/')

echo "Current version: $CURRENT_VERSION"

# Parse version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Bump based on type
case "$BUMP_TYPE" in
  major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
  patch)
    PATCH=$((PATCH + 1))
    ;;
  *)
    echo "Usage: $0 [major|minor|patch]" >&2
    exit 1
    ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
echo "New version: $NEW_VERSION"

# Update package.json
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$ROOT_DIR/package.json"
echo "✓ Updated package.json"

# Update Cargo.toml
sed -i "s/^version = \"[^\"]*\"$/version = \"$NEW_VERSION\"/" "$ROOT_DIR/src-tauri/Cargo.toml"
echo "✓ Updated src-tauri/Cargo.toml"

# Update tauri.conf.json
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$ROOT_DIR/src-tauri/tauri.conf.json"
echo "✓ Updated src-tauri/tauri.conf.json"

echo "✓ Version bumped to $NEW_VERSION"
