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
            "⚠️  Configuration file already exists at 'pixelguard.config.json'.\n\n\
             💡 Solution: Use --force to overwrite the existing configuration.\n   \
             Example: pixelguard init --force"
        );
    }

    println!("🔍 Detecting project type...");

    let project = detect_project_type(&working_dir, args.port).await?;

    let config = match &project {
        ProjectType::Storybook { base_url, stories } => {
            println!("✅ Found Storybook at {}", base_url);
            println!("✨ Discovered {} stories", stories.len());
            println!("📝 Automatically generating configuration...");

            Config {
                source: "storybook".to_string(),
                base_url: base_url.clone(),
                shots: stories.clone(),
                ..Default::default()
            }
        }

        ProjectType::DevServer { base_url } => {
            println!("✅ Found dev server at {}", base_url);
            println!(
                "  📝 Note: Add shots to 'pixelguard.config.json' to specify\n  \
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
                "🤔 Could not detect a running dev server.\n\
                 📝 Creating minimal config — you'll need to add baseUrl and shots manually.\n\n\
                 💡 Tip: Start your dev server first, then run 'pixelguard init' again."
            );

            Config::default()
        }
    };

    config.save_to_dir(&working_dir)?;
    println!("✅ Created pixelguard.config.json");

    // Create output directory
    let output_dir = working_dir.join(&config.output_dir);
    std::fs::create_dir_all(&output_dir)?;

    // Create .gitkeep in output directory
    std::fs::write(output_dir.join(".gitkeep"), "")?;

    info!("✅ Initialized Pixelguard in {:?}", working_dir);

    println!("\n🎉 Success! Next steps:");
    match &project {
        ProjectType::Storybook { .. } => {
            println!("  1️⃣ Run: npx pixelguard test");
            println!("  2️⃣ Commit .pixelguard/ as your baseline");
            println!("\n💡 Tip: Run tests in CI to catch visual regressions automatically!");
        }
        ProjectType::DevServer { .. } => {
            println!("  1️⃣ Edit pixelguard.config.json to add your shots");
            println!("  2️⃣ Run: npx pixelguard test");
            println!("  3️⃣ Commit .pixelguard/ as your baseline");
            println!("\n💡 Tip: Use 'pixelguard list' to preview your shots before testing.");
        }
        ProjectType::Unknown => {
            println!("  1️⃣ Start your dev server");
            println!("  2️⃣ Edit pixelguard.config.json to add baseUrl and shots");
            println!("  3️⃣ Run: npx pixelguard test");
            println!("  4️⃣ Commit .pixelguard/ as your baseline");
            println!("\n💡 Tip: Use 'pixelguard validate' to check your setup before testing.");
        }
    }

    Ok(())
}
