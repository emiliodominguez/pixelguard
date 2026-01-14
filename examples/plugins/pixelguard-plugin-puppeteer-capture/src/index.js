/**
 * Pixelguard Puppeteer Capture Plugin
 *
 * Uses Puppeteer instead of Playwright for screenshot capture.
 * Useful when you already have Puppeteer in your project or prefer its API.
 *
 * Configuration:
 * {
 *   "plugins": ["pixelguard-plugin-puppeteer-capture"],
 *   "pluginOptions": {
 *     "pixelguard-plugin-puppeteer-capture": {
 *       "headless": true,
 *       "timeout": 30000,
 *       "fullPage": false
 *     }
 *   }
 * }
 */

const puppeteer = require("puppeteer");
const path = require("path");
const fs = require("fs");

/**
 * Capture screenshots using Puppeteer.
 *
 * @param {Object} input - Capture input
 * @param {Array} input.shots - Shots to capture
 * @param {string} input.baseUrl - Base URL of the dev server
 * @param {Object} input.viewport - Viewport dimensions
 * @param {string} input.outputDir - Output directory for screenshots
 * @param {Object} input.options - Plugin options
 * @returns {Promise<{captured: Array, failed: Array}>}
 */
async function capture(input) {
	const { shots, baseUrl, viewport, outputDir, options } = input;

	// Ensure output directory exists
	if (!fs.existsSync(outputDir)) {
		fs.mkdirSync(outputDir, { recursive: true });
	}

	const results = {
		captured: [],
		failed: [],
	};

	// Launch browser
	const browser = await puppeteer.launch({
		headless: options.headless !== false ? "new" : false,
		args: ["--no-sandbox", "--disable-setuid-sandbox", "--disable-dev-shm-usage", "--disable-gpu"],
	});

	const timeout = options.timeout || 30000;

	try {
		// Process shots sequentially to avoid overwhelming the browser
		for (const shot of shots) {
			try {
				const page = await browser.newPage();

				// Set viewport
				await page.setViewport({
					width: viewport.width,
					height: viewport.height,
					deviceScaleFactor: options.deviceScaleFactor || 1,
				});

				// Set default timeout
				page.setDefaultTimeout(timeout);

				// Build full URL
				const url = `${baseUrl}${shot.path}`;

				// Navigate to page
				await page.goto(url, {
					waitUntil: options.waitUntil || "networkidle0",
					timeout,
				});

				// Wait for custom selector if specified
				if (shot.waitFor) {
					await page.waitForSelector(shot.waitFor, { timeout });
				}

				// Apply custom delay if specified
				if (shot.delay && shot.delay > 0) {
					await new Promise((resolve) => setTimeout(resolve, shot.delay));
				}

				// Take screenshot
				const screenshotPath = path.join(outputDir, `${shot.name}.png`);
				await page.screenshot({
					path: screenshotPath,
					fullPage: options.fullPage || false,
					type: "png",
				});

				results.captured.push({
					name: shot.name,
					path: screenshotPath,
				});

				await page.close();
			} catch (error) {
				results.failed.push({
					name: shot.name,
					error: error.message || String(error),
				});
			}
		}
	} finally {
		await browser.close();
	}

	return results;
}

module.exports = { capture };
