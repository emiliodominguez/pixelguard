//! The `apply` command for applying decisions from a JSON file.
//!
//! This command reads a decisions.json file (saved from the HTML report)
//! and updates the baseline for all approved shots.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use pixelguard_core::{capture::update_baseline, plugins};
use serde::Deserialize;
use tracing::info;

/// Default path for decisions file within the output directory.
const DEFAULT_DECISIONS_FILE: &str = "decisions.json";

/// Arguments for the apply command.
#[derive(Args)]
pub struct ApplyArgs {
    /// Path to decisions.json file (default: .pixelguard/decisions.json)
    #[arg(value_name = "FILE")]
    decisions_file: Option<PathBuf>,

    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short)]
    config: Option<String>,

    /// Dry run - show what would be updated without making changes
    #[arg(long)]
    dry_run: bool,
}

/// Structure of the decisions.json file exported from the browser.
#[derive(Debug, Deserialize)]
struct DecisionsFile {
    /// Schema version
    #[allow(dead_code)]
    version: String,
    /// When the file was exported
    #[allow(dead_code)]
    #[serde(rename = "exportedAt")]
    exported_at: String,
    /// The decisions made for each shot
    decisions: HashMap<String, Decision>,
}

/// A single decision for a shot.
#[derive(Debug, Deserialize)]
struct Decision {
    /// The action taken: "approve" or "reject"
    action: String,
    /// ISO timestamp when decision was made
    #[allow(dead_code)]
    timestamp: String,
    /// Where the decision was made (e.g., "browser")
    #[allow(dead_code)]
    source: String,
}

/// Runs the apply command.
pub async fn run(args: ApplyArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config first to get output_dir for default decisions path
    let config = super::load_config(&working_dir, args.config.as_deref())?;

    // Determine decisions file path
    let decisions_path = args.decisions_file.unwrap_or_else(|| {
        working_dir
            .join(&config.output_dir)
            .join(DEFAULT_DECISIONS_FILE)
    });

    // Load the decisions file
    let decisions_content = std::fs::read_to_string(&decisions_path).with_context(|| {
        format!(
            "Could not read decisions file at '{}'. \
             Make decisions in the HTML report first (click Save).",
            decisions_path.display()
        )
    })?;

    let decisions_file: DecisionsFile =
        serde_json::from_str(&decisions_content).with_context(|| {
            format!(
                "Invalid decisions file format in '{}'. \
                 Make sure this was saved from the Pixelguard report.",
                decisions_path.display()
            )
        })?;

    // Filter for approved decisions only
    let approved: Vec<String> = decisions_file
        .decisions
        .iter()
        .filter(|(_, decision)| decision.action == "approve")
        .map(|(name, _)| name.clone())
        .collect();

    let rejected_count = decisions_file
        .decisions
        .iter()
        .filter(|(_, decision)| decision.action == "reject")
        .count();

    if approved.is_empty() {
        println!("No approved shots found in decisions file.");
        if rejected_count > 0 {
            println!("({} shots were marked as rejected)", rejected_count);
        }
        return Ok(());
    }

    println!(
        "Found {} approved shot(s), {} rejected",
        approved.len(),
        rejected_count
    );

    for name in &approved {
        println!("  \u{2713} {}", name);
    }

    if args.dry_run {
        println!("\nDry run - no changes made.");
        println!("Run without --dry-run to update the baseline.");
        return Ok(());
    }

    // Initialize plugins (for storage plugins)
    let plugin_registry = plugins::init_plugins(&config, &working_dir)?;

    // Update baseline with approved shots
    println!("\nUpdating baseline...");
    let updated_count = update_baseline(
        &config,
        &working_dir,
        Some(&plugin_registry),
        Some(&approved),
    )?;

    if updated_count > 0 {
        info!("Updated {} baseline screenshot(s)", updated_count);
        println!("\n\u{2713} Updated {} baseline(s)", updated_count);

        if updated_count < approved.len() {
            println!("  (some approved shots may not have current screenshots)");
        }
    } else {
        println!("\nNo baselines were updated.");
        println!("Make sure 'pixelguard test' was run to capture current screenshots.");
    }

    Ok(())
}
