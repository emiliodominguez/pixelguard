# Pixelguard Example Plugins

This directory contains example plugins demonstrating how to extend Pixelguard with custom functionality.

## Available Examples

| Plugin | Category | Description |
|--------|----------|-------------|
| [pixelguard-plugin-json-reporter](./pixelguard-plugin-json-reporter/) | Reporter | Generates machine-readable JSON reports |
| [pixelguard-plugin-console-notifier](./pixelguard-plugin-console-notifier/) | Notifier | Prints formatted results to console |
| [pixelguard-plugin-slack-notifier](./pixelguard-plugin-slack-notifier/) | Notifier | Sends results to Slack via webhook |
| [pixelguard-plugin-s3-storage](./pixelguard-plugin-s3-storage/) | Storage | Stores baselines in AWS S3 |
| [pixelguard-plugin-puppeteer-capture](./pixelguard-plugin-puppeteer-capture/) | Capture | Uses Puppeteer instead of Playwright |
| [pixelguard-plugin-ssim-differ](./pixelguard-plugin-ssim-differ/) | Differ | Uses SSIM for perceptual comparison |

## Quick Demo

The easiest way to try the plugins is with the storybook-react example:

```bash
# 1. Build pixelguard
cd /path/to/pixelguard
cargo build --release

# 2. Go to the example project
cd examples/storybook-react

# 3. Install dependencies
npm install

# 4. Start Storybook (in one terminal)
npm run storybook

# 5. Run tests with plugins (in another terminal)
npm run pixelguard:test:plugins
```

This uses `pixelguard.plugins.config.json` which loads:
- **JSON Reporter**: Outputs `results.json` in the output directory
- **Console Notifier**: Prints a formatted summary to the terminal

## Using Local Plugins

You can reference local plugins by path in your config:

```json
{
  "plugins": [
    "../plugins/pixelguard-plugin-json-reporter",
    "../plugins/pixelguard-plugin-console-notifier"
  ],
  "pluginOptions": {
    "../plugins/pixelguard-plugin-json-reporter": {
      "filename": "results.json",
      "pretty": true
    }
  }
}
```

## Plugin Categories

### Reporter Plugins
Generate additional report formats. Stack with the built-in HTML report.

**Example**: JSON Reporter outputs:
```json
{
  "summary": { "total": 5, "unchanged": 4, "changed": 1 },
  "changed": [{ "name": "button--primary", "diffPercentage": 2.5 }]
}
```

### Notifier Plugins
Send results to external services. Multiple notifiers can run.

**Example**: Console Notifier outputs:
```
========================================
       PIXELGUARD TEST RESULTS
========================================

Status: CHANGES DETECTED

+----------+---------+
| Category |   Count |
+----------+---------+
| Unchanged|       4 |
| Changed  |       1 |
| Added    |       0 |
| Removed  |       0 |
+----------+---------+
```

### Storage Plugins
Store baselines remotely instead of locally. Only one active.

### Capture Plugins
Use alternative screenshot engines. Only one active.

### Differ Plugins
Use alternative comparison algorithms. Only one active.

## Creating Your Own Plugin

1. Create a new directory with `package.json`:

```json
{
  "name": "my-pixelguard-plugin",
  "main": "src/index.js",
  "pixelguard": {
    "name": "My Plugin",
    "category": "notifier",
    "entry": "src/index.js",
    "hooks": ["notify"]
  }
}
```

2. Implement the required hooks in `src/index.js`:

```javascript
async function notify(input) {
  const { result, options } = input;
  console.log(`Total: ${result.unchanged.length + result.changed.length}`);
}

module.exports = { notify };
```

3. Reference it in your config:

```json
{
  "plugins": ["./path/to/my-pixelguard-plugin"]
}
```

## TypeScript Support

For TypeScript plugins, use the `@pixelguard/plugin-types` package:

```typescript
import type { NotifierPlugin, NotifierInput } from '@pixelguard/plugin-types';

export const notify: NotifierPlugin['notify'] = async (input: NotifierInput) => {
  // TypeScript knows the shape of input
  const { result, reportPath, ciMode, options } = input;
};
```

See the [plugin-types package](../../npm/plugin-types/) for all available types.
