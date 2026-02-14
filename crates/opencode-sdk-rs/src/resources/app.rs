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

/// Media capabilities (input or output).
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModelMediaCapabilities {
    /// Whether text is supported.
    pub text: bool,
    /// Whether audio is supported.
    pub audio: bool,
    /// Whether images are supported.
    pub image: bool,
    /// Whether video is supported.
    pub video: bool,
    /// Whether PDF is supported.
    pub pdf: bool,
}

/// Model capabilities.
#[allow(clippy::struct_excessive_bools)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelCapabilities {
    /// Whether the model supports temperature tuning.
    pub temperature: bool,
    /// Whether the model supports chain-of-thought reasoning.
    pub reasoning: bool,
    /// Whether the model supports file attachments.
    pub attachment: bool,
    /// Whether the model supports tool calling.
    pub toolcall: bool,
    /// Supported input media types.
    #[serde(default)]
    pub input: ModelMediaCapabilities,
    /// Supported output media types.
    #[serde(default)]
    pub output: ModelMediaCapabilities,
    /// Can be a bool or an object with a `field` key.
    #[serde(default)]
    pub interleaved: serde_json::Value,
}

/// API endpoint information for a model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModelApi {
    /// API identifier.
    pub id: String,
    /// API endpoint URL.
    pub url: String,
    /// npm package name.
    pub npm: String,
}

/// Model lifecycle status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    /// Alpha stage model.
    Alpha,
    /// Beta stage model.
    Beta,
    /// Deprecated model.
    Deprecated,
    /// Active / generally-available model.
    Active,
}

/// Cache cost information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CostCache {
    /// Cost per cache-read token.
    pub read: f64,
    /// Cost per cache-write token.
    pub write: f64,
}

/// Experimental cost tier for over 200K context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CostExperimentalOver200K {
    /// Cost per input token.
    pub input: f64,
    /// Cost per output token.
    pub output: f64,
    /// Cache cost information.
    pub cache: CostCache,
}

/// A language-model definition exposed by a provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Model {
    /// Unique model identifier.
    pub id: String,
    /// Provider identifier.
    #[serde(rename = "providerID", default)]
    pub provider_id: String,
    /// API endpoint information.
    #[serde(default)]
    pub api: ModelApi,
    /// Human-readable model name.
    pub name: String,
    /// Model family.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Model capabilities.
    #[serde(default)]
    pub capabilities: ModelCapabilities,
    /// Cost information per token.
    pub cost: ModelCost,
    /// Context and output token limits.
    pub limit: ModelLimit,
    /// Model lifecycle status.
    #[serde(default = "default_model_status")]
    pub status: ModelStatus,
    /// Arbitrary provider-specific options.
    pub options: HashMap<String, serde_json::Value>,
    /// Custom headers for API requests.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// ISO-8601 release date.
    pub release_date: String,
    /// Model variants.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

const fn default_model_status() -> ModelStatus {
    ModelStatus::Active
}

/// Per-token cost information for a [`Model`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelCost {
    /// Cost per input token.
    pub input: f64,
    /// Cost per output token.
    pub output: f64,
    /// Cache cost information.
    #[serde(default)]
    pub cache: CostCache,
    /// Experimental pricing for over 200K context.
    #[serde(rename = "experimentalOver200K", skip_serializing_if = "Option::is_none")]
    pub experimental_over_200k: Option<CostExperimentalOver200K>,
}

/// Token limits for a [`Model`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelLimit {
    /// Maximum context window size in tokens.
    pub context: f64,
    /// Maximum input size in tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<f64>,
    /// Maximum output size in tokens.
    pub output: f64,
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

/// Provider source type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderSource {
    /// Sourced from environment variables.
    Env,
    /// Sourced from configuration file.
    Config,
    /// Custom provider.
    Custom,
    /// Sourced from an API.
    Api,
}

/// An LLM provider definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provider {
    /// Unique provider identifier.
    pub id: String,
    /// Human-readable provider name.
    pub name: String,
    /// Source of the provider configuration.
    #[serde(default = "default_provider_source")]
    pub source: ProviderSource,
    /// Environment variable names required for authentication.
    pub env: Vec<String>,
    /// Optional API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Arbitrary provider-specific options.
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
    /// Map of model identifiers to their definitions.
    pub models: HashMap<String, Model>,
}

const fn default_provider_source() -> ProviderSource {
    ProviderSource::Env
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

    /// Helper to build a minimal spec-compliant [`Model`] for tests.
    fn test_model() -> Model {
        Model {
            id: "gpt-4o".into(),
            provider_id: "openai".into(),
            api: ModelApi {
                id: "openai".into(),
                url: "https://api.openai.com/v1".into(),
                npm: "openai".into(),
            },
            name: "GPT-4o".into(),
            family: None,
            capabilities: ModelCapabilities {
                temperature: true,
                reasoning: false,
                attachment: true,
                toolcall: true,
                input: ModelMediaCapabilities {
                    text: true,
                    audio: false,
                    image: true,
                    video: false,
                    pdf: false,
                },
                output: ModelMediaCapabilities { text: true, ..Default::default() },
                interleaved: json!(false),
            },
            cost: ModelCost {
                input: 5.0,
                output: 15.0,
                cache: CostCache { read: 2.5, write: 0.0 },
                experimental_over_200k: None,
            },
            limit: ModelLimit { context: 128_000.0, input: None, output: 4_096.0 },
            status: ModelStatus::Active,
            options: HashMap::from([("streaming".into(), json!(true))]),
            headers: HashMap::new(),
            release_date: "2024-05-13".into(),
            variants: None,
        }
    }

    #[test]
    fn model_round_trip() {
        let model = test_model();
        let json_str = serde_json::to_string(&model).unwrap();
        assert!(json_str.contains("providerID"));
        assert!(json_str.contains("capabilities"));
        let back: Model = serde_json::from_str(&json_str).unwrap();
        assert_eq!(model, back);
    }

    #[test]
    fn model_cost_default_cache() {
        let cost = ModelCost {
            input: 1.0,
            output: 2.0,
            cache: CostCache::default(),
            experimental_over_200k: None,
        };
        let json_str = serde_json::to_string(&cost).unwrap();
        assert!(!json_str.contains("experimentalOver200K"));
        let back: ModelCost = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cost, back);
    }

    #[test]
    fn provider_round_trip() {
        let provider = Provider {
            id: "openai".into(),
            name: "OpenAI".into(),
            source: ProviderSource::Env,
            env: vec!["OPENAI_API_KEY".into()],
            key: None,
            options: HashMap::new(),
            models: HashMap::from([("gpt-4o".into(), test_model())]),
        };
        let json_str = serde_json::to_string(&provider).unwrap();
        assert!(json_str.contains("\"source\":\"env\""));
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
                name: "OpenAI".into(),
                source: ProviderSource::Env,
                env: vec!["OPENAI_API_KEY".into()],
                key: None,
                options: HashMap::new(),
                models: HashMap::new(),
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
    fn provider_no_key() {
        let provider = Provider {
            id: "custom".into(),
            name: "Custom".into(),
            source: ProviderSource::Custom,
            env: vec![],
            key: None,
            options: HashMap::new(),
            models: HashMap::new(),
        };
        let json_str = serde_json::to_string(&provider).unwrap();
        assert!(!json_str.contains("key"));
        assert!(json_str.contains("\"source\":\"custom\""));
        let back: Provider = serde_json::from_str(&json_str).unwrap();
        assert_eq!(provider, back);
    }

    #[test]
    fn cost_cache_round_trip() {
        let cache = CostCache { read: 1.5, write: 3.0 };
        let json_str = serde_json::to_string(&cache).unwrap();
        let back: CostCache = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cache, back);
    }

    #[test]
    fn model_cost_with_experimental() {
        let cost = ModelCost {
            input: 3.0,
            output: 6.0,
            cache: CostCache { read: 1.5, write: 0.0 },
            experimental_over_200k: Some(CostExperimentalOver200K {
                input: 6.0,
                output: 12.0,
                cache: CostCache { read: 3.0, write: 0.0 },
            }),
        };
        let json_str = serde_json::to_string(&cost).unwrap();
        assert!(json_str.contains("experimentalOver200K"));
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
            provider_id: "local".into(),
            api: ModelApi::default(),
            name: "Small Model".into(),
            family: None,
            capabilities: ModelCapabilities::default(),
            cost: ModelCost::default(),
            limit: ModelLimit { context: 4096.0, input: None, output: 512.0 },
            status: ModelStatus::Active,
            options: HashMap::new(),
            headers: HashMap::new(),
            release_date: "2025-01-01".into(),
            variants: None,
        };
        let json_str = serde_json::to_string(&model).unwrap();
        let back: Model = serde_json::from_str(&json_str).unwrap();
        assert_eq!(model, back);
    }

    #[test]
    fn model_from_spec_json() {
        let raw = json!({
            "id": "claude-sonnet-4-20250514",
            "providerID": "anthropic",
            "api": { "id": "anthropic", "url": "https://api.anthropic.com", "npm": "@anthropic-ai/sdk" },
            "name": "Claude Sonnet 4",
            "family": "claude",
            "capabilities": {
                "temperature": true,
                "reasoning": true,
                "attachment": true,
                "toolcall": true,
                "input": { "text": true, "audio": false, "image": true, "video": false, "pdf": true },
                "output": { "text": true, "audio": false, "image": false, "video": false, "pdf": false },
                "interleaved": { "field": "reasoning_content" }
            },
            "cost": {
                "input": 3.0,
                "output": 15.0,
                "cache": { "read": 0.3, "write": 3.75 }
            },
            "limit": { "context": 200000, "input": 190000, "output": 16384 },
            "status": "active",
            "options": {},
            "headers": { "anthropic-beta": "interleaved-thinking-2025-05-14" },
            "release_date": "2025-05-14"
        });
        let model: Model = serde_json::from_value(raw).unwrap();
        assert_eq!(model.id, "claude-sonnet-4-20250514");
        assert_eq!(model.provider_id, "anthropic");
        assert_eq!(model.family.as_deref(), Some("claude"));
        assert!(model.capabilities.reasoning);
        assert!(model.capabilities.input.pdf);
        assert_eq!(model.cost.cache.read, 0.3);
        assert_eq!(model.limit.input, Some(190_000.0));
        assert_eq!(model.status, ModelStatus::Active);
        assert_eq!(
            model.headers.get("anthropic-beta").map(String::as_str),
            Some("interleaved-thinking-2025-05-14")
        );
    }

    #[test]
    fn model_status_round_trip() {
        for (variant, expected) in [
            (ModelStatus::Alpha, "alpha"),
            (ModelStatus::Beta, "beta"),
            (ModelStatus::Deprecated, "deprecated"),
            (ModelStatus::Active, "active"),
        ] {
            let json_str = serde_json::to_string(&variant).unwrap();
            assert_eq!(json_str, format!("\"{}\"", expected));
            let back: ModelStatus = serde_json::from_str(&json_str).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn model_capabilities_round_trip() {
        let caps = ModelCapabilities {
            temperature: true,
            reasoning: true,
            attachment: false,
            toolcall: true,
            input: ModelMediaCapabilities {
                text: true,
                audio: false,
                image: true,
                video: false,
                pdf: true,
            },
            output: ModelMediaCapabilities { text: true, ..Default::default() },
            interleaved: json!(true),
        };
        let json_str = serde_json::to_string(&caps).unwrap();
        let back: ModelCapabilities = serde_json::from_str(&json_str).unwrap();
        assert_eq!(caps, back);
    }

    #[test]
    fn model_api_round_trip() {
        let api = ModelApi {
            id: "openai".into(),
            url: "https://api.openai.com/v1".into(),
            npm: "openai".into(),
        };
        let json_str = serde_json::to_string(&api).unwrap();
        let back: ModelApi = serde_json::from_str(&json_str).unwrap();
        assert_eq!(api, back);
    }

    #[test]
    fn provider_from_spec_json() {
        let raw = json!({
            "id": "anthropic",
            "name": "Anthropic",
            "source": "env",
            "env": ["ANTHROPIC_API_KEY"],
            "key": "sk-ant-xxx",
            "options": {},
            "models": {}
        });
        let provider: Provider = serde_json::from_value(raw).unwrap();
        assert_eq!(provider.id, "anthropic");
        assert_eq!(provider.source, ProviderSource::Env);
        assert_eq!(provider.key.as_deref(), Some("sk-ant-xxx"));
    }

    #[test]
    fn provider_source_variants() {
        for (variant, expected) in [
            (ProviderSource::Env, "env"),
            (ProviderSource::Config, "config"),
            (ProviderSource::Custom, "custom"),
            (ProviderSource::Api, "api"),
        ] {
            let json_str = serde_json::to_string(&variant).unwrap();
            assert_eq!(json_str, format!("\"{}\"", expected));
            let back: ProviderSource = serde_json::from_str(&json_str).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn model_limit_with_input() {
        let limit = ModelLimit { context: 200_000.0, input: Some(190_000.0), output: 16_384.0 };
        let json_str = serde_json::to_string(&limit).unwrap();
        assert!(json_str.contains("input"));
        let back: ModelLimit = serde_json::from_str(&json_str).unwrap();
        assert_eq!(limit, back);
    }

    #[test]
    fn model_limit_without_input() {
        let limit = ModelLimit { context: 128_000.0, input: None, output: 4_096.0 };
        let json_str = serde_json::to_string(&limit).unwrap();
        assert!(!json_str.contains("input"));
        let back: ModelLimit = serde_json::from_str(&json_str).unwrap();
        assert_eq!(limit, back);
    }
}
