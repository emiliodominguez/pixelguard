# Pixelguard

Open-source visual regression testing CLI. Zero config, git-friendly, no backend required.

## Features

- **Zero Config** — Auto-detects Storybook projects with story discovery
- **Git-Friendly** — Screenshots stored in `.pixelguard/`, committed to your repo
- **Fast** — Written in Rust with parallel screenshot capture
- **No Backend** — Everything runs locally or in CI
- **Beautiful Reports** — Interactive HTML reports with filtering, search, and approval workflow
- **Review Workflows** — Approve/reject changes in browser or interactive CLI
- **Selective Updates** — Update specific shots without affecting others
- **Extensible** — Plugin system for custom storage, reporters, and notifiers

## Quick Start

```bash
# Initialize in your project
npx pixelguard init

# Capture screenshots and create baseline
npx pixelguard test --update

# Run visual regression tests
npx pixelguard test
```

## Requirements

- Node.js 18+ (for Playwright)
- Playwright (`npm install playwright`)

## Commands

### `pixelguard init`

Auto-detect your project type and create configuration.

```bash
npx pixelguard init

# Output:
# Detecting project type...
# ✓ Found Storybook at http://localhost:6006
# ✓ Discovered 47 stories
# ✓ Created pixelguard.config.json
```

Options:
- `--force` — Overwrite existing configuration
- `--port, -p <number>` — Port to use for dev server detection

### `pixelguard test`

Capture screenshots and compare against baseline.

```bash
npx pixelguard test

# Output:
# Capturing 47 screenshots...
# Comparing against baseline...
#
# ✓ 45 unchanged
# ✗ 2 changed
#
# View report: .pixelguard/report.html
```

Options:
- `--update` — Update baseline with current screenshots
- `--update-only <names>` — Update only specific shots (comma-separated)
- `--ci` — CI mode with machine-readable JSON output
- `--filter <pattern>` — Only test shots matching pattern
- `--config, -c <path>` — Use a custom config file
- `--verbose` — Show detailed progress
- `--serve` — Serve the HTML report in browser after completion
- `--port <number>` — Port for serving the report (default: 3333)

#### Selective Updates

Update only specific shots without affecting others:

```bash
# Update specific shots
npx pixelguard test --update-only=button--primary,card--hover

# Update viewport-specific shots
npx pixelguard test --update-only=header@mobile
```

### `pixelguard list`

List all configured shots without capturing.

```bash
npx pixelguard list

# Output:
# Configured shots (47):
#
#   button--primary   → /iframe.html?id=button--primary
#   button--secondary → /iframe.html?id=button--secondary
#   card--default     → /iframe.html?id=card--default
```

Options:
- `--config, -c <path>` — Use a custom config file
- `--json` — Output as JSON

### `pixelguard plugins`

List and validate installed plugins.

```bash
npx pixelguard plugins

# Output:
# Loaded plugins (2):
#
#   ✓ JSON Reporter  Reporter  [generate]
#   ✓ Slack Notifier Notifier  [notify]
```

Options:
- `--config, -c <path>` — Use a custom config file
- `--json` — Output as JSON

### `pixelguard validate`

Validate environment prerequisites before running tests.

```bash
npx pixelguard validate

# Output:
# Environment Validation
# ──────────────────────
# ✓ config     pixelguard.config.json found and valid
# ✓ node       Node.js v20.10.0
# ✓ playwright Playwright is installed
# ✓ base_url   http://localhost:6006 is reachable
```

Options:
- `--config, -c <path>` — Use a custom config file
- `--json` — Output as JSON
- `--skip-url-check` — Skip checking if base URL is reachable

### `pixelguard apply`

Apply decisions from an exported JSON file to update the baseline.

```bash
# Apply decisions exported from the HTML report
npx pixelguard apply pixelguard-decisions.json

# Preview what would be updated
npx pixelguard apply pixelguard-decisions.json --dry-run
```

Options:
- `--config, -c <path>` — Use a custom config file
- `--dry-run` — Show what would be updated without making changes

### `pixelguard review`

Interactively review visual diffs in the terminal.

```bash
npx pixelguard review

# For each changed shot, prompts:
# [1/5] button--primary (5.23% different)
# What would you like to do?
# > Approve (update baseline)
#   Reject (keep baseline)
#   Skip (decide later)
#   View diff image
#   Quit review
```

Options:
- `--config, -c <path>` — Use a custom config file
- `--results <path>` — Path to results.json
- `--open-diff` — Open diff images during review

## Configuration

Pixelguard uses `pixelguard.config.json` in your project root:

```json
{
  "source": "storybook",
  "baseUrl": "http://localhost:6006",
  "include": ["**/*"],
  "exclude": ["**/*Deprecated*"],
  "viewport": {
    "width": 1280,
    "height": 720
  },
  "threshold": 0.01,
  "outputDir": ".pixelguard",
  "concurrency": 4
}
```

All fields are optional with sensible defaults. **Stories are discovered dynamically** from Storybook at test time.

### Configuration Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `source` | string | `""` | Project type (`storybook`, `nextjs`, `vite`, `manual`) |
| `baseUrl` | string | `""` | Base URL of your dev server |
| `include` | string[] | `["**/*"]` | Glob patterns for shots to include |
| `exclude` | string[] | `[]` | Glob patterns for shots to exclude |
| `viewport.width` | number | `1280` | Viewport width in pixels |
| `viewport.height` | number | `720` | Viewport height in pixels |
| `viewports` | NamedViewport[] | `[]` | Multiple named viewports for responsive testing |
| `threshold` | number | `0.01` | Diff threshold (0.0 to 100.0, percentage) |
| `outputDir` | string | `.pixelguard` | Directory for screenshots and reports |
| `concurrency` | number | `4` | Number of screenshots to capture in parallel |
| `shots` | Shot[] | `[]` | Optional overrides for specific shots |
| `plugins` | array | `[]` | Plugins to load (see [Plugins](docs/plugins.md)) |
| `pluginOptions` | object | `{}` | Options for plugins, keyed by plugin name |

### Shot Overrides

For Storybook projects, shots are discovered automatically. You can optionally provide overrides for specific shots that need custom configuration:

```json
{
  "shots": [
    {
      "name": "components-card--with-image",
      "delay": 500
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Shot name to override (must match discovered name) |
| `waitFor` | string | CSS selector to wait for before capture |
| `delay` | number | Delay in ms after page load |

### Multiple Viewports

Test across different screen sizes by specifying multiple viewports:

```json
{
  "viewports": [
    { "name": "desktop", "width": 1920, "height": 1080 },
    { "name": "tablet", "width": 768, "height": 1024 },
    { "name": "mobile", "width": 375, "height": 667 }
  ]
}
```

Screenshots are named `{shot}@{viewport}.png` (e.g., `button--primary@mobile.png`).

## CI Integration

### GitHub Actions

Add this to your workflow:

```yaml
name: Visual Regression

on: [pull_request]

jobs:
  visual-regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright
        run: npx playwright install chromium

      - name: Start Storybook
        run: npm run storybook &

      - name: Wait for Storybook
        run: npx wait-on http://localhost:6006

      - name: Run visual regression tests
        run: npx pixelguard test --ci

      - name: Upload report
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: pixelguard-report
          path: .pixelguard/report.html
```

## Review Workflows

Pixelguard offers multiple ways to review and approve visual changes:

### Browser-Based Review

1. Run tests: `npx pixelguard test --serve`
2. Open the HTML report in your browser
3. Use **Filter & Search** to find specific shots
4. Click **Approve** or **Reject** on each changed shot
5. Click **Export** to download `pixelguard-decisions.json`
6. Apply decisions: `npx pixelguard apply pixelguard-decisions.json`

### Interactive CLI Review

```bash
npx pixelguard review
```

Step through each changed shot and approve, reject, or skip interactively.

### Selective Update

Update specific shots directly without full review:

```bash
npx pixelguard test --update-only=button--primary,card--hover
```

## How It Works

1. **Detection**: Pixelguard probes common dev server ports and checks for framework config files
2. **Capture**: Uses Playwright headless Chromium to take screenshots
3. **Diff**: Compares images pixel-by-pixel with anti-aliasing tolerance
4. **Report**: Generates interactive HTML report with filtering, sorting, and approval actions
5. **Export**: Creates machine-readable `results.json` for CI integration

## Storage

Screenshots and reports are stored in `.pixelguard/`:

```
.pixelguard/
├── baseline/          # Reference screenshots (commit these)
│   ├── button--primary.png
│   └── card--default.png
├── current/           # Latest captures
│   ├── button--primary.png
│   └── card--default.png
├── diff/              # Visual diffs
│   └── card--default.png
├── report.html        # Interactive HTML report
└── results.json       # Machine-readable JSON export
```

Commit the `baseline/` directory to your repository. The `current/`, `diff/`, `report.html`, and `results.json` are regenerated on each run.

For large projects, consider using [Git LFS](https://git-lfs.github.com/) for PNG files.

### JSON Export

The `results.json` file is generated alongside the HTML report for CI integration:

```json
{
  "version": "1.0",
  "timestamp": "2026-01-14T12:00:00Z",
  "summary": {
    "total": 47,
    "unchanged": 45,
    "changed": 2,
    "passed": false
  },
  "results": {
    "changed": [
      { "name": "button--primary", "diffPercentage": 5.5 }
    ]
  }
}
```

## Plugins

Pixelguard supports plugins for extending functionality:

| Category | Purpose | Examples |
|----------|---------|----------|
| **storage** | Remote baseline storage | S3, Cloudflare R2, Azure Blob |
| **reporter** | Custom report formats | JSON, JUnit XML |
| **notifier** | Send notifications | Slack, Teams, webhooks |
| **capture** | Screenshot engine | Puppeteer |
| **differ** | Image comparison | SSIM |

```json
{
  "plugins": ["pixelguard-plugin-slack-notifier"],
  "pluginOptions": {
    "pixelguard-plugin-slack-notifier": {
      "webhookUrl": "https://hooks.slack.com/..."
    }
  }
}
```

See [Plugins Documentation](docs/plugins.md) for full details.

## Documentation

- [Getting Started](docs/getting-started.md) — Quick setup guide
- [Configuration](docs/configuration.md) — Full configuration reference
- [Plugins](docs/plugins.md) — Plugin system and creating plugins
- [CI Setup](docs/ci-setup.md) — CI/CD integration guide
- [Troubleshooting](docs/troubleshooting.md) — Common issues and solutions

## License

MIT
