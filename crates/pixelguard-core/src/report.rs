//! HTML report generation for visual regression results.
//!
//! This module generates a static HTML report that displays visual diff results
//! with side-by-side comparison of baseline, current, and diff images.
//! Additionally generates a machine-readable JSON export (results.json) for CI integration.

use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use tracing::info;

use crate::config::Config;
use crate::diff::DiffResult;

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
    /// Whether all tests passed (no changes)
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
}

/// Generates an HTML report and JSON export from diff results.
///
/// The report includes:
/// - Summary of unchanged, changed, added, and removed shots
/// - Side-by-side comparison for changed shots (baseline, current, diff)
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

    // Generate HTML report
    let html = generate_html(result);
    std::fs::write(&report_path, html)
        .with_context(|| format!("Failed to write report to {}", report_path.display()))?;

    // Generate JSON export
    let json = generate_results_json(result);
    let json_str =
        serde_json::to_string_pretty(&json).context("Failed to serialize results to JSON")?;
    std::fs::write(&json_path, json_str)
        .with_context(|| format!("Failed to write JSON export to {}", json_path.display()))?;

    info!("Report generated at {}", report_path.display());
    info!("JSON export generated at {}", json_path.display());
    Ok(report_path)
}

/// Generates a JSON export structure from diff results.
///
/// This creates a machine-readable format suitable for CI integration,
/// custom tooling, or programmatic analysis.
fn generate_results_json(result: &DiffResult) -> ResultsJson {
    let total =
        result.unchanged.len() + result.changed.len() + result.added.len() + result.removed.len();
    let passed = result.changed.is_empty() && result.added.is_empty() && result.removed.is_empty();

    ResultsJson {
        version: "1.0",
        timestamp: Utc::now().to_rfc3339(),
        summary: ResultsSummary {
            total,
            unchanged: result.unchanged.len(),
            changed: result.changed.len(),
            added: result.added.len(),
            removed: result.removed.len(),
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
                })
                .collect(),
            added: result.added.clone(),
            removed: result.removed.clone(),
            unchanged: result.unchanged.clone(),
        },
    }
}

/// SVG icons as inline strings
mod icons {
    pub const LOGO: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><rect x="7" y="7" width="3" height="3" fill="currentColor" stroke="none"/><rect x="14" y="7" width="3" height="3" fill="currentColor" stroke="none" opacity="0.5"/><rect x="7" y="14" width="3" height="3" fill="currentColor" stroke="none" opacity="0.5"/><rect x="14" y="14" width="3" height="3" fill="currentColor" stroke="none"/></svg>"#;

    pub const CHECK_CIRCLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/></svg>"#;

    pub const X_CIRCLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m15 9-6 6"/><path d="m9 9 6 6"/></svg>"#;

    pub const PLUS_CIRCLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>"#;

    pub const MINUS_CIRCLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M8 12h8"/></svg>"#;

    pub const IMAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>"#;

    pub const CHEVRON_DOWN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>"#;

    pub const EXTERNAL_LINK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>"#;

    pub const SUN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/></svg>"#;

    pub const MOON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/></svg>"#;

    pub const X: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>"#;

    pub const ZOOM_IN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/><path d="M11 8v6"/><path d="M8 11h6"/></svg>"#;

    pub const MONITOR: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/></svg>"#;

    pub const SEARCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>"#;

    pub const APPROVE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 12 2 2 4-4"/><path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"/></svg>"#;

    pub const REJECT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m15 9-6 6"/><path d="m9 9 6 6"/></svg>"#;

    pub const DOWNLOAD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7,10 12,15 17,10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>"#;
}

fn generate_html(result: &DiffResult) -> String {
    let total =
        result.unchanged.len() + result.changed.len() + result.added.len() + result.removed.len();

    let changed_html = if result.changed.is_empty() {
        String::new()
    } else {
        let items: String = result
            .changed
            .iter()
            .map(|shot| {
                // Generate viewport badge if multi-viewport
                let viewport_badge = shot
                    .viewport
                    .as_ref()
                    .map(|v| format!(r#"<span class="badge badge--viewport">{}</span>"#, html_escape(v)))
                    .unwrap_or_default();

                format!(
                    r#"
            <div class="shot-card" data-name="{name}" data-status="changed" data-diff="{diff}">
                <div class="shot-header">
                    <div class="shot-title">
                        <span class="shot-icon">{image_icon}</span>
                        <span class="shot-name">{name}</span>
                        {viewport_badge}
                    </div>
                    <div class="shot-header-right">
                        <div class="shot-actions">
                            <button class="action-btn action-btn--approve" data-shot="{name}" data-action="approve" title="Approve">{approve_icon}</button>
                            <button class="action-btn action-btn--reject" data-shot="{name}" data-action="reject" title="Reject">{reject_icon}</button>
                        </div>
                        <span class="badge badge--diff">{diff:.2}% changed</span>
                    </div>
                </div>
                <div class="comparison-tabs">
                    <button class="tab-btn active" data-view="side-by-side">Side by Side</button>
                    <button class="tab-btn" data-view="slider">Slider</button>
                    <button class="tab-btn" data-view="diff">Diff Only</button>
                </div>
                <div class="comparison-views">
                    <div class="view-side-by-side active">
                        <div class="comparison">
                            <div class="image-panel" data-zoomable data-src="baseline/{name}.png" data-label="Baseline">
                                <div class="image-label">Baseline</div>
                                <div class="image-frame">
                                    <img src="baseline/{name}.png" alt="Baseline" loading="lazy">
                                    <div class="zoom-hint">{zoom_icon}</div>
                                </div>
                            </div>
                            <div class="image-panel" data-zoomable data-src="current/{name}.png" data-label="Current">
                                <div class="image-label">Current</div>
                                <div class="image-frame">
                                    <img src="current/{name}.png" alt="Current" loading="lazy">
                                    <div class="zoom-hint">{zoom_icon}</div>
                                </div>
                            </div>
                            <div class="image-panel" data-zoomable data-src="diff/{name}.png" data-label="Diff">
                                <div class="image-label">Diff</div>
                                <div class="image-frame image-frame--diff">
                                    <img src="diff/{name}.png" alt="Diff" loading="lazy">
                                    <div class="zoom-hint">{zoom_icon}</div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="view-slider">
                        <div class="slider-container">
                            <div class="slider-baseline">
                                <img src="baseline/{name}.png" alt="Baseline" loading="lazy">
                            </div>
                            <div class="slider-current">
                                <img src="current/{name}.png" alt="Current" loading="lazy">
                            </div>
                            <span class="slider-label slider-label--left">Baseline</span>
                            <span class="slider-label slider-label--right">Current</span>
                            <div class="slider-handle">
                                <div class="slider-line"></div>
                                <div class="slider-grip">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg>
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="view-diff">
                        <div class="diff-only-container" data-zoomable data-src="diff/{name}.png" data-label="Diff">
                            <img src="diff/{name}.png" alt="Diff" loading="lazy">
                            <div class="zoom-hint">{zoom_icon}</div>
                        </div>
                    </div>
                </div>
            </div>
            "#,
                    name = html_escape(&shot.name),
                    diff = shot.diff_percentage,
                    viewport_badge = viewport_badge,
                    image_icon = icons::IMAGE,
                    zoom_icon = icons::ZOOM_IN,
                    approve_icon = icons::APPROVE,
                    reject_icon = icons::REJECT,
                )
            })
            .collect();

        format!(
            r#"
        <section class="section">
            <div class="section-header">
                <div class="section-title">
                    <span class="section-icon section-icon--diff">{icon}</span>
                    <h2>Changed</h2>
                    <span class="section-count">{count}</span>
                </div>
            </div>
            <div class="section-content">
                {items}
            </div>
        </section>
        "#,
            count = result.changed.len(),
            items = items,
            icon = icons::X_CIRCLE,
        )
    };

    let added_html = if result.added.is_empty() {
        String::new()
    } else {
        let items: String = result
            .added
            .iter()
            .map(|name| {
                format!(
                    r#"
            <div class="shot-card" data-name="{name}" data-status="added" data-diff="0">
                <div class="shot-header">
                    <div class="shot-title">
                        <span class="shot-icon">{image_icon}</span>
                        <span class="shot-name">{name}</span>
                    </div>
                    <span class="badge badge--new">New</span>
                </div>
                <div class="comparison comparison--single">
                    <div class="image-panel" data-zoomable data-src="current/{name}.png" data-label="Current">
                        <div class="image-label">Current</div>
                        <div class="image-frame">
                            <img src="current/{name}.png" alt="Current" loading="lazy">
                            <div class="zoom-hint">{zoom_icon}</div>
                        </div>
                    </div>
                </div>
            </div>
            "#,
                    name = html_escape(name),
                    image_icon = icons::IMAGE,
                    zoom_icon = icons::ZOOM_IN,
                )
            })
            .collect();

        format!(
            r#"
        <section class="section">
            <div class="section-header">
                <div class="section-title">
                    <span class="section-icon section-icon--new">{icon}</span>
                    <h2>Added</h2>
                    <span class="section-count">{count}</span>
                </div>
            </div>
            <div class="section-content">
                {items}
            </div>
        </section>
        "#,
            count = result.added.len(),
            items = items,
            icon = icons::PLUS_CIRCLE,
        )
    };

    let removed_html = if result.removed.is_empty() {
        String::new()
    } else {
        let items: String = result
            .removed
            .iter()
            .map(|name| {
                format!(
                    r#"
            <div class="shot-card" data-name="{name}" data-status="removed" data-diff="0">
                <div class="shot-header">
                    <div class="shot-title">
                        <span class="shot-icon">{image_icon}</span>
                        <span class="shot-name">{name}</span>
                    </div>
                    <span class="badge badge--removed">Removed</span>
                </div>
                <div class="comparison comparison--single">
                    <div class="image-panel" data-zoomable data-src="baseline/{name}.png" data-label="Baseline (deleted)">
                        <div class="image-label">Baseline (deleted)</div>
                        <div class="image-frame">
                            <img src="baseline/{name}.png" alt="Baseline" loading="lazy">
                            <div class="zoom-hint">{zoom_icon}</div>
                        </div>
                    </div>
                </div>
            </div>
            "#,
                    name = html_escape(name),
                    image_icon = icons::IMAGE,
                    zoom_icon = icons::ZOOM_IN,
                )
            })
            .collect();

        format!(
            r#"
        <section class="section">
            <div class="section-header">
                <div class="section-title">
                    <span class="section-icon section-icon--removed">{icon}</span>
                    <h2>Removed</h2>
                    <span class="section-count">{count}</span>
                </div>
            </div>
            <div class="section-content">
                {items}
            </div>
        </section>
        "#,
            count = result.removed.len(),
            items = items,
            icon = icons::MINUS_CIRCLE,
        )
    };

    let unchanged_html = if result.unchanged.is_empty() {
        String::new()
    } else {
        let items: String = result
            .unchanged
            .iter()
            .map(|name| {
                format!(
                    r#"<div class="unchanged-item">
                        <span class="unchanged-icon">{icon}</span>
                        <span>{name}</span>
                    </div>"#,
                    name = html_escape(name),
                    icon = icons::CHECK_CIRCLE,
                )
            })
            .collect();

        format!(
            r#"
        <section class="section section--unchanged">
            <details class="unchanged-details">
                <summary class="unchanged-summary">
                    <div class="section-title">
                        <span class="section-icon section-icon--pass">{icon}</span>
                        <h2>Unchanged</h2>
                        <span class="section-count">{count}</span>
                    </div>
                    <span class="chevron">{chevron}</span>
                </summary>
                <div class="unchanged-grid">
                    {items}
                </div>
            </details>
        </section>
        "#,
            count = result.unchanged.len(),
            items = items,
            icon = icons::CHECK_CIRCLE,
            chevron = icons::CHEVRON_DOWN,
        )
    };

    let has_issues =
        !result.changed.is_empty() || !result.added.is_empty() || !result.removed.is_empty();
    let status_class = if has_issues { "fail" } else { "pass" };
    let status_text = if has_issues {
        "Visual changes detected"
    } else {
        "All tests passed"
    };
    let status_icon = if has_issues {
        icons::X_CIRCLE
    } else {
        icons::CHECK_CIRCLE
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Pixelguard Report</title>
    <style>
        *{{box-sizing:border-box;margin:0;padding:0}}
        :root{{--color-bg:#0a0a0b;--color-bg-elevated:#141416;--color-bg-hover:#1c1c1f;--color-border:#27272a;--color-border-subtle:#1f1f22;--color-text:#fafafa;--color-text-secondary:#a1a1aa;--color-text-muted:#71717a;--color-success:#22c55e;--color-success-muted:rgba(34,197,94,0.15);--color-error:#ef4444;--color-error-muted:rgba(239,68,68,0.15);--color-warning:#f59e0b;--color-warning-muted:rgba(245,158,11,0.15);--color-info:#3b82f6;--color-info-muted:rgba(59,130,246,0.15);--color-accent:#8b5cf6;--radius-sm:6px;--radius-md:8px;--radius-lg:12px;--shadow-sm:0 1px 2px rgba(0,0,0,0.3);--shadow-md:0 4px 12px rgba(0,0,0,0.4);--shadow-lg:0 8px 32px rgba(0,0,0,0.5);--font-mono:'SF Mono','Fira Code','Consolas',monospace;--checker-light:#1c1c1f;--checker-dark:#0a0a0b}}
        [data-theme="light"]{{--color-bg:#f8fafc;--color-bg-elevated:#ffffff;--color-bg-hover:#f1f5f9;--color-border:#e2e8f0;--color-border-subtle:#e2e8f0;--color-text:#0f172a;--color-text-secondary:#475569;--color-text-muted:#64748b;--color-success-muted:rgba(34,197,94,0.12);--color-error-muted:rgba(239,68,68,0.12);--color-warning-muted:rgba(245,158,11,0.12);--color-info-muted:rgba(59,130,246,0.12);--shadow-sm:0 1px 2px rgba(0,0,0,0.05);--shadow-md:0 4px 12px rgba(0,0,0,0.08);--shadow-lg:0 8px 32px rgba(0,0,0,0.12);--checker-light:#f1f5f9;--checker-dark:#e2e8f0}}
        body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI','Inter',Roboto,sans-serif;background:var(--color-bg);color:var(--color-text);line-height:1.5;min-height:100vh;-webkit-font-smoothing:antialiased;transition:background-color 0.2s,color 0.2s}}
        .container{{max-width:1440px;margin:0 auto;padding:32px 24px}}
        .header{{display:flex;align-items:center;justify-content:space-between;margin-bottom:32px;padding-bottom:24px;border-bottom:1px solid var(--color-border-subtle)}}
        .logo{{display:flex;align-items:center;gap:12px;text-decoration:none;color:var(--color-text)}}
        .logo-icon{{color:var(--color-text);opacity:0.9}}
        .logo-text{{font-size:20px;font-weight:600;letter-spacing:-0.025em}}
        .header-actions{{display:flex;align-items:center;gap:8px}}
        .theme-toggle{{display:flex;align-items:center;background:var(--color-bg-elevated);border:1px solid var(--color-border);border-radius:9999px;padding:4px;gap:2px}}
        .theme-btn{{display:flex;align-items:center;justify-content:center;width:32px;height:32px;border:none;border-radius:9999px;background:transparent;color:var(--color-text-muted);cursor:pointer;transition:all 0.15s}}
        .theme-btn:hover{{color:var(--color-text);background:var(--color-bg-hover)}}
        .theme-btn.active{{background:var(--color-accent);color:white}}
        .status-banner{{display:flex;align-items:center;gap:16px;padding:20px 24px;border-radius:var(--radius-lg);margin-bottom:32px;transition:background-color 0.2s,border-color 0.2s}}
        .status-banner.pass{{background:var(--color-success-muted);border:1px solid rgba(34,197,94,0.3)}}
        .status-banner.fail{{background:var(--color-error-muted);border:1px solid rgba(239,68,68,0.3)}}
        .status-icon{{display:flex;align-items:center;justify-content:center}}
        .status-banner.pass .status-icon{{color:var(--color-success)}}
        .status-banner.fail .status-icon{{color:var(--color-error)}}
        .status-content h2{{font-size:18px;font-weight:600;margin-bottom:4px}}
        .status-content p{{color:var(--color-text-secondary);font-size:14px}}
        .stats-grid{{display:grid;grid-template-columns:repeat(4,1fr);gap:16px;margin-bottom:40px}}
        @media(max-width:768px){{.stats-grid{{grid-template-columns:repeat(2,1fr)}}}}
        .stat-card{{background:var(--color-bg-elevated);border:1px solid var(--color-border-subtle);border-radius:var(--radius-md);padding:20px;transition:border-color 0.15s,background-color 0.2s}}
        .stat-card:hover{{border-color:var(--color-border)}}
        .stat-value{{font-size:32px;font-weight:700;letter-spacing:-0.025em;margin-bottom:4px}}
        .stat-card--pass .stat-value{{color:var(--color-success)}}
        .stat-card--fail .stat-value{{color:var(--color-error)}}
        .stat-card--new .stat-value{{color:var(--color-warning)}}
        .stat-card--removed .stat-value{{color:var(--color-info)}}
        .stat-label{{display:flex;align-items:center;gap:6px;color:var(--color-text-muted);font-size:13px;font-weight:500}}
        .stat-label svg{{opacity:0.7}}
        .section{{margin-bottom:40px}}
        .section-header{{display:flex;align-items:center;justify-content:space-between;margin-bottom:20px}}
        .section-title{{display:flex;align-items:center;gap:10px}}
        .section-title h2{{font-size:16px;font-weight:600}}
        .section-icon{{display:flex;align-items:center;justify-content:center;width:32px;height:32px;border-radius:var(--radius-sm)}}
        .section-icon--diff{{background:var(--color-error-muted);color:var(--color-error)}}
        .section-icon--new{{background:var(--color-warning-muted);color:var(--color-warning)}}
        .section-icon--removed{{background:var(--color-info-muted);color:var(--color-info)}}
        .section-icon--pass{{background:var(--color-success-muted);color:var(--color-success)}}
        .section-count{{background:var(--color-bg-hover);color:var(--color-text-secondary);font-size:12px;font-weight:600;padding:2px 8px;border-radius:9999px}}
        .shot-card{{background:var(--color-bg-elevated);border:1px solid var(--color-border-subtle);border-radius:var(--radius-lg);overflow:hidden;margin-bottom:16px;transition:border-color 0.15s,background-color 0.2s}}
        .shot-card:hover{{border-color:var(--color-border)}}
        .shot-header{{display:flex;align-items:center;justify-content:space-between;padding:16px 20px;border-bottom:1px solid var(--color-border-subtle)}}
        .shot-title{{display:flex;align-items:center;gap:10px}}
        .shot-icon{{color:var(--color-text-muted);display:flex}}
        .shot-name{{font-family:var(--font-mono);font-size:14px;font-weight:500}}
        .badge{{font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:0.025em;padding:4px 10px;border-radius:9999px}}
        .badge--diff{{background:var(--color-error-muted);color:var(--color-error)}}
        .badge--new{{background:var(--color-warning-muted);color:var(--color-warning)}}
        .badge--removed{{background:var(--color-info-muted);color:var(--color-info)}}
        .badge--viewport{{background:var(--color-bg-hover);color:var(--color-text-muted);font-size:10px;padding:3px 8px}}
        .comparison-tabs{{display:flex;gap:4px;padding:12px 20px;border-bottom:1px solid var(--color-border-subtle);background:var(--color-bg)}}
        .tab-btn{{padding:8px 16px;border:none;border-radius:var(--radius-sm);background:transparent;color:var(--color-text-muted);font-size:13px;font-weight:500;cursor:pointer;transition:all 0.15s}}
        .tab-btn:hover{{color:var(--color-text);background:var(--color-bg-hover)}}
        .tab-btn.active{{color:var(--color-text);background:var(--color-bg-elevated);box-shadow:var(--shadow-sm)}}
        .comparison-views{{position:relative}}
        .view-side-by-side,.view-slider,.view-diff{{display:none}}
        .view-side-by-side.active,.view-slider.active,.view-diff.active{{display:block}}
        .comparison{{display:grid;grid-template-columns:repeat(3,1fr);gap:1px;background:var(--color-border-subtle)}}
        .comparison--single{{grid-template-columns:1fr;max-width:600px}}
        @media(max-width:1024px){{.comparison{{grid-template-columns:1fr}}}}
        .image-panel{{background:var(--color-bg);cursor:pointer;position:relative}}
        .image-label{{padding:10px 16px;font-size:12px;font-weight:500;color:var(--color-text-muted);text-transform:uppercase;letter-spacing:0.05em;border-bottom:1px solid var(--color-border-subtle)}}
        .image-frame{{padding:16px;display:flex;align-items:center;justify-content:center;background:repeating-conic-gradient(var(--checker-light) 0% 25%,var(--checker-dark) 0% 50%) 50%/16px 16px;min-height:200px;position:relative}}
        .image-frame--diff{{background:var(--color-bg)}}
        .image-frame img{{max-width:100%;height:auto;border-radius:var(--radius-sm);box-shadow:var(--shadow-md)}}
        .zoom-hint{{position:absolute;bottom:24px;right:24px;width:32px;height:32px;display:flex;align-items:center;justify-content:center;background:var(--color-bg-elevated);border:1px solid var(--color-border);border-radius:var(--radius-sm);color:var(--color-text-muted);opacity:0;transition:opacity 0.15s}}
        .image-panel:hover .zoom-hint,.diff-only-container:hover .zoom-hint{{opacity:1}}
        .slider-container{{position:relative;width:100%;cursor:ew-resize;user-select:none;background:repeating-conic-gradient(var(--checker-light) 0% 25%,var(--checker-dark) 0% 50%) 50%/16px 16px}}
        .slider-baseline{{position:relative;display:block}}
        .slider-baseline img{{width:100%;height:auto;display:block}}
        .slider-current{{position:absolute;top:0;left:0;bottom:0;width:50%;overflow:hidden}}
        .slider-current img{{display:block;height:auto;max-width:none}}
        .slider-label{{position:absolute;top:16px;padding:6px 12px;background:var(--color-bg-elevated);border:1px solid var(--color-border);border-radius:var(--radius-sm);font-size:12px;font-weight:500;color:var(--color-text-secondary);text-transform:uppercase;letter-spacing:0.05em;z-index:5}}
        .slider-label--left{{left:16px}}
        .slider-label--right{{right:16px}}
        .slider-handle{{position:absolute;top:0;bottom:0;left:50%;width:4px;transform:translateX(-50%);cursor:ew-resize;z-index:10}}
        .slider-line{{position:absolute;top:0;bottom:0;left:50%;width:2px;background:var(--color-accent);transform:translateX(-50%);box-shadow:0 0 8px rgba(139,92,246,0.5)}}
        .slider-grip{{position:absolute;top:50%;left:50%;transform:translate(-50%,-50%);width:44px;height:44px;background:var(--color-bg-elevated);border:2px solid var(--color-accent);border-radius:50%;display:flex;align-items:center;justify-content:center;color:var(--color-accent);box-shadow:var(--shadow-lg)}}
        .slider-grip svg:first-child{{margin-right:-8px}}
        .slider-grip svg:last-child{{margin-left:-8px}}
        .diff-only-container{{padding:24px;display:flex;align-items:center;justify-content:center;background:var(--color-bg);cursor:pointer;position:relative}}
        .diff-only-container img{{max-width:100%;height:auto;border-radius:var(--radius-sm);box-shadow:var(--shadow-md)}}
        .modal-overlay{{position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.9);z-index:1000;display:none;align-items:center;justify-content:center;padding:24px;opacity:0;transition:opacity 0.2s}}
        .modal-overlay.active{{display:flex;opacity:1}}
        .modal-content{{position:relative;max-width:95vw;max-height:95vh}}
        .modal-image{{max-width:100%;max-height:90vh;border-radius:var(--radius-md);box-shadow:var(--shadow-lg)}}
        .modal-label{{position:absolute;top:-40px;left:0;color:white;font-size:14px;font-weight:500}}
        .modal-close{{position:absolute;top:-48px;right:0;width:40px;height:40px;display:flex;align-items:center;justify-content:center;background:rgba(255,255,255,0.1);border:none;border-radius:var(--radius-sm);color:white;cursor:pointer;transition:background 0.15s}}
        .modal-close:hover{{background:rgba(255,255,255,0.2)}}
        .modal-hint{{position:absolute;bottom:-36px;left:50%;transform:translateX(-50%);color:rgba(255,255,255,0.5);font-size:12px}}
        .section--unchanged{{margin-top:24px}}
        .unchanged-details{{background:var(--color-bg-elevated);border:1px solid var(--color-border-subtle);border-radius:var(--radius-lg);transition:background-color 0.2s}}
        .unchanged-summary{{display:flex;align-items:center;justify-content:space-between;padding:16px 20px;cursor:pointer;list-style:none;user-select:none}}
        .unchanged-summary::-webkit-details-marker{{display:none}}
        .unchanged-summary:hover{{background:var(--color-bg-hover)}}
        .chevron{{color:var(--color-text-muted);transition:transform 0.2s;display:flex}}
        details[open] .chevron{{transform:rotate(180deg)}}
        .unchanged-grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:8px;padding:16px 20px;border-top:1px solid var(--color-border-subtle)}}
        .unchanged-item{{display:flex;align-items:center;gap:8px;padding:8px 12px;background:var(--color-bg);border-radius:var(--radius-sm);font-family:var(--font-mono);font-size:13px;color:var(--color-text-secondary)}}
        .unchanged-icon{{color:var(--color-success);display:flex;flex-shrink:0}}
        .unchanged-icon svg{{width:16px;height:16px}}
        .footer{{margin-top:48px;padding-top:24px;border-top:1px solid var(--color-border-subtle);display:flex;align-items:center;justify-content:space-between;color:var(--color-text-muted);font-size:13px}}
        .footer a{{color:var(--color-text-secondary);text-decoration:none;display:inline-flex;align-items:center;gap:4px;transition:color 0.15s}}
        .footer a:hover{{color:var(--color-text)}}
        .filter-bar{{display:flex;gap:16px;align-items:center;margin-bottom:24px;flex-wrap:wrap;padding:16px 20px;background:var(--color-bg-elevated);border:1px solid var(--color-border-subtle);border-radius:var(--radius-lg)}}
        .search-wrapper{{position:relative;display:flex;align-items:center}}
        .search-wrapper svg{{position:absolute;left:12px;color:var(--color-text-muted);pointer-events:none}}
        .search-input{{padding:10px 12px 10px 36px;border:1px solid var(--color-border);border-radius:var(--radius-md);background:var(--color-bg);color:var(--color-text);font-size:14px;min-width:220px;transition:border-color 0.15s}}
        .search-input:focus{{outline:none;border-color:var(--color-accent)}}
        .filter-buttons{{display:flex;gap:4px}}
        .filter-btn{{padding:8px 14px;border:1px solid var(--color-border);border-radius:var(--radius-sm);background:var(--color-bg);color:var(--color-text-muted);font-size:13px;font-weight:500;cursor:pointer;transition:all 0.15s}}
        .filter-btn:hover{{color:var(--color-text);background:var(--color-bg-hover)}}
        .filter-btn.active{{background:var(--color-accent);color:white;border-color:var(--color-accent)}}
        .sort-select{{padding:10px 14px;border:1px solid var(--color-border);border-radius:var(--radius-md);background:var(--color-bg);color:var(--color-text);font-size:13px;cursor:pointer}}
        .export-btn{{display:flex;align-items:center;gap:6px;padding:10px 16px;border:1px solid var(--color-border);border-radius:var(--radius-md);background:var(--color-bg);color:var(--color-text);font-size:13px;font-weight:500;cursor:pointer;transition:all 0.15s;margin-left:auto}}
        .export-btn:hover{{background:var(--color-bg-hover);border-color:var(--color-accent)}}
        .shot-header-right{{display:flex;align-items:center;gap:12px}}
        .shot-actions{{display:flex;gap:4px}}
        .action-btn{{display:flex;align-items:center;justify-content:center;width:32px;height:32px;border:1px solid var(--color-border);border-radius:var(--radius-sm);background:var(--color-bg);color:var(--color-text-muted);cursor:pointer;transition:all 0.15s}}
        .action-btn:hover{{border-color:var(--color-border)}}
        .action-btn--approve:hover,.action-btn--approve.active{{background:var(--color-success-muted);border-color:var(--color-success);color:var(--color-success)}}
        .action-btn--reject:hover,.action-btn--reject.active{{background:var(--color-error-muted);border-color:var(--color-error);color:var(--color-error)}}
        .shot-card.decision-approved{{border-color:var(--color-success);border-width:2px}}
        .shot-card.decision-rejected{{border-color:var(--color-error);border-width:2px;opacity:0.7}}
        .no-results{{text-align:center;padding:48px 24px;color:var(--color-text-muted)}}
        .no-results h3{{font-size:16px;margin-bottom:8px;color:var(--color-text-secondary)}}
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <a href="https://github.com/emiliodominguez/pixelguard" class="logo" target="_blank">
                <span class="logo-icon">{logo}</span>
                <span class="logo-text">Pixelguard</span>
            </a>
            <div class="header-actions">
                <div class="theme-toggle">
                    <button class="theme-btn" data-theme="light" title="Light theme">{sun_icon}</button>
                    <button class="theme-btn" data-theme="dark" title="Dark theme">{moon_icon}</button>
                    <button class="theme-btn active" data-theme="system" title="System theme">{monitor_icon}</button>
                </div>
            </div>
        </header>
        <div class="status-banner {status_class}">
            <div class="status-icon">{status_icon}</div>
            <div class="status-content">
                <h2>{status_text}</h2>
                <p>{total} screenshots compared</p>
            </div>
        </div>
        <div class="stats-grid">
            <div class="stat-card stat-card--pass"><div class="stat-value">{unchanged}</div><div class="stat-label">{check_icon} Unchanged</div></div>
            <div class="stat-card stat-card--fail"><div class="stat-value">{changed}</div><div class="stat-label">{x_icon} Changed</div></div>
            <div class="stat-card stat-card--new"><div class="stat-value">{added}</div><div class="stat-label">{plus_icon} Added</div></div>
            <div class="stat-card stat-card--removed"><div class="stat-value">{removed}</div><div class="stat-label">{minus_icon} Removed</div></div>
        </div>
        <div class="filter-bar">
            <div class="search-wrapper">
                {search_icon}
                <input type="text" class="search-input" id="search-input" placeholder="Search shots...">
            </div>
            <div class="filter-buttons">
                <button class="filter-btn active" data-filter="all">All</button>
                <button class="filter-btn" data-filter="changed">Changed</button>
                <button class="filter-btn" data-filter="added">Added</button>
                <button class="filter-btn" data-filter="removed">Removed</button>
            </div>
            <select class="sort-select" id="sort-select">
                <option value="name">Sort by Name</option>
                <option value="diff-desc">Sort by Diff % (High to Low)</option>
                <option value="diff-asc">Sort by Diff % (Low to High)</option>
            </select>
            <button class="export-btn" id="export-decisions" title="Export decisions">{download_icon} Export</button>
        </div>
        <main id="shots-container">{changed_html}{added_html}{removed_html}{unchanged_html}</main>
        <footer class="footer">
            <span>Generated by Pixelguard</span>
            <a href="https://github.com/emiliodominguez/pixelguard" target="_blank">View on GitHub {external_link}</a>
        </footer>
    </div>
    <div class="modal-overlay" id="modal">
        <div class="modal-content">
            <span class="modal-label" id="modal-label"></span>
            <button class="modal-close" id="modal-close">{x_icon_large}</button>
            <img class="modal-image" id="modal-image" src="" alt="">
            <span class="modal-hint">Press Escape or click outside to close</span>
        </div>
    </div>
    <script>
    (function(){{const root=document.documentElement,btns=document.querySelectorAll('.theme-btn');function sys(){{return window.matchMedia('(prefers-color-scheme:dark)').matches?'dark':'light'}}function set(t){{root.setAttribute('data-theme',t==='system'?sys():t);btns.forEach(b=>b.classList.toggle('active',b.dataset.theme===t));localStorage.setItem('pg-theme',t)}}set(localStorage.getItem('pg-theme')||'system');btns.forEach(b=>b.addEventListener('click',()=>set(b.dataset.theme)));window.matchMedia('(prefers-color-scheme:dark)').addEventListener('change',()=>{{if(localStorage.getItem('pg-theme')==='system')set('system')}})}}());
    document.querySelectorAll('.shot-card').forEach(c=>{{const tabs=c.querySelectorAll('.tab-btn'),views={{'side-by-side':c.querySelector('.view-side-by-side'),'slider':c.querySelector('.view-slider'),'diff':c.querySelector('.view-diff')}};tabs.forEach(t=>t.addEventListener('click',()=>{{tabs.forEach(x=>x.classList.remove('active'));t.classList.add('active');Object.values(views).forEach(v=>v&&v.classList.remove('active'));const v=views[t.dataset.view];if(v){{v.classList.add('active');if(t.dataset.view==='slider')window.dispatchEvent(new Event('slider-shown'))}}}}))}});
    document.querySelectorAll('.slider-container').forEach(c=>{{const cur=c.querySelector('.slider-current'),h=c.querySelector('.slider-handle'),curImg=cur.querySelector('img');let drag=false;function setImgWidth(){{if(curImg&&c.offsetWidth>0)curImg.style.width=c.offsetWidth+'px'}}function upd(x){{const r=c.getBoundingClientRect(),p=Math.max(0,Math.min(100,((x-r.left)/r.width)*100));cur.style.width=p+'%';h.style.left=p+'%'}}c.addEventListener('mousedown',e=>{{drag=true;upd(e.clientX)}});document.addEventListener('mousemove',e=>{{if(drag)upd(e.clientX)}});document.addEventListener('mouseup',()=>drag=false);c.addEventListener('touchstart',e=>{{drag=true;upd(e.touches[0].clientX)}},{{passive:true}});c.addEventListener('touchmove',e=>{{if(drag){{upd(e.touches[0].clientX);e.preventDefault()}}}},{{passive:false}});c.addEventListener('touchend',()=>drag=false);setImgWidth();if(curImg)curImg.addEventListener('load',setImgWidth);window.addEventListener('resize',setImgWidth);window.addEventListener('slider-shown',()=>setTimeout(setImgWidth,10))}});
    (function(){{const m=document.getElementById('modal'),img=document.getElementById('modal-image'),lbl=document.getElementById('modal-label'),cls=document.getElementById('modal-close');function open(s,l){{img.src=s;lbl.textContent=l;m.classList.add('active');document.body.style.overflow='hidden'}}function close(){{m.classList.remove('active');document.body.style.overflow=''}}document.querySelectorAll('[data-zoomable]').forEach(el=>el.addEventListener('click',()=>open(el.dataset.src,el.dataset.label)));cls.addEventListener('click',close);m.addEventListener('click',e=>{{if(e.target===m)close()}});document.addEventListener('keydown',e=>{{if(e.key==='Escape')close()}})}})();
    (function(){{const searchInput=document.getElementById('search-input'),sortSelect=document.getElementById('sort-select'),filterBtns=document.querySelectorAll('.filter-btn'),container=document.getElementById('shots-container');let currentFilter='all',currentSearch='';function filterShots(){{const cards=document.querySelectorAll('.shot-card');let visibleCount=0;cards.forEach(card=>{{const name=card.dataset.name.toLowerCase(),status=card.dataset.status,matchesSearch=!currentSearch||name.includes(currentSearch.toLowerCase()),matchesFilter=currentFilter==='all'||status===currentFilter;card.style.display=matchesSearch&&matchesFilter?'':'none';if(matchesSearch&&matchesFilter)visibleCount++}});document.querySelectorAll('.section').forEach(sec=>{{const visible=sec.querySelectorAll('.shot-card:not([style*="display: none"])');sec.style.display=visible.length?'':'none'}});const noResults=document.getElementById('no-results');if(noResults)noResults.style.display=visibleCount===0?'block':'none'}}function sortShots(){{const sections=document.querySelectorAll('.section-content');sections.forEach(section=>{{const cards=[...section.querySelectorAll('.shot-card')];const sortVal=sortSelect.value;cards.sort((a,b)=>{{if(sortVal==='diff-desc')return parseFloat(b.dataset.diff)-parseFloat(a.dataset.diff);if(sortVal==='diff-asc')return parseFloat(a.dataset.diff)-parseFloat(b.dataset.diff);return a.dataset.name.localeCompare(b.dataset.name)}});cards.forEach(card=>section.appendChild(card))}})}}searchInput.addEventListener('input',e=>{{currentSearch=e.target.value;filterShots()}});filterBtns.forEach(btn=>btn.addEventListener('click',()=>{{filterBtns.forEach(b=>b.classList.remove('active'));btn.classList.add('active');currentFilter=btn.dataset.filter;filterShots()}}));sortSelect.addEventListener('change',sortShots)}})();
    (function(){{const decisions=JSON.parse(localStorage.getItem('pg-decisions')||'{{}}');function updateUI(){{document.querySelectorAll('.shot-card[data-status="changed"]').forEach(card=>{{const name=card.dataset.name,decision=decisions[name];card.classList.remove('decision-approved','decision-rejected');card.querySelectorAll('.action-btn').forEach(b=>b.classList.remove('active'));if(decision){{card.classList.add('decision-'+decision.action+'d');card.querySelector('.action-btn--'+decision.action)?.classList.add('active')}}}})}}function makeDecision(name,action){{if(decisions[name]&&decisions[name].action===action){{delete decisions[name]}}else{{decisions[name]={{action:action,timestamp:new Date().toISOString(),source:'browser'}}}}localStorage.setItem('pg-decisions',JSON.stringify(decisions));updateUI()}}document.querySelectorAll('.action-btn').forEach(btn=>{{btn.addEventListener('click',e=>{{e.stopPropagation();makeDecision(btn.dataset.shot,btn.dataset.action)}})}});document.getElementById('export-decisions').addEventListener('click',()=>{{const data={{version:'1.0',exportedAt:new Date().toISOString(),decisions:decisions}};const blob=new Blob([JSON.stringify(data,null,2)],{{type:'application/json'}});const url=URL.createObjectURL(blob);const a=document.createElement('a');a.href=url;a.download='pixelguard-decisions.json';a.click();URL.revokeObjectURL(url)}});updateUI()}})();
    </script>
</body>
</html>
"##,
        total = total,
        unchanged = result.unchanged.len(),
        changed = result.changed.len(),
        added = result.added.len(),
        removed = result.removed.len(),
        status_class = status_class,
        status_text = status_text,
        status_icon = status_icon,
        changed_html = changed_html,
        added_html = added_html,
        removed_html = removed_html,
        unchanged_html = unchanged_html,
        logo = icons::LOGO,
        check_icon = icons::CHECK_CIRCLE,
        x_icon = icons::X_CIRCLE,
        plus_icon = icons::PLUS_CIRCLE,
        minus_icon = icons::MINUS_CIRCLE,
        external_link = icons::EXTERNAL_LINK,
        sun_icon = icons::SUN,
        moon_icon = icons::MOON,
        monitor_icon = icons::MONITOR,
        x_icon_large = icons::X,
        search_icon = icons::SEARCH,
        download_icon = icons::DOWNLOAD,
    )
}

/// Escapes HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
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
            }],
            added: vec!["new-component".to_string()],
            removed: vec!["old-component".to_string()],
        };

        let html = generate_html(&result);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Pixelguard"));
        assert!(html.contains("button--primary"));
        assert!(html.contains("card--default"));
        assert!(html.contains("5.50%"));
        assert!(html.contains("new-component"));
        assert!(html.contains("old-component"));
    }

    #[test]
    fn generate_html_shows_viewport_badge() {
        let result = DiffResult {
            unchanged: vec![],
            changed: vec![ChangedShot {
                name: "card--default@mobile".to_string(),
                baseline_path: "baseline/card--default@mobile.png".into(),
                current_path: "current/card--default@mobile.png".into(),
                diff_path: "diff/card--default@mobile.png".into(),
                diff_percentage: 5.5,
                viewport: Some("mobile".to_string()),
            }],
            added: vec![],
            removed: vec![],
        };

        let html = generate_html(&result);

        assert!(html.contains("badge--viewport"));
        assert!(html.contains("mobile"));
    }

    #[test]
    fn generate_html_handles_empty_results() {
        let result = DiffResult {
            unchanged: vec![],
            changed: vec![],
            added: vec![],
            removed: vec![],
        };

        let html = generate_html(&result);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains(">0<"));
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
        };

        let report_path = generate_report(&config, &result, dir.path()).unwrap();

        assert!(report_path.ends_with("report.html"));
        assert!(report_path.exists());
    }
}
