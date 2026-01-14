//! Plugin execution via Node.js subprocess.
//!
//! This module handles the IPC between Rust and Node.js plugins,
//! following the established pattern from capture.rs.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use tracing::debug;

use super::types::{LoadedPlugin, PluginResult};

/// Executes a plugin hook with the given input.
///
/// Generates a Node.js script that loads the plugin and calls the specified hook,
/// then parses the JSON output.
///
/// # Type Parameters
///
/// * `I` - Input type (must be serializable)
/// * `O` - Output type (must be deserializable)
///
/// # Arguments
///
/// * `plugin` - The loaded plugin to execute
/// * `hook_name` - Name of the hook to call (e.g., "read", "write", "notify")
/// * `input` - Input data to pass to the hook
/// * `working_dir` - Working directory for Node.js execution
///
/// # Returns
///
/// The deserialized output from the plugin hook.
///
/// # Errors
///
/// Returns an error if:
/// - The plugin script cannot be generated
/// - Node.js execution fails
/// - The plugin returns an error
/// - The output cannot be parsed
pub fn execute_hook<I, O>(
    plugin: &LoadedPlugin,
    hook_name: &str,
    input: &I,
    working_dir: &Path,
) -> Result<O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    let script = generate_hook_script(plugin, hook_name, input)?;
    let result = run_node_script(&script, working_dir)?;

    if !result.success {
        anyhow::bail!(
            "Plugin '{}' hook '{}' failed: {}",
            plugin.name(),
            hook_name,
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        );
    }

    let data = result.data.unwrap_or(serde_json::Value::Null);
    serde_json::from_value(data).with_context(|| {
        format!(
            "Failed to parse output from plugin '{}' hook '{}'",
            plugin.name(),
            hook_name
        )
    })
}

/// Executes a plugin hook that doesn't return data.
///
/// Like `execute_hook` but for hooks that only perform side effects
/// (e.g., notify, delete).
pub fn execute_hook_void<I>(
    plugin: &LoadedPlugin,
    hook_name: &str,
    input: &I,
    working_dir: &Path,
) -> Result<()>
where
    I: Serialize,
{
    let script = generate_hook_script(plugin, hook_name, input)?;
    let result = run_node_script(&script, working_dir)?;

    if !result.success {
        anyhow::bail!(
            "Plugin '{}' hook '{}' failed: {}",
            plugin.name(),
            hook_name,
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        );
    }

    Ok(())
}

/// Generates a Node.js script to execute a plugin hook.
fn generate_hook_script<I: Serialize>(
    plugin: &LoadedPlugin,
    hook_name: &str,
    input: &I,
) -> Result<String> {
    let entry_path = plugin
        .entry_path
        .to_str()
        .context("Plugin entry path is not valid UTF-8")?
        .replace('\\', "/");

    let input_json = serde_json::to_string(input)?;
    let options_json = serde_json::to_string(&plugin.options)?;

    Ok(format!(
        r#"
const plugin = require({entry_path});

async function run() {{
    try {{
        const input = {input_json};
        const options = {options_json};

        // Merge options into input if not already present
        if (typeof input === 'object' && input !== null && !Array.isArray(input)) {{
            input.options = {{ ...options, ...input.options }};
        }}

        // Check if hook exists
        if (typeof plugin.{hook_name} !== 'function') {{
            throw new Error('Hook "{hook_name}" is not implemented by this plugin');
        }}

        const result = await plugin.{hook_name}(input);

        console.log(JSON.stringify({{
            success: true,
            data: result !== undefined ? result : null
        }}));
    }} catch (error) {{
        console.log(JSON.stringify({{
            success: false,
            error: error.message || String(error)
        }}));
        process.exit(1);
    }}
}}

run();
"#,
        entry_path = serde_json::to_string(&entry_path)?,
        input_json = input_json,
        options_json = options_json,
        hook_name = hook_name,
    ))
}

/// Runs a Node.js script and parses the result.
fn run_node_script(script: &str, working_dir: &Path) -> Result<PluginResult> {
    let temp_dir = tempfile::tempdir()?;
    let script_path = temp_dir.path().join("plugin-hook.js");
    std::fs::write(&script_path, script)?;

    debug!("Executing plugin script at {:?}", script_path);

    let output = Command::new("node")
        .arg(&script_path)
        .current_dir(working_dir)
        .output()
        .context("Failed to execute Node.js. Make sure Node.js is installed.")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        // Print stderr directly to user's terminal (plugins use this for output)
        eprint!("{}", stderr);
    }

    // Try to parse stdout as JSON first
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() {
        // No output - check if process failed
        if !output.status.success() {
            return Ok(PluginResult::err(format!(
                "Plugin process exited with code {:?}. Stderr: {}",
                output.status.code(),
                stderr
            )));
        }
        // Success with no output
        return Ok(PluginResult::ok_empty());
    }

    // Try to parse as PluginResult JSON
    match serde_json::from_str::<PluginResult>(trimmed) {
        Ok(result) => Ok(result),
        Err(parse_error) => {
            // If parsing fails and process also failed, return the error
            if !output.status.success() {
                Ok(PluginResult::err(format!(
                    "Plugin failed (exit code {:?}). Output: {}. Stderr: {}",
                    output.status.code(),
                    trimmed,
                    stderr
                )))
            } else {
                // Process succeeded but output wasn't valid JSON
                Ok(PluginResult::err(format!(
                    "Failed to parse plugin output as JSON: {}. Output was: {}",
                    parse_error, trimmed
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::types::{PluginCategory, PluginManifest};
    use tempfile::tempdir;

    fn create_mock_plugin(dir: &std::path::Path, js_code: &str) -> LoadedPlugin {
        std::fs::write(dir.join("index.js"), js_code).unwrap();

        LoadedPlugin {
            manifest: PluginManifest {
                name: "Test Plugin".to_string(),
                category: PluginCategory::Storage,
                entry: "index.js".to_string(),
                hooks: vec!["testHook".to_string()],
                version: "1.0.0".to_string(),
                options_schema: None,
            },
            package_path: dir.to_path_buf(),
            entry_path: dir.join("index.js"),
            options: serde_json::json!({"testOption": "testValue"}),
        }
    }

    #[test]
    fn execute_hook_simple() {
        let dir = tempdir().unwrap();
        let plugin = create_mock_plugin(
            dir.path(),
            r#"
            module.exports = {
                testHook: async (input) => {
                    return { received: input.value, doubled: input.value * 2 };
                }
            };
            "#,
        );

        let input = serde_json::json!({"value": 21});
        let output: serde_json::Value =
            execute_hook(&plugin, "testHook", &input, dir.path()).unwrap();

        assert_eq!(output["received"], 21);
        assert_eq!(output["doubled"], 42);
    }

    #[test]
    fn execute_hook_receives_options() {
        let dir = tempdir().unwrap();
        let plugin = create_mock_plugin(
            dir.path(),
            r#"
            module.exports = {
                testHook: async (input) => {
                    return { optionValue: input.options.testOption };
                }
            };
            "#,
        );

        let input = serde_json::json!({});
        let output: serde_json::Value =
            execute_hook(&plugin, "testHook", &input, dir.path()).unwrap();

        assert_eq!(output["optionValue"], "testValue");
    }

    #[test]
    fn execute_hook_error() {
        let dir = tempdir().unwrap();
        let plugin = create_mock_plugin(
            dir.path(),
            r#"
            module.exports = {
                testHook: async (input) => {
                    throw new Error('Something went wrong');
                }
            };
            "#,
        );

        let input = serde_json::json!({});
        let result: Result<serde_json::Value> =
            execute_hook(&plugin, "testHook", &input, dir.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Something went wrong"));
    }

    #[test]
    fn execute_hook_missing_hook() {
        let dir = tempdir().unwrap();
        let plugin = create_mock_plugin(
            dir.path(),
            r#"
            module.exports = {
                otherHook: async () => {}
            };
            "#,
        );

        let input = serde_json::json!({});
        let result: Result<serde_json::Value> =
            execute_hook(&plugin, "testHook", &input, dir.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not implemented"));
    }

    #[test]
    fn execute_hook_void_success() {
        let dir = tempdir().unwrap();
        let plugin = create_mock_plugin(
            dir.path(),
            r#"
            module.exports = {
                testHook: async (input) => {
                    // Side effect only, no return
                }
            };
            "#,
        );

        let input = serde_json::json!({});
        let result = execute_hook_void(&plugin, "testHook", &input, dir.path());

        assert!(result.is_ok());
    }
}
