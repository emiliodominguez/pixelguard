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
 * @param {Object} input.viewport - Viewport dimensions (legacy single viewport)
 * @param {Array<{name: string, width: number, height: number}>} [input.viewports] - Every viewport to capture
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
		headless: options.headless !== false,
		args: ["--no-sandbox", "--disable-setuid-sandbox", "--disable-dev-shm-usage", "--disable-gpu"],
	});

	const timeout = options.timeout || 30000;

	// Older Pixelguard versions only send `viewport`; newer ones send every viewport.
	const viewports = input.viewports && input.viewports.length > 0 ? input.viewports : [{ name: "default", ...viewport }];

	try {
		// Process shots sequentially to avoid overwhelming the browser
		for (const shot of shots) {
			for (const vp of viewports) {
				// Pixelguard expects `${shot}.png` for the default viewport and `${shot}@${viewport}.png` otherwise.
				const name = vp.name === "default" ? shot.name : `${shot.name}@${vp.name}`;
				let page;
				try {
					page = await browser.newPage();

					await page.setViewport({
						width: vp.width,
						height: vp.height,
						deviceScaleFactor: options.deviceScaleFactor || 1,
					});

					page.setDefaultTimeout(timeout);

					await page.goto(`${baseUrl}${shot.path}`, {
						waitUntil: options.waitUntil || "networkidle0",
						timeout,
					});

					if (shot.waitFor) {
						await page.waitForSelector(shot.waitFor, { timeout });
					}

					if (shot.delay && shot.delay > 0) {
						await new Promise((resolve) => setTimeout(resolve, shot.delay));
					}

					const screenshotPath = path.join(outputDir, `${name}.png`);
					await page.screenshot({
						path: screenshotPath,
						fullPage: options.fullPage || false,
						type: "png",
					});

					results.captured.push({ name, path: screenshotPath });
				} catch (error) {
					results.failed.push({
						name,
						error: error.message || String(error),
					});
				} finally {
					if (page) {
						await page.close().catch(() => {});
					}
				}
			}
		}
	} finally {
		await browser.close();
	}

	return results;
}

module.exports = { capture };
