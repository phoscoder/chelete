#!/usr/bin/env bash
# Print release notes for TAG: one line per commit since the previous tag,
# leaving out merges, version bumps and roadmap bookkeeping.
set -euo pipefail

TAG="${1:?Usage: $0 <tag>}"
PREV=$(git describe --tags --abbrev=0 --match 'v*' "${TAG}^" 2>/dev/null || true)

echo "## What's Changed"
echo
git log --no-merges --reverse --format='- %s' ${PREV:+$PREV..}"$TAG" \
    | grep -v -i -e '\[skip ci\]' -e '^- chore: release v' -e 'roadmap' || true

if [ -n "$PREV" ]; then
    REPO_URL=$(git remote get-url origin 2>/dev/null | sed -E 's#^git@github.com:#https://github.com/#; s#\.git$##')
    echo
    echo "**Full Changelog**: ${REPO_URL}/compare/${PREV}...${TAG}"
fi
