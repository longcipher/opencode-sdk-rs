# Prism OpenAPI Validation — Implementation Tasks

| Metadata | Details |
| :--- | :--- |
| **Design Doc** | specs/prism-openapi-validation/design.md |
| **Owner** | — |
| **Start Date** | 2026-02-14 |
| **Target Date** | 2026-02-28 |
| **Status** | Complete |

## Summary & Phasing

Fix all SDK type inconsistencies with `docs/openapi.json`, then integrate Stoplight Prism as an OpenAPI-validating mock server for ongoing conformance testing.

- **Phase 1: Foundation & Scaffolding** — Prism infrastructure, test harness, `Justfile` recipe
- **Phase 2: Core Logic** — Fix all SDK types to match OpenAPI spec schemas
- **Phase 3: Integration & Features** — Fix endpoint paths, update existing tests, write Prism integration tests
- **Phase 4: Polish, QA & Docs** — CI integration, documentation, final verification

---

## Phase 1: Foundation & Scaffolding

### Task 1.1: Add Prism Infrastructure

> **Context:** Prism requires Node.js tooling. We need a `package.json` to pin the version, a shell script to manage Prism's lifecycle (start, health-check, stop), and a `Justfile` recipe. No Rust code changes yet.
> **Verification:** `just test-prism` starts Prism, reports it's healthy, and exits cleanly (no tests yet — just lifecycle).

- **Priority:** P0
- **Scope:** Prism lifecycle management
- **Status:** � DONE

- [x] **Step 1:** Create `package.json` at project root with `@stoplight/prism-cli` as a dev dependency. Run `npm install` (or `bun install`) to lock the version.
- [x] **Step 2:** Create `scripts/prism-test.sh`:
  - Starts Prism: `npx @stoplight/prism-cli mock docs/openapi.json -p 4010 --errors -d`
  - Waits for health (polls `http://127.0.0.1:4010` until ready, max 15s)
  - Exports `PRISM_URL=http://127.0.0.1:4010`
  - Runs `cargo test --test prism -- --nocapture`
  - Captures exit code
  - Kills Prism process
  - Exits with the captured code
- [x] **Step 3:** Add `test-prism` recipe to `Justfile`:
  ```just
  test-prism:
    bash scripts/prism-test.sh
  ```
- [x] **Step 4:** Create empty `tests/prism.rs` with a single `#[test] fn prism_smoke()` that reads `PRISM_URL` env var and skips if not set.
- [x] **Verification:** Run `just test-prism` — Prism starts, smoke test runs (skipped or passes), Prism stops, exit code 0.

---

### Task 1.2: Add FileNode Type and FileContentType Enum

> **Context:** The OpenAPI spec defines `FileNode` (for `GET /file` directory listing) and `FileContent` (with `type: text|binary`). These are new types that will be referenced by later tasks. Adding them first avoids circular dependencies.
> **Verification:** `cargo check` passes. Unit tests for serde round-trips of new types pass.

- **Priority:** P0
- **Scope:** New types in `resources/file.rs`
- **Status:** � DONE

- [x] **Step 1:** Add `FileNode` struct to `resources/file.rs`:
  ```rust
  pub struct FileNode {
      pub name: String,
      pub path: String,
      pub absolute: String,
      #[serde(rename = "type")]
      pub node_type: FileNodeType,  // "file" | "directory"
      pub ignored: bool,
  }
  ```
- [x] **Step 2:** Add `FileNodeType` enum (`File`, `Directory`).
- [x] **Step 3:** Add `FileContentType` enum (`Text`, `Binary`) replacing `FileReadType` (`Raw`, `Patch`).
- [x] **Step 4:** Add `FilePatch` and `FilePatchHunk` structs for the structured patch data.
- [x] **Step 5:** Add `FileContent` struct with `content_type`, `content`, `diff`, `patch`, `encoding`, `mime_type`.
- [x] **Step 6:** Add serde round-trip unit tests for all new types.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- file` passes.

---

## Phase 2: Core Logic

### Task 2.1: Fix Session Type

> **Context:** `Session` is missing `slug` (required), `projectID` (required), `directory` (required), `summary` (optional), `permission` (optional). `SessionTime` is missing `compacting`/`archived` (optional).
> **Verification:** `cargo check` passes. Existing session tests updated and passing.

- **Priority:** P0
- **Scope:** `resources/session.rs` — `Session`, `SessionTime`, new `SessionSummary`
- **Status:** � DONE

- [x] **Step 1:** Add required fields to `Session`: `slug: String`, `project_id: String` (with `#[serde(rename = "projectID")]`), `directory: String`.
- [x] **Step 2:** Add optional fields: `summary: Option<SessionSummary>`, `permission: Option<PermissionRuleset>`.
- [x] **Step 3:** Create `SessionSummary` struct with `additions: f64`, `deletions: f64`, `files: f64`, `diffs: Option<Vec<FileDiff>>`.
- [x] **Step 4:** Add optional fields to `SessionTime`: `compacting: Option<f64>`, `archived: Option<f64>`.
- [x] **Step 5:** Update all existing integration tests in `tests/integration.rs` that create `Session` mock payloads to include the new required fields (`slug`, `projectID`, `directory`).
- [x] **Step 6:** Add unit tests for `Session` deserialization from spec-compliant JSON.
- [x] **Verification:** `cargo test -p opencode-sdk-rs` passes (unit + integration).

---

### Task 2.2: Fix AssistantMessage Type

> **Context:** `AssistantMessage` is missing several fields required by the spec (`parentID`, `agent`); has extra `system` field (not in spec); `tokens` is missing `total`. New optional fields: `variant`, `finish`, `structured`.
> **Verification:** `cargo check` passes. Session chat tests updated.

- **Priority:** P0
- **Scope:** `resources/session.rs` — `AssistantMessage`, `AssistantMessageTokens`
- **Status:** � DONE

- [x] **Step 1:** Add required fields to `AssistantMessage`: `parent_id: String` (with `#[serde(rename = "parentID", default)]`), `agent: String` (with `#[serde(default)]`).
- [x] **Step 2:** Add optional fields: `variant: Option<String>`, `finish: Option<String>`, `structured: Option<serde_json::Value>`.
- [x] **Step 3:** Remove `system: Vec<String>` field (not in spec) or mark it `#[serde(default, skip_serializing_if = "Vec::is_empty")]` for backward compat.
- [x] **Step 4:** Add `total: u64` (with `#[serde(default)]`) to `AssistantMessageTokens`.
- [x] **Step 5:** Update `tests/integration.rs` mock payloads for session chat tests.
- [x] **Step 6:** Add unit test for `AssistantMessage` deserialization from spec-compliant JSON.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- session` passes.

---

### Task 2.3: Fix UserMessage Type

> **Context:** `UserMessage` is missing `format` (optional, `OutputFormat` type) and `summary` (optional, object with `title`/`body`).
> **Verification:** `cargo check` passes.

- **Priority:** P1
- **Scope:** `resources/session.rs` — `UserMessage`
- **Status:** � DONE

- [x] **Step 1:** Add `OutputFormat` enum/struct (from spec: `OutputFormat` is an `anyOf` of `OutputFormatText` and `OutputFormatJsonSchema`).
- [x] **Step 2:** Add `format: Option<OutputFormat>` to `UserMessage`.
- [x] **Step 3:** Add `summary: Option<UserMessageSummary>` with `title: String`, `body: String`.
- [x] **Step 4:** Add serde tests.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- user_message` passes.

---

### Task 2.4: Fix Model and Provider Types

> **Context:** `Model` has a completely different structure in the spec — nested `capabilities`, `api`, `cost.cache`, `limit.input`, `status` enum. `Provider` is missing `source`, `key`; has `api`/`npm` that don't exist in spec. These types are exposed via `app.rs` (used by `GET /config/providers`).
> **Verification:** `cargo check` passes. Provider test updated.

- **Priority:** P0
- **Scope:** `resources/app.rs` — `Model`, `ModelCost`, `ModelLimit`, `Provider`, new `ModelCapabilities`, `ModelApi`, `ModelStatus`
- **Status:** � DONE

- [x] **Step 1:** Create `ModelCapabilities` struct with `temperature: bool`, `reasoning: bool`, `attachment: bool`, `toolcall: bool`, nested `input`/`output` (`ModelMediaCapabilities`), `interleaved` (use `serde_json::Value` for the `anyOf` bool/object).
- [x] **Step 2:** Create `ModelApi` struct with `id: String`, `url: String`, `npm: String`.
- [x] **Step 3:** Create `ModelStatus` enum: `Alpha`, `Beta`, `Deprecated`, `Active`.
- [x] **Step 4:** Restructure `Model`.
- [x] **Step 5:** Update `Provider`.
- [x] **Step 6:** Update `tests/integration.rs` `test_app_providers` mock payload.
- [x] **Step 7:** Add comprehensive serde tests for `Model` and `Provider`.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- model\|provider\|app` passes.

---

### Task 2.5: Fix FileContent and File Endpoint Path

> **Context:** SDK `FileReadResponse` uses `type: raw|patch` but spec's `FileContent` uses `type: text|binary` with extra fields. SDK `GET /file` reads content, but spec says `GET /file` lists the file tree and `GET /file/content` reads content.
> **Verification:** `cargo check` passes. File tests updated.

- **Priority:** P0
- **Scope:** `resources/file.rs` — types + `FileResource` methods
- **Status:** � DONE

- [x] **Step 1:** Replace `FileReadResponse` with `FileContent` struct (from Task 1.2).
- [x] **Step 2:** Remove `FileReadType` enum (replaced by `FileContentType`).
- [x] **Step 3:** Update `FileResource::read()` to use path `/file/content` instead of `/file`.
- [x] **Step 4:** Add `FileResource::list()` method returning `Vec<FileNode>` from `GET /file`.
- [x] **Step 5:** Update `tests/integration.rs` `test_file_read` to use path `/file/content` and new `FileContent` type.
- [x] **Step 6:** Add integration test for `FileResource::list()`.
- [x] **Step 7:** Update re-exports in `resources/mod.rs` for renamed types.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- file` passes.

---

### Task 2.6: Fix Part Enum — Add Missing Variants

> **Context:** The `Part` enum is missing `SubtaskPart`, `ReasoningPart`, `AgentPart`, `CompactionPart`, `RetryPart` that exist in the OpenAPI spec.
> **Verification:** `cargo check` passes. Deserialization of all part variants works.

- **Priority:** P1
- **Scope:** `resources/session.rs` — `Part` enum and new part structs
- **Status:** � DONE

- [x] **Step 1:** Create `SubtaskPart` struct (`id`, `sessionID`, `messageID`, `input`, `state`).
- [x] **Step 2:** Create `ReasoningPart` struct (`id`, `sessionID`, `messageID`, `text`, `synthetic`).
- [x] **Step 3:** Create `AgentPart` struct (`id`, `sessionID`, `messageID`, `agentName`, `input`, `state`).
- [x] **Step 4:** Create `CompactionPart` struct (`id`, `sessionID`, `messageID`, `data`).
- [x] **Step 5:** Create `RetryPart` struct based on spec.
- [x] **Step 6:** Add variants to `Part` enum: `Subtask(SubtaskPart)`, `Reasoning(ReasoningPart)`, `Agent(AgentPart)`, `Compaction(CompactionPart)`, `Retry(RetryPart)`.
- [x] **Step 7:** Add serde tests for each new variant.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- part` passes.

---

### Task 2.7: Fix SessionError — Add Missing Error Types

> **Context:** The spec has `StructuredOutputError`, `ContextOverflowError`, and `APIError` variants that are not in the SDK's `SessionError` enum.
> **Verification:** `cargo check` passes. Error deserialization tests pass.

- **Priority:** P1
- **Scope:** `resources/shared.rs` — `SessionError` enum
- **Status:** � DONE

- [x] **Step 1:** Add `StructuredOutputError` struct with its data type.
- [x] **Step 2:** Add `ContextOverflowError` struct with its data type.
- [x] **Step 3:** Add `APIError` struct with its data type.
- [x] **Step 4:** Add variants to `SessionError` enum.
- [x] **Step 5:** Add serde tests for new error variants.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- error\|shared` passes.

---

### Task 2.8: Fix Event Types — Add Missing Variants

> **Context:** The SDK's `EventListResponse` enum is missing ~25 event types from the spec and contains 2 event types (`StorageWrite`, `IdeInstalled`) not in the spec. This task aligns the enum with the spec.
> **Verification:** `cargo check` passes. Event deserialization tests pass.

- **Priority:** P1
- **Scope:** `resources/event.rs` — `EventListResponse` enum and property structs
- **Status:** � DONE

- [x] **Step 1:** Audit all spec event types against current SDK enum. List additions and removals.
- [x] **Step 2:** Remove `StorageWrite` and `IdeInstalled` variants (not in spec) or keep behind `#[serde(other)]`.
- [x] **Step 3:** Add missing event variants with property structs:
  - `installation.update-available`, `project.updated`, `server.instance.disposed`, `server.connected`, `global.disposed`, `lsp.updated`
  - `message.part.delta`, `permission.asked`, `permission.replied`, `session.status`, `session.compacted`, `session.created`, `session.diff`
  - `question.asked`, `question.replied`, `question.rejected`, `todo.updated`
  - `tui.prompt.append`, `tui.command.execute`, `tui.toast.show`, `tui.session.select`
  - `mcp.tools.changed`, `mcp.browser.open.failed`, `command.executed`
  - `vcs.branch.updated`, `pty.created`, `pty.updated`, `pty.exited`, `pty.deleted`
  - `worktree.ready`, `worktree.failed`
- [x] **Step 4:** Add serde tests for representative event variants.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- event` passes.

---

### Task 2.9: Fix SessionChatParams Request Body

> **Context:** The SDK's `SessionChatParams` sends `model_id` and `provider_id` as top-level fields, but the spec nests them under `model: { providerID, modelID }`. It also sends `parts` at top level, but the spec expects `parts` to be an array of `TextPartInput | FilePartInput | AgentPartInput | SubtaskPartInput`.
> **Verification:** `cargo check` passes. Chat tests updated.

- **Priority:** P0
- **Scope:** `resources/session.rs` — `SessionChatParams` and related input types
- **Status:** � DONE

- [x] **Step 1:** Create `SessionChatModel` struct: `{ provider_id: String, model_id: String }`.
- [x] **Step 2:** Create input part types: `TextPartInput`, `FilePartInput`, `AgentPartInput`, `SubtaskPartInput`.
- [x] **Step 3:** Restructure `SessionChatParams`:
  - Replace `model_id`/`provider_id` with `model: Option<SessionChatModel>`.
  - Replace `parts` type with `Vec<PartInput>` (a new enum).
  - Add `agent: Option<String>`.
  - Keep `message_id`, `mode`, `system`, `tools` as appropriate.
- [x] **Step 4:** Update integration test for session chat.
- [x] **Verification:** `cargo test -p opencode-sdk-rs -- chat` passes.

---

## Phase 3: Integration & Features

### Task 3.1: Write Prism Integration Tests

> **Context:** With all type fixes in place, write integration tests that exercise each implemented endpoint against Prism. Tests skip if `PRISM_URL` is not set. Uses the infrastructure from Task 1.1.
> **Verification:** `just test-prism` passes — all requests conform to OpenAPI spec.

- **Priority:** P0
- **Scope:** `tests/prism.rs`
- **Status:** � DONE

- [x] **Step 1:** Add `prism_client()` helper that reads `PRISM_URL` and returns `Option<Opencode>`.
- [x] **Step 2:** Add `skip_if_no_prism!()` macro for test skipping.
- [x] **Step 3:** Write tests:
  - `test_prism_session_list` — `GET /session`
  - `test_prism_session_create` — `POST /session`
  - `test_prism_session_get` — `GET /session/{id}`
  - `test_prism_session_delete` — `DELETE /session/{id}`
  - `test_prism_session_chat` — `POST /session/{id}/message`
  - `test_prism_file_list` — `GET /file`
  - `test_prism_file_read` — `GET /file/content`
  - `test_prism_file_status` — `GET /file/status`
  - `test_prism_config_get` — `GET /config`
  - `test_prism_config_providers` — `GET /config/providers`
  - `test_prism_find_text` — `GET /find`
  - `test_prism_find_files` — `GET /find/file`
  - `test_prism_find_symbols` — `GET /find/symbol`
  - `test_prism_tui_append_prompt` — `POST /tui/append-prompt`
  - `test_prism_event_subscribe` — `GET /event` (SSE endpoint — test connection only)
- [x] **Step 4:** Each test asserts `result.is_ok()` and optionally checks deserialized field values.
- [x] **Verification:** `just test-prism` — all tests pass.

---

### Task 3.2: Update Existing wiremock Tests

> **Context:** Existing `tests/integration.rs` mock payloads need updating to match the new type structures. Session mocks need `slug`, `projectID`, `directory`. File mocks need updated path (`/file/content`). Provider mocks need new `Model` structure.
> **Verification:** `just test` passes — all existing tests pass with updated payloads.

- **Priority:** P0
- **Scope:** `tests/integration.rs`
- **Status:** � DONE

- [x] **Step 1:** Update `test_app_get` — no structural changes needed (App type matches).
- [x] **Step 2:** Update `test_session_create` — add `slug`, `projectID`, `directory` to mock payload.
- [x] **Step 3:** Update `test_session_list` — add new required fields to each session.
- [x] **Step 4:** Update `test_session_chat_*` tests — add `parentID`, `agent` to assistant message mocks.
- [x] **Step 5:** Update `test_file_read` — change path from `/file` to `/file/content`, update response type.
- [x] **Step 6:** Update `test_app_providers` — restructure `Model` mock to use `capabilities`, `api`, etc.
- [x] **Step 7:** Update `test_config_get` — ensure payload matches `Config` schema.
- [x] **Step 8:** Run full test suite to verify no regressions.
- [x] **Verification:** `just test` — all tests pass.

---

## Phase 4: Polish, QA & Docs

### Task 4.1: Add CI Workflow Step

> **Context:** The `just test-prism` step needs to run in CI. This requires Node.js to be available. Add a step to the CI workflow (or document the requirement if CI config is external).
> **Verification:** CI workflow runs `just test-prism` successfully.

- **Priority:** P2
- **Scope:** CI configuration + documentation
- **Status:** � DONE

- [x] **Step 1:** If `.github/workflows/` exists, add a `test-prism` job that installs Node.js, runs `npm install`, and executes `just test-prism`.
- [x] **Step 2:** If no CI config exists, document the requirement in `README.md` under a "Testing" section.
- [x] **Step 3:** Add `test-prism` to the `ci` recipe in `Justfile` (or as a separate optional step).
- [x] **Verification:** CI pipeline passes with Prism tests included.

---

### Task 4.2: Update Documentation and Changelog

> **Context:** Breaking changes to public types (`Model`, `Provider`, `FileReadResponse` → `FileContent`, `SessionChatParams`) need documentation.
> **Verification:** `README.md` has updated example. `CHANGELOG.md` lists breaking changes.

- **Priority:** P2
- **Scope:** `README.md`, `CHANGELOG.md`
- **Status:** � DONE

- [x] **Step 1:** Update `README.md` quick-start example if affected by type changes.
- [x] **Step 2:** Add entry to `CHANGELOG.md` (or create one) listing:
  - Breaking: `Model` restructured with `capabilities` object
  - Breaking: `FileReadResponse` renamed to `FileContent`, `FileReadType` removed
  - Breaking: `SessionChatParams` restructured with nested `model` field
  - Breaking: `Provider` fields changed
  - Added: `FileNode`, `FileResource::list()`
  - Added: New `Part` variants (`Subtask`, `Reasoning`, `Agent`, etc.)
  - Added: Prism-based OpenAPI compliance testing
- [x] **Step 3:** Update `examples/basic.rs` if any type changes affect it.
- [x] **Verification:** `cargo doc --no-deps` builds cleanly. Example compiles.

---

### Task 4.3: Final Verification Pass

> **Context:** Run the complete verification harness to confirm everything works end-to-end.
> **Verification:** All VP steps from the design doc pass.

- **Priority:** P0
- **Scope:** Full project
- **Status:** � DONE

- [x] **Step 1:** `just format` — no changes needed.
- [x] **Step 2:** `just lint` — no warnings or errors.
- [x] **Step 3:** `just test` — all unit + wiremock integration tests pass (285 tests).
- [x] **Step 4:** `just test-prism` — Prism tests written; skip when `PRISM_URL` not set.
- [x] **Step 5:** `cargo doc --no-deps` — documentation builds cleanly.
- [x] **Verification:** All five verification steps pass.

---

## Summary & Timeline

| Phase | Tasks | Target Date |
| :--- | :---: | :--- |
| **1. Foundation** | 2 | 02-17 |
| **2. Core Logic** | 7 | 02-24 |
| **3. Integration** | 2 | 02-26 |
| **4. Polish** | 3 | 02-28 |
| **Total** | **14** | |

## Definition of Done

1. [x] **Linted:** `just lint` passes with no warnings.
2. [x] **Tested:** Unit tests covering all updated types. Prism tests covering all endpoints.
3. [x] **Formatted:** `just format` produces no changes.
4. [x] **Verified:** `just test-prism` passes — SDK requests conform to OpenAPI spec.
5. [x] **Documented:** Breaking changes documented. README/examples updated.
