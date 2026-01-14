//! The `init` command for initializing Pixelguard configuration.
//!
//! This command auto-detects the project type and creates a
//! `pixelguard.config.json` file with sensible defaults.

use anyhow::Result;
use clap::Args;
use pixelguard_core::{detect_project_type, Config, ProjectType};
use tracing::info;

/// Arguments for the init command.
#[derive(Args)]
pub struct InitArgs {
    /// Re-initialize even if config already exists
    #[arg(long)]
    force: bool,

    /// Port to use for dev server detection
    #[arg(long, short)]
    port: Option<u16>,
}

/// Runs the init command.
pub async fn run(args: InitArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Check if config already exists
    if Config::exists(&working_dir) && !args.force {
        anyhow::bail!(
            "Configuration file already exists at 'pixelguard.config.json'.\n\
             Use --force to overwrite."
        );
    }

    println!("Detecting project type...");

    let project = detect_project_type(&working_dir, args.port).await?;

    let config = match &project {
        ProjectType::Storybook { base_url, stories } => {
            println!("\u{2713} Found Storybook at {}", base_url);
            println!("\u{2713} Discovered {} stories", stories.len());

            Config {
                source: "storybook".to_string(),
                base_url: base_url.clone(),
                shots: stories.clone(),
                ..Default::default()
            }
        }

        ProjectType::DevServer { base_url } => {
            println!("\u{2713} Found dev server at {}", base_url);
            println!(
                "  Note: Add shots to 'pixelguard.config.json' to specify\n  \
                 which pages or components to capture."
            );

            Config {
                source: "manual".to_string(),
                base_url: base_url.clone(),
                ..Default::default()
            }
        }

        ProjectType::Unknown => {
            println!(
                "Could not detect a running dev server.\n\
                 Creating minimal config — you'll need to add baseUrl and shots manually."
            );

            Config::default()
        }
    };

    config.save_to_dir(&working_dir)?;
    println!("\u{2713} Created pixelguard.config.json");

    // Create output directory
    let output_dir = working_dir.join(&config.output_dir);
    std::fs::create_dir_all(&output_dir)?;

    // Create .gitkeep in output directory
    std::fs::write(output_dir.join(".gitkeep"), "")?;

    info!("Initialized Pixelguard in {:?}", working_dir);

    println!("\nNext steps:");
    match &project {
        ProjectType::Storybook { .. } => {
            println!("  1. Run: npx pixelguard test");
            println!("  2. Commit .pixelguard/ as your baseline");
        }
        ProjectType::DevServer { .. } => {
            println!("  1. Edit pixelguard.config.json to add your shots");
            println!("  2. Run: npx pixelguard test");
            println!("  3. Commit .pixelguard/ as your baseline");
        }
        ProjectType::Unknown => {
            println!("  1. Start your dev server");
            println!("  2. Edit pixelguard.config.json to add baseUrl and shots");
            println!("  3. Run: npx pixelguard test");
            println!("  4. Commit .pixelguard/ as your baseline");
        }
    }

    Ok(())
}
