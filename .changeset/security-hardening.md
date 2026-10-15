---
"pixelguard": minor
"pixelguard-plugin-types": minor
---

Security and correctness hardening from an adversarial review.

Breaking behavior changes:

- A shot that fails to capture now fails the run (exit code 1 with `--ci`). Previously a stale screenshot from an earlier run was compared instead and could pass.
- `--ci` without a baseline now exits 1 instead of passing.
- The config is validated on load: file-name safe, unique shot and viewport names; `threshold` 0–100; `concurrency` 1–64; viewport sizes up to 16384; `delay` up to 5 minutes; shot paths must start with `/`.
- `apply` and `review` only promote the exact image that was reviewed (SHA-256 recorded in decisions). Use `apply --allow-unverified` for decisions saved by older reports.
- Approving `button` in `apply`/`review` no longer also approves `button@mobile`.

Fixes:

- Shot and viewport names can no longer write files outside the output directory.
- The report server rejects foreign `Host` headers (DNS rebinding) and cross-origin writes, validates decisions, and forbids framing. New `--no-open` flag.
- `--filter` no longer reports every other shot as removed.
- Reporter and notifier plugins now run in CI mode before the exit code is set.
- Capture and plugin hooks run under a timeout (`PIXELGUARD_TIMEOUT_SECS`); plugins may log to stdout; ESM plugins load.
- Capture plugins receive every viewport (`viewports`); `DiffResult` gains `failed`.
- Logs go to stderr, so `--ci` stdout is pure JSON.
- `exclude: ["**/*Deprecated*"]` and other globs now match story ids as documented.
- `--update` prunes baselines of deleted shots after a complete run.
- `PIXELGUARD_CONFIG` is now honored, as documented.
- npm: the binary download is verified against bundled SHA-256 checksums and installed atomically; the wrapper can no longer re-exec itself in a loop, and a binary killed by a signal no longer exits 0.
