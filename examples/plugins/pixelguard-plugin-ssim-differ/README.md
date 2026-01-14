# pixelguard-plugin-ssim-differ

A Pixelguard differ plugin that uses SSIM (Structural Similarity Index) for perceptually-aware image comparison.

## Why Use SSIM?

Traditional pixel-by-pixel comparison can be overly sensitive to:

- **Anti-aliasing differences** between browsers/OS
- **Sub-pixel rendering** variations
- **Compression artifacts** in images
- **Minor color shifts** that aren't visible to humans

SSIM measures perceived visual similarity, which means:

- Fewer false positives from rendering differences
- Better matches human visual perception
- More meaningful diff percentages

## Installation

```bash
npm install pixelguard-plugin-ssim-differ sharp ssim.js
```

## Configuration

Add the plugin to your `pixelguard.config.json`:

```json
{
  "source": "storybook",
  "baseUrl": "http://localhost:6006",
  "plugins": ["pixelguard-plugin-ssim-differ"],
  "pluginOptions": {
    "pixelguard-plugin-ssim-differ": {
      "windowSize": 11,
      "k1": 0.01,
      "k2": 0.03,
      "highlightThreshold": 0.95
    }
  }
}
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `windowSize` | `number` | `11` | Size of the sliding window for comparison |
| `k1` | `number` | `0.01` | SSIM constant for luminance stability |
| `k2` | `number` | `0.03` | SSIM constant for contrast stability |
| `highlightThreshold` | `number` | `0.95` | SSIM threshold below which pixels are highlighted in diff |

## How SSIM Works

SSIM compares images based on three components:

1. **Luminance** (l) - Average brightness comparison
2. **Contrast** (c) - Standard deviation comparison
3. **Structure** (s) - Correlation of normalized pixels

The final SSIM score combines these:

```
SSIM(x, y) = l(x, y) * c(x, y) * s(x, y)
```

- **SSIM = 1.0**: Images are identical
- **SSIM = 0.0**: Images are completely different
- **SSIM > 0.95**: Usually imperceptible differences

## Diff Image Output

The plugin generates diff images where:

- **Red-tinted areas**: Regions with low SSIM (visible differences)
- **Dimmed areas**: Regions with high SSIM (unchanged)
- **Intensity**: Proportional to the severity of difference

## Comparison: SSIM vs Pixel Diff

| Scenario | Pixel Diff | SSIM |
|----------|-----------|------|
| Font anti-aliasing changed | High diff % | Low diff % |
| Minor color shift (#fff → #fffe) | Medium diff % | Very low diff % |
| Image compressed differently | High diff % | Low-medium diff % |
| Actual content change | High diff % | High diff % |
| Layout shift | High diff % | High diff % |

## Threshold Recommendations

| Use Case | Recommended Threshold |
|----------|----------------------|
| Strict (catch everything) | 0.1 |
| Normal (balance) | 1.0 - 2.0 |
| Lenient (only major changes) | 5.0 |

Remember: With SSIM, thresholds work differently than pixel diff:
- A 1% SSIM diff is more significant than a 1% pixel diff
- SSIM tends to give lower percentages overall

## Plugin Development

This plugin demonstrates:

- **Differ plugin interface**: Implementing the `compare` hook
- **Image processing**: Using Sharp for image manipulation
- **SSIM algorithm**: Using ssim.js library
- **Diff visualization**: Creating visual diff outputs

### Key Implementation

```javascript
const { ssim } = require('ssim.js');
const sharp = require('sharp');

async function compare(input) {
  const { baselinePath, currentPath, diffPath, threshold, options } = input;

  // Load images as raw pixel data
  const baseline = await loadImage(baselinePath);
  const current = await loadImage(currentPath);

  // Run SSIM comparison
  const result = ssim(
    { data: baseline.data, width: baseline.width, height: baseline.height },
    { data: current.data, width: current.width, height: current.height },
    { windowSize: options.windowSize || 11 }
  );

  // SSIM returns 0-1 (1 = identical), convert to percentage
  const diffPercentage = (1 - result.mssim) * 100;
  const matches = diffPercentage <= threshold;

  // Generate diff image if different
  if (!matches) {
    await generateDiffImage(baseline, current, result.ssim_map, diffPath);
  }

  return { diffPercentage, matches };
}
```

## Performance

SSIM is computationally more expensive than pixel comparison:

| Image Size | Pixel Diff | SSIM |
|------------|-----------|------|
| 1280x720 | ~50ms | ~200ms |
| 1920x1080 | ~100ms | ~400ms |
| 4K | ~200ms | ~1000ms |

For large test suites, consider:

- Running tests in parallel
- Using smaller viewports when possible
- Caching baseline loads

## Troubleshooting

### "Cannot find module 'sharp'"

```bash
npm install sharp
```

Sharp requires native binaries - if installation fails:

```bash
npm rebuild sharp
```

### "ssim is not a function"

Ensure correct import:

```javascript
const { ssim } = require('ssim.js');
```

### Diff images look strange

Adjust `highlightThreshold`:

- Lower value (0.90): More aggressive highlighting
- Higher value (0.98): Only highlight major differences

## References

- [SSIM Wikipedia](https://en.wikipedia.org/wiki/Structural_similarity)
- [ssim.js Documentation](https://github.com/obartra/ssim)
- [Sharp Documentation](https://sharp.pixelplumbing.com/)

## License

MIT
