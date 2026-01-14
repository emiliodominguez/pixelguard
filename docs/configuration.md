# Configuration Reference

Pixelguard uses a `pixelguard.config.json` file in your project root. All fields are optional with sensible defaults.

## JSON Schema (IDE Support)

For autocomplete and validation in your IDE, add the `$schema` field to your config:

```json
{
	"$schema": "https://raw.githubusercontent.com/emiliodominguez/pixelguard/main/schemas/pixelguard.config.schema.json",
	"source": "storybook",
	"baseUrl": "http://localhost:6006"
}
```

This provides:
- Autocomplete for all configuration options
- Inline documentation on hover
- Validation errors for invalid values

For VS Code users, you can also configure the schema globally in your settings:

```json
{
	"json.schemas": [
		{
			"fileMatch": ["pixelguard.config.json"],
			"url": "https://raw.githubusercontent.com/emiliodominguez/pixelguard/main/schemas/pixelguard.config.schema.json"
		}
	]
}
```

## Full Example

```json
{
	"source": "storybook",
	"baseUrl": "http://localhost:6006",
	"include": ["**/*"],
	"exclude": ["**/*Deprecated*", "**/*Legacy*"],
	"viewport": {
		"width": 1280,
		"height": 720
	},
	"threshold": 0.01,
	"outputDir": ".pixelguard",
	"concurrency": 4
}
```

**Note:** For Storybook projects, shots are discovered dynamically at test time.

## Fields

### `source`

**Type:** `string`
**Default:** `""`

The project type. Automatically set by `pixelguard init`.

Values:

- `"storybook"` - Storybook project (stories auto-discovered)
- `"manual"` - Manual configuration (any other project)

### `baseUrl`

**Type:** `string`
**Default:** `""`

The base URL of your development server. All shot paths are relative to this URL.

Examples:

- `"http://localhost:6006"` - Storybook
- `"http://localhost:3000"` - Next.js, Remix
- `"http://localhost:5173"` - Vite
- `"http://localhost:4200"` - Angular

### `port`

**Type:** `number`
**Default:** (auto-detect)

Port to use for dev server detection during `pixelguard init`. Overrides automatic port probing.

```json
{
	"port": 8080
}
```

### `include`

**Type:** `string[]`
**Default:** `["**/*"]`

Glob patterns for shots to include. Only applicable when shots are auto-discovered.

Examples:

- `["**/*"]` - Include all shots
- `["components/**/*"]` - Only components
- `["button--*", "card--*"]` - Specific patterns

### `exclude`

**Type:** `string[]`
**Default:** `[]`

Glob patterns for shots to exclude.

Examples:

- `["**/*Deprecated*"]` - Exclude deprecated
- `["**/internal/**"]` - Exclude internal components

### `viewport`

**Type:** `{ width: number, height: number }`
**Default:** `{ "width": 1280, "height": 720 }`

The viewport size for screenshots.

Common sizes:

- `{ "width": 1920, "height": 1080 }` - Full HD
- `{ "width": 1280, "height": 720 }` - HD (default)
- `{ "width": 768, "height": 1024 }` - Tablet
- `{ "width": 375, "height": 667 }` - Mobile

### `threshold`

**Type:** `number` (percentage)
**Default:** `0.01`

The diff threshold as a percentage of pixels. Shots with differences below this threshold are considered unchanged.

- `0.0` - Any difference fails (strict)
- `0.01` - 0.01% tolerance (default, catches small text changes)
- `0.1` - 0.1% tolerance (lenient)
- `1.0` - 1% tolerance (very lenient)

Lower values catch more subtle changes but may produce false positives from anti-aliasing.

### `outputDir`

**Type:** `string`
**Default:** `".pixelguard"`

The directory for screenshots and reports.

### `concurrency`

**Type:** `number`
**Default:** `4`

Number of screenshots to capture in parallel. Higher values speed up capture but use more memory.

- `1` - Sequential capture (slowest, lowest memory)
- `4` - Default (good balance)
- `8` - Faster capture for powerful machines
- `16` - Maximum parallelism (high memory usage)

Example:

```json
{
	"concurrency": 8
}
```

### `shots`

**Type:** `Shot[]`
**Default:** `[]`

Optional array of shot overrides. For Storybook projects, shots are discovered automatically at test time. Use this to provide custom configuration for specific shots.

## Shot Overrides

You can override settings for specific shots by adding them to the `shots` array:

```json
{
	"shots": [
		{
			"name": "components-card--with-image",
			"delay": 500
		},
		{
			"name": "components-modal--animated",
			"waitFor": ".modal-content",
			"delay": 1000
		}
	]
}
```

### `name`

**Type:** `string`
**Required:** Yes

The shot name to override. Must match the discovered shot name exactly.

### `waitFor`

**Type:** `string`
**Required:** No

CSS selector to wait for before capturing the screenshot. Useful for async content.

### `delay`

**Type:** `number` (milliseconds)
**Required:** No

Additional delay after the page loads and `waitFor` selector is found.

Common values:

- `100` - Quick delay for minor rendering
- `500` - Medium delay for animations
- `1000` - Long delay for complex pages

## Environment-Specific Configuration

You can use different configs for different environments:

```bash
# Development
PIXELGUARD_CONFIG=pixelguard.dev.json npx pixelguard test

# CI
PIXELGUARD_CONFIG=pixelguard.ci.json npx pixelguard test --ci
```

## Multiple Viewports

Test the same shots across different screen sizes with the `viewports` array:

```json
{
	"viewports": [
		{ "name": "desktop", "width": 1920, "height": 1080 },
		{ "name": "tablet", "width": 768, "height": 1024 },
		{ "name": "mobile", "width": 375, "height": 667 }
	]
}
```

When `viewports` is set, each shot is captured at each viewport size. Screenshots are named `{shot}@{viewport}.png`:

- `button--primary@desktop.png`
- `button--primary@tablet.png`
- `button--primary@mobile.png`

This is more efficient than manually duplicating shots.

### Backwards Compatibility

If `viewports` is empty or not set, the single `viewport` field is used with the name "default", and screenshots are named without a suffix (`button--primary.png`).

### viewports

**Type:** `{ name: string, width: number, height: number }[]`
**Default:** `[]`

Each viewport must have:
- `name` - Unique identifier (e.g., "mobile", "tablet", "desktop")
- `width` - Viewport width in pixels
- `height` - Viewport height in pixels

## Plugins

Pixelguard supports plugins for extending functionality. See [Plugins](./plugins.md) for full documentation.

### `plugins`

**Type:** `(string | { name: string, options: object })[]`
**Default:** `[]`

List of plugins to load. Plugins can be npm packages or local paths.

```json
{
	"plugins": [
		"pixelguard-plugin-json-reporter",
		"./my-local-plugin",
		{
			"name": "pixelguard-plugin-slack-notifier",
			"options": {
				"webhookUrl": "https://hooks.slack.com/..."
			}
		}
	]
}
```

### `pluginOptions`

**Type:** `object`
**Default:** `{}`

Options for plugins, keyed by plugin name. Alternative to inline options.

```json
{
	"plugins": ["pixelguard-plugin-s3-storage"],
	"pluginOptions": {
		"pixelguard-plugin-s3-storage": {
			"bucket": "my-baselines",
			"region": "us-east-1"
		}
	}
}
```

## Git LFS for Large Baselines

For projects with many screenshots, consider using Git LFS:

```bash
# Install Git LFS
git lfs install

# Track PNG files in .pixelguard
echo ".pixelguard/baseline/*.png filter=lfs diff=lfs merge=lfs -text" >> .gitattributes

git add .gitattributes
git commit -m "Track baseline screenshots with Git LFS"
```
