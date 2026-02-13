use http::HeaderMap;
use serde_json::Value;

/// Primary error type for the `OpenCode` SDK.
///
/// Models the JS SDK's error hierarchy as a flat enum with variants for
/// API errors (with status codes), connection errors, timeouts, user aborts,
/// serialization errors, and generic HTTP transport errors.
#[derive(Debug, thiserror::Error)]
pub enum OpencodeError {
    /// An API error returned by the server with an HTTP status code.
    #[error("{status} {message}")]
    Api { status: u16, headers: Option<Box<HeaderMap>>, body: Option<Box<Value>>, message: String },

    /// A connection-level error (DNS, TCP, TLS, etc.).
    #[error("Connection error: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// The request timed out.
    #[error("Request timed out.")]
    Timeout,

    /// The user aborted the request.
    #[error("Request was aborted.")]
    UserAbort,

    /// Failed to serialize or deserialize JSON.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// An opaque HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl OpencodeError {
    // ── Query helpers ──────────────────────────────────────────────

    /// Returns the HTTP status code if this is an `Api` variant.
    pub const fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Whether this error should be retried.
    ///
    /// Mirrors the JS SDK logic:
    /// - Status 408, 409, 429, >= 500 → retryable
    /// - Connection errors and timeouts → retryable
    /// - Everything else → not retryable
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::Api { status, .. } => matches!(*status, 408 | 409 | 429) || *status >= 500,
            Self::Connection { .. } | Self::Timeout => true,
            Self::UserAbort | Self::Serialization(_) | Self::Http(_) => false,
        }
    }

    /// Whether this error represents a timeout.
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }

    // ── Convenience constructors for specific HTTP statuses ─────────

    /// 400 Bad Request
    pub fn bad_request(
        headers: Option<HeaderMap>,
        body: Option<Value>,
        message: impl Into<String>,
    ) -> Self {
        Self::Api {
            status: 400,
            headers: headers.map(Box::new),
            body: body.map(Box::new),
            message: message.into(),
        }
    }

    /// 401 Authentication Error
    pub fn authentication(
        headers: Option<HeaderMap>,
        body: Option<Value>,
        message: impl Into<String>,
    ) -> Self {
        Self::Api {
            status: 401,
            headers: headers.map(Box::new),
            body: body.map(Box::new),
            message: message.into(),
        }
    }

    /// 403 Permission Denied
    pub fn permission_denied(
        headers: Option<HeaderMap>,
        body: Option<Value>,
        message: impl Into<String>,
    ) -> Self {
        Self::Api {
            status: 403,
            headers: headers.map(Box::new),
            body: body.map(Box::new),
            message: message.into(),
        }
    }

    /// 404 Not Found
    pub fn not_found(
        headers: Option<HeaderMap>,
        body: Option<Value>,
        message: impl Into<String>,
    ) -> Self {
        Self::Api {
            status: 404,
            headers: headers.map(Box::new),
            body: body.map(Box::new),
            message: message.into(),
        }
    }

    /// 409 Conflict
    pub fn conflict(
        headers: Option<HeaderMap>,
        body: Option<Value>,
        message: impl Into<String>,
    ) -> Self {
        Self::Api {
            status: 409,
            headers: headers.map(Box::new),
            body: body.map(Box::new),
            message: message.into(),
        }
    }

    /// 422 Unprocessable Entity
    pub fn unprocessable_entity(
        headers: Option<HeaderMap>,
        body: Option<Value>,
        message: impl Into<String>,
    ) -> Self {
        Self::Api {
            status: 422,
            headers: headers.map(Box::new),
            body: body.map(Box::new),
            message: message.into(),
        }
    }

    /// 429 Rate Limit
    pub fn rate_limit(
        headers: Option<HeaderMap>,
        body: Option<Value>,
        message: impl Into<String>,
    ) -> Self {
        Self::Api {
            status: 429,
            headers: headers.map(Box::new),
            body: body.map(Box::new),
            message: message.into(),
        }
    }

    /// 5xx Internal Server Error
    pub fn internal_server(
        status: u16,
        headers: Option<HeaderMap>,
        body: Option<Value>,
        message: impl Into<String>,
    ) -> Self {
        debug_assert!(status >= 500, "internal_server expects status >= 500");
        Self::Api {
            status,
            headers: headers.map(Box::new),
            body: body.map(Box::new),
            message: message.into(),
        }
    }

    // ── Factory ────────────────────────────────────────────────────

    /// Create an error from an HTTP response's status, headers, and body.
    ///
    /// Maps well-known status codes to their specific constructors; falls
    /// through to a generic `Api` variant for other codes.
    pub fn from_response(status: u16, headers: Option<HeaderMap>, body: Option<Value>) -> Self {
        let message =
            body.as_ref().and_then(|b| b.get("message")).and_then(|m| m.as_str()).map_or_else(
                || {
                    body.as_ref().map_or_else(
                        || format!("{status} status code (no body)"),
                        std::string::ToString::to_string,
                    )
                },
                String::from,
            );

        match status {
            400 => Self::bad_request(headers, body, message),
            401 => Self::authentication(headers, body, message),
            403 => Self::permission_denied(headers, body, message),
            404 => Self::not_found(headers, body, message),
            409 => Self::conflict(headers, body, message),
            422 => Self::unprocessable_entity(headers, body, message),
            429 => Self::rate_limit(headers, body, message),
            s if s >= 500 => Self::internal_server(status, headers, body, message),
            _ => Self::Api {
                status,
                headers: headers.map(Box::new),
                body: body.map(Box::new),
                message,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // ── Display ────────────────────────────────────────────────────

    #[test]
    fn display_api_error() {
        let err = OpencodeError::Api {
            status: 500,
            headers: None,
            body: None,
            message: "Internal Server Error".into(),
        };
        assert_eq!(err.to_string(), "500 Internal Server Error");
    }

    #[test]
    fn display_connection_error() {
        let err = OpencodeError::Connection { message: "DNS lookup failed".into(), source: None };
        assert_eq!(err.to_string(), "Connection error: DNS lookup failed");
    }

    #[test]
    fn display_timeout() {
        assert_eq!(OpencodeError::Timeout.to_string(), "Request timed out.");
    }

    #[test]
    fn display_user_abort() {
        assert_eq!(OpencodeError::UserAbort.to_string(), "Request was aborted.");
    }

    #[test]
    fn display_serialization() {
        let raw = serde_json::from_str::<Value>("not json").unwrap_err();
        let err = OpencodeError::Serialization(raw);
        assert!(err.to_string().starts_with("Serialization error:"));
    }

    #[test]
    fn display_http() {
        let inner: Box<dyn std::error::Error + Send + Sync> = "transport broke".into();
        let err = OpencodeError::Http(inner);
        assert_eq!(err.to_string(), "HTTP error: transport broke");
    }

    // ── status() ───────────────────────────────────────────────────

    #[test]
    fn status_returns_code_for_api() {
        let err = OpencodeError::bad_request(None, None, "bad");
        assert_eq!(err.status(), Some(400));
    }

    #[test]
    fn status_returns_none_for_non_api() {
        assert_eq!(OpencodeError::Timeout.status(), None);
        assert_eq!(OpencodeError::UserAbort.status(), None);
        let conn = OpencodeError::Connection { message: "x".into(), source: None };
        assert_eq!(conn.status(), None);
    }

    // ── is_retryable() ─────────────────────────────────────────────

    #[test]
    fn retryable_status_codes() {
        // Retryable HTTP statuses
        for code in [408, 409, 429, 500, 502, 503, 504] {
            let err =
                OpencodeError::Api { status: code, headers: None, body: None, message: "x".into() };
            assert!(err.is_retryable(), "status {code} should be retryable");
        }
    }

    #[test]
    fn non_retryable_status_codes() {
        for code in [400, 401, 403, 404, 422] {
            let err =
                OpencodeError::Api { status: code, headers: None, body: None, message: "x".into() };
            assert!(!err.is_retryable(), "status {code} should NOT be retryable");
        }
    }

    #[test]
    fn connection_and_timeout_are_retryable() {
        let conn = OpencodeError::Connection { message: "fail".into(), source: None };
        assert!(conn.is_retryable());
        assert!(OpencodeError::Timeout.is_retryable());
    }

    #[test]
    fn user_abort_not_retryable() {
        assert!(!OpencodeError::UserAbort.is_retryable());
    }

    #[test]
    fn http_and_serialization_not_retryable() {
        let inner: Box<dyn std::error::Error + Send + Sync> = "oops".into();
        assert!(!OpencodeError::Http(inner).is_retryable());

        let raw = serde_json::from_str::<Value>("bad").unwrap_err();
        assert!(!OpencodeError::Serialization(raw).is_retryable());
    }

    // ── is_timeout() ───────────────────────────────────────────────

    #[test]
    fn is_timeout_only_for_timeout() {
        assert!(OpencodeError::Timeout.is_timeout());
        assert!(!OpencodeError::UserAbort.is_timeout());
        let api = OpencodeError::bad_request(None, None, "x");
        assert!(!api.is_timeout());
    }

    // ── Convenience constructors ───────────────────────────────────

    #[test]
    fn convenience_constructors_set_correct_status() {
        assert_eq!(OpencodeError::bad_request(None, None, "x").status(), Some(400));
        assert_eq!(OpencodeError::authentication(None, None, "x").status(), Some(401));
        assert_eq!(OpencodeError::permission_denied(None, None, "x").status(), Some(403));
        assert_eq!(OpencodeError::not_found(None, None, "x").status(), Some(404));
        assert_eq!(OpencodeError::conflict(None, None, "x").status(), Some(409));
        assert_eq!(OpencodeError::unprocessable_entity(None, None, "x").status(), Some(422));
        assert_eq!(OpencodeError::rate_limit(None, None, "x").status(), Some(429));
        assert_eq!(OpencodeError::internal_server(500, None, None, "x").status(), Some(500));
        assert_eq!(OpencodeError::internal_server(503, None, None, "x").status(), Some(503));
    }

    // ── from_response() ────────────────────────────────────────────

    #[test]
    fn from_response_maps_known_status_codes() {
        let cases: &[(u16, &str)] = &[
            (400, "400"),
            (401, "401"),
            (403, "403"),
            (404, "404"),
            (409, "409"),
            (422, "422"),
            (429, "429"),
            (500, "500"),
            (502, "502"),
        ];
        for &(code, prefix) in cases {
            let err = OpencodeError::from_response(code, None, None);
            assert_eq!(err.status(), Some(code), "from_response({code}) status mismatch");
            assert!(
                err.to_string().starts_with(prefix),
                "from_response({code}) display should start with {prefix}, got: {}",
                err.to_string()
            );
        }
    }

    #[test]
    fn from_response_extracts_message_from_body() {
        let body = json!({"message": "quota exceeded"});
        let err = OpencodeError::from_response(429, None, Some(body));
        assert_eq!(err.to_string(), "429 quota exceeded");
    }

    #[test]
    fn from_response_falls_back_to_json_body() {
        let body = json!({"error": "oops"});
        let err = OpencodeError::from_response(400, None, Some(body.clone()));
        // No "message" key → falls back to JSON stringification
        assert!(err.to_string().contains("oops"));
    }

    #[test]
    fn from_response_unknown_status_creates_generic_api() {
        let err = OpencodeError::from_response(418, None, None);
        assert_eq!(err.status(), Some(418));
        assert!(err.to_string().contains("418"));
    }

    #[test]
    fn from_response_preserves_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", "abc123".parse().unwrap());
        let err = OpencodeError::from_response(500, Some(headers), None);
        if let OpencodeError::Api { headers: Some(h), .. } = &err {
            assert_eq!(h.get("x-request-id").unwrap(), "abc123");
        } else {
            panic!("expected Api variant with headers");
        }
    }
}
