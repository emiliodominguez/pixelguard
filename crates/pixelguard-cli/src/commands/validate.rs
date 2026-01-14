//! The `validate` command for checking environment readiness.
//!
//! This command validates that all prerequisites are met for running
//! Pixelguard tests: config exists, Node.js is available, Playwright
//! is installed, and optionally checks if the base URL is reachable.

use std::process::Command;

use anyhow::Result;
use clap::Args;

/// Arguments for the validate command.
#[derive(Args)]
pub struct ValidateArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short)]
    config: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    json: bool,

    /// Skip checking if base URL is reachable
    #[arg(long)]
    skip_url_check: bool,
}

/// Result of a single validation check.
#[derive(Debug)]
struct CheckResult {
    name: &'static str,
    passed: bool,
    message: String,
}

impl CheckResult {
    fn pass(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            passed: true,
            message: message.into(),
        }
    }

    fn fail(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            passed: false,
            message: message.into(),
        }
    }
}

/// Runs the validate command.
pub async fn run(args: ValidateArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;
    let mut checks: Vec<CheckResult> = Vec::new();

    // Check 1: Configuration file
    let config_result = super::load_config(&working_dir, args.config.as_deref());
    let config = match config_result {
        Ok(cfg) => {
            checks.push(CheckResult::pass("config", "Configuration file is valid"));
            Some(cfg)
        }
        Err(e) => {
            checks.push(CheckResult::fail(
                "config",
                format!("Configuration error: {}", e),
            ));
            None
        }
    };

    // Check 2: Node.js
    checks.push(check_node());

    // Check 3: Playwright
    checks.push(check_playwright());

    // Check 4: Base URL (if config loaded and not skipped)
    if !args.skip_url_check {
        if let Some(ref cfg) = config {
            if !cfg.base_url.is_empty() {
                checks.push(check_base_url(&cfg.base_url).await);
            }
        }
    }

    // Check 5: Shots configured
    if let Some(ref cfg) = config {
        if cfg.shots.is_empty() && cfg.source != "storybook" {
            checks.push(CheckResult::fail(
                "shots",
                "No shots configured. Run 'pixelguard init' or add shots manually.",
            ));
        } else if cfg.source == "storybook" {
            checks.push(CheckResult::pass(
                "shots",
                "Shots will be discovered from Storybook at runtime",
            ));
        } else {
            checks.push(CheckResult::pass(
                "shots",
                format!("{} shots configured", cfg.shots.len()),
            ));
        }
    }

    // Output results
    if args.json {
        output_json(&checks)?;
    } else {
        output_table(&checks);
    }

    // Return error if any check failed
    let all_passed = checks.iter().all(|c| c.passed);
    if !all_passed {
        std::process::exit(1);
    }

    Ok(())
}

/// Checks if Node.js is available and returns version.
fn check_node() -> CheckResult {
    match Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            CheckResult::pass("node", format!("Node.js {} is available", version))
        }
        Ok(_) => CheckResult::fail("node", "Node.js command failed. Please install Node.js."),
        Err(_) => CheckResult::fail(
            "node",
            "Node.js not found. Please install Node.js (https://nodejs.org).",
        ),
    }
}

/// Checks if Playwright is installed.
fn check_playwright() -> CheckResult {
    // Try to check if playwright is installed via npx
    match Command::new("npx")
        .args(["playwright", "--version"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = version.lines().next().unwrap_or(&version);
            CheckResult::pass("playwright", format!("Playwright {} is available", version))
        }
        _ => {
            // Try checking if @playwright/test is in node_modules
            if std::path::Path::new("node_modules/@playwright/test").exists()
                || std::path::Path::new("node_modules/playwright").exists()
            {
                CheckResult::pass("playwright", "Playwright is installed in node_modules")
            } else {
                CheckResult::fail(
                    "playwright",
                    "Playwright not found. Install with: npm install -D @playwright/test",
                )
            }
        }
    }
}

/// Checks if the base URL is reachable.
async fn check_base_url(url: &str) -> CheckResult {
    match reqwest::get(url).await {
        Ok(response) if response.status().is_success() => {
            CheckResult::pass("base_url", format!("{} is reachable", url))
        }
        Ok(response) => CheckResult::fail(
            "base_url",
            format!("{} returned status {}", url, response.status()),
        ),
        Err(e) => CheckResult::fail("base_url", format!("{} is not reachable: {}", url, e)),
    }
}

/// Outputs check results as JSON.
fn output_json(checks: &[CheckResult]) -> Result<()> {
    let results: Vec<serde_json::Value> = checks
        .iter()
        .map(|c| {
            serde_json::json!({
                "check": c.name,
                "passed": c.passed,
                "message": c.message,
            })
        })
        .collect();

    let all_passed = checks.iter().all(|c| c.passed);
    let output = serde_json::json!({
        "valid": all_passed,
        "checks": results,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Outputs check results as a formatted table.
fn output_table(checks: &[CheckResult]) {
    println!("Pixelguard Environment Validation\n");

    for check in checks {
        let icon = if check.passed { "\u{2713}" } else { "\u{2717}" };
        let status = if check.passed { "PASS" } else { "FAIL" };
        println!("{} [{}] {}: {}", icon, status, check.name, check.message);
    }

    println!();

    let passed = checks.iter().filter(|c| c.passed).count();
    let total = checks.len();
    let all_passed = passed == total;

    if all_passed {
        println!(
            "All checks passed ({}/{}). Ready to run tests!",
            passed, total
        );
    } else {
        println!(
            "{}/{} checks passed. Please fix the issues above before running tests.",
            passed, total
        );
    }
}
