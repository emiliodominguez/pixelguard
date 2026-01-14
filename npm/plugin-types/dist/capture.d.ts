/**
 * Capture plugin types.
 *
 * Capture plugins provide alternative screenshot engines.
 * By default, Pixelguard uses Playwright, but plugins can use
 * Puppeteer, Cypress, or custom solutions.
 */
/**
 * A shot to capture.
 */
export interface CaptureShot {
	/** Shot name */
	name: string;
	/** URL path (appended to baseUrl) */
	path: string;
	/** Optional CSS selector to wait for before capturing */
	waitFor?: string;
	/** Optional delay in milliseconds after page load */
	delay?: number;
}
/**
 * Viewport dimensions.
 */
export interface CaptureViewport {
	/** Width in pixels */
	width: number;
	/** Height in pixels */
	height: number;
}
/**
 * Input for capture operation.
 */
export interface CaptureInput {
	/** Shots to capture */
	shots: CaptureShot[];
	/** Base URL of the dev server */
	baseUrl: string;
	/** Viewport dimensions */
	viewport: CaptureViewport;
	/** Output directory for screenshots */
	outputDir: string;
	/** Plugin options from config */
	options: Record<string, unknown>;
}
/**
 * A successfully captured shot.
 */
export interface CapturedShot {
	/** Shot name */
	name: string;
	/** Path to the screenshot file */
	path: string;
}
/**
 * A shot that failed to capture.
 */
export interface FailedShot {
	/** Shot name */
	name: string;
	/** Error message */
	error: string;
}
/**
 * Output from capture operation.
 */
export interface CaptureOutput {
	/** Successfully captured shots */
	captured: CapturedShot[];
	/** Failed shots */
	failed: FailedShot[];
}
/**
 * Capture plugin interface.
 *
 * Implement the capture hook to provide a custom screenshot engine.
 *
 * @example
 * ```typescript
 * import type { CapturePlugin, CaptureInput, CaptureOutput } from '@pixelguard/plugin-types';
 * import puppeteer from 'puppeteer';
 *
 * export const capture: CapturePlugin['capture'] = async (input: CaptureInput): Promise<CaptureOutput> => {
 *   const browser = await puppeteer.launch({ headless: true });
 *   const results: CaptureOutput = { captured: [], failed: [] };
 *
 *   for (const shot of input.shots) {
 *     try {
 *       const page = await browser.newPage();
 *       await page.setViewport(input.viewport);
 *       await page.goto(`${input.baseUrl}${shot.path}`);
 *
 *       if (shot.waitFor) {
 *         await page.waitForSelector(shot.waitFor);
 *       }
 *       if (shot.delay) {
 *         await new Promise(r => setTimeout(r, shot.delay));
 *       }
 *
 *       const path = `${input.outputDir}/${shot.name}.png`;
 *       await page.screenshot({ path });
 *       results.captured.push({ name: shot.name, path });
 *       await page.close();
 *     } catch (error) {
 *       results.failed.push({ name: shot.name, error: String(error) });
 *     }
 *   }
 *
 *   await browser.close();
 *   return results;
 * };
 * ```
 */
export interface CapturePlugin {
	/**
	 * Capture screenshots for the given shots.
	 */
	capture: (input: CaptureInput) => Promise<CaptureOutput>;
}
//# sourceMappingURL=capture.d.ts.map
