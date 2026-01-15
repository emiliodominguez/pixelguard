#!/bin/bash
set -e

# Pixelguard Manual Publish Script
# Usage: ./scripts/publish.sh [version]
# Example: ./scripts/publish.sh 0.1.0

VERSION=${1:-"0.1.0"}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "🚀 Publishing Pixelguard v$VERSION"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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
    echo "   Please commit or stash your changes before publishing."
    exit 1
fi

# Step 1: Update versions
echo "📝 Step 1: Updating versions to $VERSION..."

# Update Cargo.toml workspace version
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" "$ROOT_DIR/Cargo.toml"

# Update npm/pixelguard/package.json
cd "$ROOT_DIR/npm/pixelguard"
npm version "$VERSION" --no-git-tag-version --allow-same-version

# Update npm/plugin-types/package.json
cd "$ROOT_DIR/npm/plugin-types"
npm version "$VERSION" --no-git-tag-version --allow-same-version

cd "$ROOT_DIR"
echo -e "${GREEN}✅ Versions updated${NC}"
echo ""

# Step 2: Build Rust binaries
echo "🔨 Step 2: Building Rust binaries..."
cargo build --release

if [ ! -f "target/release/pixelguard" ]; then
    echo -e "${RED}❌ Error: Build failed - binary not found${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Build successful${NC}"
echo ""

# Step 3: Run tests
echo "🧪 Step 3: Running tests..."
cargo test
echo -e "${GREEN}✅ Tests passed${NC}"
echo ""

# Step 4: Copy binary to npm package
echo "📦 Step 4: Preparing npm package..."

# Detect platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$ARCH" = "x86_64" ]; then
    ARCH="x64"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    ARCH="arm64"
fi

BIN_DIR="$ROOT_DIR/npm/pixelguard/bin"
mkdir -p "$BIN_DIR"

# Copy binary with platform-specific name
cp "target/release/pixelguard" "$BIN_DIR/pixelguard-$PLATFORM-$ARCH"
chmod +x "$BIN_DIR/pixelguard-$PLATFORM-$ARCH"

echo -e "${GREEN}✅ Binary copied: pixelguard-$PLATFORM-$ARCH${NC}"
echo ""

# Step 5: Build plugin-types
echo "📚 Step 5: Building plugin-types..."
cd "$ROOT_DIR/npm/plugin-types"
npm run build
echo -e "${GREEN}✅ Plugin types built${NC}"
echo ""

# Step 6: Confirmation
echo "─────────────────────────────────────────"
echo ""
echo "📋 Ready to publish:"
echo "   • pixelguard@$VERSION"
echo "   • @pixelguard/plugin-types@$VERSION"
echo ""
echo "   Binary: pixelguard-$PLATFORM-$ARCH"
echo ""
echo -e "${YELLOW}⚠️  Note: This script only built for your current platform.${NC}"
echo "   For a full release, build on all platforms (macOS, Linux, Windows)"
echo "   and copy all binaries to npm/pixelguard/bin/"
echo ""

read -p "Publish to npm? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Step 7: Publish
echo ""
echo "🚀 Publishing to npm..."

cd "$ROOT_DIR/npm/pixelguard"
npm publish --access public

cd "$ROOT_DIR/npm/plugin-types"
npm publish --access public

echo ""
echo -e "${GREEN}✅ Published successfully!${NC}"
echo ""

# Step 8: Git tag
echo "🏷️  Creating git tag..."
cd "$ROOT_DIR"
git add -A
git commit -m "chore(release): v$VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"

echo ""
echo -e "${GREEN}🎉 Release complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Push the tag: git push origin v$VERSION"
echo "  2. Push main: git push origin main"
echo "  3. Create GitHub release (optional)"
