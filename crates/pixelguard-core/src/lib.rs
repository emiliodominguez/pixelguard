//! Pixelguard Core Library
//!
//! This crate provides the core functionality for visual regression testing:
//!
//! - **Configuration**: Load and manage `pixelguard.config.json`
//! - **Detection**: Auto-detect project types (Storybook, Next.js, Vite)
//! - **Capture**: Take screenshots using Playwright
//! - **Diff**: Compare images pixel-by-pixel
//! - **Report**: Generate static HTML reports
//!
//! # Example
//!
//! ```rust,no_run
//! use pixelguard_core::{Config, capture_screenshots, diff_images};
//!
//! async fn run() -> anyhow::Result<()> {
//!     // Load or create config
//!     let config = Config::load_or_default(".")?;
//!
//!     // Capture screenshots
//!     let _screenshots = capture_screenshots(&config).await?;
//!
//!     // Compare against baseline (uses current working directory)
//!     let results = diff_images(&config, ".")?;
//!
//!     println!("Changed: {}", results.changed.len());
//!     Ok(())
//! }
//! ```

pub mod capture;
pub mod config;
pub mod detect;
pub mod diff;
pub mod report;

pub use capture::capture_screenshots;
pub use config::Config;
pub use detect::{detect_project_type, fetch_storybook_stories, ProjectType};
pub use diff::{diff_images, DiffResult};
pub use report::generate_report;
