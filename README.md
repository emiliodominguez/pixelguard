# Pixelguard

Open-source visual regression testing CLI. Zero config, git-friendly, no backend required.

## Features

- **Zero Config** — Auto-detects Storybook, Next.js, and Vite projects
- **Git-Friendly** — Screenshots stored in `.pixelguard/`, committed to your repo
- **Fast** — Written in Rust for blazing performance
- **No Backend** — Everything runs locally or in CI
- **Beautiful Reports** — Static HTML reports that work offline

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
- `--ci` — CI mode with machine-readable JSON output
- `--filter <pattern>` — Only test shots matching pattern
- `--verbose` — Show detailed progress
- `--serve` — Serve the HTML report in browser after completion
- `--port <number>` — Port for serving the report (default: 3333)

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
- `--json` — Output as JSON

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
  "outputDir": ".pixelguard"
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
| `threshold` | number | `0.01` | Diff threshold (0.0 to 100.0, percentage) |
| `outputDir` | string | `.pixelguard` | Directory for screenshots and reports |
| `shots` | Shot[] | `[]` | Optional overrides for specific shots |

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

## How It Works

1. **Detection**: Pixelguard probes common dev server ports and checks for framework config files
2. **Capture**: Uses Playwright headless Chromium to take screenshots
3. **Diff**: Compares images pixel-by-pixel with anti-aliasing tolerance
4. **Report**: Generates a static HTML report with side-by-side comparisons

## Storage

Screenshots are stored in `.pixelguard/`:

```
.pixelguard/
├── baseline/          # Reference screenshots
│   ├── button--primary.png
│   └── card--default.png
├── current/           # Latest captures
│   ├── button--primary.png
│   └── card--default.png
├── diff/              # Visual diffs
│   └── card--default.png
└── report.html        # HTML report
```

Commit the `baseline/` directory to your repository. The `current/` and `diff/` directories are regenerated on each run.

For large projects, consider using [Git LFS](https://git-lfs.github.com/) for PNG files.

## License

MIT
