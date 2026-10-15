#!/bin/bash
set -euo pipefail

# Builds every release binary locally, without CI.
# Usage: ./scripts/build-binaries.sh [out-dir]   (default: dist/)
#
# Requirements: macOS host with rustup, and a running Docker daemon.
#   - macOS (arm64, x64): built natively with rustup targets
#   - Linux (x64, arm64): built in rust:1-bullseye (glibc 2.31, so binaries
#     run on Ubuntu 20.04+ / Debian 11+)
#   - Windows (x64, msvc): built in the cargo-xwin image
#
# Produces pixelguard-<target>[.exe] files plus SHA256SUMS, ready to attach
# to a GitHub release. Builds use --locked against the committed Cargo.lock.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$(mkdir -p "${1:-$ROOT_DIR/dist}" && cd "${1:-$ROOT_DIR/dist}" && pwd)"
RUST_LINUX_IMAGE="rust:1-bullseye"
XWIN_IMAGE="messense/cargo-xwin"

[ "$(uname -s)" = "Darwin" ] || { echo "❌ macOS binaries can only be built on macOS."; exit 1; }
docker info >/dev/null 2>&1 || { echo "❌ Docker is not running."; exit 1; }

cd "$ROOT_DIR"

echo "═══ macOS ═══"
for target in aarch64-apple-darwin x86_64-apple-darwin; do
    rustup target add "$target" >/dev/null
    cargo build --release --locked --target "$target"
    cp "target/$target/release/pixelguard" "$OUT_DIR/pixelguard-$target"
done

echo "═══ Linux ═══"
docker run --rm \
    -v "$ROOT_DIR":/src:ro \
    -v pixelguard-cargo-registry:/usr/local/cargo/registry \
    -v pixelguard-linux-target:/target \
    -v "$OUT_DIR":/out \
    -e CARGO_TARGET_DIR=/target \
    -w /src \
    "$RUST_LINUX_IMAGE" bash -euo pipefail -c '
        apt-get update -qq
        apt-get install -y -qq gcc-x86-64-linux-gnu gcc-aarch64-linux-gnu g++-x86-64-linux-gnu g++-aarch64-linux-gnu >/dev/null
        export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
        export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++
        export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
        for target in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
            rustup target add "$target" >/dev/null
            cargo build --release --locked --target "$target"
            cp "/target/$target/release/pixelguard" "/out/pixelguard-$target"
        done
    '

echo "═══ Windows ═══"
docker run --rm --platform linux/amd64 \
    -v "$ROOT_DIR":/src:ro \
    -v pixelguard-cargo-registry-amd64:/usr/local/cargo/registry \
    -v pixelguard-windows-target:/target \
    -v pixelguard-xwin-cache:/root/.cache/cargo-xwin \
    -v "$OUT_DIR":/out \
    -e CARGO_TARGET_DIR=/target \
    -w /src \
    "$XWIN_IMAGE" bash -euo pipefail -c '
        rustup target add x86_64-pc-windows-msvc >/dev/null
        cargo xwin build --release --locked --target x86_64-pc-windows-msvc
        cp /target/x86_64-pc-windows-msvc/release/pixelguard.exe /out/pixelguard-x86_64-pc-windows-msvc.exe
    '

echo "═══ Checksums ═══"
cd "$OUT_DIR"
shasum -a 256 pixelguard-* > SHA256SUMS
cat SHA256SUMS
