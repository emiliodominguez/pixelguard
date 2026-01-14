//! HTML report generation for visual regression results.
//!
//! This module generates a static HTML report that displays visual diff results
//! with side-by-side comparison of baseline, current, and diff images.

use std::path::Path;

use anyhow::{Context, Result};
use tracing::info;

use crate::config::Config;
use crate::diff::DiffResult;

/// Generates an HTML report from diff results.
///
/// The report includes:
/// - Summary of unchanged, changed, added, and removed shots
/// - Side-by-side comparison for changed shots (baseline, current, diff)
/// - Preview for added shots
/// - List of removed shots
///
/// # Example
///
/// ```rust,no_run
/// use pixelguard_core::{Config, diff_images, generate_report};
///
/// fn example() -> anyhow::Result<()> {
///     let config = Config::load("pixelguard.config.json")?;
///     let diff_result = diff_images(&config, ".")?;
///     generate_report(&config, &diff_result, ".")?;
///     println!("Report generated at .pixelguard/report.html");
///     Ok(())
/// }
/// ```
pub fn generate_report<P: AsRef<Path>>(
    config: &Config,
    result: &DiffResult,
    working_dir: P,
) -> Result<std::path::PathBuf> {
    let working_dir = working_dir.as_ref();
    let report_path = working_dir.join(&config.output_dir).join("report.html");

    let html = generate_html(result);

    std::fs::write(&report_path, html)
        .with_context(|| format!("Failed to write report to {}", report_path.display()))?;

    info!("Report generated at {}", report_path.display());
    Ok(report_path)
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
                format!(
                    r#"
            <div class="shot changed">
                <h3>{name} <span class="badge diff">{diff:.2}% different</span></h3>
                <div class="comparison">
                    <div class="image-container">
                        <h4>Baseline</h4>
                        <img src="baseline/{name}.png" alt="Baseline">
                    </div>
                    <div class="image-container">
                        <h4>Current</h4>
                        <img src="current/{name}.png" alt="Current">
                    </div>
                    <div class="image-container">
                        <h4>Diff</h4>
                        <img src="diff/{name}.png" alt="Diff">
                    </div>
                </div>
            </div>
            "#,
                    name = html_escape(&shot.name),
                    diff = shot.diff_percentage,
                )
            })
            .collect();

        format!(
            r#"
        <section class="changed-section">
            <h2>Changed ({count})</h2>
            {items}
        </section>
        "#,
            count = result.changed.len(),
            items = items,
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
            <div class="shot added">
                <h3>{name} <span class="badge new">New</span></h3>
                <div class="comparison">
                    <div class="image-container">
                        <h4>Current</h4>
                        <img src="current/{name}.png" alt="Current">
                    </div>
                </div>
            </div>
            "#,
                    name = html_escape(name),
                )
            })
            .collect();

        format!(
            r#"
        <section class="added-section">
            <h2>Added ({count})</h2>
            {items}
        </section>
        "#,
            count = result.added.len(),
            items = items,
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
            <div class="shot removed">
                <h3>{name} <span class="badge removed">Removed</span></h3>
                <div class="comparison">
                    <div class="image-container">
                        <h4>Baseline (no longer captured)</h4>
                        <img src="baseline/{name}.png" alt="Baseline">
                    </div>
                </div>
            </div>
            "#,
                    name = html_escape(name),
                )
            })
            .collect();

        format!(
            r#"
        <section class="removed-section">
            <h2>Removed ({count})</h2>
            {items}
        </section>
        "#,
            count = result.removed.len(),
            items = items,
        )
    };

    let unchanged_html = if result.unchanged.is_empty() {
        String::new()
    } else {
        let items: String = result
            .unchanged
            .iter()
            .map(|name| format!("<li>{}</li>", html_escape(name)))
            .collect();

        format!(
            r#"
        <section class="unchanged-section">
            <h2>Unchanged ({count})</h2>
            <details>
                <summary>Click to expand</summary>
                <ul>{items}</ul>
            </details>
        </section>
        "#,
            count = result.unchanged.len(),
            items = items,
        )
    };

    let status_class =
        if result.changed.is_empty() && result.added.is_empty() && result.removed.is_empty() {
            "pass"
        } else {
            "fail"
        };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Pixelguard Visual Regression Report</title>
    <style>
        :root {{
            --bg-primary: #1a1a2e;
            --bg-secondary: #16213e;
            --bg-tertiary: #0f3460;
            --text-primary: #eee;
            --text-secondary: #aaa;
            --accent-green: #4ade80;
            --accent-red: #f87171;
            --accent-yellow: #fbbf24;
            --accent-blue: #60a5fa;
        }}

        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            padding: 2rem;
        }}

        .container {{
            max-width: 1400px;
            margin: 0 auto;
        }}

        header {{
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--bg-tertiary);
        }}

        h1 {{
            font-size: 1.75rem;
            margin-bottom: 1rem;
        }}

        .summary {{
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
        }}

        .stat {{
            background: var(--bg-secondary);
            padding: 1rem 1.5rem;
            border-radius: 8px;
            min-width: 120px;
        }}

        .stat-value {{
            font-size: 2rem;
            font-weight: bold;
        }}

        .stat-label {{
            color: var(--text-secondary);
            font-size: 0.875rem;
        }}

        .stat.pass .stat-value {{
            color: var(--accent-green);
        }}

        .stat.fail .stat-value {{
            color: var(--accent-red);
        }}

        .stat.new .stat-value {{
            color: var(--accent-yellow);
        }}

        .stat.removed .stat-value {{
            color: var(--accent-blue);
        }}

        section {{
            margin-bottom: 2rem;
        }}

        h2 {{
            font-size: 1.25rem;
            margin-bottom: 1rem;
            color: var(--text-primary);
        }}

        .shot {{
            background: var(--bg-secondary);
            border-radius: 8px;
            padding: 1rem;
            margin-bottom: 1rem;
        }}

        .shot h3 {{
            font-size: 1rem;
            margin-bottom: 1rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}

        .badge {{
            font-size: 0.75rem;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-weight: normal;
        }}

        .badge.diff {{
            background: var(--accent-red);
            color: white;
        }}

        .badge.new {{
            background: var(--accent-yellow);
            color: black;
        }}

        .badge.removed {{
            background: var(--accent-blue);
            color: white;
        }}

        .comparison {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1rem;
        }}

        .image-container {{
            background: var(--bg-tertiary);
            border-radius: 4px;
            overflow: hidden;
        }}

        .image-container h4 {{
            padding: 0.5rem;
            font-size: 0.875rem;
            font-weight: normal;
            color: var(--text-secondary);
            background: var(--bg-primary);
        }}

        .image-container img {{
            width: 100%;
            height: auto;
            display: block;
        }}

        details {{
            background: var(--bg-secondary);
            border-radius: 8px;
            padding: 1rem;
        }}

        summary {{
            cursor: pointer;
            color: var(--text-secondary);
        }}

        details ul {{
            margin-top: 1rem;
            padding-left: 1.5rem;
        }}

        details li {{
            color: var(--text-secondary);
            padding: 0.25rem 0;
        }}

        footer {{
            margin-top: 2rem;
            padding-top: 1rem;
            border-top: 1px solid var(--bg-tertiary);
            color: var(--text-secondary);
            font-size: 0.875rem;
        }}

        @media (max-width: 768px) {{
            body {{
                padding: 1rem;
            }}

            .comparison {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Pixelguard Visual Regression Report</h1>
            <div class="summary">
                <div class="stat {status_class}">
                    <div class="stat-value">{total}</div>
                    <div class="stat-label">Total Shots</div>
                </div>
                <div class="stat pass">
                    <div class="stat-value">{unchanged}</div>
                    <div class="stat-label">Unchanged</div>
                </div>
                <div class="stat fail">
                    <div class="stat-value">{changed}</div>
                    <div class="stat-label">Changed</div>
                </div>
                <div class="stat new">
                    <div class="stat-value">{added}</div>
                    <div class="stat-label">Added</div>
                </div>
                <div class="stat removed">
                    <div class="stat-value">{removed}</div>
                    <div class="stat-label">Removed</div>
                </div>
            </div>
        </header>

        <main>
            {changed_html}
            {added_html}
            {removed_html}
            {unchanged_html}
        </main>

        <footer>
            Generated by <a href="https://github.com/pixelguard/pixelguard" style="color: var(--accent-blue);">Pixelguard</a>
        </footer>
    </div>
</body>
</html>
"#,
        total = total,
        unchanged = result.unchanged.len(),
        changed = result.changed.len(),
        added = result.added.len(),
        removed = result.removed.len(),
        status_class = status_class,
        changed_html = changed_html,
        added_html = added_html,
        removed_html = removed_html,
        unchanged_html = unchanged_html,
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
    fn generate_html_handles_empty_results() {
        let result = DiffResult {
            unchanged: vec![],
            changed: vec![],
            added: vec![],
            removed: vec![],
        };

        let html = generate_html(&result);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("0</div>"));
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
