# Pixelguard

> Percy/Chromatic for people who don't want to pay $300/mo or give their screenshots to a vendor.

Pixelguard is an open-source visual regression testing CLI tool. Zero config by default, git-friendly storage, and no backend required.

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
```

## Requirements

- Node.js 18+
- Playwright (`npm install playwright`)

## Documentation

See the full documentation at [github.com/pixelguard/pixelguard](https://github.com/pixelguard/pixelguard).

## License

MIT
