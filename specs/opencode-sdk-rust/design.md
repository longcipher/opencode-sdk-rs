# Design Document: opencode-sdk-rust

| Metadata | Details |
| :--- | :--- |
| **Author** | pb-plan agent |
| **Status** | Draft |
| **Created** | 2026-02-13 |
| **Reviewers** | — |
| **Related Issues** | N/A |

## 1. Executive Summary

**Problem:** The opencode platform currently only has a JavaScript/TypeScript SDK (`opencode-sdk-js`), generated from an OpenAPI spec by Stainless. Rust consumers of the opencode API have no idiomatic SDK and must write raw HTTP calls manually, losing type safety, retry logic, error handling, and streaming support.

**Solution:** Implement a new Rust crate `opencode-sdk` inside `crates/opencode-sdk/` that mirrors every API resource, type, and method of `opencode-sdk-js` — including the full HTTP client with retries, SSE streaming, structured error types, and typed request/response models — using `hpx` (with `rustls`) for HTTP, `serde` for serialization, `thiserror` for errors, and `tokio` for async runtime.

---

## 2. Requirements & Goals

### 2.1 Problem Statement

Rust developers integrating with the opencode platform must manually construct HTTP requests, parse JSON responses, handle retries, interpret error codes, and consume SSE streams. There is no Rust-native SDK providing compile-time type safety, ergonomic builders, automatic retries, or streaming support.

### 2.2 Functional Goals

1. **Full API Parity:** Every resource & method in `opencode-sdk-js` must have a Rust equivalent:
   - **Event:** `list()` → SSE `Stream<EventListResponse>`
   - **App:** `get()`, `init()`, `log(params)`, `modes()`, `providers()`
   - **Find:** `files(params)`, `symbols(params)`, `text(params)`
   - **File:** `read(params)`, `status()`
   - **Config:** `get()`
   - **Session:** `create()`, `list()`, `delete(id)`, `abort(id)`, `chat(id, params)`, `init(id, params)`, `messages(id)`, `revert(id, params)`, `share(id)`, `summarize(id, params)`, `unrevert(id)`, `unshare(id)`
   - **Tui:** `append_prompt(params)`, `open_help()`
2. **Full Type Parity:** Every request param struct, response struct, enum, and nested type from the JS SDK must have a corresponding Rust type with `serde` derive.
3. **Configurable Client:** `ClientOptions` struct supporting `base_url`, `timeout`, `max_retries`, `default_headers`, `default_query`.
4. **Automatic Retries:** Exponential backoff with jitter, honoring `Retry-After` / `retry-after-ms` headers, retrying on 408/409/429/5xx, respecting `x-should-retry`.
5. **SSE Streaming:** The `/event` endpoint returns Server-Sent Events. The SDK must expose an `async Stream` that yields typed `EventListResponse` variants.
6. **Structured Errors:** A typed error hierarchy mirroring the JS SDK: `OpencodeError`, `ApiError` (with status/headers/body), `ApiConnectionError`, `ApiConnectionTimeoutError`, `BadRequestError` (400), `AuthenticationError` (401), `PermissionDeniedError` (403), `NotFoundError` (404), `ConflictError` (409), `UnprocessableEntityError` (422), `RateLimitError` (429), `InternalServerError` (5xx).
7. **Shared Error Types:** `MessageAbortedError`, `ProviderAuthError`, `UnknownError` as domain error enums.
8. **Environment-Based Config:** Read `OPENCODE_BASE_URL` from env; default to `http://localhost:54321`.

### 2.3 Non-Functional Goals

- **Performance:** Zero unnecessary allocations; use `&str` borrows where feasible in request params; streaming responses avoid buffering the full body.
- **Reliability:** Retry logic with configurable max retries and exponential backoff. Graceful timeout handling.
- **Security:** TLS via `rustls` (through `hpx`). No native TLS dependency.
- **Observability:** Use `tracing` crate for structured logging at debug/info/warn/error levels (request start, retry, response status, errors).
- **Ergonomics:** Builder pattern for client construction; methods accept typed param structs; results use `Result<T, OpencodeError>`.

### 2.4 Out of Scope

- OpenAPI spec generation / code generation tooling.
- File upload support (the JS SDK has `toFile` / `Uploadable` but current API endpoints don't use multipart uploads in the resource methods).
- WebSocket or bidirectional streaming.
- CLI tool or binary — this is a library crate only.
- TUI terminal UI features — the `tui` resource is just an HTTP API wrapper.

### 2.5 Assumptions

- The opencode server API matches the types and endpoints defined in the JS SDK's source code (generated from OpenAPI).
- `hpx` with `rustls` feature provides equivalent HTTP functionality to `fetch` in the JS SDK.
- SSE events from `/event` follow standard `text/event-stream` format with JSON `data` payloads.
- The workspace will gain a root `Cargo.toml` with workspace configuration.

---

## 3. Architecture Overview

### 3.1 System Context

```text
┌─────────────────────────┐
│    User Application     │
│   (Rust binary/lib)     │
└───────────┬─────────────┘
            │  uses
┌───────────▼─────────────┐
│   opencode-sdk crate    │
│  ┌───────────────────┐  │
│  │   Opencode Client │  │
│  │  (hpx HTTP + TLS) │  │
│  ├───────────────────┤  │
│  │  Resource Modules │  │
│  │  app/session/...  │  │
│  ├───────────────────┤  │
│  │   Type Modules    │  │
│  │  models/errors    │  │
│  ├───────────────────┤  │
│  │  SSE Streaming    │  │
│  └───────────────────┘  │
└───────────┬─────────────┘
            │  HTTP/TLS
┌───────────▼─────────────┐
│   opencode Server API   │
│  (localhost:54321)       │
└─────────────────────────┘
```

The `opencode-sdk` crate sits between user application code and the opencode server. It handles HTTP transport, serialization, retries, error mapping, and streaming.

### 3.2 Key Design Principles

1. **Type-Driven Correctness:** Every API request/response is fully typed. Enums use `#[serde(tag = "...")]` or `#[serde(untagged)]` to match the JS SDK's discriminated unions.
2. **Zero-Cost Abstractions:** Resource structs hold a reference to the client, adding no overhead. Methods are thin wrappers around the core HTTP methods.
3. **Idiomatic Rust:** Use `Result<T, E>`, `async`/`await`, `impl Into<String>` for string params, `Option<T>` for optional fields. Snake_case naming.
4. **Mirrors JS SDK Structure:** Each JS resource file maps to a Rust module. Each TypeScript interface maps to a Rust struct. Each TypeScript union maps to a Rust enum.

### 3.3 Existing Components to Reuse

| Component | Location | How to Reuse |
| :--- | :--- | :--- |
| `common` crate | `crates/common/Cargo.toml` | Currently has `serde` + `thiserror`. Can be used for shared types if cross-crate sharing is needed later. For now, `opencode-sdk` will be self-contained. |
| Workspace `[lints]` config | `crates/common/Cargo.toml` | Reuse `lints.workspace = true` pattern in the new crate. |
| `.taplo.toml` | Root | Existing TOML formatter config — already applies to new Cargo.toml files. |

> The project is early-stage with only a `common` crate stub. The new `opencode-sdk` crate will be the primary library.

---

## 4. Detailed Design

### 4.1 Module Structure

```text
crates/opencode-sdk/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public API re-exports
    ├── client.rs           # Opencode client struct, builder, HTTP methods
    ├── error.rs            # Error types hierarchy
    ├── streaming.rs        # SSE stream parsing, Stream<Item> impl
    ├── config.rs           # ClientOptions, env reading
    ├── resources/
    │   ├── mod.rs           # Re-exports
    │   ├── app.rs           # AppResource + types
    │   ├── config.rs        # ConfigResource + types
    │   ├── event.rs         # EventResource + types
    │   ├── file.rs          # FileResource + types
    │   ├── find.rs          # FindResource + types
    │   ├── session.rs       # SessionResource + types
    │   ├── shared.rs        # Shared error types (domain)
    │   └── tui.rs           # TuiResource + types
    └── types.rs             # Common helper types (HashMap aliases, etc.)
```

### 4.2 Data Structures & Types

All TypeScript interfaces map 1:1 to Rust structs with `#[derive(Debug, Clone, Serialize, Deserialize)]`. Key mappings:

| TypeScript | Rust |
| :--- | :--- |
| `interface Foo { bar: string }` | `pub struct Foo { pub bar: String }` |
| `bar?: string` | `pub bar: Option<String>` |
| `{ [key: string]: T }` | `HashMap<String, T>` |
| `Array<T>` | `Vec<T>` |
| `type X = A \| B` (tagged union) | `#[serde(tag = "type")] enum X { A(A), B(B) }` |
| `type X = A \| B` (untagged) | `#[serde(untagged)] enum X { A(A), B(B) }` |
| `'foo' \| 'bar'` (string literal) | `#[serde(rename_all = "snake_case")] enum Foo { Foo, Bar }` |
| `number` | `f64` or `i64` (context-dependent) |
| `boolean` | `bool` |

**Core client struct:**

```rust
pub struct Opencode {
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    default_headers: HeaderMap,
    default_query: HashMap<String, String>,
    client: hpx::Client,  // hpx HTTP client with rustls
}
```

**Resource access pattern:**

```rust
impl Opencode {
    pub fn app(&self) -> AppResource<'_> { AppResource { client: self } }
    pub fn session(&self) -> SessionResource<'_> { SessionResource { client: self } }
    pub fn event(&self) -> EventResource<'_> { EventResource { client: self } }
    pub fn file(&self) -> FileResource<'_> { FileResource { client: self } }
    pub fn find(&self) -> FindResource<'_> { FindResource { client: self } }
    pub fn config(&self) -> ConfigResource<'_> { ConfigResource { client: self } }
    pub fn tui(&self) -> TuiResource<'_> { TuiResource { client: self } }
}
```

**Example resource:**

```rust
pub struct SessionResource<'a> {
    client: &'a Opencode,
}

impl<'a> SessionResource<'a> {
    pub async fn create(&self) -> Result<Session, OpencodeError> {
        self.client.post("/session", None::<()>).await
    }

    pub async fn chat(&self, id: &str, params: SessionChatParams) -> Result<AssistantMessage, OpencodeError> {
        self.client.post(&format!("/session/{id}/message"), Some(params)).await
    }
    // ... etc
}
```

### 4.3 Interface Design

**Public API surface (lib.rs re-exports):**

```rust
// Client
pub use client::{Opencode, ClientOptions};

// Errors
pub use error::{
    OpencodeError, ApiError, ApiConnectionError, ApiConnectionTimeoutError,
    BadRequestError, AuthenticationError, PermissionDeniedError, NotFoundError,
    ConflictError, UnprocessableEntityError, RateLimitError, InternalServerError,
};

// Streaming
pub use streaming::Stream;

// All resource types
pub use resources::*;
```

### 4.4 Logic Flow

**Request lifecycle:**

1. User calls `client.session().create().await`
2. `SessionResource::create` calls `client.post("/session", None)`
3. `Opencode::post` → `Opencode::request` builds URL, headers, body
4. `Opencode::make_request` sends via `hpx`, handles timeout
5. If error → check retry eligibility → exponential backoff → retry
6. On success → deserialize JSON body via `serde_json::from_slice`
7. On HTTP error → map status to specific error variant → return `Err`

**SSE streaming lifecycle (Event.list):**

1. User calls `client.event().list().await`
2. `EventResource::list` calls `client.get_stream("/event")`
3. Client sends GET, receives `text/event-stream` response
4. Returns `Stream<EventListResponse>` wrapping an async iterator
5. Each SSE `data:` line is parsed as JSON into `EventListResponse` enum
6. Stream yields items until connection closes or abort

### 4.5 Configuration

| Config | Source | Default |
| :--- | :--- | :--- |
| `base_url` | `ClientOptions.base_url` or `OPENCODE_BASE_URL` env | `http://localhost:54321` |
| `timeout` | `ClientOptions.timeout` | 60 seconds |
| `max_retries` | `ClientOptions.max_retries` | 2 |
| `default_headers` | `ClientOptions.default_headers` | Empty |
| `default_query` | `ClientOptions.default_query` | Empty |

### 4.6 Error Handling

**Error hierarchy (using `thiserror`):**

```rust
#[derive(Debug, thiserror::Error)]
pub enum OpencodeError {
    #[error("API error: {status} {message}")]
    Api(ApiError),

    #[error("Connection error: {0}")]
    Connection(ApiConnectionError),

    #[error("Request timed out")]
    Timeout(ApiConnectionTimeoutError),

    #[error("Request aborted by user")]
    UserAbort,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    Http(#[from] hpx::Error),
}
```

`ApiError` contains `status: u16`, `headers: HeaderMap`, `body: Option<serde_json::Value>`, and helper methods to downcast to specific error types (`is_not_found()`, `is_rate_limited()`, etc.).

Status-specific error structs (`BadRequestError`, `NotFoundError`, etc.) wrap `ApiError` and can be pattern-matched.

---

## 5. Verification & Testing Strategy

### 5.1 Unit Testing

- Test JSON serialization/deserialization round-trips for every request/response type.
- Test URL building with query parameters and path interpolation.
- Test retry delay calculation (exponential backoff with jitter bounds).
- Test error mapping from HTTP status codes to typed errors.
- Test SSE line parsing and event construction.

### 5.2 Integration Testing

- Mock HTTP server (using `wiremock` or similar) for each endpoint.
- Test full request-response cycle for every resource method.
- Test retry behavior with mock 429/500 responses.
- Test SSE streaming with mock event stream.
- Test timeout handling.

### 5.3 Validation Rules

| Test Case ID | Action | Expected Outcome | Verification Method |
| :--- | :--- | :--- | :--- |
| **TC-01** | Deserialize every response type from JSON fixture | All fields correctly populated | Unit test with snapshot JSON |
| **TC-02** | `client.app().get()` against mock server | Returns `App` struct | Integration test with wiremock |
| **TC-03** | `client.session().chat(id, params)` | POST body serialized correctly, returns `AssistantMessage` | Mock server validates request body |
| **TC-04** | Request to mock returning 429 | Retried up to `max_retries`, then `RateLimitError` | Integration test with retry counter |
| **TC-05** | SSE stream from `/event` | Yields typed `EventListResponse` variants | Integration test with mock SSE |
| **TC-06** | Request with timeout exceeded | `ApiConnectionTimeoutError` returned | Integration test with delayed mock |
| **TC-07** | All types compile with `Serialize`/`Deserialize` | No compile errors | `cargo build` |
| **TC-08** | `ClientOptions` from env | `OPENCODE_BASE_URL` respected | Unit test with env override |

---

## 6. Implementation Plan

- [ ] **Phase 1: Foundation** — Workspace setup, crate scaffolding, core client, error types, config
- [ ] **Phase 2: Core Logic** — All resource types (request/response structs), all resource methods, SSE streaming
- [ ] **Phase 3: Integration** — Wire resources to client, full public API, re-exports
- [ ] **Phase 4: Polish** — Unit tests, serde round-trip tests, documentation, CI checks

---

## 7. Cross-Functional Concerns

- **Backward Compatibility:** First release — no backward compatibility concerns.
- **Versioning:** Crate version starts at `0.1.0`, using workspace version.
- **Documentation:** All public types and methods get `///` doc comments. Module-level docs explain usage patterns.
- **CI:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` must pass.
- **Feature Flags:** Consider optional `streaming` feature flag for SSE support (pulls in extra deps). For v1, include everything by default.
- **Dependency Licensing:** All dependencies (`hpx`, `serde`, `thiserror`, `tokio`, `tracing`, `serde_json`) are MIT/Apache-2.0 compatible.
