//! The `apply` command for applying decisions from a JSON file.
//!
//! This command reads a decisions.json file (saved from the HTML report)
//! and updates the baseline for all approved shots.
//!
//! An approval only applies to the exact image that was reviewed: if the
//! current screenshot changed since (for example because tests ran again), it
//! is skipped instead of being promoted unseen.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use pixelguard_core::{capture::update_baseline_exact, plugins};
use tracing::info;

use super::decisions::{
    check_approval, ApprovalCheck, DecisionAction, DecisionsFile, DECISIONS_FILE,
};

/// Arguments for the apply command.
#[derive(Args)]
pub struct ApplyArgs {
    /// Path to decisions.json file (default: .pixelguard/decisions.json)
    #[arg(value_name = "FILE")]
    decisions_file: Option<PathBuf>,

    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short, env = "PIXELGUARD_CONFIG")]
    config: Option<String>,

    /// Dry run - show what would be updated without making changes
    #[arg(long)]
    dry_run: bool,

    /// Also apply approvals that carry no image hash (made by older reports)
    #[arg(long)]
    allow_unverified: bool,
}

/// Runs the apply command.
pub async fn run(args: ApplyArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config first to get output_dir for default decisions path
    let config = super::load_config(&working_dir, args.config.as_deref())?;
    let output_dir = working_dir.join(&config.output_dir);

    // Determine decisions file path
    let decisions_path = args
        .decisions_file
        .unwrap_or_else(|| output_dir.join(DECISIONS_FILE));

    // Load the decisions file
    let decisions_content = std::fs::read_to_string(&decisions_path).with_context(|| {
        format!(
            "❌ Could not read decisions file at '{}'.\n\n\
             💡 Solutions:\n  \
             1️⃣ Make decisions in the HTML report first\n  \
             2️⃣ Click 'Export' in the report to save decisions\n  \
             3️⃣ Or run 'pixelguard review' for terminal-based review",
            decisions_path.display(),
        )
    })?;

    let decisions_file = DecisionsFile::parse(&decisions_content).with_context(|| {
        format!(
            "❌ Invalid decisions file format in '{}'.\n\n\
             💡 Solutions:\n  \
             • Make sure this was saved from the Pixelguard report\n  \
             • Re-export from the report if corrupted",
            decisions_path.display()
        )
    })?;

    let approved = decisions_file.names_with(DecisionAction::Approve);
    let rejected_count = decisions_file.names_with(DecisionAction::Reject).len();

    if approved.is_empty() {
        println!("ℹ️  No approved shots found in decisions file.");
        if rejected_count > 0 {
            println!("   ({} shots were marked as rejected)", rejected_count);
        }
        println!(
            "\n💡 Tip: Review and approve changes in the HTML report or use 'pixelguard review'."
        );
        return Ok(());
    }

    println!(
        "📊 Found {} approved shot(s), {} rejected",
        approved.len(),
        rejected_count
    );

    let mut applicable = Vec::new();
    for name in &approved {
        let expected = decisions_file.decisions[name].hash.as_deref();
        match check_approval(&output_dir, name, expected) {
            ApprovalCheck::Matches => {
                println!("  ✓ {}", name);
                applicable.push(name.clone());
            }
            ApprovalCheck::Unverified if args.allow_unverified => {
                println!("  ✓ {} (unverified: decision has no image hash)", name);
                applicable.push(name.clone());
            }
            ApprovalCheck::Unverified => println!(
                "  ⚠️  {} skipped: decision has no image hash (re-approve it, or pass --allow-unverified)",
                name
            ),
            ApprovalCheck::Stale => println!(
                "  ⚠️  {} skipped: the screenshot changed after it was approved; review it again",
                name
            ),
            ApprovalCheck::Missing => {
                println!("  ⚠️  {} skipped: no current screenshot", name)
            }
        }
    }

    if args.dry_run {
        println!("\n🔍 Dry run - no changes made.");
        println!("💡 Run without --dry-run to update the baseline.");
        return Ok(());
    }

    if applicable.is_empty() {
        println!("\nNo baselines were updated.");
        return Ok(());
    }

    // Initialize plugins (for storage plugins)
    let plugin_registry = plugins::init_plugins(&config, &working_dir)?;

    // Update baseline with approved shots
    println!("\n📦 Updating baseline...");
    let updated =
        update_baseline_exact(&config, &working_dir, Some(&plugin_registry), &applicable)?;

    info!("✅ Updated {} baseline screenshot(s)", updated.len());
    println!("\n✅ Updated {} baseline(s)", updated.len());
    if updated.len() < approved.len() {
        println!(
            "   ℹ️  {} approved shot(s) were skipped (see above)",
            approved.len() - updated.len()
        );
    }

    println!("\n💡 Next steps:");
    println!("  1️⃣ Commit the updated baselines to version control");
    println!("  2️⃣ Run 'pixelguard test' to verify the changes");

    Ok(())
}
