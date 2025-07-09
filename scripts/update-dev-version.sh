#!/bin/bash

# Script to update version for development builds
# Usage: ./scripts/update-dev-version.sh [version]

set -e

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Change to project root
cd "$PROJECT_ROOT"

# Generate development version if not provided
if [ -z "$1" ]; then
    COMMIT_HASH=$(git rev-parse --short HEAD)
    TIMESTAMP=$(date +%Y%m%d%H%M)
    BASE_VERSION="2.0.0-alpha"
    DEV_VERSION="${BASE_VERSION}.dev.${TIMESTAMP}.${COMMIT_HASH}"
else
    DEV_VERSION="$1"
fi

echo "Updating version to: $DEV_VERSION"

# Update package.json version
echo "Updating package.json..."
npm version "$DEV_VERSION" --no-git-tag-version

# Update Cargo.toml version
echo "Updating src-tauri/Cargo.toml..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"$DEV_VERSION\"/" src-tauri/Cargo.toml
else
    # Linux/Windows
    sed -i "s/^version = \".*\"/version = \"$DEV_VERSION\"/" src-tauri/Cargo.toml
fi

# Update tauri.conf.json version
echo "Updating src-tauri/tauri.conf.json..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/\"version\": \".*\"/\"version\": \"$DEV_VERSION\"/" src-tauri/tauri.conf.json
else
    # Linux/Windows
    sed -i "s/\"version\": \".*\"/\"version\": \"$DEV_VERSION\"/" src-tauri/tauri.conf.json
fi

echo "Version updated successfully to: $DEV_VERSION"
echo ""
echo "Updated files:"
echo "  - package.json"
echo "  - src-tauri/Cargo.toml"
echo "  - src-tauri/tauri.conf.json"
echo ""
echo "To revert changes, run: git checkout -- package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json"
