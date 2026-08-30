#!/usr/bin/env bash
set -euo pipefail

VERSION_BUMP="${1:-patch}"

if [[ "$VERSION_BUMP" != "patch" && "$VERSION_BUMP" != "minor" && "$VERSION_BUMP" != "major" ]]; then
    echo "Usage: $0 [patch|minor|major]"
    exit 1
fi

# Get current version from package.json
CURRENT=$(node -p "require('./package.json').version")
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

case "$VERSION_BUMP" in
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
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
echo "Bumping version: $CURRENT -> $NEW_VERSION"

# Update package.json
sed -i "s/\"version\": \"$CURRENT\"/\"version\": \"$NEW_VERSION\"/" package.json

# Update tauri.conf.json
sed -i "s/\"version\": \"$CURRENT\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json

# Update Cargo.toml
sed -i "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml

# Create git commit and tag
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore: release v${NEW_VERSION}"
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

echo ""
echo "Created commit and tag v${NEW_VERSION}"
echo "To release, run: git push origin main --tags"
