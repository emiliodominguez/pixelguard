//! The `test` command for capturing and comparing screenshots.
//!
//! This command captures screenshots of all configured shots,
//! compares them against the baseline, and generates an HTML report.
//! Supports plugins for capture, diff, report, and notification.
//!
//! The run only trusts what it captured itself: stale files on disk are never
//! compared, and a shot that fails to capture fails the run.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use clap::Args;
use pixelguard_core::{
    capture::{
        capture_screenshots_in_dir, prepare_current_dir, prune_baseline, update_baseline_exact,
        CaptureResult, CapturedShot, FailedShot,
    },
    config::{screenshot_name, Shot},
    diff::{diff_captured, has_baseline, parse_shot_name, DiffResult},
    fetch_storybook_stories, generate_report,
    plugins::{
        self, CaptureInput, CaptureNamedViewport, CaptureOutput, CaptureShot, CaptureViewport,
        NotifierInput, PluginCategory, PluginRegistry, ReporterChangedShot, ReporterConfig,
        ReporterDiffResult, ReporterInput,
    },
    Config,
};
use regex::Regex;
use tracing::{info, warn};

/// Arguments for the test command.
#[derive(Args)]
pub struct TestArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short, env = "PIXELGUARD_CONFIG")]
    config: Option<String>,

    /// Update baseline with current screenshots
    #[arg(long)]
    update: bool,

    /// Update only specific shots (comma-separated names, implies --update)
    #[arg(long, value_delimiter = ',')]
    update_only: Option<Vec<String>>,

    /// CI mode: machine-readable output, exit code 1 on diffs or failures
    #[arg(long)]
    ci: bool,

    /// Only test shots whose name contains this text
    #[arg(long)]
    filter: Option<String>,

    /// Show detailed progress
    #[arg(long)]
    verbose: bool,

    /// Serve the report in a local web server after completion
    #[arg(long)]
    serve: bool,

    /// Port for the local web server (default: 3333)
    #[arg(long, default_value = "3333")]
    port: u16,

    /// With --serve, do not open the report in a browser
    #[arg(long)]
    no_open: bool,
}

/// Runs the test command.
pub async fn run(args: TestArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config from custom path or default
    let mut config = super::load_config(&working_dir, args.config.as_deref())?;

    // Initialize plugins
    let plugin_registry = plugins::init_plugins(&config, &working_dir)?;
    if !plugin_registry.is_empty() && !args.ci {
        info!("🔌 Loaded {} plugin(s)", plugin_registry.len());
    }

    // Dynamically discover shots if source is storybook
    if config.source == "storybook" && !config.base_url.is_empty() {
        let discovered = discover_shots(&config).await?;
        if discovered.is_empty() {
            anyhow::bail!(
                "❌ Could not discover any stories from Storybook at {}\n\n\
                 🔍 Possible causes:\n  \
                 • Storybook is not running\n  \
                 • Wrong URL or port in config\n  \
                 • Network/firewall blocking the connection\n  \
                 • include/exclude patterns filter out every story\n\n\
                 💡 Solutions:\n  \
                 1️⃣ Start Storybook: npm run storybook\n  \
                 2️⃣ Verify it's accessible at {}\n  \
                 3️⃣ Check 'baseUrl' in pixelguard.config.json\n  \
                 4️⃣ Try running 'pixelguard validate' to diagnose issues",
                config.base_url,
                config.base_url
            );
        }
        // Merge with any overrides from config
        config.shots = merge_shots(discovered, &config.shots);
        config.validate()?;
    }

    if config.shots.is_empty() {
        anyhow::bail!(
            "❌ No shots configured.\n\n\
             💡 Solutions:\n  \
             1️⃣ Run 'pixelguard init' to auto-detect your project, or\n  \
             2️⃣ Add shots manually to pixelguard.config.json:\n\n     \
                {{\n       \
                  \"shots\": [\n         \
                    {{\"name\": \"home\", \"path\": \"/\"}}\n       \
                  ]\n     \
                }}\n\n  \
             3️⃣ See docs/configuration.md for all options"
        );
    }

    // Every screenshot name the unfiltered config produces; anything else in the
    // baseline belongs to a shot that no longer exists.
    let known: HashSet<String> = config.screenshot_names().into_iter().collect();

    // Apply filter if specified
    if let Some(pattern) = &args.filter {
        let original_count = config.shots.len();
        config
            .shots
            .retain(|shot| shot.name.contains(pattern.as_str()));

        if config.shots.is_empty() {
            anyhow::bail!(
                "❌ No shots match filter '{}'.\n\
                 📊 {} shots were filtered out.\n\n\
                 💡 Solutions:\n  \
                 • The filter matches any part of a shot name; check it for typos\n  \
                 • Use 'pixelguard list' to see all available shots",
                pattern,
                original_count
            );
        }

        if !args.ci {
            println!(
                "🔍 Filtered to {} of {} shots matching '{}'",
                config.shots.len(),
                original_count,
                pattern
            );
        }
    }

    let screenshot_count = config.shots.len() * config.effective_viewports().len();

    // Early validation: Check Node.js and Playwright before capturing
    // Skip if using a capture plugin (it handles its own dependencies)
    if !plugin_registry.has_override(PluginCategory::Capture) {
        validate_capture_environment(&working_dir)?;
    }

    if !args.ci {
        println!("📸 Capturing {} screenshots...", screenshot_count);
    }

    // Capture screenshots (using plugin if available)
    let capture_result = capture_with_plugin(&config, &working_dir, &plugin_registry).await?;

    if !capture_result.failed.is_empty() {
        eprintln!(
            "\n⚠️  {} screenshot(s) failed to capture:",
            capture_result.failed.len()
        );
        for failed in &capture_result.failed {
            eprintln!("   ❌ {}: {}", failed.name, failed.error);
        }
        eprintln!("\n💡 Tip: Check that your dev server is running and URLs are correct.");
        eprintln!();
    }

    // Handle --update or --update-only flag
    if args.update || args.update_only.is_some() {
        return run_update(
            &args,
            &config,
            &working_dir,
            &plugin_registry,
            &capture_result,
            &known,
        );
    }

    // Check if baseline exists
    if !has_baseline(&config, &working_dir) {
        if args.ci {
            println!(
                "{}",
                serde_json::json!({
                    "status": "no_baseline",
                    "captured": capture_result.captured.len(),
                    "failed": capture_result.failed.len(),
                })
            );
            eprintln!(
                "❌ No baseline found in '{}'. Create it locally with 'pixelguard test --update' \
                 and commit it; CI cannot pass without something to compare against.",
                working_dir
                    .join(&config.output_dir)
                    .join("baseline")
                    .display()
            );
            std::process::exit(1);
        }

        println!(
            "\n📁 No baseline found. This appears to be your first run.\n\n\
             💡 To create your baseline:\n  \
             pixelguard test --update\n\n\
             📝 After creating the baseline:\n  \
             1️⃣ Commit .pixelguard/baseline/ to version control\n  \
             2️⃣ Run 'pixelguard test' to detect visual changes\n  \
             3️⃣ Use 'pixelguard review' for interactive approval"
        );
        return Ok(());
    }

    // Compare against baseline
    if !args.ci {
        println!("🔍 Comparing against baseline...");
    }

    let filter = args.filter.clone();
    let diff_result = diff_captured(
        &config,
        &working_dir,
        Some(&plugin_registry),
        &capture_result,
        &known,
        |name| {
            filter
                .as_deref()
                .is_none_or(|pattern| parse_shot_name(name).0.contains(pattern))
        },
    )?;

    // Generate built-in report
    let report_path = generate_report(&config, &diff_result, &working_dir)?;

    // Plugins run before any exit so CI failures still produce reports and notifications.
    let mut plugin_errors = Vec::new();
    if let Err(error) = run_reporter_plugins(&config, &diff_result, &working_dir, &plugin_registry)
    {
        plugin_errors.push(error);
    }
    if let Err(error) = run_notifier_plugins(
        &diff_result,
        Some(&report_path),
        args.ci,
        &working_dir,
        &plugin_registry,
    ) {
        plugin_errors.push(error);
    }

    if args.ci {
        let result = serde_json::json!({
            "status": if diff_result.passed() { "pass" } else { "fail" },
            "unchanged": diff_result.unchanged.len(),
            "changed": diff_result.changed.len(),
            "added": diff_result.added.len(),
            "removed": diff_result.removed.len(),
            "failed": diff_result.failed.len(),
            "report": report_path.to_string_lossy(),
        });
        println!("{}", serde_json::to_string(&result)?);
    } else {
        print_summary(&diff_result);

        if args.serve {
            println!("\n🚀 Starting server...");
        } else {
            println!("\n📊 View report: {}", report_path.display());
        }

        if diff_result.has_changes() {
            println!("\n💡 To update baseline: pixelguard test --update");
        }
    }

    for error in &plugin_errors {
        eprintln!("⚠️  {:#}", error);
    }

    if args.ci && (!diff_result.passed() || !plugin_errors.is_empty()) {
        std::process::exit(1);
    }

    // Serve the report if requested
    if args.serve && !args.ci {
        let output_dir = working_dir.join(&config.output_dir);
        super::server::serve_report(&output_dir, args.port, !args.no_open).await?;
    }

    if let Some(error) = plugin_errors.into_iter().next() {
        return Err(error);
    }

    Ok(())
}

/// Promotes captured screenshots to the baseline for `--update` / `--update-only`.
///
/// Only screenshots captured in this run are copied. Baselines for shots that
/// no longer exist are pruned, but only after a complete, unfiltered,
/// failure-free run, so a flaky capture can never delete a baseline.
fn run_update(
    args: &TestArgs,
    config: &Config,
    working_dir: &Path,
    plugin_registry: &PluginRegistry,
    capture_result: &CaptureResult,
    known: &HashSet<String>,
) -> Result<()> {
    let targets: Vec<String> = capture_result
        .captured
        .iter()
        .map(|shot| shot.name.clone())
        .filter(|name| match &args.update_only {
            None => true,
            Some(selected) => selected
                .iter()
                .any(|s| name == s || parse_shot_name(name).0 == s.as_str()),
        })
        .collect();

    let updated = update_baseline_exact(config, working_dir, Some(plugin_registry), &targets)?;

    let complete_run =
        args.filter.is_none() && args.update_only.is_none() && capture_result.failed.is_empty();
    let pruned = if complete_run {
        prune_baseline(config, working_dir, Some(plugin_registry), known).unwrap_or_else(|error| {
            warn!("⚠️  Could not prune stale baselines: {:#}", error);
            Vec::new()
        })
    } else {
        Vec::new()
    };

    if args.ci {
        println!(
            "{}",
            serde_json::json!({
                "status": if capture_result.failed.is_empty() { "updated" } else { "partial" },
                "count": updated.len(),
                "pruned": pruned.len(),
                "failed": capture_result.failed.len(),
            })
        );
    } else {
        match &args.update_only {
            Some(names) => {
                println!(
                    "\n✅ Updated {} baseline(s) for: {}",
                    updated.len(),
                    names.join(", ")
                );
                if updated.is_empty() {
                    println!("   ℹ️  No captured screenshot matched those names.");
                }
            }
            None => println!("\n✅ Updated baseline with {} screenshot(s)", updated.len()),
        }
        if !pruned.is_empty() {
            println!(
                "🧹 Removed {} baseline(s) for shots that no longer exist:",
                pruned.len()
            );
            for name in &pruned {
                println!("   └─ {}", name);
            }
        }
        println!("💡 Tip: Commit .pixelguard/baseline/ to your version control.");
    }

    if !capture_result.failed.is_empty() {
        anyhow::bail!(
            "❌ {} screenshot(s) failed to capture, so their baselines were not updated.",
            capture_result.failed.len()
        );
    }

    Ok(())
}

fn print_summary(diff_result: &DiffResult) {
    println!();

    if !diff_result.unchanged.is_empty() {
        println!("✅ {} unchanged", diff_result.unchanged.len());
    }

    if !diff_result.changed.is_empty() {
        println!("❌ {} changed", diff_result.changed.len());
        for shot in &diff_result.changed {
            println!(
                "   └─ {} ({:.2}% different)",
                shot.name, shot.diff_percentage
            );
        }
    }

    if !diff_result.added.is_empty() {
        println!("➕ {} added", diff_result.added.len());
        for name in &diff_result.added {
            println!("   └─ {}", name);
        }
    }

    if !diff_result.removed.is_empty() {
        println!("➖ {} removed", diff_result.removed.len());
        for name in &diff_result.removed {
            println!("   └─ {}", name);
        }
    }

    if !diff_result.failed.is_empty() {
        println!("💥 {} failed", diff_result.failed.len());
        for failure in &diff_result.failed {
            println!("   └─ {}: {}", failure.name, failure.error);
        }
    }
}

/// Captures screenshots using a plugin if available, otherwise uses built-in capture.
async fn capture_with_plugin(
    config: &Config,
    working_dir: &Path,
    registry: &PluginRegistry,
) -> Result<CaptureResult> {
    let Some(plugin) = registry.get(PluginCategory::Capture) else {
        return capture_screenshots_in_dir(config, working_dir).await;
    };

    info!("🔌 Using capture plugin: {}", plugin.name());

    let current_dir = prepare_current_dir(config, working_dir)?;
    let viewports = config.effective_viewports();

    let input = CaptureInput {
        shots: config
            .shots
            .iter()
            .map(|s| CaptureShot {
                name: s.name.clone(),
                path: s.path.clone(),
                wait_for: s.wait_for.clone(),
                delay: s.delay,
            })
            .collect(),
        base_url: config.base_url.trim_end_matches('/').to_string(),
        viewport: CaptureViewport {
            width: config.viewport.width,
            height: config.viewport.height,
        },
        viewports: viewports
            .iter()
            .map(|v| CaptureNamedViewport {
                name: v.name.clone(),
                width: v.width,
                height: v.height,
            })
            .collect(),
        output_dir: current_dir.to_string_lossy().to_string(),
        options: serde_json::json!({}),
    };

    let output: CaptureOutput =
        plugins::executor::execute_hook(plugin, "capture", &input, working_dir)?;

    Ok(reconcile_plugin_capture(config, &current_dir, output))
}

/// Checks a capture plugin's report against what was asked for.
///
/// Only expected names whose file actually exists in `current_dir` count as
/// captured; everything else expected is a failure. Unexpected names are ignored.
fn reconcile_plugin_capture(
    config: &Config,
    current_dir: &Path,
    output: CaptureOutput,
) -> CaptureResult {
    let mut captured = Vec::new();
    let mut failed = Vec::new();

    for shot in &config.shots {
        for viewport in config.effective_viewports() {
            let name = screenshot_name(&shot.name, &viewport.name);
            let path = current_dir.join(format!("{}.png", name));
            if let Some(failure) = output.failed.iter().find(|f| f.name == name) {
                failed.push(FailedShot {
                    name,
                    error: failure.error.clone(),
                });
            } else if path.is_file() {
                captured.push(CapturedShot { name, path });
            } else {
                failed.push(FailedShot {
                    name,
                    error: "Capture plugin did not produce this screenshot".to_string(),
                });
            }
        }
    }

    CaptureResult { captured, failed }
}

/// Runs all registered reporter plugins.
fn run_reporter_plugins(
    config: &Config,
    diff_result: &DiffResult,
    working_dir: &Path,
    registry: &PluginRegistry,
) -> Result<()> {
    let reporters = registry.reporters();
    if reporters.is_empty() {
        return Ok(());
    }

    let output_dir = working_dir
        .join(&config.output_dir)
        .to_string_lossy()
        .to_string();

    let input = ReporterInput {
        result: convert_diff_result(diff_result),
        config: ReporterConfig {
            source: config.source.clone(),
            base_url: config.base_url.clone(),
            threshold: config.threshold,
        },
        output_dir,
        options: serde_json::json!({}),
    };

    for plugin in reporters {
        info!("📊 Running reporter plugin: {}", plugin.name());
        let _output: serde_json::Value =
            plugins::executor::execute_hook(plugin, "generate", &input, working_dir)?;
    }

    Ok(())
}

/// Runs all registered notifier plugins.
fn run_notifier_plugins(
    diff_result: &DiffResult,
    report_path: Option<&Path>,
    ci_mode: bool,
    working_dir: &Path,
    registry: &PluginRegistry,
) -> Result<()> {
    let notifiers = registry.notifiers();
    if notifiers.is_empty() {
        return Ok(());
    }

    let input = NotifierInput {
        result: convert_diff_result(diff_result),
        report_path: report_path.map(|p| p.to_string_lossy().to_string()),
        report_url: None,
        ci_mode,
        options: serde_json::json!({}),
    };

    for plugin in notifiers {
        info!("🔔 Running notifier plugin: {}", plugin.name());
        plugins::executor::execute_hook_void(plugin, "notify", &input, working_dir)?;
    }

    Ok(())
}

/// Converts DiffResult to the plugin-compatible format.
fn convert_diff_result(diff_result: &DiffResult) -> ReporterDiffResult {
    ReporterDiffResult {
        unchanged: diff_result.unchanged.clone(),
        changed: diff_result
            .changed
            .iter()
            .map(|c| ReporterChangedShot {
                name: c.name.clone(),
                baseline_path: c.baseline_path.to_string_lossy().to_string(),
                current_path: c.current_path.to_string_lossy().to_string(),
                diff_path: c.diff_path.to_string_lossy().to_string(),
                diff_percentage: c.diff_percentage,
            })
            .collect(),
        added: diff_result.added.clone(),
        removed: diff_result.removed.clone(),
        failed: diff_result.failed.iter().map(|f| f.name.clone()).collect(),
    }
}

/// Discovers shots dynamically from the source (e.g., Storybook).
pub async fn discover_shots(config: &Config) -> Result<Vec<Shot>> {
    match config.source.as_str() {
        "storybook" => {
            let stories = fetch_storybook_stories(&config.base_url)
                .await
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Failed to fetch stories from Storybook at {}. Is it running?",
                        config.base_url
                    )
                })?;

            let include = compile_patterns(&config.include)?;
            let exclude = compile_patterns(&config.exclude)?;

            Ok(stories
                .into_iter()
                .filter(|shot| matches_patterns(&shot.name, &include, &exclude))
                .collect())
        }
        _ => Ok(Vec::new()),
    }
}

/// Checks if a name matches include patterns and doesn't match exclude patterns.
fn matches_patterns(name: &str, include: &[Regex], exclude: &[Regex]) -> bool {
    if exclude.iter().any(|re| re.is_match(name)) {
        return false;
    }
    include.is_empty() || include.iter().any(|re| re.is_match(name))
}

/// Compiles glob patterns, reporting the first invalid one.
fn compile_patterns(patterns: &[String]) -> Result<Vec<Regex>> {
    patterns
        .iter()
        .map(|pattern| {
            glob_to_regex(pattern)
                .with_context(|| format!("Invalid include/exclude pattern '{}'", pattern))
        })
        .collect()
}

/// Converts a glob into a case-insensitive, anchored regex over story ids.
///
/// `*` and `**` match any run of characters, `?` matches one character and a
/// leading `**/` is optional, so `**/*deprecated*` matches the story id
/// `components-button--deprecated`. Every other character is literal.
fn glob_to_regex(pattern: &str) -> Result<Regex> {
    let mut body = pattern;
    let mut regex = String::from("(?i)^");
    while let Some(rest) = body.strip_prefix("**/") {
        body = rest;
        regex.push_str("(?:.*/)?");
    }

    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' => {
                while chars.peek() == Some(&'*') {
                    chars.next();
                }
                regex.push_str(".*");
            }
            '?' => regex.push('.'),
            other => regex.push_str(&regex::escape(&other.to_string())),
        }
    }
    regex.push('$');

    Ok(Regex::new(&regex)?)
}

/// Merges discovered shots with config overrides.
///
/// Config shots can provide custom waitFor or delay for a discovered story.
pub fn merge_shots(discovered: Vec<Shot>, overrides: &[Shot]) -> Vec<Shot> {
    discovered
        .into_iter()
        .map(|mut shot| {
            if let Some(override_shot) = overrides.iter().find(|o| o.name == shot.name) {
                if override_shot.wait_for.is_some() {
                    shot.wait_for = override_shot.wait_for.clone();
                }
                if override_shot.delay.is_some() {
                    shot.delay = override_shot.delay;
                }
                // Path override is intentionally not applied - use discovered path
            }
            shot
        })
        .collect()
}

/// Validates that Node.js and Playwright are available before attempting capture.
///
/// Resolution mirrors the capture driver (`require.resolve` from the project),
/// so this never triggers an `npx` download and never disagrees with capture.
fn validate_capture_environment(working_dir: &Path) -> Result<()> {
    let output = Command::new("node")
        .arg("-e")
        .arg(PLAYWRIGHT_PROBE)
        .arg(working_dir)
        .output()
        .context(
            "❌ Node.js is required for screenshot capture but was not found.\n\n\
             💡 Installation:\n  \
             • Download from https://nodejs.org (LTS version recommended)\n  \
             • Or use a version manager like nvm or fnm\n\n  \
             🔍 After installation, restart your terminal and try again.",
        )?;

    if !output.status.success() {
        anyhow::bail!(
            "❌ Playwright is required for screenshot capture but was not found.\n\n\
             💡 Installation steps:\n  \
             1️⃣ Install Playwright: npm install -D playwright\n  \
             2️⃣ Install browsers: npx playwright install chromium\n\n  \
             📝 Note: Chromium is the only browser required for screenshots.\n  \
             🔗 Docs: https://playwright.dev/docs/intro"
        );
    }

    Ok(())
}

/// Node snippet that exits 0 when a Playwright package resolves from argv[1].
pub const PLAYWRIGHT_PROBE: &str = "const d=process.argv[1];\
for(const n of ['playwright','@playwright/test','playwright-core']){\
try{console.log(require(require.resolve(n+'/package.json',{paths:[d]})).version);process.exit(0)}catch(e){}}\
process.exit(1)";

#[cfg(test)]
mod tests {
    use super::*;

    fn patterns(globs: &[&str]) -> Vec<Regex> {
        compile_patterns(&globs.iter().map(|g| g.to_string()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn documented_exclude_pattern_matches_story_ids() {
        let exclude = patterns(&["**/*Deprecated*"]);
        let include = patterns(&["**/*"]);

        assert!(!matches_patterns(
            "components-button--deprecated",
            &include,
            &exclude
        ));
        assert!(matches_patterns(
            "components-button--primary",
            &include,
            &exclude
        ));
    }

    #[test]
    fn glob_treats_regex_metacharacters_literally() {
        let include = patterns(&["button.primary", "card(1)*"]);

        assert!(matches_patterns("button.primary", &include, &[]));
        assert!(!matches_patterns("buttonXprimary", &include, &[]));
        assert!(matches_patterns("card(1)--large", &include, &[]));
    }

    #[test]
    fn empty_include_list_includes_everything() {
        assert!(matches_patterns("anything", &[], &[]));
    }

    #[test]
    fn plugin_capture_must_produce_every_expected_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.png"), b"png").unwrap();
        let config = Config {
            shots: vec![
                Shot {
                    name: "a".to_string(),
                    path: "/a".to_string(),
                    wait_for: None,
                    delay: None,
                },
                Shot {
                    name: "b".to_string(),
                    path: "/b".to_string(),
                    wait_for: None,
                    delay: None,
                },
            ],
            ..Default::default()
        };
        let output = CaptureOutput {
            captured: vec![
                plugins::CapturedShot {
                    name: "a".to_string(),
                    path: "a.png".to_string(),
                },
                plugins::CapturedShot {
                    name: "b".to_string(),
                    path: "b.png".to_string(),
                },
                plugins::CapturedShot {
                    name: "../../evil".to_string(),
                    path: "/etc/passwd".to_string(),
                },
            ],
            failed: vec![],
        };

        let result = reconcile_plugin_capture(&config, dir.path(), output);

        assert_eq!(result.captured.len(), 1);
        assert_eq!(result.captured[0].name, "a");
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].name, "b");
    }
}
