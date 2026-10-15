#!/bin/bash
set -euo pipefail

# Pixelguard Release Script
# Usage: ./scripts/release.sh <version> [--local]
# Example: ./scripts/release.sh 0.2.1 --local
#
# Flow:
#   1. Verify versions, a clean main branch and passing tests
#   2. Build binaries and create the GitHub release with a SHA256SUMS file:
#      - default: push the tag and let CI build every platform
#      - --local: build every platform here (scripts/build-binaries.sh, needs
#        Docker) and create the release and tag in one step; CI then sees an
#        existing release and leaves its assets alone
#   3. Turn SHA256SUMS into npm/pixelguard/checksums.json, which the wrapper
#      uses to verify the downloaded binary, and commit it
#   4. Publish the npm packages (npm asks for your OTP)
#
# Tags and releases are never force-pushed or recreated, and their assets are
# never replaced: the checksums on npm pin the exact bytes. Fix problems by
# releasing a new version.
#
# Prerequisites:
# - gh CLI installed and authenticated
# - npm logged in (npm login)

VERSION=${1:?"Usage: ./scripts/release.sh <version> [--local]"}
LOCAL_BUILD=false
[ "${2:-}" = "--local" ] && LOCAL_BUILD=true
TAG="v$VERSION"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
REPO="emiliodominguez/pixelguard"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

fail() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

step() {
    echo ""
    echo -e "${BLUE}═══ $1 ═══${NC}"
}

cd "$ROOT_DIR"

step "Preflight"

command -v gh >/dev/null || fail "GitHub CLI (gh) is required: https://cli.github.com"
gh auth status >/dev/null 2>&1 || fail "GitHub CLI is not authenticated. Run: gh auth login"
npm whoami >/dev/null 2>&1 || fail "Not logged in to npm. Run: npm login"

[ "$(git branch --show-current)" = "main" ] || fail "Releases must be made from main."
git diff-index --quiet HEAD -- || fail "You have uncommitted changes."
git fetch origin main --tags
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] || fail "Local main is not in sync with origin/main."

CARGO_VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
NPM_VERSION=$(node -p "require('./npm/pixelguard/package.json').version")
TYPES_VERSION=$(node -p "require('./npm/plugin-types/package.json').version")
[ "$CARGO_VERSION" = "$VERSION" ] || fail "Cargo.toml version is $CARGO_VERSION, expected $VERSION."
[ "$NPM_VERSION" = "$VERSION" ] || fail "npm/pixelguard version is $NPM_VERSION, expected $VERSION."
[ "$TYPES_VERSION" = "$VERSION" ] || fail "npm/plugin-types version is $TYPES_VERSION, expected $VERSION."

if git rev-parse "$TAG" >/dev/null 2>&1; then
    fail "Tag $TAG already exists. Published versions are immutable; bump the version instead."
fi

echo -e "${GREEN}✅ Ready to release $TAG${NC}"

step "Test"
cargo test --locked
echo -e "${GREEN}✅ Tests passed${NC}"

if [ "$LOCAL_BUILD" = true ]; then
    step "Build binaries locally"
    DIST_DIR=$(mktemp -d)
    ./scripts/build-binaries.sh "$DIST_DIR"

    step "Create release"
    # Creating the release also creates the tag, so CI finds an existing
    # release when the tag event arrives and does not touch its assets.
    gh release create "$TAG" "$DIST_DIR"/pixelguard-* "$DIST_DIR/SHA256SUMS" \
        --repo "$REPO" --target "$(git rev-parse HEAD)" --title "$TAG" --generate-notes
    git fetch origin --tags
    echo -e "${GREEN}✅ Created $TAG with locally built binaries${NC}"
else
    step "Tag"
    git tag -a "$TAG" -m "Release $TAG"
    git push origin "$TAG"
    echo -e "${GREEN}✅ Pushed $TAG; CI is building release binaries${NC}"

    step "Wait for CI release"
    for _ in $(seq 1 120); do
        if gh release view "$TAG" --repo "$REPO" --json assets --jq '.assets[].name' 2>/dev/null | grep -qx "SHA256SUMS"; then
            break
        fi
        sleep 30
    done
    gh release view "$TAG" --repo "$REPO" --json assets --jq '.assets[].name' | grep -qx "SHA256SUMS" \
        || fail "CI did not publish $TAG with SHA256SUMS within an hour. Check the CI run."
fi

step "Checksums"
WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT
gh release download "$TAG" --repo "$REPO" --pattern "pixelguard-*" --pattern "SHA256SUMS" --dir "$WORK_DIR"
(cd "$WORK_DIR" && sha256sum -c SHA256SUMS 2>/dev/null || shasum -a 256 -c SHA256SUMS)
node -e '
const fs = require("fs");
const sums = {};
for (const line of fs.readFileSync(process.argv[1], "utf8").split("\n")) {
    const match = line.match(/^([a-f0-9]{64})\s+\*?(pixelguard-\S+)$/);
    if (match) sums[match[2]] = match[1];
}
if (Object.keys(sums).length === 0) throw new Error("SHA256SUMS is empty");
fs.writeFileSync(process.argv[2], JSON.stringify(sums, null, "\t") + "\n");
' "$WORK_DIR/SHA256SUMS" "npm/pixelguard/checksums.json"
cat npm/pixelguard/checksums.json
git add npm/pixelguard/checksums.json
git commit -m "chore(npm): add $TAG binary checksums"
git push origin main
echo -e "${GREEN}✅ checksums.json generated and committed${NC}"

step "Publish to npm"
(cd npm/plugin-types && npm run build && npm publish --access public)
(cd npm/pixelguard && npm publish --access public)

echo ""
echo -e "${GREEN}🎉 Released $TAG${NC}"
echo "View release: https://github.com/$REPO/releases/tag/$TAG"
echo "Test installation: npx pixelguard@$VERSION --version"
