# Configuration Reference

Pixelguard uses a `pixelguard.config.json` file in your project root. All fields are optional with sensible defaults.

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
	"outputDir": ".pixelguard"
}
```

**Note:** For Storybook projects, shots are discovered dynamically at test time.

## Fields

### `source`

**Type:** `string`
**Default:** `""`

The project type. Automatically set by `pixelguard init`.

Values:

- `"storybook"` - Storybook project
- `"nextjs"` - Next.js project
- `"vite"` - Vite project
- `"manual"` - Manual configuration

### `baseUrl`

**Type:** `string`
**Default:** `""`

The base URL of your development server. All shot paths are relative to this URL.

Examples:

- `"http://localhost:6006"` - Storybook
- `"http://localhost:3000"` - Next.js
- `"http://localhost:5173"` - Vite

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

To test multiple viewport sizes, create separate configs or use the `--filter` flag:

```json
{
	"shots": [
		{
			"name": "button--primary--desktop",
			"path": "/iframe.html?id=button--primary"
		},
		{
			"name": "button--primary--mobile",
			"path": "/iframe.html?id=button--primary&viewport=mobile"
		}
	]
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
