//! Configuration management for Pixelguard.
//!
//! This module handles loading, saving, and managing the `pixelguard.config.json` file.
//! All configuration fields are optional with sensible defaults.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Main configuration for Pixelguard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Source type (e.g., "storybook", "nextjs", "vite", "manual")
    #[serde(default)]
    pub source: String,

    /// Base URL for the dev server
    #[serde(default)]
    pub base_url: String,

    /// Glob patterns to include
    #[serde(default = "default_include")]
    pub include: Vec<String>,

    /// Glob patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Viewport configuration
    #[serde(default)]
    pub viewport: Viewport,

    /// Diff threshold (0.0 to 1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f64,

    /// Output directory for screenshots and reports
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// List of shots to capture
    #[serde(default)]
    pub shots: Vec<Shot>,

    /// List of plugins to load
    #[serde(default)]
    pub plugins: Vec<PluginEntry>,

    /// Plugin-specific options keyed by plugin name
    #[serde(default)]
    pub plugin_options: HashMap<String, serde_json::Value>,
}

/// A plugin entry in the configuration.
///
/// Can be either a simple string (plugin name) or an object with name and options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginEntry {
    /// Simple plugin name (e.g., "pixelguard-plugin-s3")
    Name(String),

    /// Plugin with inline options
    WithOptions {
        /// Plugin package name
        name: String,

        /// Plugin-specific options
        #[serde(default)]
        options: serde_json::Value,
    },
}

impl PluginEntry {
    /// Returns the plugin name regardless of entry type.
    pub fn name(&self) -> &str {
        match self {
            PluginEntry::Name(name) => name,
            PluginEntry::WithOptions { name, .. } => name,
        }
    }

    /// Returns the inline options if present.
    pub fn options(&self) -> Option<&serde_json::Value> {
        match self {
            PluginEntry::Name(_) => None,
            PluginEntry::WithOptions { options, .. } => Some(options),
        }
    }
}

/// Viewport dimensions for screenshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    /// Width in pixels
    #[serde(default = "default_viewport_width")]
    pub width: u32,

    /// Height in pixels
    #[serde(default = "default_viewport_height")]
    pub height: u32,
}

/// A single screenshot configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shot {
    /// Unique name for this shot
    pub name: String,

    /// URL path to navigate to
    pub path: String,

    /// Optional CSS selector to wait for before capturing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for: Option<String>,

    /// Optional delay in milliseconds after page load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<u64>,
}

fn default_include() -> Vec<String> {
    vec!["**/*".to_string()]
}

fn default_threshold() -> f64 {
    0.01
}

fn default_output_dir() -> String {
    ".pixelguard".to_string()
}

fn default_viewport_width() -> u32 {
    1280
}

fn default_viewport_height() -> u32 {
    720
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: default_viewport_width(),
            height: default_viewport_height(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source: String::new(),
            base_url: String::new(),
            include: default_include(),
            exclude: Vec::new(),
            viewport: Viewport::default(),
            threshold: default_threshold(),
            output_dir: default_output_dir(),
            shots: Vec::new(),
            plugins: Vec::new(),
            plugin_options: HashMap::new(),
        }
    }
}

impl Config {
    /// Creates a new Config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            anyhow::anyhow!(
                "Could not read config file at '{}'. \
                 Run 'pixelguard init' to create one. Error: {}",
                path.display(),
                e
            )
        })?;

        let config: Config = serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Invalid JSON in config file '{}'. Error: {}",
                path.display(),
                e
            )
        })?;

        Ok(config)
    }

    /// Loads configuration from the default location in a directory,
    /// or returns default configuration if the file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be parsed.
    pub fn load_or_default<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let config_path = dir.as_ref().join("pixelguard.config.json");

        if config_path.exists() {
            Self::load(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Saves configuration to a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Saves configuration to the default location in a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save_to_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let config_path = dir.as_ref().join("pixelguard.config.json");
        self.save(&config_path)
    }

    /// Returns the path to the config file in a directory.
    pub fn config_path<P: AsRef<Path>>(dir: P) -> std::path::PathBuf {
        dir.as_ref().join("pixelguard.config.json")
    }

    /// Checks if a config file exists in the given directory.
    pub fn exists<P: AsRef<Path>>(dir: P) -> bool {
        Self::config_path(dir).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_config_has_sensible_values() {
        let config = Config::default();

        assert_eq!(config.viewport.width, 1280);
        assert_eq!(config.viewport.height, 720);
        assert_eq!(config.threshold, 0.01);
        assert_eq!(config.output_dir, ".pixelguard");
        assert_eq!(config.include, vec!["**/*"]);
    }

    #[test]
    fn config_serializes_to_json() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        assert!(json.contains("\"viewport\""));
        assert!(json.contains("\"threshold\""));
    }

    #[test]
    fn config_deserializes_from_json() {
        let json = r#"{
            "source": "storybook",
            "baseUrl": "http://localhost:6006",
            "viewport": {
                "width": 1920,
                "height": 1080
            }
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.source, "storybook");
        assert_eq!(config.base_url, "http://localhost:6006");
        assert_eq!(config.viewport.width, 1920);
        assert_eq!(config.viewport.height, 1080);
    }

    #[test]
    fn config_uses_defaults_for_missing_fields() {
        let json = r#"{"source": "storybook"}"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.source, "storybook");
        assert_eq!(config.viewport.width, 1280);
        assert_eq!(config.threshold, 0.01);
    }

    #[test]
    fn config_saves_and_loads() {
        let dir = tempdir().unwrap();
        let config = Config {
            source: "storybook".to_string(),
            base_url: "http://localhost:6006".to_string(),
            ..Default::default()
        };

        config.save_to_dir(dir.path()).unwrap();

        let loaded = Config::load_or_default(dir.path()).unwrap();

        assert_eq!(loaded.source, "storybook");
        assert_eq!(loaded.base_url, "http://localhost:6006");
    }

    #[test]
    fn load_or_default_returns_default_when_missing() {
        let dir = tempdir().unwrap();
        let config = Config::load_or_default(dir.path()).unwrap();

        assert_eq!(config.source, "");
        assert_eq!(config.viewport.width, 1280);
    }

    #[test]
    fn config_exists_returns_false_when_missing() {
        let dir = tempdir().unwrap();
        assert!(!Config::exists(dir.path()));
    }

    #[test]
    fn config_exists_returns_true_when_present() {
        let dir = tempdir().unwrap();
        let config = Config::default();
        config.save_to_dir(dir.path()).unwrap();

        assert!(Config::exists(dir.path()));
    }

    #[test]
    fn config_parses_simple_plugin_names() {
        let json = r#"{
            "plugins": ["pixelguard-plugin-s3", "pixelguard-plugin-slack"]
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.plugins.len(), 2);
        assert_eq!(config.plugins[0].name(), "pixelguard-plugin-s3");
        assert_eq!(config.plugins[1].name(), "pixelguard-plugin-slack");
    }

    #[test]
    fn config_parses_plugin_with_options() {
        let json = r#"{
            "plugins": [
                {
                    "name": "pixelguard-plugin-slack",
                    "options": {
                        "webhookUrl": "https://hooks.slack.com/test"
                    }
                }
            ]
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.plugins.len(), 1);
        assert_eq!(config.plugins[0].name(), "pixelguard-plugin-slack");

        let options = config.plugins[0].options().unwrap();
        assert_eq!(options["webhookUrl"], "https://hooks.slack.com/test");
    }

    #[test]
    fn config_parses_mixed_plugin_entries() {
        let json = r##"{
            "plugins": [
                "pixelguard-plugin-s3",
                {
                    "name": "pixelguard-plugin-slack",
                    "options": { "channel": "#testing" }
                }
            ],
            "pluginOptions": {
                "pixelguard-plugin-s3": {
                    "bucket": "my-bucket",
                    "region": "us-east-1"
                }
            }
        }"##;

        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.plugins.len(), 2);
        assert_eq!(config.plugins[0].name(), "pixelguard-plugin-s3");
        assert!(config.plugins[0].options().is_none());

        assert_eq!(config.plugins[1].name(), "pixelguard-plugin-slack");
        assert!(config.plugins[1].options().is_some());

        assert!(config.plugin_options.contains_key("pixelguard-plugin-s3"));
        assert_eq!(
            config.plugin_options["pixelguard-plugin-s3"]["bucket"],
            "my-bucket"
        );
    }
}
