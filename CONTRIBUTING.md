# Contributing to Pixelguard

Thank you for your interest in contributing to Pixelguard! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Node.js 18+
- Playwright (`npm install playwright`)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/pixelguard/pixelguard.git
cd pixelguard

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run the CLI
cargo run -- init
cargo run -- test
cargo run -- list
```

### Project Structure

```
pixelguard/
├── crates/
│   ├── pixelguard-cli/     # CLI binary
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/   # Command implementations
│   └── pixelguard-core/    # Core library
│       └── src/
│           ├── lib.rs
│           ├── config.rs   # Configuration handling
│           ├── detect.rs   # Project detection
│           ├── capture.rs  # Screenshot capture
│           ├── diff.rs     # Image comparison
│           └── report.rs   # HTML report generation
├── npm/                    # npm wrapper package
├── docs/                   # Documentation
└── examples/               # Example projects
```

## Code Style

### Rust Formatting

- Run `cargo fmt` before every commit
- Run `cargo clippy` and fix ALL warnings
- Use `thiserror` for custom error types
- Use `anyhow` for error propagation
- Prefer `?` operator over `.unwrap()`

### Import Ordering

Group imports with blank lines between:

```rust
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::Config;
```

### Documentation

Every public function needs a doc comment:

```rust
/// Detects the project type by checking for framework config files.
///
/// # Example
///
/// ```rust,no_run
/// let project = detect_project_type(".").await?;
/// ```
pub async fn detect_project_type<P: AsRef<Path>>(dir: P) -> Result<ProjectType> {
```

### Error Messages

Error messages must be actionable:

```rust
// Good
anyhow::bail!(
    "Could not read config file at '{}'. \
     Run 'pixelguard init' to create one.",
    config_path.display()
);

// Bad
anyhow::bail!("Failed to read config");
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p pixelguard-core

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests

- Unit tests go in the same file under `#[cfg(test)]`
- Use descriptive test names
- Test error cases, not just happy paths

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_returns_error_for_invalid_json() {
        let result = Config::load("invalid.json");
        assert!(result.is_err());
    }
}
```

## Commit Messages

Use conventional commits:

```
<type>(<scope>): <description>

[optional body]
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`

Examples:
- `feat(core): add Storybook detection`
- `fix(cli): handle missing config gracefully`
- `docs: update README with CI examples`

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Run `cargo test`
6. Commit with conventional commit messages
7. Push to your fork
8. Open a pull request

### PR Checklist

- [ ] Code follows the style guidelines
- [ ] Self-review completed
- [ ] Tests added/updated
- [ ] Documentation updated (if applicable)
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes

## Reporting Issues

When reporting issues, please include:

1. Pixelguard version (`pixelguard --version`)
2. Operating system and version
3. Node.js version (`node --version`)
4. Steps to reproduce
5. Expected vs actual behavior
6. Relevant configuration (sanitized)

## Questions?

Open an issue or start a discussion on GitHub.
