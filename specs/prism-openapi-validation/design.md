# Design Document: Prism OpenAPI Validation

| Metadata | Details |
| :--- | :--- |
| **Author** | pb-plan agent |
| **Status** | Draft |
| **Created** | 2026-02-14 |
| **Reviewers** | — |
| **Related Issues** | N/A |

## 1. Executive Summary

**Problem:** The SDK types and API endpoint implementations have drifted out of sync with `docs/openapi.json`. Multiple schemas (`Session`, `AssistantMessage`, `Model`, `FileContent`, `Provider`, etc.) contain missing fields, incorrect field names, and structural mismatches. The existing `wiremock`-based tests use hand-crafted JSON payloads that are never validated against the OpenAPI spec, so regressions are invisible. There is no automated mechanism to catch future drift.

**Solution:** (1) Fix all identified inconsistencies between SDK types and the OpenAPI spec. (2) Integrate Stoplight Prism as an OpenAPI-aware mock server into the test suite. Prism validates both outgoing requests and response deserialization against `docs/openapi.json`, ensuring type-level compliance. (3) Add a CI step that runs Prism-backed integration tests, guaranteeing ongoing conformance whenever the spec or SDK changes.

---

## 2. Requirements & Goals

### 2.1 Problem Statement

The current SDK has the following categories of inconsistency with `docs/openapi.json`:

**Schema-level drift (types):**

| SDK Type | Issue |
| :--- | :--- |
| `Session` | Missing `slug`, `projectID`, `directory`, `summary`, `permission`. `time` missing `compacting`/`archived`. |
| `AssistantMessage` | Missing `parentID`, `agent`, `variant`, `finish`, `structured`. `tokens` missing `total`. `system` field not in spec. |
| `UserMessage` | Missing `format`, `summary` fields. |
| `Model` | Flat booleans (`attachment`, `reasoning`, `temperature`, `tool_call`) vs spec's nested `capabilities` object. `cost` structure wrong (`cache_read`/`cache_write` vs nested `cache.read`/`cache.write`). Missing `providerID`, `api`, `family`, `status`, `headers`, `variants`. `limit` missing `input`. |
| `Provider` | Missing `source`, `key` fields. Has `api`/`npm` that don't exist in spec. |
| `FileReadResponse` / `FileContent` | SDK uses `type: raw\|patch`; spec uses `type: text\|binary` with extra fields (`diff`, `patch`, `encoding`, `mimeType`). |
| `Part` enum | Missing `SubtaskPart`, `ReasoningPart`, `AgentPart`, `CompactionPart`, `RetryPart` variants. |
| `EventListResponse` | Many event types missing vs spec's 40+ event types. Has `StorageWrite`/`IdeInstalled` not in spec. |

**Endpoint-level drift:**

| SDK Endpoint | Issue |
| :--- | :--- |
| `GET /file` | SDK reads file content; spec says this returns `FileNode[]` (file tree). SDK should use `GET /file/content` for reading content. |
| `POST /session/{id}/message` | Request body schema uses `modelID`/`providerID` at top level; spec nests them under `model: { providerID, modelID }`. |
| `GET /mode` | Not in spec (modes are part of config). |
| Many endpoints | Not implemented: auth, project, pty, mcp, provider, permission, question, worktree, command, lsp, formatter, vcs, path, agent, skill. |

### 2.2 Functional Goals

1. **FG-1:** Fix all SDK types to match `docs/openapi.json` schemas exactly, using `#[serde(default)]` / `Option<T>` for optional fields.
2. **FG-2:** Fix endpoint URL mappings where SDK paths differ from spec (e.g., `GET /file` → `GET /file/content`).
3. **FG-3:** Integrate Stoplight Prism as a dev dependency for OpenAPI-validated mock testing.
4. **FG-4:** Create a Prism-backed integration test suite (`tests/prism_integration.rs`) that exercises all currently implemented endpoints against the OpenAPI spec.
5. **FG-5:** Add a `just test-prism` command and CI step that starts Prism, runs tests, and stops Prism.
6. **FG-6:** Retain existing `wiremock` tests for edge-case / failure-mode testing (retries, timeouts, error handling).

### 2.3 Non-Functional Goals

- **Reliability:** Prism tests must be deterministic — no flaky failures from port conflicts or startup races.
- **Performance:** Prism startup/shutdown should add < 5 seconds to the test cycle.
- **Developer Experience:** Running `just test-prism` should require only `npm` (or `npx`) — no global installs. The Prism binary can be invoked via `npx @stoplight/prism-cli`.
- **CI Compatibility:** Tests must pass in GitHub Actions with Node.js available.

### 2.4 Out of Scope

- Implementing new resource modules for unimplemented endpoints (auth, project, pty, mcp, etc.). These can be added in a follow-up feature.
- Modifying `docs/openapi.json` — it is treated as the source of truth.
- End-to-end testing against a real OpenCode server.
- Response body validation (Prism validates requests; response schemas are checked by serde deserialization).

### 2.5 Assumptions

- `docs/openapi.json` is the authoritative, current API specification.
- Node.js / npm is available in development and CI environments for running Prism.
- Prism's `--errors` flag correctly validates request bodies, query params, and headers against the spec.
- Fields present in the spec but not yet used by the SDK can be added as `Option<T>` with `#[serde(default)]` / `#[serde(skip_serializing_if = "Option::is_none")]` to maintain backward compatibility.
- The `wiremock` tests remain valuable for testing error handling, retries, timeouts, and edge cases that Prism cannot simulate.

---

## 3. Architecture Overview

### 3.1 System Context

```text
┌──────────────────────────────┐
│  Rust Test Suite             │
│                              │
│  ┌────────────────────────┐  │
│  │ tests/integration.rs   │  │  wiremock (existing)
│  │ (edge cases, errors)   │──┼──── hand-crafted mocks
│  └────────────────────────┘  │
│                              │
│  ┌────────────────────────┐  │
│  │ tests/prism.rs         │  │  Prism mock server
│  │ (OpenAPI compliance)   │──┼──── validates requests against
│  └────────────────────────┘  │     docs/openapi.json
│                              │
└──────────────────────────────┘

Prism lifecycle:
  1. `just test-prism` starts Prism on a dynamic port
  2. Rust tests read `PRISM_URL` env var
  3. Tests send SDK requests → Prism validates → returns mock responses
  4. Any request that violates the spec causes Prism to return 422
  5. Test asserts `result.is_ok()` (valid) or specific error shape
  6. Prism is stopped after tests complete
```

### 3.2 Key Design Principles

1. **Spec is truth.** `docs/openapi.json` defines correctness. SDK types are derived from it.
2. **Two-layer testing.** Prism tests for spec compliance; wiremock tests for behavioral edge cases.
3. **Fail-fast validation.** Prism with `--errors` rejects invalid requests immediately.
4. **Minimal footprint.** No new Rust dependencies — only `npx` for Prism at test time.
5. **Forward-compatible types.** Use `Option<T>` + `#[serde(default)]` for new fields; `#[serde(other)]` for unknown enum variants.

### 3.3 Existing Components to Reuse

| Component | Location | How to Reuse |
| :--- | :--- | :--- |
| `client_for()` helper | `tests/integration.rs` | Adapt for Prism base URL (read from env var) |
| `ClientOptions` | `src/config.rs` | Use `base_url` to point at Prism |
| `Opencode` client | `src/client.rs` | Use directly in Prism tests — it's the system under test |
| `RequestOptions` | `src/client.rs` | Set `max_retries: 0` for deterministic tests |
| `wiremock` test patterns | `tests/integration.rs` | Keep for error/retry/timeout tests |
| `Justfile` | root | Add `test-prism` recipe alongside existing `test` |

---

## 4. Detailed Design

### 4.1 Module Structure

```text
Changes to existing files:
  crates/opencode-sdk-rs/src/resources/
    session.rs      — Fix Session, AssistantMessage, UserMessage, Part enum
    app.rs          — Fix Model, Provider, Mode types
    file.rs         — Rename FileReadResponse → FileContent, fix type enum, fix endpoint path
    config.rs       — Align Config schema
    event.rs        — Add missing event variants
    shared.rs       — Add StructuredOutputError, ContextOverflowError, APIError

New files:
  crates/opencode-sdk-rs/tests/prism.rs            — Prism integration tests
  scripts/prism-test.sh                             — Script to start/stop Prism
  package.json                                      — Pin @stoplight/prism-cli version
```

### 4.2 Data Structures & Types

**Session (updated):**
```rust
pub struct Session {
    pub id: String,
    pub slug: String,                          // NEW (required)
    #[serde(rename = "projectID")]
    pub project_id: String,                    // NEW (required)
    pub directory: String,                     // NEW (required)
    pub title: String,
    pub version: String,
    pub time: SessionTime,
    #[serde(rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<SessionSummary>,       // NEW
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<SessionShare>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revert: Option<SessionRevert>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<PermissionRuleset>, // NEW
}
```

**Model (updated to match spec's nested structure):**
```rust
pub struct Model {
    pub id: String,
    #[serde(rename = "providerID")]
    pub provider_id: String,                   // NEW
    pub api: ModelApi,                         // NEW nested obj
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,                // NEW
    pub capabilities: ModelCapabilities,        // CHANGED: replaces flat bools
    pub cost: ModelCost,                       // CHANGED: nested cache
    pub limit: ModelLimit,                     // CHANGED: added input
    pub status: ModelStatus,                   // NEW enum
    pub options: HashMap<String, Value>,
    pub headers: HashMap<String, String>,      // NEW
    pub release_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<HashMap<String, HashMap<String, Value>>>, // NEW
}
```

**FileContent (renamed, aligned):**
```rust
pub struct FileContent {
    #[serde(rename = "type")]
    pub content_type: FileContentType,   // text | binary
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<FilePatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}
```

### 4.3 Interface Design

**File resource path fix:**
```rust
impl FileResource {
    /// List files and directories.
    /// `GET /file`
    pub async fn list(&self) -> Result<Vec<FileNode>, OpencodeError> { ... }

    /// Read file content.
    /// `GET /file/content?path=<path>`  (was GET /file)
    pub async fn read(&self, params: &FileReadParams) -> Result<FileContent, OpencodeError> { ... }

    /// Get file status.
    /// `GET /file/status`
    pub async fn status(&self) -> Result<Vec<FileInfo>, OpencodeError> { ... }
}
```

**Prism test helper:**
```rust
fn prism_client() -> Option<Opencode> {
    let url = std::env::var("PRISM_URL").ok()?;
    Some(Opencode::with_options(&ClientOptions {
        base_url: Some(url),
        max_retries: Some(0),
        timeout: Some(Duration::from_secs(10)),
        ..ClientOptions::empty()
    }).expect("client should build"))
}
```

### 4.4 Logic Flow

1. Developer runs `just test-prism`.
2. Script starts Prism: `npx @stoplight/prism-cli mock docs/openapi.json -p 0 --errors`.
3. Script captures the port and exports `PRISM_URL=http://127.0.0.1:<port>`.
4. Script runs `cargo test --test prism -- --nocapture`.
5. Each test:
   a. Reads `PRISM_URL` env var → creates `Opencode` client.
   b. Calls SDK method (e.g., `client.session().list()`).
   c. If Prism returns 2xx → deserialization succeeds → test passes.
   d. If Prism returns 422 → SDK sent invalid request → test fails with actionable error.
6. Script kills Prism process.

### 4.5 Configuration

| Item | Value | Notes |
| :--- | :--- | :--- |
| `PRISM_URL` | `http://127.0.0.1:<dynamic>` | Env var read by Prism tests |
| Prism flags | `mock --errors -p 0` | Dynamic port, strict request validation |
| `package.json` | `@stoplight/prism-cli: "^4"` | Pinned via `npx` |

### 4.6 Error Handling

- If `PRISM_URL` is not set, Prism tests are skipped (not failed) — allows `cargo test` to work without Prism.
- Prism 422 responses are mapped to `OpencodeError::Api { status: 422, .. }` — test assertions check for this.
- Prism startup failure in `scripts/prism-test.sh` causes immediate exit with non-zero code.
- Timeout waiting for Prism health check → script exits with error after 15s.

---

## 5. Verification & Testing Strategy

### 5.1 Unit Testing

- All updated types must have round-trip serde tests (`serialize → deserialize` and vice versa) using JSON payloads matching `openapi.json` examples.
- Existing unit tests in each resource module are updated to use spec-compliant payloads.

### 5.2 Integration Testing

- **Existing `tests/integration.rs`:** Updated mock payloads to match new types. Retains error/retry/timeout tests.
- **New `tests/prism.rs`:** One test per implemented endpoint. Each test calls the SDK method and asserts success. If Prism rejects the request, the test fails.

### 5.3 Critical Path Verification (The "Harness")

| Verification Step | Command | Success Criteria |
| :--- | :--- | :--- |
| **VP-01** | `just format` | No formatting changes |
| **VP-02** | `just lint` | No warnings or errors |
| **VP-03** | `just test` | All existing + updated unit/integration tests pass |
| **VP-04** | `just test-prism` | All Prism integration tests pass (requests conform to OpenAPI spec) |
| **VP-05** | `cargo doc --no-deps` | Documentation builds without warnings |

### 5.4 Validation Rules

| Test Case ID | Action | Expected Outcome | Verification Method |
| :--- | :--- | :--- | :--- |
| **TC-01** | `GET /session` via SDK → Prism | 200 + valid `Vec<Session>` deserialization | `assert!(result.is_ok())` |
| **TC-02** | `POST /session` via SDK → Prism | 200 + valid `Session` deserialization | `assert!(result.is_ok())` |
| **TC-03** | `GET /file/content?path=x` via SDK → Prism | 200 + valid `FileContent` deserialization | `assert!(result.is_ok())` |
| **TC-04** | `GET /file` via SDK → Prism | 200 + valid `Vec<FileNode>` deserialization | `assert!(result.is_ok())` |
| **TC-05** | `GET /config` via SDK → Prism | 200 + valid `Config` deserialization | `assert!(result.is_ok())` |
| **TC-06** | `GET /config/providers` via SDK → Prism | 200 + valid response deserialization | `assert!(result.is_ok())` |
| **TC-07** | `GET /find?pattern=x` via SDK → Prism | 200 + valid `FindTextResponse` | `assert!(result.is_ok())` |
| **TC-08** | `POST /session/{id}/message` via SDK → Prism | 200 + valid response (request body validated) | `assert!(result.is_ok())` |
| **TC-09** | `POST /tui/append-prompt` via SDK → Prism | 200 + valid response | `assert!(result.is_ok())` |
| **TC-10** | Send malformed request manually → Prism | 422 error | Prism returns validation error |

---

## 6. Implementation Plan

- [ ] **Phase 1: Foundation** — Prism infrastructure (script, Justfile, package.json, test skeleton)
- [ ] **Phase 2: Core Logic** — Fix all SDK types to match OpenAPI spec
- [ ] **Phase 3: Integration** — Fix endpoint paths, update existing tests, write Prism tests
- [ ] **Phase 4: Polish** — CI integration, docs, cleanup

---

## 7. Cross-Functional Concerns

- **Backward Compatibility:** Changing `FileReadResponse` → `FileContent` and `Model` structure are breaking API changes. Callers must update. Document in changelog.
- **Migration:** Users referencing `FileReadType::Raw` / `FileReadType::Patch` must switch to `FileContentType::Text` / `FileContentType::Binary`.
- **CI:** GitHub Actions workflow needs `actions/setup-node` step for Prism.
- **Versioning:** This is a semver-minor or semver-major bump depending on the project's stability guarantees (pre-1.0, so minor is acceptable).
