# Getting Started with Pixelguard

This guide will help you set up visual regression testing in your project in under 2 minutes.

## Prerequisites

- **Node.js 18+** - [Download](https://nodejs.org/)
- **Playwright** - Will be installed automatically or run `npm install playwright`

## Installation

```bash
npm install pixelguard playwright
```

Or use directly with npx:

```bash
npx pixelguard
```

## Quick Start

### 1. Initialize Pixelguard

Run the init command in your project root:

```bash
npx pixelguard init
```

Pixelguard will automatically detect your project type:

- **Storybook**: Discovers all stories from `/index.json`
- **Next.js**: Scans `app/` and `pages/` directories for routes
- **Vite**: Detects the dev server (manual shot configuration required)

Example output:

```
Detecting project type...
✓ Found Storybook at http://localhost:6006
✓ Discovered 47 stories
✓ Created pixelguard.config.json

Next steps:
  1. Run: npx pixelguard test
  2. Commit .pixelguard/ as your baseline
```

### 2. Start Your Dev Server

Before running tests, make sure your development server is running:

```bash
# Storybook
npm run storybook

# Next.js
npm run dev

# Vite
npm run dev
```

### 3. Create Your Baseline

Capture the initial screenshots that will serve as your baseline:

```bash
npx pixelguard test --update
```

This will:
1. Launch a headless browser
2. Navigate to each configured shot
3. Take screenshots
4. Save them to `.pixelguard/baseline/`

### 4. Commit the Baseline

Add the baseline to your repository:

```bash
git add .pixelguard/
git commit -m "Add visual regression baseline"
```

### 5. Run Visual Regression Tests

After making changes to your UI, run the tests:

```bash
npx pixelguard test
```

If there are visual differences, you'll see a report:

```
Capturing 47 screenshots...
Comparing against baseline...

✓ 45 unchanged
✗ 2 changed
    button--primary (5.23% different)
    card--hover (12.87% different)

View report: .pixelguard/report.html
```

### 6. Review and Update

Open the HTML report in your browser to review the differences:

```bash
# Automatically serve the report after tests
npx pixelguard test --serve

# Or open manually
open .pixelguard/report.html  # macOS
xdg-open .pixelguard/report.html  # Linux
start .pixelguard/report.html  # Windows
```

If the changes are intentional, update the baseline:

```bash
npx pixelguard test --update
git add .pixelguard/
git commit -m "Update visual regression baseline"
```

## Project Structure

After initialization, your project will have:

```
your-project/
├── pixelguard.config.json    # Configuration file
└── .pixelguard/
    ├── baseline/             # Reference screenshots (commit these)
    ├── current/              # Latest captures (generated)
    ├── diff/                 # Visual diffs (generated)
    └── report.html           # HTML report (generated)
```

## CLI Reference

### `pixelguard init`

Initialize Pixelguard in the current project.

```bash
npx pixelguard init [--force]
```

### `pixelguard test`

Capture screenshots and compare against baseline.

```bash
npx pixelguard test [options]
```

Options:
- `--update` - Update baseline with current screenshots
- `--ci` - CI mode (machine-readable output, exit code 1 on diffs)
- `--filter <pattern>` - Only test shots matching pattern
- `--config, -c <path>` - Use a custom config file
- `--serve` - Serve the HTML report after completion
- `--port <number>` - Port for serving the report (default: 3333)
- `--verbose` - Show detailed progress

### `pixelguard list`

List all configured shots without capturing.

```bash
npx pixelguard list [--config <path>]
```

### `pixelguard plugins`

List and validate installed plugins.

```bash
npx pixelguard plugins [--config <path>] [--json]
```

Options:
- `--config, -c <path>` - Use a custom config file
- `--json` - Output in JSON format

## Next Steps

- [Configuration Reference](./configuration.md) - Customize your setup
- [CI Setup Guide](./ci-setup.md) - Run in continuous integration
- [Plugins](./plugins.md) - Extend functionality with plugins
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions
