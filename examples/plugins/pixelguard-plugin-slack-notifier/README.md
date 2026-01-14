# pixelguard-plugin-slack-notifier

A Pixelguard notifier plugin that sends visual regression test results to Slack.

## Installation

```bash
npm install pixelguard-plugin-slack-notifier
```

## Configuration

Add the plugin to your `pixelguard.config.json`:

```json
{
  "source": "storybook",
  "baseUrl": "http://localhost:6006",
  "plugins": ["pixelguard-plugin-slack-notifier"],
  "pluginOptions": {
    "pixelguard-plugin-slack-notifier": {
      "webhookUrl": "https://hooks.slack.com/services/xxx/yyy/zzz",
      "channel": "#visual-tests",
      "onlyOnFailure": true
    }
  }
}
```

### Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `webhookUrl` | `string` | Yes | - | Slack incoming webhook URL |
| `channel` | `string` | No | - | Override the default webhook channel |
| `onlyOnFailure` | `boolean` | No | `false` | Only send notifications when there are visual changes |

## Setting Up Slack Webhook

1. Go to your Slack workspace settings
2. Navigate to "Apps" > "Manage" > "Custom Integrations" > "Incoming Webhooks"
3. Click "Add to Slack"
4. Choose the channel for notifications
5. Copy the webhook URL

Alternatively, create a Slack app:

1. Go to [api.slack.com/apps](https://api.slack.com/apps)
2. Click "Create New App"
3. Enable "Incoming Webhooks"
4. Add a webhook to your workspace
5. Copy the webhook URL

## Message Format

The plugin sends a rich Slack message with:

- **Header**: "Pixelguard Visual Regression Report"
- **Status**: Warning icon for changes, checkmark for all passed
- **Summary**: Total shots, unchanged, changed, and added counts
- **Changed Details**: List of up to 5 changed shots with diff percentages
- **Report Link**: Button to view the HTML report (if `reportUrl` is provided)

Example message:

```
Pixelguard Visual Regression Report

⚠️ Visual changes detected

Total Shots: 47    Unchanged: 45
Changed: 2         Added: 0

Changed shots:
• `button--primary` (2.45% diff)
• `card--hover` (0.89% diff)

[View Report]
```

## Environment Variables

For security, use environment variables for the webhook URL:

```json
{
  "pluginOptions": {
    "pixelguard-plugin-slack-notifier": {
      "webhookUrl": "${SLACK_WEBHOOK_URL}"
    }
  }
}
```

Then set the environment variable:

```bash
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/xxx/yyy/zzz"
```

## CI Integration

### GitHub Actions

```yaml
- name: Run visual tests
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
  run: npx pixelguard test --ci
```

### GitLab CI

```yaml
visual-tests:
  script:
    - npx pixelguard test --ci
  variables:
    SLACK_WEBHOOK_URL: $SLACK_WEBHOOK_URL
```

## Plugin Development

This plugin demonstrates:

- **External API integration**: Sending HTTP requests to Slack
- **Conditional notifications**: `onlyOnFailure` option
- **Rich message formatting**: Using Slack Block Kit
- **Error handling**: Graceful handling of API errors

### Key Implementation Details

```javascript
// Check if notification should be sent
const hasChanges = result.changed.length > 0 ||
                   result.added.length > 0 ||
                   result.removed.length > 0;

if (options.onlyOnFailure && !hasChanges) {
  return; // Skip notification
}

// Build Slack Block Kit message
const blocks = [
  { type: "header", text: { type: "plain_text", text: "..." } },
  { type: "section", fields: [...] },
  // ... more blocks
];

// Send to Slack
await fetch(webhookUrl, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ blocks }),
});
```

## License

MIT
