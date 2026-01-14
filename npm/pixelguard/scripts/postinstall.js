#!/usr/bin/env node

/**
 * @fileoverview Postinstall script for Pixelguard npm package.
 *
 * Downloads the correct prebuilt binary for the current platform from GitHub releases.
 * Supports macOS (x64/arm64), Linux (x64/arm64), and Windows (x64).
 *
 * @module postinstall
 * @author Pixelguard Contributors
 * @license MIT
 */

const fs = require("fs");
const path = require("path");
const https = require("https");

/**
 * Package version from package.json, used to determine the release to download.
 * @constant {string}
 */
const PACKAGE_VERSION = require("../package.json").version;

/**
 * GitHub repository path for release downloads.
 * @constant {string}
 */
const GITHUB_REPO = "pixelguard/pixelguard";

/**
 * Mapping of Node.js platform/arch combinations to Rust target triples.
 * @constant {Object<string, string>}
 */
const PLATFORM_MAP = {
	"darwin-x64": "x86_64-apple-darwin",
	"darwin-arm64": "aarch64-apple-darwin",
	"linux-x64": "x86_64-unknown-linux-gnu",
	"linux-arm64": "aarch64-unknown-linux-gnu",
	"win32-x64": "x86_64-pc-windows-msvc",
};

/**
 * Gets the platform key for the current system.
 *
 * @returns {string} Platform key in format "{platform}-{arch}"
 * @example
 * // On macOS ARM
 * getPlatformKey() // => "darwin-arm64"
 */
function getPlatformKey() {
	return `${process.platform}-${process.arch}`;
}

/**
 * Gets the Rust target triple for the current platform.
 *
 * @returns {string} Rust target triple (e.g., "x86_64-apple-darwin")
 * @throws {Error} Exits process with code 1 if platform is unsupported
 * @example
 * // On Linux x64
 * getTargetTriple() // => "x86_64-unknown-linux-gnu"
 */
function getTargetTriple() {
	const key = getPlatformKey();
	const triple = PLATFORM_MAP[key];

	if (!triple) {
		console.error(`Unsupported platform: ${key}`);
		console.error(`Supported platforms: ${Object.keys(PLATFORM_MAP).join(", ")}`);
		process.exit(1);
	}

	return triple;
}

/**
 * Gets the binary filename for the current platform.
 *
 * @returns {string} Binary filename ("pixelguard" or "pixelguard.exe" on Windows)
 * @example
 * // On Windows
 * getBinaryName() // => "pixelguard.exe"
 * // On Unix
 * getBinaryName() // => "pixelguard"
 */
function getBinaryName() {
	return process.platform === "win32" ? "pixelguard.exe" : "pixelguard";
}

/**
 * Constructs the GitHub release download URL for the binary.
 *
 * @param {string} triple - Rust target triple
 * @returns {string} Full download URL for the binary
 * @example
 * getDownloadUrl("x86_64-apple-darwin")
 * // => "https://github.com/pixelguard/pixelguard/releases/download/v0.1.0/pixelguard-x86_64-apple-darwin"
 */
function getDownloadUrl(triple) {
	return `https://github.com/${GITHUB_REPO}/releases/download/v${PACKAGE_VERSION}/pixelguard-${triple}${
		process.platform === "win32" ? ".exe" : ""
	}`;
}

/**
 * Downloads a file from a URL to a local destination.
 * Handles HTTP redirects automatically.
 *
 * @param {string} url - URL to download from
 * @param {string} dest - Local file path to save to
 * @returns {Promise<void>} Resolves when download completes
 * @throws {Error} Rejects if download fails or returns non-200 status
 * @example
 * await downloadFile("https://example.com/file.bin", "/tmp/file.bin");
 */
function downloadFile(url, dest) {
	return new Promise((resolve, reject) => {
		const file = fs.createWriteStream(dest);

		const request = (url) => {
			https
				.get(url, (response) => {
					// Handle redirects
					if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
						request(response.headers.location);
						return;
					}

					if (response.statusCode !== 200) {
						reject(new Error(`Failed to download: ${response.statusCode}`));
						return;
					}

					response.pipe(file);
					file.on("finish", () => {
						file.close(resolve);
					});
				})
				.on("error", (err) => {
					fs.unlink(dest, () => {});
					reject(err);
				});
		};

		request(url);
	});
}

/**
 * Main entry point for the postinstall script.
 *
 * Downloads the Pixelguard binary for the current platform if it doesn't exist.
 * On failure, prints instructions for building from source and exits gracefully
 * (exit code 0) to not block npm install.
 *
 * @async
 * @returns {Promise<void>}
 */
async function main() {
	const triple = getTargetTriple();
	const binaryName = getBinaryName();
	const binDir = path.join(__dirname, "..", "bin");
	const binaryPath = path.join(binDir, binaryName);

	// Create bin directory
	if (!fs.existsSync(binDir)) {
		fs.mkdirSync(binDir, { recursive: true });
	}

	// Check if binary already exists
	if (fs.existsSync(binaryPath)) {
		console.log("Pixelguard binary already exists, skipping download.");
		return;
	}

	const url = getDownloadUrl(triple);
	console.log(`Downloading Pixelguard for ${triple}...`);
	console.log(`URL: ${url}`);

	try {
		await downloadFile(url, binaryPath);

		// Make executable on Unix
		if (process.platform !== "win32") {
			fs.chmodSync(binaryPath, 0o755);
		}

		console.log("Pixelguard installed successfully!");
	} catch (err) {
		console.error(`Failed to download Pixelguard: ${err.message}`);
		console.error("");
		console.error("You can try building from source:");
		console.error("  git clone https://github.com/pixelguard/pixelguard.git");
		console.error("  cd pixelguard && cargo build --release");
		console.error("");

		// Don't fail the install - user can still build from source
		process.exit(0);
	}
}

main();
