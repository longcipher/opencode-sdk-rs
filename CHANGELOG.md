# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Breaking Changes

- **`Model` restructured** — Flat boolean fields (`attachment`, `reasoning`, `temperature`, `tool_call`) replaced with nested `capabilities: ModelCapabilities` object containing `ModelMediaCapabilities` for input/output media support. Added `api: ModelApi`, `status: ModelStatus` enum, `headers`, `variants` fields.
- **`ModelCost` restructured** — `cache_read`/`cache_write` replaced with nested `cache: CostCache`. Added `experimental_over_200k: Option<CostExperimentalOver200K>`.
- **`ModelLimit` updated** — Added `input: Option<u64>` field.
- **`Provider` restructured** — Removed `api`/`npm` fields. Added `source: ProviderSource` enum (`Env`, `Npm`, `Config`), `key: Option<String>`, `options: HashMap`.
- **`FileReadResponse` removed** — Replaced by `FileContent` with `type: text|binary` and additional fields (`diff`, `patch`, `encoding`, `mime_type`).
- **`FileReadType` removed** — Replaced by `FileContentType` enum (`Text`, `Binary`).
- **`FileResource::read()` endpoint changed** — Now hits `/file/content` instead of `/file`.
- **`SessionChatParams` restructured** — Flat `model_id`/`provider_id` replaced with nested `model: Option<SessionChatModel>`. Removed `mode` field. Added `agent`, `no_reply`, `format`, `variant` fields.
- **`PartInput` enum extended** — Added `Agent(AgentPartInput)` and `Subtask(SubtaskPartInput)` variants.
- **`MessageAbortedError` data field** — Changed from `Option<serde_json::Value>` to typed `MessageAbortedErrorData { message: Option<String> }`.
- **`SessionError` dropped `Eq`** — Now only derives `PartialEq` due to `f64` fields in new error variants.
- **Event variants removed** — `StorageWrite`, `PermissionUpdated`, `IdeInstalled` removed from `EventListResponse` (not in OpenAPI spec).
- **`FileWatcherEvent` changed** — `Rename` replaced with `Add` and `Unlink` to match spec (`add`/`change`/`unlink`).

### Added

- **`FileNode`** and **`FileNodeType`** types for directory listing.
- **`FileResource::list()`** method returning `Vec<FileNode>` from `GET /file`.
- **`FilePatch`**, **`FilePatchHunk`** structs for structured patch data.
- **Session fields** — `slug`, `project_id`, `directory` (required); `summary`, `permission` (optional).
- **`SessionTime` fields** — `compacting`, `archived` (optional).
- **`SessionSummary`**, **`FileDiff`**, **`FileDiffStatus`**, **`PermissionRule`**, **`PermissionRuleset`** types.
- **`AssistantMessage` fields** — `parent_id`, `agent` (required); `variant`, `finish`, `structured` (optional).
- **`AssistantMessageTokens.total`** field.
- **`UserMessage` fields** — `agent`, `model`, `format`, `summary`, `system`, `tools`, `variant`.
- **`OutputFormat`** enum (`Text`, `JsonSchema`), **`UserMessageSummary`**, **`UserMessageModel`** types.
- **New `Part` variants** — `Subtask`, `Reasoning`, `Agent`, `Compaction`, `Retry` with supporting structs.
- **New `SessionError` variants** — `StructuredOutputError`, `ContextOverflowError`, `APIError` with typed data structs.
- **30 new event variants** in `EventListResponse` matching the OpenAPI spec, including `message.part.delta`, `permission.asked`/`replied`, `session.status`/`created`/`compacted`/`diff`, `question.*`, `todo.updated`, `tui.*`, `mcp.*`, `pty.*`, `worktree.*`, `vcs.branch.updated`, `command.executed`.
- **Supporting event types** — `Todo`, `Pty`, `PtyStatus`, `ToastVariant`, `PermissionReply`, `EmptyProps`, and many props structs.
- **`AgentPartInput`**, **`SubtaskPartInput`** for chat request parts.
- **`SessionChatModel`** for nested model selection in chat params.
- **Prism-based OpenAPI contract tests** — Validates SDK conformance against `docs/openapi.json` using Stoplight Prism.
