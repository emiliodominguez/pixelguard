//! Project type detection for Pixelguard.
//!
//! This module auto-detects the project type by checking for framework-specific
//! configuration files and probing common development server ports.
//!
//! Detection is Storybook-first: if a `.storybook/` directory is found, stories
//! are auto-discovered. For all other projects, a generic dev server check is
//! performed and users must configure shots manually.

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use tracing::{debug, info};

use crate::config::Shot;

/// Detected project type with associated metadata.
#[derive(Debug, Clone)]
pub enum ProjectType {
    /// Storybook project with auto-discovered stories
    Storybook {
        /// Base URL of the running Storybook server
        base_url: String,
        /// List of discovered stories as shots
        stories: Vec<Shot>,
    },

    /// Generic dev server detected - user must configure shots manually
    DevServer {
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
            ProjectType::DevServer { base_url } => Some(base_url),
            ProjectType::Unknown => None,
        }
    }

    /// Returns the source type name.
    pub fn source_name(&self) -> &'static str {
        match self {
            ProjectType::Storybook { .. } => "storybook",
            ProjectType::DevServer { .. } => "manual",
            ProjectType::Unknown => "manual",
        }
    }
}

/// Detects the project type by checking for framework config files
/// and probing common dev server ports.
///
/// Detection order:
/// 1. Storybook (`.storybook/` directory) - auto-discovers stories
/// 2. Generic dev server - probes common ports, user must configure shots
///
/// # Arguments
///
/// * `dir` - Directory to check for project configuration
/// * `port` - Optional port to use instead of default port probing
///
/// # Example
///
/// ```rust,no_run
/// use pixelguard_core::detect_project_type;
///
/// async fn example() -> anyhow::Result<()> {
///     let project = detect_project_type(".", None).await?;
///     println!("Detected: {:?}", project);
///     Ok(())
/// }
/// ```
pub async fn detect_project_type<P: AsRef<Path>>(dir: P, port: Option<u16>) -> Result<ProjectType> {
    let dir = dir.as_ref();

    // Check for Storybook first - it provides the most value
    if dir.join(".storybook").exists() {
        info!("Found .storybook directory");
        if let Some(project) = detect_storybook(port).await {
            return Ok(project);
        }
    }

    // Try to detect any running dev server
    if let Some(project) = detect_dev_server(port).await {
        return Ok(project);
    }

    Ok(ProjectType::Unknown)
}

/// Default ports to probe for Storybook
const STORYBOOK_PORTS: [u16; 3] = [6006, 6007, 6008];

/// Default ports to probe for generic dev servers
const DEV_SERVER_PORTS: [u16; 4] = [3000, 5173, 8080, 4200];

async fn detect_storybook(port: Option<u16>) -> Option<ProjectType> {
    let ports: Vec<u16> = match port {
        Some(p) => vec![p],
        None => STORYBOOK_PORTS.to_vec(),
    };

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

/// Detects a generic dev server by probing common ports.
async fn detect_dev_server(port: Option<u16>) -> Option<ProjectType> {
    let ports: Vec<u16> = match port {
        Some(p) => vec![p],
        None => DEV_SERVER_PORTS.to_vec(),
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;

    for port in ports {
        let base_url = format!("http://localhost:{}", port);
        debug!("Probing dev server at {}", base_url);

        if client.get(&base_url).send().await.is_ok() {
            info!("Found dev server at {}", base_url);
            return Some(ProjectType::DevServer { base_url });
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

        let dev_server = ProjectType::DevServer {
            base_url: "http://localhost:3000".to_string(),
        };
        assert!(dev_server.is_known());

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

        let dev_server = ProjectType::DevServer {
            base_url: "http://localhost:3000".to_string(),
        };
        assert_eq!(dev_server.base_url(), Some("http://localhost:3000"));

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

        let dev_server = ProjectType::DevServer {
            base_url: String::new(),
        };
        assert_eq!(dev_server.source_name(), "manual");

        let unknown = ProjectType::Unknown;
        assert_eq!(unknown.source_name(), "manual");
    }

    #[tokio::test]
    async fn detect_returns_unknown_for_empty_dir() {
        let dir = tempdir().unwrap();
        let result = detect_project_type(dir.path(), None).await.unwrap();
        assert!(matches!(result, ProjectType::Unknown));
    }
}
