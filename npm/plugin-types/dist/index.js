"use strict";
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
var __createBinding =
	(this && this.__createBinding) ||
	(Object.create
		? function (o, m, k, k2) {
				if (k2 === undefined) k2 = k;
				var desc = Object.getOwnPropertyDescriptor(m, k);
				if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
					desc = {
						enumerable: true,
						get: function () {
							return m[k];
						},
					};
				}
				Object.defineProperty(o, k2, desc);
			}
		: function (o, m, k, k2) {
				if (k2 === undefined) k2 = k;
				o[k2] = m[k];
			});
var __exportStar =
	(this && this.__exportStar) ||
	function (m, exports) {
		for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
	};
Object.defineProperty(exports, "__esModule", { value: true });
__exportStar(require("./storage"), exports);
__exportStar(require("./reporter"), exports);
__exportStar(require("./capture"), exports);
__exportStar(require("./differ"), exports);
__exportStar(require("./notifier"), exports);
__exportStar(require("./common"), exports);
