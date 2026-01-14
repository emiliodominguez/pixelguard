//! Integration tests for the Pixelguard CLI.
//!
//! These tests verify end-to-end behavior of CLI commands.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Returns a Command configured to run pixelguard.
fn pixelguard() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("pixelguard"))
}

#[test]
fn help_command_shows_usage() {
    pixelguard()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("visual regression testing"));
}

#[test]
fn version_flag_shows_version() {
    pixelguard()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pixelguard"));
}

#[test]
fn list_command_shows_no_shots_without_config() {
    let dir = tempdir().unwrap();

    pixelguard()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No shots configured"));
}

#[test]
fn list_command_shows_shots_from_config() {
    let dir = tempdir().unwrap();

    // Create a config with shots
    let config = r#"{
        "source": "manual",
        "baseUrl": "http://localhost:3000",
        "shots": [
            { "name": "home", "path": "/" },
            { "name": "about", "path": "/about" }
        ]
    }"#;
    fs::write(dir.path().join("pixelguard.config.json"), config).unwrap();

    pixelguard()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("home"))
        .stdout(predicate::str::contains("about"))
        .stdout(predicate::str::contains("Configured shots (2)"));
}

#[test]
fn list_command_json_output() {
    let dir = tempdir().unwrap();

    let config = r#"{
        "shots": [
            { "name": "test-shot", "path": "/test" }
        ]
    }"#;
    fs::write(dir.path().join("pixelguard.config.json"), config).unwrap();

    pixelguard()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""name": "test-shot""#));
}

#[test]
fn plugins_command_shows_no_plugins() {
    let dir = tempdir().unwrap();

    // Create empty config
    fs::write(dir.path().join("pixelguard.config.json"), "{}").unwrap();

    pixelguard()
        .current_dir(dir.path())
        .arg("plugins")
        .assert()
        .success()
        .stdout(predicate::str::contains("No plugins configured"));
}

#[test]
fn validate_command_checks_environment() {
    let dir = tempdir().unwrap();

    // Create minimal config
    fs::write(dir.path().join("pixelguard.config.json"), "{}").unwrap();

    // Run validate - should pass config check at least
    pixelguard()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        // May fail on Node.js/Playwright check in CI, but should run
        .stdout(predicate::str::contains("config"));
}

#[test]
fn validate_command_json_output() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("pixelguard.config.json"), "{}").unwrap();

    pixelguard()
        .current_dir(dir.path())
        .args(["validate", "--json"])
        .assert()
        .stdout(predicate::str::contains(r#""check""#))
        .stdout(predicate::str::contains(r#""passed""#));
}

#[test]
fn init_command_creates_config() {
    let dir = tempdir().unwrap();

    // Create .storybook directory to trigger storybook detection
    fs::create_dir(dir.path().join(".storybook")).unwrap();

    pixelguard()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Config file should be created
    assert!(dir.path().join("pixelguard.config.json").exists());
}

#[test]
fn init_command_respects_force_flag() {
    let dir = tempdir().unwrap();

    // Create existing config
    fs::write(
        dir.path().join("pixelguard.config.json"),
        r#"{"source": "manual"}"#,
    )
    .unwrap();

    // Create .storybook to trigger detection
    fs::create_dir(dir.path().join(".storybook")).unwrap();

    // Without --force, should fail with error
    pixelguard()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // With --force, should overwrite successfully
    pixelguard()
        .current_dir(dir.path())
        .args(["init", "--force"])
        .assert()
        .success();
}

#[test]
fn test_command_requires_config() {
    let dir = tempdir().unwrap();

    pixelguard()
        .current_dir(dir.path())
        .arg("test")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No shots configured")
                .or(predicate::str::contains("Could not read config")),
        );
}

#[test]
fn test_command_with_empty_shots() {
    let dir = tempdir().unwrap();

    // Config with no shots
    fs::write(
        dir.path().join("pixelguard.config.json"),
        r#"{"source": "manual", "shots": []}"#,
    )
    .unwrap();

    pixelguard()
        .current_dir(dir.path())
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No shots configured"));
}

#[test]
fn config_custom_path_flag_works() {
    let dir = tempdir().unwrap();

    // Create config at custom location
    let custom_config = dir.path().join("custom.json");
    fs::write(
        &custom_config,
        r#"{"shots": [{"name": "custom", "path": "/"}]}"#,
    )
    .unwrap();

    pixelguard()
        .current_dir(dir.path())
        .args(["list", "--config", "custom.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("custom"));
}

#[test]
fn invalid_config_json_fails_gracefully() {
    let dir = tempdir().unwrap();

    // Write invalid JSON
    fs::write(
        dir.path().join("pixelguard.config.json"),
        "{ invalid json }",
    )
    .unwrap();

    pixelguard()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON"));
}
