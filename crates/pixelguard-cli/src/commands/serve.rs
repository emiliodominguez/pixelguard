//! The `serve` command for serving an existing report.
//!
//! This command serves the HTML report with the decisions API,
//! allowing you to review and approve changes without re-running tests.

use anyhow::Result;
use clap::Args;

/// Arguments for the serve command.
#[derive(Args)]
pub struct ServeArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short, env = "PIXELGUARD_CONFIG")]
    config: Option<String>,

    /// Port for the local web server
    #[arg(long, default_value = "3333")]
    port: u16,

    /// Do not open the report in a browser
    #[arg(long)]
    no_open: bool,
}

/// Runs the serve command.
pub async fn run(args: ServeArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config to get output_dir
    let config = super::load_config(&working_dir, args.config.as_deref())?;
    let output_dir = working_dir.join(&config.output_dir);

    // Check if report exists
    let report_path = output_dir.join("report.html");

    if !report_path.exists() {
        anyhow::bail!(
            "❌ No report found at '{}'.\n\n\
             💡 Solutions:\n  \
             1️⃣ Run 'pixelguard test' first to generate the report\n  \
             2️⃣ Check that tests completed successfully\n  \
             3️⃣ Verify the output directory in pixelguard.config.json",
            report_path.display()
        );
    }

    super::server::serve_report(&output_dir, args.port, !args.no_open).await
}
