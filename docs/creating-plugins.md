# Creating Pixelguard Plugins

This guide walks you through creating a Pixelguard plugin from scratch. We'll build a Slack notifier plugin as an example.

## Plugin Overview

Pixelguard plugins are Node.js packages that communicate with Pixelguard via JSON over stdin/stdout. They can:

- **Notify** - Send notifications (Slack, Teams, email)
- **Report** - Generate custom reports (JUnit XML, JSON)
- **Capture** - Provide custom screenshot capture
- **Diff** - Implement custom image comparison
- **Store** - Use custom storage backends (S3, GCS)

## Quick Start

### 1. Create the Package

```bash
mkdir pixelguard-plugin-slack-notifier
cd pixelguard-plugin-slack-notifier
npm init -y
```

### 2. Create the Manifest

Create `pixelguard-plugin.json`:

```json
{
  "name": "pixelguard-plugin-slack-notifier",
  "category": "notifier",
  "entry": "index.js",
  "hooks": ["notify"],
  "version": "1.0.0"
}
```

### 3. Create the Entry Point

Create `index.js`:

```javascript
const input = JSON.parse(process.argv[2]);
const { hook, input: hookInput, options } = input;

if (hook === 'notify') {
  const { result, ciMode } = hookInput;
  const { webhookUrl } = options;

  // Build message
  const message = buildSlackMessage(result);

  // Send to Slack
  fetch(webhookUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(message)
  }).then(() => {
    console.log(JSON.stringify({ success: true }));
  }).catch(err => {
    console.log(JSON.stringify({ success: false, error: err.message }));
  });
}

function buildSlackMessage(result) {
  const { unchanged, changed, added, removed } = result;
  const hasChanges = changed.length > 0 || added.length > 0 || removed.length > 0;

  return {
    blocks: [
      {
        type: 'header',
        text: {
          type: 'plain_text',
          text: hasChanges ? 'Visual Changes Detected' : 'Visual Tests Passed'
        }
      },
      {
        type: 'section',
        fields: [
          { type: 'mrkdwn', text: `*Unchanged:* ${unchanged.length}` },
          { type: 'mrkdwn', text: `*Changed:* ${changed.length}` },
          { type: 'mrkdwn', text: `*Added:* ${added.length}` },
          { type: 'mrkdwn', text: `*Removed:* ${removed.length}` }
        ]
      }
    ]
  };
}
```

### 4. Test Locally

```bash
# In your project using Pixelguard
npm link ../pixelguard-plugin-slack-notifier
```

Add to `pixelguard.config.json`:

```json
{
  "plugins": [
    {
      "name": "pixelguard-plugin-slack-notifier",
      "options": {
        "webhookUrl": "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
      }
    }
  ]
}
```

Run tests:

```bash
npx pixelguard test
```

## Plugin Manifest

The `pixelguard-plugin.json` manifest is required. It tells Pixelguard about your plugin.

```json
{
  "name": "pixelguard-plugin-example",
  "category": "notifier",
  "entry": "dist/index.js",
  "hooks": ["notify"],
  "version": "1.0.0",
  "optionsSchema": {
    "type": "object",
    "properties": {
      "webhookUrl": {
        "type": "string",
        "description": "Slack webhook URL"
      }
    },
    "required": ["webhookUrl"]
  }
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Package name |
| `category` | string | Yes | Plugin category (see below) |
| `entry` | string | Yes | Path to entry point (relative to package) |
| `hooks` | string[] | Yes | Hooks this plugin implements |
| `version` | string | Yes | Plugin version |
| `optionsSchema` | object | No | JSON Schema for validating options |

### Categories

| Category | Hooks | Behavior |
|----------|-------|----------|
| `notifier` | `notify` | Stackable (all run) |
| `reporter` | `generate` | Stackable (all run) |
| `capture` | `capture` | Single-winner (last wins) |
| `differ` | `diff` | Single-winner (last wins) |
| `storage` | `read`, `write`, `exists`, `list`, `copy`, `delete` | Single-winner (last wins) |

## Hook Interfaces

### Notifier: `notify`

Called after test comparison completes.

**Input:**
```typescript
interface NotifierInput {
  result: {
    unchanged: string[];
    changed: {
      name: string;
      baselinePath: string;
      currentPath: string;
      diffPath: string;
      diffPercentage: number;
    }[];
    added: string[];
    removed: string[];
  };
  reportPath: string | null;
  reportUrl: string | null;
  ciMode: boolean;
  options: Record<string, unknown>;
}
```

**Output:**
```typescript
interface NotifierOutput {
  success: boolean;
  error?: string;
}
```

### Reporter: `generate`

Called to generate custom reports.

**Input:**
```typescript
interface ReporterInput {
  result: {
    unchanged: string[];
    changed: {
      name: string;
      baselinePath: string;
      currentPath: string;
      diffPath: string;
      diffPercentage: number;
    }[];
    added: string[];
    removed: string[];
  };
  config: {
    source: string;
    baseUrl: string;
    threshold: number;
  };
  outputDir: string;
  options: Record<string, unknown>;
}
```

**Output:**
```typescript
interface ReporterOutput {
  success: boolean;
  outputPath?: string;
  error?: string;
}
```

### Capture: `capture`

Called to capture screenshots (replaces built-in capture).

**Input:**
```typescript
interface CaptureInput {
  shots: {
    name: string;
    path: string;
    waitFor?: string;
    delay?: number;
  }[];
  baseUrl: string;
  viewport: {
    width: number;
    height: number;
  };
  outputDir: string;
  options: Record<string, unknown>;
}
```

**Output:**
```typescript
interface CaptureOutput {
  captured: {
    name: string;
    path: string;
  }[];
  failed: {
    name: string;
    error: string;
  }[];
}
```

### Differ: `diff`

Called to compare images (replaces built-in diff).

**Input:**
```typescript
interface DifferInput {
  baseline: string;  // Path to baseline image
  current: string;   // Path to current image
  output: string;    // Path to write diff image
  threshold: number;
  options: Record<string, unknown>;
}
```

**Output:**
```typescript
interface DifferOutput {
  match: boolean;
  diffPercentage: number;
  error?: string;
}
```

## TypeScript Support

For TypeScript plugins, install the types package:

```bash
npm install -D @pixelguard/plugin-types
```

Use the types:

```typescript
import type {
  NotifierInput,
  NotifierOutput,
  PluginInput
} from '@pixelguard/plugin-types';

const input: PluginInput = JSON.parse(process.argv[2]);

if (input.hook === 'notify') {
  const hookInput = input.input as NotifierInput;
  // Type-safe access to hookInput.result, etc.
}
```

## Complete Example: JSON Reporter

Let's build a complete JSON reporter plugin.

### Package Structure

```
pixelguard-plugin-json-reporter/
├── package.json
├── pixelguard-plugin.json
├── tsconfig.json
└── src/
    └── index.ts
```

### package.json

```json
{
  "name": "pixelguard-plugin-json-reporter",
  "version": "1.0.0",
  "main": "dist/index.js",
  "scripts": {
    "build": "tsc",
    "prepublishOnly": "npm run build"
  },
  "devDependencies": {
    "@pixelguard/plugin-types": "^1.0.0",
    "typescript": "^5.0.0"
  }
}
```

### pixelguard-plugin.json

```json
{
  "name": "pixelguard-plugin-json-reporter",
  "category": "reporter",
  "entry": "dist/index.js",
  "hooks": ["generate"],
  "version": "1.0.0",
  "optionsSchema": {
    "type": "object",
    "properties": {
      "outputFile": {
        "type": "string",
        "default": "report.json",
        "description": "Output file name"
      },
      "pretty": {
        "type": "boolean",
        "default": true,
        "description": "Pretty-print JSON output"
      }
    }
  }
}
```

### src/index.ts

```typescript
import * as fs from 'fs';
import * as path from 'path';
import type { PluginInput, ReporterInput, ReporterOutput } from '@pixelguard/plugin-types';

const input: PluginInput = JSON.parse(process.argv[2]);

if (input.hook === 'generate') {
  const { input: hookInput, options } = input;
  const reporterInput = hookInput as ReporterInput;

  const outputFile = (options.outputFile as string) || 'report.json';
  const pretty = options.pretty !== false;

  const report = {
    timestamp: new Date().toISOString(),
    summary: {
      total: getTotalCount(reporterInput.result),
      unchanged: reporterInput.result.unchanged.length,
      changed: reporterInput.result.changed.length,
      added: reporterInput.result.added.length,
      removed: reporterInput.result.removed.length,
      passed: !hasChanges(reporterInput.result)
    },
    config: reporterInput.config,
    results: reporterInput.result
  };

  const outputPath = path.join(reporterInput.outputDir, outputFile);
  const json = pretty
    ? JSON.stringify(report, null, 2)
    : JSON.stringify(report);

  fs.writeFileSync(outputPath, json);

  const output: ReporterOutput = {
    success: true,
    outputPath
  };

  console.log(JSON.stringify(output));
}

function getTotalCount(result: ReporterInput['result']): number {
  return (
    result.unchanged.length +
    result.changed.length +
    result.added.length +
    result.removed.length
  );
}

function hasChanges(result: ReporterInput['result']): boolean {
  return (
    result.changed.length > 0 ||
    result.added.length > 0 ||
    result.removed.length > 0
  );
}
```

### Build and Test

```bash
# Build
npm run build

# Link for local testing
cd ../your-project
npm link ../pixelguard-plugin-json-reporter

# Add to config
# pixelguard.config.json:
# { "plugins": ["pixelguard-plugin-json-reporter"] }

# Run tests
npx pixelguard test
# Check .pixelguard/report.json
```

## Debugging Plugins

### Enable Debug Logging

Set `DEBUG=pixelguard:*` to see plugin execution details:

```bash
DEBUG=pixelguard:* npx pixelguard test
```

### Log to stderr

Use `console.error()` for debug output (stdout is reserved for JSON responses):

```javascript
console.error('Debug: Processing', shots.length, 'shots');
// ... do work ...
console.log(JSON.stringify({ success: true })); // Response to stdout
```

### Test Plugin Directly

Run your plugin directly with test input:

```bash
node dist/index.js '{"hook":"notify","input":{"result":{"unchanged":["a","b"],"changed":[],"added":[],"removed":[]},"ciMode":false},"options":{"webhookUrl":"test"}}'
```

## Publishing Plugins

### Naming Convention

Use the prefix `pixelguard-plugin-`:

- `pixelguard-plugin-slack-notifier`
- `pixelguard-plugin-json-reporter`
- `pixelguard-plugin-s3-storage`

### Package.json Keywords

Add keywords for discoverability:

```json
{
  "keywords": [
    "pixelguard",
    "pixelguard-plugin",
    "visual-regression",
    "screenshot-testing"
  ]
}
```

### npm Publish

```bash
# Build
npm run build

# Test
npm pack  # Creates tarball for local testing

# Publish
npm publish
```

### Documentation

Include a README.md with:

1. What the plugin does
2. Installation instructions
3. Configuration options
4. Example usage

## Plugin Ideas

Here are some plugin ideas to inspire you:

### Notifiers
- Discord webhook notifier
- Microsoft Teams notifier
- Email notifier (SMTP)
- GitHub PR comment notifier

### Reporters
- JUnit XML reporter (for CI integration)
- Markdown reporter
- CSV reporter
- Custom HTML template reporter

### Storage
- AWS S3 storage
- Google Cloud Storage
- Azure Blob Storage
- Custom HTTP API storage

### Capture
- Puppeteer-based capture (alternative to Playwright)
- Safari/WebKit capture
- Mobile device capture (via cloud services)

### Differ
- AI-powered diff (ignore expected animations)
- Perceptual diff (focus on human-visible changes)
- Region-of-interest diff (ignore specific areas)
