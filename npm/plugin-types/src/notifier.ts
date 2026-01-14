/**
 * Notifier plugin types.
 *
 * Notifier plugins send test results to external services.
 * Multiple notifiers can run simultaneously (they stack).
 */

import type { DiffResult } from "./common";

/**
 * Input for notifier notify operation.
 */
export interface NotifierInput {
	/** Diff results */
	result: DiffResult;

	/** Path to the HTML report (if generated) */
	reportPath?: string;

	/** URL to the report (if hosted) */
	reportUrl?: string;

	/** Whether running in CI mode */
	ciMode: boolean;

	/** Plugin options from config */
	options: Record<string, unknown>;
}

/**
 * Notifier plugin interface.
 *
 * Implement the notify hook to send results to external services.
 *
 * @example
 * ```typescript
 * import type { NotifierPlugin, NotifierInput } from '@pixelguard/plugin-types';
 *
 * // Slack notifier
 * export const notify: NotifierPlugin['notify'] = async (input: NotifierInput): Promise<void> => {
 *   const { result, options, reportUrl } = input;
 *   const webhookUrl = options.webhookUrl as string;
 *
 *   const hasChanges = result.changed.length > 0 ||
 *                      result.added.length > 0 ||
 *                      result.removed.length > 0;
 *
 *   // Skip notification if no changes and onlyOnFailure is set
 *   if (!hasChanges && options.onlyOnFailure) {
 *     return;
 *   }
 *
 *   const status = hasChanges
 *     ? ':warning: Visual changes detected'
 *     : ':white_check_mark: All tests passed';
 *
 *   const message = {
 *     blocks: [
 *       {
 *         type: 'section',
 *         text: {
 *           type: 'mrkdwn',
 *           text: `*Pixelguard Report*\n${status}`,
 *         },
 *       },
 *       {
 *         type: 'section',
 *         fields: [
 *           { type: 'mrkdwn', text: `*Unchanged:* ${result.unchanged.length}` },
 *           { type: 'mrkdwn', text: `*Changed:* ${result.changed.length}` },
 *           { type: 'mrkdwn', text: `*Added:* ${result.added.length}` },
 *           { type: 'mrkdwn', text: `*Removed:* ${result.removed.length}` },
 *         ],
 *       },
 *     ],
 *   };
 *
 *   if (reportUrl) {
 *     message.blocks.push({
 *       type: 'actions',
 *       elements: [
 *         {
 *           type: 'button',
 *           text: { type: 'plain_text', text: 'View Report' },
 *           url: reportUrl,
 *         },
 *       ],
 *     });
 *   }
 *
 *   await fetch(webhookUrl, {
 *     method: 'POST',
 *     headers: { 'Content-Type': 'application/json' },
 *     body: JSON.stringify(message),
 *   });
 * };
 * ```
 */
export interface NotifierPlugin {
	/**
	 * Send notification about test results.
	 */
	notify: (input: NotifierInput) => Promise<void>;
}
