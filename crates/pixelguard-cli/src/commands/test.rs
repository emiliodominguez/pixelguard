//! The `test` command for capturing and comparing screenshots.
//!
//! This command captures screenshots of all configured shots,
//! compares them against the baseline, and generates an HTML report.

use anyhow::Result;
use clap::Args;
use pixelguard_core::{
    capture::{capture_screenshots_in_dir, update_baseline},
    diff::{diff_images, has_baseline},
    generate_report, Config,
};

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
}

/// Runs the test command.
pub async fn run(args: TestArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config
    let mut config = Config::load_or_default(&working_dir)?;

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

        println!("\nView report: {}", report_path.display());

        if !diff_result.changed.is_empty()
            || !diff_result.added.is_empty()
            || !diff_result.removed.is_empty()
        {
            println!("\nTo update baseline: pixelguard test --update");
        }
    }

    Ok(())
}
