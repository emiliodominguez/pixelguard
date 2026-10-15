//! Bounded execution of Node.js helper scripts.
//!
//! Capture and plugin hooks both run generated scripts through `node`. A hung
//! browser or a plugin that never resolves must not stall CI forever, so every
//! script runs under a wall-clock timeout and is killed when it expires.

use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

/// How often the child is polled for exit while waiting.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Runs `node <script_path> <args...>` in `working_dir`, killing it after `timeout`.
///
/// Stdout and stderr are drained on background threads so a chatty child can
/// never deadlock on a full pipe.
///
/// # Errors
///
/// Returns an error if Node.js cannot be started or the timeout expires.
pub fn run_node_script(
    script_path: &Path,
    args: &[&OsStr],
    working_dir: &Path,
    timeout: Duration,
) -> Result<Output> {
    let mut child = Command::new("node")
        .arg(script_path)
        .args(args)
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(
            "Failed to execute Node.js. Make sure Node.js is installed and available in PATH.",
        )?;

    let stdout_reader = drain(child.stdout.take());
    let stderr_reader = drain(child.stderr.take());

    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!(
                "❌ Node.js script timed out after {}s and was stopped.\n\n\
                 💡 Set PIXELGUARD_TIMEOUT_SECS to allow more time if this run is legitimately slow.",
                timeout.as_secs()
            );
        }
        std::thread::sleep(POLL_INTERVAL);
    };

    Ok(Output {
        status,
        stdout: stdout_reader.join().unwrap_or_default(),
        stderr: stderr_reader.join().unwrap_or_default(),
    })
}

/// Returns the timeout override from `PIXELGUARD_TIMEOUT_SECS`, or `default`.
pub fn timeout_from_env(default: Duration) -> Duration {
    std::env::var("PIXELGUARD_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
        .unwrap_or(default)
}

fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut buffer);
        }
        buffer
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_node_script_kills_scripts_that_exceed_the_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("hang.js");
        std::fs::write(&script, "setInterval(() => {}, 1000);").unwrap();

        let started = Instant::now();
        let result = run_node_script(&script, &[], dir.path(), Duration::from_millis(500));

        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn run_node_script_collects_output() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("ok.js");
        std::fs::write(&script, "console.log('out'); console.error('err');").unwrap();

        let output = run_node_script(&script, &[], dir.path(), Duration::from_secs(30)).unwrap();

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "out");
        assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), "err");
    }
}
