//! HTML report generation for visual regression results.
//!
//! This module generates a static HTML report that displays visual diff results
//! in several comparison views (side by side, slider, flicker, heatmap, diff and
//! change-by-change), and a machine-readable JSON export (results.json) for CI
//! integration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tracing::{info, warn};

use crate::config::Config;
use crate::diff::{DiffResult, Region};

mod template;

/// What the report needs to know about the screenshot files besides the diff result.
#[derive(Debug, Default)]
pub(crate) struct Assets {
    /// SHA-256 of each changed shot's current image, by shot name
    pub hashes: HashMap<String, String>,
    /// Pixel size of each image the report shows, keyed like `current/<name>`
    pub sizes: HashMap<String, (u32, u32)>,
}

impl Assets {
    /// Hashes the changed screenshots and reads the size of every image the
    /// report shows, in parallel. Sizes come from the PNG header alone.
    ///
    /// Knowing sizes up front lets the report reserve each image's space before
    /// it loads, so scrolling a long report never jumps.
    fn collect(result: &DiffResult, output_dir: &Path, workers: usize) -> Self {
        enum Task {
            Hash(String, PathBuf),
            Size(String, PathBuf),
        }

        let mut tasks = Vec::new();
        for shot in &result.changed {
            tasks.push(Task::Hash(shot.name.clone(), shot.current_path.clone()));
            tasks.push(Task::Size(
                format!("baseline/{}", shot.name),
                shot.baseline_path.clone(),
            ));
            tasks.push(Task::Size(
                format!("current/{}", shot.name),
                shot.current_path.clone(),
            ));
        }
        for (folder, names) in [("current", &result.added), ("baseline", &result.removed)] {
            for name in names {
                tasks.push(Task::Size(
                    format!("{}/{}", folder, name),
                    output_dir.join(folder).join(format!("{}.png", name)),
                ));
            }
        }

        let mut assets = Assets::default();
        let outcomes = crate::parallel::map(&tasks, workers, |task| match task {
            Task::Hash(_, path) => screenshot_hash(path).map(Ok),
            Task::Size(_, path) => image::image_dimensions(path)
                .map(Err)
                .map_err(anyhow::Error::from),
        });
        for (task, outcome) in tasks.iter().zip(outcomes) {
            match (task, outcome) {
                (Task::Hash(name, _), Ok(Ok(hash))) => {
                    assets.hashes.insert(name.clone(), hash);
                }
                (Task::Size(key, _), Ok(Err(size))) => {
                    assets.sizes.insert(key.clone(), size);
                }
                (Task::Hash(name, _), Err(error)) => {
                    warn!("⚠️  Could not hash {}: {:#}", name, error)
                }
                // A missing or unreadable image still gets a frame; the report says so.
                _ => {}
            }
        }
        assets
    }
}

/// JSON export format for results.json
#[derive(Debug, Serialize)]
pub struct ResultsJson {
    /// Schema version
    pub version: &'static str,
    /// ISO 8601 timestamp when the report was generated
    pub timestamp: String,
    /// Summary statistics
    pub summary: ResultsSummary,
    /// Detailed results grouped by status
    pub results: ResultsDetail,
}

/// Summary statistics for the JSON export
#[derive(Debug, Serialize)]
pub struct ResultsSummary {
    /// Total number of shots compared
    pub total: usize,
    /// Number of unchanged shots
    pub unchanged: usize,
    /// Number of changed shots
    pub changed: usize,
    /// Number of added shots
    pub added: usize,
    /// Number of removed shots
    pub removed: usize,
    /// Number of shots that could not be captured or compared
    pub failed: usize,
    /// Whether all tests passed (no changes and no failures)
    pub passed: bool,
}

/// Detailed results by category
#[derive(Debug, Serialize)]
pub struct ResultsDetail {
    /// Changed shots with diff information
    pub changed: Vec<ChangedShotJson>,
    /// Names of added shots
    pub added: Vec<String>,
    /// Names of removed shots
    pub removed: Vec<String>,
    /// Names of unchanged shots
    pub unchanged: Vec<String>,
    /// Shots that could not be captured or compared
    pub failed: Vec<FailedShotJson>,
}

/// Failed shot information for JSON export
#[derive(Debug, Serialize)]
pub struct FailedShotJson {
    /// Name of the shot
    pub name: String,
    /// Why it failed
    pub error: String,
}

/// Changed shot information for JSON export
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedShotJson {
    /// Name of the shot
    pub name: String,
    /// Percentage of pixels that differ
    pub diff_percentage: f64,
    /// Viewport name (if multi-viewport)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<String>,
    /// Path to baseline image (relative to output dir)
    pub baseline_path: String,
    /// Path to current image (relative to output dir)
    pub current_path: String,
    /// Path to diff image (relative to output dir)
    pub diff_path: String,
    /// Path to the heatmap image (relative to output dir), when one was drawn
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heatmap_path: Option<String>,
    /// Areas that changed, in image pixels and reading order
    pub regions: Vec<Region>,
    /// SHA-256 of the current image, so approvals can be tied to what was reviewed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<String>,
}

/// Returns the lowercase hex SHA-256 of a file.
///
/// Review decisions record this hash, and `apply`/`review` refuse to promote a
/// screenshot whose bytes no longer match what the reviewer approved.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn screenshot_hash(path: &Path) -> Result<String> {
    let bytes =
        std::fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    Ok(digest.iter().map(|byte| format!("{:02x}", byte)).collect())
}

/// Generates an HTML report and JSON export from diff results.
///
/// The report includes:
/// - Summary of unchanged, changed, added, and removed shots
/// - Comparison views for changed shots: side by side, slider, flicker,
///   heatmap, diff, and stepping through each changed area
/// - Preview for added shots
/// - List of removed shots
///
/// Additionally generates a `results.json` file for CI integration and machine parsing.
///
/// # Example
///
/// ```rust,no_run
/// use pixelguard_core::{Config, diff_images, generate_report};
///
/// fn example() -> anyhow::Result<()> {
///     let config = Config::load("pixelguard.config.json")?;
///     let diff_result = diff_images(&config, ".", None)?;
///     generate_report(&config, &diff_result, ".")?;
///     println!("Report generated at .pixelguard/report.html");
///     println!("JSON export at .pixelguard/results.json");
///     Ok(())
/// }
/// ```
pub fn generate_report<P: AsRef<Path>>(
    config: &Config,
    result: &DiffResult,
    working_dir: P,
) -> Result<std::path::PathBuf> {
    let working_dir = working_dir.as_ref();
    let output_dir = working_dir.join(&config.output_dir);
    let report_path = output_dir.join("report.html");
    let json_path = output_dir.join("results.json");

    let assets = Assets::collect(result, &output_dir, config.concurrency);

    // Generate HTML report
    let html = generate_html(result, &assets);
    std::fs::write(&report_path, html)
        .with_context(|| format!("Failed to write report to {}", report_path.display()))?;

    // Generate JSON export
    let json = generate_results_json(result, &assets.hashes);
    let json_str =
        serde_json::to_string_pretty(&json).context("Failed to serialize results to JSON")?;
    std::fs::write(&json_path, json_str)
        .with_context(|| format!("Failed to write JSON export to {}", json_path.display()))?;

    info!("📊 Report generated at {}", report_path.display());
    info!("📊 JSON export generated at {}", json_path.display());
    Ok(report_path)
}

/// Generates a JSON export structure from diff results.
///
/// This creates a machine-readable format suitable for CI integration,
/// custom tooling, or programmatic analysis.
fn generate_results_json(result: &DiffResult, hashes: &HashMap<String, String>) -> ResultsJson {
    let total = result.unchanged.len()
        + result.changed.len()
        + result.added.len()
        + result.removed.len()
        + result.failed.len();
    let passed = result.passed();

    ResultsJson {
        version: "1.0",
        timestamp: Utc::now().to_rfc3339(),
        summary: ResultsSummary {
            total,
            unchanged: result.unchanged.len(),
            changed: result.changed.len(),
            added: result.added.len(),
            removed: result.removed.len(),
            failed: result.failed.len(),
            passed,
        },
        results: ResultsDetail {
            changed: result
                .changed
                .iter()
                .map(|shot| ChangedShotJson {
                    name: shot.name.clone(),
                    diff_percentage: shot.diff_percentage,
                    viewport: shot.viewport.clone(),
                    baseline_path: format!("baseline/{}.png", shot.name),
                    current_path: format!("current/{}.png", shot.name),
                    diff_path: format!("diff/{}.png", shot.name),
                    heatmap_path: shot
                        .heatmap_path
                        .as_ref()
                        .map(|_| format!("heatmap/{}.png", shot.name)),
                    regions: shot.regions.clone(),
                    current_hash: hashes.get(&shot.name).cloned(),
                })
                .collect(),
            added: result.added.clone(),
            removed: result.removed.clone(),
            unchanged: result.unchanged.clone(),
            failed: result
                .failed
                .iter()
                .map(|failure| FailedShotJson {
                    name: failure.name.clone(),
                    error: failure.error.clone(),
                })
                .collect(),
        },
    }
}

/// Renders the HTML report, stamped with the current time.
fn generate_html(result: &DiffResult, assets: &Assets) -> String {
    let generated_at = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    template::render(result, assets, &generated_at)
}

#[cfg(test)]
mod tests {
    use super::template::html_escape;
    use super::*;
    use crate::diff::ChangedShot;

    #[test]
    fn html_escape_handles_special_chars() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn generate_html_creates_valid_document() {
        let result = DiffResult {
            unchanged: vec!["button--primary".to_string()],
            changed: vec![ChangedShot {
                name: "card--default".to_string(),
                baseline_path: "baseline/card--default.png".into(),
                current_path: "current/card--default.png".into(),
                diff_path: "diff/card--default.png".into(),
                diff_percentage: 5.5,
                viewport: None,
                heatmap_path: None,
                regions: Vec::new(),
            }],
            added: vec!["new-component".to_string()],
            removed: vec!["old-component".to_string()],
            failed: vec![],
        };

        let html = generate_html(&result, &Assets::default());

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Pixelguard"));
        assert!(html.contains("button--primary"));
        assert!(html.contains("card--default"));
        assert!(html.contains("5.50%"));
        assert!(html.contains("new-component"));
        assert!(html.contains("old-component"));
    }

    #[test]
    fn changed_shots_carry_image_paths_and_review_hash() {
        let result = DiffResult {
            changed: vec![ChangedShot {
                name: "card--default@mobile".to_string(),
                baseline_path: "baseline/card--default@mobile.png".into(),
                current_path: "current/card--default@mobile.png".into(),
                diff_path: "diff/card--default@mobile.png".into(),
                diff_percentage: 5.5,
                viewport: Some("mobile".to_string()),
                heatmap_path: Some("heatmap/card--default@mobile.png".into()),
                regions: vec![
                    Region {
                        x: 8,
                        y: 16,
                        width: 40,
                        height: 24,
                    },
                    Region {
                        x: 0,
                        y: 200,
                        width: 320,
                        height: 8,
                    },
                ],
            }],
            ..DiffResult::default()
        };
        let assets = Assets {
            hashes: [("card--default@mobile".to_string(), "ab".repeat(32))]
                .into_iter()
                .collect(),
            sizes: [
                ("baseline/card--default@mobile".to_string(), (375, 812)),
                ("current/card--default@mobile".to_string(), (375, 900)),
            ]
            .into_iter()
            .collect(),
        };

        let html = generate_html(&result, &assets);

        assert!(html.contains("data-src-baseline=\"baseline/card--default@mobile.png\""));
        assert!(html.contains("data-src-current=\"current/card--default@mobile.png\""));
        assert!(html.contains("data-src-diff=\"diff/card--default@mobile.png\""));
        assert!(html.contains("data-src-heatmap=\"heatmap/card--default@mobile.png\""));
        assert!(html.contains("data-regions=\"8,16,40,24 0,200,320,8\""));
        assert!(html.contains("data-baseline-size=\"375x812\" data-current-size=\"375x900\""));
        assert!(html.contains("in 2 areas"));
        assert!(html.contains(&format!("data-hash=\"{}\"", "ab".repeat(32))));
        assert!(html.contains("href=\"#shot-0\""));
        assert!(html.contains("<h1>1 changed</h1>"));
        // Without scripting, a side-by-side comparison still shows, sized up front.
        assert!(html.contains(
            "<img src=\"baseline/card--default@mobile.png\" alt=\"Baseline of card--default@mobile\" width=\"375\" height=\"812\""
        ));
        assert!(html.contains("375 × 900, size changed"));
    }

    #[test]
    fn every_comparison_view_is_offered_for_changed_shots() {
        let result = DiffResult {
            changed: vec![ChangedShot {
                name: "card".to_string(),
                baseline_path: "baseline/card.png".into(),
                current_path: "current/card.png".into(),
                diff_path: "diff/card.png".into(),
                diff_percentage: 1.0,
                viewport: None,
                heatmap_path: None,
                regions: Vec::new(),
            }],
            ..DiffResult::default()
        };

        let html = generate_html(&result, &Assets::default());

        for view in ["side", "slider", "flicker", "heatmap", "diff", "changes"] {
            assert!(
                html.contains(&format!("data-view-choice=\"{}\"", view)),
                "missing {} view",
                view
            );
        }
        assert!(!html.contains("data-src-heatmap"), "no heatmap was drawn");
    }

    #[test]
    fn results_json_lists_heatmap_and_regions() {
        let result = DiffResult {
            changed: vec![ChangedShot {
                name: "card".to_string(),
                baseline_path: "baseline/card.png".into(),
                current_path: "current/card.png".into(),
                diff_path: "diff/card.png".into(),
                diff_percentage: 1.0,
                viewport: None,
                heatmap_path: Some("heatmap/card.png".into()),
                regions: vec![Region {
                    x: 0,
                    y: 8,
                    width: 16,
                    height: 8,
                }],
            }],
            ..DiffResult::default()
        };

        let json = serde_json::to_value(generate_results_json(&result, &HashMap::new())).unwrap();
        let changed = &json["results"]["changed"][0];

        assert_eq!(changed["heatmapPath"], "heatmap/card.png");
        assert_eq!(changed["regions"][0]["y"], 8);
        assert_eq!(changed["regions"][0]["width"], 16);
    }

    #[test]
    fn added_and_removed_frames_reserve_their_size() {
        let result = DiffResult {
            added: vec!["fresh".to_string()],
            removed: vec!["gone".to_string()],
            ..DiffResult::default()
        };
        let assets = Assets {
            sizes: [("current/fresh".to_string(), (800, 600))]
                .into_iter()
                .collect(),
            ..Assets::default()
        };

        let html = generate_html(&result, &assets);

        assert!(html.contains("width=\"800\" height=\"600\""));
        assert!(html.contains("file missing"), "an unreadable image says so");
    }

    #[test]
    fn passing_runs_say_so_and_list_unchanged_shots() {
        let result = DiffResult {
            unchanged: vec!["button--primary".to_string()],
            ..DiffResult::default()
        };

        let html = generate_html(&result, &Assets::default());

        assert!(html.contains("<h1>No visual changes</h1>"));
        assert!(html.contains("<details class=\"group unchanged\" open>"));
        assert!(html.contains("Nothing to review."));
    }

    #[test]
    fn generate_html_handles_empty_results() {
        let result = DiffResult {
            unchanged: vec![],
            changed: vec![],
            added: vec![],
            removed: vec![],
            failed: vec![],
        };

        let html = generate_html(&result, &Assets::default());

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<h1>Nothing was compared</h1>"));
        assert!(
            !html.contains("{{"),
            "template placeholders leaked into the report"
        );
    }

    #[test]
    fn report_is_generated_to_correct_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".pixelguard")).unwrap();

        let config = Config {
            output_dir: ".pixelguard".to_string(),
            ..Default::default()
        };

        let result = DiffResult {
            unchanged: vec![],
            changed: vec![],
            added: vec![],
            removed: vec![],
            failed: vec![],
        };

        let report_path = generate_report(&config, &result, dir.path()).unwrap();

        assert!(report_path.ends_with("report.html"));
        assert!(report_path.exists());
    }

    #[test]
    fn generate_html_escapes_and_encodes_hostile_names() {
        let result = DiffResult {
            added: vec!["a b<img src=x onerror=alert(1)>".to_string()],
            ..DiffResult::default()
        };

        let html = generate_html(&result, &Assets::default());

        assert!(!html.contains("<img src=x"));
        assert!(html.contains("current/a%20b%3Cimg%20src%3Dx%20onerror%3Dalert%281%29%3E.png"));
    }

    #[test]
    fn failed_shots_fail_the_report_and_json() {
        let result = DiffResult {
            unchanged: vec!["ok".to_string()],
            failed: vec![crate::capture::FailedShot {
                name: "broken".to_string(),
                error: "Timeout 30000ms exceeded".to_string(),
            }],
            ..DiffResult::default()
        };

        let html = generate_html(&result, &Assets::default());
        let json = generate_results_json(&result, &HashMap::new());

        assert!(html.contains("Timeout 30000ms exceeded"));
        assert!(!json.summary.passed);
        assert_eq!(json.summary.failed, 1);
    }

    #[test]
    fn screenshot_hash_is_sha256_hex() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("x.png");
        std::fs::write(&file, b"abc").unwrap();

        assert_eq!(
            screenshot_hash(&file).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
