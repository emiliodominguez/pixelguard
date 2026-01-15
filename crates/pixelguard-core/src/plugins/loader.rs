//! Plugin loading and manifest parsing.
//!
//! This module handles reading package.json files and extracting
//! the Pixelguard plugin manifest from them.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use super::types::{LoadedPlugin, PluginManifest};
use crate::config::PluginEntry;

/// Partial package.json structure for plugin loading.
#[derive(Debug, Deserialize)]
struct PackageJson {
    /// Package name
    name: String,

    /// Package version
    #[serde(default)]
    version: String,

    /// Main entry point
    #[serde(default)]
    main: String,

    /// Pixelguard plugin manifest
    pixelguard: PluginManifest,
}

/// Loads a plugin from its package path.
///
/// Reads the package.json, extracts the pixelguard manifest,
/// and creates a LoadedPlugin with resolved paths and options.
///
/// # Arguments
///
/// * `entry` - The plugin entry from config
/// * `package_path` - Path to the plugin package directory
/// * `global_options` - Plugin options from config's pluginOptions field
///
/// # Returns
///
/// A fully loaded and validated plugin.
///
/// # Errors
///
/// Returns an error if the package.json cannot be read or is invalid.
pub fn load_plugin(
    entry: &PluginEntry,
    package_path: &Path,
    global_options: &serde_json::Value,
) -> Result<LoadedPlugin> {
    let package_json_path = package_path.join("package.json");

    let content = std::fs::read_to_string(&package_json_path)
        .with_context(|| format!("Failed to read {}", package_json_path.display()))?;

    let package: PackageJson = serde_json::from_str(&content)
        .with_context(|| format!("Invalid package.json in {}", package_path.display()))?;

    // Resolve entry point
    let entry_file = if package.pixelguard.entry.is_empty() {
        if package.main.is_empty() {
            "index.js".to_string()
        } else {
            package.main
        }
    } else {
        package.pixelguard.entry.clone()
    };

    let entry_path = package_path.join(&entry_file);

    if !entry_path.exists() {
        anyhow::bail!(
            "❌ Plugin '{}' entry point '{}' not found at '{}'.\n\n\
             💡 Solutions:\n  \
             • Check plugin is installed: npm list {}\n  \
             • Reinstall the plugin: npm install {}\n  \
             • Verify package.json 'main' field is correct",
            package.name,
            entry_file,
            entry_path.display(),
            package.name,
            package.name
        );
    }

    // Merge options: global options take precedence, then inline options
    let options = merge_options(entry, global_options);

    // Create manifest with version from package.json
    let mut manifest = package.pixelguard;
    if manifest.version.is_empty() {
        manifest.version = package.version;
    }

    Ok(LoadedPlugin {
        manifest,
        package_path: package_path.to_path_buf(),
        entry_path,
        options,
    })
}

/// Merges plugin options from different sources.
///
/// Priority (highest to lowest):
/// 1. Global pluginOptions from config
/// 2. Inline options in the plugin entry
/// 3. Empty object
fn merge_options(entry: &PluginEntry, global_options: &serde_json::Value) -> serde_json::Value {
    let mut result = serde_json::json!({});

    // Start with inline options
    if let Some(serde_json::Value::Object(map)) = entry.options() {
        if let serde_json::Value::Object(ref mut result_map) = result {
            for (k, v) in map {
                result_map.insert(k.clone(), v.clone());
            }
        }
    }

    // Override with global options
    if let serde_json::Value::Object(map) = global_options {
        if let serde_json::Value::Object(ref mut result_map) = result {
            for (k, v) in map {
                result_map.insert(k.clone(), v.clone());
            }
        }
    }

    result
}

/// Validates a loaded plugin's manifest.
///
/// Checks that required fields are present and valid.
pub fn validate_manifest(plugin: &LoadedPlugin) -> Result<()> {
    let manifest = &plugin.manifest;

    if manifest.name.is_empty() {
        anyhow::bail!(
            "❌ Plugin at '{}' has empty name in pixelguard manifest.\n\n\
             💡 Solution: Add 'name' field to 'pixelguard' section in package.json",
            plugin.package_path.display()
        );
    }

    if manifest.hooks.is_empty() {
        anyhow::bail!(
            "❌ Plugin '{}' has no hooks defined.\n\n\
             💡 Solution: At least one hook is required in the 'pixelguard' manifest.\n\n\
             📚 Example:\n  \
             {{\n    \
             \"pixelguard\": {{\n      \
             \"hooks\": [\"notify\"]\n    \
             }}\n  \
             }}",
            manifest.name
        );
    }

    // Validate hooks for the category
    validate_hooks_for_category(plugin)?;

    Ok(())
}

/// Validates that the plugin's hooks are valid for its category.
fn validate_hooks_for_category(plugin: &LoadedPlugin) -> Result<()> {
    use super::types::PluginCategory;

    let valid_hooks: &[&str] = match plugin.manifest.category {
        PluginCategory::Storage => &["read", "write", "exists", "list", "delete"],
        PluginCategory::Reporter => &["generate"],
        PluginCategory::Capture => &["capture"],
        PluginCategory::Differ => &["compare"],
        PluginCategory::Notifier => &["notify"],
    };

    for hook in &plugin.manifest.hooks {
        if !valid_hooks.contains(&hook.as_str()) {
            anyhow::bail!(
                "❌ Plugin '{}' declares invalid hook '{}' for category {:?}.\n\n\
                 ✅ Valid hooks for {:?}: {:?}\n\n\
                 💡 Solution: Update the 'hooks' array in package.json to use valid hooks.",
                plugin.manifest.name,
                hook,
                plugin.manifest.category,
                plugin.manifest.category,
                valid_hooks
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PluginEntry;
    use tempfile::tempdir;

    fn create_test_plugin(dir: &Path, manifest: &str) {
        std::fs::write(
            dir.join("package.json"),
            format!(
                r#"{{
                    "name": "test-plugin",
                    "version": "1.0.0",
                    "main": "index.js",
                    "pixelguard": {}
                }}"#,
                manifest
            ),
        )
        .unwrap();
        std::fs::write(dir.join("index.js"), "module.exports = {}").unwrap();
    }

    #[test]
    fn load_plugin_basic() {
        let dir = tempdir().unwrap();
        create_test_plugin(
            dir.path(),
            r#"{
                "name": "Test Plugin",
                "category": "storage",
                "entry": "index.js",
                "hooks": ["read", "write"]
            }"#,
        );

        let entry = PluginEntry::Name("test-plugin".to_string());
        let plugin = load_plugin(&entry, dir.path(), &serde_json::json!({})).unwrap();

        assert_eq!(plugin.manifest.name, "Test Plugin");
        assert_eq!(plugin.manifest.hooks, vec!["read", "write"]);
        assert_eq!(plugin.manifest.version, "1.0.0");
    }

    #[test]
    fn load_plugin_with_inline_options() {
        let dir = tempdir().unwrap();
        create_test_plugin(
            dir.path(),
            r#"{
                "name": "Test Plugin",
                "category": "storage",
                "entry": "index.js",
                "hooks": ["read"]
            }"#,
        );

        let entry = PluginEntry::WithOptions {
            name: "test-plugin".to_string(),
            options: serde_json::json!({"bucket": "test-bucket"}),
        };
        let plugin = load_plugin(&entry, dir.path(), &serde_json::json!({})).unwrap();

        assert_eq!(plugin.options["bucket"], "test-bucket");
    }

    #[test]
    fn load_plugin_global_options_override() {
        let dir = tempdir().unwrap();
        create_test_plugin(
            dir.path(),
            r#"{
                "name": "Test Plugin",
                "category": "storage",
                "entry": "index.js",
                "hooks": ["read"]
            }"#,
        );

        let entry = PluginEntry::WithOptions {
            name: "test-plugin".to_string(),
            options: serde_json::json!({"bucket": "inline-bucket", "region": "us-west-1"}),
        };
        let global = serde_json::json!({"bucket": "global-bucket"});
        let plugin = load_plugin(&entry, dir.path(), &global).unwrap();

        // Global should override inline
        assert_eq!(plugin.options["bucket"], "global-bucket");
        // Inline should be preserved for non-overridden keys
        assert_eq!(plugin.options["region"], "us-west-1");
    }

    #[test]
    fn validate_manifest_empty_name() {
        let dir = tempdir().unwrap();
        create_test_plugin(
            dir.path(),
            r#"{
                "name": "",
                "category": "storage",
                "entry": "index.js",
                "hooks": ["read"]
            }"#,
        );

        let entry = PluginEntry::Name("test-plugin".to_string());
        let plugin = load_plugin(&entry, dir.path(), &serde_json::json!({})).unwrap();
        let result = validate_manifest(&plugin);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty name"));
    }

    #[test]
    fn validate_manifest_no_hooks() {
        let dir = tempdir().unwrap();
        create_test_plugin(
            dir.path(),
            r#"{
                "name": "Test Plugin",
                "category": "storage",
                "entry": "index.js",
                "hooks": []
            }"#,
        );

        let entry = PluginEntry::Name("test-plugin".to_string());
        let plugin = load_plugin(&entry, dir.path(), &serde_json::json!({})).unwrap();
        let result = validate_manifest(&plugin);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no hooks"));
    }

    #[test]
    fn validate_manifest_invalid_hook() {
        let dir = tempdir().unwrap();
        create_test_plugin(
            dir.path(),
            r#"{
                "name": "Test Plugin",
                "category": "storage",
                "entry": "index.js",
                "hooks": ["read", "invalidHook"]
            }"#,
        );

        let entry = PluginEntry::Name("test-plugin".to_string());
        let plugin = load_plugin(&entry, dir.path(), &serde_json::json!({})).unwrap();
        let result = validate_manifest(&plugin);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid hook"));
    }
}
