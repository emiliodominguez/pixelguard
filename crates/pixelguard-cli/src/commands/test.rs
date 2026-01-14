//! The `test` command for capturing and comparing screenshots.
//!
//! This command captures screenshots of all configured shots,
//! compares them against the baseline, and generates an HTML report.
//! Supports plugins for capture, diff, report, and notification.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Args;
use pixelguard_core::{
    capture::{capture_screenshots_in_dir, update_baseline},
    config::Shot,
    diff::{diff_images, has_baseline, DiffResult},
    fetch_storybook_stories, generate_report,
    plugins::{
        self, CaptureInput, CaptureOutput, CaptureShot, CaptureViewport, NotifierInput,
        PluginCategory, PluginRegistry, ReporterChangedShot, ReporterConfig, ReporterDiffResult,
        ReporterInput,
    },
    Config,
};
use tower_http::services::ServeDir;
use tracing::info;

/// Arguments for the test command.
#[derive(Args)]
pub struct TestArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short)]
    config: Option<String>,

    /// Update baseline with current screenshots
    #[arg(long)]
    update: bool,

    /// Update only specific shots (comma-separated names, implies --update)
    #[arg(long, value_delimiter = ',')]
    update_only: Option<Vec<String>>,

    /// CI mode: machine-readable output, exit code 1 on diffs
    #[arg(long)]
    ci: bool,

    /// Only test shots matching this pattern
    #[arg(long)]
    filter: Option<String>,

    /// Show detailed progress
    #[arg(long)]
    verbose: bool,

    /// Serve the report in a local web server after completion
    #[arg(long)]
    serve: bool,

    /// Port for the local web server (default: 3333)
    #[arg(long, default_value = "3333")]
    port: u16,
}

/// Runs the test command.
pub async fn run(args: TestArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config from custom path or default
    let mut config = super::load_config(&working_dir, args.config.as_deref())?;

    // Initialize plugins
    let plugin_registry = plugins::init_plugins(&config, &working_dir)?;
    if !plugin_registry.is_empty() && !args.ci {
        info!("Loaded {} plugin(s)", plugin_registry.len());
    }

    // Dynamically discover shots if source is storybook and no shots configured
    if config.source == "storybook" && !config.base_url.is_empty() {
        let discovered = discover_shots(&config).await?;
        if discovered.is_empty() {
            anyhow::bail!(
                "Could not discover any stories from Storybook at {}. \
                 Make sure Storybook is running.",
                config.base_url
            );
        }
        // Merge with any overrides from config
        config.shots = merge_shots(discovered, &config.shots);
    }

    if config.shots.is_empty() {
        anyhow::bail!(
            "No shots configured. Run 'pixelguard init' first, \
             or add shots to pixelguard.config.json manually."
        );
    }

    // Apply filter if specified
    if let Some(pattern) = &args.filter {
        let original_count = config.shots.len();
        config.shots.retain(|shot| shot.name.contains(pattern));

        if config.shots.is_empty() {
            anyhow::bail!(
                "No shots match filter '{}'. {} shots were filtered out.",
                pattern,
                original_count
            );
        }

        if !args.ci {
            println!(
                "Filtered to {} of {} shots matching '{}'",
                config.shots.len(),
                original_count,
                pattern
            );
        }
    }

    let shot_count = config.shots.len();

    // Early validation: Check Node.js and Playwright before capturing
    // Skip if using a capture plugin (it handles its own dependencies)
    if !plugin_registry.has_override(PluginCategory::Capture) {
        validate_capture_environment()?;
    }

    if !args.ci {
        println!("Capturing {} screenshots...", shot_count);
    }

    // Capture screenshots (using plugin if available)
    let capture_result = capture_with_plugin(&config, &working_dir, &plugin_registry).await?;

    if !capture_result.failed.is_empty() {
        eprintln!(
            "\nWarning: {} shots failed to capture:",
            capture_result.failed.len()
        );
        for failed in &capture_result.failed {
            eprintln!("  - {}: {}", failed.name, failed.error);
        }
    }

    // Handle --update or --update-only flag
    let should_update = args.update || args.update_only.is_some();

    if should_update {
        let filter = args.update_only.as_deref();
        let updated_count = update_baseline(&config, &working_dir, Some(&plugin_registry), filter)?;

        if args.ci {
            println!("{{\"status\":\"updated\",\"count\":{}}}", updated_count);
        } else if let Some(names) = &args.update_only {
            println!(
                "\n\u{2713} Updated {} baseline(s): {}",
                updated_count,
                names.join(", ")
            );
        } else {
            println!(
                "\n\u{2713} Updated baseline with {} screenshots",
                updated_count
            );
        }

        return Ok(());
    }

    // Check if baseline exists
    if !has_baseline(&config, &working_dir) {
        if args.ci {
            println!(
                "{{\"status\":\"no_baseline\",\"captured\":{}}}",
                capture_result.captured.len()
            );
            // In CI mode, no baseline means first run - exit successfully
            // User should run with --update to create baseline
            return Ok(());
        }

        println!(
            "\nNo baseline found. This appears to be your first run.\n\
             Run 'pixelguard test --update' to create the baseline."
        );
        return Ok(());
    }

    // Compare against baseline
    if !args.ci {
        println!("Comparing against baseline...");
    }

    let diff_result = diff_images(&config, &working_dir, Some(&plugin_registry))?;

    // Generate built-in report
    let report_path = generate_report(&config, &diff_result, &working_dir)?;

    // Run additional reporter plugins
    run_reporter_plugins(&config, &diff_result, &working_dir, &plugin_registry)?;

    // Output results
    if args.ci {
        let result = serde_json::json!({
            "status": if diff_result.has_changes() { "fail" } else { "pass" },
            "unchanged": diff_result.unchanged.len(),
            "changed": diff_result.changed.len(),
            "added": diff_result.added.len(),
            "removed": diff_result.removed.len(),
            "report": report_path.to_string_lossy(),
        });
        println!("{}", serde_json::to_string(&result)?);

        // Exit with code 1 if there are diffs
        if diff_result.has_changes() {
            std::process::exit(1);
        }
    } else {
        println!();

        if !diff_result.unchanged.is_empty() {
            println!("\u{2713} {} unchanged", diff_result.unchanged.len());
        }

        if !diff_result.changed.is_empty() {
            println!("\u{2717} {} changed", diff_result.changed.len());
            for shot in &diff_result.changed {
                println!("    {} ({:.2}% different)", shot.name, shot.diff_percentage);
            }
        }

        if !diff_result.added.is_empty() {
            println!("+ {} added", diff_result.added.len());
            for name in &diff_result.added {
                println!("    {}", name);
            }
        }

        if !diff_result.removed.is_empty() {
            println!("- {} removed", diff_result.removed.len());
            for name in &diff_result.removed {
                println!("    {}", name);
            }
        }

        if args.serve {
            println!("\nStarting server...");
        } else {
            println!("\nView report: {}", report_path.display());
        }

        if diff_result.has_changes() {
            println!("\nTo update baseline: pixelguard test --update");
        }
    }

    // Run notifier plugins
    run_notifier_plugins(
        &diff_result,
        Some(&report_path),
        args.ci,
        &working_dir,
        &plugin_registry,
    )?;

    // Serve the report if requested
    if args.serve && !args.ci {
        let output_dir = working_dir.join(&config.output_dir);
        serve_report(&output_dir, args.port).await?;
    }

    Ok(())
}

/// Captures screenshots using a plugin if available, otherwise uses built-in capture.
async fn capture_with_plugin(
    config: &Config,
    working_dir: &Path,
    registry: &PluginRegistry,
) -> Result<pixelguard_core::capture::CaptureResult> {
    if let Some(plugin) = registry.get(PluginCategory::Capture) {
        info!("Using capture plugin: {}", plugin.name());

        let output_dir = working_dir
            .join(&config.output_dir)
            .join("current")
            .to_string_lossy()
            .to_string();

        // Create output directory
        std::fs::create_dir_all(working_dir.join(&config.output_dir).join("current"))?;

        let input = CaptureInput {
            shots: config
                .shots
                .iter()
                .map(|s| CaptureShot {
                    name: s.name.clone(),
                    path: s.path.clone(),
                    wait_for: s.wait_for.clone(),
                    delay: s.delay,
                })
                .collect(),
            base_url: config.base_url.clone(),
            viewport: CaptureViewport {
                width: config.viewport.width,
                height: config.viewport.height,
            },
            output_dir,
            options: serde_json::json!({}),
        };

        let output: CaptureOutput =
            plugins::executor::execute_hook(plugin, "capture", &input, working_dir)?;

        Ok(pixelguard_core::capture::CaptureResult {
            captured: output
                .captured
                .into_iter()
                .map(|s| pixelguard_core::capture::CapturedShot {
                    name: s.name,
                    path: std::path::PathBuf::from(s.path),
                })
                .collect(),
            failed: output
                .failed
                .into_iter()
                .map(|s| pixelguard_core::capture::FailedShot {
                    name: s.name,
                    error: s.error,
                })
                .collect(),
        })
    } else {
        // Use built-in capture
        capture_screenshots_in_dir(config, working_dir).await
    }
}

/// Runs all registered reporter plugins.
fn run_reporter_plugins(
    config: &Config,
    diff_result: &DiffResult,
    working_dir: &Path,
    registry: &PluginRegistry,
) -> Result<()> {
    let reporters = registry.reporters();
    if reporters.is_empty() {
        return Ok(());
    }

    let output_dir = working_dir
        .join(&config.output_dir)
        .to_string_lossy()
        .to_string();

    let input = ReporterInput {
        result: convert_diff_result(diff_result),
        config: ReporterConfig {
            source: config.source.clone(),
            base_url: config.base_url.clone(),
            threshold: config.threshold,
        },
        output_dir,
        options: serde_json::json!({}),
    };

    for plugin in reporters {
        info!("Running reporter plugin: {}", plugin.name());
        let _output: serde_json::Value =
            plugins::executor::execute_hook(plugin, "generate", &input, working_dir)?;
    }

    Ok(())
}

/// Runs all registered notifier plugins.
fn run_notifier_plugins(
    diff_result: &DiffResult,
    report_path: Option<&Path>,
    ci_mode: bool,
    working_dir: &Path,
    registry: &PluginRegistry,
) -> Result<()> {
    let notifiers = registry.notifiers();
    if notifiers.is_empty() {
        return Ok(());
    }

    let input = NotifierInput {
        result: convert_diff_result(diff_result),
        report_path: report_path.map(|p| p.to_string_lossy().to_string()),
        report_url: None,
        ci_mode,
        options: serde_json::json!({}),
    };

    for plugin in notifiers {
        info!("Running notifier plugin: {}", plugin.name());
        plugins::executor::execute_hook_void(plugin, "notify", &input, working_dir)?;
    }

    Ok(())
}

/// Converts DiffResult to the plugin-compatible format.
fn convert_diff_result(diff_result: &DiffResult) -> ReporterDiffResult {
    ReporterDiffResult {
        unchanged: diff_result.unchanged.clone(),
        changed: diff_result
            .changed
            .iter()
            .map(|c| ReporterChangedShot {
                name: c.name.clone(),
                baseline_path: c.baseline_path.to_string_lossy().to_string(),
                current_path: c.current_path.to_string_lossy().to_string(),
                diff_path: c.diff_path.to_string_lossy().to_string(),
                diff_percentage: c.diff_percentage,
            })
            .collect(),
        added: diff_result.added.clone(),
        removed: diff_result.removed.clone(),
    }
}

/// State for the serve endpoints.
#[derive(Clone)]
struct ServeState {
    output_dir: PathBuf,
}

/// Serves the report directory on a local HTTP server with save endpoint.
async fn serve_report(output_dir: &Path, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://localhost:{}/report.html", port);

    let state = Arc::new(ServeState {
        output_dir: output_dir.to_path_buf(),
    });

    let app = Router::new()
        .route("/api/decisions", post(save_decisions))
        .route("/api/decisions", get(load_decisions))
        .with_state(state)
        .fallback_service(ServeDir::new(output_dir));

    println!("Serving report at: {}", url);
    println!("Press Ctrl+C to stop the server\n");

    // Open browser
    if let Err(e) = open::that(&url) {
        eprintln!("Could not open browser: {}. Open {} manually.", e, url);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler to save decisions to the output directory.
async fn save_decisions(
    State(state): State<Arc<ServeState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let decisions_path = state.output_dir.join("decisions.json");

    let content = serde_json::to_string_pretty(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    std::fs::write(&decisions_path, content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Handler to load decisions from the output directory.
async fn load_decisions(
    State(state): State<Arc<ServeState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let decisions_path = state.output_dir.join("decisions.json");

    if !decisions_path.exists() {
        return Ok(Json(serde_json::json!({
            "version": "1.0",
            "exportedAt": "",
            "decisions": {}
        })));
    }

    let content = std::fs::read_to_string(&decisions_path).map_err(|_| StatusCode::NOT_FOUND)?;

    let data: serde_json::Value =
        serde_json::from_str(&content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(data))
}

/// Discovers shots dynamically from the source (e.g., Storybook).
async fn discover_shots(config: &Config) -> Result<Vec<Shot>> {
    match config.source.as_str() {
        "storybook" => {
            let stories = fetch_storybook_stories(&config.base_url)
                .await
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Failed to fetch stories from Storybook at {}. Is it running?",
                        config.base_url
                    )
                })?;

            // Apply include/exclude filters
            let filtered: Vec<Shot> = stories
                .into_iter()
                .filter(|shot| matches_patterns(&shot.name, &config.include, &config.exclude))
                .collect();

            Ok(filtered)
        }
        _ => Ok(Vec::new()),
    }
}

/// Checks if a name matches include patterns and doesn't match exclude patterns.
fn matches_patterns(name: &str, include: &[String], exclude: &[String]) -> bool {
    // Check exclude first
    for pattern in exclude {
        if glob_match(pattern, name) {
            return false;
        }
    }

    // Then check include
    for pattern in include {
        if glob_match(pattern, name) {
            return true;
        }
    }

    // If no include patterns, include everything
    include.is_empty()
}

/// Simple glob matching (supports * and **).
fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern == "**/*" || pattern == "*" {
        return true;
    }

    // Convert glob to regex-like matching
    let pattern = pattern.replace("**", ".*").replace('*', "[^/]*");
    regex::Regex::new(&format!("^{}$", pattern))
        .map(|re| re.is_match(name))
        .unwrap_or(false)
}

/// Merges discovered shots with config overrides.
///
/// Config shots can provide custom waitFor, delay, or completely override a shot.
fn merge_shots(discovered: Vec<Shot>, overrides: &[Shot]) -> Vec<Shot> {
    discovered
        .into_iter()
        .map(|mut shot| {
            // Check if there's an override for this shot
            if let Some(override_shot) = overrides.iter().find(|o| o.name == shot.name) {
                // Apply overrides
                if override_shot.wait_for.is_some() {
                    shot.wait_for = override_shot.wait_for.clone();
                }
                if override_shot.delay.is_some() {
                    shot.delay = override_shot.delay;
                }
                // Path override is intentionally not applied - use discovered path
            }
            shot
        })
        .collect()
}

/// Validates that Node.js and Playwright are available before attempting capture.
///
/// This provides a clear error message upfront rather than failing mid-capture
/// with a confusing "command not found" error.
fn validate_capture_environment() -> Result<()> {
    // Check Node.js
    Command::new("node").arg("--version").output().context(
        "Node.js is required for screenshot capture but was not found. \
             Please install Node.js from https://nodejs.org",
    )?;

    // Check Playwright (via npx or node_modules)
    let npx_check = Command::new("npx")
        .args(["playwright", "--version"])
        .output();

    let has_playwright = npx_check.map(|o| o.status.success()).unwrap_or(false)
        || Path::new("node_modules/@playwright/test").exists()
        || Path::new("node_modules/playwright").exists();

    if !has_playwright {
        anyhow::bail!(
            "Playwright is required for screenshot capture but was not found.\n\
             Install it with: npm install -D @playwright/test\n\
             Then run: npx playwright install chromium"
        );
    }

    Ok(())
}
