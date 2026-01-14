//! The `review` command for interactively reviewing visual diffs.
//!
//! This command reads the results.json file and prompts the user to
//! approve or reject each changed shot interactively in the terminal.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use pixelguard_core::{capture::update_baseline, plugins};
use serde::Deserialize;
use tracing::info;

/// Arguments for the review command.
#[derive(Args)]
pub struct ReviewArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short)]
    config: Option<String>,

    /// Path to results.json file (default: .pixelguard/results.json)
    #[arg(long)]
    results: Option<PathBuf>,

    /// Open diff images during review
    #[arg(long)]
    open_diff: bool,
}

/// Structure of the results.json file.
#[derive(Debug, Deserialize)]
struct ResultsJson {
    /// Summary statistics
    summary: ResultsSummary,
    /// Detailed results
    results: ResultsDetail,
}

/// Summary of results.
#[derive(Debug, Deserialize)]
struct ResultsSummary {
    /// Whether all tests passed
    passed: bool,
    /// Number of changed shots
    changed: usize,
}

/// Detailed results by category.
#[derive(Debug, Deserialize)]
struct ResultsDetail {
    /// Changed shots with diff info
    changed: Vec<ChangedShot>,
}

/// A changed shot.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangedShot {
    /// Name of the shot
    name: String,
    /// Percentage of pixels that differ
    diff_percentage: f64,
    /// Viewport name (if multi-viewport)
    #[allow(dead_code)]
    viewport: Option<String>,
    /// Path to diff image
    diff_path: String,
}

/// Action the user can take for each shot.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ReviewAction {
    Approve,
    Reject,
    Skip,
    ViewDiff,
    Quit,
}

impl std::fmt::Display for ReviewAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewAction::Approve => write!(f, "Approve (update baseline)"),
            ReviewAction::Reject => write!(f, "Reject (keep baseline)"),
            ReviewAction::Skip => write!(f, "Skip (decide later)"),
            ReviewAction::ViewDiff => write!(f, "View diff image"),
            ReviewAction::Quit => write!(f, "Quit review"),
        }
    }
}

/// Runs the review command.
pub async fn run(args: ReviewArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config
    let config = super::load_config(&working_dir, args.config.as_deref())?;

    // Determine results file path
    let results_path = args
        .results
        .unwrap_or_else(|| working_dir.join(&config.output_dir).join("results.json"));

    // Load results.json
    let results_content = std::fs::read_to_string(&results_path).with_context(|| {
        format!(
            "Could not read results file at '{}'. \
             Run 'pixelguard test' first to generate results.",
            results_path.display()
        )
    })?;

    let results: ResultsJson = serde_json::from_str(&results_content).with_context(|| {
        format!(
            "Invalid results file format in '{}'. \
             Try running 'pixelguard test' again.",
            results_path.display()
        )
    })?;

    // Check if there are changes to review
    if results.summary.passed || results.summary.changed == 0 {
        println!("\u{2713} All tests passed! No changes to review.");
        return Ok(());
    }

    let changed = &results.results.changed;
    println!(
        "\nFound {} changed screenshot(s) to review\n",
        changed.len()
    );

    let theme = ColorfulTheme::default();
    let mut approved: Vec<String> = Vec::new();
    let mut rejected: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    let output_dir = working_dir.join(&config.output_dir);

    // Review each changed shot
    for (i, shot) in changed.iter().enumerate() {
        loop {
            println!(
                "\n[{}/{}] {} ({:.2}% different)",
                i + 1,
                changed.len(),
                shot.name,
                shot.diff_percentage
            );

            let actions = vec![
                ReviewAction::Approve,
                ReviewAction::Reject,
                ReviewAction::Skip,
                ReviewAction::ViewDiff,
                ReviewAction::Quit,
            ];

            let selection = Select::with_theme(&theme)
                .with_prompt("What would you like to do?")
                .items(&actions)
                .default(0)
                .interact()?;

            match actions[selection] {
                ReviewAction::Approve => {
                    approved.push(shot.name.clone());
                    println!("  \u{2713} Approved: {}", shot.name);
                    break;
                }
                ReviewAction::Reject => {
                    rejected.push(shot.name.clone());
                    println!("  \u{2717} Rejected: {}", shot.name);
                    break;
                }
                ReviewAction::Skip => {
                    skipped.push(shot.name.clone());
                    println!("  \u{2192} Skipped: {}", shot.name);
                    break;
                }
                ReviewAction::ViewDiff => {
                    let diff_path = output_dir.join(&shot.diff_path);
                    if diff_path.exists() {
                        if let Err(e) = open::that(&diff_path) {
                            eprintln!("Could not open diff image: {}", e);
                        } else {
                            println!("  Opened diff image");
                        }
                    } else {
                        eprintln!("  Diff image not found at {}", diff_path.display());
                    }
                    // Continue the loop to prompt again
                }
                ReviewAction::Quit => {
                    println!("\nQuitting review...");
                    // Treat remaining as skipped
                    for remaining in changed.iter().skip(i) {
                        skipped.push(remaining.name.clone());
                    }
                    break;
                }
            }
        }

        // Check if user quit
        if skipped.len() > changed.len() - i - 1 {
            break;
        }
    }

    // Summary
    println!("\n\u{2500}\u{2500}\u{2500} Review Summary \u{2500}\u{2500}\u{2500}");
    println!("\u{2713} Approved: {}", approved.len());
    println!("\u{2717} Rejected: {}", rejected.len());
    println!("\u{2192} Skipped:  {}", skipped.len());

    if approved.is_empty() {
        println!("\nNo changes approved. Baseline remains unchanged.");
        return Ok(());
    }

    // Confirm before applying
    println!("\nApproved shots to update:");
    for name in &approved {
        println!("  - {}", name);
    }

    let confirm = Confirm::with_theme(&theme)
        .with_prompt("Update baseline with approved shots?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("\nCancelled. No changes made.");
        return Ok(());
    }

    // Initialize plugins (for storage plugins)
    let plugin_registry = plugins::init_plugins(&config, &working_dir)?;

    // Apply approved changes
    let updated_count = update_baseline(
        &config,
        &working_dir,
        Some(&plugin_registry),
        Some(&approved),
    )?;

    info!("Updated {} baseline screenshot(s)", updated_count);
    println!("\n\u{2713} Updated {} baseline(s)", updated_count);

    Ok(())
}
