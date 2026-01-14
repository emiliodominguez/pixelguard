#!/usr/bin/env node

/**
 * Postinstall script for Pixelguard npm package.
 *
 * Downloads the correct prebuilt binary for the current platform.
 */

const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const PACKAGE_VERSION = require("../package.json").version;
const GITHUB_REPO = "pixelguard/pixelguard";

// Map Node.js platform/arch to Rust target triples
const PLATFORM_MAP = {
	"darwin-x64": "x86_64-apple-darwin",
	"darwin-arm64": "aarch64-apple-darwin",
	"linux-x64": "x86_64-unknown-linux-gnu",
	"linux-arm64": "aarch64-unknown-linux-gnu",
	"win32-x64": "x86_64-pc-windows-msvc",
};

function getPlatformKey() {
	return `${process.platform}-${process.arch}`;
}

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

function getBinaryName() {
	return process.platform === "win32" ? "pixelguard.exe" : "pixelguard";
}

function getDownloadUrl(triple) {
	const binaryName = getBinaryName();

	return `https://github.com/${GITHUB_REPO}/releases/download/v${PACKAGE_VERSION}/pixelguard-${triple}${
		process.platform === "win32" ? ".exe" : ""
	}`;
}

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
