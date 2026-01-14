# Pixelguard — Claude Code Build Prompt

## Project Overview

Build **Pixelguard**, an open-source visual regression testing CLI tool.

Key principles:
- **Zero config by default** — auto-detect Storybook, Next.js, Vite
- **Git-friendly storage** — screenshots live in `.pixelguard/`, committed to repo
- **DX obsessed** — if setup takes more than 2 minutes, we failed
- **No backend** — everything runs locally or in CI

---

## Tech Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| CLI | Rust | Fast, zero runtime deps, cross-platform binaries |
| Screenshots | Playwright (via Node.js subprocess) | Industry standard, stable API |
| Diffing | `image` crate + custom pixelmatch-style algorithm | No external deps, fast |
| Storage | Local filesystem (`.pixelguard/`) | Git-friendly, no cloud required |
| Report | Static HTML | No server needed, works offline |
| Distribution | npm wrapper with prebuilt binaries | `npx pixelguard` just works |

---

## Repository Structure

```
pixelguard/
├── .github/
│   └── workflows/
│       ├── ci.yml                    # Project CI (lint, test, build)
│       └── visual-regression.yml     # Template for users to copy
├── crates/
│   ├── pixelguard-cli/               # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   │           ├── mod.rs
│   │           ├── init.rs
│   │           ├── test.rs
│   │           └── list.rs
│   └── pixelguard-core/              # Core library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── config.rs
│           ├── detect.rs
│           ├── capture.rs
│           ├── diff.rs
│           └── report.rs
├── npm/
│   └── pixelguard/                   # npm wrapper package
│       ├── package.json
│       ├── README.md
│       └── scripts/
│           ├── postinstall.js        # Downloads correct binary
│           └── run.js                # Executes binary
├── docs/
│   ├── getting-started.md
│   ├── configuration.md
│   ├── ci-setup.md
│   └── troubleshooting.md
├── examples/
│   └── storybook-react/              # Example project for testing
├── Cargo.toml                        # Workspace root
├── README.md
├── CONTRIBUTING.md
├── LICENSE                           # MIT
└── .gitignore
```

---

## CLI Commands Spec

### `pixelguard init`

Auto-detect project type and create config.

```bash
$ npx pixelguard init

Detecting project type...
✓ Found Storybook at http://localhost:6006
✓ Discovered 47 stories
✓ Created pixelguard.config.json

Next steps:
  1. Run: npx pixelguard test
  2. Commit .pixelguard/ as your baseline
```

**Flags:**
- `--force` — Re-initialize even if config exists

---

### `pixelguard test`

Capture screenshots and compare against baseline.

```bash
$ npx pixelguard test

Capturing 47 screenshots...
Comparing against baseline...

✓ 45 unchanged
✗ 2 changed

View report: .pixelguard/report.html
```

**Flags:**
- `--update` — Update baseline with current screenshots
- `--ci` — CI mode (machine-readable output, exit code 1 on diffs)
- `--filter <pattern>` — Only test shots matching pattern
- `--verbose` — Show detailed progress
- `--serve` — Serve the HTML report after completion
- `--port <number>` — Port for serving the report (default: 3333)

---

### `pixelguard list`

List all configured shots without capturing.

```bash
$ npx pixelguard list

Configured shots (47):

  button--primary → /iframe.html?id=button--primary
  button--secondary → /iframe.html?id=button--secondary
  card--default → /iframe.html?id=card--default
  ...
```

---

## Configuration Schema

```json
{
  "source": "storybook",
  "baseUrl": "http://localhost:6006",
  "include": ["**/*"],
  "exclude": ["**/*Deprecated*"],
  "viewport": {
    "width": 1280,
    "height": 720
  },
  "threshold": 0.1,
  "outputDir": ".pixelguard",
  "shots": [
    {
      "name": "button--primary",
      "path": "/iframe.html?id=button--primary",
      "waitFor": "#storybook-root",
      "delay": 100
    }
  ]
}
```

All fields optional — sensible defaults for everything.

---

## Core Module Specs

### `detect.rs`

Detect project type by checking:

1. **Storybook**: `.storybook/` exists → try ports 6006, 6007, 6008 → fetch `/index.json` or `/stories.json`
2. **Next.js**: `next.config.{js,mjs,ts}` exists → try ports 3000, 3001 → scan `app/` and `pages/` for routes
3. **Vite**: `vite.config.{js,ts,mjs}` exists → try ports 5173, 5174, 3000
4. **Unknown**: Return minimal config, let user define shots manually

### `capture.rs`

Generate a temporary Node.js script that uses Playwright to:

1. Launch headless Chromium
2. Navigate to each shot URL
3. Wait for selector (if specified)
4. Wait for delay (if specified)
5. Take screenshot
6. Save to output directory

Playwright must be installed in the user's project or globally.

### `diff.rs`

Compare two images pixel-by-pixel:

1. Load baseline and current images
2. Handle size mismatch (= 100% different)
3. Compare each pixel with tolerance (anti-aliasing)
4. Generate diff image (red overlay on differences, dimmed original for context)
5. Return diff percentage

### `report.rs`

Generate static HTML report:

1. Summary (unchanged, changed, added, removed counts)
2. Changed section with side-by-side images (baseline, current, diff)
3. Added section with current image only
4. Removed section with baseline image only
5. Dark/light theme with toggle, clean UI
6. Image zoom modal for detailed inspection
7. Comparison slider for visual diffs

---

## Code Quality Requirements

### Rust Style

- Run `cargo fmt` before every commit — non-negotiable
- Run `cargo clippy` and fix ALL warnings before committing
- Use `thiserror` for custom error types
- Use `anyhow` for error propagation in application code
- Prefer `?` operator over `.unwrap()` — unwrap only in tests or when logically impossible to fail
- Group imports with blank lines between groups:
  1. `std` imports
  2. External crate imports
  3. Internal module imports (`crate::` and `super::`)

### Documentation

- **Every** public function gets a `///` doc comment
- **Every** module gets a `//!` module-level doc comment at the top
- Doc comments explain **why** and **when to use**, not just what
- Include code examples in doc comments for non-trivial functions:

```rust
/// Detects the project type by checking for framework config files
/// and probing common dev server ports.
///
/// # Example
///
/// ```rust
/// let project = detect_project_type().await?;
/// match project {
///     ProjectType::Storybook { base_url, stories } => { /* ... */ }
///     ProjectType::Unknown => { /* ... */ }
/// }
/// ```
pub async fn detect_project_type() -> Result<ProjectType> {
```

### Formatting & Whitespace

- One blank line between functions
- One blank line between logical sections within long functions
- Two blank lines between `impl` blocks
- Line length: 100 chars max
- Trailing commas in multi-line constructs (structs, enums, function args)
- No trailing whitespace
- Files end with a single newline
- No commented-out code — delete it, git has history

### Naming Conventions

- `snake_case`: functions, variables, modules, file names
- `PascalCase`: types, traits, enum variants
- `SCREAMING_SNAKE_CASE`: constants and statics
- Descriptive names over abbreviations:
  - Good: `screenshot_directory`, `baseline_path`
  - Bad: `ss_dir`, `bl_p`
  - Acceptable abbreviations: `config`, `dir`, `url`, `html`, `json`

### Error Messages

User-facing errors must be **actionable**. Tell users what to do:

```rust
// Bad
anyhow::bail!("Failed to read config");

// Good
anyhow::bail!(
    "Could not read config file at '{}'. \
     Run 'pixelguard init' to create one.",
    config_path.display()
);
```

Include context: file paths, URLs, expected vs actual values.

### Testing

- Unit tests go in the same file under `#[cfg(test)]` module
- Integration tests go in `tests/` directory
- Test error cases, not just happy paths
- Use descriptive test names:

```rust
#[test]
fn parse_config_returns_error_for_invalid_json() { }

#[test]
fn diff_detects_single_pixel_change() { }
```

---

## Commit Guidelines

### Commit Discipline

- Commit after each logical unit of work
- Every commit should compile and pass tests
- Atomic commits — one concern per commit

### Commit Message Format

Use conventional commits style:

```
<type>(<scope>): <description>

[optional body]
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`

Examples:
- `feat(core): add Storybook detection with index.json parsing`
- `fix(cli): handle missing config file gracefully`
- `docs: add CI setup guide`
- `refactor(diff): extract pixel comparison to separate function`

### What NOT to Do

- Do **NOT** add `Co-authored-by: Claude` or any AI attribution
- Do **NOT** add `Generated by` comments in code
- Do **NOT** squash everything into one giant commit
- Do **NOT** commit with message like `WIP`, `fix`, `stuff`

---

## Definition of Done

The project is complete when ALL of these pass:

```bash
cargo build --release    # Succeeds with no warnings
cargo test               # All tests pass
cargo clippy             # Zero warnings
cargo fmt --check        # No formatting issues
```

And functionally:
- `npx pixelguard init` in a Storybook project creates valid config
- `npx pixelguard test` captures screenshots, compares, generates report
- `npx pixelguard test --update` updates baseline
- `npx pixelguard list` shows configured shots
- HTML report opens in browser and displays diffs correctly

---

## Technical Notes

- Rust edition: `2021` (not 2024 — doesn't exist yet)
- Storybook 7+ uses `/index.json`, older versions use `/stories.json` — support both
- Playwright Chromium download can be slow — consider showing progress or helpful message
- The `.pixelguard/` directory should be committed by users — document `.gitattributes` for LFS
- For CI mode, consider adding `--json` flag for machine-readable output
