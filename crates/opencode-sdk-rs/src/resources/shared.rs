//! Shared domain types mirroring the JS SDK's `resources/shared.ts`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Individual error structs
// ---------------------------------------------------------------------------

/// An error indicating the message was aborted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageAbortedError {
    /// Structured error data.
    pub data: MessageAbortedErrorData,
}

/// Data payload for [`MessageAbortedError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageAbortedErrorData {
    /// Optional human-readable error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
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

/// An error indicating that structured output validation failed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredOutputError {
    /// Structured error data.
    pub data: StructuredOutputErrorData,
}

/// Data payload for [`StructuredOutputError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredOutputErrorData {
    /// Human-readable error message.
    pub message: String,
    /// Number of retries attempted.
    pub retries: f64,
}

/// An error indicating the context window was exceeded.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextOverflowError {
    /// Structured error data.
    pub data: ContextOverflowErrorData,
}

/// Data payload for [`ContextOverflowError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextOverflowErrorData {
    /// Human-readable error message.
    pub message: String,
    /// Optional response body from the provider.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "responseBody")]
    pub response_body: Option<String>,
}

/// An error originating from the upstream API provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    /// Structured error data.
    pub data: ApiErrorData,
}

/// Data payload for [`ApiError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorData {
    /// Human-readable error message.
    pub message: String,
    /// HTTP status code returned by the provider, if available.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "statusCode")]
    pub status_code: Option<f64>,
    /// Whether the error is retryable.
    #[serde(rename = "isRetryable")]
    pub is_retryable: bool,
    /// Response headers from the provider, if available.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "responseHeaders")]
    pub response_headers: Option<HashMap<String, String>>,
    /// Response body from the provider, if available.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "responseBody")]
    pub response_body: Option<String>,
    /// Additional metadata about the error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

// ---------------------------------------------------------------------------
// Discriminated union
// ---------------------------------------------------------------------------

/// A session-level error – one of the known error kinds.
///
/// Serialised with a `"name"` tag so the JSON representation matches the JS
/// SDK's discriminated union: `{ "name": "ProviderAuthError", "data": … }`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "name")]
pub enum SessionError {
    /// The message was aborted by the user / system.
    MessageAbortedError {
        /// Structured error data.
        data: MessageAbortedErrorData,
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
    /// Structured output validation failed.
    StructuredOutputError {
        /// Structured error data.
        data: StructuredOutputErrorData,
    },
    /// The context window was exceeded.
    ContextOverflowError {
        /// Structured error data.
        data: ContextOverflowErrorData,
    },
    /// An error originating from the upstream API provider.
    #[allow(clippy::upper_case_acronyms)]
    APIError {
        /// Structured error data.
        data: ApiErrorData,
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

impl From<StructuredOutputError> for SessionError {
    fn from(e: StructuredOutputError) -> Self {
        Self::StructuredOutputError { data: e.data }
    }
}

impl From<ContextOverflowError> for SessionError {
    fn from(e: ContextOverflowError) -> Self {
        Self::ContextOverflowError { data: e.data }
    }
}

impl From<ApiError> for SessionError {
    fn from(e: ApiError) -> Self {
        Self::APIError { data: e.data }
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
        let err = MessageAbortedError {
            data: MessageAbortedErrorData { message: Some("user cancelled".into()) },
        };
        let json = serde_json::to_string(&err).unwrap();
        let back: MessageAbortedError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn message_aborted_error_null_message() {
        let err = MessageAbortedError { data: MessageAbortedErrorData { message: None } };
        let json = serde_json::to_string(&err).unwrap();
        let back: MessageAbortedError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn message_aborted_error_from_empty_object() {
        let input = json!({"data": {}});
        let err: MessageAbortedError = serde_json::from_value(input).unwrap();
        assert_eq!(err.data.message, None);
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
            "data": {}
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(
            err,
            SessionError::MessageAbortedError { data: MessageAbortedErrorData { message: None } }
        );
    }

    #[test]
    fn session_error_message_aborted_with_message() {
        let input = json!({
            "name": "MessageAbortedError",
            "data": { "message": "cancelled" }
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(
            err,
            SessionError::MessageAbortedError {
                data: MessageAbortedErrorData { message: Some("cancelled".into()) }
            }
        );
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
    fn session_error_message_aborted_round_trip_with_message() {
        let err = SessionError::MessageAbortedError {
            data: MessageAbortedErrorData { message: Some("user pressed ctrl-c".into()) },
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "MessageAbortedError");
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn session_error_message_aborted_round_trip_no_message() {
        let err =
            SessionError::MessageAbortedError { data: MessageAbortedErrorData { message: None } };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "MessageAbortedError");
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

    // -- StructuredOutputError tests --

    #[test]
    fn structured_output_error_round_trip() {
        let err = StructuredOutputError {
            data: StructuredOutputErrorData { message: "schema mismatch".into(), retries: 3.0 },
        };
        let json = serde_json::to_string(&err).unwrap();
        let back: StructuredOutputError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn session_error_structured_output() {
        let input = json!({
            "name": "StructuredOutputError",
            "data": {
                "message": "invalid schema",
                "retries": 2.0
            }
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(
            err,
            SessionError::StructuredOutputError {
                data: StructuredOutputErrorData { message: "invalid schema".into(), retries: 2.0 }
            }
        );
    }

    #[test]
    fn session_error_structured_output_round_trip() {
        let err = SessionError::StructuredOutputError {
            data: StructuredOutputErrorData { message: "bad output".into(), retries: 5.0 },
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "StructuredOutputError");
        assert_eq!(json["data"]["retries"], 5.0);
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn structured_output_error_from_conversion() {
        let err = StructuredOutputError {
            data: StructuredOutputErrorData { message: "fail".into(), retries: 1.0 },
        };
        let session: SessionError = err.into();
        assert!(matches!(session, SessionError::StructuredOutputError { .. }));
    }

    // -- ContextOverflowError tests --

    #[test]
    fn context_overflow_error_round_trip() {
        let err = ContextOverflowError {
            data: ContextOverflowErrorData {
                message: "context too large".into(),
                response_body: Some("truncated".into()),
            },
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("responseBody"));
        let back: ContextOverflowError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn context_overflow_error_no_response_body() {
        let err = ContextOverflowError {
            data: ContextOverflowErrorData { message: "overflow".into(), response_body: None },
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(!json.contains("responseBody"));
        let back: ContextOverflowError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn session_error_context_overflow() {
        let input = json!({
            "name": "ContextOverflowError",
            "data": {
                "message": "window exceeded",
                "responseBody": "partial response"
            }
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(
            err,
            SessionError::ContextOverflowError {
                data: ContextOverflowErrorData {
                    message: "window exceeded".into(),
                    response_body: Some("partial response".into()),
                }
            }
        );
    }

    #[test]
    fn session_error_context_overflow_round_trip() {
        let err = SessionError::ContextOverflowError {
            data: ContextOverflowErrorData { message: "too big".into(), response_body: None },
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "ContextOverflowError");
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn context_overflow_error_from_conversion() {
        let err = ContextOverflowError {
            data: ContextOverflowErrorData { message: "overflow".into(), response_body: None },
        };
        let session: SessionError = err.into();
        assert!(matches!(session, SessionError::ContextOverflowError { .. }));
    }

    // -- APIError tests --

    #[test]
    fn api_error_round_trip() {
        let mut headers = HashMap::new();
        headers.insert("x-request-id".into(), "abc123".into());
        let err = ApiError {
            data: ApiErrorData {
                message: "rate limited".into(),
                status_code: Some(429.0),
                is_retryable: true,
                response_headers: Some(headers),
                response_body: Some("{\"error\": \"too many requests\"}".into()),
                metadata: None,
            },
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("statusCode"));
        assert!(json.contains("isRetryable"));
        assert!(json.contains("responseHeaders"));
        assert!(json.contains("responseBody"));
        let back: ApiError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn api_error_minimal() {
        let err = ApiError {
            data: ApiErrorData {
                message: "server error".into(),
                status_code: None,
                is_retryable: false,
                response_headers: None,
                response_body: None,
                metadata: None,
            },
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(!json.contains("statusCode"));
        assert!(!json.contains("responseHeaders"));
        assert!(!json.contains("responseBody"));
        assert!(!json.contains("metadata"));
        let back: ApiError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn session_error_api_error() {
        let input = json!({
            "name": "APIError",
            "data": {
                "message": "upstream failure",
                "statusCode": 500.0,
                "isRetryable": true
            }
        });
        let err: SessionError = serde_json::from_value(input).unwrap();
        assert_eq!(
            err,
            SessionError::APIError {
                data: ApiErrorData {
                    message: "upstream failure".into(),
                    status_code: Some(500.0),
                    is_retryable: true,
                    response_headers: None,
                    response_body: None,
                    metadata: None,
                }
            }
        );
    }

    #[test]
    fn session_error_api_error_round_trip() {
        let mut meta = HashMap::new();
        meta.insert("region".into(), "us-east-1".into());
        let err = SessionError::APIError {
            data: ApiErrorData {
                message: "bad gateway".into(),
                status_code: Some(502.0),
                is_retryable: true,
                response_headers: None,
                response_body: None,
                metadata: Some(meta),
            },
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["name"], "APIError");
        assert_eq!(json["data"]["statusCode"], 502.0);
        assert_eq!(json["data"]["isRetryable"], true);
        let back: SessionError = serde_json::from_value(json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn api_error_from_conversion() {
        let err = ApiError {
            data: ApiErrorData {
                message: "oops".into(),
                status_code: None,
                is_retryable: false,
                response_headers: None,
                response_body: None,
                metadata: None,
            },
        };
        let session: SessionError = err.into();
        assert!(matches!(session, SessionError::APIError { .. }));
    }

    #[test]
    fn api_error_data_field_renames() {
        let data = ApiErrorData {
            message: "test".into(),
            status_code: Some(401.0),
            is_retryable: false,
            response_headers: None,
            response_body: None,
            metadata: None,
        };
        let v = serde_json::to_value(&data).unwrap();
        assert!(v.get("statusCode").is_some());
        assert!(v.get("status_code").is_none());
        assert!(v.get("isRetryable").is_some());
        assert!(v.get("is_retryable").is_none());
        let back: ApiErrorData = serde_json::from_value(v).unwrap();
        assert_eq!(data, back);
    }
}
