/**
 * Reporter plugin types.
 *
 * Reporter plugins generate reports in additional formats.
 * The built-in HTML report is always generated, but plugins
 * can add JUnit XML, JSON, Markdown, or custom formats.
 */
import type { ConfigSummary, DiffResult } from "./common";
/**
 * Input for reporter generate operation.
 */
export interface ReporterInput {
	/** Diff results */
	result: DiffResult;
	/** Configuration summary */
	config: ConfigSummary;
	/** Output directory */
	outputDir: string;
	/** Plugin options from config */
	options: Record<string, unknown>;
}
/**
 * Output from reporter generate operation.
 */
export interface ReporterOutput {
	/** Path to generated report (if local file) */
	reportPath?: string;
	/** URL to report (if hosted) */
	reportUrl?: string;
}
/**
 * Reporter plugin interface.
 *
 * Implement the generate hook to create custom report formats.
 *
 * @example
 * ```typescript
 * import type { ReporterPlugin, ReporterInput, ReporterOutput } from '@pixelguard/plugin-types';
 * import * as fs from 'fs';
 * import * as path from 'path';
 *
 * // JUnit XML reporter
 * export const generate: ReporterPlugin['generate'] = async (input: ReporterInput): Promise<ReporterOutput> => {
 *   const { result, outputDir } = input;
 *
 *   const testcases = [
 *     ...result.unchanged.map(name =>
 *       `<testcase name="${name}" classname="pixelguard"/>`
 *     ),
 *     ...result.changed.map(shot =>
 *       `<testcase name="${shot.name}" classname="pixelguard">
 *         <failure message="Visual difference: ${shot.diffPercentage.toFixed(2)}%"/>
 *       </testcase>`
 *     ),
 *     ...result.added.map(name =>
 *       `<testcase name="${name}" classname="pixelguard">
 *         <failure message="New screenshot (no baseline)"/>
 *       </testcase>`
 *     ),
 *     ...result.removed.map(name =>
 *       `<testcase name="${name}" classname="pixelguard">
 *         <failure message="Removed (baseline exists, no current)"/>
 *       </testcase>`
 *     ),
 *   ];
 *
 *   const failures = result.changed.length + result.added.length + result.removed.length;
 *   const total = result.unchanged.length + failures;
 *
 *   const xml = `<?xml version="1.0" encoding="UTF-8"?>
 *   <testsuite name="pixelguard" tests="${total}" failures="${failures}">
 *     ${testcases.join('\n    ')}
 *   </testsuite>`;
 *
 *   const reportPath = path.join(outputDir, 'junit.xml');
 *   fs.writeFileSync(reportPath, xml);
 *
 *   return { reportPath };
 * };
 * ```
 */
export interface ReporterPlugin {
	/**
	 * Generate a report from diff results.
	 */
	generate: (input: ReporterInput) => Promise<ReporterOutput>;
}
//# sourceMappingURL=reporter.d.ts.map
