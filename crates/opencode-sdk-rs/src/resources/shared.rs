//! Shared domain types mirroring the JS SDK's `resources/shared.ts`.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Individual error structs
// ---------------------------------------------------------------------------

/// An error indicating the message was aborted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageAbortedError {
    /// Arbitrary payload (maps to `unknown` in the JS SDK).
    pub data: Option<serde_json::Value>,
}

/// An error indicating a provider authentication failure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderAuthError {
    /// Structured error data.
    pub data: ProviderAuthErrorData,
}

/// Data payload for [`ProviderAuthError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderAuthErrorData {
    /// Human-readable error message.
    pub message: String,
    /// The identifier of the provider that rejected authentication.
    #[serde(rename = "providerID")]
    pub provider_id: String,
}

/// A generic / unknown error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnknownError {
    /// Structured error data.
    pub data: UnknownErrorData,
}

/// Data payload for [`UnknownError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnknownErrorData {
    /// Human-readable error message.
    pub message: String,
}

/// An error indicating the message output exceeded the allowed length.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageOutputLengthError {
    /// Arbitrary payload (maps to `unknown` in the JS SDK).
    pub data: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Discriminated union
// ---------------------------------------------------------------------------

/// A session-level error – one of the four known error kinds.
///
/// Serialised with a `"name"` tag so the JSON representation matches the JS
/// SDK's discriminated union: `{ "name": "ProviderAuthError", "data": … }`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "name")]
pub enum SessionError {
    /// The message was aborted by the user / system.
    MessageAbortedError {
        /// Arbitrary payload.
        data: Option<serde_json::Value>,
    },
    /// Provider authentication failed.
    ProviderAuthError {
        /// Structured error data.
        data: ProviderAuthErrorData,
    },
    /// A generic / unknown error.
    UnknownError {
        /// Structured error data.
        data: UnknownErrorData,
    },
    /// The message output exceeded the allowed length.
    MessageOutputLengthError {
        /// Arbitrary payload.
        data: Option<serde_json::Value>,
    },
}

// ---------------------------------------------------------------------------
// Conversions from individual structs into the enum
// ---------------------------------------------------------------------------

impl From<MessageAbortedError> for SessionError {
    fn from(e: MessageAbortedError) -> Self {
        Self::MessageAbortedError { data: e.data }
    }
}

impl From<ProviderAuthError> for SessionError {
    fn from(e: ProviderAuthError) -> Self {
        Self::ProviderAuthError { data: e.data }
    }
}

impl From<UnknownError> for SessionError {
    fn from(e: UnknownError) -> Self {
        Self::UnknownError { data: e.data }
    }
}

impl From<MessageOutputLengthError> for SessionError {
    fn from(e: MessageOutputLengthError) -> Self {
        Self::MessageOutputLengthError { data: e.data }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // -- Individual struct round-trips --

    #[test]
    fn message_aborted_error_round_trip() {
        let err = MessageAbortedError { data: Some(json!({"reason": "user cancelled"})) };
        let json = serde_json::to_string(&err).unwrap();
        let back: MessageAbortedError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn message_aborted_error_null_data() {
        let err = MessageAbortedError { data: None };
        let json = serde_json::to_string(&err).unwrap();
        let back: MessageAbortedError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn provider_auth_error_round_trip() {
        let err = ProviderAuthError {
            data: ProviderAuthErrorData {
                message: "invalid token".into(),
                provider_id: "openai".into(),
            },
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("providerID"));
        let back: ProviderAuthError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn unknown_error_round_trip() {
        let err =
            UnknownError { data: UnknownErrorData { message: "something went wrong".into() } };
        let json = serde_json::to_string(&err).unwrap();
        let back: UnknownError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn message_output_length_error_round_trip() {
        let err = MessageOutputLengthError { data: Some(json!(42)) };
        let json = serde_json::to_string(&err).unwrap();
        let back: MessageOutputLengthError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    // -- SessionError enum deserialisation via `name` tag --

    #[test]
    fn session_error_message_aborted() {
        let input = json!({
            "name": "MessageAbortedError",
            "data": null
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(err, SessionError::MessageAbortedError { data: None });
    }

    #[test]
    fn session_error_provider_auth() {
        let input = json!({
            "name": "ProviderAuthError",
            "data": {
                "message": "bad credentials",
                "providerID": "anthropic"
            }
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(
            err,
            SessionError::ProviderAuthError {
                data: ProviderAuthErrorData {
                    message: "bad credentials".into(),
                    provider_id: "anthropic".into(),
                }
            }
        );
    }

    #[test]
    fn session_error_unknown() {
        let input = json!({
            "name": "UnknownError",
            "data": {
                "message": "oops"
            }
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(
            err,
            SessionError::UnknownError { data: UnknownErrorData { message: "oops".into() } }
        );
    }

    #[test]
    fn session_error_message_output_length() {
        let input = json!({
            "name": "MessageOutputLengthError",
            "data": {"limit": 4096}
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(
            err,
            SessionError::MessageOutputLengthError { data: Some(json!({"limit": 4096})) }
        );
    }

    #[test]
    fn session_error_round_trip_serialization() {
        let err = SessionError::ProviderAuthError {
            data: ProviderAuthErrorData { message: "expired".into(), provider_id: "google".into() },
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "ProviderAuthError");
        assert_eq!(json["data"]["providerID"], "google");

        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    // -- Edge cases: full round-trip for every SessionError variant --

    #[test]
    fn session_error_message_aborted_round_trip_with_data() {
        let err = SessionError::MessageAbortedError {
            data: Some(json!({"reason": "user pressed ctrl-c"})),
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "MessageAbortedError");
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn session_error_message_aborted_round_trip_null_data() {
        let err = SessionError::MessageAbortedError { data: None };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "MessageAbortedError");
        assert_eq!(json["data"], serde_json::Value::Null);
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn session_error_unknown_round_trip() {
        let err =
            SessionError::UnknownError { data: UnknownErrorData { message: "kaboom".into() } };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "UnknownError");
        assert_eq!(json["data"]["message"], "kaboom");
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn session_error_output_length_round_trip_with_data() {
        let err = SessionError::MessageOutputLengthError {
            data: Some(json!({"limit": 8192, "actual": 10000})),
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "MessageOutputLengthError");
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn session_error_output_length_round_trip_null_data() {
        let err = SessionError::MessageOutputLengthError { data: None };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "MessageOutputLengthError");
        assert_eq!(json["data"], serde_json::Value::Null);
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn provider_auth_error_data_fields() {
        let data = ProviderAuthErrorData {
            message: "token expired".into(),
            provider_id: "azure-openai".into(),
        };
        let v = serde_json::to_value(&data).unwrap();
        // Verify rename: Rust field is provider_id, JSON key is providerID
        assert_eq!(v["providerID"], "azure-openai");
        assert!(v.get("provider_id").is_none());
        assert_eq!(v["message"], "token expired");
        let back: ProviderAuthErrorData = serde_json::from_value(v).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn message_output_length_error_null_data() {
        let err = MessageOutputLengthError { data: None };
        let json_str = serde_json::to_string(&err).unwrap();
        let back: MessageOutputLengthError = serde_json::from_str(&json_str).unwrap();
        assert_eq!(err, back);
    }
}
