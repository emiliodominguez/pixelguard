//! CLI command implementations.
//!
//! Each subcommand is implemented in its own module:
//! - `init`: Initialize Pixelguard configuration
//! - `test`: Capture and compare screenshots
//! - `list`: List configured shots
//! - `plugins`: List and validate installed plugins
//! - `validate`: Check environment prerequisites
//! - `apply`: Apply decisions from exported JSON file
//! - `review`: Interactively review visual diffs
//! - `serve`: Serve an existing report with decisions API

use std::path::Path;

use anyhow::Result;
use pixelguard_core::Config;

pub mod apply;
pub mod init;
pub mod list;
pub mod plugins;
pub mod review;
pub mod serve;
pub mod test;
pub mod validate;

/// Loads config from a custom path or the default location.
pub fn load_config(working_dir: &Path, config_path: Option<&str>) -> Result<Config> {
    match config_path {
        Some(path) => Config::load(working_dir.join(path)),
        None => Config::load_or_default(working_dir),
    }
}
