# Pixelguard

Open-source visual regression testing CLI. Zero config, git-friendly, no backend required.

## Installation

```bash
npm install pixelguard
# or
npx pixelguard
```

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

### Test Options

- `--update` — Update baseline with current screenshots
- `--ci` — CI mode with JSON output
- `--filter <pattern>` — Only test matching shots
- `--serve` — Serve the report in browser after tests
- `--port <number>` — Port for serving (default: 3333)

## Requirements

- Node.js 18+
- Playwright (`npm install playwright`)

## Documentation

See the full documentation at [github.com/emiliodominguez/pixelguard](https://github.com/emiliodominguez/pixelguard).

## License

MIT
