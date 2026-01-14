//! Storage abstraction for baseline screenshots.
//!
//! This module provides a unified interface for reading and writing baseline
//! screenshots, supporting both local filesystem (default) and plugin-based
//! storage backends like S3, Cloudflare R2, or Azure Blob.

use std::path::PathBuf;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tracing::debug;

use crate::plugins::{
    executor, LoadedPlugin, PluginCategory, PluginRegistry, StorageInput, StorageOutput,
};

/// Storage backend for baseline screenshots.
///
/// Provides a unified interface that works with both local filesystem
/// and remote storage via plugins.
pub struct Storage<'a> {
    /// Base directory for local storage
    base_dir: PathBuf,
    /// Optional storage plugin
    plugin: Option<&'a LoadedPlugin>,
    /// Working directory for plugin execution
    working_dir: PathBuf,
}

impl<'a> Storage<'a> {
    /// Creates a new storage instance.
    ///
    /// If a storage plugin is registered, it will be used instead of
    /// local filesystem operations.
    pub fn new(
        base_dir: PathBuf,
        working_dir: PathBuf,
        plugin_registry: Option<&'a PluginRegistry>,
    ) -> Self {
        let plugin = plugin_registry.and_then(|r| r.get_override(PluginCategory::Storage));

        Self {
            base_dir,
            plugin,
            working_dir,
        }
    }

    /// Creates a storage instance for local-only operations.
    pub fn local(base_dir: PathBuf) -> Self {
        Self {
            base_dir: base_dir.clone(),
            plugin: None,
            working_dir: base_dir,
        }
    }

    /// Checks if using a storage plugin.
    pub fn is_remote(&self) -> bool {
        self.plugin.is_some()
    }

    /// Reads a file from storage.
    ///
    /// Returns the file contents as bytes.
    pub fn read(&self, relative_path: &str) -> Result<Vec<u8>> {
        if let Some(plugin) = self.plugin {
            self.read_plugin(plugin, relative_path)
        } else {
            self.read_local(relative_path)
        }
    }

    /// Writes a file to storage.
    pub fn write(&self, relative_path: &str, data: &[u8]) -> Result<()> {
        if let Some(plugin) = self.plugin {
            self.write_plugin(plugin, relative_path, data)
        } else {
            self.write_local(relative_path, data)
        }
    }

    /// Checks if a file exists in storage.
    pub fn exists(&self, relative_path: &str) -> Result<bool> {
        if let Some(plugin) = self.plugin {
            self.exists_plugin(plugin, relative_path)
        } else {
            self.exists_local(relative_path)
        }
    }

    /// Lists files in a directory.
    ///
    /// Returns relative paths of all files.
    pub fn list(&self, relative_path: &str) -> Result<Vec<String>> {
        if let Some(plugin) = self.plugin {
            self.list_plugin(plugin, relative_path)
        } else {
            self.list_local(relative_path)
        }
    }

    /// Deletes a file from storage.
    pub fn delete(&self, relative_path: &str) -> Result<()> {
        if let Some(plugin) = self.plugin {
            self.delete_plugin(plugin, relative_path)
        } else {
            self.delete_local(relative_path)
        }
    }

    /// Copies a file within storage.
    ///
    /// For local storage, uses filesystem copy.
    /// For plugins, reads then writes.
    pub fn copy(&self, from: &str, to: &str) -> Result<()> {
        if self.plugin.is_some() {
            // For remote storage, read and write
            let data = self.read(from)?;
            self.write(to, &data)?;
        } else {
            // Local filesystem copy
            let src = self.base_dir.join(from);
            let dst = self.base_dir.join(to);

            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::copy(&src, &dst).with_context(|| {
                format!("Failed to copy {} to {}", src.display(), dst.display())
            })?;
        }
        Ok(())
    }

    // Local filesystem operations

    fn read_local(&self, relative_path: &str) -> Result<Vec<u8>> {
        let path = self.base_dir.join(relative_path);
        debug!("Reading local file: {}", path.display());
        std::fs::read(&path).with_context(|| format!("Failed to read file: {}", path.display()))
    }

    fn write_local(&self, relative_path: &str, data: &[u8]) -> Result<()> {
        let path = self.base_dir.join(relative_path);
        debug!("Writing local file: {}", path.display());

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, data)
            .with_context(|| format!("Failed to write file: {}", path.display()))
    }

    fn exists_local(&self, relative_path: &str) -> Result<bool> {
        let path = self.base_dir.join(relative_path);
        Ok(path.exists())
    }

    fn list_local(&self, relative_path: &str) -> Result<Vec<String>> {
        let path = self.base_dir.join(relative_path);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    files.push(name.to_string());
                }
            }
        }

        Ok(files)
    }

    fn delete_local(&self, relative_path: &str) -> Result<()> {
        let path = self.base_dir.join(relative_path);
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to delete file: {}", path.display()))?;
        }
        Ok(())
    }

    // Plugin-based operations

    fn read_plugin(&self, plugin: &LoadedPlugin, relative_path: &str) -> Result<Vec<u8>> {
        debug!("Reading via plugin: {}", relative_path);

        let input = StorageInput {
            path: relative_path.to_string(),
            data: None,
            options: serde_json::Value::Null,
        };

        let output: StorageOutput =
            executor::execute_hook(plugin, "read", &input, &self.working_dir)?;

        let data = output
            .data
            .ok_or_else(|| anyhow::anyhow!("Plugin did not return data for read operation"))?;

        BASE64
            .decode(&data)
            .context("Failed to decode base64 data from plugin")
    }

    fn write_plugin(&self, plugin: &LoadedPlugin, relative_path: &str, data: &[u8]) -> Result<()> {
        debug!("Writing via plugin: {}", relative_path);

        let input = StorageInput {
            path: relative_path.to_string(),
            data: Some(BASE64.encode(data)),
            options: serde_json::Value::Null,
        };

        executor::execute_hook_void(plugin, "write", &input, &self.working_dir)
    }

    fn exists_plugin(&self, plugin: &LoadedPlugin, relative_path: &str) -> Result<bool> {
        debug!("Checking exists via plugin: {}", relative_path);

        let input = StorageInput {
            path: relative_path.to_string(),
            data: None,
            options: serde_json::Value::Null,
        };

        let output: StorageOutput =
            executor::execute_hook(plugin, "exists", &input, &self.working_dir)?;

        Ok(output.exists.unwrap_or(false))
    }

    fn list_plugin(&self, plugin: &LoadedPlugin, relative_path: &str) -> Result<Vec<String>> {
        debug!("Listing via plugin: {}", relative_path);

        let input = StorageInput {
            path: relative_path.to_string(),
            data: None,
            options: serde_json::Value::Null,
        };

        let output: StorageOutput =
            executor::execute_hook(plugin, "list", &input, &self.working_dir)?;

        Ok(output.files.unwrap_or_default())
    }

    fn delete_plugin(&self, plugin: &LoadedPlugin, relative_path: &str) -> Result<()> {
        debug!("Deleting via plugin: {}", relative_path);

        let input = StorageInput {
            path: relative_path.to_string(),
            data: None,
            options: serde_json::Value::Null,
        };

        executor::execute_hook_void(plugin, "delete", &input, &self.working_dir)
    }
}

/// Gets the baseline storage path.
pub fn baseline_path(name: &str) -> String {
    format!("baseline/{}.png", name)
}

/// Gets the current screenshot path.
pub fn current_path(name: &str) -> String {
    format!("current/{}.png", name)
}

/// Gets the diff image path.
pub fn diff_path(name: &str) -> String {
    format!("diff/{}.png", name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn local_storage_write_read() {
        let dir = tempdir().unwrap();
        let storage = Storage::local(dir.path().to_path_buf());

        let data = b"test content";
        storage.write("test.txt", data).unwrap();

        let read_data = storage.read("test.txt").unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn local_storage_exists() {
        let dir = tempdir().unwrap();
        let storage = Storage::local(dir.path().to_path_buf());

        assert!(!storage.exists("missing.txt").unwrap());

        storage.write("exists.txt", b"data").unwrap();
        assert!(storage.exists("exists.txt").unwrap());
    }

    #[test]
    fn local_storage_list() {
        let dir = tempdir().unwrap();
        let storage = Storage::local(dir.path().to_path_buf());

        // Create a subdirectory with files
        std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("subdir/a.txt"), "a").unwrap();
        std::fs::write(dir.path().join("subdir/b.txt"), "b").unwrap();

        let files = storage.list("subdir").unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"a.txt".to_string()));
        assert!(files.contains(&"b.txt".to_string()));
    }

    #[test]
    fn local_storage_delete() {
        let dir = tempdir().unwrap();
        let storage = Storage::local(dir.path().to_path_buf());

        storage.write("to_delete.txt", b"data").unwrap();
        assert!(storage.exists("to_delete.txt").unwrap());

        storage.delete("to_delete.txt").unwrap();
        assert!(!storage.exists("to_delete.txt").unwrap());
    }

    #[test]
    fn local_storage_copy() {
        let dir = tempdir().unwrap();
        let storage = Storage::local(dir.path().to_path_buf());

        storage.write("original.txt", b"content").unwrap();
        storage.copy("original.txt", "copy.txt").unwrap();

        let copy_data = storage.read("copy.txt").unwrap();
        assert_eq!(copy_data, b"content");
    }

    #[test]
    fn path_helpers() {
        assert_eq!(baseline_path("button"), "baseline/button.png");
        assert_eq!(current_path("button"), "current/button.png");
        assert_eq!(diff_path("button"), "diff/button.png");
    }
}
