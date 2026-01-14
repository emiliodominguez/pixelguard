# Multi-Viewport Example

This example demonstrates Pixelguard's multi-viewport feature for responsive design testing.

## What's Showcased

- **Multiple viewports**: Test the same pages across desktop (1920x1080), tablet (768x1024), and mobile (375x667)
- **Responsive CSS**: The landing page adapts its layout for different screen sizes
- **Viewport-based naming**: Screenshots are named `{shot}@{viewport}.png`
- **JSON Schema**: Config file uses `$schema` for IDE autocomplete

## Quick Start

```bash
# 1. Install dependencies
npm install

# 2. Start the dev server
npm run dev

# 3. In another terminal, validate your environment
npm run pixelguard:validate

# 4. Create baseline screenshots (9 total: 3 pages x 3 viewports)
npm run pixelguard:test:visual:update

# 5. Run visual regression tests
npm run pixelguard:test:visual
```

## Configuration

The `pixelguard.config.json` demonstrates multi-viewport setup:

```json
{
  "$schema": "../../schemas/pixelguard.config.schema.json",
  "baseUrl": "http://localhost:5173",
  "viewports": [
    { "name": "desktop", "width": 1920, "height": 1080 },
    { "name": "tablet", "width": 768, "height": 1024 },
    { "name": "mobile", "width": 375, "height": 667 }
  ],
  "shots": [
    { "name": "home", "path": "/" },
    { "name": "features", "path": "/features" },
    { "name": "pricing", "path": "/pricing" }
  ]
}
```

## Generated Screenshots

After running tests, you'll have these screenshots in `.pixelguard/baseline/`:

```
.pixelguard/baseline/
├── home@desktop.png
├── home@tablet.png
├── home@mobile.png
├── features@desktop.png
├── features@tablet.png
├── features@mobile.png
├── pricing@desktop.png
├── pricing@tablet.png
└── pricing@mobile.png
```

## Validate Command

Before running tests, validate your environment:

```bash
npm run pixelguard:validate
```

This checks:
- Configuration file validity
- Node.js installation
- Playwright availability
- Base URL reachability

## Responsive Breakpoints

The example app uses these CSS breakpoints:

| Viewport | Width   | Layout Changes                          |
|----------|---------|----------------------------------------|
| Mobile   | < 768px | Stacked navigation, single column grid |
| Tablet   | 768px+  | Horizontal nav, 2-column grid          |
| Desktop  | 1024px+ | Full layout, 3-column grid             |

## Try It Out

1. Make a CSS change that only affects mobile (e.g., change button color in a media query)
2. Run `npm run pixelguard:test:visual`
3. Only the `@mobile` screenshots should show as changed
4. Open the report to see side-by-side comparison
