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

    /// Port to use for dev server detection (overrides default port probing)
    #[serde(default)]
    pub port: Option<u16>,

    /// Glob patterns to include
    #[serde(default = "default_include")]
    pub include: Vec<String>,

    /// Glob patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Default viewport configuration (used when `viewports` is empty)
    #[serde(default)]
    pub viewport: Viewport,

    /// Multiple named viewports for responsive testing.
    ///
    /// When set, each shot is captured at each viewport size.
    /// Screenshots are named `{shot}@{viewport}.png`.
    #[serde(default)]
    pub viewports: Vec<NamedViewport>,

    /// Diff threshold (0.0 to 1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f64,

    /// Output directory for screenshots and reports
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// Number of concurrent screenshot captures (default: 4)
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

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

/// A named viewport for multi-viewport testing.
///
/// When multiple viewports are configured, each shot is captured at each viewport size,
/// with screenshots named `{shot}@{viewport}.png`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedViewport {
    /// Unique name for this viewport (e.g., "desktop", "mobile")
    pub name: String,

    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,
}

/// A single screenshot configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shot {
    /// Unique name for this shot
    pub name: String,

    /// URL path to navigate to, appended to `baseUrl`.
    ///
    /// Optional so Storybook overrides can list only `name` plus `delay`/`waitFor`.
    #[serde(default)]
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

fn default_concurrency() -> usize {
    4
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
            port: None,
            include: default_include(),
            exclude: Vec::new(),
            viewport: Viewport::default(),
            viewports: Vec::new(),
            threshold: default_threshold(),
            output_dir: default_output_dir(),
            concurrency: default_concurrency(),
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
                "❌ Could not read config file at '{}'.\n\n\
                 💡 Solutions:\n  \
                 1️⃣ Run 'pixelguard init' to create a configuration\n  \
                 2️⃣ Check file permissions\n  \
                 3️⃣ Verify the file exists at the expected location\n\n\
                 🔍 Error details: {}",
                path.display(),
                e
            )
        })?;

        let config: Config = serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "❌ Invalid JSON in config file '{}'.\n\n\
                 🔍 Error details: {}\n\n\
                 💡 Common issues:\n  \
                 • Missing commas between properties\n  \
                 • Trailing commas (not allowed in JSON)\n  \
                 • Unquoted property names\n  \
                 • Invalid escape sequences\n\n\
                 🛠️  Use a JSON validator to check syntax: https://jsonlint.com",
                path.display(),
                e
            )
        })?;

        config.validate().map_err(|e| {
            anyhow::anyhow!(
                "❌ Invalid configuration in '{}'.\n\n🔍 {}",
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

    /// Returns the viewports to use for testing.
    ///
    /// If `viewports` is configured, returns those. Otherwise, returns a single
    /// viewport named "default" based on the legacy `viewport` field.
    ///
    /// This provides backward compatibility: configs with only `viewport` work
    /// as before, while configs with `viewports` enable multi-viewport testing.
    pub fn effective_viewports(&self) -> Vec<NamedViewport> {
        if !self.viewports.is_empty() {
            self.viewports.clone()
        } else {
            vec![NamedViewport {
                name: "default".to_string(),
                width: self.viewport.width,
                height: self.viewport.height,
            }]
        }
    }

    /// Returns the screenshot names this config produces, one per shot and viewport.
    ///
    /// Names follow the on-disk convention: `{shot}` for the default viewport and
    /// `{shot}@{viewport}` for named viewports. Use this to know exactly which
    /// screenshots a run is expected to produce, instead of trusting whatever
    /// happens to be on disk.
    pub fn screenshot_names(&self) -> Vec<String> {
        let viewports = self.effective_viewports();
        let mut names = Vec::with_capacity(self.shots.len() * viewports.len());
        for shot in &self.shots {
            for viewport in &viewports {
                names.push(screenshot_name(&shot.name, &viewport.name));
            }
        }
        names
    }

    /// Validates every value that later ends up in file paths, URLs, generated
    /// scripts or loop bounds.
    ///
    /// Called by [`Config::load`]. Call it again after mutating `shots` (for
    /// example after Storybook discovery) so remotely discovered names get the
    /// same checks as hand-written ones.
    ///
    /// # Errors
    ///
    /// Returns an error describing the first invalid value found.
    pub fn validate(&self) -> Result<()> {
        if !self.threshold.is_finite() || !(0.0..=100.0).contains(&self.threshold) {
            anyhow::bail!(
                "'threshold' must be a percentage between 0 and 100, got {}.",
                self.threshold
            );
        }

        if self.concurrency == 0 || self.concurrency > MAX_CONCURRENCY {
            anyhow::bail!(
                "'concurrency' must be between 1 and {}, got {}.",
                MAX_CONCURRENCY,
                self.concurrency
            );
        }

        if self.output_dir.trim().is_empty() {
            anyhow::bail!("'outputDir' must not be empty. The default is '.pixelguard'.");
        }

        if !self.base_url.is_empty() {
            validate_base_url(&self.base_url)?;
        }

        validate_dimensions("viewport", self.viewport.width, self.viewport.height)?;

        let mut viewport_names = std::collections::HashSet::new();
        for viewport in &self.viewports {
            validate_name("viewport", &viewport.name)?;
            validate_dimensions(&viewport.name, viewport.width, viewport.height)?;
            if !viewport_names.insert(viewport.name.as_str()) {
                anyhow::bail!("Duplicate viewport name '{}'.", viewport.name);
            }
        }

        let mut shot_names = std::collections::HashSet::new();
        for shot in &self.shots {
            validate_shot(shot)?;
            if !shot_names.insert(shot.name.as_str()) {
                anyhow::bail!(
                    "Duplicate shot name '{}'. Each shot needs a unique name because it \
                     becomes the screenshot file name.",
                    shot.name
                );
            }
        }

        Ok(())
    }
}

/// Upper bound for `concurrency`; each unit is a live browser context.
pub const MAX_CONCURRENCY: usize = 64;

/// Upper bound for viewport width and height in pixels.
pub const MAX_VIEWPORT_DIMENSION: u32 = 16_384;

/// Upper bound for a shot's `delay` (5 minutes).
pub const MAX_DELAY_MS: u64 = 300_000;

/// Upper bound for shot and viewport name length.
const MAX_NAME_LENGTH: usize = 200;

/// Builds the screenshot name for a shot at a viewport.
///
/// The `default` viewport keeps the bare shot name for backward compatibility.
pub fn screenshot_name(shot_name: &str, viewport_name: &str) -> String {
    if viewport_name == "default" {
        shot_name.to_string()
    } else {
        format!("{}@{}", shot_name, viewport_name)
    }
}

/// Validates a single shot definition.
///
/// # Errors
///
/// Returns an error if the name is not filesystem-safe, the path cannot be
/// appended to the base URL, or the delay is out of range.
pub fn validate_shot(shot: &Shot) -> Result<()> {
    validate_name("shot", &shot.name)?;

    if !shot.path.is_empty() && !shot.path.starts_with('/') {
        anyhow::bail!(
            "Shot '{}' has path '{}'. Paths must start with '/' because they are \
             appended to 'baseUrl'.",
            shot.name,
            shot.path
        );
    }
    if shot.path.starts_with("//") {
        anyhow::bail!(
            "Shot '{}' has path '{}'. A path starting with '//' would navigate away \
             from 'baseUrl'.",
            shot.name,
            shot.path
        );
    }
    if shot.path.chars().any(char::is_control) {
        anyhow::bail!("Shot '{}' has control characters in its path.", shot.name);
    }

    if let Some(delay) = shot.delay {
        if delay > MAX_DELAY_MS {
            anyhow::bail!(
                "Shot '{}' has delay {}ms; the maximum is {}ms.",
                shot.name,
                delay,
                MAX_DELAY_MS
            );
        }
    }

    Ok(())
}

/// Validates a shot or viewport name that becomes part of a file name.
///
/// Rejects anything that could escape the screenshot directory, collide with
/// the `shot@viewport` naming scheme, break report URLs, or is not portable
/// across filesystems.
///
/// # Errors
///
/// Returns an error naming the offending value and the rule it broke.
pub fn validate_name(kind: &str, name: &str) -> Result<()> {
    const FORBIDDEN: &[char] = &['/', '\\', '@', '<', '>', ':', '"', '|', '?', '*', '#', '%'];

    if name.is_empty() {
        anyhow::bail!("A {} name must not be empty.", kind);
    }
    if name.len() > MAX_NAME_LENGTH {
        anyhow::bail!(
            "The {} name '{}…' is longer than {} bytes.",
            kind,
            name.chars().take(40).collect::<String>(),
            MAX_NAME_LENGTH
        );
    }
    if name.starts_with('.') || name.ends_with('.') || name.trim() != name {
        anyhow::bail!(
            "The {} name '{}' must not start or end with '.' or whitespace.",
            kind,
            name
        );
    }
    if let Some(bad) = name
        .chars()
        .find(|c| FORBIDDEN.contains(c) || c.is_control())
    {
        anyhow::bail!(
            "The {} name '{}' contains '{}'. Names become file names, so these \
             characters are not allowed: / \\ @ < > : \" | ? * # % and control characters.",
            kind,
            name,
            bad.escape_default()
        );
    }

    Ok(())
}

fn validate_dimensions(label: &str, width: u32, height: u32) -> Result<()> {
    let valid = 1..=MAX_VIEWPORT_DIMENSION;
    if !valid.contains(&width) || !valid.contains(&height) {
        anyhow::bail!(
            "Viewport '{}' is {}x{}; width and height must be between 1 and {}.",
            label,
            width,
            height,
            MAX_VIEWPORT_DIMENSION
        );
    }
    Ok(())
}

fn validate_base_url(base_url: &str) -> Result<()> {
    let scheme_ok = ["http://", "https://", "file://"]
        .iter()
        .any(|scheme| base_url.to_ascii_lowercase().starts_with(scheme));
    if !scheme_ok {
        anyhow::bail!(
            "'baseUrl' must start with http://, https:// or file://, got '{}'.",
            base_url
        );
    }
    if base_url
        .chars()
        .any(|c| c.is_control() || c.is_whitespace())
    {
        anyhow::bail!("'baseUrl' must not contain whitespace or control characters.");
    }
    Ok(())
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

    #[test]
    fn effective_viewports_returns_default_when_viewports_empty() {
        let config = Config {
            viewport: Viewport {
                width: 1920,
                height: 1080,
            },
            viewports: Vec::new(),
            ..Default::default()
        };

        let viewports = config.effective_viewports();
        assert_eq!(viewports.len(), 1);
        assert_eq!(viewports[0].name, "default");
        assert_eq!(viewports[0].width, 1920);
        assert_eq!(viewports[0].height, 1080);
    }

    #[test]
    fn effective_viewports_returns_configured_viewports() {
        let config = Config {
            viewports: vec![
                NamedViewport {
                    name: "desktop".to_string(),
                    width: 1920,
                    height: 1080,
                },
                NamedViewport {
                    name: "mobile".to_string(),
                    width: 375,
                    height: 667,
                },
            ],
            ..Default::default()
        };

        let viewports = config.effective_viewports();
        assert_eq!(viewports.len(), 2);
        assert_eq!(viewports[0].name, "desktop");
        assert_eq!(viewports[1].name, "mobile");
    }

    #[test]
    fn config_parses_viewports_from_json() {
        let json = r#"{
            "viewports": [
                { "name": "desktop", "width": 1920, "height": 1080 },
                { "name": "tablet", "width": 768, "height": 1024 },
                { "name": "mobile", "width": 375, "height": 667 }
            ]
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.viewports.len(), 3);
        assert_eq!(config.viewports[0].name, "desktop");
        assert_eq!(config.viewports[1].name, "tablet");
        assert_eq!(config.viewports[2].name, "mobile");
    }
}
