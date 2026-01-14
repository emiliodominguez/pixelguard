# Performance Guide

This guide covers optimizing Pixelguard for speed and resource efficiency, especially important for large projects and CI environments.

## Performance Factors

Pixelguard performance depends on:

1. **Number of shots** - More shots = longer runtime
2. **Concurrency** - Parallel capture speeds things up
3. **Image sizes** - Larger images take longer to compare
4. **Network latency** - Slow dev servers bottleneck capture
5. **CI resources** - CPU, memory, and disk speed matter

## Concurrency Tuning

### Understanding Concurrency

The `concurrency` config controls how many screenshots are captured in parallel:

```json
{
  "concurrency": 4
}
```

Each concurrent capture:
- Uses a browser page (memory)
- Makes network requests (bandwidth)
- Writes to disk (I/O)

### Recommended Values

| Environment | Shots | Concurrency | Notes |
|-------------|-------|-------------|-------|
| Local (laptop) | < 50 | 4 | Default, balanced |
| Local (powerful) | < 50 | 8 | More parallelism |
| CI (standard runner) | Any | 4 | Conservative |
| CI (large runner) | > 100 | 8-12 | More resources |
| Memory constrained | Any | 2 | Reduce memory usage |

### Finding Optimal Concurrency

Run tests with different values and compare:

```bash
# Test with concurrency 2
time npx pixelguard test --config pixelguard.c2.json

# Test with concurrency 4
time npx pixelguard test --config pixelguard.c4.json

# Test with concurrency 8
time npx pixelguard test --config pixelguard.c8.json
```

Plot results - you'll typically see diminishing returns above a certain point.

## Memory Optimization

### Browser Memory

Each Playwright browser context uses ~50-150MB. With high concurrency:

```
concurrency=4  → ~200-600MB for browsers
concurrency=8  → ~400-1200MB for browsers
concurrency=16 → ~800-2400MB for browsers
```

### Image Processing Memory

Diff processing loads images into memory. For a 1920x1080 screenshot:

```
1 image (RGBA) = 1920 × 1080 × 4 bytes = ~8MB
3 images (baseline + current + diff) = ~24MB per comparison
```

For 100 changed shots processed in parallel: ~2.4GB peak

### Reducing Memory Usage

1. **Lower concurrency**:
   ```json
   { "concurrency": 2 }
   ```

2. **Use smaller viewports**:
   ```json
   { "viewport": { "width": 1280, "height": 720 } }
   ```

3. **Filter shots** during development:
   ```bash
   npx pixelguard test --filter "button"
   ```

4. **Increase Node.js heap** if needed:
   ```bash
   NODE_OPTIONS="--max-old-space-size=4096" npx pixelguard test
   ```

## CI Optimization

### Caching Dependencies

Cache Playwright browsers and node_modules:

```yaml
# GitHub Actions
- name: Cache Playwright browsers
  uses: actions/cache@v4
  with:
    path: ~/.cache/ms-playwright
    key: playwright-${{ runner.os }}-${{ hashFiles('**/package-lock.json') }}

- name: Cache node_modules
  uses: actions/cache@v4
  with:
    path: node_modules
    key: node-${{ runner.os }}-${{ hashFiles('**/package-lock.json') }}
```

### Parallel CI Jobs

Split tests across multiple CI jobs:

```yaml
jobs:
  visual-test:
    strategy:
      matrix:
        shard: [1, 2, 3, 4]
    steps:
      - run: npx pixelguard test --filter "shard-${{ matrix.shard }}"
```

Organize shots with naming conventions:

```json
{
  "shots": [
    { "name": "shard-1-button-primary", "path": "..." },
    { "name": "shard-1-button-secondary", "path": "..." },
    { "name": "shard-2-card-default", "path": "..." },
    { "name": "shard-2-card-large", "path": "..." }
  ]
}
```

### Pre-built Storybook

Don't build Storybook in CI - use a pre-built static export:

```yaml
# In your main CI workflow
- run: npm run build-storybook
- uses: actions/upload-artifact@v4
  with:
    name: storybook-static
    path: storybook-static/

# In visual regression workflow
- uses: actions/download-artifact@v4
  with:
    name: storybook-static
    path: storybook-static/
- run: npx serve storybook-static -p 6006 &
- run: npx wait-on http://localhost:6006
- run: npx pixelguard test --ci
```

### Lightweight CI Images

Use slim CI images with only required tools:

```yaml
container:
  image: mcr.microsoft.com/playwright:v1.40.0-focal
```

This image includes Playwright browsers pre-installed.

## Large Project Strategies

### Component-Based Testing

Instead of testing everything, test component libraries:

```json
{
  "include": ["components/**/*", "atoms/**/*"],
  "exclude": ["pages/**/*", "layouts/**/*"]
}
```

### Critical Path Testing

Identify critical visual components and test only those:

```json
{
  "shots": [
    { "name": "checkout-form", "path": "...", "critical": true },
    { "name": "product-card", "path": "...", "critical": true }
  ]
}
```

### Incremental Testing

Only test what changed (requires custom scripting):

```bash
#!/bin/bash
# Get changed files
CHANGED_FILES=$(git diff --name-only HEAD~1)

# Build filter pattern
PATTERN=""
for file in $CHANGED_FILES; do
  component=$(echo $file | sed 's/.*\/\([^/]*\)\..*/\1/')
  PATTERN="$PATTERN|$component"
done

# Run filtered test
npx pixelguard test --filter "${PATTERN:1}"
```

## Network Optimization

### Local Dev Server

Ensure your dev server is fast:

1. **Use production build** for testing:
   ```bash
   npm run build-storybook
   npx serve storybook-static -p 6006
   ```

2. **Disable hot reload** and dev tools:
   ```javascript
   // .storybook/main.js
   export default {
     features: {
       storyStoreV7: true,
     },
   };
   ```

### Wait Strategies

Avoid unnecessary waits:

```json
{
  "shots": [
    // Bad: Always waits 2 seconds
    { "name": "fast-component", "delay": 2000 },

    // Good: Only wait when needed
    { "name": "slow-component", "waitFor": ".loaded", "delay": 100 }
  ]
}
```

Use `waitFor` selectors instead of fixed delays when possible.

## Baseline Management

### Git LFS

For projects with many baselines, Git LFS improves clone times:

```bash
# Without LFS: Downloads all baseline history
git clone repo  # 500MB+ for large projects

# With LFS: Downloads only current baselines
git clone repo  # Much faster initial clone
```

Setup:
```bash
git lfs install
git lfs track ".pixelguard/baseline/*.png"
```

### Baseline Cleanup

Periodically remove unused baselines:

```bash
# List baselines without corresponding shots
diff <(ls .pixelguard/baseline/*.png | xargs -n1 basename | sed 's/.png//') \
     <(npx pixelguard list --json | jq -r '.[].name') | grep '^<'
```

### Compression

PNGs can be compressed without quality loss:

```bash
# Using optipng
find .pixelguard/baseline -name "*.png" -exec optipng -o7 {} \;

# Using pngquant (lossy but smaller)
find .pixelguard/baseline -name "*.png" -exec pngquant --force --ext .png {} \;
```

## Monitoring Performance

### Timing Individual Phases

Add timing to your CI:

```yaml
- name: Capture screenshots
  run: |
    START=$(date +%s)
    npx pixelguard test --ci 2>&1 | tee output.log
    END=$(date +%s)
    echo "Capture time: $((END-START)) seconds"

- name: Report timing
  run: |
    grep -E "(Capturing|Comparing|Report)" output.log
```

### Tracking Trends

Store metrics over time:

```yaml
- name: Record metrics
  run: |
    echo "$(date -I),$(wc -l < shot-list.txt),$(grep 'real' timing.txt | awk '{print $2}')" >> metrics.csv
    git add metrics.csv
    git commit -m "chore: update perf metrics" || true
```

## Performance Checklist

### Before Testing

- [ ] Dev server is running and responsive
- [ ] Storybook/app is fully loaded
- [ ] No unnecessary animations or transitions

### Configuration

- [ ] Concurrency matches available resources
- [ ] Viewport size is appropriate (not oversized)
- [ ] Include/exclude patterns filter unnecessary shots
- [ ] Delays are minimal and necessary

### CI Setup

- [ ] Dependencies are cached
- [ ] Playwright browsers are cached
- [ ] Using pre-built Storybook
- [ ] Appropriate runner size selected

### Baseline Management

- [ ] Git LFS configured for large projects
- [ ] Unused baselines cleaned up
- [ ] Images optimized (if applicable)

## Benchmarks

Typical performance numbers (your results may vary):

| Shots | Concurrency | Capture Time | Diff Time | Total |
|-------|-------------|--------------|-----------|-------|
| 10 | 4 | ~5s | ~1s | ~6s |
| 50 | 4 | ~20s | ~5s | ~25s |
| 100 | 4 | ~40s | ~10s | ~50s |
| 100 | 8 | ~25s | ~10s | ~35s |
| 500 | 8 | ~120s | ~50s | ~170s |

Factors that increase time:
- Complex pages with many resources
- Pages with animations (need delays)
- Slow dev server
- Low concurrency
- Large viewport sizes
