//! API resource modules, one per endpoint group.

pub mod app;
pub mod config;
pub mod event;
pub mod file;
pub mod find;
pub mod session;
pub mod shared;
pub mod tui;

// Re-export all types for convenience
pub use app::*;
// Re-export config types explicitly to avoid ambiguity with `ModelCost` / `ModelLimit`
// which are already re-exported from `app`.
pub use config::{
    Agent, AgentConfig, Config, ConfigResource, Experimental, Hook, HookCommand, KeybindsConfig,
    Layout, McpConfig, McpLocalConfig, McpRemoteConfig, ModeConfig, ModeMap, ProviderConfig,
    ProviderModelConfig, ProviderOptions, ShareMode,
};
pub use event::*;
pub use file::*;
pub use find::*;
pub use session::*;
pub use shared::*;
pub use tui::*;
