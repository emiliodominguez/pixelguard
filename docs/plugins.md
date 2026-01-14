# Plugins

Pixelguard's plugin system lets you extend functionality with custom storage backends, reporters, screenshot engines, differ algorithms, and notifiers.

## Quick Start

1. Install a plugin:
   ```bash
   npm install pixelguard-plugin-slack-notifier
   ```

2. Add it to your config:
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

3. Run your tests:
   ```bash
   npx pixelguard test
   ```

## Plugin Categories

| Category | Purpose | Stacking |
|----------|---------|----------|
| **storage** | Where baselines are stored (S3, R2, Azure) | Single (last wins) |
| **capture** | Screenshot engine (Playwright, Puppeteer) | Single (last wins) |
| **differ** | Image comparison algorithm (pixel, SSIM) | Single (last wins) |
| **reporter** | Report formats (HTML, JSON, JUnit) | Multiple (all run) |
| **notifier** | Notifications (Slack, Teams, webhook) | Multiple (all run) |

**Single** means only one plugin of that category can be active. If you configure multiple, the last one wins.

**Multiple** means all configured plugins run. You can have both a Slack notifier and an email notifier.

## Configuration

### Basic Usage

List plugins by name (installed via npm):

```json
{
  "plugins": [
    "pixelguard-plugin-json-reporter",
    "pixelguard-plugin-slack-notifier"
  ]
}
```

### Plugin Options

Configure plugins via `pluginOptions`:

```json
{
  "plugins": ["pixelguard-plugin-slack-notifier"],
  "pluginOptions": {
    "pixelguard-plugin-slack-notifier": {
      "webhookUrl": "https://hooks.slack.com/...",
      "channel": "#visual-tests",
      "onlyOnFailure": true
    }
  }
}
```

### Local Plugins

Reference local plugins by path:

```json
{
  "plugins": [
    "./my-plugins/custom-reporter"
  ]
}
```

### Inline Options

You can also specify options inline:

```json
{
  "plugins": [
    {
      "name": "pixelguard-plugin-slack-notifier",
      "options": {
        "webhookUrl": "https://hooks.slack.com/..."
      }
    }
  ]
}
```

## Available Hooks

### Storage Hooks

| Hook | Input | Output | Description |
|------|-------|--------|-------------|
| `read` | `{ path, options }` | `{ data }` | Read a file (base64) |
| `write` | `{ path, data, options }` | - | Write a file (base64) |
| `exists` | `{ path, options }` | `{ exists }` | Check if file exists |
| `list` | `{ path, options }` | `{ files }` | List files in directory |
| `delete` | `{ path, options }` | - | Delete a file |

### Capture Hook

| Hook | Input | Output | Description |
|------|-------|--------|-------------|
| `capture` | `{ shots, baseUrl, viewport, outputDir, options }` | `{ captured, failed }` | Take screenshots |

### Differ Hook

| Hook | Input | Output | Description |
|------|-------|--------|-------------|
| `compare` | `{ baselinePath, currentPath, diffPath, threshold, options }` | `{ diffPercentage, matches }` | Compare two images |

### Reporter Hook

| Hook | Input | Output | Description |
|------|-------|--------|-------------|
| `generate` | `{ result, config, outputDir, options }` | - | Generate a report |

### Notifier Hook

| Hook | Input | Output | Description |
|------|-------|--------|-------------|
| `notify` | `{ result, reportPath, ciMode, options }` | - | Send notification |

## Creating a Plugin

### 1. Create package.json

```json
{
  "name": "pixelguard-plugin-my-notifier",
  "version": "1.0.0",
  "main": "src/index.js",
  "pixelguard": {
    "name": "My Notifier",
    "category": "notifier",
    "entry": "src/index.js",
    "hooks": ["notify"]
  }
}
```

The `pixelguard` field is required:
- `name`: Human-readable name
- `category`: One of `storage`, `capture`, `differ`, `reporter`, `notifier`
- `entry`: Path to the entry file
- `hooks`: Array of hooks this plugin implements

### 2. Implement Hooks

```javascript
// src/index.js

async function notify(input) {
  const { result, reportPath, ciMode, options } = input;

  const hasChanges = result.changed.length > 0 ||
                     result.added.length > 0 ||
                     result.removed.length > 0;

  if (options.onlyOnFailure && !hasChanges) {
    return; // Skip notification
  }

  // Send your notification here
  console.error(`Visual tests: ${result.unchanged.length} passed`);
}

module.exports = { notify };
```

### 3. Test Locally

Reference your plugin by path in the config:

```json
{
  "plugins": ["./path/to/my-plugin"]
}
```

## TypeScript Support

Install the type definitions:

```bash
npm install -D @pixelguard/plugin-types
```

Use them in your plugin:

```typescript
import type {
  NotifierPlugin,
  NotifierInput
} from '@pixelguard/plugin-types';

export const notify: NotifierPlugin['notify'] = async (input: NotifierInput) => {
  const { result, reportPath, ciMode, options } = input;
  // Your implementation
};
```

## Plugin Examples

See the [examples/plugins](../examples/plugins/) directory for complete examples:

- **JSON Reporter** - Outputs machine-readable JSON
- **Console Notifier** - Prints formatted summary to terminal
- **Slack Notifier** - Sends results to Slack
- **S3 Storage** - Stores baselines in AWS S3
- **Puppeteer Capture** - Uses Puppeteer instead of Playwright
- **SSIM Differ** - Uses SSIM for perceptual comparison

## Best Practices

### Error Handling

Provide clear error messages:

```javascript
async function notify(input) {
  if (!input.options.webhookUrl) {
    throw new Error(
      'Missing webhookUrl. Add it to pluginOptions in your config.'
    );
  }
  // ...
}
```

### Output

Use `console.error()` for user-facing output (stdout is reserved for the plugin protocol):

```javascript
// Good - user will see this
console.error('Notification sent successfully');

// Bad - interferes with plugin protocol
console.log('Notification sent');
```

### Options Validation

Validate options early and fail fast:

```javascript
async function capture(input) {
  const { shots, options } = input;

  if (options.timeout && typeof options.timeout !== 'number') {
    throw new Error('timeout must be a number');
  }

  // ...
}
```

### Performance

For reporters/notifiers, batch operations when possible:

```javascript
// Good - single API call
await sendBatchNotification(results);

// Less good - multiple API calls
for (const result of results) {
  await sendNotification(result);
}
```

## Troubleshooting

### Plugin not loading

1. Check the plugin is installed: `npm ls pixelguard-plugin-*`
2. Verify the `pixelguard` field in package.json
3. Check the entry file exists at the specified path

### Plugin errors

Run with verbose logging to see plugin output:

```bash
npx pixelguard test --verbose
```

### Hook not called

Ensure your plugin declares the hook in the `hooks` array in package.json.
