# Architecture Overview

This document explains the internal architecture of Pixelguard, including crate structure, module responsibilities, and data flow.

## Crate Structure

Pixelguard is organized as a Cargo workspace with two crates:

```
crates/
├── pixelguard-cli/     # CLI binary
│   └── src/
│       ├── main.rs     # Entry point, CLI parsing
│       └── commands/   # Command implementations
│           ├── init.rs
│           ├── test.rs
│           ├── list.rs
│           ├── plugins.rs
│           └── validate.rs
│
└── pixelguard-core/    # Core library
    └── src/
        ├── lib.rs      # Public API exports
        ├── config.rs   # Configuration management
        ├── detect.rs   # Project type detection
        ├── capture.rs  # Screenshot capture
        ├── diff.rs     # Image comparison
        ├── report.rs   # HTML report generation
        ├── storage.rs  # File storage abstraction
        └── plugins/    # Plugin system
            ├── mod.rs
            ├── types.rs
            ├── discovery.rs
            ├── loader.rs
            ├── registry.rs
            └── executor.rs
```

### pixelguard-cli

The CLI crate is the user-facing binary. It handles:

- Command-line argument parsing (using [clap](https://docs.rs/clap))
- User interaction and output formatting
- Orchestrating core library functions

The CLI is intentionally thin - most logic lives in `pixelguard-core`.

### pixelguard-core

The core library contains all business logic. It's designed to be:

- Usable as a library (not just via CLI)
- Well-tested with unit tests
- Decoupled from I/O where possible

## Module Responsibilities

### config.rs

Manages the `pixelguard.config.json` file:

- Loads and parses JSON configuration
- Provides sensible defaults for all fields
- Validates configuration structure
- Supports both simple and complex plugin entries

Key types:
- `Config` - Main configuration struct
- `Shot` - Individual screenshot configuration
- `Viewport` - Screen dimensions
- `PluginEntry` - Plugin reference (string or object with options)

### detect.rs

Detects the project type and dev server:

- Checks for framework config files (`.storybook/`, `vite.config.js`, etc.)
- Probes common ports for running dev servers
- Fetches story lists from Storybook's `/index.json` endpoint
- Returns `ProjectType` with discovered configuration

Detection order:
1. Storybook (checks `.storybook/` directory)
2. Next.js (checks `next.config.*` files)
3. Vite (checks `vite.config.*` files)
4. Unknown (falls back to manual configuration)

### capture.rs

Handles screenshot capture using Playwright:

1. Generates a temporary Node.js script
2. Executes it via `node` subprocess
3. Script uses Playwright to:
   - Launch headless Chromium
   - Navigate to each shot URL
   - Wait for selectors/delays
   - Capture and save screenshots

Key types:
- `CaptureResult` - Success/failure for all shots
- `CapturedShot` - Successfully captured screenshot info
- `FailedShot` - Failed capture with error message

### diff.rs

Compares screenshots against baselines:

- Pixel-by-pixel comparison with anti-aliasing tolerance
- Generates diff images highlighting changes
- Handles size mismatches (always 100% different)
- Tracks added/removed/changed/unchanged shots

Key types:
- `DiffResult` - Complete comparison result
- `ChangedShot` - Shot with visual differences

Algorithm:
1. List all baseline and current screenshots
2. Identify added (no baseline) and removed (no current) shots
3. For matching shots, compare pixel data
4. Apply threshold to determine if "changed"
5. Generate diff image with red overlay on differences

### report.rs

Generates the HTML report:

- Self-contained HTML with embedded CSS/JS
- Comparison slider for visual diffs
- Image zoom modal for detailed inspection
- Dark/light theme support
- Summary statistics

### storage.rs

Abstracts file storage operations:

- `LocalStorage` - Default filesystem storage
- Designed for future cloud storage plugins (S3, etc.)

Operations:
- `read()` / `write()` - File I/O
- `exists()` / `list()` - Directory operations
- `copy()` / `delete()` - File management
- `is_remote()` - Check if storage is cloud-based

### plugins/

The plugin system enables extending Pixelguard:

- **discovery.rs** - Finds plugins in node_modules
- **loader.rs** - Loads plugin manifests and validates them
- **registry.rs** - Stores and organizes loaded plugins
- **executor.rs** - Runs plugin hooks via Node.js subprocess
- **types.rs** - Plugin type definitions and IPC contracts

## Data Flow

### Init Command Flow

```
┌─────────────┐
│ pixelguard  │
│    init     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  detect.rs  │ Check for .storybook/, vite.config.*, etc.
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Probe ports │ Try 6006, 6007, 3000, 5173...
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Fetch shots │ GET /index.json from Storybook
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  config.rs  │ Create and save config file
└─────────────┘
```

### Test Command Flow

```
┌─────────────┐
│ pixelguard  │
│    test     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  config.rs  │ Load configuration
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  plugins/   │ Initialize plugins (if any)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ detect.rs   │ Discover shots from Storybook (if configured)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ capture.rs  │ Generate Playwright script, execute via Node.js
└──────┬──────┘ Save screenshots to .pixelguard/current/
       │
       ▼
┌─────────────┐
│   diff.rs   │ Compare current vs baseline
└──────┬──────┘ Generate diff images
       │
       ▼
┌─────────────┐
│  report.rs  │ Generate HTML report
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  plugins/   │ Run reporter/notifier plugins
└─────────────┘
```

### Update Baseline Flow

```
┌─────────────┐
│ pixelguard  │
│test --update│
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ capture.rs  │ Capture current screenshots
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ storage.rs  │ Copy current/*.png → baseline/*.png
└─────────────┘
```

## Plugin System

### Plugin Categories

| Category | Behavior | Example Use Case |
|----------|----------|------------------|
| Storage | Single-winner (last wins) | S3 storage backend |
| Capture | Single-winner (last wins) | Custom browser automation |
| Differ | Single-winner (last wins) | ML-based image comparison |
| Reporter | Stackable (all run) | JUnit XML, JSON output |
| Notifier | Stackable (all run) | Slack, Teams, email |

### Plugin IPC Protocol

Plugins communicate with Pixelguard via JSON over stdin/stdout:

```
┌────────────────┐     JSON input      ┌────────────────┐
│   Pixelguard   │ ──────────────────► │  Node.js       │
│   (Rust)       │                     │  Plugin Runner │
│                │ ◄────────────────── │                │
└────────────────┘     JSON output     └────────────────┘
```

1. Pixelguard spawns Node.js subprocess
2. Sends JSON input via stdin
3. Plugin processes and writes JSON to stdout
4. Pixelguard parses response

Example plugin execution:
```javascript
// Input (stdin)
{
  "hook": "notify",
  "input": { "result": {...}, "ciMode": true },
  "options": { "webhookUrl": "..." }
}

// Output (stdout)
{ "success": true }
```

### Plugin Discovery

Plugins are discovered in order:
1. Local path (if starts with `./` or `/`)
2. `node_modules/{plugin-name}/`
3. `node_modules/@scope/{plugin-name}/`

Each plugin must have a `pixelguard-plugin.json` manifest:
```json
{
  "name": "pixelguard-plugin-slack",
  "category": "notifier",
  "entry": "dist/index.js",
  "hooks": ["notify"],
  "version": "1.0.0"
}
```

## Error Handling

Pixelguard uses two error handling strategies:

### Core Library (pixelguard-core)

Uses `anyhow::Result` for error propagation with context:

```rust
fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Could not read config file")?;

    serde_json::from_str(&content)
        .context("Invalid JSON in config file")
}
```

### User-Facing Errors

Error messages are actionable - they tell users what to do:

```rust
// Bad: "Failed to read config"
// Good: "Could not read config file at 'pixelguard.config.json'.
//        Run 'pixelguard init' to create one."
```

## Testing Strategy

### Unit Tests

Each module has `#[cfg(test)]` tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.viewport.width, 1280);
    }
}
```

### Integration Tests

Integration tests in `tests/` directory test end-to-end flows:

- `pixelguard init` creates valid config
- `pixelguard test` captures and compares screenshots
- `pixelguard validate` checks environment

## Performance Considerations

### Concurrency

Screenshot capture runs in parallel (default: 4 concurrent captures). The concurrency level is configurable via `config.concurrency`.

### Memory Usage

- Diff images are processed one at a time to limit memory
- Large baseline directories should use Git LFS

### Subprocess Overhead

Each plugin execution spawns a Node.js process. For many plugins, consider:
- Batching operations where possible
- Using built-in functionality when plugins aren't needed

## Future Architecture

Planned improvements:

1. **Multi-viewport support**: Test same shots at different screen sizes
2. **Parallel plugin execution**: Run independent plugins concurrently
3. **Incremental testing**: Only test shots affected by code changes
4. **Browser pool**: Reuse browser instances for faster capture
