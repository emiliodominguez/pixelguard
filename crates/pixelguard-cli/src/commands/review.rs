//! The `review` command for interactively reviewing visual diffs.
//!
//! This command reads the results.json file and prompts the user to
//! approve or reject each changed shot interactively in the terminal.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use pixelguard_core::{
    capture::{update_baseline_exact, validate_screenshot_name},
    plugins,
};
use serde::Deserialize;
use tracing::info;

use super::decisions::{check_approval, ApprovalCheck};

/// Arguments for the review command.
#[derive(Args)]
pub struct ReviewArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short, env = "PIXELGUARD_CONFIG")]
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
    /// Detailed results
    results: ResultsDetail,
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
    /// SHA-256 of the current image when the report was generated
    #[serde(default)]
    current_hash: Option<String>,
    /// Areas that changed; only counted here
    #[serde(default)]
    regions: Vec<serde_json::Value>,
}

/// Action the user can take for each shot.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ReviewAction {
    Approve,
    Reject,
    Skip,
    ViewDiff,
    ViewHeatmap,
    Quit,
}

impl std::fmt::Display for ReviewAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewAction::Approve => write!(f, "Approve (update baseline)"),
            ReviewAction::Reject => write!(f, "Reject (keep baseline)"),
            ReviewAction::Skip => write!(f, "Skip (decide later)"),
            ReviewAction::ViewDiff => write!(f, "View diff image"),
            ReviewAction::ViewHeatmap => write!(f, "View heatmap image"),
            ReviewAction::Quit => write!(f, "Quit review"),
        }
    }
}

/// Runs the review command.
pub async fn run(args: ReviewArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config
    let config = super::load_config(&working_dir, args.config.as_deref())?;
    let output_dir = working_dir.join(&config.output_dir);

    // Determine results file path
    let results_path = args
        .results
        .unwrap_or_else(|| output_dir.join("results.json"));

    // Load results.json
    let results_content = std::fs::read_to_string(&results_path).with_context(|| {
        format!(
            "❌ Could not read results file at '{}'.\n\n\
             💡 Solutions:\n  \
             1️⃣ Run 'pixelguard test' first to generate results\n  \
             2️⃣ Check that tests completed successfully\n  \
             3️⃣ Verify the output directory path",
            results_path.display()
        )
    })?;

    let results: ResultsJson = serde_json::from_str(&results_content).with_context(|| {
        format!(
            "❌ Invalid results file format in '{}'.\n\n\
             💡 Solution: Try running 'pixelguard test' again to regenerate results.",
            results_path.display()
        )
    })?;

    let changed: Vec<ChangedShot> = results
        .results
        .changed
        .into_iter()
        .filter(|shot| validate_screenshot_name(&shot.name).is_ok())
        .collect();

    // Check if there are changes to review
    if changed.is_empty() {
        println!("✅ No changed screenshots to review.");
        return Ok(());
    }

    println!(
        "\n🔍 Found {} changed screenshot(s) to review\n",
        changed.len()
    );

    let theme = ColorfulTheme::default();
    let mut approved: Vec<&ChangedShot> = Vec::new();
    let mut rejected = 0usize;
    let mut skipped = 0usize;

    let actions = [
        ReviewAction::Approve,
        ReviewAction::Reject,
        ReviewAction::Skip,
        ReviewAction::ViewDiff,
        ReviewAction::ViewHeatmap,
        ReviewAction::Quit,
    ];

    'review: for (i, shot) in changed.iter().enumerate() {
        if args.open_diff {
            open_image(&output_dir, Visual::Diff, &shot.name);
        }

        loop {
            let areas = match shot.regions.len() {
                0 => String::new(),
                1 => ", 1 area".to_string(),
                count => format!(", {} areas", count),
            };
            println!(
                "\n[{}/{}] {} ({:.2}% different{})",
                i + 1,
                changed.len(),
                shot.name,
                shot.diff_percentage,
                areas
            );

            let selection = Select::with_theme(&theme)
                .with_prompt("What would you like to do?")
                .items(actions)
                .default(0)
                .interact()?;

            match actions[selection] {
                ReviewAction::Approve => {
                    approved.push(shot);
                    println!("  \u{2713} Approved: {}", shot.name);
                    break;
                }
                ReviewAction::Reject => {
                    rejected += 1;
                    println!("  \u{2717} Rejected: {}", shot.name);
                    break;
                }
                ReviewAction::Skip => {
                    skipped += 1;
                    println!("  \u{2192} Skipped: {}", shot.name);
                    break;
                }
                ReviewAction::ViewDiff => open_image(&output_dir, Visual::Diff, &shot.name),
                ReviewAction::ViewHeatmap => open_image(&output_dir, Visual::Heatmap, &shot.name),
                ReviewAction::Quit => {
                    println!("\nQuitting review...");
                    skipped += changed.len() - i;
                    break 'review;
                }
            }
        }
    }

    // Summary
    println!("\n─── 📋 Review Summary ───");
    println!("✅ Approved: {}", approved.len());
    println!("❌ Rejected: {}", rejected);
    println!("⏭️  Skipped:  {}", skipped);

    if approved.is_empty() {
        println!("\nℹ️  No changes approved. Baseline remains unchanged.");
        return Ok(());
    }

    // Only promote images that are still exactly what the report showed.
    let mut applicable = Vec::new();
    for shot in &approved {
        match check_approval(&output_dir, &shot.name, shot.current_hash.as_deref()) {
            ApprovalCheck::Matches | ApprovalCheck::Unverified => {
                applicable.push(shot.name.clone())
            }
            ApprovalCheck::Stale => println!(
                "⚠️  {} changed on disk since the report was generated; skipping it. \
                 Run 'pixelguard test' and review again.",
                shot.name
            ),
            ApprovalCheck::Missing => {
                println!("⚠️  {} has no current screenshot; skipping it.", shot.name)
            }
        }
    }

    if applicable.is_empty() {
        println!("\nℹ️  Nothing left to apply. Baseline remains unchanged.");
        return Ok(());
    }

    // Confirm before applying
    println!("\nApproved shots to update:");
    for name in &applicable {
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
    let updated =
        update_baseline_exact(&config, &working_dir, Some(&plugin_registry), &applicable)?;

    info!("Updated {} baseline screenshot(s)", updated.len());
    println!("\n✅ Updated {} baseline(s)", updated.len());
    println!("\n💡 Next steps:");
    println!("  1️⃣ Commit the updated baselines to version control");
    println!("  2️⃣ Run 'pixelguard test' to verify the changes");

    Ok(())
}

/// An image generated for a changed screenshot.
#[derive(Debug, Clone, Copy)]
enum Visual {
    Diff,
    Heatmap,
}

impl Visual {
    fn folder(self) -> &'static str {
        match self {
            Visual::Diff => "diff",
            Visual::Heatmap => "heatmap",
        }
    }
}

/// Opens the diff or heatmap image for a validated screenshot name.
///
/// The path is derived from the name, never read from results.json, so a
/// tampered results file cannot make this open an arbitrary file or app.
fn open_image(output_dir: &std::path::Path, visual: Visual, name: &str) {
    let path = output_dir
        .join(visual.folder())
        .join(format!("{}.png", name));
    if !path.is_file() {
        match visual {
            Visual::Diff => eprintln!("  Diff image not found at {}", path.display()),
            // Screenshots that changed size have no heatmap.
            Visual::Heatmap => eprintln!(
                "  No heatmap at {}; the screenshots may differ in size",
                path.display()
            ),
        }
        return;
    }
    match open::that(&path) {
        Ok(()) => println!("  Opened {} image", visual.folder()),
        Err(e) => eprintln!("Could not open {} image: {}", visual.folder(), e),
    }
}
