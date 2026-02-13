# opencode-sdk-rust — Implementation Tasks

| Metadata | Details |
| :--- | :--- |
| **Design Doc** | specs/opencode-sdk-rust/design.md |
| **Owner** | — |
| **Start Date** | 2026-02-13 |
| **Target Date** | 2026-03-07 |
| **Status** | Planning |

## Summary & Phasing

Implement a complete Rust SDK crate mirroring all APIs of `opencode-sdk-js`. The work is divided into four phases: foundation (workspace + client core), types & resources, integration & streaming, and polish.

- **Phase 1: Foundation & Scaffolding** — Workspace Cargo.toml, crate skeleton, core HTTP client, error types, config
- **Phase 2: Core Logic** — All resource type definitions, all resource method implementations
- **Phase 3: Integration & Features** — SSE streaming, full public API wiring, re-exports
- **Phase 4: Polish, QA & Docs** — Tests, documentation, formatting, linting, CI readiness

---

## Phase 1: Foundation & Scaffolding

### Task 1.1: Create Root Workspace Cargo.toml

> **Context:** The project currently has no root `Cargo.toml`. A workspace manifest is required to manage both `crates/common` and the new `crates/opencode-sdk`. Per project instructions, only the root has version numbers; sub-crates use `workspace = true`.
> **Verification:** `cargo metadata` succeeds and lists both workspace members.

- **Priority:** P0
- **Scope:** Workspace configuration
- **Status:** � DONE

- [x] **Step 1:** Create root `Cargo.toml` with `[workspace]` containing `members = ["crates/*"]`, `resolver = "3"`, `[workspace.package]` with `version = "0.1.0"` and `edition = "2024"`.
- [x] **Step 2:** Add `[workspace.dependencies]` with: `serde = "1"`, `serde_json = "1"`, `thiserror = "2"`, `tokio = "1"`, `tracing = "0.1"`, `hpx = "0.1"`, `futures-core = "0.3"`, `http = "1"`. Use `cargo add --workspace` to determine latest versions.
- [x] **Step 3:** Add `[workspace.lints.clippy]` with sensible defaults.
- [x] **Verification:** `cargo check --workspace` compiles with no errors.

---

### Task 1.2: Scaffold `opencode-sdk` Crate

> **Context:** Create the new library crate in `crates/opencode-sdk/` with proper workspace inheritance. This is the primary deliverable crate.
> **Verification:** `cargo check -p opencode-sdk` succeeds.

- **Priority:** P0
- **Scope:** Crate skeleton
- **Status:** � DONE

- [x] **Step 1:** Create `crates/opencode-sdk/Cargo.toml` with `name = "opencode-sdk"`, `version.workspace = true`, `edition.workspace = true`, `lints.workspace = true`.
- [x] **Step 2:** Add dependencies using `cargo add -p opencode-sdk --workspace`: `serde` (features = `["derive"]`), `serde_json`, `thiserror`, `tokio` (features = `["rt", "macros", "time"]`), `tracing`, `hpx` (features = `["rustls"]`), `futures-core`, `http`.
- [x] **Step 3:** Create `crates/opencode-sdk/src/lib.rs` with placeholder module declarations for `client`, `error`, `config`, `streaming`, `types`, and `resources`.
- [x] **Step 4:** Create empty module files: `client.rs`, `error.rs`, `config.rs`, `streaming.rs`, `types.rs`, `resources/mod.rs`.
- [x] **Verification:** `cargo check -p opencode-sdk` compiles.

---

### Task 1.3: Implement Error Types

> **Context:** The JS SDK has a structured error hierarchy. The Rust equivalent uses `thiserror` to create an `OpencodeError` enum and status-specific error types. This must be in place before any HTTP methods can return results.
> **Verification:** All error types compile and can be constructed/matched in a unit test.

- **Priority:** P0
- **Scope:** Error module
- **Status:** � DONE

- [x] **Step 1:** In `error.rs`, define `OpencodeError` enum with variants: `Api { status: u16, headers: Option<http::HeaderMap>, body: Option<serde_json::Value>, message: String }`, `Connection { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> }`, `Timeout`, `UserAbort`, `Serialization(serde_json::Error)`, `Http(Box<dyn std::error::Error + Send + Sync>)`.
- [x] **Step 2:** Implement helper methods: `OpencodeError::status()`, `is_retryable()`, `is_timeout()`, convenience constructors for each HTTP status (400→`bad_request`, 401→`authentication`, 403→`permission_denied`, 404→`not_found`, 409→`conflict`, 422→`unprocessable_entity`, 429→`rate_limit`, 5xx→`internal_server`).
- [x] **Step 3:** Implement `OpencodeError::from_response(status, headers, body)` factory that maps status codes to the appropriate variant.
- [x] **Verification:** Write unit test constructing each error variant and asserting `Display` output and `status()` return values.

---

### Task 1.4: Implement Client Config & Builder

> **Context:** The `Opencode` client needs a builder/constructor that reads `OPENCODE_BASE_URL` from env, accepts `ClientOptions`, and sets defaults matching the JS SDK (timeout=60s, max_retries=2, base_url=`http://localhost:54321`).
> **Verification:** `Opencode::new()` and `Opencode::builder().base_url("...").build()` both compile and produce valid clients.

- **Priority:** P0
- **Scope:** Client configuration
- **Status:** � DONE

- [x] **Step 1:** In `config.rs`, define `ClientOptions` struct with `base_url: Option<String>`, `timeout: Option<Duration>`, `max_retries: Option<u32>`, `default_headers: Option<HeaderMap>`, `default_query: Option<HashMap<String, String>>`.
- [x] **Step 2:** Implement `Default` for `ClientOptions` reading `OPENCODE_BASE_URL` from env.
- [x] **Step 3:** In `client.rs`, define `Opencode` struct holding `base_url: String`, `timeout: Duration`, `max_retries: u32`, `default_headers: HeaderMap`, `default_query: HashMap<String, String>`, `http_client: hpx::Client`.
- [x] **Step 4:** Implement `Opencode::new()` (default config), `Opencode::with_options(opts: ClientOptions)`, and `Opencode::builder()` returning a builder.
- [x] **Verification:** Unit test creating client with default and custom options, asserting field values.

---

### Task 1.5: Implement Core HTTP Methods

> **Context:** The JS SDK's `Opencode` class has `get`, `post`, `put`, `patch`, `delete` methods that build URLs, set headers, serialize bodies, send requests, handle retries and errors. The Rust client needs equivalent async methods.
> **Verification:** A mock-server integration test can execute `client.get::<serde_json::Value>("/test")` and receive the expected response.

- **Priority:** P0
- **Scope:** HTTP transport layer
- **Status:** ✅ DONE

- [x] **Step 1:** Implement `Opencode::build_url(&self, path, query)` — joins base_url + path, appends query params and default_query.
- [x] **Step 2:** Implement `Opencode::build_headers(&self, extra_headers, retry_count)` — merges default_headers, Accept: application/json, User-Agent.
- [x] **Step 3:** Implement internal `Opencode::make_request<T: DeserializeOwned>(&self, method, path, body, query, options)` — builds request, sends via `hpx`, handles response parsing or error mapping. Use `tracing::debug!` for request logging.
- [x] **Step 4:** Implement retry logic inside `make_request`: check `is_retryable()` on error/status, compute backoff delay (exponential with jitter), respect `retry-after` / `retry-after-ms` / `x-should-retry` headers, loop up to `max_retries`.
- [x] **Step 5:** Implement public convenience methods: `get<T>`, `post<T>`, `put<T>`, `patch<T>`, `delete<T>` — each calls `make_request` with the appropriate HTTP method.
- [x] **Step 6:** Implement `RequestOptions` struct for per-request overrides (extra headers, timeout, signal/cancellation).
- [x] **Verification:** Unit tests for `build_url`, `build_headers`, retry helpers, and `RequestOptions`. 52 total tests pass.

---

## Phase 2: Core Logic

### Task 2.1: Implement Shared Domain Types

> **Context:** `resources/shared.ts` defines `MessageAbortedError`, `ProviderAuthError`, `UnknownError` — domain error types used across session and event responses.
> **Verification:** Types compile with `Serialize`/`Deserialize` and round-trip a JSON fixture.

- **Priority:** P0
- **Scope:** Shared types module
- **Status:** ✅ DONE

- [x] **Step 1:** In `resources/shared.rs`, define `MessageAbortedError`, `ProviderAuthError` (with `Data { message, provider_id }`), `UnknownError` (with `Data { message }`).
- [x] **Step 2:** Use `#[serde(tag = "name")]` for discriminated unions where `name` field is the discriminator.
- [x] **Verification:** Unit test deserializing JSON fixtures for each type.

---

### Task 2.2: Implement App Resource Types & Methods

> **Context:** Maps `resources/app.ts` — types: `App`, `Mode`, `Model`, `Provider`, `AppInitResponse`, `AppLogResponse`, `AppModesResponse`, `AppProvidersResponse`, `AppLogParams`. Methods: `get()`, `init()`, `log(params)`, `modes()`, `providers()`.
> **Verification:** All types round-trip JSON. All methods compile and accept correct parameters.

- **Priority:** P0
- **Scope:** App resource module
- **Status:** ✅ DONE

- [x] **Step 1:** In `resources/app.rs`, define all types: `App` (with nested `Path`, `Time`), `Mode` (with nested `Model`), `Model` (with nested `Cost`, `Limit`), `Provider`, `AppProvidersResponse` (with `default` and `providers` fields), `AppLogParams` (with `level` enum, `message`, `service`, `extra`).
- [x] **Step 2:** Define type aliases: `AppInitResponse = bool`, `AppLogResponse = bool`, `AppModesResponse = Vec<Mode>`.
- [x] **Step 3:** Implement `AppResource<'a>` struct with methods: `get()`, `init()`, `log(params)`, `modes()`, `providers()`, each calling the appropriate client HTTP method.
- [x] **Step 4:** Wire `Opencode::app(&self) -> AppResource<'_>`.
- [x] **Verification:** JSON round-trip tests for `App`, `Mode`, `Model`, `Provider`. Compile check for all methods.

---

### Task 2.3: Implement Config Resource Types & Methods

> **Context:** Maps `resources/config.ts` — types: `Config`, `KeybindsConfig`, `McpLocalConfig`, `McpRemoteConfig`, `ModeConfig`. Method: `get()`.
> **Verification:** `Config` struct deserializes from a real-world JSON fixture.

- **Priority:** P0
- **Scope:** Config resource module
- **Status:** ✅ DONE

- [x] **Step 1:** In `resources/config.rs`, define `Config` struct with all fields (including nested `Agent`, `Experimental`, `Mode`, `Provider` namespaces, `KeybindsConfig`, `McpLocalConfig`, `McpRemoteConfig`, `ModeConfig`).
- [x] **Step 2:** Handle `$schema` field using `#[serde(rename = "$schema")]`.
- [x] **Step 3:** Handle the MCP config map: `mcp: Option<HashMap<String, McpConfig>>` where `McpConfig` is `#[serde(tag = "type")]` enum with `Local(McpLocalConfig)` / `Remote(McpRemoteConfig)`.
- [x] **Step 4:** Implement `ConfigResource<'a>` with `get()` method.
- [x] **Step 5:** Wire `Opencode::config(&self) -> ConfigResource<'_>`.
- [x] **Verification:** Deserialize a comprehensive Config JSON fixture. All optional fields work correctly.

---

### Task 2.4: Implement File Resource Types & Methods

> **Context:** Maps `resources/file.ts` — types: `File`, `FileReadResponse`, `FileStatusResponse`, `FileReadParams`. Methods: `read(params)`, `status()`.
> **Verification:** Types round-trip JSON.

- **Priority:** P0
- **Scope:** File resource module
- **Status:** ✅ DONE

- [x] **Step 1:** In `resources/file.rs`, define `File` (with `added`, `path`, `removed`, `status` enum), `FileReadResponse` (with `content`, `type_` enum), `FileReadParams` (with `path`).
- [x] **Step 2:** Type alias `FileStatusResponse = Vec<File>`.
- [x] **Step 3:** Implement `FileResource<'a>` with `read(params)` (GET /file with query) and `status()` (GET /file/status).
- [x] **Step 4:** Wire `Opencode::file(&self) -> FileResource<'_>`.
- [x] **Verification:** JSON round-trip tests.

---

### Task 2.5: Implement Find Resource Types & Methods

> **Context:** Maps `resources/find.ts` — types: `Symbol`, `FindFilesResponse`, `FindSymbolsResponse`, `FindTextResponse`, params. Methods: `files(params)`, `symbols(params)`, `text(params)`.
> **Verification:** Types round-trip JSON including deeply nested `Symbol.Location.Range`.

- **Priority:** P0
- **Scope:** Find resource module
- **Status:** ✅ DONE

- [x] **Step 1:** In `resources/find.rs`, define `Symbol` (with nested `Location`, `Range`, `Position`), `FindTextResponseItem` (with `Lines`, `Path`, `Submatch`, `Match`).
- [x] **Step 2:** Type aliases: `FindFilesResponse = Vec<String>`, `FindSymbolsResponse = Vec<Symbol>`, `FindTextResponse = Vec<FindTextResponseItem>`.
- [x] **Step 3:** Define params: `FindFilesParams { query }`, `FindSymbolsParams { query }`, `FindTextParams { pattern }`.
- [x] **Step 4:** Implement `FindResource<'a>` with `files(params)`, `symbols(params)`, `text(params)`.
- [x] **Step 5:** Wire `Opencode::find(&self) -> FindResource<'_>`.
- [x] **Verification:** JSON round-trip tests for `Symbol` with nested location data.

---

### Task 2.6: Implement Session Resource Types & Methods

> **Context:** The largest resource. Maps `resources/session.ts` — 20+ types and 12 methods. Includes complex discriminated unions for `Part`, `Message`, `ToolState`, `FilePartSource`.
> **Verification:** All types deserialize from JSON fixtures. All 12 methods compile and have correct HTTP method + path.

- **Priority:** P0
- **Scope:** Session resource module
- **Status:** ✅ DONE

- [x] **Step 1:** Define message types: `UserMessage`, `AssistantMessage` (with nested `Path`, `Time`, `Tokens`, `Cache`, error union). Define `Message` as `#[serde(tag = "role")]` enum.
- [x] **Step 2:** Define part types: `TextPart`, `FilePart`, `ToolPart`, `StepStartPart`, `StepFinishPart`, `SnapshotPart`, `PatchPart`. Define `Part` as `#[serde(tag = "type")]` enum.
- [x] **Step 3:** Define tool state types: `ToolStatePending`, `ToolStateRunning`, `ToolStateCompleted`, `ToolStateError`. Define `ToolState` as `#[serde(tag = "status")]` enum.
- [x] **Step 4:** Define source types: `FileSource`, `SymbolSource`, `FilePartSourceText`. Define `FilePartSource` as enum.
- [x] **Step 5:** Define input types: `TextPartInput`, `FilePartInput`.
- [x] **Step 6:** Define `Session` struct (with nested `Time`, `Revert`, `Share`).
- [x] **Step 7:** Define response aliases: `SessionListResponse = Vec<Session>`, `SessionDeleteResponse = bool`, `SessionAbortResponse = bool`, `SessionInitResponse = bool`, `SessionSummarizeResponse = bool`, `SessionMessagesResponse = Vec<SessionMessagesResponseItem>`.
- [x] **Step 8:** Define params: `SessionChatParams`, `SessionInitParams`, `SessionRevertParams`, `SessionSummarizeParams`.
- [x] **Step 9:** Implement `SessionResource<'a>` with all 12 methods: `create`, `list`, `delete(id)`, `abort(id)`, `chat(id, params)`, `init(id, params)`, `messages(id)`, `revert(id, params)`, `share(id)`, `summarize(id, params)`, `unrevert(id)`, `unshare(id)`.
- [x] **Step 10:** Wire `Opencode::session(&self) -> SessionResource<'_>`.
- [x] **Verification:** JSON round-trip tests for all types, especially discriminated unions. Compile check for all methods.

---

### Task 2.7: Implement Tui Resource Types & Methods

> **Context:** Maps `resources/tui.ts` — types: `TuiAppendPromptResponse`, `TuiOpenHelpResponse`, `TuiAppendPromptParams`. Methods: `append_prompt(params)`, `open_help()`.
> **Verification:** Types round-trip JSON.

- **Priority:** P1
- **Scope:** Tui resource module
- **Status:** ✅ DONE

- [x] **Step 1:** In `resources/tui.rs`, define `TuiAppendPromptParams { text }`, type aliases `TuiAppendPromptResponse = bool`, `TuiOpenHelpResponse = bool`.
- [x] **Step 2:** Implement `TuiResource<'a>` with `append_prompt(params)` and `open_help()`.
- [x] **Step 3:** Wire `Opencode::tui(&self) -> TuiResource<'_>`.
- [x] **Verification:** Compile check.

---

### Task 2.8: Implement Event Resource Types & Methods

> **Context:** Maps `resources/event.ts` — `EventListResponse` is a large discriminated union of 15 event types. The `list()` method returns an SSE stream, but the types must be defined first. Streaming implementation is in Phase 3.
> **Verification:** `EventListResponse` deserializes from JSON for each variant.

- **Priority:** P0
- **Scope:** Event types module
- **Status:** ✅ DONE

- [x] **Step 1:** In `resources/event.rs`, define the `EventListResponse` enum with all 15 variants using `#[serde(tag = "type")]`: `InstallationUpdated`, `LspClientDiagnostics`, `MessageUpdated`, `MessageRemoved`, `MessagePartUpdated`, `MessagePartRemoved`, `StorageWrite`, `PermissionUpdated`, `FileEdited`, `SessionUpdated`, `SessionDeleted`, `SessionIdle`, `SessionError`, `FileWatcherUpdated`, `IdeInstalled`.
- [x] **Step 2:** Define nested `Properties` structs for each variant, referencing `Session`, `Message`, `Part` types from the session module.
- [x] **Step 3:** Define `EventResource<'a>` struct with a `list()` method signature that returns `Result<Stream<EventListResponse>, OpencodeError>` (implementation deferred to Task 3.1).
- [x] **Step 4:** Wire `Opencode::event(&self) -> EventResource<'_>`.
- [x] **Verification:** JSON round-trip tests for each event variant.

---

## Phase 3: Integration & Features

### Task 3.1: Implement SSE Streaming

> **Context:** The `/event` endpoint returns Server-Sent Events (SSE). The JS SDK uses a `Stream` class that parses SSE lines and yields JSON-parsed items. The Rust SDK needs an equivalent using `futures_core::Stream` trait. The `hpx` client should support reading a streaming response body.
> **Verification:** Integration test with a mock SSE server yields correctly typed events.

- **Priority:** P0
- **Scope:** Streaming module
- **Status:** ✅ DONE

- [x] **Step 1:** In `streaming.rs`, implement an SSE line decoder that buffers incoming bytes and yields complete `ServerSentEvent { event, data, raw }` structs when a double-newline is encountered.
- [x] **Step 2:** Implement `SseStream<T: DeserializeOwned>` struct wrapping an async byte stream from `hpx`, implementing `futures_core::Stream<Item = Result<T, OpencodeError>>`.
- [x] **Step 3:** In `client.rs`, add `Opencode::get_stream<T>(&self, path)` method that sends a GET request and returns `SseStream<T>`.
- [x] **Step 4:** Complete `EventResource::list()` implementation to call `client.get_stream("/event")`.
- [x] **Verification:** Unit test the SSE parser with sample event data. Integration test against mock SSE endpoint yielding multiple events.

---

### Task 3.2: Wire Public API & Re-exports

> **Context:** The `lib.rs` must re-export all public types and the client so that users can `use opencode_sdk::{Opencode, Session, App, ...}`. Mirror the JS SDK's `index.ts` exports.
> **Verification:** A downstream crate can `use opencode_sdk::*` and access all types and the client.

- **Priority:** P0
- **Scope:** Public API surface
- **Status:** ✅ DONE

- [x] **Step 1:** In `resources/mod.rs`, re-export all types and resource structs from each resource sub-module.
- [x] **Step 2:** In `lib.rs`, add `pub use` for: `client::Opencode`, `client::ClientOptions`, `error::*`, `streaming::SseStream`, and `resources::*`.
- [x] **Step 3:** Ensure no naming conflicts (e.g., `File` type vs `std::fs::File` — consider keeping it as `resources::File` or renaming to `FileInfo`).
- [x] **Verification:** Write a smoke-test integration test that imports key types and constructs a client.

---

## Phase 4: Polish, QA & Docs

### Task 4.1: Serde Round-Trip Tests for All Types

> **Context:** Every type in the SDK must correctly serialize/deserialize. Create JSON fixture files from the JS SDK's test data and validate round-trips.
> **Verification:** `cargo test` — all round-trip tests pass.

- **Priority:** P1
- **Scope:** Unit tests
- **Status:** ✅ DONE

- [x] **Step 1:** Create `tests/fixtures/` directory with JSON files for each resource's response types, based on the JS SDK's test fixtures.
- [x] **Step 2:** Write `#[cfg(test)] mod tests` in each resource module with `serde_json::from_str` / `serde_json::to_string` round-trip assertions.
- [x] **Step 3:** Test edge cases: optional fields absent, empty arrays, nested nulls, discriminated union variants.
- [x] **Verification:** `cargo test -p opencode-sdk` — all tests pass.

---

### Task 4.2: Integration Tests with Mock Server

> **Context:** Test the full request lifecycle — client construction, request building, response parsing, error handling, retries — against a mock HTTP server.
> **Verification:** `cargo test` — all integration tests pass.

- **Priority:** P1
- **Scope:** Integration tests
- **Status:** ✅ DONE

- [x] **Step 1:** Add `wiremock` (or `mockito`) as a dev-dependency.
- [x] **Step 2:** Write integration tests for at least: `app.get()`, `session.create()`, `session.chat()`, `session.list()`, `file.read()`, `config.get()`.
- [x] **Step 3:** Write retry integration tests: mock 429 response, verify retry with backoff, verify eventual success or max-retry failure.
- [x] **Step 4:** Write timeout test: mock slow response, verify `Timeout` error.
- [x] **Verification:** `cargo test -p opencode-sdk` — all integration tests pass.

---

### Task 4.3: Documentation & Examples

> **Context:** All public items need doc comments. A `README.md` for the crate and a usage example help users get started.
> **Verification:** `cargo doc --no-deps -p opencode-sdk` builds with no warnings.

- **Priority:** P2
- **Scope:** Documentation
- **Status:** ✅ DONE

- [x] **Step 1:** Add `///` doc comments to all public structs, enums, methods, and type aliases.
- [x] **Step 2:** Add module-level `//!` docs to `lib.rs` with a usage example.
- [x] **Step 3:** Create `crates/opencode-sdk/examples/basic.rs` demonstrating client creation, calling `app.get()`, and listing sessions.
- [x] **Step 4:** Update root `README.md` to mention the Rust SDK.
- [x] **Verification:** `cargo doc --no-deps -p opencode-sdk` — no warnings. Example compiles with `cargo check --example basic -p opencode-sdk`.

---

### Task 4.4: Formatting, Linting & CI Readiness

> **Context:** Per project instructions, `just format`, `just lint`, `just test` must all pass. Ensure the new crate conforms.
> **Verification:** All three commands pass with zero warnings/errors.

- **Priority:** P1
- **Scope:** Code quality
- **Status:** ✅ DONE

- [x] **Step 1:** Run `cargo fmt --all -- --check` and fix any formatting issues.
- [x] **Step 2:** Run `cargo clippy --workspace -- -D warnings` and fix all warnings.
- [x] **Step 3:** Ensure `cargo test --workspace` passes.
- [x] **Step 4:** Verify `justfile` commands work (if `justfile` exists) or document the required commands.
- [x] **Verification:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` — all pass cleanly.

---

## Summary & Timeline

| Phase | Tasks | Target Date |
| :--- | :---: | :--- |
| **1. Foundation** | 5 | 02-18 |
| **2. Core Logic** | 8 | 02-27 |
| **3. Integration** | 2 | 03-02 |
| **4. Polish** | 4 | 03-07 |
| **Total** | **19** | |

## Definition of Done

1. [x] **Linted:** `cargo clippy --workspace -- -D warnings` passes.
2. [x] **Tested:** Unit tests covering all type serialization and core client logic.
3. [x] **Formatted:** `cargo fmt --all -- --check` passes.
4. [x] **Verified:** Each task's specific Verification criterion met.
5. [x] **API Parity:** Every resource, method, and type from `opencode-sdk-js` has a Rust equivalent.
6. [x] **No Forbidden Crates:** No `anyhow`, `log`, `reqwest`, `dashmap` used.
