# pixelguard-plugin-console-notifier

A simple Pixelguard notifier plugin that prints formatted results to the console.

This plugin serves as a learning example for how to build notifier plugins.

## Installation

```bash
npm install pixelguard-plugin-console-notifier
```

## Usage

Add the plugin to your `pixelguard.config.json`:

```json
{
	"plugins": ["pixelguard-plugin-console-notifier"]
}
```

## Options

| Option        | Type    | Default | Description                          |
| ------------- | ------- | ------- | ------------------------------------ |
| `showDetails` | boolean | `true`  | Show individual changed/added/removed shots |
| `useEmoji`    | boolean | `true`  | Use emoji in output                  |

### Example with options

```json
{
	"plugins": ["pixelguard-plugin-console-notifier"],
	"pluginOptions": {
		"pixelguard-plugin-console-notifier": {
			"showDetails": true,
			"useEmoji": false
		}
	}
}
```

## Example Output

```
==================================================
  PIXELGUARD VISUAL REGRESSION RESULTS
==================================================

  Status: CHANGES DETECTED

  Summary:
    Total shots:  47
    Unchanged:    45
    Changed:      2
    Added:        0
    Removed:      0

  Changed shots:
    - card--default (0.15% different)
    - button--primary (0.08% different)

  Report: .pixelguard/report.html

==================================================
```

## Building Your Own Notifier

This plugin demonstrates the notifier plugin interface:

```javascript
async function notify(input) {
	const { result, reportPath, ciMode, options } = input;

	// result.unchanged - array of shot names
	// result.changed - array of { name, diffPercentage, ... }
	// result.added - array of shot names
	// result.removed - array of shot names

	// Do something with the results...
}

module.exports = { notify };
```

You can use this pattern to send notifications to:
- Slack
- Microsoft Teams
- Discord
- Email
- Custom webhooks
- Any external service
