//! # `OpenCode` SDK for Rust
//!
//! A Rust client library for the [OpenCode](https://opencode.ai) API,
//! providing type-safe access to all endpoints.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use opencode_sdk_rs::Opencode;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), opencode_sdk_rs::OpencodeError> {
//!     // Uses OPENCODE_BASE_URL env var or defaults to localhost:54321
//!     let client = Opencode::new()?;
//!
//!     // Get app info
//!     let app = client.app().get(None).await?;
//!     println!("Connected to: {}", app.hostname);
//!
//!     // List sessions
//!     let sessions = client.session().list(None).await?;
//!     println!("Found {} sessions", sessions.len());
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod resources;
pub mod streaming;
pub mod types;

// Re-export key types at the crate root for convenience
pub use client::{Opencode, OpencodeBuilder, RequestOptions};
pub use config::ClientOptions;
pub use error::OpencodeError;
pub use streaming::SseStream;
