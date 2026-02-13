use std::{collections::HashMap, time::Duration};

use http::HeaderMap;

/// Default base URL matching the JS SDK.
pub const DEFAULT_BASE_URL: &str = "http://localhost:54321";

/// Default request timeout (60 seconds), matching the JS SDK.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_mins(1);

/// Default maximum number of retries, matching the JS SDK.
pub const DEFAULT_MAX_RETRIES: u32 = 2;

/// Environment variable name for the base URL override.
pub const ENV_BASE_URL: &str = "OPENCODE_BASE_URL";

/// Configuration options for building an [`crate::client::Opencode`] client.
///
/// All fields are optional; unset fields fall back to defaults that match the
/// JS SDK behaviour.  The `Default` implementation reads `OPENCODE_BASE_URL`
/// from the environment when available.
#[derive(Debug, Clone)]
pub struct ClientOptions {
    /// Base URL of the `OpenCode` server.
    pub base_url: Option<String>,

    /// Per-request timeout.
    pub timeout: Option<Duration>,

    /// Maximum number of automatic retries for retryable errors.
    pub max_retries: Option<u32>,

    /// Headers sent with every request.
    pub default_headers: Option<HeaderMap>,

    /// Query parameters appended to every request URL.
    pub default_query: Option<HashMap<String, String>>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        let base_url = std::env::var(ENV_BASE_URL).ok();

        Self {
            base_url,
            timeout: None,
            max_retries: None,
            default_headers: None,
            default_query: None,
        }
    }
}

impl ClientOptions {
    /// Create a new `ClientOptions` with all fields set to `None` (no env
    /// lookup).  Useful when you want to set every field explicitly.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            base_url: None,
            timeout: None,
            max_retries: None,
            default_headers: None,
            default_query: None,
        }
    }

    /// Resolve each option to its concrete value, falling back to the
    /// compile-time defaults.
    #[must_use]
    pub(crate) fn resolve_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL)
    }

    #[must_use]
    pub(crate) fn resolve_timeout(&self) -> Duration {
        self.timeout.unwrap_or(DEFAULT_TIMEOUT)
    }

    #[must_use]
    pub(crate) fn resolve_max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(DEFAULT_MAX_RETRIES)
    }

    #[must_use]
    pub(crate) fn resolve_default_headers(&self) -> HeaderMap {
        self.default_headers.clone().unwrap_or_default()
    }

    #[must_use]
    pub(crate) fn resolve_default_query(&self) -> HashMap<String, String> {
        self.default_query.clone().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_has_all_none() {
        let opts = ClientOptions::empty();
        assert!(opts.base_url.is_none());
        assert!(opts.timeout.is_none());
        assert!(opts.max_retries.is_none());
        assert!(opts.default_headers.is_none());
        assert!(opts.default_query.is_none());
    }

    #[test]
    fn resolve_falls_back_to_defaults() {
        let opts = ClientOptions::empty();
        assert_eq!(opts.resolve_base_url(), DEFAULT_BASE_URL);
        assert_eq!(opts.resolve_timeout(), DEFAULT_TIMEOUT);
        assert_eq!(opts.resolve_max_retries(), DEFAULT_MAX_RETRIES);
        assert!(opts.resolve_default_headers().is_empty());
        assert!(opts.resolve_default_query().is_empty());
    }

    #[test]
    fn resolve_uses_provided_values() {
        let opts = ClientOptions {
            base_url: Some("http://example.com".to_owned()),
            timeout: Some(Duration::from_secs(30)),
            max_retries: Some(5),
            default_headers: None,
            default_query: None,
        };
        assert_eq!(opts.resolve_base_url(), "http://example.com");
        assert_eq!(opts.resolve_timeout(), Duration::from_secs(30));
        assert_eq!(opts.resolve_max_retries(), 5);
    }
}
