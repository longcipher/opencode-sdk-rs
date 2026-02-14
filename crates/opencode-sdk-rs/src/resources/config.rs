//! Config resource types and methods mirroring the JS SDK's `resources/config.ts`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    client::{Opencode, RequestOptions},
    error::OpencodeError,
};

// ---------------------------------------------------------------------------
// ModeConfig
// ---------------------------------------------------------------------------

/// Configuration for an operational mode (shared base for agents/modes).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModeConfig {
    /// Whether this mode is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable: Option<bool>,

    /// Optional model override for this mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Optional system prompt override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Optional temperature override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Map of tool names to their enabled state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, bool>>,
}

// ---------------------------------------------------------------------------
// Agent types
// ---------------------------------------------------------------------------

/// Configuration for a single agent entry.
///
/// Combines a human-readable `description` with all fields from [`ModeConfig`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    /// Human-readable description of this agent.
    pub description: String,

    /// Flattened mode configuration fields.
    #[serde(flatten)]
    pub mode: ModeConfig,
}

/// Map of agent names to their configuration.
///
/// The key `"general"` is the conventional default agent entry in the JS SDK.
pub type Agent = HashMap<String, AgentConfig>;

// ---------------------------------------------------------------------------
// Experimental / Hooks
// ---------------------------------------------------------------------------

/// A hook command entry (used by both `file_edited` and `session_completed`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookCommand {
    /// The command and its arguments.
    pub command: Vec<String>,

    /// Optional environment variables for the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
}

/// Hook configuration within [`Experimental`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Hook {
    /// Hooks triggered when a file is edited, keyed by glob / pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_edited: Option<HashMap<String, Vec<HookCommand>>>,

    /// Hooks triggered when a session completes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_completed: Option<Vec<HookCommand>>,
}

/// Experimental features configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Experimental {
    /// Hook definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook: Option<Hook>,
}

// ---------------------------------------------------------------------------
// Keybinds
// ---------------------------------------------------------------------------

/// Keybinding configuration mirroring the JS SDK's `KeybindsConfig`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct KeybindsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_exit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_open: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_close: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_diff_toggle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_clear: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_newline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_paste: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_submit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_copy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_first: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_half_page_down: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_half_page_up: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_last: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_layout_toggle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_page_down: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_page_up: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_previous: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_redo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_revert: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_undo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_init: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_compact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_export: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_interrupt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_new: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_share: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_unshare: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_mode_reverse: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_details: Option<String>,
}

// ---------------------------------------------------------------------------
// MCP config
// ---------------------------------------------------------------------------

/// Local MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpLocalConfig {
    /// The command and its arguments.
    pub command: Vec<String>,

    /// Whether this MCP server is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Optional environment variables for the process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
}

/// Remote MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpRemoteConfig {
    /// The URL of the remote MCP server.
    pub url: String,

    /// Whether this MCP server is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Optional HTTP headers to send with requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// Discriminated union of MCP server configurations, tagged by `"type"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum McpConfig {
    /// A locally-spawned MCP server.
    #[serde(rename = "local")]
    Local(McpLocalConfig),

    /// A remote MCP server accessed over HTTP.
    #[serde(rename = "remote")]
    Remote(McpRemoteConfig),
}

// ---------------------------------------------------------------------------
// Provider config
// ---------------------------------------------------------------------------

/// Cost information for a model (input/output tokens).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelCost {
    /// Cost per input token.
    pub input: f64,

    /// Cost per output token.
    pub output: f64,

    /// Cost per cached-read token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<f64>,

    /// Cost per cached-write token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<f64>,
}

/// Context and output limits for a model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelLimit {
    /// Maximum context window size in tokens.
    pub context: u64,

    /// Maximum output size in tokens.
    pub output: u64,
}

/// Configuration for a single model within a provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderModelConfig {
    /// Model identifier override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Whether the model supports file attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<bool>,

    /// Token cost information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<ModelCost>,

    /// Context and output limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<ModelLimit>,

    /// Display name for the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Arbitrary model-specific options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,

    /// Whether the model supports reasoning / chain-of-thought.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,

    /// Release date string (e.g. `"2024-05-13"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,

    /// Whether the model supports temperature adjustment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<bool>,

    /// Whether the model supports tool calling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<bool>,
}

/// Provider-level options (API key, base URL, and arbitrary extras).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProviderOptions {
    /// API key for authenticating with the provider.
    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Override for the provider's base URL.
    #[serde(rename = "baseURL", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// Arbitrary additional options.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Configuration for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    /// Map of model identifiers to their configuration.
    pub models: HashMap<String, ProviderModelConfig>,

    /// Provider identifier override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Provider API endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,

    /// Environment variable names used for authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<String>>,

    /// Display name for the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// NPM package name (JS SDK compatibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,

    /// Provider-level options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ProviderOptions>,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// How session sharing is configured.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShareMode {
    /// Share only on explicit user action.
    Manual,
    /// Share automatically.
    Auto,
    /// Sharing is disabled.
    Disabled,
}

/// UI layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    /// Automatically choose layout.
    Auto,
    /// Stretch to fill available space.
    Stretch,
}

// ---------------------------------------------------------------------------
// Mode map
// ---------------------------------------------------------------------------

/// Map of mode names (e.g. `"build"`, `"plan"`) to their configuration.
pub type ModeMap = HashMap<String, ModeConfig>;

// ---------------------------------------------------------------------------
// Top-level Config
// ---------------------------------------------------------------------------

/// Top-level configuration returned by `GET /config`.
///
/// All fields are optional to tolerate partial / minimal server responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    /// JSON Schema reference.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Agent configuration map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<Agent>,

    /// Whether to auto-share sessions (deprecated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoshare: Option<bool>,

    /// Whether to auto-update the application.
    /// Can be a boolean or the string "notify" to show update notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoupdate: Option<serde_json::Value>,

    /// List of disabled provider identifiers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_providers: Option<Vec<String>>,

    /// Experimental features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Experimental>,

    /// Custom instructions / system prompts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Vec<String>>,

    /// Keybinding configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keybinds: Option<KeybindsConfig>,

    /// UI layout (deprecated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,

    /// MCP server configurations, keyed by server name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp: Option<HashMap<String, McpConfig>>,

    /// Mode configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ModeMap>,

    /// Default model identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Provider configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<HashMap<String, ProviderConfig>>,

    /// Session sharing mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<ShareMode>,

    /// Default small-model identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_model: Option<String>,

    /// UI theme name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,

    /// Display username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Handle for the `/config` resource.
#[derive(Debug, Clone)]
pub struct ConfigResource<'a> {
    client: &'a Opencode,
}

impl<'a> ConfigResource<'a> {
    /// Create a new `ConfigResource` bound to the given client.
    pub(crate) const fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    /// Retrieve the current configuration (`GET /config`).
    pub async fn get(&self, options: Option<&RequestOptions>) -> Result<Config, OpencodeError> {
        self.client.get("/config", options).await
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
    fn mode_config_round_trip() {
        let mc = ModeConfig {
            disable: Some(false),
            model: Some("gpt-4o".into()),
            prompt: Some("You are helpful.".into()),
            temperature: Some(0.7),
            tools: Some(HashMap::from([("bash".into(), true), ("file_write".into(), false)])),
        };
        let json_str = serde_json::to_string(&mc).unwrap();
        let back: ModeConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(mc, back);
    }

    #[test]
    fn mode_config_empty() {
        let mc = ModeConfig::default();
        let json_str = serde_json::to_string(&mc).unwrap();
        assert_eq!(json_str, "{}");
        let back: ModeConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(mc, back);
    }

    #[test]
    fn mcp_local_round_trip() {
        let cfg = McpConfig::Local(McpLocalConfig {
            command: vec!["npx".into(), "mcp-server".into()],
            enabled: Some(true),
            environment: Some(HashMap::from([("NODE_ENV".into(), "production".into())])),
        });
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["type"], "local");
        assert_eq!(v["command"], json!(["npx", "mcp-server"]));
        let back: McpConfig = serde_json::from_value(v).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn mcp_remote_round_trip() {
        let cfg = McpConfig::Remote(McpRemoteConfig {
            url: "https://mcp.example.com".into(),
            enabled: None,
            headers: Some(HashMap::from([("Authorization".into(), "Bearer tok".into())])),
        });
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["type"], "remote");
        assert_eq!(v["url"], "https://mcp.example.com");
        let back: McpConfig = serde_json::from_value(v).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn keybinds_config_round_trip() {
        let kb = KeybindsConfig {
            app_exit: Some("ctrl+q".into()),
            app_help: Some("ctrl+h".into()),
            editor_open: Some("ctrl+e".into()),
            file_close: Some("ctrl+w".into()),
            file_diff_toggle: Some("ctrl+d".into()),
            file_list: Some("ctrl+l".into()),
            file_search: Some("ctrl+f".into()),
            input_clear: Some("ctrl+u".into()),
            input_newline: Some("shift+enter".into()),
            input_paste: Some("ctrl+v".into()),
            input_submit: Some("enter".into()),
            leader: Some("ctrl+space".into()),
            messages_copy: Some("ctrl+c".into()),
            messages_first: Some("home".into()),
            messages_half_page_down: Some("ctrl+d".into()),
            messages_half_page_up: Some("ctrl+u".into()),
            messages_last: Some("end".into()),
            messages_layout_toggle: Some("ctrl+t".into()),
            messages_next: Some("ctrl+n".into()),
            messages_page_down: Some("pagedown".into()),
            messages_page_up: Some("pageup".into()),
            messages_previous: Some("ctrl+p".into()),
            messages_redo: Some("ctrl+y".into()),
            messages_revert: Some("ctrl+r".into()),
            messages_undo: Some("ctrl+z".into()),
            model_list: Some("ctrl+m".into()),
            project_init: Some("ctrl+i".into()),
            session_compact: Some("ctrl+k".into()),
            session_export: Some("ctrl+shift+e".into()),
            session_interrupt: Some("escape".into()),
            session_list: Some("ctrl+s".into()),
            session_new: Some("ctrl+shift+n".into()),
            session_share: Some("ctrl+shift+s".into()),
            session_unshare: Some("ctrl+shift+u".into()),
            switch_mode: Some("tab".into()),
            switch_mode_reverse: Some("shift+tab".into()),
            theme_list: Some("ctrl+shift+t".into()),
            tool_details: Some("ctrl+shift+d".into()),
        };
        let json_str = serde_json::to_string(&kb).unwrap();
        let back: KeybindsConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(kb, back);
    }

    #[test]
    fn config_with_schema_field() {
        let cfg = Config {
            schema: Some("https://opencode.ai/config.schema.json".into()),
            ..Default::default()
        };
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["$schema"], "https://opencode.ai/config.schema.json");
        assert!(v.get("schema").is_none(), "$schema must not appear as 'schema'");
        let back: Config = serde_json::from_value(v).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn config_full_round_trip() {
        let cfg = Config {
            schema: Some("https://opencode.ai/schema.json".into()),
            agent: Some(HashMap::from([(
                "general".into(),
                AgentConfig {
                    description: "Default agent".into(),
                    mode: ModeConfig {
                        model: Some("claude-3-opus".into()),
                        temperature: Some(0.5),
                        ..Default::default()
                    },
                },
            )])),
            autoshare: Some(false),
            autoupdate: Some(serde_json::Value::Bool(true)),
            disabled_providers: Some(vec!["azure".into()]),
            experimental: Some(Experimental {
                hook: Some(Hook {
                    file_edited: Some(HashMap::from([(
                        "*.rs".into(),
                        vec![HookCommand {
                            command: vec!["cargo".into(), "fmt".into()],
                            environment: None,
                        }],
                    )])),
                    session_completed: Some(vec![HookCommand {
                        command: vec!["notify-send".into(), "done".into()],
                        environment: Some(HashMap::from([("DISPLAY".into(), ":0".into())])),
                    }]),
                }),
            }),
            instructions: Some(vec!["Be concise.".into()]),
            keybinds: None,
            layout: Some(Layout::Auto),
            mcp: Some(HashMap::from([(
                "local-server".into(),
                McpConfig::Local(McpLocalConfig {
                    command: vec!["node".into(), "server.js".into()],
                    enabled: Some(true),
                    environment: None,
                }),
            )])),
            mode: Some(HashMap::from([(
                "build".into(),
                ModeConfig { model: Some("gpt-4o".into()), ..Default::default() },
            )])),
            model: Some("claude-3-opus".into()),
            provider: Some(HashMap::from([(
                "openai".into(),
                ProviderConfig {
                    models: HashMap::from([(
                        "gpt-4o".into(),
                        ProviderModelConfig {
                            id: Some("gpt-4o".into()),
                            attachment: Some(true),
                            cost: Some(ModelCost {
                                input: 5.0,
                                output: 15.0,
                                cache_read: None,
                                cache_write: None,
                            }),
                            limit: Some(ModelLimit { context: 128_000, output: 4_096 }),
                            name: Some("GPT-4o".into()),
                            options: None,
                            reasoning: Some(false),
                            release_date: Some("2024-05-13".into()),
                            temperature: Some(true),
                            tool_call: Some(true),
                        },
                    )]),
                    id: Some("openai".into()),
                    api: Some("https://api.openai.com/v1".into()),
                    env: Some(vec!["OPENAI_API_KEY".into()]),
                    name: Some("OpenAI".into()),
                    npm: None,
                    options: Some(ProviderOptions {
                        api_key: None,
                        base_url: Some("https://api.openai.com/v1".into()),
                        extra: HashMap::new(),
                    }),
                },
            )])),
            share: Some(ShareMode::Manual),
            small_model: Some("gpt-4o-mini".into()),
            theme: Some("dark".into()),
            username: Some("developer".into()),
        };
        let json_str = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn share_mode_serde() {
        for (variant, expected) in [
            (ShareMode::Manual, "manual"),
            (ShareMode::Auto, "auto"),
            (ShareMode::Disabled, "disabled"),
        ] {
            let json_str = serde_json::to_string(&variant).unwrap();
            assert_eq!(json_str, format!("\"{expected}\""));
            let back: ShareMode = serde_json::from_str(&json_str).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn layout_serde() {
        for (variant, expected) in [(Layout::Auto, "auto"), (Layout::Stretch, "stretch")] {
            let json_str = serde_json::to_string(&variant).unwrap();
            assert_eq!(json_str, format!("\"{expected}\""));
            let back: Layout = serde_json::from_str(&json_str).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn agent_config_flatten() {
        let ac = AgentConfig {
            description: "Build agent".into(),
            mode: ModeConfig {
                model: Some("gpt-4o".into()),
                tools: Some(HashMap::from([("bash".into(), true)])),
                ..Default::default()
            },
        };
        let v = serde_json::to_value(&ac).unwrap();
        // Flattened fields appear at the top level
        assert_eq!(v["description"], "Build agent");
        assert_eq!(v["model"], "gpt-4o");
        assert_eq!(v["tools"]["bash"], true);
        let back: AgentConfig = serde_json::from_value(v).unwrap();
        assert_eq!(ac, back);
    }

    #[test]
    fn provider_options_with_extras() {
        let opts = ProviderOptions {
            api_key: Some("sk-test".into()),
            base_url: None,
            extra: HashMap::from([("organization".into(), json!("org-123"))]),
        };
        let v = serde_json::to_value(&opts).unwrap();
        assert_eq!(v["apiKey"], "sk-test");
        assert_eq!(v["organization"], "org-123");
        let back: ProviderOptions = serde_json::from_value(v).unwrap();
        assert_eq!(opts, back);
    }

    #[test]
    fn config_empty_round_trip() {
        let cfg = Config::default();
        let json_str = serde_json::to_string(&cfg).unwrap();
        assert_eq!(json_str, "{}");
        let back: Config = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cfg, back);
    }

    // -- Edge cases --

    #[test]
    fn config_minimal_partial_fields() {
        let cfg = Config {
            theme: Some("dark".into()),
            autoupdate: Some(serde_json::Value::Bool(false)),
            ..Default::default()
        };
        let json_str = serde_json::to_string(&cfg).unwrap();
        // Only the two set fields should appear
        assert!(json_str.contains("theme"));
        assert!(json_str.contains("autoupdate"));
        assert!(!json_str.contains("$schema"));
        assert!(!json_str.contains("agent"));
        assert!(!json_str.contains("mcp"));
        let back: Config = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn mcp_local_minimal() {
        let cfg = McpConfig::Local(McpLocalConfig {
            command: vec!["my-server".into()],
            enabled: None,
            environment: None,
        });
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["type"], "local");
        assert!(v.get("enabled").is_none());
        assert!(v.get("environment").is_none());
        let back: McpConfig = serde_json::from_value(v).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn mcp_remote_minimal() {
        let cfg = McpConfig::Remote(McpRemoteConfig {
            url: "https://remote.example.com".into(),
            enabled: None,
            headers: None,
        });
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["type"], "remote");
        assert!(v.get("enabled").is_none());
        assert!(v.get("headers").is_none());
        let back: McpConfig = serde_json::from_value(v).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn mode_config_single_field() {
        let mc = ModeConfig { temperature: Some(0.3), ..Default::default() };
        let v = serde_json::to_value(&mc).unwrap();
        assert_eq!(v["temperature"], 0.3);
        assert!(v.get("disable").is_none());
        assert!(v.get("model").is_none());
        assert!(v.get("prompt").is_none());
        assert!(v.get("tools").is_none());
        let back: ModeConfig = serde_json::from_value(v).unwrap();
        assert_eq!(mc, back);
    }

    #[test]
    fn config_with_empty_collections() {
        let cfg = Config {
            disabled_providers: Some(vec![]),
            instructions: Some(vec![]),
            mcp: Some(HashMap::new()),
            mode: Some(HashMap::new()),
            provider: Some(HashMap::new()),
            ..Default::default()
        };
        let json_str = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json_str).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn hook_command_minimal() {
        let hc = HookCommand { command: vec!["echo".into(), "done".into()], environment: None };
        let v = serde_json::to_value(&hc).unwrap();
        assert!(v.get("environment").is_none());
        let back: HookCommand = serde_json::from_value(v).unwrap();
        assert_eq!(hc, back);
    }

    #[test]
    fn experimental_no_hooks() {
        let exp = Experimental { hook: None };
        let v = serde_json::to_value(&exp).unwrap();
        assert!(v.get("hook").is_none());
        let back: Experimental = serde_json::from_value(v).unwrap();
        assert_eq!(exp, back);
    }

    #[test]
    fn provider_config_minimal() {
        let pc = ProviderConfig {
            models: HashMap::new(),
            id: None,
            api: None,
            env: None,
            name: None,
            npm: None,
            options: None,
        };
        let v = serde_json::to_value(&pc).unwrap();
        assert!(v.get("id").is_none());
        assert!(v.get("api").is_none());
        assert!(v.get("name").is_none());
        let back: ProviderConfig = serde_json::from_value(v).unwrap();
        assert_eq!(pc, back);
    }

    #[test]
    fn provider_model_config_all_none() {
        let pmc = ProviderModelConfig::default();
        let json_str = serde_json::to_string(&pmc).unwrap();
        assert_eq!(json_str, "{}");
        let back: ProviderModelConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(pmc, back);
    }
}
