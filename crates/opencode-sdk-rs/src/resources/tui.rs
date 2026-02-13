//! Tui resource types and methods mirroring the JS SDK's `resources/tui.ts`.

use serde::{Deserialize, Serialize};

use crate::{client::Opencode, error::OpencodeError};

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Response type for [`TuiResource::append_prompt`].
pub type TuiAppendPromptResponse = bool;

/// Response type for [`TuiResource::open_help`].
pub type TuiOpenHelpResponse = bool;

// ---------------------------------------------------------------------------
// Params
// ---------------------------------------------------------------------------

/// Parameters for [`TuiResource::append_prompt`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TuiAppendPromptParams {
    /// The text to append.
    pub text: String,
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Provides access to `/tui/*` endpoints.
pub struct TuiResource<'a> {
    client: &'a Opencode,
}

impl<'a> TuiResource<'a> {
    pub(crate) const fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    /// Append text to the TUI prompt.
    ///
    /// `POST /tui/append-prompt`
    pub async fn append_prompt(
        &self,
        params: &TuiAppendPromptParams,
    ) -> Result<TuiAppendPromptResponse, OpencodeError> {
        self.client.post("/tui/append-prompt", Some(params), None).await
    }

    /// Open the TUI help panel.
    ///
    /// `POST /tui/open-help`
    pub async fn open_help(&self) -> Result<TuiOpenHelpResponse, OpencodeError> {
        self.client.post::<TuiOpenHelpResponse, ()>("/tui/open-help", None, None).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tui_append_prompt_params_round_trip() {
        let params = TuiAppendPromptParams { text: "hello world".into() };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(json_str.contains(r#""text":"hello world"#));
        let back: TuiAppendPromptParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }
}
