use opencode_sdk_rs::{
    Opencode,
    config::ClientOptions,
    resources::file::{FileListParams, FileReadParams},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

/// Helper: create a client pointing at the mock server with no retries.
fn client_for(server: &MockServer) -> Opencode {
    Opencode::with_options(&ClientOptions {
        base_url: Some(server.uri()),
        timeout: Some(std::time::Duration::from_secs(5)),
        max_retries: Some(0),
        ..ClientOptions::empty()
    })
    .expect("client should build")
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_app_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "git": true,
            "hostname": "test-host",
            "path": {
                "config": "/cfg",
                "cwd": "/cwd",
                "data": "/data",
                "root": "/root",
                "state": "/state"
            },
            "time": { "initialized": 1234 }
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let app = client.app().get(None).await.unwrap();
    assert!(app.git);
    assert_eq!(app.hostname, "test-host");
}

#[tokio::test]
async fn test_app_modes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/mode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "name": "code", "tools": {} }
        ])))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let modes = client.app().modes(None).await.unwrap();
    assert_eq!(modes.len(), 1);
    assert_eq!(modes[0].name, "code");
}

#[tokio::test]
async fn test_app_providers() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/config/providers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "default": { "mode": "anthropic/claude-sonnet" },
            "providers": [{
                "id": "anthropic",
                "name": "Anthropic",
                "source": "env",
                "env": ["ANTHROPIC_API_KEY"],
                "options": {},
                "models": {}
            }]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let prov = client.app().providers(None).await.unwrap();
    assert_eq!(prov.providers.len(), 1);
    assert_eq!(prov.providers[0].id, "anthropic");
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_session_create() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/session"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "sess-1",
            "slug": "test-session",
            "projectID": "proj-1",
            "directory": "/tmp/test",
            "time": { "created": 100.0, "updated": 200.0 },
            "title": "Test Session",
            "version": "1.0"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let session = client.session().create(None).await.unwrap();
    assert_eq!(session.id, "sess-1");
    assert_eq!(session.title, "Test Session");
}

#[tokio::test]
async fn test_session_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/session"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": "s1", "slug": "s1", "projectID": "p1", "directory": "/d1", "time": { "created": 1.0, "updated": 2.0 }, "title": "S1", "version": "1" },
            { "id": "s2", "slug": "s2", "projectID": "p2", "directory": "/d2", "time": { "created": 3.0, "updated": 4.0 }, "title": "S2", "version": "1" }
        ])))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let sessions = client.session().list(None).await.unwrap();
    assert_eq!(sessions.len(), 2);
}

#[tokio::test]
async fn test_session_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/session/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(true))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let result = client.session().delete("abc", None).await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_session_chat_accepts_missing_assistant_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/session/sess-1/message"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "info": {
                "role": "assistant",
                "sessionID": "sess-1",
                "parentID": "",
                "agent": "",
                "time": { "created": 123.0 },
                "tokens": { "cache": { "read": 0, "write": 0 }, "input": 0, "output": 0, "reasoning": 0, "total": 0 }
            },
            "parts": []
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let params = opencode_sdk_rs::resources::session::SessionChatParams {
        parts: vec![],
        model: Some(opencode_sdk_rs::resources::session::SessionChatModel {
            provider_id: "openai".to_owned(),
            model_id: "gpt-4o".to_owned(),
        }),
        message_id: None,
        agent: None,
        no_reply: None,
        format: None,
        system: None,
        variant: None,
        tools: None,
    };

    let resp = client.session().chat("sess-1", &params, None).await.unwrap();
    match resp.info {
        opencode_sdk_rs::resources::session::Message::Assistant(msg) => {
            assert_eq!(msg.id, "");
            assert_eq!(msg.cost, 0.0);
            assert_eq!(msg.mode, "");
        }
        _ => panic!("expected assistant message"),
    }
}

#[tokio::test]
async fn test_session_chat_accepts_unknown_part_variant() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/session/sess-1/message"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "info": {
                "role": "assistant",
                "id": "msg-1",
                "cost": 0.0,
                "mode": "code",
                "modelID": "gpt-4o",
                "providerID": "openai",
                "sessionID": "sess-1",
                "parentID": "",
                "agent": "",
                "path": { "cwd": ".", "root": "." },
                "system": [],
                "time": { "created": 123.0 },
                "tokens": { "cache": { "read": 0, "write": 0 }, "input": 0, "output": 0, "reasoning": 0, "total": 0 }
            },
            "parts": [{ "type": "reasoning", "id": "p-1", "sessionID": "sess-1", "messageID": "msg-1", "text": "thinking...", "time": { "start": 1700000000.0 } }]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let params = opencode_sdk_rs::resources::session::SessionChatParams {
        parts: vec![],
        model: Some(opencode_sdk_rs::resources::session::SessionChatModel {
            provider_id: "openai".to_owned(),
            model_id: "gpt-4o".to_owned(),
        }),
        message_id: None,
        agent: None,
        no_reply: None,
        format: None,
        system: None,
        variant: None,
        tools: None,
    };

    let resp = client.session().chat("sess-1", &params, None).await.unwrap();
    assert_eq!(resp.parts.len(), 1);
    match &resp.parts[0] {
        opencode_sdk_rs::resources::session::Part::Reasoning(r) => {
            assert_eq!(r.id, "p-1");
            assert_eq!(r.text, "thinking...");
            assert_eq!(r.session_id, "sess-1");
            assert_eq!(r.message_id, "msg-1");
        }
        other => panic!("expected Part::Reasoning, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// File
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_file_read() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/file/content"))
        .and(query_param("path", "src/main.rs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": "fn main() {}",
            "type": "text"
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp =
        client.file().read(&FileReadParams { path: "src/main.rs".to_owned() }).await.unwrap();
    assert_eq!(resp.content, "fn main() {}");
}

#[tokio::test]
async fn test_file_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/file"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "name": "main.rs", "path": "src/main.rs", "absolute": "/home/user/project/src/main.rs", "type": "file", "ignored": false },
            { "name": "target", "path": "target", "absolute": "/home/user/project/target", "type": "directory", "ignored": true }
        ])))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let params = FileListParams { path: ".".into() };
    let files = client.file().list(Some(&params)).await.unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].name, "main.rs");
    assert!(!files[0].ignored);
    assert!(files[1].ignored);
}

#[tokio::test]
async fn test_file_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/file/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "added": 10, "path": "src/lib.rs", "removed": 2, "status": "modified" }
        ])))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let files = client.file().status().await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "src/lib.rs");
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_config_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "theme": "dark",
            "autoupdate": true
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let config = client.config().get(None).await.unwrap();
    assert_eq!(config.theme.as_deref(), Some("dark"));
    assert_eq!(config.autoupdate, Some(serde_json::Value::Bool(true)));
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_error_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/app"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let err = client.app().get(None).await.unwrap_err();
    assert_eq!(err.status(), Some(404));
}

#[tokio::test]
async fn test_error_500() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/app"))
        .respond_with(ResponseTemplate::new(500).set_body_string("server error"))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let err = client.app().get(None).await.unwrap_err();
    assert_eq!(err.status(), Some(500));
    assert!(err.is_retryable());
}

// ---------------------------------------------------------------------------
// Retry behaviour
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_retry_on_429() {
    let server = MockServer::start().await;

    // First request returns 429, second returns 200.
    Mock::given(method("GET"))
        .and(path("/app"))
        .respond_with(ResponseTemplate::new(429).set_body_string("rate limited"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "git": false,
            "hostname": "retry-host",
            "path": { "config": "/c", "cwd": "/w", "data": "/d", "root": "/r", "state": "/s" },
            "time": {}
        })))
        .mount(&server)
        .await;

    let client = Opencode::with_options(&ClientOptions {
        base_url: Some(server.uri()),
        timeout: Some(std::time::Duration::from_secs(10)),
        max_retries: Some(2),
        ..ClientOptions::empty()
    })
    .unwrap();

    let app = client.app().get(None).await.unwrap();
    assert_eq!(app.hostname, "retry-host");
}

// ---------------------------------------------------------------------------
// Timeout
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/app"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "git": true,
                    "hostname": "h",
                    "path": { "config": "", "cwd": "", "data": "", "root": "", "state": "" },
                    "time": {}
                }))
                .set_delay(std::time::Duration::from_secs(5)),
        )
        .mount(&server)
        .await;

    let client = Opencode::with_options(&ClientOptions {
        base_url: Some(server.uri()),
        timeout: Some(std::time::Duration::from_millis(100)),
        max_retries: Some(0),
        ..ClientOptions::empty()
    })
    .unwrap();

    let err = client.app().get(None).await.unwrap_err();
    assert!(err.is_timeout());
}
