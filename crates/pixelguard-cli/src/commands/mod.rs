//! CLI command implementations.
//!
//! Each subcommand is implemented in its own module:
//! - `init`: Initialize Pixelguard configuration
//! - `test`: Capture and compare screenshots
//! - `list`: List configured shots
//! - `plugins`: List and validate installed plugins

pub mod init;
pub mod list;
pub mod plugins;
pub mod test;
