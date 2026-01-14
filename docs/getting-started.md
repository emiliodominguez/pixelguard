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

Pixelguard will automatically detect your project:

- **Storybook**: Auto-discovers all stories from `/index.json`
- **Other projects**: Detects the dev server, you configure shots manually

Example output for Storybook:

```
Detecting project type...
✓ Found Storybook at http://localhost:6006
✓ Discovered 47 stories
✓ Created pixelguard.config.json

Next steps:
  1. Run: npx pixelguard test
  2. Commit .pixelguard/ as your baseline
```

For non-Storybook projects:

```
Detecting project type...
✓ Found dev server at http://localhost:3000
  Note: Add shots to 'pixelguard.config.json' to specify
  which pages or components to capture.
✓ Created pixelguard.config.json

Next steps:
  1. Edit pixelguard.config.json to add your shots
  2. Run: npx pixelguard test
  3. Commit .pixelguard/ as your baseline
```

### 2. Start Your Dev Server

Before running tests, make sure your development server is running:

```bash
# Storybook
npm run storybook

# Any other project
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

#### Report Features

The HTML report includes powerful review tools:

- **Filter & Search**: Search shots by name and filter by status (Changed, Added, Removed)
- **Sort Options**: Sort by name or diff percentage
- **Approve/Reject**: Click approve or reject on each changed shot
- **Export Decisions**: Export your decisions to a JSON file for later processing

#### Updating the Baseline

If all changes are intentional, update the entire baseline:

```bash
npx pixelguard test --update
git add .pixelguard/
git commit -m "Update visual regression baseline"
```

#### Selective Updates

Update only specific shots using `--update-only`:

```bash
# Update only certain shots
npx pixelguard test --update-only=button--primary,card--default

# Update a shot at a specific viewport
npx pixelguard test --update-only=button--primary@mobile
```

#### Interactive Review

Use the `review` command for an interactive terminal-based review:

```bash
npx pixelguard review
```

This lets you step through each changed shot and approve, reject, or skip.

#### Browser-Based Workflow (Recommended)

1. Run tests: `npx pixelguard test`
2. Serve the report: `npx pixelguard serve`
3. Click **Approve** or **Reject** on each changed shot (auto-saves)
4. Apply the decisions:

```bash
npx pixelguard apply
git add .pixelguard/
git commit -m "Apply reviewed visual changes"
```

Decisions are automatically saved to `.pixelguard/decisions.json` when using `serve`.

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
npx pixelguard init [--force] [--port <number>]
```

Options:
- `--force` - Re-initialize even if config exists
- `--port, -p <number>` - Port to use for dev server detection

### `pixelguard test`

Capture screenshots and compare against baseline.

```bash
npx pixelguard test [options]
```

Options:
- `--update` - Update baseline with current screenshots
- `--update-only <names>` - Update only specific shots (comma-separated, implies `--update`)
- `--ci` - CI mode (machine-readable output, exit code 1 on diffs)
- `--filter <pattern>` - Only test shots matching pattern
- `--config, -c <path>` - Use a custom config file
- `--serve` - Serve the HTML report after completion
- `--port <number>` - Port for serving the report (default: 3333)
- `--verbose` - Show detailed progress

Examples:

```bash
# Update entire baseline
npx pixelguard test --update

# Update only specific shots
npx pixelguard test --update-only=button--primary,card--hover

# Update shots matching a pattern at a viewport
npx pixelguard test --update-only=header@mobile
```

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

### `pixelguard validate`

Validate environment prerequisites before running tests.

```bash
npx pixelguard validate [options]
```

Options:
- `--config, -c <path>` - Use a custom config file
- `--json` - Output in JSON format
- `--skip-url-check` - Skip checking if base URL is reachable

This command checks:
- Configuration file validity
- Node.js installation
- Playwright installation
- Base URL reachability (when configured)

### `pixelguard apply`

Apply decisions from the HTML report to update the baseline.

```bash
npx pixelguard apply [decisions-file] [options]
```

Options:
- `--config, -c <path>` - Use a custom config file
- `--dry-run` - Show what would be updated without making changes

Example:

```bash
# Apply saved decisions (reads from .pixelguard/decisions.json)
npx pixelguard apply

# Preview what would be updated
npx pixelguard apply --dry-run

# Apply from a specific file
npx pixelguard apply path/to/decisions.json
```

### `pixelguard review`

Interactively review visual diffs in the terminal.

```bash
npx pixelguard review [options]
```

Options:
- `--config, -c <path>` - Use a custom config file
- `--results <path>` - Path to results.json (default: .pixelguard/results.json)
- `--open-diff` - Open diff images during review

This command:
1. Reads the results.json from the last test run
2. Prompts you to approve, reject, or skip each changed shot
3. Shows a summary of your decisions
4. Optionally updates the baseline with approved changes

### `pixelguard serve`

Serve an existing report for browser-based review without re-running tests.

```bash
npx pixelguard serve [options]
```

Options:
- `--config, -c <path>` - Use a custom config file
- `--port <number>` - Port for serving the report (default: 3333)

This is useful when you want to review a report from an earlier test run without needing your dev server running. Decisions are automatically saved to disk as you approve/reject.

## Generated Files

The `test` command generates several files:

| File | Description |
|------|-------------|
| `report.html` | Interactive HTML report with filtering, search, and approval workflow |
| `results.json` | Machine-readable JSON export with all test results |
| `baseline/*.png` | Baseline screenshots (commit these) |
| `current/*.png` | Current screenshots from the latest run |
| `diff/*.png` | Visual diff images highlighting changes |

The `results.json` file is useful for CI integration and custom tooling:

```json
{
  "version": "1.0",
  "timestamp": "2026-01-14T12:00:00Z",
  "summary": {
    "total": 47,
    "unchanged": 45,
    "changed": 2,
    "added": 0,
    "removed": 0,
    "passed": false
  },
  "results": {
    "changed": [
      { "name": "button--primary", "diffPercentage": 5.5 }
    ],
    "added": [],
    "removed": [],
    "unchanged": ["card--default", "..."]
  }
}
```

## Next Steps

- [Configuration Reference](./configuration.md) - Customize your setup
- [CI Setup Guide](./ci-setup.md) - Run in continuous integration
- [Plugins](./plugins.md) - Extend functionality with plugins
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions
