//! Plugin execution via Node.js subprocess.
//!
//! This module handles the IPC between Rust and Node.js plugins. A static
//! runner script receives its input through a JSON file and writes the hook's
//! result to another file, so plugin logging never interferes with the protocol.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use tracing::debug;

use super::types::{LoadedPlugin, PluginResult};
use crate::process;

/// Checks the plugin result for errors and returns a formatted error message.
fn check_plugin_error(result: &PluginResult, plugin_name: &str, hook_name: &str) -> Result<()> {
    if !result.success {
        anyhow::bail!(
            "❌ Plugin '{}' hook '{}' failed: {}\n\n\
             💡 Troubleshooting:\n  \
             • Check plugin is properly installed\n  \
             • Verify plugin options in config\n  \
             • Review plugin documentation\n  \
             • Check console output above for details",
            plugin_name,
            hook_name,
            result.error.as_deref().unwrap_or("Unknown error")
        );
    }
    Ok(())
}

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
    let result = run_hook(plugin, hook_name, input, working_dir)?;
    check_plugin_error(&result, plugin.name(), hook_name)?;

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
    let result = run_hook(plugin, hook_name, input, working_dir)?;
    check_plugin_error(&result, plugin.name(), hook_name)?;

    Ok(())
}

/// Static hook runner. The entry path, hook name, input and options arrive as
/// data, and the result goes to a dedicated file, so a plugin that logs to
/// stdout cannot corrupt the protocol.
const HOOK_RUNNER_SCRIPT: &str = r#"
const fs = require('fs');
const { pathToFileURL } = require('url');
const [entryPath, inputPath, outputPath] = process.argv.slice(2);

function writeResult(result) {
    fs.writeFileSync(outputPath, JSON.stringify(result));
}

async function loadPlugin() {
    try {
        return require(entryPath);
    } catch (error) {
        if (error.code !== 'ERR_REQUIRE_ESM' && error.code !== 'ERR_REQUIRE_ASYNC_MODULE') throw error;
        const mod = await import(pathToFileURL(entryPath).href);
        return mod.default || mod;
    }
}

async function run() {
    try {
        const { hook, input, options } = JSON.parse(fs.readFileSync(inputPath, 'utf8'));
        const plugin = await loadPlugin();

        if (input !== null && typeof input === 'object' && !Array.isArray(input)) {
            input.options = { ...options, ...(input.options || {}) };
        }

        if (!plugin || !Object.prototype.hasOwnProperty.call(plugin, hook) || typeof plugin[hook] !== 'function') {
            throw new Error(`Hook "${hook}" is not implemented by this plugin`);
        }

        const result = await plugin[hook](input);
        writeResult({ success: true, data: result === undefined ? null : result });
    } catch (error) {
        writeResult({ success: false, error: (error && error.message) || String(error) });
        process.exitCode = 1;
    }
}

run();
"#;

/// Default wall-clock limit for a single plugin hook.
const DEFAULT_HOOK_TIMEOUT: Duration = Duration::from_secs(600);

/// Runs `hook_name` of `plugin` with `input` and returns the raw plugin result.
fn run_hook<I: Serialize>(
    plugin: &LoadedPlugin,
    hook_name: &str,
    input: &I,
    working_dir: &Path,
) -> Result<PluginResult> {
    if hook_name.is_empty()
        || !hook_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        anyhow::bail!("Invalid plugin hook name '{}'", hook_name);
    }

    let temp_dir = tempfile::tempdir()?;
    let script_path = temp_dir.path().join("plugin-hook.js");
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");

    std::fs::write(&script_path, HOOK_RUNNER_SCRIPT)?;
    std::fs::write(
        &input_path,
        serde_json::to_vec(&serde_json::json!({
            "hook": hook_name,
            "input": input,
            "options": plugin.options,
        }))?,
    )?;

    let entry_path = std::path::absolute(&plugin.entry_path)?;

    debug!("🔌 Executing plugin script at {:?}", script_path);

    let output = process::run_node_script(
        &script_path,
        &[
            entry_path.as_os_str(),
            input_path.as_os_str(),
            output_path.as_os_str(),
        ],
        working_dir,
        process::timeout_from_env(DEFAULT_HOOK_TIMEOUT),
    )
    .with_context(|| {
        format!(
            "Plugin '{}' hook '{}' did not complete",
            plugin.name(),
            hook_name
        )
    })?;

    // Plugins use stdout and stderr for human-readable output; pass both through.
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        eprint!("{}", stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    match std::fs::read(&output_path) {
        Ok(bytes) => Ok(
            serde_json::from_slice::<PluginResult>(&bytes).unwrap_or_else(|parse_error| {
                PluginResult::err(format!("Failed to parse plugin result: {}", parse_error))
            }),
        ),
        Err(_) => Ok(PluginResult::err(format!(
            "Plugin exited with code {:?} without producing a result. Stderr: {}",
            output.status.code(),
            stderr.trim()
        ))),
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

    #[test]
    fn execute_hook_tolerates_plugins_that_log_to_stdout() {
        let dir = tempdir().unwrap();
        let plugin = create_mock_plugin(
            dir.path(),
            r#"
            module.exports = {
                testHook: async () => {
                    console.log('chatty plugin output');
                    return { ok: true };
                }
            };
            "#,
        );

        let output: serde_json::Value =
            execute_hook(&plugin, "testHook", &serde_json::json!({}), dir.path()).unwrap();

        assert_eq!(output["ok"], true);
    }

    #[test]
    fn execute_hook_treats_input_as_data_not_code() {
        let dir = tempdir().unwrap();
        let marker = dir.path().join("pwned");
        let plugin = create_mock_plugin(
            dir.path(),
            "module.exports = { testHook: async (input) => ({ echoed: input.value }) };",
        );
        let payload = format!(
            "\"}};require('fs').writeFileSync({:?},'x');//",
            marker.to_string_lossy()
        );

        let output: serde_json::Value = execute_hook(
            &plugin,
            "testHook",
            &serde_json::json!({ "value": payload }),
            dir.path(),
        )
        .unwrap();

        assert_eq!(output["echoed"], payload);
        assert!(!marker.exists());
    }

    #[test]
    fn execute_hook_rejects_inherited_properties_as_hooks() {
        let dir = tempdir().unwrap();
        let plugin = create_mock_plugin(dir.path(), "module.exports = {};");

        let result: Result<serde_json::Value> =
            execute_hook(&plugin, "constructor", &serde_json::json!({}), dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn execute_hook_reports_plugins_that_exit_without_a_result() {
        let dir = tempdir().unwrap();
        let plugin = create_mock_plugin(
            dir.path(),
            "module.exports = { testHook: async () => { process.exit(0); } };",
        );

        let result: Result<serde_json::Value> =
            execute_hook(&plugin, "testHook", &serde_json::json!({}), dir.path());

        assert!(result.is_err());
    }
}
