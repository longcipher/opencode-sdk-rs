//! Basic usage example for the OpenCode SDK.
//!
//! Run with:
//! ```sh
//! cargo run -p opencode-sdk-rs --example basic

use opencode_sdk_rs::{Opencode, OpencodeError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), OpencodeError> {
    // Create client with defaults
    let client = Opencode::new()?;
    println!("Base URL: {}", client.base_url());

    // Get app info
    let app = client.app().get(None).await?;
    println!("Hostname: {}", app.hostname);
    println!("Git: {}", app.git);
    println!("Root: {}", app.path.root);

    // List modes
    let modes = client.app().modes(None).await?;
    for mode in &modes {
        println!("Mode: {}", mode.name);
    }

    // List sessions
    let sessions = client.session().list(None).await?;
    println!("\nSessions ({}):", sessions.len());
    for session in &sessions {
        println!("  {} - {}", session.id, session.title);
    }

    // Get config
    let config = client.config().get(None).await?;
    if let Some(theme) = &config.theme {
        println!("\nTheme: {theme}");
    }

    // Get file status
    let files = client.file().status().await?;
    println!("\nModified files ({}):", files.len());
    for f in &files {
        println!("  {} ({:?})", f.path, f.status);
    }

    Ok(())
}
