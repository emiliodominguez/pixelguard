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
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba, RgbaImage};
use tracing::{debug, info};

use crate::capture::{CaptureResult, FailedShot};
use crate::config::Config;
use crate::plugins::{
    executor, DifferInput, DifferOutput, LoadedPlugin, PluginCategory, PluginRegistry,
};

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

    crate::capture::ensure_not_symlink(&diff_dir)?;
    crate::capture::ensure_not_symlink(&baseline_dir)?;
    std::fs::create_dir_all(&diff_dir)?;
    clear_png_files(&diff_dir)?;

    let baseline_shots: HashSet<String> = list_png_stems(&baseline_dir)?.into_iter().collect();
    let differ_plugin = plugin_registry.and_then(|r| r.get(PluginCategory::Differ));

    let mut result = DiffResult {
        failed: capture.failed.clone(),
        ..DiffResult::default()
    };

    for shot in &capture.captured {
        let name = &shot.name;
        if !baseline_shots.contains(name) {
            result.added.push(name.clone());
            continue;
        }

        let baseline_path = baseline_dir.join(format!("{}.png", name));
        let current_path = current_dir.join(format!("{}.png", name));
        let diff_path = diff_dir.join(format!("{}.png", name));

        debug!("🔍 Comparing: {}", name);

        let comparison = match differ_plugin {
            Some(plugin) => compare_with_plugin(
                plugin,
                &baseline_path,
                &current_path,
                &diff_path,
                config.threshold,
                working_dir,
            ),
            None => compare_images(&baseline_path, &current_path, &diff_path, config.threshold),
        };

        let diff_percentage = match comparison.and_then(validate_percentage) {
            Ok(value) => value,
            Err(error) => {
                result.failed.push(FailedShot {
                    name: name.clone(),
                    error: format!("{:#}", error),
                });
                continue;
            }
        };

        if diff_percentage > config.threshold {
            info!("⚠️  {}: {:.2}% different", name, diff_percentage);
            let (_, viewport) = parse_shot_name(name);
            result.changed.push(ChangedShot {
                name: name.clone(),
                baseline_path,
                current_path,
                diff_path,
                diff_percentage,
                viewport: viewport.map(String::from),
            });
        } else {
            debug!("✅ {}: unchanged", name);
            result.unchanged.push(name.clone());
            let _ = std::fs::remove_file(&diff_path);
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

/// Compares two images using a differ plugin.
///
/// Executes the plugin's `compare` hook and returns the diff percentage.
fn compare_with_plugin(
    plugin: &LoadedPlugin,
    baseline_path: &Path,
    current_path: &Path,
    diff_path: &Path,
    threshold: f64,
    working_dir: &Path,
) -> Result<f64> {
    let input = DifferInput {
        baseline_path: baseline_path.to_string_lossy().to_string(),
        current_path: current_path.to_string_lossy().to_string(),
        diff_path: diff_path.to_string_lossy().to_string(),
        threshold,
        options: serde_json::Value::Null,
    };

    let output: DifferOutput = executor::execute_hook(plugin, "compare", &input, working_dir)?;

    Ok(output.diff_percentage)
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

/// Compares two images and generates a diff image.
///
/// Returns the percentage of pixels that differ.
fn compare_images(
    baseline_path: &Path,
    current_path: &Path,
    diff_path: &Path,
    threshold: f64,
) -> Result<f64> {
    let baseline = load_image(baseline_path, "baseline")?;
    let current = load_image(current_path, "current")?;

    // Handle size mismatch
    if baseline.dimensions() != current.dimensions() {
        // Create a diff image showing the size difference
        let (bw, bh) = baseline.dimensions();
        let (cw, ch) = current.dimensions();
        let max_w = bw.max(cw);
        let max_h = bh.max(ch);

        let mut diff_img: RgbaImage = ImageBuffer::new(max_w, max_h);

        // Fill with red to show size mismatch
        for pixel in diff_img.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 255]);
        }

        diff_img.save(diff_path)?;
        return Ok(100.0);
    }

    let (width, height) = baseline.dimensions();
    let total_pixels = u64::from(width) * u64::from(height);
    if total_pixels == 0 {
        anyhow::bail!(
            "Screenshot '{}' is empty (0 pixels)",
            current_path.display()
        );
    }
    let mut diff_count = 0u64;

    let mut diff_img: RgbaImage = ImageBuffer::new(width, height);

    for (x, y, current_pixel) in current.enumerate_pixels() {
        let baseline_pixel = baseline.get_pixel(x, y);

        if pixels_differ(baseline_pixel, current_pixel, threshold) {
            diff_count += 1;

            // Create diff pixel: red overlay on dimmed current
            diff_img.put_pixel(
                x,
                y,
                Rgba([
                    255,
                    ((current_pixel[1] as f32) * 0.3) as u8,
                    ((current_pixel[2] as f32) * 0.3) as u8,
                    255,
                ]),
            );
        } else {
            // Dimmed version of current for context
            diff_img.put_pixel(
                x,
                y,
                Rgba([
                    ((current_pixel[0] as f32) * 0.5) as u8,
                    ((current_pixel[1] as f32) * 0.5) as u8,
                    ((current_pixel[2] as f32) * 0.5) as u8,
                    current_pixel[3],
                ]),
            );
        }
    }

    let diff_percentage = (diff_count as f64 / total_pixels as f64) * 100.0;

    // Only save diff image if there are differences
    if diff_count > 0 {
        diff_img
            .save(diff_path)
            .with_context(|| format!("Failed to save diff image: {}", diff_path.display()))?;
    }

    Ok(diff_percentage)
}

/// Checks if two pixels differ beyond the tolerance threshold.
///
/// Uses color distance calculation with anti-aliasing tolerance.
fn pixels_differ(a: &Rgba<u8>, b: &Rgba<u8>, threshold: f64) -> bool {
    // Quick check for identical pixels
    if a == b {
        return false;
    }

    // Calculate color distance (simple Euclidean distance in RGB space)
    let dr = (a[0] as f64 - b[0] as f64).powi(2);
    let dg = (a[1] as f64 - b[1] as f64).powi(2);
    let db = (a[2] as f64 - b[2] as f64).powi(2);
    let da = (a[3] as f64 - b[3] as f64).powi(2);

    let distance = (dr + dg + db + da).sqrt();

    // Normalize to 0-1 range (max distance is sqrt(4 * 255^2) = 510)
    let normalized = distance / 510.0;

    // Apply anti-aliasing tolerance
    // We use a small threshold per-pixel to account for anti-aliasing differences
    let pixel_threshold = threshold * 0.1;

    normalized > pixel_threshold
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
