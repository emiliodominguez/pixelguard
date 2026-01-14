//! Project type detection for Pixelguard.
//!
//! This module auto-detects the project type by checking for framework-specific
//! configuration files and probing common development server ports.

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use tracing::{debug, info};

use crate::config::Shot;

/// Detected project type with associated metadata.
#[derive(Debug, Clone)]
pub enum ProjectType {
    /// Storybook project with discovered stories
    Storybook {
        /// Base URL of the running Storybook server
        base_url: String,
        /// List of discovered stories as shots
        stories: Vec<Shot>,
    },

    /// Next.js project
    NextJs {
        /// Base URL of the running dev server
        base_url: String,
        /// List of discovered routes as shots
        routes: Vec<Shot>,
    },

    /// Vite project
    Vite {
        /// Base URL of the running dev server
        base_url: String,
    },

    /// Unknown project type - user must configure manually
    Unknown,
}

/// Storybook index.json format (Storybook 7+)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StorybookIndex {
    v: Option<u32>,
    entries: Option<std::collections::HashMap<String, StorybookEntry>>,
}

/// Storybook stories.json format (older versions)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StorybookStories {
    v: Option<u32>,
    stories: Option<std::collections::HashMap<String, StorybookStory>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StorybookEntry {
    id: String,
    name: String,
    title: String,
    #[serde(rename = "type")]
    entry_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StorybookStory {
    id: String,
    name: String,
    title: String,
    kind: Option<String>,
}

impl ProjectType {
    /// Returns true if this is a known project type (not Unknown).
    pub fn is_known(&self) -> bool {
        !matches!(self, ProjectType::Unknown)
    }

    /// Returns the base URL if available.
    pub fn base_url(&self) -> Option<&str> {
        match self {
            ProjectType::Storybook { base_url, .. } => Some(base_url),
            ProjectType::NextJs { base_url, .. } => Some(base_url),
            ProjectType::Vite { base_url, .. } => Some(base_url),
            ProjectType::Unknown => None,
        }
    }

    /// Returns the source type name.
    pub fn source_name(&self) -> &'static str {
        match self {
            ProjectType::Storybook { .. } => "storybook",
            ProjectType::NextJs { .. } => "nextjs",
            ProjectType::Vite { .. } => "vite",
            ProjectType::Unknown => "manual",
        }
    }
}

/// Detects the project type by checking for framework config files
/// and probing common dev server ports.
///
/// Detection order:
/// 1. Storybook (`.storybook/` directory)
/// 2. Next.js (`next.config.{js,mjs,ts}`)
/// 3. Vite (`vite.config.{js,ts,mjs}`)
///
/// # Example
///
/// ```rust,no_run
/// use pixelguard_core::detect_project_type;
///
/// async fn example() -> anyhow::Result<()> {
///     let project = detect_project_type(".").await?;
///     println!("Detected: {:?}", project);
///     Ok(())
/// }
/// ```
pub async fn detect_project_type<P: AsRef<Path>>(dir: P) -> Result<ProjectType> {
    let dir = dir.as_ref();

    // Check for Storybook
    if dir.join(".storybook").exists() {
        info!("Found .storybook directory");
        if let Some(project) = detect_storybook().await {
            return Ok(project);
        }
    }

    // Check for Next.js
    if has_next_config(dir) {
        info!("Found Next.js config");
        if let Some(project) = detect_nextjs(dir).await {
            return Ok(project);
        }
    }

    // Check for Vite
    if has_vite_config(dir) {
        info!("Found Vite config");
        if let Some(project) = detect_vite().await {
            return Ok(project);
        }
    }

    Ok(ProjectType::Unknown)
}

fn has_next_config(dir: &Path) -> bool {
    ["next.config.js", "next.config.mjs", "next.config.ts"]
        .iter()
        .any(|f| dir.join(f).exists())
}

fn has_vite_config(dir: &Path) -> bool {
    ["vite.config.js", "vite.config.ts", "vite.config.mjs"]
        .iter()
        .any(|f| dir.join(f).exists())
}

async fn detect_storybook() -> Option<ProjectType> {
    let ports = [6006, 6007, 6008];

    for port in ports {
        let base_url = format!("http://localhost:{}", port);
        debug!("Probing Storybook at {}", base_url);

        if let Some(stories) = fetch_storybook_stories(&base_url).await {
            info!(
                "Found Storybook at {} with {} stories",
                base_url,
                stories.len()
            );
            return Some(ProjectType::Storybook { base_url, stories });
        }
    }

    None
}

/// Fetches stories from a running Storybook server.
///
/// Tries the index.json endpoint first (Storybook 7+), then falls back to
/// stories.json for older versions.
///
/// # Example
///
/// ```rust,no_run
/// use pixelguard_core::detect::fetch_storybook_stories;
///
/// async fn example() -> Option<Vec<pixelguard_core::config::Shot>> {
///     fetch_storybook_stories("http://localhost:6006").await
/// }
/// ```
pub async fn fetch_storybook_stories(base_url: &str) -> Option<Vec<Shot>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok()?;

    // Try index.json first (Storybook 7+)
    let index_url = format!("{}/index.json", base_url);
    if let Ok(response) = client.get(&index_url).send().await {
        if response.status().is_success() {
            if let Ok(index) = response.json::<StorybookIndex>().await {
                if let Some(entries) = index.entries {
                    let shots: Vec<Shot> = entries
                        .into_iter()
                        .filter(|(_, entry)| entry.entry_type.as_deref() != Some("docs"))
                        .map(|(_, entry)| Shot {
                            name: entry.id.clone(),
                            path: format!("/iframe.html?id={}&viewMode=story", entry.id),
                            wait_for: Some("#storybook-root".to_string()),
                            delay: Some(100),
                        })
                        .collect();

                    if !shots.is_empty() {
                        return Some(shots);
                    }
                }
            }
        }
    }

    // Fall back to stories.json (older versions)
    let stories_url = format!("{}/stories.json", base_url);
    if let Ok(response) = client.get(&stories_url).send().await {
        if response.status().is_success() {
            if let Ok(stories) = response.json::<StorybookStories>().await {
                if let Some(stories_map) = stories.stories {
                    let shots: Vec<Shot> = stories_map
                        .into_values()
                        .map(|story| Shot {
                            name: story.id.clone(),
                            path: format!("/iframe.html?id={}&viewMode=story", story.id),
                            wait_for: Some("#storybook-root".to_string()),
                            delay: Some(100),
                        })
                        .collect();

                    if !shots.is_empty() {
                        return Some(shots);
                    }
                }
            }
        }
    }

    None
}

async fn detect_nextjs(dir: &Path) -> Option<ProjectType> {
    let ports = [3000, 3001];

    for port in ports {
        let base_url = format!("http://localhost:{}", port);
        debug!("Probing Next.js at {}", base_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .ok()?;

        if client.get(&base_url).send().await.is_ok() {
            info!("Found Next.js dev server at {}", base_url);

            // Scan for routes in app/ and pages/ directories
            let routes = scan_nextjs_routes(dir);

            return Some(ProjectType::NextJs { base_url, routes });
        }
    }

    None
}

fn scan_nextjs_routes(dir: &Path) -> Vec<Shot> {
    let mut routes = Vec::new();

    // Scan app/ directory (App Router)
    let app_dir = dir.join("app");
    if app_dir.exists() {
        scan_app_router(&app_dir, "", &mut routes);
    }

    // Scan pages/ directory (Pages Router)
    let pages_dir = dir.join("pages");
    if pages_dir.exists() {
        scan_pages_router(&pages_dir, "", &mut routes);
    }

    // Scan src/app/ and src/pages/ as well
    let src_app_dir = dir.join("src/app");
    if src_app_dir.exists() {
        scan_app_router(&src_app_dir, "", &mut routes);
    }

    let src_pages_dir = dir.join("src/pages");
    if src_pages_dir.exists() {
        scan_pages_router(&src_pages_dir, "", &mut routes);
    }

    routes
}

fn scan_app_router(dir: &Path, prefix: &str, routes: &mut Vec<Shot>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip special directories and files
            if name.starts_with('_') || name.starts_with('.') || name == "api" {
                continue;
            }

            if path.is_dir() {
                // Check for page.tsx/page.js
                let has_page = ["page.tsx", "page.jsx", "page.js"]
                    .iter()
                    .any(|f| path.join(f).exists());

                let route_path = if name.starts_with('(') && name.ends_with(')') {
                    // Route group - don't add to path
                    prefix.to_string()
                } else if name.starts_with('[') && name.ends_with(']') {
                    // Dynamic segment - use placeholder
                    format!("{}/{}", prefix, name.trim_matches(|c| c == '[' || c == ']'))
                } else {
                    format!("{}/{}", prefix, name)
                };

                if has_page {
                    let route = if route_path.is_empty() {
                        "/"
                    } else {
                        &route_path
                    };
                    routes.push(Shot {
                        name: format!("page-{}", route.replace('/', "-").trim_matches('-')),
                        path: route.to_string(),
                        wait_for: None,
                        delay: Some(500),
                    });
                }

                // Recurse into subdirectories
                scan_app_router(&path, &route_path, routes);
            }
        }
    }
}

fn scan_pages_router(dir: &Path, prefix: &str, routes: &mut Vec<Shot>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip special files
            if name.starts_with('_') || name.starts_with('.') || name == "api" {
                continue;
            }

            if path.is_dir() {
                scan_pages_router(&path, &format!("{}/{}", prefix, name), routes);
            } else if let Some(stem) = path.file_stem() {
                let ext = path.extension().and_then(|e| e.to_str());
                if matches!(ext, Some("tsx" | "jsx" | "js" | "ts")) {
                    let stem = stem.to_string_lossy();
                    let route = if stem == "index" {
                        if prefix.is_empty() {
                            "/".to_string()
                        } else {
                            prefix.to_string()
                        }
                    } else {
                        format!("{}/{}", prefix, stem)
                    };

                    routes.push(Shot {
                        name: format!("page-{}", route.replace('/', "-").trim_matches('-')),
                        path: route,
                        wait_for: None,
                        delay: Some(500),
                    });
                }
            }
        }
    }
}

async fn detect_vite() -> Option<ProjectType> {
    let ports = [5173, 5174, 3000];

    for port in ports {
        let base_url = format!("http://localhost:{}", port);
        debug!("Probing Vite at {}", base_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .ok()?;

        if client.get(&base_url).send().await.is_ok() {
            info!("Found Vite dev server at {}", base_url);
            return Some(ProjectType::Vite { base_url });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn project_type_is_known() {
        let storybook = ProjectType::Storybook {
            base_url: "http://localhost:6006".to_string(),
            stories: vec![],
        };
        assert!(storybook.is_known());

        let unknown = ProjectType::Unknown;
        assert!(!unknown.is_known());
    }

    #[test]
    fn project_type_base_url() {
        let storybook = ProjectType::Storybook {
            base_url: "http://localhost:6006".to_string(),
            stories: vec![],
        };
        assert_eq!(storybook.base_url(), Some("http://localhost:6006"));

        let unknown = ProjectType::Unknown;
        assert_eq!(unknown.base_url(), None);
    }

    #[test]
    fn project_type_source_name() {
        let storybook = ProjectType::Storybook {
            base_url: String::new(),
            stories: vec![],
        };
        assert_eq!(storybook.source_name(), "storybook");

        let nextjs = ProjectType::NextJs {
            base_url: String::new(),
            routes: vec![],
        };
        assert_eq!(nextjs.source_name(), "nextjs");

        let vite = ProjectType::Vite {
            base_url: String::new(),
        };
        assert_eq!(vite.source_name(), "vite");

        let unknown = ProjectType::Unknown;
        assert_eq!(unknown.source_name(), "manual");
    }

    #[test]
    fn has_next_config_detects_files() {
        let dir = tempdir().unwrap();
        assert!(!has_next_config(dir.path()));

        std::fs::write(dir.path().join("next.config.js"), "").unwrap();
        assert!(has_next_config(dir.path()));
    }

    #[test]
    fn has_vite_config_detects_files() {
        let dir = tempdir().unwrap();
        assert!(!has_vite_config(dir.path()));

        std::fs::write(dir.path().join("vite.config.ts"), "").unwrap();
        assert!(has_vite_config(dir.path()));
    }

    #[tokio::test]
    async fn detect_returns_unknown_for_empty_dir() {
        let dir = tempdir().unwrap();
        let result = detect_project_type(dir.path()).await.unwrap();
        assert!(matches!(result, ProjectType::Unknown));
    }
}
