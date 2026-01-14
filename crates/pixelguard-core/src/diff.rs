//! Image diffing algorithm for visual regression testing.
//!
//! This module provides pixel-by-pixel comparison of images with anti-aliasing
//! tolerance and generates visual diff images. Supports custom differ plugins
//! for alternative algorithms like SSIM.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba, RgbaImage};
use tracing::{debug, info};

use crate::config::Config;
use crate::plugins::{
    executor, DifferInput, DifferOutput, LoadedPlugin, PluginCategory, PluginRegistry,
};

/// Result of comparing images.
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Shots that are unchanged from baseline
    pub unchanged: Vec<String>,
    /// Shots that have visual differences
    pub changed: Vec<ChangedShot>,
    /// Shots that exist in current but not in baseline (new)
    pub added: Vec<String>,
    /// Shots that exist in baseline but not in current (removed)
    pub removed: Vec<String>,
}

impl DiffResult {
    /// Returns true if there are any changes (changed, added, or removed).
    pub fn has_changes(&self) -> bool {
        !self.changed.is_empty() || !self.added.is_empty() || !self.removed.is_empty()
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
fn parse_shot_name(name: &str) -> (&str, Option<&str>) {
    if let Some(at_pos) = name.rfind('@') {
        let base = &name[..at_pos];
        let viewport = &name[at_pos + 1..];
        (base, Some(viewport))
    } else {
        (name, None)
    }
}

/// Compares current screenshots against baseline.
///
/// This function:
/// 1. Loads baseline and current images
/// 2. Compares them pixel-by-pixel
/// 3. Generates diff images for changed shots
/// 4. Returns categorized results
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
    let baseline_dir = working_dir.join(&config.output_dir).join("baseline");
    let current_dir = working_dir.join(&config.output_dir).join("current");
    let diff_dir = working_dir.join(&config.output_dir).join("diff");

    // Create diff directory
    std::fs::create_dir_all(&diff_dir)?;

    let mut result = DiffResult {
        unchanged: Vec::new(),
        changed: Vec::new(),
        added: Vec::new(),
        removed: Vec::new(),
    };

    // Get all current screenshots
    let mut current_shots: std::collections::HashSet<String> = std::collections::HashSet::new();
    if current_dir.exists() {
        for entry in std::fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "png") {
                if let Some(stem) = path.file_stem() {
                    current_shots.insert(stem.to_string_lossy().to_string());
                }
            }
        }
    }

    // Get all baseline screenshots
    let mut baseline_shots: std::collections::HashSet<String> = std::collections::HashSet::new();
    if baseline_dir.exists() {
        for entry in std::fs::read_dir(&baseline_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "png") {
                if let Some(stem) = path.file_stem() {
                    baseline_shots.insert(stem.to_string_lossy().to_string());
                }
            }
        }
    }

    // Find added shots (in current but not baseline)
    result
        .added
        .extend(current_shots.difference(&baseline_shots).cloned());

    // Find removed shots (in baseline but not current)
    result
        .removed
        .extend(baseline_shots.difference(&current_shots).cloned());

    // Get differ plugin if available
    let differ_plugin = plugin_registry.and_then(|r| r.get(PluginCategory::Differ));

    // Compare shots that exist in both
    for name in current_shots.intersection(&baseline_shots) {
        let baseline_path = baseline_dir.join(format!("{}.png", name));
        let current_path = current_dir.join(format!("{}.png", name));
        let diff_path = diff_dir.join(format!("{}.png", name));

        debug!("Comparing: {}", name);

        let diff_percentage = if let Some(plugin) = differ_plugin {
            // Use plugin for comparison
            compare_with_plugin(
                plugin,
                &baseline_path,
                &current_path,
                &diff_path,
                config.threshold,
                working_dir,
            )?
        } else {
            // Use built-in comparison
            compare_images(&baseline_path, &current_path, &diff_path, config.threshold)?
        };

        if diff_percentage > config.threshold {
            info!("{}: {:.2}% different", name, diff_percentage);
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
            debug!("{}: unchanged", name);
            result.unchanged.push(name.clone());
            // Remove diff file if it exists and shot is unchanged
            let _ = std::fs::remove_file(&diff_path);
        }
    }

    // Sort for consistent output
    result.unchanged.sort();
    result.added.sort();
    result.removed.sort();
    result.changed.sort_by(|a, b| a.name.cmp(&b.name));

    info!(
        "Diff complete: {} unchanged, {} changed, {} added, {} removed",
        result.unchanged.len(),
        result.changed.len(),
        result.added.len(),
        result.removed.len()
    );

    Ok(result)
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

/// Compares two images and generates a diff image.
///
/// Returns the percentage of pixels that differ.
fn compare_images(
    baseline_path: &Path,
    current_path: &Path,
    diff_path: &Path,
    threshold: f64,
) -> Result<f64> {
    let baseline = image::open(baseline_path)
        .with_context(|| format!("Failed to load baseline image: {}", baseline_path.display()))?
        .to_rgba8();

    let current = image::open(current_path)
        .with_context(|| format!("Failed to load current image: {}", current_path.display()))?
        .to_rgba8();

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
    let total_pixels = (width * height) as f64;
    let mut diff_count = 0u64;

    let mut diff_img: RgbaImage = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let baseline_pixel = baseline.get_pixel(x, y);
            let current_pixel = current.get_pixel(x, y);

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
    }

    let diff_percentage = (diff_count as f64 / total_pixels) * 100.0;

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
}
