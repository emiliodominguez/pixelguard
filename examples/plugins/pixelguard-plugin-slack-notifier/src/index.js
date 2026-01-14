/**
 * Pixelguard Slack Notifier Plugin
 *
 * Sends visual regression test results to a Slack channel.
 *
 * Configuration:
 * {
 *   "plugins": ["pixelguard-plugin-slack-notifier"],
 *   "pluginOptions": {
 *     "pixelguard-plugin-slack-notifier": {
 *       "webhookUrl": "https://hooks.slack.com/services/xxx/yyy/zzz",
 *       "channel": "#visual-tests",
 *       "onlyOnFailure": true
 *     }
 *   }
 * }
 */

/**
 * Send a notification to Slack about test results.
 *
 * @param {Object} input - Notifier input
 * @param {Object} input.result - Diff results
 * @param {string} input.reportPath - Path to HTML report
 * @param {string} input.reportUrl - URL to hosted report (if available)
 * @param {boolean} input.ciMode - Whether running in CI
 * @param {Object} input.options - Plugin options
 * @returns {Promise<void>}
 */
async function notify(input) {
	const { result, reportPath, reportUrl, options } = input;

	const webhookUrl = options.webhookUrl;

	if (!webhookUrl) {
		throw new Error("Slack webhook URL is required. Set it in pluginOptions.pixelguard-plugin-slack-notifier.webhookUrl");
	}

	// Check if we should notify
	const hasChanges = result.changed.length > 0 || result.added.length > 0 || result.removed.length > 0;

	if (options.onlyOnFailure && !hasChanges) {
		console.error("Slack notification skipped (no changes and onlyOnFailure=true)");
		return;
	}

	// Build Slack message
	const status = hasChanges ? ":warning: Visual changes detected" : ":white_check_mark: All visual tests passed";

	const total = result.unchanged.length + result.changed.length + result.added.length + result.removed.length;

	const blocks = [
		{
			type: "header",
			text: {
				type: "plain_text",
				text: "Pixelguard Visual Regression Report",
				emoji: true,
			},
		},
		{
			type: "section",
			text: {
				type: "mrkdwn",
				text: status,
			},
		},
		{
			type: "divider",
		},
		{
			type: "section",
			fields: [
				{
					type: "mrkdwn",
					text: `*Total Shots:*\n${total}`,
				},
				{
					type: "mrkdwn",
					text: `*Unchanged:*\n${result.unchanged.length}`,
				},
				{
					type: "mrkdwn",
					text: `*Changed:*\n${result.changed.length}`,
				},
				{
					type: "mrkdwn",
					text: `*Added:*\n${result.added.length}`,
				},
			],
		},
	];

	// Add changed shots details (up to 5)
	if (result.changed.length > 0) {
		const changedList = result.changed
			.slice(0, 5)
			.map((s) => `• \`${s.name}\` (${s.diffPercentage.toFixed(2)}% diff)`)
			.join("\n");

		const moreText = result.changed.length > 5 ? `\n_...and ${result.changed.length - 5} more_` : "";

		blocks.push({
			type: "section",
			text: {
				type: "mrkdwn",
				text: `*Changed shots:*\n${changedList}${moreText}`,
			},
		});
	}

	// Add report link if available
	const reportLink = reportUrl || reportPath;
	if (reportLink) {
		blocks.push({
			type: "divider",
		});

		if (reportUrl) {
			blocks.push({
				type: "actions",
				elements: [
					{
						type: "button",
						text: {
							type: "plain_text",
							text: "View Report",
							emoji: true,
						},
						url: reportUrl,
						style: hasChanges ? "danger" : "primary",
					},
				],
			});
		} else {
			blocks.push({
				type: "context",
				elements: [
					{
						type: "mrkdwn",
						text: `Report: \`${reportPath}\``,
					},
				],
			});
		}
	}

	// Build the message payload
	const payload = {
		blocks,
	};

	if (options.channel) {
		payload.channel = options.channel;
	}

	// Send to Slack
	const response = await fetch(webhookUrl, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(payload),
	});

	if (!response.ok) {
		const text = await response.text();
		throw new Error(`Slack API error: ${response.status} - ${text}`);
	}

	console.error("Slack notification sent successfully");
}

module.exports = { notify };
