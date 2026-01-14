//! Plugin registry for managing loaded plugins.
//!
//! The registry stores all loaded plugins organized by category and name,
//! providing lookup methods for the test command to find relevant plugins.

use std::collections::HashMap;

use super::types::{LoadedPlugin, PluginCategory};

/// Registry of loaded plugins.
///
/// Organizes plugins by category for efficient lookup. Handles the distinction
/// between stackable categories (notifiers, reporters) and single-winner
/// categories (storage, capture, differ).
#[derive(Debug, Default)]
pub struct PluginRegistry {
    /// Single plugin per category (storage, capture, differ)
    by_category: HashMap<PluginCategory, LoadedPlugin>,

    /// Stackable plugins: notifiers
    notifiers: Vec<LoadedPlugin>,

    /// Stackable plugins: reporters
    reporters: Vec<LoadedPlugin>,

    /// All plugins by name for direct lookup
    by_name: HashMap<String, LoadedPlugin>,
}

impl PluginRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a plugin in the registry.
    ///
    /// For stackable categories (notifier, reporter), all plugins are kept.
    /// For other categories, later plugins override earlier ones.
    pub fn register(&mut self, plugin: LoadedPlugin) {
        let name = plugin.name().to_string();
        let category = plugin.category();

        match category {
            PluginCategory::Notifier => {
                self.notifiers.push(plugin.clone());
            }
            PluginCategory::Reporter => {
                self.reporters.push(plugin.clone());
            }
            _ => {
                self.by_category.insert(category, plugin.clone());
            }
        }

        self.by_name.insert(name, plugin);
    }

    /// Gets the active plugin for a single-winner category.
    ///
    /// Returns `None` for stackable categories (use `notifiers()` or `reporters()` instead).
    pub fn get(&self, category: PluginCategory) -> Option<&LoadedPlugin> {
        if category.can_stack() {
            return None;
        }
        self.by_category.get(&category)
    }

    /// Gets a plugin by name.
    pub fn get_by_name(&self, name: &str) -> Option<&LoadedPlugin> {
        self.by_name.get(name)
    }

    /// Gets all registered notifier plugins.
    pub fn notifiers(&self) -> &[LoadedPlugin] {
        &self.notifiers
    }

    /// Gets all registered reporter plugins.
    pub fn reporters(&self) -> &[LoadedPlugin] {
        &self.reporters
    }

    /// Checks if a category has an override plugin registered.
    pub fn has_override(&self, category: PluginCategory) -> bool {
        match category {
            PluginCategory::Notifier => !self.notifiers.is_empty(),
            PluginCategory::Reporter => !self.reporters.is_empty(),
            _ => self.by_category.contains_key(&category),
        }
    }

    /// Returns the total number of registered plugins.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Returns true if no plugins are registered.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// Lists all registered plugin names.
    pub fn plugin_names(&self) -> Vec<&str> {
        self.by_name.keys().map(|s| s.as_str()).collect()
    }

    /// Gets all plugins of stackable categories plus any single-winner overrides.
    ///
    /// Useful for displaying all active plugins.
    pub fn all_active(&self) -> Vec<&LoadedPlugin> {
        let mut active: Vec<&LoadedPlugin> = Vec::new();

        // Add single-winner plugins
        for plugin in self.by_category.values() {
            active.push(plugin);
        }

        // Add stackable plugins
        active.extend(self.notifiers.iter());
        active.extend(self.reporters.iter());

        active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::types::PluginManifest;
    use std::path::PathBuf;

    fn create_test_plugin(name: &str, category: PluginCategory) -> LoadedPlugin {
        LoadedPlugin {
            manifest: PluginManifest {
                name: name.to_string(),
                category,
                entry: "index.js".to_string(),
                hooks: vec!["test".to_string()],
                version: "1.0.0".to_string(),
                options_schema: None,
            },
            package_path: PathBuf::from("/test"),
            entry_path: PathBuf::from("/test/index.js"),
            options: serde_json::json!({}),
        }
    }

    #[test]
    fn registry_new_is_empty() {
        let registry = PluginRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn registry_single_winner_category() {
        let mut registry = PluginRegistry::new();

        let plugin1 = create_test_plugin("Storage A", PluginCategory::Storage);
        let plugin2 = create_test_plugin("Storage B", PluginCategory::Storage);

        registry.register(plugin1);
        registry.register(plugin2);

        // Last one wins
        let active = registry.get(PluginCategory::Storage).unwrap();
        assert_eq!(active.name(), "Storage B");

        // Both are in by_name though
        assert!(registry.get_by_name("Storage A").is_some());
        assert!(registry.get_by_name("Storage B").is_some());
    }

    #[test]
    fn registry_stackable_notifiers() {
        let mut registry = PluginRegistry::new();

        let plugin1 = create_test_plugin("Slack", PluginCategory::Notifier);
        let plugin2 = create_test_plugin("Teams", PluginCategory::Notifier);

        registry.register(plugin1);
        registry.register(plugin2);

        // Both should be available
        let notifiers = registry.notifiers();
        assert_eq!(notifiers.len(), 2);
        assert_eq!(notifiers[0].name(), "Slack");
        assert_eq!(notifiers[1].name(), "Teams");

        // get() returns None for stackable categories
        assert!(registry.get(PluginCategory::Notifier).is_none());
    }

    #[test]
    fn registry_stackable_reporters() {
        let mut registry = PluginRegistry::new();

        let plugin1 = create_test_plugin("JUnit", PluginCategory::Reporter);
        let plugin2 = create_test_plugin("JSON", PluginCategory::Reporter);

        registry.register(plugin1);
        registry.register(plugin2);

        let reporters = registry.reporters();
        assert_eq!(reporters.len(), 2);
    }

    #[test]
    fn registry_has_override() {
        let mut registry = PluginRegistry::new();

        assert!(!registry.has_override(PluginCategory::Storage));
        assert!(!registry.has_override(PluginCategory::Notifier));

        registry.register(create_test_plugin("S3", PluginCategory::Storage));
        registry.register(create_test_plugin("Slack", PluginCategory::Notifier));

        assert!(registry.has_override(PluginCategory::Storage));
        assert!(registry.has_override(PluginCategory::Notifier));
        assert!(!registry.has_override(PluginCategory::Capture));
    }

    #[test]
    fn registry_all_active() {
        let mut registry = PluginRegistry::new();

        registry.register(create_test_plugin("S3", PluginCategory::Storage));
        registry.register(create_test_plugin("Slack", PluginCategory::Notifier));
        registry.register(create_test_plugin("Teams", PluginCategory::Notifier));
        registry.register(create_test_plugin("JUnit", PluginCategory::Reporter));

        let active = registry.all_active();
        assert_eq!(active.len(), 4);
    }

    #[test]
    fn registry_plugin_names() {
        let mut registry = PluginRegistry::new();

        registry.register(create_test_plugin("Plugin A", PluginCategory::Storage));
        registry.register(create_test_plugin("Plugin B", PluginCategory::Notifier));

        let names = registry.plugin_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Plugin A"));
        assert!(names.contains(&"Plugin B"));
    }
}
