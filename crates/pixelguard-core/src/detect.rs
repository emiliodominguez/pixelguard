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
use tracing::{debug, info, warn};

use crate::config::{validate_shot, Shot};

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
struct StorybookIndex {
    entries: Option<std::collections::HashMap<String, StorybookEntry>>,
}

/// Storybook stories.json format (older versions)
#[derive(Debug, Deserialize)]
struct StorybookStories {
    stories: Option<std::collections::HashMap<String, StorybookStory>>,
}

#[derive(Debug, Deserialize)]
struct StorybookEntry {
    id: String,
    #[serde(rename = "type")]
    entry_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StorybookStory {
    id: String,
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
        info!("✨ Found .storybook directory");
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
        debug!("🔍 Probing Storybook at {}", base_url);

        if let Some(stories) = fetch_storybook_stories(&base_url).await {
            info!(
                "✨ Found Storybook at {} with {} stories",
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

    let base_url = base_url.trim_end_matches('/');

    // Try index.json first (Storybook 7+)
    let index_url = format!("{}/index.json", base_url);
    if let Ok(response) = client.get(&index_url).send().await {
        if response.status().is_success() {
            if let Ok(index) = response.json::<StorybookIndex>().await {
                if let Some(entries) = index.entries {
                    let shots = stories_to_shots(
                        entries
                            .into_values()
                            .filter(|entry| entry.entry_type.as_deref() != Some("docs"))
                            .map(|entry| entry.id),
                    );

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
                    let shots = stories_to_shots(stories_map.into_values().map(|story| story.id));

                    if !shots.is_empty() {
                        return Some(shots);
                    }
                }
            }
        }
    }

    None
}

/// Turns Storybook story ids into shots, sorted by name.
///
/// Story ids come from a server response, so each one is validated like a
/// hand-written shot name and percent-encoded in the URL; ids that would be
/// unsafe as file names are skipped with a warning.
fn stories_to_shots(ids: impl Iterator<Item = String>) -> Vec<Shot> {
    let mut shots: Vec<Shot> = ids
        .filter_map(|id| {
            let shot = Shot {
                path: format!("/iframe.html?id={}&viewMode=story", encode_query_value(&id)),
                name: id,
                wait_for: Some("#storybook-root".to_string()),
                delay: Some(100),
            };
            match validate_shot(&shot) {
                Ok(()) => Some(shot),
                Err(error) => {
                    warn!("⚠️  Skipping story {:?}: {:#}", shot.name, error);
                    None
                }
            }
        })
        .collect();
    shots.sort_by(|a, b| a.name.cmp(&b.name));
    shots.dedup_by(|a, b| a.name == b.name);
    shots
}

/// Percent-encodes a query string value.
fn encode_query_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{:02X}", byte));
        }
    }
    encoded
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
        debug!("🔍 Probing dev server at {}", base_url);

        if client.get(&base_url).send().await.is_ok() {
            info!("✅ Found dev server at {}", base_url);
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

    #[test]
    fn stories_to_shots_skips_unsafe_ids_and_encodes_urls() {
        let shots = stories_to_shots(
            vec![
                "button--primary".to_string(),
                "../../../etc/evil".to_string(),
                "card--a b".to_string(),
                "button--primary".to_string(),
            ]
            .into_iter(),
        );

        let names: Vec<&str> = shots.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["button--primary", "card--a b"]);
        assert_eq!(shots[1].path, "/iframe.html?id=card--a%20b&viewMode=story");
    }

    #[tokio::test]
    async fn detect_returns_unknown_for_empty_dir() {
        let dir = tempdir().unwrap();
        // Use a specific port that's unlikely to have a service running
        // to avoid flakiness from detecting dev servers on common ports
        let result = detect_project_type(dir.path(), Some(59999)).await.unwrap();
        assert!(matches!(result, ProjectType::Unknown));
    }
}
