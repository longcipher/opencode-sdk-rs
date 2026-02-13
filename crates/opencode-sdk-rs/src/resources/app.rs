//! App resource types and methods mirroring the JS SDK's `resources/app.ts`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    client::{Opencode, RequestOptions},
    error::OpencodeError,
};

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

/// Top-level application information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct App {
    /// Whether the project is a git repository.
    pub git: bool,
    /// The hostname of the machine.
    pub hostname: String,
    /// Relevant filesystem paths.
    pub path: AppPath,
    /// Timing information.
    pub time: AppTime,
}

/// Filesystem paths used by the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppPath {
    /// Path to the configuration directory.
    pub config: String,
    /// Current working directory.
    pub cwd: String,
    /// Path to the data directory.
    pub data: String,
    /// Project root directory.
    pub root: String,
    /// Path to the state directory.
    pub state: String,
}

/// Timing metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppTime {
    /// Timestamp (epoch seconds) when the app was initialised.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialized: Option<f64>,
}

// ---------------------------------------------------------------------------
// Mode
// ---------------------------------------------------------------------------

/// An operational mode with associated tools and optional model override.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mode {
    /// Human-readable mode name.
    pub name: String,
    /// Map of tool names to their enabled state.
    pub tools: HashMap<String, bool>,
    /// Optional model override for this mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModeModel>,
    /// Optional system prompt override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Optional temperature override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}

/// Model reference used inside a [`Mode`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModeModel {
    /// The model identifier.
    #[serde(rename = "modelID")]
    pub model_id: String,
    /// The provider identifier.
    #[serde(rename = "providerID")]
    pub provider_id: String,
}

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

/// A language-model definition exposed by a provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Model {
    /// Unique model identifier.
    pub id: String,
    /// Whether the model supports file attachments.
    pub attachment: bool,
    /// Cost information per token.
    pub cost: ModelCost,
    /// Context and output token limits.
    pub limit: ModelLimit,
    /// Human-readable model name.
    pub name: String,
    /// Arbitrary provider-specific options.
    pub options: HashMap<String, serde_json::Value>,
    /// Whether the model supports chain-of-thought reasoning.
    pub reasoning: bool,
    /// ISO-8601 release date.
    pub release_date: String,
    /// Whether the model supports temperature tuning.
    pub temperature: bool,
    /// Whether the model supports tool calling.
    pub tool_call: bool,
}

/// Per-token cost information for a [`Model`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelCost {
    /// Cost per input token.
    pub input: f64,
    /// Cost per output token.
    pub output: f64,
    /// Cost per cache-read token (if supported).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<f64>,
    /// Cost per cache-write token (if supported).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<f64>,
}

/// Token limits for a [`Model`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelLimit {
    /// Maximum context window size in tokens.
    pub context: u64,
    /// Maximum output size in tokens.
    pub output: u64,
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

/// An LLM provider definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provider {
    /// Unique provider identifier.
    pub id: String,
    /// Environment variable names required for authentication.
    pub env: Vec<String>,
    /// Map of model identifiers to their definitions.
    pub models: HashMap<String, Model>,
    /// Human-readable provider name.
    pub name: String,
    /// Optional API base URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    /// Optional npm package name (JS SDK specific, preserved for compat).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

/// Type alias for `POST /app/init` response.
pub type AppInitResponse = bool;

/// Type alias for `POST /log` response.
pub type AppLogResponse = bool;

/// Type alias for `GET /mode` response.
pub type AppModesResponse = Vec<Mode>;

/// Response from `GET /config/providers`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppProvidersResponse {
    /// Map of provider ID to its default model ID.
    pub default: HashMap<String, String>,
    /// List of available providers.
    pub providers: Vec<Provider>,
}

/// Log level for [`AppLogParams`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug-level log message.
    Debug,
    /// Informational log message.
    Info,
    /// Error-level log message.
    Error,
    /// Warning-level log message.
    Warn,
}

/// Parameters for `POST /log`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppLogParams {
    /// Severity level.
    pub level: LogLevel,
    /// The log message body.
    pub message: String,
    /// Name of the originating service / component.
    pub service: String,
    /// Optional extra structured data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// AppResource
// ---------------------------------------------------------------------------

/// Provides access to the App-related API endpoints.
pub struct AppResource<'a> {
    client: &'a Opencode,
}

impl<'a> AppResource<'a> {
    /// Create a new `AppResource` bound to the given client.
    pub(crate) const fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    /// Retrieve application information (`GET /app`).
    pub async fn get(&self, options: Option<&RequestOptions>) -> Result<App, OpencodeError> {
        self.client.get("/app", options).await
    }

    /// Initialise the application (`POST /app/init`).
    pub async fn init(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<AppInitResponse, OpencodeError> {
        self.client.post::<bool, ()>("/app/init", None, options).await
    }

    /// Send a log entry (`POST /log`).
    pub async fn log(
        &self,
        params: &AppLogParams,
        options: Option<&RequestOptions>,
    ) -> Result<AppLogResponse, OpencodeError> {
        self.client.post("/log", Some(params), options).await
    }

    /// List available modes (`GET /mode`).
    pub async fn modes(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<AppModesResponse, OpencodeError> {
        self.client.get("/mode", options).await
    }

    /// List providers and their default models (`GET /config/providers`).
    pub async fn providers(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<AppProvidersResponse, OpencodeError> {
        self.client.get("/config/providers", options).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn app_round_trip() {
        let app = App {
            git: true,
            hostname: "dev-machine".into(),
            path: AppPath {
                config: "/home/user/.config/opencode".into(),
                cwd: "/home/user/project".into(),
                data: "/home/user/.local/share/opencode".into(),
                root: "/home/user/project".into(),
                state: "/home/user/.local/state/opencode".into(),
            },
            time: AppTime { initialized: Some(1_700_000_000.0) },
        };
        let json_str = serde_json::to_string(&app).unwrap();
        let back: App = serde_json::from_str(&json_str).unwrap();
        assert_eq!(app, back);
    }

    #[test]
    fn app_time_optional_initialized() {
        let app = App {
            git: false,
            hostname: "ci".into(),
            path: AppPath {
                config: "/tmp/cfg".into(),
                cwd: "/tmp".into(),
                data: "/tmp/data".into(),
                root: "/tmp".into(),
                state: "/tmp/state".into(),
            },
            time: AppTime { initialized: None },
        };
        let json_str = serde_json::to_string(&app).unwrap();
        // `initialized` should be absent from serialised output.
        assert!(!json_str.contains("initialized"));
        let back: App = serde_json::from_str(&json_str).unwrap();
        assert_eq!(app, back);
    }

    #[test]
    fn mode_full_round_trip() {
        let mode = Mode {
            name: "code".into(),
            tools: HashMap::from([("bash".into(), true), ("edit".into(), false)]),
            model: Some(ModeModel { model_id: "gpt-4o".into(), provider_id: "openai".into() }),
            prompt: Some("You are a coding assistant.".into()),
            temperature: Some(0.7),
        };
        let json_str = serde_json::to_string(&mode).unwrap();
        // Verify camelCase renames.
        assert!(json_str.contains("modelID"));
        assert!(json_str.contains("providerID"));
        let back: Mode = serde_json::from_str(&json_str).unwrap();
        assert_eq!(mode, back);
    }

    #[test]
    fn mode_minimal() {
        let mode = Mode {
            name: "default".into(),
            tools: HashMap::new(),
            model: None,
            prompt: None,
            temperature: None,
        };
        let json_str = serde_json::to_string(&mode).unwrap();
        assert!(!json_str.contains("model"));
        assert!(!json_str.contains("prompt"));
        assert!(!json_str.contains("temperature"));
        let back: Mode = serde_json::from_str(&json_str).unwrap();
        assert_eq!(mode, back);
    }

    #[test]
    fn model_round_trip() {
        let model = Model {
            id: "gpt-4o".into(),
            attachment: true,
            cost: ModelCost { input: 5.0, output: 15.0, cache_read: Some(2.5), cache_write: None },
            limit: ModelLimit { context: 128_000, output: 4_096 },
            name: "GPT-4o".into(),
            options: HashMap::from([("streaming".into(), json!(true))]),
            reasoning: false,
            release_date: "2024-05-13".into(),
            temperature: true,
            tool_call: true,
        };
        let json_str = serde_json::to_string(&model).unwrap();
        let back: Model = serde_json::from_str(&json_str).unwrap();
        assert_eq!(model, back);
    }

    #[test]
    fn model_cost_no_cache() {
        let cost = ModelCost { input: 1.0, output: 2.0, cache_read: None, cache_write: None };
        let json_str = serde_json::to_string(&cost).unwrap();
        assert!(!json_str.contains("cache_read"));
        assert!(!json_str.contains("cache_write"));
        let back: ModelCost = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cost, back);
    }

    #[test]
    fn provider_round_trip() {
        let provider = Provider {
            id: "openai".into(),
            env: vec!["OPENAI_API_KEY".into()],
            models: HashMap::from([(
                "gpt-4o".into(),
                Model {
                    id: "gpt-4o".into(),
                    attachment: true,
                    cost: ModelCost {
                        input: 5.0,
                        output: 15.0,
                        cache_read: None,
                        cache_write: None,
                    },
                    limit: ModelLimit { context: 128_000, output: 4_096 },
                    name: "GPT-4o".into(),
                    options: HashMap::new(),
                    reasoning: false,
                    release_date: "2024-05-13".into(),
                    temperature: true,
                    tool_call: true,
                },
            )]),
            name: "OpenAI".into(),
            api: Some("https://api.openai.com/v1".into()),
            npm: None,
        };
        let json_str = serde_json::to_string(&provider).unwrap();
        let back: Provider = serde_json::from_str(&json_str).unwrap();
        assert_eq!(provider, back);
    }

    #[test]
    fn app_log_params_with_extra() {
        let params = AppLogParams {
            level: LogLevel::Info,
            message: "server started".into(),
            service: "api-gateway".into(),
            extra: Some(HashMap::from([
                ("port".into(), json!(8080)),
                ("env".into(), json!("production")),
            ])),
        };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(json_str.contains(r#""level":"info"#));
        let back: AppLogParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    #[test]
    fn app_log_params_without_extra() {
        let params = AppLogParams {
            level: LogLevel::Error,
            message: "something broke".into(),
            service: "worker".into(),
            extra: None,
        };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(!json_str.contains("extra"));
        assert!(json_str.contains(r#""level":"error"#));
        let back: AppLogParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    #[test]
    fn log_level_variants() {
        for (variant, expected) in [
            (LogLevel::Debug, "debug"),
            (LogLevel::Info, "info"),
            (LogLevel::Error, "error"),
            (LogLevel::Warn, "warn"),
        ] {
            let json_str = serde_json::to_string(&variant).unwrap();
            assert_eq!(json_str, format!("\"{expected}\""));
            let back: LogLevel = serde_json::from_str(&json_str).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn app_providers_response_round_trip() {
        let resp = AppProvidersResponse {
            default: HashMap::from([
                ("openai".into(), "gpt-4o".into()),
                ("anthropic".into(), "claude-3-opus".into()),
            ]),
            providers: vec![Provider {
                id: "openai".into(),
                env: vec!["OPENAI_API_KEY".into()],
                models: HashMap::new(),
                name: "OpenAI".into(),
                api: None,
                npm: None,
            }],
        };
        let json_str = serde_json::to_string(&resp).unwrap();
        let back: AppProvidersResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(resp, back);
    }

    #[test]
    fn mode_model_serde_rename() {
        let m = ModeModel { model_id: "claude-3-opus".into(), provider_id: "anthropic".into() };
        let v: serde_json::Value = serde_json::to_value(&m).unwrap();
        assert_eq!(v["modelID"], "claude-3-opus");
        assert_eq!(v["providerID"], "anthropic");
        let back: ModeModel = serde_json::from_value(v).unwrap();
        assert_eq!(m, back);
    }

    // -- Edge cases --

    #[test]
    fn provider_no_api_no_npm() {
        let provider = Provider {
            id: "custom".into(),
            env: vec![],
            models: HashMap::new(),
            name: "Custom".into(),
            api: None,
            npm: None,
        };
        let json_str = serde_json::to_string(&provider).unwrap();
        assert!(!json_str.contains("api"));
        assert!(!json_str.contains("npm"));
        let back: Provider = serde_json::from_str(&json_str).unwrap();
        assert_eq!(provider, back);
    }

    #[test]
    fn model_cost_cache_read_only() {
        let cost = ModelCost { input: 3.0, output: 6.0, cache_read: Some(1.5), cache_write: None };
        let json_str = serde_json::to_string(&cost).unwrap();
        assert!(json_str.contains("cache_read"));
        assert!(!json_str.contains("cache_write"));
        let back: ModelCost = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cost, back);
    }

    #[test]
    fn model_cost_cache_write_only() {
        let cost = ModelCost { input: 3.0, output: 6.0, cache_read: None, cache_write: Some(4.0) };
        let json_str = serde_json::to_string(&cost).unwrap();
        assert!(!json_str.contains("cache_read"));
        assert!(json_str.contains("cache_write"));
        let back: ModelCost = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cost, back);
    }

    #[test]
    fn app_time_initialized_absent_from_json() {
        // Verify deserialization when the key is completely absent
        let raw = r#"{"git":true,"hostname":"h","path":{"config":"c","cwd":"w","data":"d","root":"r","state":"s"},"time":{}}"#;
        let app: App = serde_json::from_str(raw).unwrap();
        assert_eq!(app.time.initialized, None);
    }

    #[test]
    fn app_log_params_extra_empty_map() {
        let params = AppLogParams {
            level: LogLevel::Debug,
            message: "trace".into(),
            service: "svc".into(),
            extra: Some(HashMap::new()),
        };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(json_str.contains("extra"));
        let back: AppLogParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    #[test]
    fn mode_with_empty_tools_and_some_model() {
        let mode = Mode {
            name: "review".into(),
            tools: HashMap::new(),
            model: Some(ModeModel { model_id: "o1".into(), provider_id: "openai".into() }),
            prompt: None,
            temperature: None,
        };
        let json_str = serde_json::to_string(&mode).unwrap();
        assert!(!json_str.contains("prompt"));
        assert!(!json_str.contains("temperature"));
        assert!(json_str.contains("modelID"));
        let back: Mode = serde_json::from_str(&json_str).unwrap();
        assert_eq!(mode, back);
    }

    #[test]
    fn model_with_empty_options() {
        let model = Model {
            id: "small".into(),
            attachment: false,
            cost: ModelCost { input: 0.1, output: 0.2, cache_read: None, cache_write: None },
            limit: ModelLimit { context: 4096, output: 512 },
            name: "Small Model".into(),
            options: HashMap::new(),
            reasoning: false,
            release_date: "2025-01-01".into(),
            temperature: false,
            tool_call: false,
        };
        let json_str = serde_json::to_string(&model).unwrap();
        let back: Model = serde_json::from_str(&json_str).unwrap();
        assert_eq!(model, back);
    }
}
