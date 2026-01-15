# Publishing Pixelguard

This guide covers how to release new versions of Pixelguard to npm.

## Prerequisites

- Node.js 18+
- Rust toolchain (for building binaries)
- npm account with publish access to `pixelguard` and `@pixelguard/plugin-types`
- Git with clean working directory

## Quick Publish (Single Platform)

For quick releases during development:

```bash
# Publish version 0.1.0
./scripts/publish.sh 0.1.0
```

This script will:
1. Update version numbers in all package files
2. Build the Rust binary for your current platform
3. Run tests
4. Copy binary to the npm package
5. Build TypeScript types
6. Publish both packages to npm
7. Create a git tag

**Note:** This only builds for your current platform. For full releases, see below.

## Full Multi-Platform Release

For production releases, you need binaries for all supported platforms:

### 1. Build on Each Platform

**macOS (Intel):**
```bash
cargo build --release
cp target/release/pixelguard npm/pixelguard/bin/pixelguard-darwin-x64
```

**macOS (Apple Silicon):**
```bash
cargo build --release
cp target/release/pixelguard npm/pixelguard/bin/pixelguard-darwin-arm64
```

**Linux (x64):**
```bash
cargo build --release
cp target/release/pixelguard npm/pixelguard/bin/pixelguard-linux-x64
```

**Linux (ARM64):**
```bash
cargo build --release
cp target/release/pixelguard npm/pixelguard/bin/pixelguard-linux-arm64
```

**Windows (x64):**
```bash
cargo build --release
copy target\release\pixelguard.exe npm\pixelguard\bin\pixelguard-win32-x64.exe
```

### 2. Update Versions

```bash
# Update all package.json and Cargo.toml files
VERSION=0.2.0
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
cd npm/pixelguard && npm version $VERSION --no-git-tag-version
cd ../plugin-types && npm version $VERSION --no-git-tag-version
```

### 3. Build Plugin Types

```bash
cd npm/plugin-types
npm run build
```

### 4. Publish

```bash
# Publish main package
cd npm/pixelguard
npm publish --access public

# Publish plugin types
cd ../plugin-types
npm publish --access public
```

### 5. Tag Release

```bash
git add -A
git commit -m "chore(release): v0.2.0"
git tag -a "v0.2.0" -m "Release v0.2.0"
git push origin main --tags
```

## Using Changesets

For tracking changes between releases:

```bash
# Add a changeset for your changes
npm run changeset

# When ready to release, bump versions
npm run version

# Then publish
npm run publish:cli
```

## Version Guidelines

- **Patch (0.1.x):** Bug fixes, documentation updates
- **Minor (0.x.0):** New features, non-breaking changes
- **Major (x.0.0):** Breaking changes to CLI interface or config format

## Supported Platforms

| Platform | Architecture | Binary Name |
|----------|--------------|-------------|
| macOS | Intel (x64) | `pixelguard-darwin-x64` |
| macOS | Apple Silicon (arm64) | `pixelguard-darwin-arm64` |
| Linux | x64 | `pixelguard-linux-x64` |
| Linux | ARM64 | `pixelguard-linux-arm64` |
| Windows | x64 | `pixelguard-win32-x64.exe` |

## Troubleshooting

### "npm ERR! 403 Forbidden"
You need to be logged in and have publish access:
```bash
npm login
npm access grant read-write <your-username> pixelguard
```

### Binary not found after install
Make sure the postinstall script is executable:
```bash
chmod +x npm/pixelguard/scripts/postinstall.js
chmod +x npm/pixelguard/scripts/run.js
```

### Version mismatch
Ensure all version numbers match:
- `Cargo.toml` (workspace version)
- `npm/pixelguard/package.json`
- `npm/plugin-types/package.json`
