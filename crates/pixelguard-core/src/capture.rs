//! Screenshot capture using Playwright.
//!
//! This module generates and executes a Node.js script that uses Playwright
//! to capture screenshots of configured shots.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::config::Config;

/// Result of capturing screenshots.
#[derive(Debug, Clone)]
pub struct CaptureResult {
    /// List of successfully captured screenshots
    pub captured: Vec<CapturedShot>,
    /// List of shots that failed to capture
    pub failed: Vec<FailedShot>,
}

/// A successfully captured screenshot.
#[derive(Debug, Clone)]
pub struct CapturedShot {
    /// Name of the shot
    pub name: String,
    /// Path to the captured screenshot file
    pub path: PathBuf,
}

/// A shot that failed to capture.
#[derive(Debug, Clone)]
pub struct FailedShot {
    /// Name of the shot
    pub name: String,
    /// Error message
    pub error: String,
}

/// Captures screenshots for all configured shots.
///
/// This function:
/// 1. Creates the output directory if needed
/// 2. Generates a Playwright script
/// 3. Executes it via Node.js
/// 4. Returns the paths to captured screenshots
///
/// # Requirements
///
/// - Node.js must be installed and available in PATH
/// - Playwright must be installed in the project or globally
///
/// # Example
///
/// ```rust,no_run
/// use pixelguard_core::{Config, capture_screenshots};
///
/// async fn example() -> anyhow::Result<()> {
///     let config = Config::load("pixelguard.config.json")?;
///     let result = capture_screenshots(&config).await?;
///     println!("Captured {} screenshots", result.captured.len());
///     Ok(())
/// }
/// ```
pub async fn capture_screenshots(config: &Config) -> Result<CaptureResult> {
    capture_screenshots_in_dir(config, ".").await
}

/// Captures screenshots for all configured shots, relative to a working directory.
pub async fn capture_screenshots_in_dir<P: AsRef<Path>>(
    config: &Config,
    working_dir: P,
) -> Result<CaptureResult> {
    let working_dir = working_dir.as_ref();
    let output_dir = working_dir.join(&config.output_dir).join("current");

    // Create output directory
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    if config.shots.is_empty() {
        return Ok(CaptureResult {
            captured: Vec::new(),
            failed: Vec::new(),
        });
    }

    // Generate and execute Playwright script
    let script = generate_playwright_script(config, &output_dir)?;
    let result = execute_playwright_script(&script, working_dir).await?;

    Ok(result)
}

/// Generates a Playwright script for capturing screenshots.
fn generate_playwright_script(config: &Config, output_dir: &Path) -> Result<String> {
    let output_dir_str = output_dir
        .to_str()
        .context("Output directory path is not valid UTF-8")?
        .replace('\\', "/");

    let shots_json = serde_json::to_string(&config.shots)?;

    let script = format!(
        r#"
const {{ chromium }} = require('playwright');

const config = {{
    baseUrl: {base_url},
    viewport: {{ width: {width}, height: {height} }},
    outputDir: {output_dir},
    shots: {shots}
}};

async function captureScreenshots() {{
    const results = {{ captured: [], failed: [] }};

    const browser = await chromium.launch({{ headless: true }});
    const context = await browser.newContext({{
        viewport: config.viewport,
        deviceScaleFactor: 1
    }});

    try {{
        for (const shot of config.shots) {{
            const page = await context.newPage();

            try {{
                const url = config.baseUrl + shot.path;
                console.error(`Capturing: ${{shot.name}}`);

                await page.goto(url, {{
                    waitUntil: 'networkidle',
                    timeout: 30000
                }});

                if (shot.waitFor) {{
                    await page.waitForSelector(shot.waitFor, {{ timeout: 10000 }});
                }}

                if (shot.delay) {{
                    await new Promise(resolve => setTimeout(resolve, shot.delay));
                }}

                const screenshotPath = `${{config.outputDir}}/${{shot.name}}.png`;
                await page.screenshot({{
                    path: screenshotPath,
                    fullPage: false
                }});

                results.captured.push({{
                    name: shot.name,
                    path: screenshotPath
                }});
            }} catch (error) {{
                results.failed.push({{
                    name: shot.name,
                    error: error.message
                }});
                console.error(`Failed to capture ${{shot.name}}: ${{error.message}}`);
            }} finally {{
                await page.close();
            }}
        }}
    }} finally {{
        await browser.close();
    }}

    console.log(JSON.stringify(results));
}}

captureScreenshots().catch(error => {{
    console.error('Fatal error:', error.message);
    process.exit(1);
}});
"#,
        base_url = serde_json::to_string(&config.base_url)?,
        width = config.viewport.width,
        height = config.viewport.height,
        output_dir = serde_json::to_string(&output_dir_str)?,
        shots = shots_json,
    );

    Ok(script)
}

/// Executes the Playwright script via Node.js.
async fn execute_playwright_script(script: &str, working_dir: &Path) -> Result<CaptureResult> {
    // Write script to temp file
    let temp_dir = tempfile::tempdir()?;
    let script_path = temp_dir.path().join("capture.js");
    std::fs::write(&script_path, script)?;

    debug!("Executing Playwright script at {:?}", script_path);

    // Execute with Node.js
    let output = Command::new("node")
        .arg(&script_path)
        .current_dir(working_dir)
        .output()
        .context(
            "Failed to execute Node.js. Make sure Node.js is installed and available in PATH.",
        )?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        for line in stderr.lines() {
            if line.starts_with("Capturing:") {
                info!("{}", line);
            } else if line.starts_with("Failed to capture") {
                tracing::warn!("{}", line);
            } else {
                debug!("Playwright: {}", line);
            }
        }
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Cannot find module 'playwright'") {
            anyhow::bail!(
                "Playwright is not installed. Run 'npm install playwright' \
                 or 'npx playwright install' to set it up."
            );
        }
        anyhow::bail!(
            "Playwright script failed with exit code {:?}.\nStderr: {}",
            output.status.code(),
            stderr
        );
    }

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(stdout.trim()).context(
        "Failed to parse Playwright output. This might indicate a Playwright script error.",
    )?;

    let captured: Vec<CapturedShot> = result
        .get("captured")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    Some(CapturedShot {
                        name: v.get("name")?.as_str()?.to_string(),
                        path: PathBuf::from(v.get("path")?.as_str()?),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let failed: Vec<FailedShot> = result
        .get("failed")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    Some(FailedShot {
                        name: v.get("name")?.as_str()?.to_string(),
                        error: v.get("error")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    info!(
        "Captured {} screenshots, {} failed",
        captured.len(),
        failed.len()
    );

    Ok(CaptureResult { captured, failed })
}

/// Copies current screenshots to the baseline directory.
///
/// This is used when updating the baseline with `--update` flag.
pub fn update_baseline<P: AsRef<Path>>(config: &Config, working_dir: P) -> Result<()> {
    let working_dir = working_dir.as_ref();
    let current_dir = working_dir.join(&config.output_dir).join("current");
    let baseline_dir = working_dir.join(&config.output_dir).join("baseline");

    if !current_dir.exists() {
        anyhow::bail!(
            "No current screenshots found. Run 'pixelguard test' first to capture screenshots."
        );
    }

    // Create baseline directory
    std::fs::create_dir_all(&baseline_dir)?;

    // Copy all PNG files from current to baseline
    for entry in std::fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "png") {
            let filename = path.file_name().unwrap();
            let dest = baseline_dir.join(filename);
            std::fs::copy(&path, &dest)?;
            debug!("Updated baseline: {:?}", filename);
        }
    }

    info!("Baseline updated with current screenshots");
    Ok(())
}

/// Returns the paths to baseline and current screenshot directories.
pub fn get_screenshot_dirs<P: AsRef<Path>>(config: &Config, working_dir: P) -> (PathBuf, PathBuf) {
    let working_dir = working_dir.as_ref();
    let baseline_dir = working_dir.join(&config.output_dir).join("baseline");
    let current_dir = working_dir.join(&config.output_dir).join("current");
    (baseline_dir, current_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_script_creates_valid_javascript() {
        let config = Config {
            base_url: "http://localhost:6006".to_string(),
            viewport: crate::config::Viewport {
                width: 1280,
                height: 720,
            },
            shots: vec![crate::config::Shot {
                name: "button--primary".to_string(),
                path: "/iframe.html?id=button--primary".to_string(),
                wait_for: Some("#storybook-root".to_string()),
                delay: Some(100),
            }],
            ..Default::default()
        };

        let script = generate_playwright_script(&config, Path::new("/tmp/output")).unwrap();

        assert!(script.contains("chromium"));
        assert!(script.contains("http://localhost:6006"));
        assert!(script.contains("button--primary"));
        assert!(script.contains("1280"));
        assert!(script.contains("720"));
    }

    #[test]
    fn get_screenshot_dirs_returns_correct_paths() {
        let config = Config {
            output_dir: ".pixelguard".to_string(),
            ..Default::default()
        };

        let (baseline, current) = get_screenshot_dirs(&config, "/project");

        assert_eq!(baseline, PathBuf::from("/project/.pixelguard/baseline"));
        assert_eq!(current, PathBuf::from("/project/.pixelguard/current"));
    }
}
