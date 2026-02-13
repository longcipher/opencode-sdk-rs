# OpenCode SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/opencode-sdk-rs)](https://crates.io/crates/opencode-sdk-rs)
[![Documentation](https://docs.rs/opencode-sdk-rs/badge.svg)](https://docs.rs/opencode-sdk-rs)

A Rust client library for the [OpenCode](https://opencode.ai) API, providing type-safe access to all endpoints with automatic retries, SSE streaming, and structured error handling.

## Features

- **Full API Coverage** — All resources: App, Config, Event, File, Find, Session, Tui
- **Type-Safe** — Complete request/response types with serde serialization
- **Automatic Retries** — Exponential backoff with jitter, honoring `Retry-After` headers
- **SSE Streaming** — Real-time event consumption via async streams
- **Structured Errors** — Typed error hierarchy with retry logic hints
- **Configurable** — Builder pattern with environment variable support
- **Async/Await** — Built on `tokio` and `hpx` for high-performance HTTP

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
opencode-sdk-rs = "0.1.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use opencode_sdk_rs::{Opencode, OpencodeError};

#[tokio::main]
async fn main() -> Result<(), OpencodeError> {
    // Create client from environment or defaults
    let client = Opencode::new()?;

    // Get app information
    let app = client.app().get(None).await?;
    println!("Connected to: {}", app.hostname);

    // List sessions
    let sessions = client.session().list(None).await?;
    println!("Found {} sessions", sessions.len());

    Ok(())
}
```

## Configuration

The client can be configured via environment variables or programmatically:

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENCODE_BASE_URL` | OpenCode server URL | `http://localhost:54321` |

### Builder Pattern

```rust
use opencode_sdk_rs::Opencode;
use std::time::Duration;

let client = Opencode::builder()
    .base_url("http://my-server:54321")
    .timeout(Duration::from_secs(30))
    .max_retries(3)
    .build()?;
```

## Resources

### App

```rust
// Get app info
let app = client.app().get(None).await?;

// List available modes
let modes = client.app().modes(None).await?;

// List providers
let providers = client.app().providers(None).await?;
```

### Session

```rust
use opencode_sdk_rs::resources::SessionChatParams;

// Create a new session
let session = client.session().create().await?;

// Send a message
let params = SessionChatParams {
    message: "Hello, OpenCode!".to_string(),
    ..Default::default()
};
let response = client.session().chat(&session.id, params).await?;

// List all sessions
let sessions = client.session().list(None).await?;

// Delete a session
client.session().delete(&session.id).await?;
```

### File

```rust
// Read a file
let content = client.file().read(&file_path).await?;

// Get file status
let files = client.file().status().await?;
for file in &files {
    println!("{}: {:?}", file.path, file.status);
}
```

### Find

```rust
use opencode_sdk_rs::resources::{FindFilesParams, FindSymbolsParams};

// Search for files
let file_results = client
    .find()
    .files(FindFilesParams {
        query: "*.rs".to_string(),
        ..Default::default()
    })
    .await?;

// Search for symbols
let symbol_results = client
    .find()
    .symbols(FindSymbolsParams {
        query: "my_function".to_string(),
        ..Default::default()
    })
    .await?;
```

### Config

```rust
let config = client.config().get(None).await?;
if let Some(theme) = &config.theme {
    println!("Current theme: {}", theme);
}
```

### Event (SSE Streaming)

```rust
use futures_core::Stream;

// Subscribe to server-sent events
let mut stream = client.event().list().await?;

while let Some(event) = stream.next().await {
    match event? {
        EventListResponse::Message(msg) => println!("Message: {:?}", msg),
        EventListResponse::Status(status) => println!("Status: {:?}", status),
        // Handle other event types...
    }
}
```

## Error Handling

The SDK provides a typed error hierarchy:

```rust
use opencode_sdk_rs::OpencodeError;

match client.session().get("invalid-id").await {
    Err(OpencodeError::Api { status: 404, .. }) => {
        eprintln!("Session not found");
    }
    Err(OpencodeError::Timeout) => {
        eprintln!("Request timed out");
    }
    Err(e) if e.is_retryable() => {
        eprintln!("Transient error: {}", e);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
    Ok(session) => {
        println!("Found session: {}", session.id);
    }
}
```

### Error Types

- `OpencodeError::Api` — HTTP error responses (400, 404, 429, 500, etc.)
- `OpencodeError::Connection` — Network/connection failures
- `OpencodeError::Timeout` — Request timeout
- `OpencodeError::Serialization` — JSON parsing errors
- `OpencodeError::UserAbort` — User-initiated cancellation

## Retry Behavior

The client automatically retries on:

- **HTTP Status**: 408, 409, 429, 5xx
- **Network Errors**: Connection failures, timeouts
- **Respects**: `x-should-retry`, `retry-after`, `retry-after-ms` headers

Retry delays use exponential backoff with jitter (0.5s to 8s capped).

## Examples

See the [examples/](crates/opencode-sdk-rs/examples/) directory:

```bash
# Run basic example
cargo run -p opencode-sdk-rs --example basic
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --all-features --workspace
```

### Linting

```bash
# Format code
cargo +nightly fmt --all

# Run clippy
cargo +nightly clippy --all -- -D warnings
```

### Full CI Check

```bash
just ci
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Links

- [OpenCode](https://opencode.ai)
- [Documentation](https://docs.rs/opencode-sdk-rs)
- [Crates.io](https://crates.io/crates/opencode-sdk-rs)
