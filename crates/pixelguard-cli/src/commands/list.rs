//! The `list` command for displaying configured shots.
//!
//! This command lists all shots that would be captured without
//! actually taking screenshots.

use anyhow::Result;
use clap::Args;

/// Arguments for the list command.
#[derive(Args)]
pub struct ListArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short, env = "PIXELGUARD_CONFIG")]
    config: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    json: bool,
}

/// Runs the list command.
pub async fn run(args: ListArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config from custom path or default
    let mut config = super::load_config(&working_dir, args.config.as_deref())?;

    // Storybook shots are discovered at test time; list what a test run would capture.
    if config.source == "storybook" && !config.base_url.is_empty() {
        let discovered = super::test::discover_shots(&config).await?;
        config.shots = super::test::merge_shots(discovered, &config.shots);
    }

    if config.shots.is_empty() {
        if args.json {
            println!("[]");
        } else {
            println!("ℹ️  No shots configured.");
            println!("\n💡 Solutions:");
            println!("  • Run 'pixelguard init' to auto-detect shots");
            println!("  • Or add them manually to pixelguard.config.json");
        }
        return Ok(());
    }

    if args.json {
        let output: Vec<_> = config
            .shots
            .iter()
            .map(|shot| {
                serde_json::json!({
                    "name": shot.name,
                    "path": shot.path,
                    "waitFor": shot.wait_for,
                    "delay": shot.delay,
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("📸 Configured shots ({}):\n", config.shots.len());

        let max_name_len = config.shots.iter().map(|s| s.name.len()).max().unwrap_or(0);

        for shot in &config.shots {
            println!(
                "  {:<width$} \u{2192} {}",
                shot.name,
                shot.path,
                width = max_name_len
            );
        }

        println!("\n🌐 Base URL: {}", config.base_url);
        println!(
            "📐 Viewport: {}x{}",
            config.viewport.width, config.viewport.height
        );
    }

    Ok(())
}
