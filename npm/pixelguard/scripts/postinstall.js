#!/usr/bin/env node

/**
 * @fileoverview Postinstall script for Pixelguard npm package.
 *
 * Downloads the prebuilt binary for the current platform from GitHub releases and
 * verifies it against the SHA-256 checksums shipped inside this npm package before
 * it is ever made executable. The download goes to a temporary file that is only
 * renamed into place after verification, so an interrupted or tampered download can
 * never be left behind as the binary.
 *
 * @module postinstall
 * @author Pixelguard Contributors
 * @license MIT
 */

const crypto = require("crypto");
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
const GITHUB_REPO = "emiliodominguez/pixelguard";

/**
 * Maximum number of HTTP redirects to follow.
 * @constant {number}
 */
const MAX_REDIRECTS = 5;

/**
 * Socket inactivity timeout for the download, in milliseconds.
 * @constant {number}
 */
const DOWNLOAD_TIMEOUT_MS = 60_000;

/**
 * Upper bound for the binary size, in bytes.
 * @constant {number}
 */
const MAX_BINARY_BYTES = 200 * 1024 * 1024;

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
 * Gets the Rust target triple for the current platform.
 *
 * @returns {string | null} Rust target triple, or null if the platform is unsupported
 */
function getTargetTriple() {
	return PLATFORM_MAP[`${process.platform}-${process.arch}`] ?? null;
}

/**
 * Gets the binary filename for the current platform.
 *
 * @returns {string} "pixelguard.exe" on Windows, "pixelguard" elsewhere
 */
function getBinaryName() {
	return process.platform === "win32" ? "pixelguard.exe" : "pixelguard";
}

/**
 * Gets the release asset name for a target triple.
 *
 * @param {string} triple - Rust target triple
 * @returns {string} Asset name, e.g. "pixelguard-aarch64-apple-darwin"
 */
function getAssetName(triple) {
	return `pixelguard-${triple}${process.platform === "win32" ? ".exe" : ""}`;
}

/**
 * Reads the expected SHA-256 for an asset from the checksums bundled in this package.
 *
 * @param {string} assetName - Release asset name
 * @returns {string | null} Lowercase hex digest, or null when no checksum is known
 */
function getExpectedChecksum(assetName) {
	try {
		const checksums = JSON.parse(fs.readFileSync(path.join(__dirname, "..", "checksums.json"), "utf8"));
		const digest = checksums[assetName];
		return typeof digest === "string" && /^[a-f0-9]{64}$/i.test(digest) ? digest.toLowerCase() : null;
	} catch {
		return null;
	}
}

/**
 * Computes the SHA-256 of a file.
 *
 * @param {string} filePath - File to hash
 * @returns {string} Lowercase hex digest
 */
function sha256File(filePath) {
	return crypto.createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

/**
 * Downloads a URL to a file over HTTPS, following a bounded number of HTTPS redirects.
 *
 * @param {string} url - HTTPS URL to download
 * @param {string} dest - Destination file path (created exclusively)
 * @param {number} [redirectsLeft] - Remaining redirects allowed
 * @returns {Promise<void>} Resolves when the file is completely written
 */
function downloadFile(url, dest, redirectsLeft = MAX_REDIRECTS) {
	return new Promise((resolve, reject) => {
		if (!url.startsWith("https://")) {
			reject(new Error(`Refusing to download over a non-HTTPS URL: ${url}`));
			return;
		}

		const request = https.get(url, (response) => {
			const { statusCode, headers } = response;

			if (statusCode >= 300 && statusCode < 400 && headers.location) {
				response.resume();
				if (redirectsLeft <= 0) {
					reject(new Error("Too many redirects"));
					return;
				}
				const next = new URL(headers.location, url).toString();
				downloadFile(next, dest, redirectsLeft - 1).then(resolve, reject);
				return;
			}

			if (statusCode !== 200) {
				response.resume();
				reject(new Error(`Download failed with HTTP ${statusCode}`));
				return;
			}

			let received = 0;
			const file = fs.createWriteStream(dest, { flags: "wx", mode: 0o600 });
			response.on("data", (chunk) => {
				received += chunk.length;
				if (received > MAX_BINARY_BYTES) {
					request.destroy(new Error("Download exceeds the maximum binary size"));
				}
			});
			response.on("error", reject);
			file.on("error", reject);
			file.on("finish", () => file.close(resolve));
			response.pipe(file);
		});

		request.setTimeout(DOWNLOAD_TIMEOUT_MS, () => request.destroy(new Error("Download timed out")));
		request.on("error", reject);
	});
}

/**
 * Prints instructions for installing without the prebuilt binary.
 *
 * @param {string} reason - Why the prebuilt binary is not available
 */
function printFallback(reason) {
	console.error(`Pixelguard binary not installed: ${reason}`);
	console.error("");
	console.error("You can build it from source and point the wrapper at it:");
	console.error("  git clone https://github.com/emiliodominguez/pixelguard.git");
	console.error("  cd pixelguard && cargo build --release");
	console.error("  export PIXELGUARD_BINARY=$PWD/target/release/pixelguard");
}

/**
 * Downloads, verifies and installs the binary for this platform into the package's bin directory.
 *
 * Shared by postinstall and by the wrapper, which calls it on first run when the package
 * manager skipped install scripts (npm 12+ does so by default for dependencies).
 *
 * @param {object} [settings] - Install settings
 * @param {(message: string) => void} [settings.log] - Progress logger
 * @returns {Promise<string>} Path to the verified binary
 * @throws {Error} When the platform is unsupported, no checksum is known, or the download fails verification
 */
async function install({ log = console.log } = {}) {
	const triple = getTargetTriple();
	if (!triple) {
		throw new Error(`unsupported platform ${process.platform}-${process.arch}`);
	}

	const assetName = getAssetName(triple);
	const expected = getExpectedChecksum(assetName);
	if (!expected) {
		throw new Error(`no published checksum for ${assetName} in this package version`);
	}

	const binDir = path.join(__dirname, "..", "bin");
	const binaryPath = path.join(binDir, getBinaryName());
	fs.mkdirSync(binDir, { recursive: true });

	if (fs.existsSync(binaryPath) && sha256File(binaryPath) === expected) {
		log("Pixelguard binary already installed and verified.");
		return binaryPath;
	}

	const url = `https://github.com/${GITHUB_REPO}/releases/download/v${PACKAGE_VERSION}/${assetName}`;
	const tempPath = `${binaryPath}.download-${process.pid}-${crypto.randomBytes(4).toString("hex")}`;
	log(`Downloading Pixelguard for ${triple}...`);

	try {
		await downloadFile(url, tempPath);

		const actual = sha256File(tempPath);
		if (actual !== expected) {
			throw new Error(`checksum mismatch for ${assetName} (expected ${expected}, got ${actual})`);
		}

		if (process.platform !== "win32") {
			fs.chmodSync(tempPath, 0o755);
		}
		fs.renameSync(tempPath, binaryPath);
		log("Pixelguard installed and verified successfully!");
		return binaryPath;
	} finally {
		fs.rmSync(tempPath, { force: true });
	}
}

/**
 * Main entry point for the postinstall script.
 *
 * Never fails the surrounding `npm install`; the wrapper retries the download on first run.
 *
 * @returns {Promise<void>}
 */
async function main() {
	try {
		await install();
	} catch (err) {
		printFallback(err.message);
	}
}

if (require.main === module) {
	main();
}

module.exports = { install, downloadFile, getAssetName, getExpectedChecksum, sha256File };
