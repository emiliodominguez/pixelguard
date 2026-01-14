//! The `plugins` command for listing and validating installed plugins.
//!
//! This command shows all configured plugins and their status.

use anyhow::Result;
use clap::Args;
use pixelguard_core::{plugins, Config};

/// Arguments for the plugins command.
#[derive(Args)]
pub struct PluginsArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short)]
    config: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    json: bool,
}

/// Runs the plugins command.
pub async fn run(args: PluginsArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config from custom path or default
    let config = if let Some(config_path) = &args.config {
        Config::load(working_dir.join(config_path))?
    } else {
        Config::load_or_default(&working_dir)?
    };

    if config.plugins.is_empty() {
        if args.json {
            println!("[]");
        } else {
            println!("No plugins configured.");
            println!("\nTo add plugins, update your pixelguard.config.json:");
            println!("  {{");
            println!("    \"plugins\": [\"pixelguard-plugin-example\"]");
            println!("  }}");
        }
        return Ok(());
    }

    // Try to load plugins
    let registry = plugins::init_plugins(&config, &working_dir);

    if args.json {
        output_json(&config, &registry)?;
    } else {
        output_table(&config, &registry)?;
    }

    Ok(())
}

/// Outputs plugin information as JSON.
fn output_json(_config: &Config, registry: &Result<plugins::PluginRegistry>) -> Result<()> {
    match registry {
        Ok(reg) => {
            let plugins_info: Vec<serde_json::Value> = reg
                .all_active()
                .iter()
                .map(|plugin| {
                    serde_json::json!({
                        "name": plugin.manifest.name,
                        "category": format!("{:?}", plugin.category()),
                        "hooks": plugin.manifest.hooks,
                        "version": plugin.manifest.version,
                        "status": "loaded"
                    })
                })
                .collect();

            println!("{}", serde_json::to_string_pretty(&plugins_info)?);
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "error": e.to_string()
                }))?
            );
        }
    }

    Ok(())
}

/// Outputs plugin information as a formatted table.
fn output_table(_config: &Config, registry: &Result<plugins::PluginRegistry>) -> Result<()> {
    match registry {
        Ok(reg) => {
            let all_plugins = reg.all_active();

            if all_plugins.is_empty() {
                println!("No plugins loaded.");
                return Ok(());
            }

            println!("Loaded plugins ({}):\n", all_plugins.len());

            // Find max name length for alignment
            let max_name = all_plugins
                .iter()
                .map(|p| p.manifest.name.len())
                .max()
                .unwrap_or(10);

            for plugin in &all_plugins {
                println!(
                    "  \u{2713} {:<width$}  {:?}  [{}]",
                    plugin.manifest.name,
                    plugin.category(),
                    plugin.manifest.hooks.join(", "),
                    width = max_name
                );
            }

            // Show summary by category
            println!();
            print_category_summary(reg);
        }
        Err(e) => {
            println!("Error loading plugins: {}", e);
        }
    }

    Ok(())
}

/// Prints a summary of plugins by category.
fn print_category_summary(registry: &plugins::PluginRegistry) {
    let mut has_output = false;

    // Check for override categories
    if registry.has_override(plugins::PluginCategory::Storage) {
        if !has_output {
            println!("Active overrides:");
            has_output = true;
        }
        if let Some(p) = registry.get(plugins::PluginCategory::Storage) {
            println!("  Storage: {}", p.name());
        }
    }

    if registry.has_override(plugins::PluginCategory::Capture) {
        if !has_output {
            println!("Active overrides:");
            has_output = true;
        }
        if let Some(p) = registry.get(plugins::PluginCategory::Capture) {
            println!("  Capture: {}", p.name());
        }
    }

    if registry.has_override(plugins::PluginCategory::Differ) {
        if !has_output {
            println!("Active overrides:");
            has_output = true;
        }
        if let Some(p) = registry.get(plugins::PluginCategory::Differ) {
            println!("  Differ: {}", p.name());
        }
    }

    // Check stackable categories
    let notifiers = registry.notifiers();
    let reporters = registry.reporters();

    if !notifiers.is_empty() || !reporters.is_empty() {
        if has_output {
            println!();
        }
        println!("Stackable plugins:");

        for plugin in reporters {
            println!("  Reporter: {}", plugin.name());
        }

        for plugin in notifiers {
            println!("  Notifier: {}", plugin.name());
        }
    }
}
