#!/usr/bin/env node

/**
 * @fileoverview Runner script for Pixelguard npm package.
 *
 * Locates and executes the Pixelguard binary, forwarding all command-line arguments
 * and the exit status. Lookup order: the binary installed by postinstall, then
 * `PIXELGUARD_BINARY`, then the system PATH.
 *
 * The PATH lookup skips this wrapper itself (npm puts `node_modules/.bin` on PATH,
 * and `.bin/pixelguard` links back to this script), and a guard variable stops any
 * remaining chance of the wrapper re-executing itself in a loop.
 *
 * @module run
 * @author Pixelguard Contributors
 * @license MIT
 */

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

/**
 * Environment variable set for the child so a nested wrapper refuses to recurse.
 * @constant {string}
 */
const GUARD_VARIABLE = "PIXELGUARD_WRAPPER_ACTIVE";

/**
 * Gets the binary filename for the current platform.
 *
 * @returns {string} "pixelguard.exe" on Windows, "pixelguard" elsewhere
 */
function getBinaryName() {
	return process.platform === "win32" ? "pixelguard.exe" : "pixelguard";
}

/**
 * Resolves symlinks, returning null when the path does not exist.
 *
 * @param {string} candidate - Path to resolve
 * @returns {string | null} Real path, or null
 */
function realpathOrNull(candidate) {
	try {
		return fs.realpathSync(candidate);
	} catch {
		return null;
	}
}

/**
 * Checks whether a path is a real Pixelguard binary rather than this wrapper or another script.
 *
 * @param {string} candidate - Path to check
 * @returns {boolean} True when the path is an executable file that is not a script
 */
function isNativeBinary(candidate) {
	const real = realpathOrNull(candidate);
	if (!real || real === realpathOrNull(__filename)) {
		return false;
	}
	try {
		if (!fs.statSync(real).isFile()) {
			return false;
		}
		const fd = fs.openSync(real, "r");
		const header = Buffer.alloc(2);
		fs.readSync(fd, header, 0, 2, 0);
		fs.closeSync(fd);
		return header.toString("latin1") !== "#!";
	} catch {
		return false;
	}
}

/**
 * Searches for the Pixelguard binary in known locations.
 *
 * @returns {string | null} Absolute path to the binary, or null if not found
 */
function findBinary() {
	const binaryName = getBinaryName();

	const packageBin = path.join(__dirname, "..", "bin", binaryName);
	if (isNativeBinary(packageBin)) {
		return packageBin;
	}

	const override = process.env.PIXELGUARD_BINARY;
	if (override && isNativeBinary(override)) {
		return path.resolve(override);
	}

	const pathDirs = (process.env.PATH || "").split(path.delimiter).filter(Boolean);
	for (const dir of pathDirs) {
		const candidate = path.join(dir, binaryName);
		if (isNativeBinary(candidate)) {
			return candidate;
		}
	}

	return null;
}

/**
 * Maps a child's exit to this process's exit, re-raising fatal signals.
 *
 * A child killed by a signal must never look like success to CI.
 *
 * @param {number | null} code - Child exit code
 * @param {NodeJS.Signals | null} signal - Signal that terminated the child
 */
function exitLikeChild(code, signal) {
	if (signal) {
		process.removeAllListeners(signal);
		process.kill(process.pid, signal);
		const signalNumber = require("os").constants.signals[signal] ?? 1;
		process.exit(128 + signalNumber);
	}
	process.exit(code ?? 1);
}

/**
 * Main entry point for the runner script.
 */
function main() {
	if (process.env[GUARD_VARIABLE]) {
		console.error("Error: the Pixelguard wrapper resolved to itself; refusing to recurse.");
		console.error("Set PIXELGUARD_BINARY to the path of the pixelguard binary.");
		process.exit(1);
	}

	const binaryPath = findBinary();

	if (!binaryPath) {
		console.error("Error: Pixelguard binary not found.");
		console.error("");
		console.error("Try reinstalling the package:");
		console.error("  npm install pixelguard");
		console.error("");
		console.error("Or build from source and set PIXELGUARD_BINARY:");
		console.error("  git clone https://github.com/emiliodominguez/pixelguard.git");
		console.error("  cd pixelguard && cargo build --release");
		console.error("  export PIXELGUARD_BINARY=$PWD/target/release/pixelguard");
		process.exit(1);
	}

	const child = spawn(binaryPath, process.argv.slice(2), {
		stdio: "inherit",
		windowsHide: true,
		env: { ...process.env, [GUARD_VARIABLE]: "1" },
	});

	for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
		process.on(signal, () => child.kill(signal));
	}

	child.on("error", (err) => {
		console.error(`Failed to execute Pixelguard: ${err.message}`);
		process.exit(1);
	});

	child.on("exit", exitLikeChild);
}

main();
