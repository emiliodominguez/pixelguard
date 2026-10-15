# Creating Pixelguard Plugins

This guide walks you through creating a Pixelguard plugin from scratch. We'll build a Slack notifier plugin as an example.

## Plugin Overview

A Pixelguard plugin is a Node.js package that exports async hook functions. Pixelguard loads the package in a short-lived Node.js process, calls the hook with a JSON-serializable input object, and reads back the value the hook returns. Plugins can:

- **Notify** - Send notifications (Slack, Teams, email)
- **Report** - Generate custom reports (JUnit XML, JSON)
- **Capture** - Provide custom screenshot capture
- **Diff** - Implement custom image comparison
- **Store** - Use custom storage backends (S3, GCS)

## Quick Start

### 1. Create the Package

```bash
mkdir pixelguard-plugin-slack-notifier
cd pixelguard-plugin-slack-notifier
npm init -y
```

### 2. Add the Manifest to package.json

The manifest lives in the `pixelguard` field of your `package.json`:

```json
{
	"name": "pixelguard-plugin-slack-notifier",
	"version": "1.0.0",
	"main": "index.js",
	"pixelguard": {
		"name": "Slack Notifier",
		"category": "notifier",
		"entry": "index.js",
		"hooks": ["notify"]
	}
}
```

### 3. Export the Hook

Create `index.js`:

```javascript
/**
 * Sends a summary of the run to Slack.
 *
 * @param {import("pixelguard-plugin-types").NotifierInput} input - Hook input
 */
async function notify(input) {
	const { result, options } = input;
	if (!options.webhookUrl) {
		throw new Error("Missing webhookUrl. Add it to pluginOptions in your config.");
	}

	const response = await fetch(options.webhookUrl, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(buildSlackMessage(result)),
	});
	if (!response.ok) {
		throw new Error(`Slack responded with HTTP ${response.status}`);
	}
}

/**
 * Builds the Slack message body.
 *
 * @param {import("pixelguard-plugin-types").DiffResult} result - Run result
 * @returns {object} Slack Block Kit payload
 */
function buildSlackMessage(result) {
	const { unchanged, changed, added, removed, failed = [] } = result;
	const passed = changed.length + added.length + removed.length + failed.length === 0;

	return {
		blocks: [
			{
				type: "header",
				text: { type: "plain_text", text: passed ? "Visual Tests Passed" : "Visual Tests Need Attention" },
			},
			{
				type: "section",
				fields: [
					{ type: "mrkdwn", text: `*Unchanged:* ${unchanged.length}` },
					{ type: "mrkdwn", text: `*Changed:* ${changed.length}` },
					{ type: "mrkdwn", text: `*Added:* ${added.length}` },
					{ type: "mrkdwn", text: `*Removed:* ${removed.length}` },
					{ type: "mrkdwn", text: `*Failed:* ${failed.length}` },
				],
			},
		],
	};
}

module.exports = { notify };
```

Throwing an error marks the hook as failed; returning normally marks it as successful. Whatever the hook returns is passed back to Pixelguard.

ES modules work too: if `require()` cannot load your entry point, Pixelguard falls back to `import()` and uses the module's `default` export (or the module namespace).

### 4. Test Locally

```bash
# In your project using Pixelguard
npm link ../pixelguard-plugin-slack-notifier
```

Add to `pixelguard.config.json`:

```json
{
	"plugins": [
		{
			"name": "pixelguard-plugin-slack-notifier",
			"options": {
				"webhookUrl": "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
			}
		}
	]
}
```

Run tests:

```bash
npx pixelguard test
```

## Plugin Manifest

The `pixelguard` field in `package.json` tells Pixelguard about your plugin.

| Field           | Type     | Required | Description                                                                     |
| --------------- | -------- | -------- | ------------------------------------------------------------------------------- |
| `name`          | string   | Yes      | Human-readable plugin name                                                      |
| `category`      | string   | Yes      | Plugin category (see below)                                                     |
| `hooks`         | string[] | Yes      | Hooks this plugin implements                                                    |
| `entry`         | string   | No       | Entry point relative to the package; defaults to `main`, then `index.js`        |
| `version`       | string   | No       | Defaults to the package version                                                 |
| `optionsSchema` | object   | No       | JSON Schema describing your options (informational)                             |

The entry point must resolve to a file inside the package directory.

### Categories

| Category   | Hooks                                       | Behavior                  |
| ---------- | ------------------------------------------- | ------------------------- |
| `notifier` | `notify`                                    | Stackable (all run)       |
| `reporter` | `generate`                                  | Stackable (all run)       |
| `capture`  | `capture`                                   | Single-winner (last wins) |
| `differ`   | `compare`                                   | Single-winner (last wins) |
| `storage`  | `read`, `write`, `exists`, `list`, `delete` | Single-winner (last wins) |

### Options

Every hook input carries an `options` object built from, in increasing priority:

1. Inline `options` on the plugin entry in `plugins`
2. `pluginOptions["<package name>"]`

## Runtime Contract

- Each hook call runs in a fresh Node.js process with the project directory as the working directory.
- Hooks run with a 10 minute timeout by default. Set `PIXELGUARD_TIMEOUT_SECS` to change it.
- `console.log` and `console.error` are both safe: the result travels through a separate channel, and all output is shown to the user on stderr.
- Calling `process.exit()` before the hook resolves is reported as a failure, because no result was produced.

## Hook Interfaces

Install the types package for editor support:

```bash
npm install -D pixelguard-plugin-types
```

### Notifier: `notify`

Called after comparison completes, in CI mode as well, including when the run fails.

```typescript
interface NotifierInput {
	result: DiffResult;
	reportPath?: string;
	reportUrl?: string;
	ciMode: boolean;
	options: Record<string, unknown>;
}

interface DiffResult {
	unchanged: string[];
	changed: {
		name: string;
		baselinePath: string;
		currentPath: string;
		diffPath: string;
		diffPercentage: number;
		/** Absent when no heatmap could be drawn (the screenshots differ in size) */
		heatmapPath?: string;
		/** Changed areas in image pixels, in reading order */
		regions: { x: number; y: number; width: number; height: number }[];
	}[];
	added: string[];
	removed: string[];
	/** Shots that could not be captured or compared. Any entry means the run failed. */
	failed: string[];
}
```

Return value is ignored.

### Reporter: `generate`

```typescript
interface ReporterInput {
	result: DiffResult;
	config: { source: string; baseUrl: string; threshold: number };
	outputDir: string;
	options: Record<string, unknown>;
}

interface ReporterOutput {
	reportPath?: string;
	reportUrl?: string;
}
```

### Capture: `capture`

Replaces the built-in Playwright capture.

```typescript
interface CaptureInput {
	shots: { name: string; path: string; waitFor?: string; delay?: number }[];
	baseUrl: string;
	/** Legacy single viewport */
	viewport: { width: number; height: number };
	/** Every viewport to capture; `default` when the config has no `viewports` list */
	viewports: { name: string; width: number; height: number }[];
	outputDir: string;
	options: Record<string, unknown>;
}

interface CaptureOutput {
	captured: { name: string; path: string }[];
	failed: { name: string; error: string }[];
}
```

Write each screenshot to `outputDir` as `${shot.name}.png` for the `default` viewport and `${shot.name}@${viewport.name}.png` otherwise. Pixelguard empties `outputDir` of PNGs before calling you, and afterwards only trusts files that exist under those exact names: an expected screenshot that is missing is reported as failed, whatever `captured` says.

### Differ: `compare`

Replaces the built-in pixel comparison. Your plugin decides whether a shot changed and writes the diff image; for shots it reports as changed, Pixelguard still draws the heatmap and locates the changed areas with its own pixel comparison, so every report view keeps working.

```typescript
interface DifferInput {
	baselinePath: string;
	currentPath: string;
	/** Where to write a diff image, if you produce one */
	diffPath: string;
	/** Percentage of pixels (0-100) allowed to differ */
	threshold: number;
	options: Record<string, unknown>;
}

interface DifferOutput {
	/** Must be a finite number between 0 and 100 */
	diffPercentage: number;
	matches: boolean;
}
```

A `diffPercentage` that is not a finite number in `0..=100` is treated as a comparison failure, never as a pass.

### Storage: `read`, `write`, `exists`, `list`, `delete`

Stores baselines remotely. Paths are relative to the output directory, e.g. `baseline/button--primary.png`.

```typescript
interface StorageInput {
	path: string;
	/** Base64 file contents, for `write` */
	data?: string;
	options: Record<string, unknown>;
}

interface StorageOutput {
	/** Base64 file contents, for `read` */
	data?: string;
	/** For `exists` */
	exists?: boolean;
	/** File names, for `list` */
	files?: string[];
}
```

`list` and `delete` are used by `pixelguard test --update` to remove baselines of shots that no longer exist.

## Debugging Plugins

Run Pixelguard with `--verbose` to see plugin execution details:

```bash
npx pixelguard --verbose test
```

To exercise a hook directly, call it from a small script:

```bash
node -e '
const plugin = require("./index.js");
plugin.notify({
	result: { unchanged: ["a"], changed: [], added: [], removed: [], failed: [] },
	ciMode: false,
	options: { webhookUrl: process.env.SLACK_WEBHOOK_URL },
}).then(() => console.log("ok"), (error) => console.error(error));
'
```

## Publishing Plugins

### Naming Convention

Use the prefix `pixelguard-plugin-`:

- `pixelguard-plugin-slack-notifier`
- `pixelguard-plugin-json-reporter`
- `pixelguard-plugin-s3-storage`

Scoped packages such as `@acme/pixelguard-plugin-s3` work as well.

### Package.json Keywords

```json
{
	"keywords": ["pixelguard", "pixelguard-plugin", "visual-regression", "screenshot-testing"]
}
```

### Documentation

Include a README.md with:

1. What the plugin does
2. Installation instructions
3. Configuration options
4. Example usage

See `examples/plugins/` in this repository for complete, working plugins of every category.

## Plugin Ideas

### Notifiers

- Discord webhook notifier
- Microsoft Teams notifier
- GitHub PR comment notifier

### Reporters

- JUnit XML reporter (for CI integration)
- Markdown reporter

### Storage

- Google Cloud Storage
- Azure Blob Storage

### Capture

- Safari/WebKit capture
- Mobile device capture (via cloud services)

### Differ

- Perceptual diff (focus on human-visible changes)
- Region-of-interest diff (ignore specific areas)
