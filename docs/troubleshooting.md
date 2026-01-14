# Troubleshooting

Common issues and solutions when using Pixelguard.

## Installation Issues

### "Cannot find module 'playwright'"

Playwright is not installed. Install it with:

```bash
npm install playwright
npx playwright install chromium
```

### "Failed to execute Node.js"

Node.js is not in your PATH. Ensure Node.js 18+ is installed:

```bash
node --version  # Should be v18.0.0 or higher
```

### Binary download fails

If the prebuilt binary fails to download, you can build from source:

```bash
git clone https://github.com/emiliodominguez/pixelguard.git
cd pixelguard
cargo build --release
# Binary will be at target/release/pixelguard
```

## Detection Issues

### "Could not auto-detect project type"

Pixelguard couldn't find your framework. Make sure:

1. Your dev server is running
2. It's on a standard port (6006 for Storybook, 3000 for Next.js, 5173 for Vite)
3. The framework config file exists (`.storybook/`, `next.config.js`, etc.)

You can configure shots manually in `pixelguard.config.json`:

```json
{
  "baseUrl": "http://localhost:8080",
  "shots": [
    {
      "name": "home",
      "path": "/"
    }
  ]
}
```

### "Discovered 0 stories"

For Storybook, ensure:

1. Storybook is running
2. The `/index.json` or `/stories.json` endpoint is accessible
3. You're using Storybook 6.4+ (earlier versions don't expose these endpoints)

Check manually:
```bash
curl http://localhost:6006/index.json
```

## Screenshot Issues

### Screenshots are blank or wrong size

The page might not be fully loaded. Increase the delay:

```json
{
  "shots": [
    {
      "name": "slow-loading-component",
      "path": "/path",
      "waitFor": "#main-content",
      "delay": 1000
    }
  ]
}
```

### Screenshots differ between local and CI

This is usually caused by:

1. **Different browser versions**: Pin Playwright version in `package.json`
2. **Font rendering**: Use web fonts instead of system fonts
3. **Timezone differences**: Mock dates in your components
4. **Random data**: Use deterministic test data

### Animations cause flaky tests

Disable animations in your app during tests:

```css
/* Add to your test styles */
*, *::before, *::after {
  animation-duration: 0s !important;
  animation-delay: 0s !important;
  transition-duration: 0s !important;
}
```

Or in Storybook's `.storybook/preview.js`:

```javascript
export const parameters = {
  chromatic: { disableSnapshot: false },
};
```

## Diff Issues

### False positives (unchanged shots marked as changed)

Increase the threshold:

```json
{
  "threshold": 0.5
}
```

Or the issue might be:
- Anti-aliasing differences (use consistent browser)
- Sub-pixel rendering (round dimensions to even numbers)
- Font smoothing (standardize fonts)

### False negatives (changed shots marked as unchanged)

Decrease the threshold:

```json
{
  "threshold": 0.01
}
```

### "100% different" for similar images

This happens when image dimensions don't match. Ensure consistent viewport:

```json
{
  "viewport": {
    "width": 1280,
    "height": 720
  }
}
```

## Performance Issues

### Screenshots are slow

1. **Increase concurrency**: Capture more screenshots in parallel
   ```json
   {
     "concurrency": 8
   }
   ```
2. **Reduce delays**: Only use delays where necessary
3. **Filter shots**: Use `--filter` to test specific shots
   ```bash
   npx pixelguard test --filter "button"
   ```

### Too many screenshots

Use `exclude` patterns:

```json
{
  "exclude": [
    "**/*Deprecated*",
    "**/*Internal*",
    "**/docs-*"
  ]
}
```

## CI-Specific Issues

### Tests pass locally but fail in CI

1. **Check browser versions**: Ensure same Playwright version
2. **Check fonts**: Install fonts in CI or use web fonts
3. **Check viewport**: CI might have different screen size
4. **Check resources**: CI might be slower, increase delays

### Can't access artifacts

Ensure artifacts are uploaded on failure:

```yaml
- uses: actions/upload-artifact@v4
  if: failure()
  with:
    name: pixelguard-report
    path: .pixelguard/
```

### Git LFS issues

If baseline images aren't fetching:

```bash
git lfs install
git lfs pull
```

## Report Issues

### Report doesn't open

The report is a static HTML file. Open it directly in a browser:

```bash
# macOS
open .pixelguard/report.html

# Linux
xdg-open .pixelguard/report.html

# Windows
start .pixelguard/report.html
```

### Images not loading in report

Check that the image files exist in `.pixelguard/baseline/`, `.pixelguard/current/`, and `.pixelguard/diff/`.

## Getting Help

If your issue isn't listed here:

1. Check [GitHub Issues](https://github.com/emiliodominguez/pixelguard/issues)
2. Run with `--verbose` for detailed logs
3. Open a new issue with:
   - Pixelguard version
   - Node.js version
   - Operating system
   - Steps to reproduce
   - Error messages
   - Configuration (sanitized)
