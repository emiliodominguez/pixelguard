# pixelguard-plugin-json-reporter

A Pixelguard plugin that generates machine-readable JSON reports.

## Installation

```bash
npm install pixelguard-plugin-json-reporter
```

## Usage

Add the plugin to your `pixelguard.config.json`:

```json
{
	"plugins": ["pixelguard-plugin-json-reporter"]
}
```

## Options

| Option     | Type    | Default         | Description                    |
| ---------- | ------- | --------------- | ------------------------------ |
| `filename` | string  | `"report.json"` | Output filename                |
| `pretty`   | boolean | `true`          | Pretty-print JSON with 2-space indent |

### Example with options

```json
{
	"plugins": ["pixelguard-plugin-json-reporter"],
	"pluginOptions": {
		"pixelguard-plugin-json-reporter": {
			"filename": "visual-regression-results.json",
			"pretty": true
		}
	}
}
```

## Output Format

```json
{
	"timestamp": "2024-01-15T10:30:00.000Z",
	"summary": {
		"total": 47,
		"unchanged": 45,
		"changed": 2,
		"added": 0,
		"removed": 0,
		"passed": false
	},
	"config": {
		"source": "storybook",
		"baseUrl": "http://localhost:6006",
		"threshold": 0.01
	},
	"results": {
		"unchanged": ["button--primary", "button--secondary", "..."],
		"changed": [
			{
				"name": "card--default",
				"diffPercentage": 0.15,
				"baselinePath": ".pixelguard/baseline/card--default.png",
				"currentPath": ".pixelguard/current/card--default.png",
				"diffPath": ".pixelguard/diff/card--default.png"
			}
		],
		"added": [],
		"removed": []
	}
}
```

## Use Cases

- **CI/CD Integration**: Parse JSON results to fail builds on visual changes
- **Custom Dashboards**: Feed results into monitoring systems
- **Historical Tracking**: Store JSON reports for trend analysis
