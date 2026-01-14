/**
 * Common types shared across plugin categories.
 */
/**
 * Plugin categories that can extend Pixelguard functionality.
 */
export type PluginCategory = "storage" | "reporter" | "capture" | "differ" | "notifier";
/**
 * Plugin manifest declared in package.json's "pixelguard" field.
 */
export interface PluginManifest {
	/** Human-readable plugin name */
	name: string;
	/** Plugin category */
	category: PluginCategory;
	/** Entry point relative to package root */
	entry: string;
	/** Hooks this plugin implements */
	hooks: string[];
	/** Optional JSON schema for plugin options validation */
	optionsSchema?: Record<string, unknown>;
}
/**
 * Diff result summary passed to reporter and notifier plugins.
 */
export interface DiffResult {
	/** Names of unchanged shots */
	unchanged: string[];
	/** Changed shots with details */
	changed: ChangedShot[];
	/** Names of added shots (new, no baseline) */
	added: string[];
	/** Names of removed shots (baseline exists, no current) */
	removed: string[];
}
/**
 * A shot that changed between baseline and current.
 */
export interface ChangedShot {
	/** Shot name */
	name: string;
	/** Path to baseline image */
	baselinePath: string;
	/** Path to current image */
	currentPath: string;
	/** Path to diff image */
	diffPath: string;
	/** Percentage of pixels that differ (0-100) */
	diffPercentage: number;
}
/**
 * Configuration summary for reporter plugins.
 */
export interface ConfigSummary {
	/** Source type (storybook, nextjs, vite, manual) */
	source: string;
	/** Base URL of the dev server */
	baseUrl: string;
	/** Diff threshold */
	threshold: number;
}
//# sourceMappingURL=common.d.ts.map
