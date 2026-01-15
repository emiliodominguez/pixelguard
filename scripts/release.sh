#!/bin/bash
set -e

# Pixelguard Fully Automated Release Script
# Usage: ./scripts/release.sh [version]
# Example: ./scripts/release.sh 0.1.0
#
# Prerequisites:
# - gh CLI installed (brew install gh)
# - npm logged in (npm login)
# - On main branch with no uncommitted changes

VERSION=${1:-"0.1.0"}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "🚀 Automated release for Pixelguard v$VERSION"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ Error: GitHub CLI (gh) is required but not installed.${NC}"
    echo ""
    echo "Install it with:"
    echo "  macOS: brew install gh"
    echo "  Linux: https://github.com/cli/cli/blob/trunk/docs/install_linux.md"
    echo ""
    exit 1
fi

# Check if gh is authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${YELLOW}⚠️  GitHub CLI not authenticated.${NC}"
    echo "Running: gh auth login"
    gh auth login
fi

# Check npm login
if ! npm whoami &> /dev/null; then
    echo -e "${RED}❌ Error: Not logged in to npm.${NC}"
    echo "Run: npm login"
    exit 1
fi

# Check if we're on main branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo -e "${YELLOW}⚠️  Warning: You are not on the main branch (current: $BRANCH)${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}❌ Error: You have uncommitted changes.${NC}"
    echo "   Please commit or stash your changes before releasing."
    exit 1
fi

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 1: Build Release Binary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

cargo build --release

if [ ! -f "target/release/pixelguard" ]; then
    echo -e "${RED}❌ Error: Build failed - binary not found${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Build successful${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 2: Run Tests${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

cargo test
echo -e "${GREEN}✅ Tests passed${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 3: Determine Platform${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

# Detect platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$PLATFORM" = "darwin" ] && [ "$ARCH" = "x86_64" ]; then
  TARGET="x86_64-apple-darwin"
elif [ "$PLATFORM" = "darwin" ] && [ "$ARCH" = "arm64" ]; then
  TARGET="aarch64-apple-darwin"
elif [ "$PLATFORM" = "linux" ] && [ "$ARCH" = "x86_64" ]; then
  TARGET="x86_64-unknown-linux-gnu"
elif [ "$PLATFORM" = "linux" ] && [ "$ARCH" = "aarch64" ]; then
  TARGET="aarch64-unknown-linux-gnu"
elif [ "$PLATFORM" = "mingw64" ] || [ "$PLATFORM" = "msys" ]; then
  TARGET="x86_64-pc-windows-msvc"
else
  echo -e "${RED}❌ Error: Unsupported platform: $PLATFORM-$ARCH${NC}"
  exit 1
fi

echo "Platform: $TARGET"
echo -e "${GREEN}✅ Platform detected${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 4: Prepare Release Binary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

# Copy binary with correct name
if [ "$PLATFORM" = "mingw64" ] || [ "$PLATFORM" = "msys" ]; then
  BINARY_NAME="pixelguard-$TARGET.exe"
  cp target/release/pixelguard.exe "$BINARY_NAME"
else
  BINARY_NAME="pixelguard-$TARGET"
  cp target/release/pixelguard "$BINARY_NAME"
fi

echo "Binary: $BINARY_NAME"
echo -e "${GREEN}✅ Binary prepared${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 5: Create/Update Git Tag${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

# Check if tag already exists
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Tag v$VERSION already exists${NC}"
    read -p "Delete and recreate tag? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git tag -d "v$VERSION"
        git push origin ":refs/tags/v$VERSION" 2>/dev/null || true
        git tag -a "v$VERSION" -m "Release v$VERSION"
        echo -e "${GREEN}✅ Tag recreated${NC}"
    else
        echo "Using existing tag"
    fi
else
    git tag -a "v$VERSION" -m "Release v$VERSION"
    echo -e "${GREEN}✅ Tag created${NC}"
fi
echo ""

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 6: Push Tag to GitHub${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

git push origin "v$VERSION" --force
echo -e "${GREEN}✅ Tag pushed${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 7: Create GitHub Release${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

# Check if release already exists
if gh release view "v$VERSION" &>/dev/null; then
    echo -e "${YELLOW}⚠️  Release v$VERSION already exists${NC}"
    read -p "Delete and recreate release? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        gh release delete "v$VERSION" --yes
        gh release create "v$VERSION" \
            --title "v$VERSION - Initial Release" \
            --notes "See [ROADMAP.md](https://github.com/emiliodominguez/pixelguard/blob/main/ROADMAP.md) for planned features.

## Installation

\`\`\`bash
npx pixelguard@$VERSION init
\`\`\`

## Platform Support

This release includes a binary for: **$TARGET**

Other platforms can build from source." \
            "$BINARY_NAME"
        echo -e "${GREEN}✅ Release recreated with binary${NC}"
    else
        # Just upload the binary to existing release
        gh release upload "v$VERSION" "$BINARY_NAME" --clobber
        echo -e "${GREEN}✅ Binary uploaded to existing release${NC}"
    fi
else
    gh release create "v$VERSION" \
        --title "v$VERSION - Initial Release" \
        --notes "See [ROADMAP.md](https://github.com/emiliodominguez/pixelguard/blob/main/ROADMAP.md) for planned features.

## Installation

\`\`\`bash
npx pixelguard@$VERSION init
\`\`\`

## Platform Support

This release includes a binary for: **$TARGET**

Other platforms can build from source." \
        "$BINARY_NAME"
    echo -e "${GREEN}✅ Release created with binary${NC}"
fi

# Clean up local binary
rm "$BINARY_NAME"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 8: Build Plugin Types${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

cd "$ROOT_DIR/npm/plugin-types"
npm run build
echo -e "${GREEN}✅ Plugin types built${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 9: Publish to npm${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}"
echo ""

echo "Publishing pixelguard-plugin-types@$VERSION..."
npm publish --access public
echo -e "${GREEN}✅ Plugin types published${NC}"
echo ""

cd "$ROOT_DIR/npm/pixelguard"
echo "Publishing pixelguard@$VERSION..."
npm publish --access public
echo -e "${GREEN}✅ Main package published${NC}"
echo ""

echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo -e "${GREEN}🎉 Release Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo ""
echo "Released: v$VERSION"
echo "Binary platform: $TARGET"
echo ""
echo "View release: https://github.com/emiliodominguez/pixelguard/releases/tag/v$VERSION"
echo "npm package: https://www.npmjs.com/package/pixelguard"
echo ""
echo "Test installation:"
echo "  npx pixelguard@$VERSION --version"
echo ""
