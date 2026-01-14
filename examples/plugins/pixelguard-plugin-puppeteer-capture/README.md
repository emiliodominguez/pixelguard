# pixelguard-plugin-puppeteer-capture

A Pixelguard capture plugin that uses Puppeteer instead of the default Playwright.

## Why Use Puppeteer?

- **Existing dependency**: If your project already uses Puppeteer, avoid adding Playwright
- **Familiarity**: Stick with the browser automation tool your team knows
- **Specific features**: Access Puppeteer-specific features or extensions

## Installation

```bash
npm install pixelguard-plugin-puppeteer-capture puppeteer
```

## Configuration

Add the plugin to your `pixelguard.config.json`:

```json
{
  "source": "storybook",
  "baseUrl": "http://localhost:6006",
  "plugins": ["pixelguard-plugin-puppeteer-capture"],
  "pluginOptions": {
    "pixelguard-plugin-puppeteer-capture": {
      "headless": true,
      "timeout": 30000,
      "fullPage": false
    }
  }
}
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `headless` | `boolean` | `true` | Run browser in headless mode |
| `timeout` | `number` | `30000` | Navigation timeout in milliseconds |
| `fullPage` | `boolean` | `false` | Capture full scrollable page |
| `waitUntil` | `string` | `networkidle0` | When to consider navigation complete |
| `deviceScaleFactor` | `number` | `1` | Device scale factor (for retina) |

### waitUntil Options

- `load` - Wait for the `load` event
- `domcontentloaded` - Wait for `DOMContentLoaded` event
- `networkidle0` - Wait until no network connections for 500ms
- `networkidle2` - Wait until ≤2 network connections for 500ms

## How It Works

When you run `pixelguard test`, this plugin:

1. Launches a headless Chromium browser via Puppeteer
2. Creates a new page for each shot
3. Sets the viewport to configured dimensions
4. Navigates to the shot URL
5. Waits for any custom selector (from shot config)
6. Applies any delay (from shot config)
7. Takes a PNG screenshot
8. Closes the page and moves to the next shot

## Per-Shot Configuration

Individual shots can have custom wait conditions:

```json
{
  "shots": [
    {
      "name": "modal--open",
      "path": "/iframe.html?id=modal--open",
      "waitFor": ".modal-content",
      "delay": 500
    }
  ]
}
```

## Debugging

To debug capture issues, disable headless mode:

```json
{
  "pluginOptions": {
    "pixelguard-plugin-puppeteer-capture": {
      "headless": false,
      "timeout": 60000
    }
  }
}
```

This opens a visible browser window so you can see what's happening.

## CI/CD Considerations

### Docker

When running in Docker, use these browser arguments (already included):

```javascript
puppeteer.launch({
  args: [
    '--no-sandbox',
    '--disable-setuid-sandbox',
    '--disable-dev-shm-usage',
  ]
});
```

### GitHub Actions

```yaml
- name: Install dependencies
  run: npm ci

- name: Run visual tests
  run: npx pixelguard test --ci
```

### GitLab CI

```yaml
visual-tests:
  image: node:20
  before_script:
    - apt-get update
    - apt-get install -y chromium
  script:
    - npm ci
    - npx pixelguard test --ci
```

## Plugin Development

This plugin demonstrates:

- **Capture plugin interface**: Implementing the `capture` hook
- **Browser automation**: Launching and controlling Puppeteer
- **Error handling**: Graceful handling of page failures
- **Resource management**: Proper browser cleanup

### Key Implementation

```javascript
async function capture(input) {
  const { shots, baseUrl, viewport, outputDir, options } = input;

  const browser = await puppeteer.launch({
    headless: options.headless !== false ? 'new' : false,
  });

  const results = { captured: [], failed: [] };

  for (const shot of shots) {
    try {
      const page = await browser.newPage();
      await page.setViewport(viewport);
      await page.goto(`${baseUrl}${shot.path}`);

      if (shot.waitFor) {
        await page.waitForSelector(shot.waitFor);
      }

      if (shot.delay) {
        await new Promise(r => setTimeout(r, shot.delay));
      }

      const path = `${outputDir}/${shot.name}.png`;
      await page.screenshot({ path });

      results.captured.push({ name: shot.name, path });
      await page.close();
    } catch (error) {
      results.failed.push({ name: shot.name, error: String(error) });
    }
  }

  await browser.close();
  return results;
}
```

## Comparison: Puppeteer vs Playwright

| Feature | Puppeteer | Playwright |
|---------|-----------|------------|
| Browser support | Chromium, Firefox (experimental) | Chromium, Firefox, WebKit |
| Auto-wait | Manual | Built-in |
| Network interception | Yes | Yes |
| Download size | ~300MB (Chromium) | ~200MB per browser |
| API style | Promise-based | Promise-based with auto-wait |

## Troubleshooting

### "Failed to launch browser"

- Ensure Puppeteer is installed: `npm install puppeteer`
- In Docker, use a compatible base image with Chrome dependencies
- Try with `headless: false` to see if browser starts

### "Navigation timeout"

- Increase `timeout` option
- Check if the dev server is running
- Verify the URL is correct

### "Screenshot is blank"

- Add a `delay` to the shot configuration
- Use `waitFor` to wait for a specific element
- Check for JavaScript errors in the page

## License

MIT
