//! Type definitions for the plugin system.
//!
//! This module contains all the core types used throughout the plugin system,
//! including plugin categories, manifests, and loaded plugin representations.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Plugin categories that can extend Pixelguard functionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginCategory {
    /// Storage plugins handle where baselines are stored (local, S3, R2, etc.)
    Storage,

    /// Reporter plugins generate reports in different formats (HTML, JSON, JUnit, etc.)
    Reporter,

    /// Capture plugins provide alternative screenshot engines (Playwright, Puppeteer, etc.)
    Capture,

    /// Differ plugins provide alternative image comparison algorithms
    Differ,

    /// Notifier plugins send results to external services (Slack, Teams, etc.)
    Notifier,
}

impl PluginCategory {
    /// Returns whether multiple plugins of this category can be active simultaneously.
    ///
    /// Notifiers and reporters can stack (all are called), while storage, capture,
    /// and differ only use the last configured plugin.
    pub fn can_stack(&self) -> bool {
        matches!(self, PluginCategory::Notifier | PluginCategory::Reporter)
    }
}

/// Plugin manifest loaded from package.json's "pixelguard" field.
///
/// This struct represents the metadata about a plugin as declared
/// in its package.json file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    /// Human-readable plugin name
    pub name: String,

    /// Plugin category
    pub category: PluginCategory,

    /// Entry point relative to package root
    pub entry: String,

    /// Hooks this plugin implements
    pub hooks: Vec<String>,

    /// Plugin version (from package.json version field)
    #[serde(default)]
    pub version: String,

    /// Optional JSON schema for plugin options validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_schema: Option<serde_json::Value>,
}

/// A fully loaded and validated plugin ready for execution.
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    /// Plugin manifest from package.json
    pub manifest: PluginManifest,

    /// Path to the plugin package directory
    pub package_path: PathBuf,

    /// Full path to the plugin entry point
    pub entry_path: PathBuf,

    /// Resolved options for this plugin (merged from config)
    pub options: serde_json::Value,
}

impl LoadedPlugin {
    /// Returns the plugin's display name.
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// Returns the plugin's category.
    pub fn category(&self) -> PluginCategory {
        self.manifest.category
    }

    /// Checks if the plugin implements a specific hook.
    pub fn has_hook(&self, hook: &str) -> bool {
        self.manifest.hooks.iter().any(|h| h == hook)
    }
}

/// Result from executing a plugin hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    /// Whether the hook executed successfully
    pub success: bool,

    /// Data returned by the hook (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl PluginResult {
    /// Creates a successful result with data.
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Creates a successful result without data.
    pub fn ok_empty() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
        }
    }

    /// Creates a failed result with an error message.
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Input data for storage plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInput {
    /// The relative path to the file
    pub path: String,

    /// Base64-encoded file data (for write operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,

    /// Plugin options
    #[serde(default)]
    pub options: serde_json::Value,
}

/// Output data from storage plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageOutput {
    /// Base64-encoded file data (for read operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,

    /// Whether the file exists (for exists operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,

    /// List of files (for list operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
}

/// Input data for capture plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureInput {
    /// Shots to capture
    pub shots: Vec<CaptureShot>,

    /// Base URL of the dev server
    pub base_url: String,

    /// Viewport dimensions
    pub viewport: CaptureViewport,

    /// Output directory for screenshots
    pub output_dir: String,

    /// Plugin options
    #[serde(default)]
    pub options: serde_json::Value,
}

/// A shot to capture (simplified for plugin input).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureShot {
    /// Shot name
    pub name: String,

    /// URL path
    pub path: String,

    /// Optional wait selector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for: Option<String>,

    /// Optional delay in ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<u64>,
}

/// Viewport for capture input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureViewport {
    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,
}

/// Output data from capture plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureOutput {
    /// Successfully captured shots
    pub captured: Vec<CapturedShot>,

    /// Failed shots
    pub failed: Vec<FailedShot>,
}

/// A successfully captured shot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedShot {
    /// Shot name
    pub name: String,

    /// Path to the screenshot file
    pub path: String,
}

/// A failed shot capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedShot {
    /// Shot name
    pub name: String,

    /// Error message
    pub error: String,
}

/// Input data for differ plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifferInput {
    /// Path to baseline image
    pub baseline_path: String,

    /// Path to current image
    pub current_path: String,

    /// Path to write diff image
    pub diff_path: String,

    /// Diff threshold (0.0 to 1.0)
    pub threshold: f64,

    /// Plugin options
    #[serde(default)]
    pub options: serde_json::Value,
}

/// Output data from differ plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifferOutput {
    /// Percentage of pixels that differ
    pub diff_percentage: f64,

    /// Whether images match (within threshold)
    pub matches: bool,
}

/// Input data for reporter plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReporterInput {
    /// Diff results
    pub result: ReporterDiffResult,

    /// Configuration summary
    pub config: ReporterConfig,

    /// Output directory
    pub output_dir: String,

    /// Plugin options
    #[serde(default)]
    pub options: serde_json::Value,
}

/// Diff result for reporter input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReporterDiffResult {
    /// Unchanged shots
    pub unchanged: Vec<String>,

    /// Changed shots
    pub changed: Vec<ReporterChangedShot>,

    /// Added shots (new, no baseline)
    pub added: Vec<String>,

    /// Removed shots (baseline exists, no current)
    pub removed: Vec<String>,
}

/// A changed shot for reporter input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReporterChangedShot {
    /// Shot name
    pub name: String,

    /// Path to baseline image
    pub baseline_path: String,

    /// Path to current image
    pub current_path: String,

    /// Path to diff image
    pub diff_path: String,

    /// Diff percentage
    pub diff_percentage: f64,
}

/// Config summary for reporter input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReporterConfig {
    /// Source type
    pub source: String,

    /// Base URL
    pub base_url: String,

    /// Threshold
    pub threshold: f64,
}

/// Output data from reporter plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReporterOutput {
    /// Path to generated report (if local)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,

    /// URL to report (if hosted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_url: Option<String>,
}

/// Input data for notifier plugin hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifierInput {
    /// Diff results
    pub result: ReporterDiffResult,

    /// Path to generated report
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,

    /// URL to report
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_url: Option<String>,

    /// Whether running in CI mode
    pub ci_mode: bool,

    /// Plugin options
    #[serde(default)]
    pub options: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_category_can_stack() {
        assert!(PluginCategory::Notifier.can_stack());
        assert!(PluginCategory::Reporter.can_stack());
        assert!(!PluginCategory::Storage.can_stack());
        assert!(!PluginCategory::Capture.can_stack());
        assert!(!PluginCategory::Differ.can_stack());
    }

    #[test]
    fn plugin_manifest_deserializes() {
        let json = r#"{
            "name": "S3 Storage",
            "category": "storage",
            "entry": "dist/index.js",
            "hooks": ["read", "write", "exists"]
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json).unwrap();

        assert_eq!(manifest.name, "S3 Storage");
        assert_eq!(manifest.category, PluginCategory::Storage);
        assert_eq!(manifest.entry, "dist/index.js");
        assert_eq!(manifest.hooks, vec!["read", "write", "exists"]);
    }

    #[test]
    fn plugin_result_ok() {
        let result = PluginResult::ok(serde_json::json!({"test": true}));

        assert!(result.success);
        assert!(result.data.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn plugin_result_err() {
        let result = PluginResult::err("Something went wrong");

        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error.unwrap(), "Something went wrong");
    }
}
