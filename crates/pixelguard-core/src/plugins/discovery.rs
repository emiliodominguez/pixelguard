//! Plugin discovery from node_modules.
//!
//! This module handles finding plugin packages based on the explicit
//! plugin list in the configuration.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::PluginEntry;

/// Resolves plugin entries to their package paths.
///
/// Takes the list of plugin entries from config and resolves each
/// to its actual path in node_modules or as a local path.
///
/// # Arguments
///
/// * `plugins` - List of plugin entries from config
/// * `working_dir` - The project's working directory
///
/// # Returns
///
/// A vector of resolved plugin paths paired with their original entry.
///
/// # Errors
///
/// Returns an error if a plugin cannot be found.
pub fn resolve_plugins(
    plugins: &[PluginEntry],
    working_dir: &Path,
) -> Result<Vec<(PluginEntry, PathBuf)>> {
    let mut resolved = Vec::new();

    for plugin in plugins {
        let name = plugin.name();
        let path = resolve_plugin_path(name, working_dir)?;
        resolved.push((plugin.clone(), path));
    }

    Ok(resolved)
}

/// Resolves a single plugin name to its package path.
///
/// Supports:
/// - Local paths starting with `.` or `/`
/// - npm package names in node_modules
/// - Scoped packages like `@scope/pixelguard-plugin-x`
fn resolve_plugin_path(name: &str, working_dir: &Path) -> Result<PathBuf> {
    // Handle local paths
    if name.starts_with('.') || name.starts_with('/') {
        let path = if name.starts_with('/') {
            PathBuf::from(name)
        } else {
            working_dir.join(name)
        };

        if path.exists() {
            return Ok(path);
        } else {
            anyhow::bail!(
                "❌ Local plugin '{}' not found at '{}'.\n\n\
                 💡 Solutions:\n  \
                 • Check that the path is correct\n  \
                 • Verify the plugin directory exists\n  \
                 • Use relative path starting with './'",
                name,
                path.display()
            );
        }
    }

    // Handle npm packages
    let node_modules = working_dir.join("node_modules");
    let package_path = node_modules.join(name);

    if package_path.exists() {
        return Ok(package_path);
    }

    // Try to find in parent node_modules (for monorepos)
    let mut current = working_dir.to_path_buf();
    while let Some(parent) = current.parent() {
        let parent_node_modules = parent.join("node_modules").join(name);
        if parent_node_modules.exists() {
            return Ok(parent_node_modules);
        }
        current = parent.to_path_buf();
    }

    anyhow::bail!(
        "❌ Plugin '{}' not found in node_modules.\n\n\
         💡 Solutions:\n  \
         1️⃣ Install it: npm install {}\n  \
         2️⃣ Or use pnpm: pnpm add {}\n  \
         3️⃣ Or use yarn: yarn add {}\n\n\
         📍 Check spelling in pixelguard.config.json",
        name,
        name,
        name,
        name
    )
}

/// Validates that a path is a valid plugin package.
///
/// Checks that the path contains a package.json with a pixelguard field.
pub fn validate_plugin_path(path: &Path) -> Result<()> {
    let package_json = path.join("package.json");

    if !package_json.exists() {
        anyhow::bail!(
            "❌ Plugin at '{}' is missing package.json.\n\n\
             💡 Solution: This doesn't appear to be a valid npm package.",
            path.display()
        );
    }

    let content = std::fs::read_to_string(&package_json)
        .with_context(|| format!("Failed to read {}", package_json.display()))?;

    if !content.contains("\"pixelguard\"") {
        anyhow::bail!(
            "❌ Plugin at '{}' is missing 'pixelguard' field in package.json.\n\n\
             💡 Solution: This doesn't appear to be a valid Pixelguard plugin.\n\n\
             📚 A valid plugin needs:\n  \
             {{\n    \
             \"pixelguard\": {{\n      \
             \"category\": \"notifier\",\n      \
             \"hooks\": [\"notify\"]\n    \
             }}\n  \
             }}",
            path.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolve_plugins_empty_list() {
        let dir = tempdir().unwrap();
        let result = resolve_plugins(&[], dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_local_plugin() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join("local-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("package.json"),
            r#"{"name": "local-plugin", "pixelguard": {}}"#,
        )
        .unwrap();

        let plugins = vec![PluginEntry::Name("./local-plugin".to_string())];
        let result = resolve_plugins(&plugins, dir.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, plugin_dir);
    }

    #[test]
    fn resolve_npm_plugin() {
        let dir = tempdir().unwrap();
        let node_modules = dir.path().join("node_modules");
        let plugin_dir = node_modules.join("pixelguard-plugin-test");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("package.json"),
            r#"{"name": "pixelguard-plugin-test", "pixelguard": {}}"#,
        )
        .unwrap();

        let plugins = vec![PluginEntry::Name("pixelguard-plugin-test".to_string())];
        let result = resolve_plugins(&plugins, dir.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, plugin_dir);
    }

    #[test]
    fn resolve_missing_plugin_fails() {
        let dir = tempdir().unwrap();
        let plugins = vec![PluginEntry::Name("nonexistent-plugin".to_string())];
        let result = resolve_plugins(&plugins, dir.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn validate_plugin_path_missing_package_json() {
        let dir = tempdir().unwrap();
        let result = validate_plugin_path(dir.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("package.json"));
    }

    #[test]
    fn validate_plugin_path_missing_pixelguard_field() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "not-a-plugin"}"#,
        )
        .unwrap();

        let result = validate_plugin_path(dir.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pixelguard"));
    }

    #[test]
    fn validate_plugin_path_valid() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "valid-plugin", "pixelguard": {"category": "storage"}}"#,
        )
        .unwrap();

        let result = validate_plugin_path(dir.path());
        assert!(result.is_ok());
    }
}
