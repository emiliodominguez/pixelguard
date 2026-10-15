//! Image diffing algorithm for visual regression testing.
//!
//! This module provides pixel-by-pixel comparison of images with anti-aliasing
//! tolerance and generates visual diff images. Supports custom differ plugins
//! for alternative algorithms like SSIM.
//!
//! Comparison is fail-safe: anything that cannot be compared (unreadable
//! images, a broken differ plugin, a nonsensical percentage) is reported as a
//! failure instead of silently counting as unchanged.

use std::collections::HashSet;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::capture::{CaptureResult, FailedShot};
use crate::config::Config;
use crate::plugins::{
    executor, DifferInput, DifferOutput, LoadedPlugin, PluginCategory, PluginRegistry,
};

/// Side, in pixels, of the square cells that the heatmap and the change
/// regions are measured in. Each cell becomes one pixel of the heatmap image.
pub(crate) const CELL: u32 = 8;

/// Most change regions reported for one screenshot. Nearby changes are merged
/// until they fit, so a reviewer steps through meaningful areas, not noise.
const MAX_REGIONS: usize = 64;

/// Result of comparing images.
#[derive(Debug, Clone, Default)]
pub struct DiffResult {
    /// Shots that are unchanged from baseline
    pub unchanged: Vec<String>,
    /// Shots that have visual differences
    pub changed: Vec<ChangedShot>,
    /// Shots that exist in current but not in baseline (new)
    pub added: Vec<String>,
    /// Shots that exist in baseline but not in current (removed)
    pub removed: Vec<String>,
    /// Shots that could not be captured or compared
    pub failed: Vec<FailedShot>,
}

impl DiffResult {
    /// Returns true if there are any changes (changed, added, or removed).
    pub fn has_changes(&self) -> bool {
        !self.changed.is_empty() || !self.added.is_empty() || !self.removed.is_empty()
    }

    /// Returns true if any shot could not be captured or compared.
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }

    /// Returns true only when every expected shot was captured, compared and unchanged.
    pub fn passed(&self) -> bool {
        !self.has_changes() && !self.has_failures()
    }
}

/// A shot with visual differences.
#[derive(Debug, Clone)]
pub struct ChangedShot {
    /// Name of the shot (includes viewport suffix if multi-viewport, e.g., "button@mobile")
    pub name: String,
    /// Path to baseline image
    pub baseline_path: PathBuf,
    /// Path to current image
    pub current_path: PathBuf,
    /// Path to diff image
    pub diff_path: PathBuf,
    /// Percentage of pixels that differ (0.0 to 100.0)
    pub diff_percentage: f64,
    /// Viewport name if multi-viewport (e.g., "mobile"), None for default viewport
    pub viewport: Option<String>,
    /// Path to the heatmap image, when one could be drawn (both screenshots the same size)
    pub heatmap_path: Option<PathBuf>,
    /// Areas that changed, in reading order, for stepping from one difference to the next
    pub regions: Vec<Region>,
}

/// A rectangle of a screenshot, in image pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    /// Left edge
    pub x: u32,
    /// Top edge
    pub y: u32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

/// Parses a shot name into base name and viewport.
///
/// For multi-viewport shots named `shot@viewport`, returns `(shot, Some(viewport))`.
/// For single-viewport shots named `shot`, returns `(shot, None)`.
pub fn parse_shot_name(name: &str) -> (&str, Option<&str>) {
    if let Some(at_pos) = name.rfind('@') {
        let base = &name[..at_pos];
        let viewport = &name[at_pos + 1..];
        (base, Some(viewport))
    } else {
        (name, None)
    }
}

/// Compares every screenshot in `current/` against the baseline.
///
/// This scans the directories, so whatever is on disk is compared. Prefer
/// [`diff_captured`] after a capture run: it only trusts what that run produced.
///
/// # Example
///
/// ```rust,no_run
/// use pixelguard_core::{Config, diff_images};
///
/// fn example() -> anyhow::Result<()> {
///     let config = Config::load("pixelguard.config.json")?;
///     let result = diff_images(&config, ".", None)?;
///
///     println!("Unchanged: {}", result.unchanged.len());
///     println!("Changed: {}", result.changed.len());
///     Ok(())
/// }
/// ```
pub fn diff_images<P: AsRef<Path>>(
    config: &Config,
    working_dir: P,
    plugin_registry: Option<&PluginRegistry>,
) -> Result<DiffResult> {
    let working_dir = working_dir.as_ref();
    let current_dir = working_dir.join(&config.output_dir).join("current");
    let current = list_png_stems(&current_dir)?;

    let capture = CaptureResult {
        captured: current
            .iter()
            .map(|name| crate::capture::CapturedShot {
                name: name.clone(),
                path: current_dir.join(format!("{}.png", name)),
            })
            .collect(),
        failed: Vec::new(),
    };
    let known: HashSet<String> = current.into_iter().collect();

    diff_captured(
        config,
        working_dir,
        plugin_registry,
        &capture,
        &known,
        |_| true,
    )
}

/// Compares the screenshots produced by one capture run against the baseline.
///
/// * Captured shots with a baseline are compared; without one they are `added`.
/// * Failed captures are carried into `failed`.
/// * A baseline is `removed` when `in_scope` accepts its name and `known` (every
///   screenshot name the unfiltered config produces) does not contain it.
///
/// # Errors
///
/// Returns an error only for I/O problems with the output directories;
/// per-shot problems are reported in [`DiffResult::failed`].
pub fn diff_captured<P, F>(
    config: &Config,
    working_dir: P,
    plugin_registry: Option<&PluginRegistry>,
    capture: &CaptureResult,
    known: &HashSet<String>,
    in_scope: F,
) -> Result<DiffResult>
where
    P: AsRef<Path>,
    F: Fn(&str) -> bool,
{
    let working_dir = working_dir.as_ref();
    let baseline_dir = working_dir.join(&config.output_dir).join("baseline");
    let current_dir = working_dir.join(&config.output_dir).join("current");
    let diff_dir = working_dir.join(&config.output_dir).join("diff");
    let heatmap_dir = working_dir.join(&config.output_dir).join("heatmap");

    crate::capture::ensure_not_symlink(&diff_dir)?;
    crate::capture::ensure_not_symlink(&heatmap_dir)?;
    crate::capture::ensure_not_symlink(&baseline_dir)?;
    std::fs::create_dir_all(&diff_dir)?;
    std::fs::create_dir_all(&heatmap_dir)?;
    clear_png_files(&diff_dir)?;
    clear_png_files(&heatmap_dir)?;

    let baseline_shots: HashSet<String> = list_png_stems(&baseline_dir)?.into_iter().collect();
    let differ_plugin = plugin_registry.and_then(|r| r.get(PluginCategory::Differ));

    let mut result = DiffResult {
        failed: capture.failed.clone(),
        ..DiffResult::default()
    };

    let mut jobs = Vec::new();
    for shot in &capture.captured {
        let name = &shot.name;
        if !baseline_shots.contains(name) {
            result.added.push(name.clone());
            continue;
        }
        jobs.push(Job {
            name: name.clone(),
            baseline: baseline_dir.join(format!("{}.png", name)),
            current: current_dir.join(format!("{}.png", name)),
            diff: diff_dir.join(format!("{}.png", name)),
            heatmap: heatmap_dir.join(format!("{}.png", name)),
        });
    }

    // Built-in comparisons are independent and CPU-bound, so they run in
    // parallel. Differ plugins are external processes that may share state on
    // disk, so they run one at a time.
    let comparisons: Vec<Result<Comparison>> = match differ_plugin {
        Some(plugin) => jobs
            .iter()
            .map(|job| compare_with_plugin(plugin, job, config.threshold, working_dir))
            .collect(),
        None => crate::parallel::map(&jobs, config.concurrency, |job| {
            compare_images(job, config.threshold)
        }),
    };

    for (job, comparison) in jobs.into_iter().zip(comparisons) {
        let comparison = match comparison {
            Ok(value) => value,
            Err(error) => {
                result.failed.push(FailedShot {
                    name: job.name,
                    error: format!("{:#}", error),
                });
                continue;
            }
        };

        if comparison.diff_percentage > config.threshold {
            info!(
                "⚠️  {}: {:.2}% different",
                job.name, comparison.diff_percentage
            );
            let viewport = parse_shot_name(&job.name).1.map(String::from);
            result.changed.push(ChangedShot {
                name: job.name,
                baseline_path: job.baseline,
                current_path: job.current,
                diff_path: job.diff,
                diff_percentage: comparison.diff_percentage,
                viewport,
                heatmap_path: comparison.heatmap.then_some(job.heatmap),
                regions: comparison.regions,
            });
        } else {
            debug!("✅ {}: unchanged", job.name);
            // A differ plugin may have written a diff image anyway.
            let _ = std::fs::remove_file(&job.diff);
            result.unchanged.push(job.name);
        }
    }

    let attempted: HashSet<&str> = capture
        .captured
        .iter()
        .map(|c| c.name.as_str())
        .chain(capture.failed.iter().map(|f| f.name.as_str()))
        .collect();
    result.removed = baseline_shots
        .into_iter()
        .filter(|name| {
            !known.contains(name) && !attempted.contains(name.as_str()) && in_scope(name)
        })
        .collect();

    result.unchanged.sort();
    result.added.sort();
    result.removed.sort();
    result.changed.sort_by(|a, b| a.name.cmp(&b.name));
    result.failed.sort_by(|a, b| a.name.cmp(&b.name));

    info!(
        "📊 Diff complete: {} unchanged, {} changed, {} added, {} removed, {} failed",
        result.unchanged.len(),
        result.changed.len(),
        result.added.len(),
        result.removed.len(),
        result.failed.len()
    );

    Ok(result)
}

/// Rejects percentages that are not a finite number in `0..=100`.
fn validate_percentage(value: f64) -> Result<f64> {
    if value.is_finite() && (0.0..=100.0).contains(&value) {
        Ok(value)
    } else {
        anyhow::bail!(
            "Differ returned an invalid diff percentage ({}); expected a number between 0 and 100",
            value
        )
    }
}

/// Lists the stems of PNG files in `dir`, or nothing if it does not exist.
fn list_png_stems(dir: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    if !dir.exists() {
        return Ok(names);
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "png") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Removes diff images from a previous run so the report never shows stale ones.
fn clear_png_files(dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "png") {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// One screenshot to compare, with the files it reads and writes.
struct Job {
    name: String,
    baseline: PathBuf,
    current: PathBuf,
    diff: PathBuf,
    heatmap: PathBuf,
}

/// Outcome of comparing one screenshot.
struct Comparison {
    /// Percentage of pixels that differ (0.0 to 100.0)
    diff_percentage: f64,
    /// Where it changed; empty when unchanged
    regions: Vec<Region>,
    /// Whether a heatmap image was written
    heatmap: bool,
}

/// Compares two images using a differ plugin.
///
/// The plugin decides the percentage and writes the diff image. When it
/// reports a change, the built-in comparison still draws the heatmap and finds
/// the change regions, so every report view works whichever differ ran; if
/// that fails, the shot is still reported, without those views.
fn compare_with_plugin(
    plugin: &LoadedPlugin,
    job: &Job,
    threshold: f64,
    working_dir: &Path,
) -> Result<Comparison> {
    debug!("🔍 Comparing with plugin: {}", job.name);
    let input = DifferInput {
        baseline_path: job.baseline.to_string_lossy().to_string(),
        current_path: job.current.to_string_lossy().to_string(),
        diff_path: job.diff.to_string_lossy().to_string(),
        threshold,
        options: serde_json::Value::Null,
    };

    let output: DifferOutput = executor::execute_hook(plugin, "compare", &input, working_dir)?;
    let diff_percentage = validate_percentage(output.diff_percentage)?;
    if diff_percentage <= threshold {
        return Ok(Comparison {
            diff_percentage,
            regions: Vec::new(),
            heatmap: false,
        });
    }

    let (regions, heatmap) = match measure_files(job, threshold)
        .and_then(|measurement| measurement.write_heatmap(&job.heatmap))
    {
        Ok(visuals) => visuals,
        Err(error) => {
            warn!(
                "⚠️  {}: no heatmap or change regions: {:#}",
                job.name, error
            );
            (Vec::new(), false)
        }
    };

    Ok(Comparison {
        diff_percentage,
        regions,
        heatmap,
    })
}

/// Loads an image, explaining the common Git LFS pointer-file failure.
fn load_image(path: &Path, label: &str) -> Result<RgbaImage> {
    match image::open(path) {
        Ok(image) => Ok(image.to_rgba8()),
        Err(error) => {
            let is_lfs_pointer = std::fs::read(path)
                .map(|bytes| bytes.starts_with(b"version https://git-lfs"))
                .unwrap_or(false);
            if is_lfs_pointer {
                anyhow::bail!(
                    "{} image '{}' is a Git LFS pointer, not a PNG. Run 'git lfs pull' \
                     (or enable LFS in your CI checkout) before running pixelguard.",
                    label,
                    path.display()
                );
            }
            Err(error)
                .with_context(|| format!("Failed to load {} image: {}", label, path.display()))
        }
    }
}

/// Compares two images with the built-in pixel comparison.
///
/// The diff image, heatmap and change regions are only produced for shots
/// that changed beyond `threshold`; encoding images nobody will look at is
/// most of the cost of comparing an unchanged screenshot.
fn compare_images(job: &Job, threshold: f64) -> Result<Comparison> {
    debug!("🔍 Comparing: {}", job.name);
    let measurement = measure_files(job, threshold)?;
    let diff_percentage = validate_percentage(measurement.diff_percentage)?;
    if diff_percentage <= threshold {
        return Ok(Comparison {
            diff_percentage,
            regions: Vec::new(),
            heatmap: false,
        });
    }

    let (width, height) = measurement.diff_image.dimensions();
    save_png(&job.diff, measurement.diff_image.as_raw(), width, height)
        .with_context(|| format!("Failed to save diff image: {}", job.diff.display()))?;
    let (regions, heatmap) = measurement.write_heatmap(&job.heatmap)?;

    Ok(Comparison {
        diff_percentage,
        regions,
        heatmap,
    })
}

/// Pixel-level measurement of a baseline against the current screenshot.
struct Measurement {
    diff_percentage: f64,
    /// Changed pixels in red over a dimmed copy of the current screenshot
    diff_image: RgbaImage,
    /// Changed pixels per cell; `None` when the screenshots differ in size
    grid: Option<ChangeGrid>,
}

impl Measurement {
    /// Writes the heatmap image and returns the change regions, with whether a
    /// heatmap was written. Screenshots of different sizes have no meaningful
    /// heatmap; their one region is the whole canvas.
    fn write_heatmap(&self, path: &Path) -> Result<(Vec<Region>, bool)> {
        let Some(grid) = &self.grid else {
            let (width, height) = self.diff_image.dimensions();
            return Ok((
                vec![Region {
                    x: 0,
                    y: 0,
                    width,
                    height,
                }],
                false,
            ));
        };
        let heat = grid.heat_pixels();
        save_png(path, &heat, grid.columns, grid.rows)
            .with_context(|| format!("Failed to save heatmap: {}", path.display()))?;
        Ok((grid.regions(), true))
    }
}

/// Loads both screenshots of a job and measures how they differ.
fn measure_files(job: &Job, threshold: f64) -> Result<Measurement> {
    let baseline = load_image(&job.baseline, "baseline")?;
    let current = load_image(&job.current, "current")?;
    measure(&baseline, &current, threshold)
        .with_context(|| format!("Could not compare '{}'", job.current.display()))
}

/// Measures how `current` differs from `baseline`.
fn measure(baseline: &RgbaImage, current: &RgbaImage, threshold: f64) -> Result<Measurement> {
    if baseline.dimensions() != current.dimensions() {
        // Every pixel counts as changed; the diff image shows the larger canvas in red.
        let (bw, bh) = baseline.dimensions();
        let (cw, ch) = current.dimensions();
        let diff_image = RgbaImage::from_pixel(bw.max(cw), bh.max(ch), Rgba([255, 0, 0, 255]));
        return Ok(Measurement {
            diff_percentage: 100.0,
            diff_image,
            grid: None,
        });
    }

    let (width, height) = current.dimensions();
    let total_pixels = u64::from(width) * u64::from(height);
    if total_pixels == 0 {
        anyhow::bail!("The screenshot is empty (0 pixels)");
    }

    let limit = squared_limit(threshold);
    let mut grid = ChangeGrid::new(width, height);
    let mut diff = vec![0u8; current.as_raw().len()];
    let mut diff_count = 0u64;
    let row_bytes = width as usize * 4;

    for (y, ((before_row, after_row), out_row)) in baseline
        .as_raw()
        .chunks_exact(row_bytes)
        .zip(current.as_raw().chunks_exact(row_bytes))
        .zip(diff.chunks_exact_mut(row_bytes))
        .enumerate()
    {
        let cell_row = (y / CELL as usize) * grid.columns as usize;
        for (x, ((before, after), out)) in before_row
            .chunks_exact(4)
            .zip(after_row.chunks_exact(4))
            .zip(out_row.chunks_exact_mut(4))
            .enumerate()
        {
            if before != after && squared_distance(before, after) > limit {
                diff_count += 1;
                grid.counts[cell_row + x / CELL as usize] += 1;
                // Red over the current pixel's dimmed green and blue
                out[0] = 255;
                out[1] = (u16::from(after[1]) * 3 / 10) as u8;
                out[2] = (u16::from(after[2]) * 3 / 10) as u8;
                out[3] = 255;
            } else {
                // Dimmed current for context
                out[0] = after[0] / 2;
                out[1] = after[1] / 2;
                out[2] = after[2] / 2;
                out[3] = after[3];
            }
        }
    }

    let diff_image = RgbaImage::from_raw(width, height, diff)
        .context("Diff buffer does not match the screenshot size")?;
    Ok(Measurement {
        diff_percentage: (diff_count as f64 / total_pixels as f64) * 100.0,
        diff_image,
        grid: Some(grid),
    })
}

/// Squared RGBA distance above which two pixels count as different.
///
/// A pixel differs when its Euclidean RGBA distance, normalized to 0..1 by
/// the largest possible distance (510), exceeds a tenth of the threshold: a
/// small per-pixel tolerance for anti-aliasing. Comparing squared distances
/// gives the same answer without a square root per pixel.
fn squared_limit(threshold: f64) -> f64 {
    let limit = threshold * 0.1 * 510.0;
    limit * limit
}

/// Squared Euclidean distance between two RGBA pixels.
fn squared_distance(a: &[u8], b: &[u8]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| {
            let d = i32::from(*x) - i32::from(*y);
            (d * d) as u32
        })
        .sum::<u32>() as f64
}

/// Checks if two pixels differ beyond the tolerance threshold.
///
/// Uses color distance calculation with anti-aliasing tolerance.
#[cfg(test)]
fn pixels_differ(a: &Rgba<u8>, b: &Rgba<u8>, threshold: f64) -> bool {
    a != b && squared_distance(&a.0, &b.0) > squared_limit(threshold)
}

/// Writes RGBA pixels as a PNG, favoring speed over size: these images are
/// regenerated on every run and never committed.
fn save_png(path: &Path, rgba: &[u8], width: u32, height: u32) -> Result<()> {
    let file =
        File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    PngEncoder::new_with_quality(
        BufWriter::new(file),
        CompressionType::Fast,
        FilterType::Adaptive,
    )
    .write_image(rgba, width, height, ExtendedColorType::Rgba8)
    .with_context(|| format!("Failed to encode {}", path.display()))
}

/// Changed pixels counted per `CELL`×`CELL` cell of a screenshot.
struct ChangeGrid {
    /// Screenshot width in pixels
    width: u32,
    /// Screenshot height in pixels
    height: u32,
    columns: u32,
    rows: u32,
    /// Row-major counts, `columns * rows` long
    counts: Vec<u32>,
}

impl ChangeGrid {
    fn new(width: u32, height: u32) -> Self {
        let columns = width.div_ceil(CELL);
        let rows = height.div_ceil(CELL);
        Self {
            width,
            height,
            columns,
            rows,
            counts: vec![0; columns as usize * rows as usize],
        }
    }

    /// Pixels in a cell; cells on the right and bottom edges can be smaller.
    fn cell_area(&self, column: u32, row: u32) -> u32 {
        let w = CELL.min(self.width - column * CELL);
        let h = CELL.min(self.height - row * CELL);
        w * h
    }

    /// RGBA pixels of the heatmap, one per cell: transparent where nothing
    /// changed, then amber to crimson, more opaque, as more of the cell changed.
    /// The report scales it up smoothly over the screenshot, which turns the
    /// cells into soft glows.
    fn heat_pixels(&self) -> Vec<u8> {
        const COOL: [f32; 3] = [255.0, 196.0, 0.0];
        const HOT: [f32; 3] = [225.0, 29.0, 72.0];

        let mut pixels = vec![0u8; self.counts.len() * 4];
        for (index, (count, pixel)) in self
            .counts
            .iter()
            .zip(pixels.chunks_exact_mut(4))
            .enumerate()
        {
            if *count == 0 {
                continue;
            }
            let column = index as u32 % self.columns;
            let row = index as u32 / self.columns;
            // Square root so a few changed pixels still show clearly.
            let share = (*count as f32 / self.cell_area(column, row) as f32).sqrt();
            for channel in 0..3 {
                pixel[channel] = (COOL[channel] + (HOT[channel] - COOL[channel]) * share) as u8;
            }
            pixel[3] = (120.0 + 135.0 * share) as u8;
        }
        pixels
    }

    /// Groups changed cells into rectangles in reading order.
    ///
    /// Changed cells separated by at most `2 × reach` unchanged cells belong
    /// to the same region. `reach` starts at two cells (up to 32px gaps) and
    /// doubles while there are more than `MAX_REGIONS`, merging scattered
    /// changes into fewer, larger areas.
    fn regions(&self) -> Vec<Region> {
        let mut reach = 2;
        loop {
            let regions = self.regions_within(reach);
            if regions.len() <= MAX_REGIONS || reach >= self.columns.max(self.rows) as usize {
                return regions;
            }
            reach *= 2;
        }
    }

    fn regions_within(&self, reach: usize) -> Vec<Region> {
        let columns = self.columns as usize;
        let rows = self.rows as usize;
        let changed: Vec<bool> = self.counts.iter().map(|count| *count > 0).collect();
        let grown = dilate(&changed, columns, rows, reach);

        // Label 8-connected areas of the grown mask; each original changed cell
        // widens the bounds of the area it falls in.
        let mut labels = vec![usize::MAX; grown.len()];
        let mut bounds: Vec<[usize; 4]> = Vec::new();
        let mut stack = Vec::new();
        for start in 0..grown.len() {
            if !changed[start] || labels[start] != usize::MAX {
                continue;
            }
            let label = bounds.len();
            bounds.push([usize::MAX, usize::MAX, 0, 0]);
            labels[start] = label;
            stack.push(start);
            while let Some(index) = stack.pop() {
                let (column, row) = (index % columns, index / columns);
                if changed[index] {
                    let b = &mut bounds[label];
                    b[0] = b[0].min(column);
                    b[1] = b[1].min(row);
                    b[2] = b[2].max(column);
                    b[3] = b[3].max(row);
                }
                for dy in -1i64..=1 {
                    for dx in -1i64..=1 {
                        let (c, r) = (column as i64 + dx, row as i64 + dy);
                        if c < 0 || r < 0 || c >= columns as i64 || r >= rows as i64 {
                            continue;
                        }
                        let neighbor = r as usize * columns + c as usize;
                        if grown[neighbor] && labels[neighbor] == usize::MAX {
                            labels[neighbor] = label;
                            stack.push(neighbor);
                        }
                    }
                }
            }
        }

        let mut regions: Vec<Region> = bounds
            .into_iter()
            .map(|[left, top, right, bottom]| {
                let x = left as u32 * CELL;
                let y = top as u32 * CELL;
                Region {
                    x,
                    y,
                    width: ((right as u32 + 1) * CELL).min(self.width) - x,
                    height: ((bottom as u32 + 1) * CELL).min(self.height) - y,
                }
            })
            .collect();
        regions.sort_by_key(|region| (region.y, region.x));
        regions
    }
}

/// Grows every set cell of a `columns`×`rows` mask by `reach` cells in each
/// direction (a square dilation), in linear time using running sums.
fn dilate(mask: &[bool], columns: usize, rows: usize, reach: usize) -> Vec<bool> {
    let mut horizontal = vec![false; mask.len()];
    let mut sums = vec![0u32; columns.max(rows) + 1];
    for row in 0..rows {
        for column in 0..columns {
            sums[column + 1] = sums[column] + u32::from(mask[row * columns + column]);
        }
        for column in 0..columns {
            let from = column.saturating_sub(reach);
            let to = (column + reach + 1).min(columns);
            horizontal[row * columns + column] = sums[to] > sums[from];
        }
    }

    let mut grown = vec![false; mask.len()];
    for column in 0..columns {
        for row in 0..rows {
            sums[row + 1] = sums[row] + u32::from(horizontal[row * columns + column]);
        }
        for row in 0..rows {
            let from = row.saturating_sub(reach);
            let to = (row + reach + 1).min(rows);
            grown[row * columns + column] = sums[to] > sums[from];
        }
    }
    grown
}

/// Checks if a baseline exists for comparison.
pub fn has_baseline<P: AsRef<Path>>(config: &Config, working_dir: P) -> bool {
    let baseline_dir = working_dir
        .as_ref()
        .join(&config.output_dir)
        .join("baseline");
    if !baseline_dir.exists() {
        return false;
    }

    // Check if there are any PNG files
    if let Ok(entries) = std::fs::read_dir(&baseline_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|ext| ext == "png") {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    #[test]
    fn squared_comparison_matches_the_normalized_distance_rule() {
        let samples = [
            ([10, 20, 30, 255], [10, 20, 30, 255]),
            ([100, 100, 100, 255], [102, 101, 99, 255]),
            ([0, 0, 0, 255], [3, 0, 0, 255]),
            ([0, 0, 0, 0], [255, 255, 255, 255]),
            ([200, 10, 90, 128], [190, 30, 80, 140]),
        ];
        for threshold in [0.0, 0.01, 0.1, 0.5, 1.0] {
            for (a, b) in samples {
                let distance = a
                    .iter()
                    .zip(b)
                    .map(|(x, y)| (*x as f64 - y as f64).powi(2))
                    .sum::<f64>()
                    .sqrt();
                let expected = a != b && distance / 510.0 > threshold * 0.1;
                assert_eq!(
                    pixels_differ(&Rgba(a), &Rgba(b), threshold),
                    expected,
                    "{:?} vs {:?} at {}",
                    a,
                    b,
                    threshold
                );
            }
        }
    }

    /// A white 200×120 image with black boxes at the given (x, y, w, h).
    fn with_boxes(boxes: &[(u32, u32, u32, u32)]) -> RgbaImage {
        let mut image = RgbaImage::from_pixel(200, 120, Rgba([255, 255, 255, 255]));
        for &(x0, y0, w, h) in boxes {
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                }
            }
        }
        image
    }

    #[test]
    fn separate_changes_become_regions_in_reading_order() {
        let baseline = with_boxes(&[]);
        let current = with_boxes(&[(150, 10, 10, 10), (10, 80, 20, 12), (12, 12, 6, 6)]);

        let measurement = measure(&baseline, &current, 0.1).unwrap();
        let regions = measurement.grid.as_ref().unwrap().regions();

        assert_eq!(regions.len(), 3);
        // Reading order: top to bottom, then left to right.
        assert_eq!((regions[0].x, regions[0].y), (8, 8));
        assert_eq!((regions[1].x, regions[1].y), (144, 8));
        assert_eq!((regions[2].x, regions[2].y), (8, 80));
        // Each region covers its change, rounded out to whole cells.
        assert!(regions[2].width >= 20 && regions[2].height >= 12);
    }

    #[test]
    fn nearby_changes_merge_into_one_region() {
        let baseline = with_boxes(&[]);
        let current = with_boxes(&[(20, 20, 4, 4), (34, 20, 4, 4)]);

        let measurement = measure(&baseline, &current, 0.1).unwrap();

        assert_eq!(measurement.grid.unwrap().regions().len(), 1);
    }

    #[test]
    fn scattered_noise_is_merged_down_to_a_reviewable_number_of_regions() {
        let baseline = RgbaImage::from_pixel(800, 800, Rgba([255, 255, 255, 255]));
        let mut current = baseline.clone();
        // 100 isolated dots, too far apart to merge at the first reach.
        for y in (0..800).step_by(80) {
            for x in (0..800).step_by(80) {
                current.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }

        let measurement = measure(&baseline, &current, 0.1).unwrap();
        let regions = measurement.grid.unwrap().regions();

        assert!(!regions.is_empty());
        assert!(regions.len() <= MAX_REGIONS, "{} regions", regions.len());
    }

    #[test]
    fn heatmap_has_one_pixel_per_cell_and_marks_only_changed_cells() {
        let baseline = with_boxes(&[]);
        let current = with_boxes(&[(0, 0, 8, 8)]);
        let grid = measure(&baseline, &current, 0.1).unwrap().grid.unwrap();

        let heat = grid.heat_pixels();

        assert_eq!((grid.columns, grid.rows), (25, 15));
        assert_eq!(heat.len(), 25 * 15 * 4);
        assert_eq!(heat[3], 255, "a fully changed cell is fully opaque");
        assert_eq!(heat[7], 0, "an unchanged cell is transparent");
    }

    #[test]
    fn changed_shots_get_diff_heatmap_and_regions_unchanged_get_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let baseline = dir.path().join(".pixelguard/baseline");
        let current = dir.path().join(".pixelguard/current");
        std::fs::create_dir_all(&baseline).unwrap();
        std::fs::create_dir_all(&current).unwrap();
        with_boxes(&[]).save(baseline.join("moved.png")).unwrap();
        with_boxes(&[(40, 40, 30, 30)])
            .save(current.join("moved.png"))
            .unwrap();
        with_boxes(&[]).save(baseline.join("same.png")).unwrap();
        with_boxes(&[]).save(current.join("same.png")).unwrap();

        let capture = captured(&current, &["moved", "same"]);
        let known: HashSet<String> = ["moved".to_string(), "same".to_string()]
            .into_iter()
            .collect();
        let result = diff_captured(&config, dir.path(), None, &capture, &known, |_| true).unwrap();

        assert_eq!(result.unchanged, vec!["same".to_string()]);
        let moved = &result.changed[0];
        assert_eq!(moved.regions.len(), 1);
        assert!(moved.diff_path.is_file());
        assert!(moved.heatmap_path.as_ref().unwrap().is_file());
        assert!(!dir.path().join(".pixelguard/diff/same.png").exists());
        assert!(!dir.path().join(".pixelguard/heatmap/same.png").exists());
    }

    #[test]
    fn size_changes_have_one_whole_canvas_region_and_no_heatmap() {
        let baseline = RgbaImage::from_pixel(100, 50, Rgba([255, 255, 255, 255]));
        let current = RgbaImage::from_pixel(100, 80, Rgba([255, 255, 255, 255]));
        let dir = tempfile::tempdir().unwrap();

        let measurement = measure(&baseline, &current, 0.1).unwrap();
        let (regions, heatmap) = measurement
            .write_heatmap(&dir.path().join("heat.png"))
            .unwrap();

        assert_eq!(measurement.diff_percentage, 100.0);
        assert!(!heatmap);
        assert_eq!(
            regions,
            vec![Region {
                x: 0,
                y: 0,
                width: 100,
                height: 80
            }]
        );
    }

    #[test]
    fn identical_pixels_do_not_differ() {
        let a = Rgba([100, 150, 200, 255]);
        let b = Rgba([100, 150, 200, 255]);
        assert!(!pixels_differ(&a, &b, 0.1));
    }

    #[test]
    fn very_different_pixels_differ() {
        let a = Rgba([0, 0, 0, 255]);
        let b = Rgba([255, 255, 255, 255]);
        assert!(pixels_differ(&a, &b, 0.1));
    }

    #[test]
    fn slightly_different_pixels_within_threshold() {
        let a = Rgba([100, 100, 100, 255]);
        let b = Rgba([102, 101, 99, 255]);
        // Small difference should be within anti-aliasing tolerance
        assert!(!pixels_differ(&a, &b, 0.5));
    }

    #[test]
    fn diff_result_starts_empty() {
        let result = DiffResult {
            unchanged: Vec::new(),
            changed: Vec::new(),
            added: Vec::new(),
            removed: Vec::new(),
            failed: Vec::new(),
        };

        assert!(result.unchanged.is_empty());
        assert!(result.changed.is_empty());
        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
    }

    #[test]
    fn has_changes_returns_false_for_empty_result() {
        let result = DiffResult {
            unchanged: vec!["test".to_string()],
            changed: Vec::new(),
            added: Vec::new(),
            removed: Vec::new(),
            failed: Vec::new(),
        };
        assert!(!result.has_changes());
    }

    #[test]
    fn has_changes_returns_true_when_changed() {
        let result = DiffResult {
            unchanged: Vec::new(),
            changed: vec![ChangedShot {
                name: "test".to_string(),
                baseline_path: PathBuf::new(),
                current_path: PathBuf::new(),
                diff_path: PathBuf::new(),
                diff_percentage: 1.0,
                viewport: None,
                heatmap_path: None,
                regions: Vec::new(),
            }],
            added: Vec::new(),
            removed: Vec::new(),
            failed: Vec::new(),
        };
        assert!(result.has_changes());
    }

    #[test]
    fn has_changes_returns_true_when_added() {
        let result = DiffResult {
            unchanged: Vec::new(),
            changed: Vec::new(),
            added: vec!["new".to_string()],
            removed: Vec::new(),
            failed: Vec::new(),
        };
        assert!(result.has_changes());
    }

    #[test]
    fn has_changes_returns_true_when_removed() {
        let result = DiffResult {
            unchanged: Vec::new(),
            changed: Vec::new(),
            added: Vec::new(),
            removed: vec!["old".to_string()],
            failed: Vec::new(),
        };
        assert!(result.has_changes());
    }

    #[test]
    fn has_baseline_returns_false_for_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            output_dir: ".pixelguard".to_string(),
            ..Default::default()
        };

        assert!(!has_baseline(&config, dir.path()));
    }

    #[test]
    fn has_baseline_returns_true_when_images_exist() {
        let dir = tempfile::tempdir().unwrap();
        let baseline_dir = dir.path().join(".pixelguard/baseline");
        std::fs::create_dir_all(&baseline_dir).unwrap();

        // Create a dummy PNG file
        let img: RgbaImage = ImageBuffer::new(10, 10);
        img.save(baseline_dir.join("test.png")).unwrap();

        let config = Config {
            output_dir: ".pixelguard".to_string(),
            ..Default::default()
        };

        assert!(has_baseline(&config, dir.path()));
    }

    fn write_png(path: &Path, color: [u8; 4]) {
        let img: RgbaImage = ImageBuffer::from_pixel(4, 4, Rgba(color));
        img.save(path).unwrap();
    }

    fn captured(dir: &Path, names: &[&str]) -> CaptureResult {
        CaptureResult {
            captured: names
                .iter()
                .map(|name| crate::capture::CapturedShot {
                    name: name.to_string(),
                    path: dir.join(format!("{}.png", name)),
                })
                .collect(),
            failed: Vec::new(),
        }
    }

    #[test]
    fn diff_captured_ignores_stale_files_and_reports_failures() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let baseline = dir.path().join(".pixelguard/baseline");
        let current = dir.path().join(".pixelguard/current");
        std::fs::create_dir_all(&baseline).unwrap();
        std::fs::create_dir_all(&current).unwrap();
        write_png(&baseline.join("a.png"), [0, 0, 0, 255]);
        write_png(&baseline.join("b.png"), [0, 0, 0, 255]);
        write_png(&current.join("a.png"), [0, 0, 0, 255]);
        write_png(&current.join("b.png"), [0, 0, 0, 255]);

        let mut capture = captured(&current, &["a"]);
        capture.failed.push(FailedShot {
            name: "b".to_string(),
            error: "timeout".to_string(),
        });
        let known: HashSet<String> = ["a".to_string(), "b".to_string()].into_iter().collect();

        let result = diff_captured(&config, dir.path(), None, &capture, &known, |_| true).unwrap();

        assert_eq!(result.unchanged, vec!["a".to_string()]);
        assert_eq!(result.failed.len(), 1);
        assert!(result.removed.is_empty());
        assert!(!result.passed());
    }

    #[test]
    fn diff_captured_only_reports_removals_in_scope() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let baseline = dir.path().join(".pixelguard/baseline");
        let current = dir.path().join(".pixelguard/current");
        std::fs::create_dir_all(&baseline).unwrap();
        std::fs::create_dir_all(&current).unwrap();
        write_png(&baseline.join("button.png"), [0, 0, 0, 255]);
        write_png(&baseline.join("card.png"), [0, 0, 0, 255]);
        write_png(&baseline.join("deleted-story.png"), [0, 0, 0, 255]);
        write_png(&current.join("button.png"), [0, 0, 0, 255]);

        let capture = captured(&current, &["button"]);
        let known: HashSet<String> = ["button".to_string(), "card".to_string()]
            .into_iter()
            .collect();

        let filtered = diff_captured(&config, dir.path(), None, &capture, &known, |n| {
            n.contains("button")
        })
        .unwrap();
        assert!(filtered.removed.is_empty());
        assert!(filtered.passed());

        let unfiltered =
            diff_captured(&config, dir.path(), None, &capture, &known, |_| true).unwrap();
        assert_eq!(unfiltered.removed, vec!["deleted-story".to_string()]);
    }

    #[test]
    fn unreadable_baseline_is_a_failure_not_a_pass() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let baseline = dir.path().join(".pixelguard/baseline");
        let current = dir.path().join(".pixelguard/current");
        std::fs::create_dir_all(&baseline).unwrap();
        std::fs::create_dir_all(&current).unwrap();
        std::fs::write(
            baseline.join("a.png"),
            "version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 1\n",
        )
        .unwrap();
        write_png(&current.join("a.png"), [0, 0, 0, 255]);

        let capture = captured(&current, &["a"]);
        let known: HashSet<String> = ["a".to_string()].into_iter().collect();
        let result = diff_captured(&config, dir.path(), None, &capture, &known, |_| true).unwrap();

        assert_eq!(result.failed.len(), 1);
        assert!(result.failed[0].error.contains("Git LFS"));
    }

    #[test]
    fn invalid_plugin_percentages_are_rejected() {
        assert!(validate_percentage(f64::NAN).is_err());
        assert!(validate_percentage(-1.0).is_err());
        assert!(validate_percentage(250.0).is_err());
        assert_eq!(validate_percentage(12.5).unwrap(), 12.5);
    }
}
