---
"pixelguard": minor
"pixelguard-plugin-types": minor
---

More ways to compare screenshots, and a faster report and diff.

Report:

- Six comparison views, switched with `1`–`6`: side by side, a before/after slider, flicker (alternates baseline and current in place; `f` pauses, with three speeds), a heatmap of where pixels changed, the diff image, and Changes, which shows one changed area at a time, baseline and current cropped side by side.
- `n` and `p` step to the next and previous changed area across the whole report, crossing into the next screenshot.
- Each changed screenshot says how many areas changed.
- Views are built only for screenshots near the viewport, and every image reserves its size before loading, so long reports load quickly, use little memory and never jump while scrolling. The report still shows side-by-side images without JavaScript.

Diffing:

- Screenshots are compared in parallel, up to `concurrency` at a time, and diff images use fast PNG compression. Unchanged screenshots no longer encode a diff image at all.
- A heatmap (`heatmap/<name>.png`) is written for each changed screenshot of unchanged size. Add `.pixelguard/heatmap/` to `.gitignore` and to CI artifacts next to `diff/`.

Output:

- `results.json` and reporter plugin input list `heatmapPath` and `regions` (changed areas in image pixels, in reading order) for each changed shot.
- `pixelguard review` can open the heatmap and shows how many areas changed.
