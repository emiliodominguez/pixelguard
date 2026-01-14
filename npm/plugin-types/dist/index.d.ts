/**
 * @pixelguard/plugin-types
 *
 * TypeScript type definitions for creating Pixelguard plugins.
 *
 * @example
 * ```typescript
 * import type { StoragePlugin, NotifierPlugin } from '@pixelguard/plugin-types';
 *
 * export const read: StoragePlugin['read'] = async ({ path, options }) => {
 *   // Implement storage read
 * };
 *
 * export const notify: NotifierPlugin['notify'] = async (input) => {
 *   // Send notification
 * };
 * ```
 */
export * from "./storage";
export * from "./reporter";
export * from "./capture";
export * from "./differ";
export * from "./notifier";
export * from "./common";
//# sourceMappingURL=index.d.ts.map
