//! The review decisions file shared by the report server, `apply` and `review`.
//!
//! Decisions are written by the browser report (or the local server) and later
//! turned into baseline updates, so they are parsed strictly: unknown actions
//! and unsafe screenshot names are rejected rather than ignored.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use pixelguard_core::capture::validate_screenshot_name;
use pixelguard_core::report::screenshot_hash;
use serde::{Deserialize, Serialize};

/// Name of the decisions file inside the output directory.
pub const DECISIONS_FILE: &str = "decisions.json";

/// Structure of the decisions.json file exported from the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionsFile {
    /// Schema version
    #[serde(default = "default_version")]
    pub version: String,
    /// When the file was exported
    #[serde(default)]
    pub exported_at: String,
    /// The decisions made for each screenshot, keyed by screenshot name
    pub decisions: BTreeMap<String, Decision>,
}

/// A single decision for a screenshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// The action taken
    pub action: DecisionAction,
    /// ISO timestamp when the decision was made
    #[serde(default)]
    pub timestamp: String,
    /// Where the decision was made (e.g., "browser")
    #[serde(default)]
    pub source: String,
    /// SHA-256 of the current image the reviewer looked at
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// What the reviewer decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecisionAction {
    /// Promote the current screenshot to baseline
    Approve,
    /// Keep the existing baseline
    Reject,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl DecisionsFile {
    /// Parses and validates decisions JSON.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON, unknown actions, unsafe screenshot
    /// names or hashes that are not 64 hex characters.
    pub fn parse(content: &str) -> Result<Self> {
        let file: DecisionsFile = serde_json::from_str(content)?;
        file.validate()?;
        Ok(file)
    }

    /// Validates every key and hash.
    ///
    /// # Errors
    ///
    /// Returns an error naming the first invalid entry.
    pub fn validate(&self) -> Result<()> {
        for (name, decision) in &self.decisions {
            validate_screenshot_name(name)
                .with_context(|| format!("Invalid screenshot name in decisions: {:?}", name))?;
            if let Some(hash) = &decision.hash {
                let valid = hash.is_empty()
                    || (hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()));
                if !valid {
                    anyhow::bail!("Invalid image hash for '{}' in decisions", name);
                }
            }
        }
        Ok(())
    }

    /// Names with the given action, sorted.
    pub fn names_with(&self, action: DecisionAction) -> Vec<String> {
        self.decisions
            .iter()
            .filter(|(_, decision)| decision.action == action)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

/// Outcome of checking an approval against the image currently on disk.
#[derive(Debug, PartialEq, Eq)]
pub enum ApprovalCheck {
    /// The current image is exactly the one that was reviewed
    Matches,
    /// The decision predates image hashing; nothing to compare against
    Unverified,
    /// The current image changed after it was reviewed
    Stale,
    /// There is no current image for this name
    Missing,
}

/// Checks that `current/{name}.png` is still the image the reviewer approved.
pub fn check_approval(output_dir: &Path, name: &str, expected_hash: Option<&str>) -> ApprovalCheck {
    let path = output_dir.join("current").join(format!("{}.png", name));
    let Ok(actual) = screenshot_hash(&path) else {
        return ApprovalCheck::Missing;
    };
    match expected_hash {
        None | Some("") => ApprovalCheck::Unverified,
        Some(expected) if expected.eq_ignore_ascii_case(&actual) => ApprovalCheck::Matches,
        Some(_) => ApprovalCheck::Stale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_path_traversal_keys() {
        let json = r#"{"decisions":{"../../etc/x":{"action":"approve"}}}"#;
        assert!(DecisionsFile::parse(json).is_err());
    }

    #[test]
    fn parse_rejects_unknown_actions() {
        let json = r#"{"decisions":{"button":{"action":"delete-everything"}}}"#;
        assert!(DecisionsFile::parse(json).is_err());
    }

    #[test]
    fn parse_accepts_browser_exports() {
        let json = r#"{"version":"1.0","exportedAt":"2026-01-01T00:00:00Z","decisions":{
            "button@mobile":{"action":"approve","timestamp":"t","source":"browser","hash":""},
            "card":{"action":"reject","timestamp":"t","source":"browser"}}}"#;
        let file = DecisionsFile::parse(json).unwrap();
        assert_eq!(
            file.names_with(DecisionAction::Approve),
            vec!["button@mobile"]
        );
        assert_eq!(file.names_with(DecisionAction::Reject), vec!["card"]);
    }

    #[test]
    fn check_approval_detects_images_that_changed_after_review() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("current");
        std::fs::create_dir_all(&current).unwrap();
        std::fs::write(current.join("a.png"), b"reviewed").unwrap();
        let reviewed = screenshot_hash(&current.join("a.png")).unwrap();

        assert_eq!(
            check_approval(dir.path(), "a", Some(&reviewed)),
            ApprovalCheck::Matches
        );

        std::fs::write(current.join("a.png"), b"something else").unwrap();
        assert_eq!(
            check_approval(dir.path(), "a", Some(&reviewed)),
            ApprovalCheck::Stale
        );
        assert_eq!(
            check_approval(dir.path(), "a", None),
            ApprovalCheck::Unverified
        );
        assert_eq!(
            check_approval(dir.path(), "missing", None),
            ApprovalCheck::Missing
        );
    }
}
