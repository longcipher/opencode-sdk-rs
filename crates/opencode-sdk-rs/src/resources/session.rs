//! Session resource types and methods mirroring the JS SDK's `resources/session.ts`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::shared::SessionError;
use crate::{
    client::{Opencode, RequestOptions},
    error::OpencodeError,
};

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// A conversation session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    /// Unique session identifier.
    pub id: String,
    /// URL-friendly session slug.
    #[serde(default)]
    pub slug: String,
    /// Project identifier.
    #[serde(rename = "projectID", default)]
    pub project_id: String,
    /// Working directory for this session.
    #[serde(default)]
    pub directory: String,
    /// Timing information.
    pub time: SessionTime,
    /// Human-readable session title.
    pub title: String,
    /// Session schema version.
    pub version: String,
    /// Parent session identifier (for branched sessions).
    #[serde(rename = "parentID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Revert metadata, if the session was reverted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revert: Option<SessionRevert>,
    /// Share metadata, if the session was shared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<SessionShare>,
    /// Summary of changes in this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<SessionSummary>,
    /// Permission rules for this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<PermissionRuleset>,
}

/// Timing information for a [`Session`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionTime {
    /// Epoch timestamp when the session was created.
    pub created: f64,
    /// Epoch timestamp when the session was last updated.
    pub updated: f64,
    /// Epoch timestamp when compaction started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compacting: Option<f64>,
    /// Epoch timestamp when the session was archived.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<f64>,
}

/// Revert metadata attached to a [`Session`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionRevert {
    /// The message that was reverted to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// Optional diff content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    /// Optional part identifier.
    #[serde(rename = "partID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_id: Option<String>,
    /// Optional snapshot content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
}

/// Share metadata attached to a [`Session`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionShare {
    /// Public URL of the shared session.
    pub url: String,
}

/// Status of a file diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileDiffStatus {
    /// File was added.
    Added,
    /// File was deleted.
    Deleted,
    /// File was modified.
    Modified,
}

/// A file diff within a session summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileDiff {
    /// The file path.
    pub file: String,
    /// Content before the change.
    pub before: String,
    /// Content after the change.
    pub after: String,
    /// Number of added lines.
    pub additions: f64,
    /// Number of deleted lines.
    pub deletions: f64,
    /// Optional diff status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<FileDiffStatus>,
}

/// Summary of changes in a session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionSummary {
    /// Number of additions.
    pub additions: f64,
    /// Number of deletions.
    pub deletions: f64,
    /// Number of files changed.
    pub files: f64,
    /// Optional list of file diffs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffs: Option<Vec<FileDiff>>,
}

/// A permission rule governing tool access.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionRule {
    /// The permission name.
    pub permission: String,
    /// Glob pattern for matching.
    pub pattern: String,
    /// Action to take (use string for flexibility with `PermissionAction`).
    pub action: String,
}

/// A set of permission rules.
pub type PermissionRuleset = Vec<PermissionRule>;

// ---------------------------------------------------------------------------
// Output Format
// ---------------------------------------------------------------------------

/// Output format specification — text or JSON schema.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OutputFormat {
    /// Plain text output.
    #[serde(rename = "text")]
    Text,
    /// JSON schema output.
    #[serde(rename = "json_schema")]
    JsonSchema {
        /// The JSON schema.
        schema: serde_json::Value,
        /// Number of retries for structured output.
        #[serde(rename = "retryCount", skip_serializing_if = "Option::is_none")]
        retry_count: Option<u64>,
    },
}

// ---------------------------------------------------------------------------
// User Message Supporting Types
// ---------------------------------------------------------------------------

/// Summary information for a user message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMessageSummary {
    /// Optional summary title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional summary body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// File diffs.
    pub diffs: Vec<FileDiff>,
}

/// Model information for a user message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UserMessageModel {
    /// Provider identifier.
    #[serde(rename = "providerID")]
    pub provider_id: String,
    /// Model identifier.
    #[serde(rename = "modelID")]
    pub model_id: String,
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// A user-sent message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMessage {
    /// Unique message identifier.
    pub id: String,
    /// The session this message belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Timing information.
    pub time: UserMessageTime,
    /// Agent name.
    #[serde(default)]
    pub agent: String,
    /// Model information.
    #[serde(default)]
    pub model: UserMessageModel,
    /// Output format specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,
    /// Summary information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<UserMessageSummary>,
    /// System prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Map of tool names to enabled state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, bool>>,
    /// Variant identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// Timing information for a [`UserMessage`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMessageTime {
    /// Epoch timestamp when the message was created.
    pub created: f64,
}

/// An assistant-generated message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssistantMessage {
    /// Unique message identifier.
    #[serde(default)]
    pub id: String,
    /// Monetary cost of generating this message.
    #[serde(default)]
    pub cost: f64,
    /// The mode used for generation.
    #[serde(default)]
    pub mode: String,
    /// The model identifier used.
    #[serde(rename = "modelID", default)]
    pub model_id: String,
    /// Filesystem paths relevant to this message.
    #[serde(default)]
    pub path: AssistantMessagePath,
    /// The provider identifier used.
    #[serde(rename = "providerID", default)]
    pub provider_id: String,
    /// The session this message belongs to.
    #[serde(rename = "sessionID", default)]
    pub session_id: String,
    /// Parent message identifier.
    #[serde(rename = "parentID", default)]
    pub parent_id: String,
    /// Agent name.
    #[serde(default)]
    pub agent: String,
    /// System prompt segments (not in `OpenAPI` spec, kept for backward compatibility).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub system: Vec<String>,
    /// Timing information.
    #[serde(default)]
    pub time: AssistantMessageTime,
    /// Token usage breakdown.
    #[serde(default)]
    pub tokens: AssistantMessageTokens,
    /// Optional error that occurred during generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SessionError>,
    /// Whether this message is a summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<bool>,
    /// Variant identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Finish reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish: Option<String>,
    /// Structured output data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured: Option<serde_json::Value>,
}

/// Filesystem paths for an [`AssistantMessage`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AssistantMessagePath {
    /// Current working directory.
    #[serde(default)]
    pub cwd: String,
    /// Project root directory.
    #[serde(default)]
    pub root: String,
}

/// Timing information for an [`AssistantMessage`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AssistantMessageTime {
    /// Epoch timestamp when the message was created.
    #[serde(default)]
    pub created: f64,
    /// Epoch timestamp when generation completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<f64>,
}

/// Token usage breakdown for an [`AssistantMessage`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AssistantMessageTokens {
    /// Cache token details.
    #[serde(default)]
    pub cache: TokenCache,
    /// Number of input tokens.
    #[serde(default)]
    pub input: u64,
    /// Number of output tokens.
    #[serde(default)]
    pub output: u64,
    /// Number of reasoning tokens.
    #[serde(default)]
    pub reasoning: u64,
    /// Total number of tokens.
    #[serde(default)]
    pub total: u64,
}

/// Cache token breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TokenCache {
    /// Number of tokens read from cache.
    #[serde(default)]
    pub read: u64,
    /// Number of tokens written to cache.
    #[serde(default)]
    pub write: u64,
}

/// A message in a session — either from the user or the assistant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "role")]
pub enum Message {
    /// A user-sent message.
    #[serde(rename = "user")]
    User(Box<UserMessage>),
    /// An assistant-generated message.
    #[serde(rename = "assistant")]
    Assistant(Box<AssistantMessage>),
}

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

/// A text part within a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPart {
    /// Unique part identifier.
    pub id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// The text content.
    pub text: String,
    /// Whether this part was synthetically generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthetic: Option<bool>,
    /// Timing information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TextPartTime>,
}

/// Timing information for a [`TextPart`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPartTime {
    /// Epoch timestamp when text streaming started.
    pub start: f64,
    /// Epoch timestamp when text streaming ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
}

/// A file attachment part within a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilePart {
    /// Unique part identifier.
    pub id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// MIME type of the file.
    pub mime: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// URL to the file content.
    pub url: String,
    /// Optional human-readable filename.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Optional source information for the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<FilePartSource>,
}

/// A tool invocation part within a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolPart {
    /// Unique part identifier.
    pub id: String,
    /// Tool call identifier.
    #[serde(rename = "callID")]
    pub call_id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Current state of the tool invocation.
    pub state: ToolState,
    /// Name of the tool.
    pub tool: String,
}

/// Marks the beginning of a reasoning step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepStartPart {
    /// Unique part identifier.
    pub id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

/// Marks the end of a reasoning step with cost and token info.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepFinishPart {
    /// Unique part identifier.
    pub id: String,
    /// Monetary cost of this step.
    pub cost: f64,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Token usage for this step.
    pub tokens: StepFinishTokens,
}

/// Token usage for a [`StepFinishPart`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepFinishTokens {
    /// Cache token details.
    pub cache: TokenCache,
    /// Number of input tokens.
    pub input: u64,
    /// Number of output tokens.
    pub output: u64,
    /// Number of reasoning tokens.
    pub reasoning: u64,
}

/// A snapshot of the session state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotPart {
    /// Unique part identifier.
    pub id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Snapshot content.
    pub snapshot: String,
}

/// A patch describing file modifications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchPart {
    /// Unique part identifier.
    pub id: String,
    /// List of affected file paths.
    pub files: Vec<String>,
    /// Hash of the patch content.
    pub hash: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

/// A subtask delegation part within a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubtaskPart {
    /// Unique part identifier.
    pub id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The prompt for the subtask.
    pub prompt: String,
    /// Human-readable description of the subtask.
    pub description: String,
    /// The agent handling this subtask.
    pub agent: String,
    /// Optional model information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<SubtaskPartModel>,
    /// Optional command to run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// Model information for a [`SubtaskPart`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubtaskPartModel {
    /// Provider identifier.
    #[serde(rename = "providerID")]
    pub provider_id: String,
    /// Model identifier.
    #[serde(rename = "modelID")]
    pub model_id: String,
}

/// A reasoning/thinking step part within a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningPart {
    /// Unique part identifier.
    pub id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The reasoning text content.
    pub text: String,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Timing information.
    pub time: ReasoningPartTime,
}

/// Timing information for a [`ReasoningPart`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningPartTime {
    /// Epoch timestamp when reasoning started.
    pub start: f64,
    /// Epoch timestamp when reasoning ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
}

/// An agent delegation part within a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPart {
    /// Unique part identifier.
    pub id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The name of the agent.
    pub name: String,
    /// Optional source information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<AgentPartSource>,
}

/// Source information for an [`AgentPart`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPartSource {
    /// The source value.
    pub value: String,
    /// Start offset.
    pub start: i64,
    /// End offset.
    pub end: i64,
}

/// A compaction marker part within a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactionPart {
    /// Unique part identifier.
    pub id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// Whether this compaction was automatic.
    pub auto: bool,
}

/// A retry part within a message, indicating a failed attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryPart {
    /// Unique part identifier.
    pub id: String,
    /// The session this part belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// The message this part belongs to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The attempt number.
    pub attempt: f64,
    /// The error that occurred.
    pub error: serde_json::Value,
    /// Timing information.
    pub time: RetryPartTime,
}

/// Timing information for a [`RetryPart`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryPartTime {
    /// Epoch timestamp when the retry was created.
    pub created: f64,
}

/// A part within a message — discriminated by `type`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Part {
    /// A text content part.
    #[serde(rename = "text")]
    Text(TextPart),
    /// A file attachment part.
    #[serde(rename = "file")]
    File(FilePart),
    /// A tool invocation part.
    #[serde(rename = "tool")]
    Tool(ToolPart),
    /// Start of a reasoning step.
    #[serde(rename = "step-start")]
    StepStart(StepStartPart),
    /// End of a reasoning step.
    #[serde(rename = "step-finish")]
    StepFinish(StepFinishPart),
    /// A session state snapshot.
    #[serde(rename = "snapshot")]
    Snapshot(SnapshotPart),
    /// A file patch.
    #[serde(rename = "patch")]
    Patch(PatchPart),
    /// A subtask delegation.
    #[serde(rename = "subtask")]
    Subtask(SubtaskPart),
    /// A reasoning/thinking step.
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningPart),
    /// An agent delegation.
    #[serde(rename = "agent")]
    Agent(AgentPart),
    /// A compaction marker.
    #[serde(rename = "compaction")]
    Compaction(CompactionPart),
    /// A retry attempt.
    #[serde(rename = "retry")]
    Retry(RetryPart),
    /// Any unknown part variant returned by newer server versions.
    #[serde(other)]
    Unknown,
}

// ---------------------------------------------------------------------------
// Tool States
// ---------------------------------------------------------------------------

/// A pending tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolStatePending {}

/// A currently-running tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolStateRunning {
    /// Timing information.
    pub time: ToolStateRunningTime,
    /// Optional input data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Optional provider-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Optional human-readable title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Timing for [`ToolStateRunning`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolStateRunningTime {
    /// Epoch timestamp when the tool started running.
    pub start: f64,
}

/// A successfully completed tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolStateCompleted {
    /// Input data passed to the tool.
    pub input: HashMap<String, serde_json::Value>,
    /// Provider-specific metadata.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Tool output text.
    pub output: String,
    /// Timing information.
    pub time: ToolStateCompletedTime,
    /// Human-readable title.
    pub title: String,
}

/// Timing for [`ToolStateCompleted`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolStateCompletedTime {
    /// Epoch timestamp when the tool finished.
    pub end: f64,
    /// Epoch timestamp when the tool started.
    pub start: f64,
}

/// A tool invocation that resulted in an error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolStateError {
    /// Error description.
    pub error: String,
    /// Input data passed to the tool.
    pub input: HashMap<String, serde_json::Value>,
    /// Timing information.
    pub time: ToolStateErrorTime,
}

/// Timing for [`ToolStateError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolStateErrorTime {
    /// Epoch timestamp when the tool finished with an error.
    pub end: f64,
    /// Epoch timestamp when the tool started.
    pub start: f64,
}

/// The current state of a tool invocation — discriminated by `status`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum ToolState {
    /// The tool is waiting to execute.
    #[serde(rename = "pending")]
    Pending(ToolStatePending),
    /// The tool is currently executing.
    #[serde(rename = "running")]
    Running(ToolStateRunning),
    /// The tool completed successfully.
    #[serde(rename = "completed")]
    Completed(ToolStateCompleted),
    /// The tool finished with an error.
    #[serde(rename = "error")]
    Error(ToolStateError),
}

// ---------------------------------------------------------------------------
// File Part Source Types
// ---------------------------------------------------------------------------

/// Text content extracted from a source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilePartSourceText {
    /// End offset (byte or character index).
    pub end: u64,
    /// Start offset (byte or character index).
    pub start: u64,
    /// The extracted text value.
    pub value: String,
}

/// A file-based source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileSource {
    /// Filesystem path.
    pub path: String,
    /// Extracted text content.
    pub text: FilePartSourceText,
}

/// A symbol-based source (e.g. function, class).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolSource {
    /// Symbol kind (numeric identifier from the language server).
    pub kind: u64,
    /// Symbol name.
    pub name: String,
    /// Filesystem path containing the symbol.
    pub path: String,
    /// Character range of the symbol.
    pub range: SymbolSourceRange,
    /// Extracted text content.
    pub text: FilePartSourceText,
}

/// Range of a [`SymbolSource`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolSourceRange {
    /// End position.
    pub end: SymbolSourcePosition,
    /// Start position.
    pub start: SymbolSourcePosition,
}

/// A line/character position within a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolSourcePosition {
    /// Zero-based character offset on the line.
    pub character: u64,
    /// Zero-based line number.
    pub line: u64,
}

/// Source of a file part — either a file or a symbol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum FilePartSource {
    /// A plain file source.
    #[serde(rename = "file")]
    File(FileSource),
    /// A symbol source (function, class, etc.).
    #[serde(rename = "symbol")]
    Symbol(SymbolSource),
}

// ---------------------------------------------------------------------------
// Input Types
// ---------------------------------------------------------------------------

/// A text input part for the chat endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPartInput {
    /// The text content.
    pub text: String,
    /// Optional part identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Whether this input was synthetically generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthetic: Option<bool>,
    /// Whether this input should be ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored: Option<bool>,
    /// Optional timing information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TextPartInputTime>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Timing information for a [`TextPartInput`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPartInputTime {
    /// Epoch timestamp when text input started.
    pub start: f64,
    /// Epoch timestamp when text input ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
}

/// A file input part for the chat endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilePartInput {
    /// MIME type of the file.
    pub mime: String,
    /// URL to the file content.
    pub url: String,
    /// Optional part identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional human-readable filename.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Optional source information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<FilePartSource>,
}

/// An agent input part for the chat endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPartInput {
    /// Agent name.
    pub name: String,
    /// Optional part identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional source information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<AgentPartSource>,
}

/// A subtask input part for the chat endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubtaskPartInput {
    /// The prompt text.
    pub prompt: String,
    /// Description of the subtask.
    pub description: String,
    /// Agent to handle the subtask.
    pub agent: String,
    /// Optional part identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional model selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<SessionChatModel>,
    /// Optional command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// An input part — text, file, agent, or subtask — discriminated by `type`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PartInput {
    /// A text input.
    #[serde(rename = "text")]
    Text(TextPartInput),
    /// A file input.
    #[serde(rename = "file")]
    File(FilePartInput),
    /// An agent delegation input.
    #[serde(rename = "agent")]
    Agent(AgentPartInput),
    /// A subtask delegation input.
    #[serde(rename = "subtask")]
    Subtask(SubtaskPartInput),
}

// ---------------------------------------------------------------------------
// Response Types
// ---------------------------------------------------------------------------

/// A single item in the session messages response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionMessagesResponseItem {
    /// The message metadata.
    pub info: Message,
    /// The parts that compose this message.
    pub parts: Vec<Part>,
}

/// Response type for listing session messages.
pub type SessionMessagesResponse = Vec<SessionMessagesResponseItem>;

/// Response type for listing sessions.
pub type SessionListResponse = Vec<Session>;

/// Response type for deleting a session.
pub type SessionDeleteResponse = bool;

/// Response type for aborting a session.
pub type SessionAbortResponse = bool;

/// Response type for initialising a session.
pub type SessionInitResponse = bool;

/// Response type for summarising a session.
pub type SessionSummarizeResponse = bool;

// ---------------------------------------------------------------------------
// Param Types
// ---------------------------------------------------------------------------

/// Model selection for the chat endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionChatModel {
    /// Provider identifier.
    #[serde(rename = "providerID")]
    pub provider_id: String,
    /// Model identifier.
    #[serde(rename = "modelID")]
    pub model_id: String,
}

/// Parameters for the chat endpoint (`POST /session/{id}/message`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionChatParams {
    /// Input parts (text, file, agent, subtask).
    pub parts: Vec<PartInput>,
    /// Optional model selection (nested providerID + modelID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<SessionChatModel>,
    /// Optional message identifier for continuing a conversation.
    #[serde(rename = "messageID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Optional agent override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Whether to suppress the reply.
    #[serde(rename = "noReply")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_reply: Option<bool>,
    /// Optional output format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,
    /// Optional system prompt override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Optional variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Optional map of tool names to their enabled state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, bool>>,
}

/// Parameters for session initialisation (`POST /session/{id}/init`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionInitParams {
    /// The message identifier.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// The model to use.
    #[serde(rename = "modelID")]
    pub model_id: String,
    /// The provider to use.
    #[serde(rename = "providerID")]
    pub provider_id: String,
}

/// Parameters for reverting a session (`POST /session/{id}/revert`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionRevertParams {
    /// The message to revert to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// Optional part identifier to revert to.
    #[serde(rename = "partID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_id: Option<String>,
}

/// Parameters for summarising a session (`POST /session/{id}/summarize`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSummarizeParams {
    /// The model to use for summarisation.
    #[serde(rename = "modelID")]
    pub model_id: String,
    /// The provider to use for summarisation.
    #[serde(rename = "providerID")]
    pub provider_id: String,
}

// ---------------------------------------------------------------------------
// SessionResource
// ---------------------------------------------------------------------------

/// Provides access to the Session-related API endpoints.
pub struct SessionResource<'a> {
    client: &'a Opencode,
}

impl<'a> SessionResource<'a> {
    /// Create a new `SessionResource` bound to the given client.
    pub(crate) const fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    /// Create a new session (`POST /session`).
    pub async fn create(&self, options: Option<&RequestOptions>) -> Result<Session, OpencodeError> {
        self.client.post::<Session, ()>("/session", None, options).await
    }

    /// List all sessions (`GET /session`).
    pub async fn list(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SessionListResponse, OpencodeError> {
        self.client.get("/session", options).await
    }

    /// Delete a session (`DELETE /session/{id}`).
    pub async fn delete(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SessionDeleteResponse, OpencodeError> {
        self.client.delete::<bool, ()>(&format!("/session/{id}"), None, options).await
    }

    /// Abort a running session (`POST /session/{id}/abort`).
    pub async fn abort(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SessionAbortResponse, OpencodeError> {
        self.client.post::<bool, ()>(&format!("/session/{id}/abort"), None, options).await
    }

    /// Send a chat message (`POST /session/{id}/message`).
    pub async fn chat(
        &self,
        id: &str,
        params: &SessionChatParams,
        options: Option<&RequestOptions>,
    ) -> Result<SessionMessagesResponseItem, OpencodeError> {
        self.client.post(&format!("/session/{id}/message"), Some(params), options).await
    }

    /// Initialise a session (`POST /session/{id}/init`).
    pub async fn init(
        &self,
        id: &str,
        params: &SessionInitParams,
        options: Option<&RequestOptions>,
    ) -> Result<SessionInitResponse, OpencodeError> {
        self.client.post(&format!("/session/{id}/init"), Some(params), options).await
    }

    /// List messages in a session (`GET /session/{id}/message`).
    pub async fn messages(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SessionMessagesResponse, OpencodeError> {
        self.client.get(&format!("/session/{id}/message"), options).await
    }

    /// Revert a session to a previous state (`POST /session/{id}/revert`).
    pub async fn revert(
        &self,
        id: &str,
        params: &SessionRevertParams,
        options: Option<&RequestOptions>,
    ) -> Result<Session, OpencodeError> {
        self.client.post(&format!("/session/{id}/revert"), Some(params), options).await
    }

    /// Share a session (`POST /session/{id}/share`).
    pub async fn share(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<Session, OpencodeError> {
        self.client.post::<Session, ()>(&format!("/session/{id}/share"), None, options).await
    }

    /// Summarise a session (`POST /session/{id}/summarize`).
    pub async fn summarize(
        &self,
        id: &str,
        params: &SessionSummarizeParams,
        options: Option<&RequestOptions>,
    ) -> Result<SessionSummarizeResponse, OpencodeError> {
        self.client.post(&format!("/session/{id}/summarize"), Some(params), options).await
    }

    /// Unrevert a session (`POST /session/{id}/unrevert`).
    pub async fn unrevert(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<Session, OpencodeError> {
        self.client.post::<Session, ()>(&format!("/session/{id}/unrevert"), None, options).await
    }

    /// Unshare a session (`DELETE /session/{id}/share`).
    pub async fn unshare(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<Session, OpencodeError> {
        self.client.delete::<Session, ()>(&format!("/session/{id}/share"), None, options).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // -- Session round-trips --

    #[test]
    fn session_full_round_trip() {
        let session = Session {
            id: "sess_001".into(),
            slug: "my-session".into(),
            project_id: "proj_001".into(),
            directory: "/home/user/project".into(),
            time: SessionTime {
                created: 1_700_000_000.0,
                updated: 1_700_001_000.0,
                compacting: None,
                archived: None,
            },
            title: "My Session".into(),
            version: "1".into(),
            parent_id: Some("sess_000".into()),
            revert: Some(SessionRevert {
                message_id: "msg_001".into(),
                diff: Some("--- a/file\n+++ b/file".into()),
                part_id: Some("part_001".into()),
                snapshot: Some("snapshot_data".into()),
            }),
            share: Some(SessionShare { url: "https://example.com/share/abc".into() }),
            summary: None,
            permission: None,
        };
        let json_str = serde_json::to_string(&session).unwrap();
        assert!(json_str.contains("parentID"));
        assert!(json_str.contains("messageID"));
        assert!(json_str.contains("partID"));
        let back: Session = serde_json::from_str(&json_str).unwrap();
        assert_eq!(session, back);
    }

    #[test]
    fn session_minimal_round_trip() {
        let session = Session {
            id: "sess_002".into(),
            slug: "empty".into(),
            project_id: "proj_002".into(),
            directory: "/tmp".into(),
            time: SessionTime {
                created: 1_700_000_000.0,
                updated: 1_700_000_000.0,
                compacting: None,
                archived: None,
            },
            title: "Empty".into(),
            version: "1".into(),
            parent_id: None,
            revert: None,
            share: None,
            summary: None,
            permission: None,
        };
        let json_str = serde_json::to_string(&session).unwrap();
        assert!(!json_str.contains("parentID"));
        assert!(!json_str.contains("revert"));
        assert!(!json_str.contains("share"));
        let back: Session = serde_json::from_str(&json_str).unwrap();
        assert_eq!(session, back);
    }

    // -- Message round-trips --

    #[test]
    fn user_message_round_trip() {
        let msg = UserMessage {
            id: "msg_u001".into(),
            session_id: "sess_001".into(),
            time: UserMessageTime { created: 1_700_000_100.0 },
            agent: "coder".into(),
            model: UserMessageModel { provider_id: "openai".into(), model_id: "gpt-4o".into() },
            format: None,
            summary: None,
            system: None,
            tools: None,
            variant: None,
        };
        let json_str = serde_json::to_string(&msg).unwrap();
        assert!(json_str.contains("sessionID"));
        assert!(json_str.contains("agent"));
        assert!(json_str.contains("providerID"));
        assert!(json_str.contains("modelID"));
        let back: UserMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn assistant_message_round_trip() {
        let msg = AssistantMessage {
            id: "msg_a001".into(),
            cost: 0.0032,
            mode: "code".into(),
            model_id: "gpt-4o".into(),
            path: AssistantMessagePath {
                cwd: "/home/user/project".into(),
                root: "/home/user/project".into(),
            },
            provider_id: "openai".into(),
            session_id: "sess_001".into(),
            parent_id: "msg_parent".into(),
            agent: "default".into(),
            system: vec!["You are a helpful assistant.".into()],
            time: AssistantMessageTime {
                created: 1_700_000_200.0,
                completed: Some(1_700_000_210.0),
            },
            tokens: AssistantMessageTokens {
                cache: TokenCache { read: 100, write: 50 },
                input: 500,
                output: 200,
                reasoning: 0,
                total: 700,
            },
            error: None,
            summary: None,
            variant: None,
            finish: None,
            structured: None,
        };
        let json_str = serde_json::to_string(&msg).unwrap();
        assert!(json_str.contains("modelID"));
        assert!(json_str.contains("providerID"));
        assert!(json_str.contains("sessionID"));
        assert!(json_str.contains("parentID"));
        assert!(json_str.contains("agent"));
        let back: AssistantMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn assistant_message_with_error() {
        let msg = AssistantMessage {
            id: "msg_a002".into(),
            cost: 0.0,
            mode: "code".into(),
            model_id: "gpt-4o".into(),
            path: AssistantMessagePath { cwd: "/tmp".into(), root: "/tmp".into() },
            provider_id: "openai".into(),
            session_id: "sess_001".into(),
            parent_id: "msg_parent".into(),
            agent: "coder".into(),
            system: vec![],
            time: AssistantMessageTime { created: 1_700_000_300.0, completed: None },
            tokens: AssistantMessageTokens {
                cache: TokenCache { read: 0, write: 0 },
                input: 0,
                output: 0,
                reasoning: 0,
                total: 0,
            },
            error: Some(SessionError::ProviderAuthError {
                data: super::super::shared::ProviderAuthErrorData {
                    message: "invalid key".into(),
                    provider_id: "openai".into(),
                },
            }),
            summary: Some(true),
            variant: None,
            finish: None,
            structured: None,
        };
        let json_str = serde_json::to_string(&msg).unwrap();
        assert!(json_str.contains("ProviderAuthError"));
        let back: AssistantMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg, back);
    }

    // -- Message enum --

    #[test]
    fn message_enum_user_variant() {
        let msg = Message::User(Box::new(UserMessage {
            id: "msg_u002".into(),
            session_id: "sess_001".into(),
            time: UserMessageTime { created: 1_700_000_100.0 },
            agent: "coder".into(),
            model: UserMessageModel { provider_id: "openai".into(), model_id: "gpt-4o".into() },
            format: None,
            summary: None,
            system: None,
            tools: None,
            variant: None,
        }));
        let json_str = serde_json::to_string(&msg).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["role"], "user");
        let back: Message = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn message_enum_assistant_variant() {
        let msg = Message::Assistant(Box::new(AssistantMessage {
            id: "msg_a003".into(),
            cost: 0.001,
            mode: "default".into(),
            model_id: "claude-3-opus".into(),
            path: AssistantMessagePath { cwd: "/home".into(), root: "/home".into() },
            provider_id: "anthropic".into(),
            session_id: "sess_002".into(),
            parent_id: "msg_a002".into(),
            agent: "reviewer".into(),
            system: vec![],
            time: AssistantMessageTime {
                created: 1_700_000_500.0,
                completed: Some(1_700_000_510.0),
            },
            tokens: AssistantMessageTokens {
                cache: TokenCache { read: 10, write: 5 },
                input: 100,
                output: 50,
                reasoning: 20,
                total: 170,
            },
            error: None,
            summary: None,
            variant: None,
            finish: None,
            structured: None,
        }));
        let json_str = serde_json::to_string(&msg).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["role"], "assistant");
        let back: Message = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg, back);
    }

    // -- Part enum variants --

    #[test]
    fn part_text_round_trip() {
        let part = Part::Text(TextPart {
            id: "p_001".into(),
            message_id: "msg_a001".into(),
            session_id: "sess_001".into(),
            text: "Hello, world!".into(),
            synthetic: None,
            time: Some(TextPartTime { start: 1_700_000_200.0, end: Some(1_700_000_201.0) }),
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "text");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn part_tool_round_trip() {
        let part = Part::Tool(ToolPart {
            id: "p_002".into(),
            call_id: "call_001".into(),
            message_id: "msg_a001".into(),
            session_id: "sess_001".into(),
            state: ToolState::Completed(ToolStateCompleted {
                input: HashMap::from([("cmd".into(), json!("ls"))]),
                metadata: HashMap::new(),
                output: "file1.rs\nfile2.rs".into(),
                time: ToolStateCompletedTime { end: 1_700_000_205.0, start: 1_700_000_202.0 },
                title: "bash".into(),
            }),
            tool: "bash".into(),
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "tool");
        assert_eq!(v["state"]["status"], "completed");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn part_step_start_round_trip() {
        let part = Part::StepStart(StepStartPart {
            id: "p_003".into(),
            message_id: "msg_a001".into(),
            session_id: "sess_001".into(),
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "step-start");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn part_step_finish_round_trip() {
        let part = Part::StepFinish(StepFinishPart {
            id: "p_004".into(),
            cost: 0.001,
            message_id: "msg_a001".into(),
            session_id: "sess_001".into(),
            tokens: StepFinishTokens {
                cache: TokenCache { read: 10, write: 5 },
                input: 100,
                output: 50,
                reasoning: 0,
            },
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "step-finish");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn part_patch_round_trip() {
        let part = Part::Patch(PatchPart {
            id: "p_005".into(),
            files: vec!["src/main.rs".into(), "Cargo.toml".into()],
            hash: "abc123".into(),
            message_id: "msg_a001".into(),
            session_id: "sess_001".into(),
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "patch");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn part_snapshot_round_trip() {
        let part = Part::Snapshot(SnapshotPart {
            id: "p_006".into(),
            message_id: "msg_a001".into(),
            session_id: "sess_001".into(),
            snapshot: "{\"state\":\"data\"}".into(),
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "snapshot");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn part_file_round_trip() {
        let part = Part::File(FilePart {
            id: "p_007".into(),
            message_id: "msg_a001".into(),
            mime: "image/png".into(),
            session_id: "sess_001".into(),
            url: "https://example.com/img.png".into(),
            filename: Some("screenshot.png".into()),
            source: None,
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "file");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    // -- ToolState enum --

    #[test]
    fn tool_state_pending() {
        let state = ToolState::Pending(ToolStatePending {});
        let json_str = serde_json::to_string(&state).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["status"], "pending");
        let back: ToolState = serde_json::from_str(&json_str).unwrap();
        assert_eq!(state, back);
    }

    #[test]
    fn tool_state_running() {
        let state = ToolState::Running(ToolStateRunning {
            time: ToolStateRunningTime { start: 1_700_000_200.0 },
            input: Some(json!({"command": "echo hello"})),
            metadata: Some(HashMap::from([("key".into(), json!("value"))])),
            title: Some("Running bash".into()),
        });
        let json_str = serde_json::to_string(&state).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["status"], "running");
        let back: ToolState = serde_json::from_str(&json_str).unwrap();
        assert_eq!(state, back);
    }

    #[test]
    fn tool_state_completed() {
        let state = ToolState::Completed(ToolStateCompleted {
            input: HashMap::from([("cmd".into(), json!("ls -la"))]),
            metadata: HashMap::from([("exit_code".into(), json!(0))]),
            output: "total 42\ndrwxr-xr-x ...".into(),
            time: ToolStateCompletedTime { end: 1_700_000_210.0, start: 1_700_000_200.0 },
            title: "bash".into(),
        });
        let json_str = serde_json::to_string(&state).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["status"], "completed");
        let back: ToolState = serde_json::from_str(&json_str).unwrap();
        assert_eq!(state, back);
    }

    #[test]
    fn tool_state_error() {
        let state = ToolState::Error(ToolStateError {
            error: "command not found".into(),
            input: HashMap::from([("cmd".into(), json!("nonexistent"))]),
            time: ToolStateErrorTime { end: 1_700_000_201.0, start: 1_700_000_200.0 },
        });
        let json_str = serde_json::to_string(&state).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["status"], "error");
        let back: ToolState = serde_json::from_str(&json_str).unwrap();
        assert_eq!(state, back);
    }

    // -- FilePartSource enum --

    #[test]
    fn file_part_source_file_variant() {
        let src = FilePartSource::File(FileSource {
            path: "/home/user/main.rs".into(),
            text: FilePartSourceText { end: 100, start: 0, value: "fn main() {}".into() },
        });
        let json_str = serde_json::to_string(&src).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "file");
        let back: FilePartSource = serde_json::from_str(&json_str).unwrap();
        assert_eq!(src, back);
    }

    #[test]
    fn file_part_source_symbol_variant() {
        let src = FilePartSource::Symbol(SymbolSource {
            kind: 12,
            name: "main".into(),
            path: "/home/user/main.rs".into(),
            range: SymbolSourceRange {
                end: SymbolSourcePosition { character: 1, line: 2 },
                start: SymbolSourcePosition { character: 0, line: 0 },
            },
            text: FilePartSourceText {
                end: 50,
                start: 0,
                value: "fn main() {\n    println!(\"hello\");\n}".into(),
            },
        });
        let json_str = serde_json::to_string(&src).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "symbol");
        let back: FilePartSource = serde_json::from_str(&json_str).unwrap();
        assert_eq!(src, back);
    }

    // -- SessionChatParams --

    #[test]
    fn session_chat_params_full_round_trip() {
        let params = SessionChatParams {
            parts: vec![
                PartInput::Text(TextPartInput {
                    text: "Hello!".into(),
                    id: Some("input_001".into()),
                    synthetic: None,
                    ignored: None,
                    time: Some(TextPartInputTime { start: 1_700_000_000.0, end: None }),
                    metadata: None,
                }),
                PartInput::File(FilePartInput {
                    mime: "text/plain".into(),
                    url: "file:///tmp/test.txt".into(),
                    id: None,
                    filename: Some("test.txt".into()),
                    source: None,
                }),
            ],
            model: Some(SessionChatModel {
                provider_id: "openai".into(),
                model_id: "gpt-4o".into(),
            }),
            message_id: Some("msg_001".into()),
            agent: None,
            no_reply: None,
            format: None,
            system: Some("Be concise.".into()),
            variant: None,
            tools: Some(HashMap::from([("bash".into(), true)])),
        };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(json_str.contains("modelID"));
        assert!(json_str.contains("providerID"));
        assert!(json_str.contains("messageID"));
        let back: SessionChatParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    #[test]
    fn session_chat_params_minimal() {
        let params = SessionChatParams {
            parts: vec![PartInput::Text(TextPartInput {
                text: "Hi".into(),
                id: None,
                synthetic: None,
                ignored: None,
                time: None,
                metadata: None,
            })],
            model: None,
            message_id: None,
            agent: None,
            no_reply: None,
            format: None,
            system: None,
            variant: None,
            tools: None,
        };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(!json_str.contains("messageID"));
        assert!(!json_str.contains("model"));
        assert!(!json_str.contains("system"));
        assert!(!json_str.contains("tools"));
        let back: SessionChatParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    // -- PartInput enum --

    #[test]
    fn part_input_text_round_trip() {
        let input = PartInput::Text(TextPartInput {
            text: "Hello".into(),
            id: None,
            synthetic: Some(true),
            ignored: None,
            time: None,
            metadata: None,
        });
        let json_str = serde_json::to_string(&input).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "text");
        let back: PartInput = serde_json::from_str(&json_str).unwrap();
        assert_eq!(input, back);
    }

    #[test]
    fn part_input_file_round_trip() {
        let input = PartInput::File(FilePartInput {
            mime: "image/png".into(),
            url: "https://example.com/img.png".into(),
            id: Some("fi_001".into()),
            filename: Some("photo.png".into()),
            source: Some(FilePartSource::File(FileSource {
                path: "/tmp/photo.png".into(),
                text: FilePartSourceText { end: 0, start: 0, value: String::new() },
            })),
        });
        let json_str = serde_json::to_string(&input).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "file");
        let back: PartInput = serde_json::from_str(&json_str).unwrap();
        assert_eq!(input, back);
    }

    // -- SessionMessagesResponseItem --

    #[test]
    fn session_messages_response_item_round_trip() {
        let item = SessionMessagesResponseItem {
            info: Message::User(Box::new(UserMessage {
                id: "msg_u010".into(),
                session_id: "sess_001".into(),
                time: UserMessageTime { created: 1_700_000_000.0 },
                agent: "coder".into(),
                model: UserMessageModel { provider_id: "openai".into(), model_id: "gpt-4o".into() },
                format: None,
                summary: None,
                system: None,
                tools: None,
                variant: None,
            })),
            parts: vec![Part::Text(TextPart {
                id: "p_010".into(),
                message_id: "msg_u010".into(),
                session_id: "sess_001".into(),
                text: "What is Rust?".into(),
                synthetic: None,
                time: None,
            })],
        };
        let json_str = serde_json::to_string(&item).unwrap();
        let back: SessionMessagesResponseItem = serde_json::from_str(&json_str).unwrap();
        assert_eq!(item, back);
    }

    // -- Param types --

    #[test]
    fn session_init_params_round_trip() {
        let params = SessionInitParams {
            message_id: "msg_001".into(),
            model_id: "gpt-4o".into(),
            provider_id: "openai".into(),
        };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(json_str.contains("messageID"));
        assert!(json_str.contains("modelID"));
        assert!(json_str.contains("providerID"));
        let back: SessionInitParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    #[test]
    fn session_revert_params_round_trip() {
        let params =
            SessionRevertParams { message_id: "msg_001".into(), part_id: Some("part_001".into()) };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(json_str.contains("messageID"));
        assert!(json_str.contains("partID"));
        let back: SessionRevertParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    #[test]
    fn session_summarize_params_round_trip() {
        let params =
            SessionSummarizeParams { model_id: "gpt-4o".into(), provider_id: "openai".into() };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(json_str.contains("modelID"));
        assert!(json_str.contains("providerID"));
        let back: SessionSummarizeParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    // -- Deserialization from JS-compatible JSON --

    #[test]
    fn deserialize_message_from_js_json() {
        let js_json = json!({
            "role": "user",
            "id": "msg_from_js",
            "sessionID": "sess_js",
            "time": { "created": 1700000000.0 }
        });
        let msg: Message = serde_json::from_value(js_json).unwrap();
        match msg {
            Message::User(u) => {
                assert_eq!(u.id, "msg_from_js");
                assert_eq!(u.session_id, "sess_js");
            }
            _ => panic!("expected User variant"),
        }
    }

    #[test]
    fn deserialize_part_from_js_json() {
        let js_json = json!({
            "type": "step-start",
            "id": "p_js_001",
            "messageID": "msg_js_001",
            "sessionID": "sess_js"
        });
        let part: Part = serde_json::from_value(js_json).unwrap();
        match part {
            Part::StepStart(s) => {
                assert_eq!(s.id, "p_js_001");
                assert_eq!(s.message_id, "msg_js_001");
            }
            _ => panic!("expected StepStart variant"),
        }
    }

    #[test]
    fn deserialize_tool_state_from_js_json() {
        let js_json = json!({
            "status": "error",
            "error": "timeout",
            "input": { "cmd": "sleep 999" },
            "time": { "start": 1700000000.0, "end": 1700000030.0 }
        });
        let state: ToolState = serde_json::from_value(js_json).unwrap();
        match state {
            ToolState::Error(e) => {
                assert_eq!(e.error, "timeout");
            }
            _ => panic!("expected Error variant"),
        }
    }

    // -- Edge cases --

    #[test]
    fn tool_state_running_minimal() {
        let state = ToolState::Running(ToolStateRunning {
            time: ToolStateRunningTime { start: 1_700_000_000.0 },
            input: None,
            metadata: None,
            title: None,
        });
        let json_str = serde_json::to_string(&state).unwrap();
        assert!(!json_str.contains("input"));
        assert!(!json_str.contains("metadata"));
        assert!(!json_str.contains("title"));
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["status"], "running");
        let back: ToolState = serde_json::from_str(&json_str).unwrap();
        assert_eq!(state, back);
    }

    #[test]
    fn text_part_no_synthetic_no_time() {
        let part = TextPart {
            id: "tp_001".into(),
            message_id: "msg_001".into(),
            session_id: "sess_001".into(),
            text: "bare text".into(),
            synthetic: None,
            time: None,
        };
        let json_str = serde_json::to_string(&part).unwrap();
        assert!(!json_str.contains("synthetic"));
        assert!(!json_str.contains("time"));
        let back: TextPart = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn file_part_no_filename_no_source() {
        let part = FilePart {
            id: "fp_001".into(),
            message_id: "msg_001".into(),
            mime: "application/octet-stream".into(),
            session_id: "sess_001".into(),
            url: "https://example.com/data.bin".into(),
            filename: None,
            source: None,
        };
        let json_str = serde_json::to_string(&part).unwrap();
        assert!(!json_str.contains("filename"));
        assert!(!json_str.contains("source"));
        let back: FilePart = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn part_file_minimal_round_trip() {
        let part = Part::File(FilePart {
            id: "fp_002".into(),
            message_id: "msg_001".into(),
            mime: "text/plain".into(),
            session_id: "sess_001".into(),
            url: "file:///tmp/a.txt".into(),
            filename: None,
            source: None,
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "file");
        assert!(v.get("filename").is_none());
        assert!(v.get("source").is_none());
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn assistant_message_no_error_no_summary() {
        let msg = AssistantMessage {
            id: "msg_edge".into(),
            cost: 0.0,
            mode: "plan".into(),
            model_id: "o1".into(),
            path: AssistantMessagePath { cwd: "/app".into(), root: "/app".into() },
            provider_id: "openai".into(),
            session_id: "sess_edge".into(),
            parent_id: "msg_prev".into(),
            agent: "planner".into(),
            system: vec![],
            time: AssistantMessageTime { created: 1_700_000_000.0, completed: None },
            tokens: AssistantMessageTokens {
                cache: TokenCache { read: 0, write: 0 },
                input: 10,
                output: 5,
                reasoning: 0,
                total: 15,
            },
            error: None,
            summary: None,
            variant: None,
            finish: None,
            structured: None,
        };
        let json_str = serde_json::to_string(&msg).unwrap();
        assert!(!json_str.contains("error"));
        assert!(!json_str.contains("summary"));
        assert!(!json_str.contains("variant"));
        assert!(!json_str.contains("finish"));
        assert!(!json_str.contains("structured"));
        assert!(!json_str.contains("system"));
        let back: AssistantMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn part_input_text_minimal() {
        let input = PartInput::Text(TextPartInput {
            text: "hi".into(),
            id: None,
            synthetic: None,
            ignored: None,
            time: None,
            metadata: None,
        });
        let json_str = serde_json::to_string(&input).unwrap();
        assert!(!json_str.contains("\"id\""));
        assert!(!json_str.contains("synthetic"));
        assert!(!json_str.contains("ignored"));
        assert!(!json_str.contains("time"));
        assert!(!json_str.contains("metadata"));
        let back: PartInput = serde_json::from_str(&json_str).unwrap();
        assert_eq!(input, back);
    }

    #[test]
    fn part_input_file_minimal() {
        let input = PartInput::File(FilePartInput {
            mime: "text/csv".into(),
            url: "file:///data.csv".into(),
            id: None,
            filename: None,
            source: None,
        });
        let json_str = serde_json::to_string(&input).unwrap();
        assert!(!json_str.contains("\"id\""));
        assert!(!json_str.contains("filename"));
        assert!(!json_str.contains("source"));
        let back: PartInput = serde_json::from_str(&json_str).unwrap();
        assert_eq!(input, back);
    }

    #[test]
    fn session_revert_minimal() {
        let revert = SessionRevert {
            message_id: "msg_r001".into(),
            diff: None,
            part_id: None,
            snapshot: None,
        };
        let json_str = serde_json::to_string(&revert).unwrap();
        assert!(!json_str.contains("diff"));
        assert!(!json_str.contains("partID"));
        assert!(!json_str.contains("snapshot"));
        let back: SessionRevert = serde_json::from_str(&json_str).unwrap();
        assert_eq!(revert, back);
    }

    #[test]
    fn text_part_time_no_end() {
        let t = TextPartTime { start: 1_700_000_000.0, end: None };
        let json_str = serde_json::to_string(&t).unwrap();
        assert!(!json_str.contains("end"));
        let back: TextPartTime = serde_json::from_str(&json_str).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn assistant_message_time_no_completed() {
        let t = AssistantMessageTime { created: 1_700_000_000.0, completed: None };
        let json_str = serde_json::to_string(&t).unwrap();
        assert!(!json_str.contains("completed"));
        let back: AssistantMessageTime = serde_json::from_str(&json_str).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn session_revert_params_no_part_id() {
        let params = SessionRevertParams { message_id: "msg_001".into(), part_id: None };
        let json_str = serde_json::to_string(&params).unwrap();
        assert!(!json_str.contains("partID"));
        let back: SessionRevertParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, back);
    }

    #[test]
    fn file_part_with_symbol_source() {
        let part = Part::File(FilePart {
            id: "fp_sym".into(),
            message_id: "msg_001".into(),
            mime: "text/x-rust".into(),
            session_id: "sess_001".into(),
            url: "file:///src/lib.rs".into(),
            filename: Some("lib.rs".into()),
            source: Some(FilePartSource::Symbol(SymbolSource {
                kind: 6,
                name: "MyStruct".into(),
                path: "/src/lib.rs".into(),
                range: SymbolSourceRange {
                    end: SymbolSourcePosition { character: 1, line: 10 },
                    start: SymbolSourcePosition { character: 0, line: 5 },
                },
                text: FilePartSourceText {
                    end: 200,
                    start: 100,
                    value: "struct MyStruct {}".into(),
                },
            })),
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["source"]["type"], "symbol");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    // -- New type round-trips --

    #[test]
    fn session_summary_round_trip() {
        let summary = SessionSummary {
            additions: 10.0,
            deletions: 3.0,
            files: 2.0,
            diffs: Some(vec![FileDiff {
                file: "src/main.rs".into(),
                before: "fn old() {}".into(),
                after: "fn new() {}".into(),
                additions: 5.0,
                deletions: 2.0,
                status: Some(FileDiffStatus::Modified),
            }]),
        };
        let json_str = serde_json::to_string(&summary).unwrap();
        assert!(json_str.contains("\"status\":\"modified\""));
        let back: SessionSummary = serde_json::from_str(&json_str).unwrap();
        assert_eq!(summary, back);
    }

    #[test]
    fn session_summary_minimal() {
        let summary = SessionSummary { additions: 0.0, deletions: 0.0, files: 0.0, diffs: None };
        let json_str = serde_json::to_string(&summary).unwrap();
        assert!(!json_str.contains("diffs"));
        let back: SessionSummary = serde_json::from_str(&json_str).unwrap();
        assert_eq!(summary, back);
    }

    #[test]
    fn file_diff_round_trip() {
        let diff = FileDiff {
            file: "README.md".into(),
            before: "# Old".into(),
            after: "# New".into(),
            additions: 1.0,
            deletions: 1.0,
            status: Some(FileDiffStatus::Modified),
        };
        let json_str = serde_json::to_string(&diff).unwrap();
        let back: FileDiff = serde_json::from_str(&json_str).unwrap();
        assert_eq!(diff, back);
    }

    #[test]
    fn file_diff_no_status() {
        let diff = FileDiff {
            file: "new.rs".into(),
            before: String::new(),
            after: "fn main() {}".into(),
            additions: 1.0,
            deletions: 0.0,
            status: None,
        };
        let json_str = serde_json::to_string(&diff).unwrap();
        assert!(!json_str.contains("status"));
        let back: FileDiff = serde_json::from_str(&json_str).unwrap();
        assert_eq!(diff, back);
    }

    #[test]
    fn file_diff_status_variants() {
        for (variant, expected) in [
            (FileDiffStatus::Added, "\"added\""),
            (FileDiffStatus::Deleted, "\"deleted\""),
            (FileDiffStatus::Modified, "\"modified\""),
        ] {
            let json_str = serde_json::to_string(&variant).unwrap();
            assert_eq!(json_str, expected);
            let back: FileDiffStatus = serde_json::from_str(&json_str).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn permission_rule_round_trip() {
        let rule = PermissionRule {
            permission: "file:write".into(),
            pattern: "src/**".into(),
            action: "allow".into(),
        };
        let json_str = serde_json::to_string(&rule).unwrap();
        let back: PermissionRule = serde_json::from_str(&json_str).unwrap();
        assert_eq!(rule, back);
    }

    #[test]
    fn permission_ruleset_round_trip() {
        let ruleset: PermissionRuleset = vec![
            PermissionRule {
                permission: "file:write".into(),
                pattern: "src/**".into(),
                action: "allow".into(),
            },
            PermissionRule {
                permission: "file:read".into(),
                pattern: "**".into(),
                action: "deny".into(),
            },
        ];
        let json_str = serde_json::to_string(&ruleset).unwrap();
        let back: PermissionRuleset = serde_json::from_str(&json_str).unwrap();
        assert_eq!(ruleset, back);
    }

    #[test]
    fn session_time_with_compacting_archived() {
        let time = SessionTime {
            created: 1_700_000_000.0,
            updated: 1_700_001_000.0,
            compacting: Some(1_700_002_000.0),
            archived: Some(1_700_003_000.0),
        };
        let json_str = serde_json::to_string(&time).unwrap();
        assert!(json_str.contains("compacting"));
        assert!(json_str.contains("archived"));
        let back: SessionTime = serde_json::from_str(&json_str).unwrap();
        assert_eq!(time, back);
    }

    #[test]
    fn session_time_without_compacting_archived() {
        let time = SessionTime {
            created: 1_700_000_000.0,
            updated: 1_700_001_000.0,
            compacting: None,
            archived: None,
        };
        let json_str = serde_json::to_string(&time).unwrap();
        assert!(!json_str.contains("compacting"));
        assert!(!json_str.contains("archived"));
        let back: SessionTime = serde_json::from_str(&json_str).unwrap();
        assert_eq!(time, back);
    }

    #[test]
    fn session_from_spec_compliant_json() {
        let json = json!({
            "id": "ses_abc123",
            "slug": "my-session",
            "projectID": "proj_xyz",
            "directory": "/home/user/project",
            "title": "Full Session",
            "version": "2",
            "time": {
                "created": 1_700_000_000.0,
                "updated": 1_700_001_000.0,
                "compacting": 1_700_002_000.0,
                "archived": 1_700_003_000.0
            },
            "parentID": "ses_parent",
            "summary": {
                "additions": 10.0,
                "deletions": 3.0,
                "files": 2.0,
                "diffs": [
                    {
                        "file": "src/main.rs",
                        "before": "old code",
                        "after": "new code",
                        "additions": 5.0,
                        "deletions": 2.0,
                        "status": "added"
                    }
                ]
            },
            "share": { "url": "https://example.com/share/abc" },
            "permission": [
                { "permission": "file:write", "pattern": "src/**", "action": "allow" }
            ],
            "revert": {
                "messageID": "msg_001",
                "diff": "some diff",
                "partID": "part_001",
                "snapshot": "snap"
            }
        });
        let session: Session = serde_json::from_value(json).unwrap();
        assert_eq!(session.id, "ses_abc123");
        assert_eq!(session.slug, "my-session");
        assert_eq!(session.project_id, "proj_xyz");
        assert_eq!(session.directory, "/home/user/project");
        assert_eq!(session.time.compacting, Some(1_700_002_000.0));
        assert_eq!(session.time.archived, Some(1_700_003_000.0));
        assert!(session.summary.is_some());
        let summary = session.summary.unwrap();
        assert_eq!(summary.additions, 10.0);
        assert_eq!(summary.diffs.as_ref().unwrap().len(), 1);
        assert_eq!(summary.diffs.as_ref().unwrap()[0].status, Some(FileDiffStatus::Added));
        assert!(session.permission.is_some());
        assert_eq!(session.permission.unwrap().len(), 1);
        assert_eq!(session.parent_id.as_deref(), Some("ses_parent"));
    }

    #[test]
    fn session_deserialize_without_new_fields() {
        // Ensures backwards compatibility: old JSON without slug/projectID/directory still works.
        let json = json!({
            "id": "ses_old",
            "title": "Old Session",
            "version": "1",
            "time": { "created": 100.0, "updated": 200.0 }
        });
        let session: Session = serde_json::from_value(json).unwrap();
        assert_eq!(session.id, "ses_old");
        assert_eq!(session.slug, "");
        assert_eq!(session.project_id, "");
        assert_eq!(session.directory, "");
        assert!(session.summary.is_none());
        assert!(session.permission.is_none());
        assert!(session.time.compacting.is_none());
        assert!(session.time.archived.is_none());
    }

    #[test]
    fn assistant_message_from_spec_compliant_json() {
        let json = json!({
            "id": "msg_spec",
            "sessionID": "sess_spec",
            "role": "assistant",
            "parentID": "msg_parent_spec",
            "modelID": "gpt-4o",
            "providerID": "openai",
            "mode": "code",
            "agent": "coder",
            "path": { "cwd": "/project", "root": "/project" },
            "cost": 0.005,
            "time": { "created": 1_700_000_000.0, "completed": 1_700_000_010.0 },
            "tokens": {
                "total": 1500,
                "input": 1000,
                "output": 400,
                "reasoning": 100,
                "cache": { "read": 500, "write": 200 }
            }
        });
        let msg: AssistantMessage = serde_json::from_value(json).unwrap();
        assert_eq!(msg.id, "msg_spec");
        assert_eq!(msg.parent_id, "msg_parent_spec");
        assert_eq!(msg.agent, "coder");
        assert_eq!(msg.tokens.total, 1500);
        assert_eq!(msg.tokens.input, 1000);
        assert_eq!(msg.tokens.output, 400);
        assert_eq!(msg.tokens.reasoning, 100);
        assert_eq!(msg.tokens.cache.read, 500);
        assert_eq!(msg.tokens.cache.write, 200);
        assert!(msg.variant.is_none());
        assert!(msg.finish.is_none());
        assert!(msg.structured.is_none());
    }

    #[test]
    fn assistant_message_with_optional_fields_populated() {
        let json = json!({
            "id": "msg_opt",
            "sessionID": "sess_opt",
            "parentID": "msg_p",
            "modelID": "claude-3-opus",
            "providerID": "anthropic",
            "mode": "code",
            "agent": "reviewer",
            "path": { "cwd": "/home", "root": "/home" },
            "cost": 0.01,
            "time": { "created": 1_700_000_000.0 },
            "tokens": {
                "total": 500,
                "input": 300,
                "output": 150,
                "reasoning": 50,
                "cache": { "read": 100, "write": 50 }
            },
            "variant": "v2",
            "finish": "stop",
            "structured": { "key": "value" }
        });
        let msg: AssistantMessage = serde_json::from_value(json).unwrap();
        assert_eq!(msg.variant.as_deref(), Some("v2"));
        assert_eq!(msg.finish.as_deref(), Some("stop"));
        assert_eq!(msg.structured.as_ref().unwrap()["key"], "value");
        assert_eq!(msg.parent_id, "msg_p");
        assert_eq!(msg.agent, "reviewer");
        assert_eq!(msg.tokens.total, 500);
    }

    // -- UserMessage new fields --

    #[test]
    fn user_message_from_spec_json() {
        let json = json!({
            "id": "msg_u_spec",
            "sessionID": "sess_spec",
            "role": "user",
            "time": { "created": 1_700_000_000.0 },
            "agent": "coder",
            "model": { "providerID": "openai", "modelID": "gpt-4o" },
            "format": { "type": "text" },
            "summary": {
                "title": "Summary Title",
                "body": "Summary body text",
                "diffs": [
                    {
                        "file": "src/main.rs",
                        "before": "old",
                        "after": "new",
                        "additions": 1.0,
                        "deletions": 1.0,
                        "status": "modified"
                    }
                ]
            },
            "system": "Be concise.",
            "tools": { "bash": true, "read_file": false },
            "variant": "v2"
        });
        let msg: UserMessage = serde_json::from_value(json).unwrap();
        assert_eq!(msg.id, "msg_u_spec");
        assert_eq!(msg.session_id, "sess_spec");
        assert_eq!(msg.agent, "coder");
        assert_eq!(msg.model.provider_id, "openai");
        assert_eq!(msg.model.model_id, "gpt-4o");
        assert!(matches!(msg.format, Some(OutputFormat::Text)));
        let summary = msg.summary.unwrap();
        assert_eq!(summary.title.as_deref(), Some("Summary Title"));
        assert_eq!(summary.body.as_deref(), Some("Summary body text"));
        assert_eq!(summary.diffs.len(), 1);
        assert_eq!(msg.system.as_deref(), Some("Be concise."));
        let tools = msg.tools.unwrap();
        assert_eq!(tools.get("bash"), Some(&true));
        assert_eq!(tools.get("read_file"), Some(&false));
        assert_eq!(msg.variant.as_deref(), Some("v2"));
    }

    #[test]
    fn user_message_with_output_format() {
        // Text variant
        let msg_text = UserMessage {
            id: "msg_fmt_text".into(),
            session_id: "sess_001".into(),
            time: UserMessageTime { created: 1_700_000_000.0 },
            agent: "coder".into(),
            model: UserMessageModel { provider_id: "openai".into(), model_id: "gpt-4o".into() },
            format: Some(OutputFormat::Text),
            summary: None,
            system: None,
            tools: None,
            variant: None,
        };
        let json_str = serde_json::to_string(&msg_text).unwrap();
        assert!(json_str.contains("\"type\":\"text\""));
        let back: UserMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg_text, back);

        // JsonSchema variant
        let msg_schema = UserMessage {
            id: "msg_fmt_schema".into(),
            session_id: "sess_001".into(),
            time: UserMessageTime { created: 1_700_000_000.0 },
            agent: "coder".into(),
            model: UserMessageModel {
                provider_id: "anthropic".into(),
                model_id: "claude-3-opus".into(),
            },
            format: Some(OutputFormat::JsonSchema {
                schema: json!({ "type": "object", "properties": { "answer": { "type": "string" } } }),
                retry_count: Some(3),
            }),
            summary: None,
            system: None,
            tools: None,
            variant: None,
        };
        let json_str = serde_json::to_string(&msg_schema).unwrap();
        assert!(json_str.contains("json_schema"));
        assert!(json_str.contains("retryCount"));
        let back: UserMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg_schema, back);
    }

    #[test]
    fn user_message_with_summary() {
        let msg = UserMessage {
            id: "msg_sum".into(),
            session_id: "sess_001".into(),
            time: UserMessageTime { created: 1_700_000_000.0 },
            agent: "reviewer".into(),
            model: UserMessageModel { provider_id: "openai".into(), model_id: "gpt-4o".into() },
            format: None,
            summary: Some(UserMessageSummary {
                title: Some("Refactored main".into()),
                body: Some("Cleaned up imports".into()),
                diffs: vec![FileDiff {
                    file: "src/main.rs".into(),
                    before: "use old;".into(),
                    after: "use new;".into(),
                    additions: 1.0,
                    deletions: 1.0,
                    status: Some(FileDiffStatus::Modified),
                }],
            }),
            system: None,
            tools: None,
            variant: None,
        };
        let json_str = serde_json::to_string(&msg).unwrap();
        assert!(json_str.contains("Refactored main"));
        assert!(json_str.contains("Cleaned up imports"));
        let back: UserMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(msg, back);
    }

    // -- New Part variant round-trips --

    #[test]
    fn part_subtask_round_trip() {
        let part = Part::Subtask(SubtaskPart {
            id: "p_sub_001".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            prompt: "Fix the bug".into(),
            description: "Fix the null pointer bug in parser".into(),
            agent: "coder".into(),
            model: Some(SubtaskPartModel {
                provider_id: "openai".into(),
                model_id: "gpt-4o".into(),
            }),
            command: Some("cargo test".into()),
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "subtask");
        assert_eq!(v["sessionID"], "sess_001");
        assert_eq!(v["messageID"], "msg_a001");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);

        // Minimal (no model, no command)
        let minimal = Part::Subtask(SubtaskPart {
            id: "p_sub_002".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            prompt: "Do it".into(),
            description: "desc".into(),
            agent: "coder".into(),
            model: None,
            command: None,
        });
        let json_str = serde_json::to_string(&minimal).unwrap();
        assert!(!json_str.contains("model"));
        assert!(!json_str.contains("command"));
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(minimal, back);
    }

    #[test]
    fn part_reasoning_round_trip() {
        let part = Part::Reasoning(ReasoningPart {
            id: "p_reason_001".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            text: "Let me think about this...".into(),
            metadata: Some(HashMap::from([("key".into(), json!("value"))])),
            time: ReasoningPartTime { start: 1_700_000_200.0, end: Some(1_700_000_201.0) },
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "reasoning");
        assert_eq!(v["sessionID"], "sess_001");
        assert_eq!(v["messageID"], "msg_a001");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);

        // Minimal (no metadata, no end time)
        let minimal = Part::Reasoning(ReasoningPart {
            id: "p_reason_002".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            text: "thinking".into(),
            metadata: None,
            time: ReasoningPartTime { start: 1_700_000_200.0, end: None },
        });
        let json_str = serde_json::to_string(&minimal).unwrap();
        assert!(!json_str.contains("metadata"));
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(minimal, back);
    }

    #[test]
    fn part_agent_round_trip() {
        let part = Part::Agent(AgentPart {
            id: "p_agent_001".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            name: "coder".into(),
            source: Some(AgentPartSource {
                value: "some source content".into(),
                start: 0,
                end: 42,
            }),
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "agent");
        assert_eq!(v["sessionID"], "sess_001");
        assert_eq!(v["messageID"], "msg_a001");
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);

        // Minimal (no source)
        let minimal = Part::Agent(AgentPart {
            id: "p_agent_002".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            name: "reviewer".into(),
            source: None,
        });
        let json_str = serde_json::to_string(&minimal).unwrap();
        assert!(!json_str.contains("source"));
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(minimal, back);
    }

    #[test]
    fn part_compaction_round_trip() {
        let part = Part::Compaction(CompactionPart {
            id: "p_compact_001".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            auto: true,
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "compaction");
        assert_eq!(v["sessionID"], "sess_001");
        assert_eq!(v["messageID"], "msg_a001");
        assert_eq!(v["auto"], true);
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);

        // auto = false
        let part_false = Part::Compaction(CompactionPart {
            id: "p_compact_002".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            auto: false,
        });
        let json_str = serde_json::to_string(&part_false).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["auto"], false);
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part_false, back);
    }

    #[test]
    fn part_retry_round_trip() {
        let part = Part::Retry(RetryPart {
            id: "p_retry_001".into(),
            session_id: "sess_001".into(),
            message_id: "msg_a001".into(),
            attempt: 2.0,
            error: json!({ "message": "rate limited", "code": 429 }),
            time: RetryPartTime { created: 1_700_000_200.0 },
        });
        let json_str = serde_json::to_string(&part).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["type"], "retry");
        assert_eq!(v["sessionID"], "sess_001");
        assert_eq!(v["messageID"], "msg_a001");
        assert_eq!(v["attempt"], 2.0);
        let back: Part = serde_json::from_str(&json_str).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn output_format_round_trip() {
        // Text
        let text = OutputFormat::Text;
        let json_str = serde_json::to_string(&text).unwrap();
        let back: OutputFormat = serde_json::from_str(&json_str).unwrap();
        assert_eq!(text, back);

        // JsonSchema without retry_count
        let schema_no_retry =
            OutputFormat::JsonSchema { schema: json!({ "type": "string" }), retry_count: None };
        let json_str = serde_json::to_string(&schema_no_retry).unwrap();
        assert!(!json_str.contains("retryCount"));
        let back: OutputFormat = serde_json::from_str(&json_str).unwrap();
        assert_eq!(schema_no_retry, back);

        // JsonSchema with retry_count
        let schema_retry =
            OutputFormat::JsonSchema { schema: json!({ "type": "object" }), retry_count: Some(2) };
        let json_str = serde_json::to_string(&schema_retry).unwrap();
        assert!(json_str.contains("retryCount"));
        let back: OutputFormat = serde_json::from_str(&json_str).unwrap();
        assert_eq!(schema_retry, back);
    }
}
