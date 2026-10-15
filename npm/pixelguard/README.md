# Pixelguard

Open-source visual regression testing CLI. Zero config, git-friendly, no backend required.

## Installation

```bash
npm install -D pixelguard
# or
npx pixelguard
```

The package downloads the prebuilt binary for your platform from the matching GitHub release and verifies it against SHA-256 checksums shipped inside the package. If your package manager skips install scripts (npm 12+ does by default), the binary is downloaded and verified the first time you run `pixelguard`.

Prebuilt binaries: macOS (arm64, x64), Linux glibc 2.30+ (x64, arm64), Windows (x64). Elsewhere, build from source and set `PIXELGUARD_BINARY`.

## Quick Start

```bash
# Initialize in your project
npx pixelguard init

# Capture screenshots and create baseline
npx pixelguard test --update

# Run visual regression tests
npx pixelguard test

# View the report in your browser
npx pixelguard test --serve
```

## Commands

- `pixelguard init` — Auto-detect project type and create config
- `pixelguard test` — Capture and compare screenshots
- `pixelguard list` — List configured shots
- `pixelguard review` — Approve or reject changes interactively in the terminal
- `pixelguard serve` — Serve the report to approve or reject changes in the browser
- `pixelguard apply` — Promote approved changes to the baseline
- `pixelguard validate` — Check Node.js, Playwright and the dev server
- `pixelguard plugins` — List and validate plugins

### Test Options

- `--update` — Update baseline with current screenshots
- `--update-only <names>` — Update only specific shots
- `--ci` — CI mode: JSON on stdout, exit code 1 on changes or failed captures
- `--filter <pattern>` — Only test matching shots
- `--serve` — Serve the report in browser after tests
- `--port <number>` — Port for serving (default: 3333)

## Requirements

- Node.js 18+
- Playwright (`npm install -D playwright && npx playwright install chromium`)

## Documentation

See the full documentation at [github.com/emiliodominguez/pixelguard](https://github.com/emiliodominguez/pixelguard).

## License

MIT
