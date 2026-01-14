/**
 * Differ plugin types.
 *
 * Differ plugins provide alternative image comparison algorithms.
 * By default, Pixelguard uses pixel-by-pixel comparison, but plugins
 * can use SSIM, perceptual hashing, or other approaches.
 */
/**
 * Input for differ compare operation.
 */
export interface DifferInput {
	/** Path to baseline image */
	baselinePath: string;
	/** Path to current image */
	currentPath: string;
	/** Path to write diff image */
	diffPath: string;
	/** Diff threshold (0.0 to 1.0) */
	threshold: number;
	/** Plugin options from config */
	options: Record<string, unknown>;
}
/**
 * Output from differ compare operation.
 */
export interface DifferOutput {
	/** Percentage of pixels that differ (0-100) */
	diffPercentage: number;
	/** Whether images match (within threshold) */
	matches: boolean;
}
/**
 * Differ plugin interface.
 *
 * Implement the compare hook to provide a custom comparison algorithm.
 *
 * @example
 * ```typescript
 * import type { DifferPlugin, DifferInput, DifferOutput } from '@pixelguard/plugin-types';
 * import { ssim } from 'ssim.js';
 * import sharp from 'sharp';
 *
 * export const compare: DifferPlugin['compare'] = async (input: DifferInput): Promise<DifferOutput> => {
 *   const baseline = await sharp(input.baselinePath).raw().toBuffer({ resolveWithObject: true });
 *   const current = await sharp(input.currentPath).raw().toBuffer({ resolveWithObject: true });
 *
 *   const result = ssim(
 *     { data: baseline.data, width: baseline.info.width, height: baseline.info.height },
 *     { data: current.data, width: current.info.width, height: current.info.height }
 *   );
 *
 *   const diffPercentage = (1 - result.mssim) * 100;
 *   const matches = diffPercentage <= input.threshold;
 *
 *   // Generate diff image if needed
 *   if (!matches) {
 *     // ... generate diff visualization
 *   }
 *
 *   return { diffPercentage, matches };
 * };
 * ```
 */
export interface DifferPlugin {
	/**
	 * Compare two images and generate a diff.
	 */
	compare: (input: DifferInput) => Promise<DifferOutput>;
}
//# sourceMappingURL=differ.d.ts.map
