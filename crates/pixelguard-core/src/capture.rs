//! Screenshot capture using Playwright.
//!
//! This module generates and executes a Node.js script that uses Playwright
//! to capture screenshots of configured shots.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::plugins::PluginRegistry;
use crate::storage::Storage;

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
    let script = generate_playwright_script(config, &output_dir, working_dir)?;
    let result = execute_playwright_script(&script, working_dir).await?;

    Ok(result)
}

/// Generates a Playwright script for capturing screenshots.
fn generate_playwright_script(
    config: &Config,
    output_dir: &Path,
    working_dir: &Path,
) -> Result<String> {
    let output_dir_str = output_dir
        .to_str()
        .context("Output directory path is not valid UTF-8")?
        .replace('\\', "/");

    let working_dir_str = working_dir
        .to_str()
        .context("Working directory path is not valid UTF-8")?
        .replace('\\', "/");

    let shots_json = serde_json::to_string(&config.shots)?;

    let viewports = config.effective_viewports();
    let viewports_json = serde_json::to_string(&viewports)?;

    let script = format!(
        r#"
// Resolve playwright from the project's node_modules
const path = require('path');
const playwrightPath = require.resolve('playwright', {{ paths: [{working_dir}] }});
const {{ chromium }} = require(playwrightPath);

const config = {{
    baseUrl: {base_url},
    viewports: {viewports},
    outputDir: {output_dir},
    shots: {shots},
    concurrency: {concurrency}
}};

// Generate filename for a shot+viewport combination
function getFilename(shotName, viewportName) {{
    if (viewportName === 'default') {{
        return `${{shotName}}.png`;
    }}
    return `${{shotName}}@${{viewportName}}.png`;
}}

// Capture a single screenshot at a specific viewport
async function captureOne(browser, shot, viewport, results) {{
    const context = await browser.newContext({{
        viewport: {{ width: viewport.width, height: viewport.height }},
        deviceScaleFactor: 1
    }});
    const page = await context.newPage();

    const filename = getFilename(shot.name, viewport.name);
    const displayName = viewport.name === 'default' ? shot.name : `${{shot.name}}@${{viewport.name}}`;

    try {{
        const url = config.baseUrl + shot.path;
        console.error(`Capturing: ${{displayName}}`);

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

        const screenshotPath = `${{config.outputDir}}/${{filename}}`;
        await page.screenshot({{
            path: screenshotPath,
            fullPage: false
        }});

        results.captured.push({{
            name: displayName,
            path: screenshotPath
        }});
    }} catch (error) {{
        results.failed.push({{
            name: displayName,
            error: error.message
        }});
        console.error(`Failed to capture ${{displayName}}: ${{error.message}}`);
    }} finally {{
        await page.close();
        await context.close();
    }}
}}

// Build list of all shot+viewport combinations
function buildCaptureList(shots, viewports) {{
    const captureList = [];
    for (const shot of shots) {{
        for (const viewport of viewports) {{
            captureList.push({{ shot, viewport }});
        }}
    }}
    return captureList;
}}

// Process items in batches for parallel capture
async function processBatch(browser, items, results) {{
    await Promise.all(items.map(item => captureOne(browser, item.shot, item.viewport, results)));
}}

async function captureScreenshots() {{
    const results = {{ captured: [], failed: [] }};

    const browser = await chromium.launch({{ headless: true }});

    try {{
        // Build list of all shot+viewport combinations
        const captureList = buildCaptureList(config.shots, config.viewports);

        // Split into batches based on concurrency
        const batches = [];
        for (let i = 0; i < captureList.length; i += config.concurrency) {{
            batches.push(captureList.slice(i, i + config.concurrency));
        }}

        // Process batches sequentially, items within batch in parallel
        for (const batch of batches) {{
            await processBatch(browser, batch, results);
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
        working_dir = serde_json::to_string(&working_dir_str)?,
        base_url = serde_json::to_string(&config.base_url)?,
        viewports = viewports_json,
        output_dir = serde_json::to_string(&output_dir_str)?,
        shots = shots_json,
        concurrency = config.concurrency,
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
/// Supports storage plugins for remote baseline storage.
///
/// # Arguments
///
/// * `config` - The configuration
/// * `working_dir` - The working directory
/// * `plugin_registry` - Optional plugin registry for storage plugins
/// * `filter` - Optional list of shot names to update (if None, updates all)
///
/// # Returns
///
/// The number of screenshots that were updated.
pub fn update_baseline<P: AsRef<Path>>(
    config: &Config,
    working_dir: P,
    plugin_registry: Option<&PluginRegistry>,
    filter: Option<&[String]>,
) -> Result<usize> {
    let working_dir = working_dir.as_ref();
    let output_dir = working_dir.join(&config.output_dir);
    let current_dir = output_dir.join("current");

    if !current_dir.exists() {
        anyhow::bail!(
            "No current screenshots found. Run 'pixelguard test' first to capture screenshots."
        );
    }

    // Create storage instance
    let storage = Storage::new(output_dir, working_dir.to_path_buf(), plugin_registry);

    // Get list of current screenshots
    let current_files: Vec<_> = std::fs::read_dir(&current_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "png"))
        .collect();

    // Create baseline directory for local storage
    if !storage.is_remote() {
        std::fs::create_dir_all(working_dir.join(&config.output_dir).join("baseline"))?;
    }

    let mut updated_count = 0;

    // Copy PNG files from current to baseline (optionally filtered)
    for entry in current_files {
        let path = entry.path();

        let Some(filename) = path.file_name() else {
            warn!("Skipping entry with no filename: {:?}", path);
            continue;
        };
        let filename = filename.to_string_lossy();

        let Some(name) = path.file_stem() else {
            warn!("Skipping entry with no file stem: {:?}", path);
            continue;
        };
        let name = name.to_string_lossy();

        // If filter is provided, skip shots that don't match
        if let Some(filter_names) = filter {
            let matches = filter_names.iter().any(|filter_name| {
                // Match exact name or name with viewport suffix (e.g., "button" matches "button@desktop")
                name == filter_name.as_str()
                    || name.starts_with(&format!("{}@", filter_name))
                    || filter_name == name.as_ref()
            });
            if !matches {
                debug!("Skipping {} (not in filter)", name);
                continue;
            }
        }

        let current_path = format!("current/{}", filename);
        let baseline_path = format!("baseline/{}", filename);

        storage.copy(&current_path, &baseline_path)?;
        debug!("Updated baseline: {}", name);
        updated_count += 1;
    }

    if let Some(filter_names) = filter {
        info!(
            "Updated {} baseline screenshot(s) matching filter: {:?}",
            updated_count, filter_names
        );
    } else {
        info!(
            "Baseline updated with {} current screenshots",
            updated_count
        );
    }

    Ok(updated_count)
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

        let script =
            generate_playwright_script(&config, Path::new("/tmp/output"), Path::new("/project"))
                .unwrap();

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
