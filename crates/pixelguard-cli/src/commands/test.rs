//! The `test` command for capturing and comparing screenshots.
//!
//! This command captures screenshots of all configured shots,
//! compares them against the baseline, and generates an HTML report.

use std::net::SocketAddr;
use std::path::Path;

use anyhow::Result;
use axum::Router;
use clap::Args;
use pixelguard_core::{
    capture::{capture_screenshots_in_dir, update_baseline},
    config::Shot,
    diff::{diff_images, has_baseline},
    fetch_storybook_stories, generate_report, Config,
};
use tower_http::services::ServeDir;

/// Arguments for the test command.
#[derive(Args)]
pub struct TestArgs {
    /// Update baseline with current screenshots
    #[arg(long)]
    update: bool,

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

    // Load config
    let mut config = Config::load_or_default(&working_dir)?;

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

    if !args.ci {
        println!("Capturing {} screenshots...", shot_count);
    }

    // Capture screenshots
    let capture_result = capture_screenshots_in_dir(&config, &working_dir).await?;

    if !capture_result.failed.is_empty() {
        eprintln!(
            "\nWarning: {} shots failed to capture:",
            capture_result.failed.len()
        );
        for failed in &capture_result.failed {
            eprintln!("  - {}: {}", failed.name, failed.error);
        }
    }

    // Handle --update flag
    if args.update {
        update_baseline(&config, &working_dir)?;

        if args.ci {
            println!(
                "{{\"status\":\"updated\",\"count\":{}}}",
                capture_result.captured.len()
            );
        } else {
            println!(
                "\n\u{2713} Updated baseline with {} screenshots",
                capture_result.captured.len()
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

    let diff_result = diff_images(&config, &working_dir)?;

    // Generate report
    let report_path = generate_report(&config, &diff_result, &working_dir)?;

    // Output results
    if args.ci {
        let result = serde_json::json!({
            "status": if diff_result.changed.is_empty() && diff_result.added.is_empty() && diff_result.removed.is_empty() {
                "pass"
            } else {
                "fail"
            },
            "unchanged": diff_result.unchanged.len(),
            "changed": diff_result.changed.len(),
            "added": diff_result.added.len(),
            "removed": diff_result.removed.len(),
            "report": report_path.to_string_lossy(),
        });
        println!("{}", serde_json::to_string(&result)?);

        // Exit with code 1 if there are diffs
        if !diff_result.changed.is_empty()
            || !diff_result.added.is_empty()
            || !diff_result.removed.is_empty()
        {
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

        if !diff_result.changed.is_empty()
            || !diff_result.added.is_empty()
            || !diff_result.removed.is_empty()
        {
            println!("\nTo update baseline: pixelguard test --update");
        }
    }

    // Serve the report if requested
    if args.serve && !args.ci {
        let output_dir = working_dir.join(&config.output_dir);
        serve_report(&output_dir, args.port).await?;
    }

    Ok(())
}

/// Serves the report directory on a local HTTP server.
async fn serve_report(output_dir: &Path, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://localhost:{}/report.html", port);

    let app = Router::new().fallback_service(ServeDir::new(output_dir));

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
