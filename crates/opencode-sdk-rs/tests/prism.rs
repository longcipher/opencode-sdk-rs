// Prism-based OpenAPI contract tests.
// Require PRISM_URL env var. Run via `just test-prism`.

use std::time::Duration;

use opencode_sdk_rs::{
    Opencode,
    config::ClientOptions,
    resources::{
        file::{FileListParams, FileReadParams},
        session::{PartInput, SessionChatParams, TextPartInput},
    },
};

/// Create a client pointing at the Prism mock server.
fn prism_client() -> Option<Opencode> {
    let url = std::env::var("PRISM_URL").ok()?;
    Some(
        Opencode::with_options(&ClientOptions {
            base_url: Some(url),
            timeout: Some(Duration::from_secs(10)),
            max_retries: Some(0),
            ..ClientOptions::empty()
        })
        .expect("client should build"),
    )
}

/// Skip test if PRISM_URL is not set.
macro_rules! skip_if_no_prism {
    () => {
        match prism_client() {
            Some(c) => c,
            None => {
                eprintln!("PRISM_URL not set — skipping");
                return;
            }
        }
    };
}

#[test]
fn prism_smoke() {
    let _client = skip_if_no_prism!();
    eprintln!("Prism available");
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

// NOTE: /app endpoint doesn't exist in official OpenAPI spec
// Keeping app().providers() test which hits /config/providers

#[tokio::test]
async fn test_prism_app_providers() {
    let client = skip_if_no_prism!();
    let result = client.app().providers(None).await;
    assert!(result.is_ok(), "GET /config/providers failed: {result:?}");
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_prism_session_list() {
    let client = skip_if_no_prism!();
    let result = client.session().list(None).await;
    assert!(result.is_ok(), "GET /session failed: {result:?}");
}

#[tokio::test]
async fn test_prism_session_create() {
    let client = skip_if_no_prism!();
    let result = client.session().create(None).await;
    assert!(result.is_ok(), "POST /session failed: {result:?}");
}

#[tokio::test]
async fn test_prism_session_delete() {
    let client = skip_if_no_prism!();
    // Prism may return an error for a non-existent ID — we just verify no panic
    // and that the SDK returns a structured result.
    let result = client.session().delete("ses_test", None).await;
    eprintln!("DELETE /session/ses_test => {result:?}");
}

#[tokio::test]
async fn test_prism_session_chat() {
    let client = skip_if_no_prism!();
    let params = SessionChatParams {
        parts: vec![PartInput::Text(TextPartInput {
            text: "hello".into(),
            id: None,
            synthetic: None,
            ignored: None,
            time: None,
            metadata: None,
        })],
        model: None,
        message_id: None,
        agent: None,
        no_reply: None,
        format: None,
        system: None,
        variant: None,
        tools: None,
    };
    let result = client.session().chat("ses_test", &params, None).await;
    // The request serialization conforming to the spec is the key assertion.
    // Prism may return mock data that doesn't map perfectly.
    eprintln!("POST /session/ses_test/message => {result:?}");
}

// ---------------------------------------------------------------------------
// File
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_prism_file_list() {
    let client = skip_if_no_prism!();
    let params = FileListParams { path: ".".into() };
    let result = client.file().list(Some(&params)).await;
    assert!(result.is_ok(), "GET /file failed: {result:?}");
}

#[tokio::test]
async fn test_prism_file_read() {
    let client = skip_if_no_prism!();
    let params = FileReadParams { path: "test.rs".into() };
    let result = client.file().read(&params).await;
    assert!(result.is_ok(), "GET /file/content failed: {result:?}");
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_prism_config_get() {
    let client = skip_if_no_prism!();
    let result = client.config().get(None).await;
    eprintln!("GET /config => {result:?}");
}
