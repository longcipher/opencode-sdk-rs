use std::{collections::HashMap, time::Duration};

use http::{HeaderMap, header::HeaderValue};
use serde::{Serialize, de::DeserializeOwned};

use crate::{config::ClientOptions, error::OpencodeError, resources::app::AppResource};

/// SDK version from `Cargo.toml`, used in the `User-Agent` header.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Per-request option overrides.
///
/// All fields are optional; unset fields fall back to the client-level
/// defaults configured via [`Opencode::builder`] or [`ClientOptions`].
#[derive(Debug, Default, Clone)]
pub struct RequestOptions {
    /// Extra headers to send with this request only.
    pub extra_headers: Option<HeaderMap>,
    /// Override the per-request timeout.
    pub timeout: Option<Duration>,
    /// Override the maximum number of retries.
    pub max_retries: Option<u32>,
}

/// The main `OpenCode` SDK client.
///
/// Holds connection settings and an inner HTTP client.  Construct via
/// [`Opencode::new`], [`Opencode::with_options`], or [`Opencode::builder`].
#[derive(Clone)]
pub struct Opencode {
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    default_headers: HeaderMap,
    default_query: HashMap<String, String>,
    pub(crate) http_client: hpx::Client,
}

impl std::fmt::Debug for Opencode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Opencode")
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .field("default_headers", &self.default_headers)
            .field("default_query", &self.default_query)
            .field("http_client", &"hpx::Client { .. }")
            .finish()
    }
}

impl Opencode {
    /// Create a client with default configuration.
    ///
    /// Reads `OPENCODE_BASE_URL` from the environment; all other settings use
    /// the JS SDK defaults (timeout = 60 s, `max_retries` = 2).
    ///
    /// # Errors
    ///
    /// Returns [`OpencodeError::Http`] if the underlying HTTP client cannot be
    /// built (e.g. TLS back-end init failure).
    pub fn new() -> Result<Self, OpencodeError> {
        Self::with_options(&ClientOptions::default())
    }

    /// Create a client from explicit [`ClientOptions`].
    ///
    /// # Errors
    ///
    /// Returns [`OpencodeError::Http`] if the underlying HTTP client cannot be
    /// built.
    pub fn with_options(opts: &ClientOptions) -> Result<Self, OpencodeError> {
        let timeout = opts.resolve_timeout();
        let default_headers = opts.resolve_default_headers();

        let http_client = hpx::Client::builder()
            .timeout(timeout)
            .default_headers(default_headers.clone())
            .build()
            .map_err(|e| OpencodeError::Http(Box::new(e)))?;

        Ok(Self {
            base_url: opts.resolve_base_url().to_owned(),
            timeout,
            max_retries: opts.resolve_max_retries(),
            default_headers,
            default_query: opts.resolve_default_query(),
            http_client,
        })
    }

    /// Return an [`OpencodeBuilder`] for fluent configuration.
    #[must_use]
    pub fn builder() -> OpencodeBuilder {
        OpencodeBuilder { options: ClientOptions::default() }
    }

    // ── Getters ────────────────────────────────────────────────────

    /// The resolved base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The per-request timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Maximum automatic retries.
    #[must_use]
    pub const fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Default headers sent with every request.
    #[must_use]
    pub const fn default_headers(&self) -> &HeaderMap {
        &self.default_headers
    }

    /// Default query parameters appended to every request.
    #[must_use]
    pub const fn default_query(&self) -> &HashMap<String, String> {
        &self.default_query
    }

    // ── Resource accessors ─────────────────────────────────────

    /// Access the App resource.
    pub const fn app(&self) -> AppResource<'_> {
        AppResource::new(self)
    }

    /// Access the Config resource.
    pub const fn config(&self) -> crate::resources::config::ConfigResource<'_> {
        crate::resources::config::ConfigResource::new(self)
    }

    /// Access the Event resource.
    pub const fn event(&self) -> crate::resources::event::EventResource<'_> {
        crate::resources::event::EventResource::new(self)
    }

    /// Access the File resource.
    pub const fn file(&self) -> crate::resources::file::FileResource<'_> {
        crate::resources::file::FileResource::new(self)
    }

    /// Access the Find resource.
    pub const fn find(&self) -> crate::resources::find::FindResource<'_> {
        crate::resources::find::FindResource::new(self)
    }

    /// Access the Session resource.
    pub const fn session(&self) -> crate::resources::session::SessionResource<'_> {
        crate::resources::session::SessionResource::new(self)
    }

    /// Access the Tui resource.
    pub const fn tui(&self) -> crate::resources::tui::TuiResource<'_> {
        crate::resources::tui::TuiResource::new(self)
    }

    // ── URL & Header Building ──────────────────────────────────

    /// Build a full URL by joining `base_url` + `path`, then appending
    /// `default_query` and any extra `query` parameters.
    ///
    /// Query keys are sorted for deterministic output.
    pub(crate) fn build_url(&self, path: &str, query: Option<&HashMap<String, String>>) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path_part = if path.starts_with('/') { path.to_owned() } else { format!("/{path}") };

        let mut params: Vec<(&str, &str)> =
            self.default_query.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

        if let Some(q) = query {
            params.extend(q.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        }

        if params.is_empty() {
            format!("{base}{path_part}")
        } else {
            params.sort_by_key(|(k, _)| *k);
            let qs = params.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&");
            format!("{base}{path_part}?{qs}")
        }
    }

    /// Build request headers: default headers + `Accept` + `User-Agent` +
    /// extras.
    ///
    /// Includes `x-retry-count` when `retry_count > 0`.
    pub(crate) fn build_headers(
        &self,
        extra_headers: Option<&HeaderMap>,
        retry_count: u32,
    ) -> HeaderMap {
        let mut headers = self.default_headers.clone();

        headers.insert(http::header::ACCEPT, HeaderValue::from_static("application/json"));

        if let Ok(ua) = HeaderValue::from_str(&format!("opencode-sdk-rs/{VERSION}")) {
            headers.insert(http::header::USER_AGENT, ua);
        }

        if retry_count > 0 &&
            let Ok(val) = HeaderValue::from_str(&retry_count.to_string())
        {
            headers.insert("x-retry-count", val);
        }

        if let Some(extra) = extra_headers {
            for (key, value) in extra {
                headers.insert(key, value.clone());
            }
        }

        headers
    }

    // ── Internal request engine ────────────────────────────────

    /// Send an HTTP request with automatic retries and error mapping.
    ///
    /// The caller supplies a pre-serialised `body` (as [`serde_json::Value`])
    /// and an optional serialisable `query` struct.  On success the JSON
    /// response is deserialised into `T`.
    async fn make_request<T, Q>(
        &self,
        method: http::Method,
        path: &str,
        body: Option<serde_json::Value>,
        query: Option<&Q>,
        options: Option<&RequestOptions>,
    ) -> Result<T, OpencodeError>
    where
        T: DeserializeOwned,
        Q: Serialize + Sync + ?Sized,
    {
        let url = self.build_url(path, None);
        let max_retries = options.and_then(|o| o.max_retries).unwrap_or(self.max_retries);
        let timeout = options.and_then(|o| o.timeout).unwrap_or(self.timeout);
        let extra_headers = options.and_then(|o| o.extra_headers.as_ref());

        let mut last_error: Option<OpencodeError> = None;

        for attempt in 0..=max_retries {
            let headers = self.build_headers(extra_headers, attempt);

            tracing::debug!(
                method = %method,
                url = %url,
                attempt,
                "sending request"
            );

            let mut req =
                self.http_client.request(method.clone(), &url).headers(headers).timeout(timeout);

            if let Some(q) = query {
                req = req.query(q);
            }

            if let Some(ref b) = body {
                req = req.json(b);
            }

            let result = req.send().await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    let resp_headers = resp.headers().clone();

                    if status.is_success() {
                        let bytes =
                            resp.bytes().await.map_err(|e| OpencodeError::Http(Box::new(e)))?;
                        let parsed: T = serde_json::from_slice(&bytes)?;
                        return Ok(parsed);
                    }

                    // Error response — read body then decide to retry or fail.
                    let body_bytes = resp.bytes().await.ok();
                    let body_value: Option<serde_json::Value> =
                        body_bytes.as_ref().and_then(|b| serde_json::from_slice(b).ok());

                    let err = OpencodeError::from_response(
                        status.as_u16(),
                        Some(resp_headers.clone()),
                        body_value,
                    );

                    if attempt < max_retries && should_retry(&err, &resp_headers) {
                        let delay = retry_delay(attempt, &resp_headers);
                        tracing::debug!(
                            attempt,
                            delay_ms = delay.as_millis() as u64,
                            "retrying after error"
                        );
                        tokio::time::sleep(delay).await;
                        last_error = Some(err);
                        continue;
                    }

                    return Err(err);
                }
                Err(send_err) => {
                    let err = classify_transport_error(send_err);

                    if attempt < max_retries && err.is_retryable() {
                        let delay = retry_delay(attempt, &HeaderMap::new());
                        tracing::debug!(
                            attempt,
                            delay_ms = delay.as_millis() as u64,
                            "retrying after transport error"
                        );
                        tokio::time::sleep(delay).await;
                        last_error = Some(err);
                        continue;
                    }

                    return Err(err);
                }
            }
        }

        // Should be unreachable given the loop guarantees, but handle
        // gracefully.
        Err(last_error
            .unwrap_or_else(|| OpencodeError::Http("max retries exhausted".to_owned().into())))
    }

    // ── Public convenience methods ─────────────────────────────

    /// Send a `GET` request and deserialise the JSON response.
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        options: Option<&RequestOptions>,
    ) -> Result<T, OpencodeError> {
        self.make_request::<T, ()>(http::Method::GET, path, None, None, options).await
    }

    /// Send a `GET` request with query parameters.
    pub async fn get_with_query<T, Q>(
        &self,
        path: &str,
        query: Option<&Q>,
        options: Option<&RequestOptions>,
    ) -> Result<T, OpencodeError>
    where
        T: DeserializeOwned,
        Q: Serialize + Sync + ?Sized,
    {
        self.make_request(http::Method::GET, path, None, query, options).await
    }

    /// Send a `POST` request with an optional JSON body.
    pub async fn post<T, B>(
        &self,
        path: &str,
        body: Option<&B>,
        options: Option<&RequestOptions>,
    ) -> Result<T, OpencodeError>
    where
        T: DeserializeOwned,
        B: Serialize + Sync,
    {
        let body_value = body.map(serde_json::to_value).transpose()?;
        self.make_request::<T, ()>(http::Method::POST, path, body_value, None, options).await
    }

    /// Send a `PUT` request with an optional JSON body.
    pub async fn put<T, B>(
        &self,
        path: &str,
        body: Option<&B>,
        options: Option<&RequestOptions>,
    ) -> Result<T, OpencodeError>
    where
        T: DeserializeOwned,
        B: Serialize + Sync,
    {
        let body_value = body.map(serde_json::to_value).transpose()?;
        self.make_request::<T, ()>(http::Method::PUT, path, body_value, None, options).await
    }

    /// Send a `PATCH` request with an optional JSON body.
    pub async fn patch<T, B>(
        &self,
        path: &str,
        body: Option<&B>,
        options: Option<&RequestOptions>,
    ) -> Result<T, OpencodeError>
    where
        T: DeserializeOwned,
        B: Serialize + Sync,
    {
        let body_value = body.map(serde_json::to_value).transpose()?;
        self.make_request::<T, ()>(http::Method::PATCH, path, body_value, None, options).await
    }

    /// Send a GET request and return a streaming SSE response.
    ///
    /// Unlike other HTTP methods, this does NOT parse the full response body.
    /// Instead it returns an [`crate::SseStream`] that lazily decodes each SSE
    /// event's `data` field as JSON of type `T`.
    pub async fn get_stream<T: DeserializeOwned + 'static>(
        &self,
        path: &str,
    ) -> Result<crate::streaming::SseStream<T>, OpencodeError> {
        let url = self.build_url(path, None);
        let headers = self.build_headers(None, 0);

        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(classify_transport_error)?;

        let status = response.status();
        if !status.is_success() {
            let resp_headers = response.headers().clone();
            let body_bytes = response.bytes().await.ok();
            let body_value: Option<serde_json::Value> =
                body_bytes.as_ref().and_then(|b| serde_json::from_slice(b).ok());
            return Err(OpencodeError::from_response(
                status.as_u16(),
                Some(resp_headers),
                body_value,
            ));
        }

        Ok(crate::streaming::SseStream::new(response.bytes_stream()))
    }

    /// Send a `DELETE` request with an optional JSON body.
    pub async fn delete<T, B>(
        &self,
        path: &str,
        body: Option<&B>,
        options: Option<&RequestOptions>,
    ) -> Result<T, OpencodeError>
    where
        T: DeserializeOwned,
        B: Serialize + Sync,
    {
        let body_value = body.map(serde_json::to_value).transpose()?;
        self.make_request::<T, ()>(http::Method::DELETE, path, body_value, None, options).await
    }
}

// ── Free helper functions ──────────────────────────────────────────

/// Decide whether a request should be retried, honouring `x-should-retry`.
fn should_retry(err: &OpencodeError, headers: &HeaderMap) -> bool {
    if let Some(val) = headers.get("x-should-retry") &&
        let Ok(s) = val.to_str()
    {
        match s {
            "true" => return true,
            "false" => return false,
            _ => {}
        }
    }
    err.is_retryable()
}

/// Calculate the retry delay from response headers or exponential backoff.
///
/// Checks `retry-after-ms`, then `retry-after` (seconds), then falls back
/// to `min(0.5 * 2^attempt, 8) * jitter`.
fn retry_delay(attempt: u32, headers: &HeaderMap) -> Duration {
    // Prefer explicit server hints.
    if let Some(ms) = header_u64(headers, "retry-after-ms") {
        return Duration::from_millis(ms);
    }

    if let Some(val) = headers.get("retry-after") &&
        let Ok(s) = val.to_str() &&
        let Ok(secs) = s.parse::<f64>()
    {
        return Duration::from_secs_f64(secs);
    }

    // Exponential back-off with jitter.
    let base = (0.5 * 2.0_f64.powi(attempt.cast_signed())).min(8.0);
    Duration::from_secs_f64(base * jitter_factor())
}

/// Try to parse a header value as `u64`.
fn header_u64(headers: &HeaderMap, name: &str) -> Option<u64> {
    headers.get(name)?.to_str().ok()?.parse().ok()
}

/// Generate a jitter factor in `[0.75, 1.0)` using system-clock entropy.
fn jitter_factor() -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (f64::from(nanos % 1000) / 1000.0).mul_add(-0.25, 1.0)
}

/// Map an `hpx` transport error to the appropriate [`OpencodeError`] variant.
fn classify_transport_error(err: hpx::Error) -> OpencodeError {
    if err.is_timeout() {
        OpencodeError::Timeout
    } else if err.is_connect() {
        OpencodeError::Connection { message: err.to_string(), source: Some(Box::new(err)) }
    } else {
        OpencodeError::Http(Box::new(err))
    }
}

/// Fluent builder for [`Opencode`].
#[derive(Debug)]
pub struct OpencodeBuilder {
    options: ClientOptions,
}

impl OpencodeBuilder {
    /// Override the base URL.
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.options.base_url = Some(url.into());
        self
    }

    /// Override the per-request timeout.
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.options.timeout = Some(timeout);
        self
    }

    /// Override the maximum number of retries.
    #[must_use]
    pub const fn max_retries(mut self, retries: u32) -> Self {
        self.options.max_retries = Some(retries);
        self
    }

    /// Set default headers for every request.
    #[must_use]
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.options.default_headers = Some(headers);
        self
    }

    /// Set default query parameters for every request.
    #[must_use]
    pub fn default_query(mut self, query: HashMap<String, String>) -> Self {
        self.options.default_query = Some(query);
        self
    }

    /// Build the [`Opencode`] client.
    ///
    /// # Errors
    ///
    /// Returns [`OpencodeError::Http`] if the underlying HTTP client cannot be
    /// built.
    pub fn build(self) -> Result<Opencode, OpencodeError> {
        Opencode::with_options(&self.options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_BASE_URL, DEFAULT_MAX_RETRIES, DEFAULT_TIMEOUT};

    // ── Helper ─────────────────────────────────────────────────

    /// Build a client with known defaults and no env-var side-effects.
    fn test_client() -> Opencode {
        Opencode::with_options(&ClientOptions::empty()).expect("test client")
    }

    fn test_client_with_defaults(
        base: &str,
        dq: HashMap<String, String>,
        dh: HeaderMap,
    ) -> Opencode {
        Opencode::with_options(&ClientOptions {
            base_url: Some(base.to_owned()),
            timeout: None,
            max_retries: None,
            default_headers: Some(dh),
            default_query: Some(dq),
        })
        .expect("test client")
    }

    // ── Constructor / builder tests (existing) ─────────────────

    #[test]
    fn with_empty_options_uses_defaults() {
        let client = Opencode::with_options(&ClientOptions::empty()).expect("client");
        assert_eq!(client.base_url(), DEFAULT_BASE_URL);
        assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
        assert_eq!(client.max_retries(), DEFAULT_MAX_RETRIES);
        assert!(client.default_headers().is_empty());
        assert!(client.default_query().is_empty());
    }

    #[test]
    fn with_options_custom() {
        let opts = ClientOptions {
            base_url: Some("http://myhost:8080".to_owned()),
            timeout: Some(Duration::from_secs(10)),
            max_retries: Some(5),
            default_headers: None,
            default_query: None,
        };
        let client = Opencode::with_options(&opts).expect("client");
        assert_eq!(client.base_url(), "http://myhost:8080");
        assert_eq!(client.timeout(), Duration::from_secs(10));
        assert_eq!(client.max_retries(), 5);
    }

    #[test]
    fn builder_overrides() {
        let client = Opencode::builder()
            .base_url("http://builder:1234")
            .timeout(Duration::from_secs(15))
            .max_retries(0)
            .build()
            .expect("client");

        assert_eq!(client.base_url(), "http://builder:1234");
        assert_eq!(client.timeout(), Duration::from_secs(15));
        assert_eq!(client.max_retries(), 0);
    }

    #[test]
    fn builder_with_explicit_empty_falls_back() {
        let client = Opencode::with_options(&ClientOptions::empty()).expect("client");
        assert_eq!(client.base_url(), DEFAULT_BASE_URL);
        assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
        assert_eq!(client.max_retries(), DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn builder_base_url_overrides_option() {
        let client = Opencode::builder().base_url("http://explicit:2222").build().expect("client");
        assert_eq!(client.base_url(), "http://explicit:2222");
    }

    // ── build_url tests ────────────────────────────────────────

    #[test]
    fn build_url_simple_path() {
        let client = test_client();
        let url = client.build_url("/app", None);
        assert_eq!(url, format!("{DEFAULT_BASE_URL}/app"));
    }

    #[test]
    fn build_url_strips_trailing_slash_from_base() {
        let client =
            test_client_with_defaults("http://example.com/", HashMap::new(), HeaderMap::new());
        assert_eq!(client.build_url("/path", None), "http://example.com/path");
    }

    #[test]
    fn build_url_adds_leading_slash() {
        let client = test_client();
        let url = client.build_url("session", None);
        assert_eq!(url, format!("{DEFAULT_BASE_URL}/session"));
    }

    #[test]
    fn build_url_with_default_query() {
        let mut dq = HashMap::new();
        dq.insert("version".to_owned(), "2".to_owned());
        let client = test_client_with_defaults("http://host", dq, HeaderMap::new());

        let url = client.build_url("/api", None);
        assert_eq!(url, "http://host/api?version=2");
    }

    #[test]
    fn build_url_with_extra_query() {
        let client = test_client_with_defaults("http://host", HashMap::new(), HeaderMap::new());

        let mut extra = HashMap::new();
        extra.insert("foo".to_owned(), "bar".to_owned());

        let url = client.build_url("/api", Some(&extra));
        assert_eq!(url, "http://host/api?foo=bar");
    }

    #[test]
    fn build_url_merges_default_and_extra_query() {
        let mut dq = HashMap::new();
        dq.insert("a".to_owned(), "1".to_owned());

        let client = test_client_with_defaults("http://host", dq, HeaderMap::new());

        let mut extra = HashMap::new();
        extra.insert("b".to_owned(), "2".to_owned());

        let url = client.build_url("/x", Some(&extra));
        // Sorted by key
        assert_eq!(url, "http://host/x?a=1&b=2");
    }

    #[test]
    fn build_url_no_query_no_question_mark() {
        let client = test_client();
        let url = client.build_url("/clean", None);
        assert!(!url.contains('?'));
    }

    // ── build_headers tests ────────────────────────────────────

    #[test]
    fn build_headers_sets_accept_json() {
        let client = test_client();
        let headers = client.build_headers(None, 0);
        assert_eq!(
            headers.get(http::header::ACCEPT).map(|v| v.to_str().ok()),
            Some(Some("application/json"))
        );
    }

    #[test]
    fn build_headers_sets_user_agent() {
        let client = test_client();
        let headers = client.build_headers(None, 0);
        let ua =
            headers.get(http::header::USER_AGENT).expect("user-agent").to_str().expect("ascii");
        assert!(ua.starts_with("opencode-sdk-rs/"), "unexpected user-agent: {ua}");
    }

    #[test]
    fn build_headers_no_retry_count_on_first_attempt() {
        let client = test_client();
        let headers = client.build_headers(None, 0);
        assert!(headers.get("x-retry-count").is_none());
    }

    #[test]
    fn build_headers_includes_retry_count() {
        let client = test_client();
        let headers = client.build_headers(None, 3);
        assert_eq!(headers.get("x-retry-count").map(|v| v.to_str().ok()), Some(Some("3")));
    }

    #[test]
    fn build_headers_merges_extra() {
        let client = test_client();
        let mut extra = HeaderMap::new();
        extra.insert("x-custom", HeaderValue::from_static("yes"));

        let headers = client.build_headers(Some(&extra), 0);
        assert_eq!(headers.get("x-custom").map(|v| v.to_str().ok()), Some(Some("yes")));
        // Standard headers still present
        assert!(headers.get(http::header::ACCEPT).is_some());
    }

    #[test]
    fn build_headers_includes_default_headers() {
        let mut dh = HeaderMap::new();
        dh.insert("x-default", HeaderValue::from_static("value"));

        let client = test_client_with_defaults(DEFAULT_BASE_URL, HashMap::new(), dh);
        let headers = client.build_headers(None, 0);
        assert_eq!(headers.get("x-default").map(|v| v.to_str().ok()), Some(Some("value")));
    }

    #[test]
    fn build_headers_extra_overrides_default() {
        let mut dh = HeaderMap::new();
        dh.insert("x-key", HeaderValue::from_static("default"));

        let client = test_client_with_defaults(DEFAULT_BASE_URL, HashMap::new(), dh);

        let mut extra = HeaderMap::new();
        extra.insert("x-key", HeaderValue::from_static("override"));

        let headers = client.build_headers(Some(&extra), 0);
        assert_eq!(headers.get("x-key").map(|v| v.to_str().ok()), Some(Some("override")));
    }

    // ── should_retry tests ─────────────────────────────────────

    #[test]
    fn should_retry_honours_x_should_retry_true() {
        let err = OpencodeError::bad_request(None, None, "nope");
        let mut headers = HeaderMap::new();
        headers.insert("x-should-retry", HeaderValue::from_static("true"));
        assert!(should_retry(&err, &headers));
    }

    #[test]
    fn should_retry_honours_x_should_retry_false() {
        let err = OpencodeError::internal_server(500, None, None, "fail");
        let mut headers = HeaderMap::new();
        headers.insert("x-should-retry", HeaderValue::from_static("false"));
        assert!(!should_retry(&err, &headers));
    }

    #[test]
    fn should_retry_falls_back_to_is_retryable() {
        let retryable = OpencodeError::rate_limit(None, None, "slow down");
        assert!(should_retry(&retryable, &HeaderMap::new()));

        let not_retryable = OpencodeError::not_found(None, None, "gone");
        assert!(!should_retry(&not_retryable, &HeaderMap::new()));
    }

    // ── retry_delay tests ──────────────────────────────────────

    #[test]
    fn retry_delay_uses_retry_after_ms() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after-ms", HeaderValue::from_static("1500"));
        let delay = retry_delay(0, &headers);
        assert_eq!(delay, Duration::from_millis(1500));
    }

    #[test]
    fn retry_delay_uses_retry_after_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", HeaderValue::from_static("2"));
        let delay = retry_delay(0, &headers);
        assert_eq!(delay, Duration::from_secs(2));
    }

    #[test]
    fn retry_delay_exponential_backoff_attempt_0() {
        // base = min(0.5 * 2^0, 8) = 0.5 → * jitter [0.75, 1.0)
        let delay = retry_delay(0, &HeaderMap::new());
        let secs = delay.as_secs_f64();
        assert!((0.375..=0.5).contains(&secs), "attempt 0 delay {secs}s out of range");
    }

    #[test]
    fn retry_delay_exponential_backoff_attempt_4() {
        // base = min(0.5 * 2^4, 8) = min(8, 8) = 8 → * jitter [0.75, 1.0)
        let delay = retry_delay(4, &HeaderMap::new());
        let secs = delay.as_secs_f64();
        assert!((6.0..=8.0).contains(&secs), "attempt 4 delay {secs}s out of range");
    }

    #[test]
    fn retry_delay_caps_at_8_seconds() {
        // base = min(0.5 * 2^10, 8) = 8 → * jitter
        let delay = retry_delay(10, &HeaderMap::new());
        let secs = delay.as_secs_f64();
        assert!(secs <= 8.0, "delay {secs}s should be capped at 8");
    }

    // ── jitter_factor tests ────────────────────────────────────

    #[test]
    fn jitter_factor_in_range() {
        for _ in 0..100 {
            let j = jitter_factor();
            assert!((0.75..=1.0).contains(&j), "jitter {j} out of [0.75, 1.0]");
        }
    }

    // ── RequestOptions defaults ────────────────────────────────

    #[test]
    fn request_options_default_is_all_none() {
        let opts = RequestOptions::default();
        assert!(opts.extra_headers.is_none());
        assert!(opts.timeout.is_none());
        assert!(opts.max_retries.is_none());
    }
}
