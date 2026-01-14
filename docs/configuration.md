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
  "threshold": 0.1,
  "outputDir": ".pixelguard",
  "shots": [
    {
      "name": "button--primary",
      "path": "/iframe.html?id=button--primary",
      "waitFor": "#storybook-root",
      "delay": 100
    }
  ]
}
```

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

**Type:** `number` (0.0 to 1.0)
**Default:** `0.1`

The diff threshold as a percentage. Shots with differences below this threshold are considered unchanged.

- `0.0` - Any difference fails (strict)
- `0.1` - 0.1% tolerance (default)
- `0.5` - 0.5% tolerance (lenient)
- `1.0` - 1% tolerance (very lenient)

Lower values catch more subtle changes but may produce false positives from anti-aliasing.

### `outputDir`

**Type:** `string`
**Default:** `".pixelguard"`

The directory for screenshots and reports.

### `shots`

**Type:** `Shot[]`
**Default:** `[]`

Array of shot configurations. Auto-populated by `pixelguard init` for Storybook and Next.js projects.

## Shot Configuration

Each shot in the `shots` array has these fields:

### `name`

**Type:** `string`
**Required:** Yes

Unique identifier for the shot. Used for the screenshot filename.

Examples:
- `"button--primary"`
- `"page-home"`
- `"modal-confirmation"`

### `path`

**Type:** `string`
**Required:** Yes

URL path appended to `baseUrl`.

Examples:
- `"/iframe.html?id=button--primary"` - Storybook
- `"/about"` - Next.js page
- `"/"` - Homepage

### `waitFor`

**Type:** `string`
**Required:** No

CSS selector to wait for before capturing the screenshot. Useful for async content.

Examples:
- `"#storybook-root"` - Storybook root
- `"[data-loaded='true']"` - Custom loading indicator
- `".main-content"` - Specific element

### `delay`

**Type:** `number` (milliseconds)
**Required:** No

Additional delay after the page loads and `waitFor` selector is found.

Useful for:
- Animations to complete
- Fonts to load
- Images to render

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
