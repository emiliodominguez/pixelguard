# Pixelguard Examples

This directory contains example projects demonstrating how to use Pixelguard with different frameworks and configurations.

## Available Examples

| Example                            | Framework            | Port | Auto-Discovery | Purpose                                        |
| ---------------------------------- | -------------------- | ---- | -------------- | ---------------------------------------------- |
| [storybook-vite](./storybook-vite) | Storybook 8 + Vite 6 | 6006 | Yes            | Demonstrates automatic story discovery         |
| [nextjs-app](./nextjs-app)         | Next.js 15           | 3000 | No             | Shows App Router integration with manual shots |
| [vite-react](./vite-react)         | Vite 7 + React 19    | 5173 | No             | SPA with client-side routing                   |
| [astro-site](./astro-site)         | Astro 5              | 4321 | No             | Static site generation workflow                |
| [multi-viewport](./multi-viewport) | Vite 7 + React 19    | 5173 | No             | Multi-viewport responsive testing              |

## Example Details

### storybook-vite

**Purpose:** Showcases Pixelguard's primary use case - automatic visual regression testing for component libraries.

**Key Features:**

- Automatic story discovery from Storybook's `/index.json` endpoint
- No manual shot configuration needed
- Includes multiple component types (Button, Card, Badge, Alert, Input)
- Plugin testing configuration included

**Best For:** Teams using Storybook for component development who want zero-config visual testing.

### nextjs-app

**Purpose:** Demonstrates Pixelguard with a server-rendered React application using Next.js App Router.

**Key Features:**

- Manual shot configuration for page routes
- App Router structure (`src/app/`)
- Server-side rendering compatibility

**Best For:** Next.js applications where you want to test specific pages rather than isolated components.

### vite-react

**Purpose:** Shows Pixelguard with a client-side single-page application.

**Key Features:**

- React Router for client-side navigation
- Manual shot configuration
- Latest Vite 7 and React 19

**Best For:** SPAs built with Vite where you need to test multiple routes.

### astro-site

**Purpose:** Demonstrates Pixelguard with a static site generator.

**Key Features:**

- Astro components and layouts
- Static HTML generation
- Manual shot configuration

**Best For:** Content-focused sites and documentation where visual consistency matters.

### multi-viewport

**Purpose:** Showcases multi-viewport testing for responsive design validation.

**Key Features:**

- Tests across desktop (1920x1080), tablet (768x1024), and mobile (375x667)
- Responsive landing page with CSS breakpoints
- Screenshots named `{shot}@{viewport}.png`
- Uses JSON Schema for IDE autocomplete
- Includes `pixelguard validate` command demo

**Best For:** Teams who need to ensure their UI works across different screen sizes.

## Quick Start

All examples follow the same workflow:

```bash
# 1. Navigate to an example
cd examples/<example-name>

# 2. Install dependencies
npm install

# 3. Start the dev server
npm run dev          # For nextjs-app, vite-react, astro-site
npm run storybook    # For storybook-vite

# 4. In another terminal, create baseline screenshots
npm run pixelguard:test:visual:update

# 5. Run visual regression tests
npm run pixelguard:test:visual
```

## Configuration Patterns

### Storybook (Auto-Discovery)

```json
{
	"source": "storybook",
	"baseUrl": "http://localhost:6006"
}
```

Stories are discovered automatically - no `shots` array needed.

### Framework Apps (Manual Shots)

```json
{
	"baseUrl": "http://localhost:3000",
	"shots": [
		{ "name": "home", "path": "/" },
		{ "name": "about", "path": "/about" },
		{ "name": "contact", "path": "/contact" }
	]
}
```

Each shot requires a `name` (used for the screenshot filename) and `path` (appended to `baseUrl`).

### Optional Shot Configuration

```json
{
	"shots": [
		{
			"name": "dashboard",
			"path": "/dashboard",
			"waitFor": ".dashboard-loaded",
			"delay": 500
		}
	]
}
```

- `waitFor`: CSS selector to wait for before capturing
- `delay`: Additional milliseconds to wait after page load

### Multi-Viewport (Responsive Testing)

```json
{
	"baseUrl": "http://localhost:5173",
	"viewports": [
		{ "name": "desktop", "width": 1920, "height": 1080 },
		{ "name": "tablet", "width": 768, "height": 1024 },
		{ "name": "mobile", "width": 375, "height": 667 }
	],
	"shots": [
		{ "name": "home", "path": "/" }
	]
}
```

Screenshots are named `{shot}@{viewport}.png` (e.g., `home@mobile.png`). See the [multi-viewport](./multi-viewport) example for a complete demo.

## Available Scripts

Each example includes these npm scripts:

| Script                                  | Description                     |
| --------------------------------------- | ------------------------------- |
| `npm run dev`                           | Start the development server    |
| `npm run pixelguard`                    | Run the pixelguard CLI directly |
| `npm run pixelguard:test:visual`        | Run visual regression tests     |
| `npm run pixelguard:test:visual:update` | Update baseline screenshots     |

## Prerequisites

Before running examples, ensure you have:

1. **Node.js 18+** installed
2. **Playwright browsers** installed:
    ```bash
    npx playwright install chromium
    ```
3. **Pixelguard binary** built:
    ```bash
    # From the repository root
    cargo build --release
    ```

## Troubleshooting

### "Connection refused" errors

The dev server must be running before executing Pixelguard commands. Start the server in one terminal, then run Pixelguard in another.

### Screenshots look different

- Check viewport settings in `pixelguard.config.json`
- Ensure fonts are loaded (add `delay` if needed)
- Verify the dev server is fully ready before capturing

### Storybook stories not discovered

- Ensure Storybook is running and accessible
- Check that `/index.json` endpoint returns stories
- For older Storybook versions, `/stories.json` is also supported

## Further Reading

- [Configuration Reference](../docs/configuration.md)
- [Plugin System](../docs/plugins.md)
- [CI/CD Setup](../docs/ci-setup.md)
- [Troubleshooting Guide](../docs/troubleshooting.md)
