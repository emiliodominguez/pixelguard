#!/usr/bin/env node

/**
 * @fileoverview Runner script for Pixelguard npm package.
 *
 * Locates and executes the prebuilt Pixelguard binary, forwarding all
 * command-line arguments. First checks the package's bin directory,
 * then falls back to searching the system PATH.
 *
 * @module run
 * @author Pixelguard Contributors
 * @license MIT
 */

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

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
 * Searches for the Pixelguard binary in known locations.
 *
 * Search order:
 * 1. Package bin directory (npm/pixelguard/bin/)
 * 2. System PATH directories
 *
 * @returns {string|null} Absolute path to the binary, or null if not found
 * @example
 * const binary = findBinary();
 * if (binary) {
 *   console.log(`Found binary at: ${binary}`);
 * }
 */
function findBinary() {
	const binaryName = getBinaryName();

	// Check in package bin directory
	const packageBin = path.join(__dirname, "..", "bin", binaryName);
	if (fs.existsSync(packageBin)) {
		return packageBin;
	}

	// Check in PATH
	const pathDirs = process.env.PATH?.split(path.delimiter) || [];
	for (const dir of pathDirs) {
		const candidate = path.join(dir, binaryName);
		if (fs.existsSync(candidate)) {
			return candidate;
		}
	}

	return null;
}

/**
 * Main entry point for the runner script.
 *
 * Finds the Pixelguard binary and spawns it as a child process,
 * forwarding all command-line arguments. Inherits stdio from the
 * parent process for seamless terminal interaction.
 *
 * Exit codes:
 * - 0: Success (or binary exit code 0)
 * - 1: Binary not found or execution error
 * - Other: Forwarded from the binary process
 *
 * @returns {void}
 */
function main() {
	const binaryPath = findBinary();

	if (!binaryPath) {
		console.error("Error: Pixelguard binary not found.");
		console.error("");
		console.error("Try reinstalling the package:");
		console.error("  npm install pixelguard");
		console.error("");
		console.error("Or build from source:");
		console.error("  git clone https://github.com/emiliodominguez/pixelguard.git");
		console.error("  cd pixelguard && cargo build --release");
		process.exit(1);
	}

	// Forward all arguments to the binary
	const args = process.argv.slice(2);

	const child = spawn(binaryPath, args, {
		stdio: "inherit",
		windowsHide: true,
	});

	child.on("error", (err) => {
		console.error(`Failed to execute Pixelguard: ${err.message}`);
		process.exit(1);
	});

	child.on("exit", (code) => {
		process.exit(code ?? 0);
	});
}

main();
