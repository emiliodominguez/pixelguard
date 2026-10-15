//! Screenshot capture using Playwright.
//!
//! This module runs a static Node.js driver that uses Playwright to capture
//! screenshots of configured shots. Everything the driver needs (URLs, file
//! paths, viewports) is computed in Rust and handed over as a JSON file, so no
//! config value or discovered story id is ever spliced into JavaScript source.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::config::{screenshot_name, validate_name, Config};
use crate::plugins::PluginRegistry;
use crate::process;
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
#[derive(Debug, Clone, Deserialize)]
pub struct FailedShot {
    /// Name of the shot
    pub name: String,
    /// Error message
    pub error: String,
}

/// Captures screenshots for all configured shots.
///
/// This function:
/// 1. Creates the output directory and clears stale screenshots
/// 2. Expands shots × viewports into capture jobs
/// 3. Executes the Playwright driver via Node.js
/// 4. Returns the captured and failed screenshots
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
///
/// Stale PNGs in `current/` are removed first, so a shot that fails to capture
/// can never be compared using an image left over from an earlier run.
pub async fn capture_screenshots_in_dir<P: AsRef<Path>>(
    config: &Config,
    working_dir: P,
) -> Result<CaptureResult> {
    let working_dir = working_dir.as_ref();
    let output_dir = prepare_current_dir(config, working_dir)?;

    if config.shots.is_empty() {
        return Ok(CaptureResult {
            captured: Vec::new(),
            failed: Vec::new(),
        });
    }

    let jobs = build_capture_jobs(config, &output_dir)?;
    execute_playwright(&jobs, config.concurrency, working_dir)
}

/// Creates `{outputDir}/current` and deletes the PNGs a previous run left there.
///
/// Capture plugins must be handed a directory prepared by this function so the
/// stale-screenshot guarantee holds for them too.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or cleaned.
pub fn prepare_current_dir(config: &Config, working_dir: &Path) -> Result<PathBuf> {
    let current_dir = working_dir.join(&config.output_dir).join("current");
    ensure_not_symlink(&current_dir)?;
    std::fs::create_dir_all(&current_dir).with_context(|| {
        format!(
            "Failed to create output directory '{}'",
            current_dir.display()
        )
    })?;

    for entry in std::fs::read_dir(&current_dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "png") {
            std::fs::remove_file(&path).with_context(|| {
                format!("Failed to remove stale screenshot '{}'", path.display())
            })?;
        }
    }

    Ok(current_dir)
}

/// Refuses to use a screenshot directory that is a symbolic link.
///
/// Pixelguard writes into and deletes PNGs from `current/`, `diff/`,
/// `heatmap/` and `baseline/`. A repository could commit one of them as a symlink to an
/// unrelated directory, which would turn that cleanup into deleting files
/// elsewhere on the machine.
///
/// # Errors
///
/// Returns an error if `dir` exists and is a symbolic link.
pub fn ensure_not_symlink(dir: &Path) -> Result<()> {
    if let Ok(metadata) = std::fs::symlink_metadata(dir) {
        if metadata.file_type().is_symlink() {
            anyhow::bail!(
                "❌ '{}' is a symbolic link. Pixelguard writes and deletes screenshots in this \
                 directory, so it refuses to follow links. Replace it with a real directory.",
                dir.display()
            );
        }
    }
    Ok(())
}

/// One screenshot to take: a shot rendered at one viewport.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureJob {
    name: String,
    url: String,
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_for: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delay: Option<u64>,
    width: u32,
    height: u32,
}

/// Expands shots and viewports into concrete jobs with absolute URLs and file paths.
///
/// File names are computed here, from validated names, rather than in the
/// driver, so nothing outside `output_dir` can ever be written.
fn build_capture_jobs(config: &Config, output_dir: &Path) -> Result<Vec<CaptureJob>> {
    config.validate()?;

    let base_url = config.base_url.trim_end_matches('/');
    let viewports = config.effective_viewports();
    let mut jobs = Vec::with_capacity(config.shots.len() * viewports.len());

    for shot in &config.shots {
        for viewport in &viewports {
            let name = screenshot_name(&shot.name, &viewport.name);
            let file = output_dir.join(format!("{}.png", name));
            jobs.push(CaptureJob {
                file: file
                    .to_str()
                    .context("Output directory path is not valid UTF-8")?
                    .to_string(),
                name,
                url: format!("{}{}", base_url, shot.path),
                wait_for: shot.wait_for.clone(),
                delay: shot.delay,
                width: viewport.width,
                height: viewport.height,
            });
        }
    }

    Ok(jobs)
}

/// Static Playwright driver; its inputs arrive through `process.argv` and a JSON file.
const PLAYWRIGHT_SCRIPT: &str = r#"
const fs = require('fs');
const [jobsPath, resultPath, workingDir] = process.argv.slice(2);

function loadChromium() {
    for (const name of ['playwright', '@playwright/test', 'playwright-core']) {
        try {
            const resolved = require.resolve(name, { paths: [workingDir] });
            const chromium = require(resolved).chromium;
            if (chromium) return chromium;
        } catch (error) {
            if (error.code !== 'MODULE_NOT_FOUND') throw error;
        }
    }
    const error = new Error("Cannot find module 'playwright'");
    error.code = 'PIXELGUARD_NO_PLAYWRIGHT';
    throw error;
}

async function captureOne(browser, job, results) {
    let context;
    try {
        console.error(`Capturing: ${job.name}`);
        context = await browser.newContext({
            viewport: { width: job.width, height: job.height },
            deviceScaleFactor: 1
        });
        const page = await context.newPage();
        await page.goto(job.url, { waitUntil: 'networkidle', timeout: 30000 });
        if (job.waitFor) {
            await page.waitForSelector(job.waitFor, { timeout: 10000 });
        }
        if (job.delay) {
            await new Promise(resolve => setTimeout(resolve, job.delay));
        }
        await page.screenshot({ path: job.file, fullPage: false });
        results.captured.push({ name: job.name, path: job.file });
    } catch (error) {
        const message = (error && error.message) || String(error);
        results.failed.push({ name: job.name, error: message.split('\n')[0] });
        console.error(`Failed to capture ${job.name}: ${message}`);
    } finally {
        if (context) {
            await context.close().catch(() => {});
        }
    }
}

async function main() {
    const { jobs, concurrency } = JSON.parse(fs.readFileSync(jobsPath, 'utf8'));
    const results = { captured: [], failed: [] };
    const chromium = loadChromium();
    const browser = await chromium.launch({ headless: true });

    try {
        let next = 0;
        async function worker() {
            while (next < jobs.length) {
                const job = jobs[next++];
                await captureOne(browser, job, results);
            }
        }
        const workers = Math.max(1, Math.min(concurrency, jobs.length));
        await Promise.all(Array.from({ length: workers }, worker));
    } finally {
        await browser.close().catch(() => {});
    }

    fs.writeFileSync(resultPath, JSON.stringify(results));
}

main().catch(error => {
    console.error('Fatal error:', (error && error.message) || error);
    process.exit(error && error.code === 'PIXELGUARD_NO_PLAYWRIGHT' ? 3 : 1);
});
"#;

/// Exit code the driver uses when no Playwright package can be resolved.
const EXIT_NO_PLAYWRIGHT: i32 = 3;

/// Raw result written by the capture driver.
#[derive(Debug, Deserialize)]
struct RawCaptureResult {
    captured: Vec<RawCaptured>,
    failed: Vec<FailedShot>,
}

#[derive(Debug, Deserialize)]
struct RawCaptured {
    name: String,
}

/// Runs the Playwright driver for `jobs` and collects its result.
fn execute_playwright(
    jobs: &[CaptureJob],
    concurrency: usize,
    working_dir: &Path,
) -> Result<CaptureResult> {
    let temp_dir = tempfile::tempdir()?;
    let script_path = temp_dir.path().join("capture.js");
    let jobs_path = temp_dir.path().join("jobs.json");
    let result_path = temp_dir.path().join("result.json");

    std::fs::write(&script_path, PLAYWRIGHT_SCRIPT)?;
    std::fs::write(
        &jobs_path,
        serde_json::to_vec(&serde_json::json!({ "jobs": jobs, "concurrency": concurrency }))?,
    )?;

    let absolute_working_dir = std::path::absolute(working_dir)?;
    let timeout = process::timeout_from_env(capture_timeout(jobs, concurrency));

    debug!("🎬 Executing Playwright script at {:?}", script_path);

    let output = process::run_node_script(
        &script_path,
        &[
            jobs_path.as_os_str(),
            result_path.as_os_str(),
            absolute_working_dir.as_os_str(),
        ],
        working_dir,
        timeout,
    )?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        if line.starts_with("Capturing:") {
            info!("📸 {}", line);
        } else if line.starts_with("Failed to capture") {
            warn!("❌ {}", line);
        } else {
            debug!("Playwright: {}", line);
        }
    }

    if output.status.code() == Some(EXIT_NO_PLAYWRIGHT) {
        anyhow::bail!(
            "❌ Playwright is not installed.\n\n\
             💡 Installation steps:\n  \
             1️⃣ Run: npm install -D playwright\n  \
             2️⃣ Then: npx playwright install chromium\n\n\
             🔗 Docs: https://playwright.dev/docs/intro"
        );
    }

    if !output.status.success() {
        anyhow::bail!(
            "❌ Playwright script failed with exit code {:?}.\n\n\
             🔍 Error details:\n{}\n\n\
             💡 Common causes:\n  \
             • Dev server is not running\n  \
             • Incorrect baseUrl in config\n  \
             • Browser not installed: npx playwright install chromium",
            output.status.code(),
            stderr
        );
    }

    let raw = std::fs::read(&result_path).context(
        "❌ Playwright finished without writing results.\n\n\
         💡 Verify the browser is installed: npx playwright install chromium",
    )?;
    let raw: RawCaptureResult =
        serde_json::from_slice(&raw).context("❌ Failed to parse Playwright capture results.")?;

    let result = reconcile_capture_result(jobs, &raw);

    info!(
        "✅ Captured {} screenshots, {} failed",
        result.captured.len(),
        result.failed.len()
    );

    Ok(result)
}

/// Turns the driver's report into a result covering every job exactly once.
///
/// A job the driver neither captured nor reported as failed, or whose file is
/// missing, is recorded as failed so it can never silently disappear.
fn reconcile_capture_result(jobs: &[CaptureJob], raw: &RawCaptureResult) -> CaptureResult {
    let mut failed: Vec<FailedShot> = Vec::new();
    let mut captured: Vec<CapturedShot> = Vec::new();

    for job in jobs {
        if let Some(failure) = raw.failed.iter().find(|f| f.name == job.name) {
            failed.push(failure.clone());
            continue;
        }
        let reported = raw.captured.iter().any(|c| c.name == job.name);
        let path = PathBuf::from(&job.file);
        if reported && path.is_file() {
            captured.push(CapturedShot {
                name: job.name.clone(),
                path,
            });
        } else {
            failed.push(FailedShot {
                name: job.name.clone(),
                error: "No screenshot was produced".to_string(),
            });
        }
    }

    CaptureResult { captured, failed }
}

/// Generous wall-clock budget for a whole capture run.
///
/// Each job may legitimately spend up to 30s navigating, 10s waiting for its
/// selector and its configured delay; jobs run `concurrency` at a time.
fn capture_timeout(jobs: &[CaptureJob], concurrency: usize) -> Duration {
    let per_job_ms: u64 = jobs.iter().map(|job| 45_000 + job.delay.unwrap_or(0)).sum();
    let parallel_ms = per_job_ms / concurrency.max(1) as u64;
    Duration::from_millis(120_000 + parallel_ms * 2)
}

/// Copies current screenshots to the baseline directory.
///
/// This is used when updating the baseline with `--update` flag.
/// Supports storage plugins for remote baseline storage.
///
/// When `filter` is given, a name matches a screenshot if it is equal to it or
/// to its shot part, so `button` selects `button@mobile` and `button@desktop`.
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
    let current = list_current_screenshots(config, working_dir)?;

    let selected: Vec<String> = current
        .into_iter()
        .filter(|name| match filter {
            None => true,
            Some(filter_names) => filter_names
                .iter()
                .any(|f| name == f || name.starts_with(&format!("{}@", f))),
        })
        .collect();

    copy_to_baseline(config, working_dir, plugin_registry, &selected)?;

    if let Some(filter_names) = filter {
        info!(
            "✅ Updated {} baseline screenshot(s) matching filter: {:?}",
            selected.len(),
            filter_names
        );
    } else {
        info!(
            "✅ Baseline updated with {} current screenshots",
            selected.len()
        );
    }

    Ok(selected.len())
}

/// Copies exactly the named screenshots from `current/` to the baseline.
///
/// Unlike [`update_baseline`], names are never expanded to other viewports:
/// approving `button@mobile` must not also approve `button@desktop`. Names
/// without a current screenshot are skipped.
///
/// # Returns
///
/// The names that were actually copied.
pub fn update_baseline_exact<P: AsRef<Path>>(
    config: &Config,
    working_dir: P,
    plugin_registry: Option<&PluginRegistry>,
    names: &[String],
) -> Result<Vec<String>> {
    let working_dir = working_dir.as_ref();
    let current: HashSet<String> = list_current_screenshots(config, working_dir)?
        .into_iter()
        .collect();

    let mut selected = Vec::new();
    for name in names {
        if validate_screenshot_name(name).is_err() {
            warn!("⚠️  Ignoring invalid screenshot name: {:?}", name);
            continue;
        }
        if current.contains(name) && !selected.contains(name) {
            selected.push(name.clone());
        }
    }

    copy_to_baseline(config, working_dir, plugin_registry, &selected)?;
    Ok(selected)
}

/// Deletes baseline screenshots that are not in `keep`.
///
/// Call this only after a complete, unfiltered, failure-free update; otherwise
/// a baseline for a shot that merely failed to capture would be thrown away.
///
/// # Returns
///
/// The names of the baselines that were deleted.
pub fn prune_baseline<P: AsRef<Path>>(
    config: &Config,
    working_dir: P,
    plugin_registry: Option<&PluginRegistry>,
    keep: &HashSet<String>,
) -> Result<Vec<String>> {
    let working_dir = working_dir.as_ref();
    let output_dir = working_dir.join(&config.output_dir);
    ensure_not_symlink(&output_dir.join("baseline"))?;
    let storage = Storage::new(output_dir, working_dir.to_path_buf(), plugin_registry);

    let mut pruned = Vec::new();
    for file in storage.list("baseline")? {
        let Some(name) = file.strip_suffix(".png") else {
            continue;
        };
        if keep.contains(name) || validate_screenshot_name(name).is_err() {
            continue;
        }
        storage.delete(&format!("baseline/{}", file))?;
        pruned.push(name.to_string());
    }

    pruned.sort();
    Ok(pruned)
}

/// Validates a screenshot name of the form `shot` or `shot@viewport`.
///
/// # Errors
///
/// Returns an error if either part is not a valid name.
pub fn validate_screenshot_name(name: &str) -> Result<()> {
    match name.split_once('@') {
        Some((shot, viewport)) => {
            validate_name("shot", shot)?;
            validate_name("viewport", viewport)
        }
        None => validate_name("shot", name),
    }
}

/// Lists screenshot names (file stems) present in `current/`.
fn list_current_screenshots(config: &Config, working_dir: &Path) -> Result<Vec<String>> {
    let current_dir = working_dir.join(&config.output_dir).join("current");

    if !current_dir.exists() {
        anyhow::bail!(
            "❌ No current screenshots found.\n\n\
             💡 Solution: Run 'pixelguard test' first to capture screenshots.\n\n\
             📍 Expected location: {}",
            current_dir.display()
        );
    }

    let mut names = Vec::new();
    for entry in std::fs::read_dir(&current_dir)? {
        let path = entry?.path();
        if !path.is_file() || path.extension().is_none_or(|ext| ext != "png") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            warn!("⚠️  Skipping screenshot with a non UTF-8 name: {:?}", path);
            continue;
        };
        names.push(stem.to_string());
    }
    names.sort();
    Ok(names)
}

/// Copies `current/{name}.png` to `baseline/{name}.png` for each name.
fn copy_to_baseline(
    config: &Config,
    working_dir: &Path,
    plugin_registry: Option<&PluginRegistry>,
    names: &[String],
) -> Result<()> {
    let output_dir = working_dir.join(&config.output_dir);
    let storage = Storage::new(
        output_dir.clone(),
        working_dir.to_path_buf(),
        plugin_registry,
    );

    if !storage.is_remote() {
        ensure_not_symlink(&output_dir.join("baseline"))?;
        std::fs::create_dir_all(output_dir.join("baseline"))?;
    }

    for name in names {
        if storage.is_remote() {
            let data = std::fs::read(output_dir.join("current").join(format!("{}.png", name)))?;
            storage.write(&format!("baseline/{}.png", name), &data)?;
        } else {
            storage.copy(
                &format!("current/{}.png", name),
                &format!("baseline/{}.png", name),
            )?;
        }
        debug!("✅ Updated baseline: {}", name);
    }

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
    use crate::config::{NamedViewport, Shot};

    fn shot(name: &str, path: &str) -> Shot {
        Shot {
            name: name.to_string(),
            path: path.to_string(),
            wait_for: None,
            delay: None,
        }
    }

    #[test]
    fn capture_jobs_use_rust_computed_paths_inside_output_dir() {
        let config = Config {
            base_url: "http://localhost:6006/".to_string(),
            shots: vec![shot("button--primary", "/iframe.html?id=button--primary")],
            viewports: vec![NamedViewport {
                name: "mobile".to_string(),
                width: 375,
                height: 667,
            }],
            ..Default::default()
        };

        let jobs = build_capture_jobs(&config, Path::new("/out/current")).unwrap();

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "button--primary@mobile");
        assert_eq!(
            jobs[0].url,
            "http://localhost:6006/iframe.html?id=button--primary"
        );
        assert_eq!(
            PathBuf::from(&jobs[0].file),
            Path::new("/out/current").join("button--primary@mobile.png")
        );
    }

    #[test]
    fn capture_jobs_reject_path_traversal_in_shot_names() {
        let config = Config {
            base_url: "http://localhost:6006".to_string(),
            shots: vec![shot("../../../escaped", "/a.html")],
            ..Default::default()
        };

        assert!(build_capture_jobs(&config, Path::new("/out/current")).is_err());
    }

    #[test]
    fn driver_script_contains_no_template_placeholders() {
        assert!(!PLAYWRIGHT_SCRIPT.contains("{{"));
        assert!(PLAYWRIGHT_SCRIPT.contains("process.argv"));
    }

    #[test]
    fn prepare_current_dir_removes_stale_screenshots_only() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let current = dir.path().join(".pixelguard/current");
        std::fs::create_dir_all(&current).unwrap();
        std::fs::write(current.join("stale.png"), b"old").unwrap();
        std::fs::write(current.join("notes.txt"), b"keep").unwrap();

        prepare_current_dir(&config, dir.path()).unwrap();

        assert!(!current.join("stale.png").exists());
        assert!(current.join("notes.txt").exists());
    }

    #[test]
    fn reconcile_marks_unreported_jobs_as_failed() {
        let dir = tempfile::tempdir().unwrap();
        let captured_file = dir.path().join("a.png");
        std::fs::write(&captured_file, b"png").unwrap();
        let jobs = vec![
            CaptureJob {
                name: "a".to_string(),
                url: String::new(),
                file: captured_file.to_string_lossy().to_string(),
                wait_for: None,
                delay: None,
                width: 1,
                height: 1,
            },
            CaptureJob {
                name: "b".to_string(),
                url: String::new(),
                file: dir.path().join("b.png").to_string_lossy().to_string(),
                wait_for: None,
                delay: None,
                width: 1,
                height: 1,
            },
        ];
        let raw = RawCaptureResult {
            captured: vec![
                RawCaptured {
                    name: "a".to_string(),
                },
                RawCaptured {
                    name: "b".to_string(),
                },
            ],
            failed: Vec::new(),
        };

        let result = reconcile_capture_result(&jobs, &raw);

        assert_eq!(result.captured.len(), 1);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].name, "b");
    }

    #[test]
    fn update_baseline_exact_does_not_expand_to_other_viewports() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let current = dir.path().join(".pixelguard/current");
        std::fs::create_dir_all(&current).unwrap();
        std::fs::write(current.join("card@mobile.png"), b"m").unwrap();
        std::fs::write(current.join("card@desktop.png"), b"d").unwrap();

        let copied = update_baseline_exact(
            &config,
            dir.path(),
            None,
            &["card@mobile".to_string(), "../evil".to_string()],
        )
        .unwrap();

        assert_eq!(copied, vec!["card@mobile".to_string()]);
        let baseline = dir.path().join(".pixelguard/baseline");
        assert!(baseline.join("card@mobile.png").exists());
        assert!(!baseline.join("card@desktop.png").exists());
    }

    #[test]
    fn prune_baseline_removes_only_unkept_screenshots() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let baseline = dir.path().join(".pixelguard/baseline");
        std::fs::create_dir_all(&baseline).unwrap();
        std::fs::write(baseline.join("kept.png"), b"k").unwrap();
        std::fs::write(baseline.join("gone.png"), b"g").unwrap();

        let keep: HashSet<String> = ["kept".to_string()].into_iter().collect();
        let pruned = prune_baseline(&config, dir.path(), None, &keep).unwrap();

        assert_eq!(pruned, vec!["gone".to_string()]);
        assert!(baseline.join("kept.png").exists());
        assert!(!baseline.join("gone.png").exists());
    }

    #[cfg(unix)]
    #[test]
    fn prepare_current_dir_refuses_symlinked_directories() {
        let dir = tempfile::tempdir().unwrap();
        let victim = tempfile::tempdir().unwrap();
        std::fs::write(victim.path().join("precious.png"), b"keep").unwrap();
        std::fs::create_dir_all(dir.path().join(".pixelguard")).unwrap();
        std::os::unix::fs::symlink(victim.path(), dir.path().join(".pixelguard/current")).unwrap();

        assert!(prepare_current_dir(&Config::default(), dir.path()).is_err());
        assert!(victim.path().join("precious.png").exists());
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
