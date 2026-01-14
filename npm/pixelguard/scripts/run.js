#!/usr/bin/env node

/**
 * Runner script for Pixelguard npm package.
 *
 * Executes the prebuilt binary with the provided arguments.
 */

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

function getBinaryName() {
	return process.platform === "win32" ? "pixelguard.exe" : "pixelguard";
}

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

function main() {
	const binaryPath = findBinary();

	if (!binaryPath) {
		console.error("Error: Pixelguard binary not found.");
		console.error("");
		console.error("Try reinstalling the package:");
		console.error("  npm install pixelguard");
		console.error("");
		console.error("Or build from source:");
		console.error("  git clone https://github.com/pixelguard/pixelguard.git");
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
