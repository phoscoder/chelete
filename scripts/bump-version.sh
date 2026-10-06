#!/usr/bin/env bash
set -euo pipefail

VERSION_BUMP="${1:-patch}"

if [[ "$VERSION_BUMP" != "patch" && "$VERSION_BUMP" != "minor" && "$VERSION_BUMP" != "major" ]]; then
    echo "Usage: $0 [patch|minor|major]"
    exit 1
fi

# Current version from the package table in Cargo.toml.
CURRENT=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "([^"]+)"/\1/')
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

# Update Cargo.toml (first match only: the package version) and the lockfile entry.
sed -i "0,/^version = \"$CURRENT\"/s//version = \"$NEW_VERSION\"/" Cargo.toml
cargo update --offline --workspace >/dev/null 2>&1 || cargo update --workspace >/dev/null

# Keep the AUR package in step with the release.
sed -i "s/^pkgver=.*/pkgver=${NEW_VERSION}/" aur/PKGBUILD

# Create git commit and tag
git add Cargo.toml Cargo.lock aur/PKGBUILD
git commit -m "chore: release v${NEW_VERSION} [skip ci]"
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

echo ""
echo "Created commit and tag v${NEW_VERSION}"
echo "To release, run: git push origin main --tags"
