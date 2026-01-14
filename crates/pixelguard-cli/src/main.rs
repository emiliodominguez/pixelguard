//! Pixelguard CLI - Visual Regression Testing Made Simple
//!
//! This is the main entry point for the Pixelguard CLI tool.
//! It provides commands for initializing, testing, and managing
//! visual regression tests.

mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

/// Pixelguard - Visual regression testing for everyone.
///
/// Zero-config screenshot testing that runs locally and in CI.
/// No cloud, no vendor lock-in, just simple visual regression testing.
#[derive(Parser)]
#[command(name = "pixelguard")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Pixelguard in the current project
    Init(commands::init::InitArgs),

    /// Capture screenshots and compare against baseline
    Test(commands::test::TestArgs),

    /// List all configured shots
    List(commands::list::ListArgs),

    /// List and validate installed plugins
    Plugins(commands::plugins::PluginsArgs),

    /// Validate environment prerequisites (Node.js, Playwright, etc.)
    Validate(commands::validate::ValidateArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();

    match cli.command {
        Commands::Init(args) => commands::init::run(args).await,
        Commands::Test(args) => commands::test::run(args).await,
        Commands::List(args) => commands::list::run(args).await,
        Commands::Plugins(args) => commands::plugins::run(args).await,
        Commands::Validate(args) => commands::validate::run(args).await,
    }
}
