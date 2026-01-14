/**
 * Pixelguard SSIM Differ Plugin
 *
 * Uses Structural Similarity Index (SSIM) for image comparison instead of
 * pixel-by-pixel comparison. SSIM is more perceptually accurate and better
 * handles anti-aliasing, compression artifacts, and minor rendering differences.
 *
 * Configuration:
 * {
 *   "plugins": ["pixelguard-plugin-ssim-differ"],
 *   "pluginOptions": {
 *     "pixelguard-plugin-ssim-differ": {
 *       "windowSize": 11,
 *       "k1": 0.01,
 *       "k2": 0.03
 *     }
 *   }
 * }
 */

const sharp = require("sharp");
const { ssim } = require("ssim.js");
const fs = require("fs");

/**
 * Compare two images using SSIM algorithm.
 *
 * @param {Object} input - Differ input
 * @param {string} input.baselinePath - Path to baseline image
 * @param {string} input.currentPath - Path to current image
 * @param {string} input.diffPath - Path to write diff image
 * @param {number} input.threshold - Diff threshold (0.0 to 1.0)
 * @param {Object} input.options - Plugin options
 * @returns {Promise<{diffPercentage: number, matches: boolean}>}
 */
async function compare(input) {
	const { baselinePath, currentPath, diffPath, threshold, options } = input;

	// Load images with sharp
	const [baselineData, currentData] = await Promise.all([loadImage(baselinePath), loadImage(currentPath)]);

	// Check for size mismatch
	if (baselineData.width !== currentData.width || baselineData.height !== currentData.height) {
		// Generate a diff image showing size mismatch
		await generateSizeMismatchDiff(baselineData, currentData, diffPath);

		return {
			diffPercentage: 100,
			matches: false,
		};
	}

	// Configure SSIM options
	const ssimOptions = {
		windowSize: options.windowSize || 11,
		k1: options.k1 || 0.01,
		k2: options.k2 || 0.03,
	};

	// Run SSIM comparison
	const result = ssim(
		{
			data: baselineData.data,
			width: baselineData.width,
			height: baselineData.height,
		},
		{
			data: currentData.data,
			width: currentData.width,
			height: currentData.height,
		},
		ssimOptions,
	);

	// Convert SSIM score (0-1, where 1 = identical) to diff percentage
	// SSIM of 1.0 = 0% diff, SSIM of 0.0 = 100% diff
	const diffPercentage = (1 - result.mssim) * 100;
	const matches = diffPercentage <= threshold;

	// Generate diff image showing areas of difference
	if (!matches) {
		await generateDiffImage(baselineData, currentData, result.ssim_map, diffPath, options);
	}

	return {
		diffPercentage: Math.round(diffPercentage * 100) / 100,
		matches,
	};
}

/**
 * Load an image and return raw pixel data.
 *
 * @param {string} imagePath - Path to image
 * @returns {Promise<{data: Buffer, width: number, height: number}>}
 */
async function loadImage(imagePath) {
	const image = sharp(imagePath).raw().ensureAlpha();
	const { data, info } = await image.toBuffer({ resolveWithObject: true });

	return {
		data: new Uint8Array(data),
		width: info.width,
		height: info.height,
	};
}

/**
 * Generate a diff image highlighting areas with low SSIM scores.
 *
 * @param {Object} baseline - Baseline image data
 * @param {Object} current - Current image data
 * @param {Object} ssimMap - SSIM map from comparison
 * @param {string} outputPath - Path to write diff image
 * @param {Object} options - Plugin options
 */
async function generateDiffImage(baseline, current, ssimMap, outputPath, options) {
	const width = baseline.width;
	const height = baseline.height;

	// Create diff image buffer (RGBA)
	const diffData = new Uint8Array(width * height * 4);

	// Threshold for highlighting differences
	const highlightThreshold = options.highlightThreshold || 0.95;

	// Process each pixel
	for (let y = 0; y < height; y++) {
		for (let x = 0; x < width; x++) {
			const pixelIndex = (y * width + x) * 4;

			// Get SSIM value for this region (ssim_map is lower resolution)
			const mapX = Math.min(Math.floor(x / (options.windowSize || 11)), ssimMap.width - 1);
			const mapY = Math.min(Math.floor(y / (options.windowSize || 11)), ssimMap.height - 1);
			const ssimValue = ssimMap.data[mapY * ssimMap.width + mapX];

			// Get current pixel
			const r = current.data[pixelIndex];
			const g = current.data[pixelIndex + 1];
			const b = current.data[pixelIndex + 2];
			const a = current.data[pixelIndex + 3];

			if (ssimValue < highlightThreshold) {
				// Highlight differences in red
				const intensity = 1 - ssimValue;
				diffData[pixelIndex] = Math.min(255, r + 200 * intensity); // Red
				diffData[pixelIndex + 1] = Math.max(0, g - 100 * intensity); // Green
				diffData[pixelIndex + 2] = Math.max(0, b - 100 * intensity); // Blue
				diffData[pixelIndex + 3] = a;
			} else {
				// Dim unchanged areas
				diffData[pixelIndex] = Math.floor(r * 0.5);
				diffData[pixelIndex + 1] = Math.floor(g * 0.5);
				diffData[pixelIndex + 2] = Math.floor(b * 0.5);
				diffData[pixelIndex + 3] = a;
			}
		}
	}

	// Write diff image
	await sharp(Buffer.from(diffData), {
		raw: {
			width,
			height,
			channels: 4,
		},
	})
		.png()
		.toFile(outputPath);
}

/**
 * Generate a diff image for size mismatch.
 *
 * @param {Object} baseline - Baseline image data
 * @param {Object} current - Current image data
 * @param {string} outputPath - Path to write diff image
 */
async function generateSizeMismatchDiff(baseline, current, outputPath) {
	// Create a simple diff showing both images side by side
	const maxWidth = Math.max(baseline.width, current.width);
	const maxHeight = Math.max(baseline.height, current.height);

	// Create red-tinted version of current image to indicate error
	const diffData = new Uint8Array(maxWidth * maxHeight * 4);

	// Fill with red background
	for (let i = 0; i < diffData.length; i += 4) {
		diffData[i] = 255; // Red
		diffData[i + 1] = 0; // Green
		diffData[i + 2] = 0; // Blue
		diffData[i + 3] = 128; // Alpha
	}

	// Overlay current image
	for (let y = 0; y < current.height; y++) {
		for (let x = 0; x < current.width; x++) {
			const srcIndex = (y * current.width + x) * 4;
			const dstIndex = (y * maxWidth + x) * 4;

			diffData[dstIndex] = current.data[srcIndex];
			diffData[dstIndex + 1] = current.data[srcIndex + 1];
			diffData[dstIndex + 2] = current.data[srcIndex + 2];
			diffData[dstIndex + 3] = current.data[srcIndex + 3];
		}
	}

	await sharp(Buffer.from(diffData), {
		raw: {
			width: maxWidth,
			height: maxHeight,
			channels: 4,
		},
	})
		.png()
		.toFile(outputPath);
}

module.exports = { compare };
