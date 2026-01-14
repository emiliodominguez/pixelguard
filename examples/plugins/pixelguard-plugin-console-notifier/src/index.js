/**
 * Pixelguard Console Notifier Plugin
 *
 * A simple notifier that prints a formatted summary to the console.
 * Great for learning how notifier plugins work.
 *
 * Configuration:
 * {
 *   "plugins": ["pixelguard-plugin-console-notifier"],
 *   "pluginOptions": {
 *     "pixelguard-plugin-console-notifier": {
 *       "showDetails": true,
 *       "useEmoji": true
 *     }
 *   }
 * }
 */

/**
 * Send a notification about test results.
 *
 * @param {Object} input - Notifier input
 * @param {Object} input.result - Diff results
 * @param {string} input.reportPath - Path to HTML report
 * @param {boolean} input.ciMode - Whether running in CI
 * @param {Object} input.options - Plugin options
 * @returns {Promise<void>}
 */
async function notify(input) {
	const { result, reportPath, ciMode, options } = input;

	const showDetails = options.showDetails !== false;
	const useEmoji = options.useEmoji !== false;

	// Calculate totals
	const total = result.unchanged.length + result.changed.length + result.added.length + result.removed.length;

	const hasChanges = result.changed.length > 0 || result.added.length > 0 || result.removed.length > 0;

	// Build the output
	const lines = [];

	// Header
	lines.push("");
	lines.push("=".repeat(50));
	lines.push(useEmoji ? "  PIXELGUARD VISUAL REGRESSION RESULTS" : "  PIXELGUARD RESULTS");
	lines.push("=".repeat(50));
	lines.push("");

	// Status
	if (hasChanges) {
		lines.push(useEmoji ? "  Status: CHANGES DETECTED" : "  Status: CHANGES DETECTED");
	} else {
		lines.push(useEmoji ? "  Status: ALL TESTS PASSED" : "  Status: PASSED");
	}
	lines.push("");

	// Summary
	lines.push("  Summary:");
	lines.push(`    Total shots:  ${total}`);
	lines.push(`    Unchanged:    ${result.unchanged.length} ${useEmoji && result.unchanged.length > 0 ? "" : ""}`);
	lines.push(`    Changed:      ${result.changed.length} ${useEmoji && result.changed.length > 0 ? "" : ""}`);
	lines.push(`    Added:        ${result.added.length} ${useEmoji && result.added.length > 0 ? "" : ""}`);
	lines.push(`    Removed:      ${result.removed.length} ${useEmoji && result.removed.length > 0 ? "" : ""}`);
	lines.push("");

	// Details (if enabled and there are changes)
	if (showDetails && hasChanges) {
		if (result.changed.length > 0) {
			lines.push("  Changed shots:");
			for (const shot of result.changed) {
				lines.push(`    - ${shot.name} (${shot.diffPercentage.toFixed(2)}% different)`);
			}
			lines.push("");
		}

		if (result.added.length > 0) {
			lines.push("  Added shots (no baseline):");
			for (const name of result.added) {
				lines.push(`    + ${name}`);
			}
			lines.push("");
		}

		if (result.removed.length > 0) {
			lines.push("  Removed shots (baseline orphaned):");
			for (const name of result.removed) {
				lines.push(`    - ${name}`);
			}
			lines.push("");
		}
	}

	// Report path
	if (reportPath) {
		lines.push(`  Report: ${reportPath}`);
		lines.push("");
	}

	lines.push("=".repeat(50));
	lines.push("");

	// Print to stderr (so it doesn't interfere with JSON output)
	console.error(lines.join("\n"));
}

module.exports = { notify };
