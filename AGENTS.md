# AGENTS.md - OpenCode SDK for Rust

Guidelines for agentic coding agents working in this repository.

## Build / Lint / Test Commands

```bash
# Format all code (Rust, TOML, Markdown)
just format

# Run all lints (typos, markdown, clippy, unused deps check)
just lint

# Run all tests
just test

# Run a single test by name
just test -- <test_name>

# Run tests with coverage
just test-coverage

# Check compilation without building
just check

# Full CI check (lint + test + build)
just ci

# Clean build artifacts
just clean
```

### Running Single Tests

```bash
# Run specific test
just test -- test_app_get

# Run all tests in a module
just test -- error::tests

# Run with output visible
just test -- --nocapture test_app_get
```

## Project Structure

```text
├── Cargo.toml                 # Workspace manifest
├── Justfile                   # Task runner commands
├── crates/
│   └── opencode-sdk-rs/       # Main SDK crate
│       ├── src/
│       │   ├── lib.rs         # Crate root with re-exports
│       │   ├── client.rs      # Main Opencode client
│       │   ├── config.rs      # Client configuration
│       │   ├── error.rs       # Error types (thiserror)
│       │   ├── types.rs       # Shared types
│       │   ├── streaming.rs   # SSE streaming
│       │   └── resources/     # API resource modules
│       ├── tests/
│       │   └── integration.rs # Integration tests (wiremock)
│       └── examples/
│           └── basic.rs       # Usage examples
├── specs/                     # Design specifications
└── target/                    # Build output
```

## Code Style Guidelines

### Formatting

- **Rust**: Uses nightly rustfmt with `.rustfmt.toml`:
  - Imports: grouped as `StdExternalCrate`, granularity = `Crate`
  - Comment width: 100 chars
  - Trailing commas: vertical style
  - Field init shorthand: enabled

- **TOML**: Formatted with `taplo` (see `.taplo.toml`)

- **Markdown**: Linted with `rumdl` (see `.rumdl.toml`)
  - Line length: 200 chars
  - HTML allowed (MD033 disabled)

### Imports

```rust
// Group order: std -> external -> crate
use std::{collections::HashMap, time::Duration};

use http::{HeaderMap, header::HeaderValue};
use serde::{Serialize, de::DeserializeOwned};

use crate::{config::ClientOptions, error::OpencodeError};
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `OpencodeError`, `FileReadParams`)
- **Functions/Methods**: `snake_case` (e.g., `get_client`, `is_retryable`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `VERSION`)
- **Modules**: `snake_case` (e.g., `resources/file.rs`)
- **Error variants**: `PascalCase` describing the error (e.g., `Timeout`, `Connection`)

### Error Handling

- **Library code**: Use `thiserror` (NEVER `anyhow`)
- **Error enum**: Flat structure with descriptive variants
- **Constructors**: Provide convenience methods for common HTTP status codes
- **Source chains**: Use `#[source]` for error wrapping

```rust
#[derive(Debug, thiserror::Error)]
pub enum OpencodeError {
    #[error("{status} {message}")]
    Api { status: u16, headers: Option<Box<HeaderMap>>, body: Option<Box<Value>>, message: String },

    #[error("Connection error: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
```

### Types & Serialization

- Use `serde` for JSON serialization with `derive` feature
- Prefer strongly-typed structs over `serde_json::Value`
- Use `impl Into<String>` for string parameters
- Document public types with `///`

### Async Patterns

- Use `tokio` runtime with `macros` and `rt` features
- Prefer `async fn` for I/O operations
- Use `hpx` (not `reqwest`) for HTTP client

### Testing

- **Unit tests**: Inline in source files under `#[cfg(test)] mod tests`
- **Integration tests**: In `tests/` directory using `wiremock`
- Test naming: descriptive `snake_case` (e.g., `test_retry_on_429`)
- Use `#[tokio::test]` for async tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_retryable_returns_true_for_timeouts() {
        assert!(OpencodeError::Timeout.is_retryable());
    }
}
```

## Dependency Management

### Adding Dependencies

**NEVER** manually edit versions in `Cargo.toml`. Use `cargo add`:

```bash
# Add to workspace dependencies
cargo add <crate> --workspace

# Add to specific crate
cargo add <crate> -p opencode-sdk-rs --workspace
```

### Preferred Libraries

- **HTTP**: `hpx` (with `rustls-tls`, `json`, `stream` features) > `reqwest`
- **Concurrency**: `scc` > `dashmap`, `arc-swap` > `RwLock` for config
- **Parsing**: `winnow` or `pest` > manual parsing
- **Observability**: `tracing` (NEVER `log`), OpenTelemetry OTLP (NOT Prometheus)
- **Errors**: `thiserror` (NEVER `anyhow`)

## Clippy Lints

Configured in root `Cargo.toml` under `[workspace.lints.clippy]`:

- `all`, `pedantic`, `nursery`: warn
- `unwrap_used`, `expect_used`, `panic`: warn
- `cast_possible_truncation`, `module_name_repetitions`: allow
- `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`: allow

## Conventional Commits

Uses `git-cliff` for changelog generation (see `cliff.toml`):

- `feat:` - New features
- `fix:` - Bug fixes
- `doc:` - Documentation
- `perf:` - Performance improvements
- `refactor:` - Code refactoring
- `test:` - Tests
- `chore:` - Miscellaneous

## CI Checklist (Before Submitting)

Run and ensure all pass:

```bash
just format
just lint
just test
just build
```

## Frontend (Leptos CSR)

When working on the frontend:

```bash
# Dev server with hot reload
just fe-dev

# Build for release
just fe-build

# Serve built app
just fe-serve
```

**MUST** use `cargo-leptos` (NEVER `trunk`).

## Language Requirements

- **English ONLY** for all documentation, comments, and commit messages
- No Chinese characters allowed (check with `just check-cn`)

## Safety & Performance

- No `unsafe` unless strictly required and documented
- Leverage type system to eliminate bugs at compile time
- Consider CPU cache and memory allocation patterns
- Use lock-free data structures where possible (`scc`, `arc-swap`)
