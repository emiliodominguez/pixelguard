//! Plugin system for extending Pixelguard functionality.
//!
//! This module provides a plugin architecture that allows extending Pixelguard
//! with custom storage backends, reporters, capture engines, differ algorithms,
//! and notifiers.
//!
//! # Plugin Categories
//!
//! - **Storage**: Where baselines are stored (local filesystem, S3, R2, Azure, etc.)
//! - **Reporter**: Report formats (HTML, JSON, JUnit XML, etc.)
//! - **Capture**: Screenshot engines (Playwright, Puppeteer, etc.)
//! - **Differ**: Image comparison algorithms (pixel diff, SSIM, etc.)
//! - **Notifier**: Notification services (Slack, Teams, webhooks, etc.)
//!
//! # Architecture
//!
//! Plugins are npm packages that export hooks as async JavaScript functions.
//! The Rust code executes plugins via Node.js subprocess, communicating through
//! JSON over stdin/stdout.
//!
//! # Example Configuration
//!
//! ```json
//! {
//!   "plugins": [
//!     "pixelguard-plugin-s3",
//!     {
//!       "name": "pixelguard-plugin-slack",
//!       "options": {
//!         "webhookUrl": "https://hooks.slack.com/..."
//!       }
//!     }
//!   ],
//!   "pluginOptions": {
//!     "pixelguard-plugin-s3": {
//!       "bucket": "my-baselines",
//!       "region": "us-east-1"
//!     }
//!   }
//! }
//! ```
//!
//! # Creating Plugins
//!
//! A plugin is an npm package with a `pixelguard` field in package.json:
//!
//! ```json
//! {
//!   "name": "pixelguard-plugin-example",
//!   "main": "dist/index.js",
//!   "pixelguard": {
//!     "name": "Example Plugin",
//!     "category": "notifier",
//!     "entry": "dist/index.js",
//!     "hooks": ["notify"]
//!   }
//! }
//! ```

pub mod discovery;
pub mod executor;
pub mod loader;
pub mod registry;
pub mod types;

use std::path::Path;

use anyhow::Result;
use tracing::{debug, info};

use crate::Config;

pub use registry::PluginRegistry;
pub use types::{
    CaptureInput, CaptureOutput, CaptureShot, CaptureViewport, CapturedShot, DifferInput,
    DifferOutput, FailedShot, LoadedPlugin, NotifierInput, PluginCategory, PluginManifest,
    PluginResult, ReporterChangedShot, ReporterConfig, ReporterDiffResult, ReporterInput,
    ReporterOutput, StorageInput, StorageOutput,
};

/// Initializes the plugin system by discovering and loading configured plugins.
///
/// This function:
/// 1. Resolves plugin paths from config's plugins array
/// 2. Loads and validates each plugin's manifest
/// 3. Builds a registry for efficient plugin lookup
///
/// # Arguments
///
/// * `config` - The Pixelguard configuration containing plugin entries
/// * `working_dir` - The project's working directory
///
/// # Returns
///
/// A `PluginRegistry` containing all loaded plugins, or an error if any
/// plugin cannot be loaded.
///
/// # Example
///
/// ```rust,no_run
/// use pixelguard_core::{Config, plugins};
///
/// async fn example() -> anyhow::Result<()> {
///     let config = Config::load("pixelguard.config.json")?;
///     let working_dir = std::env::current_dir()?;
///     let registry = plugins::init_plugins(&config, &working_dir)?;
///
///     if registry.has_override(plugins::PluginCategory::Storage) {
///         println!("Using custom storage backend");
///     }
///
///     Ok(())
/// }
/// ```
pub fn init_plugins(config: &Config, working_dir: &Path) -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::new();

    if config.plugins.is_empty() {
        debug!("No plugins configured");
        return Ok(registry);
    }

    info!("Loading {} plugin(s)...", config.plugins.len());

    // Resolve plugin paths
    let resolved = discovery::resolve_plugins(&config.plugins, working_dir)?;

    // Load and validate each plugin
    for (entry, path) in resolved {
        debug!("Loading plugin: {} from {:?}", entry.name(), path);

        // Validate the plugin directory
        discovery::validate_plugin_path(&path)?;

        // Get global options for this plugin
        let global_options = config
            .plugin_options
            .get(entry.name())
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        // Load the plugin
        let plugin = loader::load_plugin(&entry, &path, &global_options)?;

        // Validate the manifest
        loader::validate_manifest(&plugin)?;

        info!(
            "  Loaded: {} ({:?}, hooks: {:?})",
            plugin.name(),
            plugin.category(),
            plugin.manifest.hooks
        );

        registry.register(plugin);
    }

    Ok(registry)
}

/// Helper function to check if Node.js is available.
///
/// This should be called early to provide a clear error message if Node.js
/// is not installed.
pub fn check_node_available() -> Result<()> {
    use std::process::Command;

    let output = Command::new("node").arg("--version").output();

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout);
            debug!("Node.js version: {}", version.trim());
            Ok(())
        }
        Ok(out) => {
            anyhow::bail!(
                "Node.js returned an error: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Err(e) => {
            anyhow::bail!(
                "Node.js is required for plugins but was not found. \
                 Please install Node.js from https://nodejs.org/. Error: {}",
                e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_config(plugins: Vec<crate::config::PluginEntry>) -> Config {
        Config {
            plugins,
            ..Default::default()
        }
    }

    #[test]
    fn init_plugins_empty_config() {
        let dir = tempdir().unwrap();
        let config = create_test_config(vec![]);

        let registry = init_plugins(&config, dir.path()).unwrap();

        assert!(registry.is_empty());
    }

    #[test]
    fn init_plugins_missing_plugin() {
        let dir = tempdir().unwrap();
        let config = create_test_config(vec![crate::config::PluginEntry::Name(
            "nonexistent".to_string(),
        )]);

        let result = init_plugins(&config, dir.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn init_plugins_valid_plugin() {
        let dir = tempdir().unwrap();

        // Create a mock plugin
        let plugin_dir = dir
            .path()
            .join("node_modules")
            .join("pixelguard-plugin-test");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("package.json"),
            r#"{
                "name": "pixelguard-plugin-test",
                "version": "1.0.0",
                "main": "index.js",
                "pixelguard": {
                    "name": "Test Plugin",
                    "category": "notifier",
                    "entry": "index.js",
                    "hooks": ["notify"]
                }
            }"#,
        )
        .unwrap();
        std::fs::write(plugin_dir.join("index.js"), "module.exports = {}").unwrap();

        let config = create_test_config(vec![crate::config::PluginEntry::Name(
            "pixelguard-plugin-test".to_string(),
        )]);

        let registry = init_plugins(&config, dir.path()).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.has_override(PluginCategory::Notifier));
        assert_eq!(registry.notifiers().len(), 1);
        assert_eq!(registry.notifiers()[0].name(), "Test Plugin");
    }
}
