/**
 * Pixelguard JSON Reporter Plugin
 *
 * Generates a machine-readable JSON report of visual regression test results.
 * Useful for CI/CD integration and custom dashboards.
 *
 * Configuration:
 * {
 *   "plugins": ["pixelguard-plugin-json-reporter"],
 *   "pluginOptions": {
 *     "pixelguard-plugin-json-reporter": {
 *       "filename": "report.json",
 *       "pretty": true
 *     }
 *   }
 * }
 */

const fs = require("fs");
const path = require("path");

/**
 * Generate a JSON report from diff results.
 *
 * @param {Object} input - Reporter input
 * @param {Object} input.result - Diff results
 * @param {Object} input.config - Pixelguard configuration
 * @param {string} input.outputDir - Output directory path
 * @param {Object} input.options - Plugin options
 * @returns {Promise<Object>} Report output with path
 */
async function generate(input) {
	const { result, config, outputDir, options } = input;

	// Build the report object
	const report = {
		timestamp: new Date().toISOString(),
		summary: {
			total: result.unchanged.length + result.changed.length + result.added.length + result.removed.length,
			unchanged: result.unchanged.length,
			changed: result.changed.length,
			added: result.added.length,
			removed: result.removed.length,
			passed: result.changed.length === 0 && result.added.length === 0 && result.removed.length === 0,
		},
		config: {
			source: config.source,
			baseUrl: config.baseUrl,
			threshold: config.threshold,
		},
		results: {
			unchanged: result.unchanged,
			changed: result.changed.map((shot) => ({
				name: shot.name,
				diffPercentage: shot.diffPercentage,
				baselinePath: shot.baselinePath,
				currentPath: shot.currentPath,
				diffPath: shot.diffPath,
			})),
			added: result.added,
			removed: result.removed,
		},
	};

	// Determine output filename
	const filename = options.filename || "report.json";
	const reportPath = path.join(outputDir, filename);

	// Format JSON (pretty or compact)
	const pretty = options.pretty !== false;
	const json = pretty ? JSON.stringify(report, null, 2) : JSON.stringify(report);

	// Write the report
	fs.writeFileSync(reportPath, json);

	console.error(`JSON report written to: ${reportPath}`);

	return { reportPath };
}

module.exports = { generate };
