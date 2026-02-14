//! Event resource types and the `EventResource` struct.
//!
//! Events are delivered via Server-Sent Events (SSE).  The [`EventResource`]
//! will expose a streaming `list()` method once SSE support is wired up
//! (Task 3.1).  For now only the data types are defined.

use serde::{Deserialize, Serialize};

use super::{
    session::{FileDiff, Message, Part, Session},
    shared::SessionError,
};
use crate::client::Opencode;

// ---------------------------------------------------------------------------
// EventListResponse — internally-tagged discriminated union
// ---------------------------------------------------------------------------

/// A single event from the `/event` SSE stream.
///
/// Internally tagged on `"type"` to match the JS SDK representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EventListResponse {
    // ----- installation -----
    /// An installation was updated to a new version.
    #[serde(rename = "installation.updated")]
    InstallationUpdated {
        /// Payload.
        properties: InstallationUpdatedProps,
    },

    /// A newer version is available for installation.
    #[serde(rename = "installation.update-available")]
    InstallationUpdateAvailable {
        /// Payload.
        properties: InstallationUpdateAvailableProps,
    },

    // ----- project -----
    /// A project was updated.
    #[serde(rename = "project.updated")]
    ProjectUpdated {
        /// Payload.
        properties: ProjectUpdatedProps,
    },

    // ----- server -----
    /// A server instance was disposed.
    #[serde(rename = "server.instance.disposed")]
    ServerInstanceDisposed {
        /// Payload.
        properties: ServerInstanceDisposedProps,
    },

    /// A server connected.
    #[serde(rename = "server.connected")]
    ServerConnected {
        /// Payload.
        properties: EmptyProps,
    },

    // ----- global -----
    /// Global disposed.
    #[serde(rename = "global.disposed")]
    GlobalDisposed {
        /// Payload.
        properties: EmptyProps,
    },

    // ----- lsp -----
    /// LSP client diagnostics were received.
    #[serde(rename = "lsp.client.diagnostics")]
    LspClientDiagnostics {
        /// Payload.
        properties: LspClientDiagnosticsProps,
    },

    /// LSP state was updated.
    #[serde(rename = "lsp.updated")]
    LspUpdated {
        /// Payload.
        properties: EmptyProps,
    },

    // ----- file -----
    /// A file was edited.
    #[serde(rename = "file.edited")]
    FileEdited {
        /// Payload.
        properties: FileEditedProps,
    },

    /// A file-watcher event was received.
    #[serde(rename = "file.watcher.updated")]
    FileWatcherUpdated {
        /// Payload.
        properties: FileWatcherUpdatedProps,
    },

    // ----- message -----
    /// A message was updated.
    #[serde(rename = "message.updated")]
    MessageUpdated {
        /// Payload.
        properties: MessageUpdatedProps,
    },

    /// A message was removed.
    #[serde(rename = "message.removed")]
    MessageRemoved {
        /// Payload.
        properties: MessageRemovedProps,
    },

    /// A message part was updated.
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated {
        /// Payload.
        properties: MessagePartUpdatedProps,
    },

    /// A streaming delta for a message part.
    #[serde(rename = "message.part.delta")]
    MessagePartDelta {
        /// Payload.
        properties: MessagePartDeltaProps,
    },

    /// A message part was removed.
    #[serde(rename = "message.part.removed")]
    MessagePartRemoved {
        /// Payload.
        properties: MessagePartRemovedProps,
    },

    // ----- permission -----
    /// A permission was asked.
    #[serde(rename = "permission.asked")]
    PermissionAsked {
        /// Payload (complex `PermissionRequest` type, represented as JSON value).
        properties: serde_json::Value,
    },

    /// A permission was replied to.
    #[serde(rename = "permission.replied")]
    PermissionReplied {
        /// Payload.
        properties: PermissionRepliedProps,
    },

    // ----- session -----
    /// A session was created.
    #[serde(rename = "session.created")]
    SessionCreated {
        /// Payload.
        properties: SessionCreatedProps,
    },

    /// A session was updated.
    #[serde(rename = "session.updated")]
    SessionUpdated {
        /// Payload.
        properties: SessionUpdatedProps,
    },

    /// A session was deleted.
    #[serde(rename = "session.deleted")]
    SessionDeleted {
        /// Payload.
        properties: SessionDeletedProps,
    },

    /// Session status changed.
    #[serde(rename = "session.status")]
    SessionStatus {
        /// Payload.
        properties: SessionStatusProps,
    },

    /// A session became idle.
    #[serde(rename = "session.idle")]
    SessionIdle {
        /// Payload.
        properties: SessionIdleProps,
    },

    /// A session diff was produced.
    #[serde(rename = "session.diff")]
    SessionDiff {
        /// Payload.
        properties: SessionDiffProps,
    },

    /// A session was compacted.
    #[serde(rename = "session.compacted")]
    SessionCompacted {
        /// Payload.
        properties: SessionCompactedProps,
    },

    /// A session encountered an error.
    #[serde(rename = "session.error")]
    SessionError {
        /// Payload.
        properties: SessionErrorProps,
    },

    // ----- question -----
    /// A question was asked.
    #[serde(rename = "question.asked")]
    QuestionAsked {
        /// Payload (complex `QuestionRequest` type, represented as JSON value).
        properties: serde_json::Value,
    },

    /// A question was replied to.
    #[serde(rename = "question.replied")]
    QuestionReplied {
        /// Payload.
        properties: QuestionRepliedProps,
    },

    /// A question was rejected.
    #[serde(rename = "question.rejected")]
    QuestionRejected {
        /// Payload.
        properties: QuestionRejectedProps,
    },

    // ----- todo -----
    /// Todos were updated.
    #[serde(rename = "todo.updated")]
    TodoUpdated {
        /// Payload.
        properties: TodoUpdatedProps,
    },

    // ----- tui -----
    /// Append text to the TUI prompt.
    #[serde(rename = "tui.prompt.append")]
    TuiPromptAppend {
        /// Payload.
        properties: TuiPromptAppendProps,
    },

    /// Execute a TUI command.
    #[serde(rename = "tui.command.execute")]
    TuiCommandExecute {
        /// Payload.
        properties: TuiCommandExecuteProps,
    },

    /// Show a TUI toast notification.
    #[serde(rename = "tui.toast.show")]
    TuiToastShow {
        /// Payload.
        properties: TuiToastShowProps,
    },

    /// Select a TUI session.
    #[serde(rename = "tui.session.select")]
    TuiSessionSelect {
        /// Payload.
        properties: TuiSessionSelectProps,
    },

    // ----- mcp -----
    /// MCP tools changed.
    #[serde(rename = "mcp.tools.changed")]
    McpToolsChanged {
        /// Payload.
        properties: McpToolsChangedProps,
    },

    /// MCP browser open failed.
    #[serde(rename = "mcp.browser.open.failed")]
    McpBrowserOpenFailed {
        /// Payload.
        properties: McpBrowserOpenFailedProps,
    },

    // ----- command -----
    /// A command was executed.
    #[serde(rename = "command.executed")]
    CommandExecuted {
        /// Payload.
        properties: CommandExecutedProps,
    },

    // ----- vcs -----
    /// VCS branch was updated.
    #[serde(rename = "vcs.branch.updated")]
    VcsBranchUpdated {
        /// Payload.
        properties: VcsBranchUpdatedProps,
    },

    // ----- pty -----
    /// A PTY was created.
    #[serde(rename = "pty.created")]
    PtyCreated {
        /// Payload.
        properties: PtyCreatedProps,
    },

    /// A PTY was updated.
    #[serde(rename = "pty.updated")]
    PtyUpdated {
        /// Payload.
        properties: PtyUpdatedProps,
    },

    /// A PTY exited.
    #[serde(rename = "pty.exited")]
    PtyExited {
        /// Payload.
        properties: PtyExitedProps,
    },

    /// A PTY was deleted.
    #[serde(rename = "pty.deleted")]
    PtyDeleted {
        /// Payload.
        properties: PtyDeletedProps,
    },

    // ----- worktree -----
    /// A worktree is ready.
    #[serde(rename = "worktree.ready")]
    WorktreeReady {
        /// Payload.
        properties: WorktreeReadyProps,
    },

    /// A worktree operation failed.
    #[serde(rename = "worktree.failed")]
    WorktreeFailed {
        /// Payload.
        properties: WorktreeFailedProps,
    },
}

// ---------------------------------------------------------------------------
// Property structs
// ---------------------------------------------------------------------------

/// Empty properties payload for events with no extra data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmptyProps {}

/// Properties for [`EventListResponse::InstallationUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallationUpdatedProps {
    /// New version string.
    pub version: String,
}

/// Properties for [`EventListResponse::InstallationUpdateAvailable`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallationUpdateAvailableProps {
    /// Available version string.
    pub version: String,
}

/// Properties for [`EventListResponse::ProjectUpdated`].
///
/// The `Project` type is complex; represented as a JSON value.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectUpdatedProps {
    /// The updated project (complex type, serialised as `serde_json::Value`).
    pub properties: serde_json::Value,
}

/// Properties for [`EventListResponse::ServerInstanceDisposed`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerInstanceDisposedProps {
    /// Directory of the disposed server instance.
    pub directory: String,
}

/// Properties for [`EventListResponse::LspClientDiagnostics`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspClientDiagnosticsProps {
    /// File path.
    pub path: String,
    /// Language-server identifier.
    #[serde(rename = "serverID")]
    pub server_id: String,
}

/// Properties for [`EventListResponse::MessageUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageUpdatedProps {
    /// The updated message.
    pub info: Message,
}

/// Properties for [`EventListResponse::MessageRemoved`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageRemovedProps {
    /// ID of the removed message.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// Session the message belonged to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

/// Properties for [`EventListResponse::MessagePartUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessagePartUpdatedProps {
    /// The updated part.
    pub part: Part,
}

/// Properties for [`EventListResponse::MessagePartDelta`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessagePartDeltaProps {
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Message ID.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// Part ID.
    #[serde(rename = "partID")]
    pub part_id: String,
    /// The field being updated.
    pub field: String,
    /// The delta text.
    pub delta: String,
}

/// Properties for [`EventListResponse::MessagePartRemoved`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessagePartRemovedProps {
    /// Session the part belonged to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Message the part belonged to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// ID of the removed part.
    #[serde(rename = "partID")]
    pub part_id: String,
}

/// Reply action for a permission request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionReply {
    /// Allow once.
    #[serde(rename = "once")]
    Once,
    /// Always allow.
    #[serde(rename = "always")]
    Always,
    /// Reject the permission.
    #[serde(rename = "reject")]
    Reject,
}

/// Properties for [`EventListResponse::PermissionReplied`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionRepliedProps {
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Request ID.
    #[serde(rename = "requestID")]
    pub request_id: String,
    /// The reply action.
    pub reply: PermissionReply,
}

/// Properties for [`EventListResponse::SessionCreated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionCreatedProps {
    /// The created session.
    pub info: Session,
}

/// Properties for [`EventListResponse::SessionUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionUpdatedProps {
    /// The updated session.
    pub info: Session,
}

/// Properties for [`EventListResponse::SessionDeleted`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionDeletedProps {
    /// The deleted session.
    pub info: Session,
}

/// Properties for [`EventListResponse::SessionStatus`].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionStatusProps {
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Session status (complex tagged union, represented as `serde_json::Value`).
    pub status: serde_json::Value,
}

/// Properties for [`EventListResponse::SessionIdle`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionIdleProps {
    /// The idle session's ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

/// Properties for [`EventListResponse::SessionDiff`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionDiffProps {
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// The file diffs.
    pub diff: Vec<FileDiff>,
}

/// Properties for [`EventListResponse::SessionCompacted`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionCompactedProps {
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

/// Properties for [`EventListResponse::SessionError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionErrorProps {
    /// The error, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SessionError>,
    /// The session ID, if available.
    #[serde(rename = "sessionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Properties for [`EventListResponse::QuestionReplied`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuestionRepliedProps {
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Request ID.
    #[serde(rename = "requestID")]
    pub request_id: String,
    /// Answers.
    pub answers: Vec<Vec<String>>,
}

/// Properties for [`EventListResponse::QuestionRejected`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuestionRejectedProps {
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Request ID.
    #[serde(rename = "requestID")]
    pub request_id: String,
}

/// A single to-do item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    /// To-do content.
    pub content: String,
    /// Status of the to-do.
    pub status: String,
    /// Priority of the to-do.
    pub priority: String,
}

/// Properties for [`EventListResponse::TodoUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoUpdatedProps {
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// The updated to-do list.
    pub todos: Vec<Todo>,
}

/// Properties for [`EventListResponse::FileEdited`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEditedProps {
    /// The edited file path.
    pub file: String,
}

/// Kind of file-watcher event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileWatcherEvent {
    /// A file was added.
    #[serde(rename = "add")]
    Add,
    /// A file was changed.
    #[serde(rename = "change")]
    Change,
    /// A file was unlinked (removed).
    #[serde(rename = "unlink")]
    Unlink,
}

/// Properties for [`EventListResponse::FileWatcherUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileWatcherUpdatedProps {
    /// The kind of file-system event.
    pub event: FileWatcherEvent,
    /// The affected file path.
    pub file: String,
}

/// Properties for [`EventListResponse::TuiPromptAppend`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TuiPromptAppendProps {
    /// Text to append.
    pub text: String,
}

/// Properties for [`EventListResponse::TuiCommandExecute`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TuiCommandExecuteProps {
    /// Command to execute.
    pub command: String,
}

/// Variant for a TUI toast notification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToastVariant {
    /// Informational toast.
    #[serde(rename = "info")]
    Info,
    /// Success toast.
    #[serde(rename = "success")]
    Success,
    /// Warning toast.
    #[serde(rename = "warning")]
    Warning,
    /// Error toast.
    #[serde(rename = "error")]
    Error,
}

/// Properties for [`EventListResponse::TuiToastShow`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuiToastShowProps {
    /// Optional title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Toast message.
    pub message: String,
    /// Toast variant.
    pub variant: ToastVariant,
    /// Optional duration in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
}

/// Properties for [`EventListResponse::TuiSessionSelect`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TuiSessionSelectProps {
    /// Session ID to select.
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

/// Properties for [`EventListResponse::McpToolsChanged`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpToolsChangedProps {
    /// MCP server name.
    pub server: String,
}

/// Properties for [`EventListResponse::McpBrowserOpenFailed`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpBrowserOpenFailedProps {
    /// MCP name.
    #[serde(rename = "mcpName")]
    pub mcp_name: String,
    /// URL that failed to open.
    pub url: String,
}

/// Properties for [`EventListResponse::CommandExecuted`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandExecutedProps {
    /// Command name.
    pub name: String,
    /// Session ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Command arguments.
    pub arguments: String,
    /// Message ID.
    #[serde(rename = "messageID")]
    pub message_id: String,
}

/// Properties for [`EventListResponse::VcsBranchUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VcsBranchUpdatedProps {
    /// Branch name, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// Status of a PTY process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PtyStatus {
    /// The PTY is running.
    #[serde(rename = "running")]
    Running,
    /// The PTY has exited.
    #[serde(rename = "exited")]
    Exited,
}

/// A pseudo-terminal (PTY) descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pty {
    /// PTY identifier.
    pub id: String,
    /// PTY title.
    pub title: String,
    /// Command being run.
    pub command: String,
    /// Command arguments.
    pub args: Vec<String>,
    /// Working directory.
    pub cwd: String,
    /// PTY status.
    pub status: PtyStatus,
    /// Process ID.
    pub pid: f64,
}

/// Properties for [`EventListResponse::PtyCreated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PtyCreatedProps {
    /// PTY info.
    pub info: Pty,
}

/// Properties for [`EventListResponse::PtyUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PtyUpdatedProps {
    /// PTY info.
    pub info: Pty,
}

/// Properties for [`EventListResponse::PtyExited`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PtyExitedProps {
    /// PTY identifier.
    pub id: String,
    /// Exit code.
    #[serde(rename = "exitCode")]
    pub exit_code: f64,
}

/// Properties for [`EventListResponse::PtyDeleted`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PtyDeletedProps {
    /// PTY identifier.
    pub id: String,
}

/// Properties for [`EventListResponse::WorktreeReady`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorktreeReadyProps {
    /// Worktree name.
    pub name: String,
    /// Branch name.
    pub branch: String,
}

/// Properties for [`EventListResponse::WorktreeFailed`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorktreeFailedProps {
    /// Error message.
    pub message: String,
}

// ---------------------------------------------------------------------------
// EventResource
// ---------------------------------------------------------------------------

/// Resource accessor for the `/event` SSE endpoint.
pub struct EventResource<'a> {
    client: &'a Opencode,
}

impl<'a> EventResource<'a> {
    /// Create a new `EventResource` bound to the given client.
    pub(crate) const fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    /// List events as an SSE stream.
    ///
    /// The `/event` endpoint returns a Server-Sent Events stream where
    /// each event's `data` field is a JSON-encoded [`EventListResponse`].
    pub async fn list(
        &self,
    ) -> Result<crate::streaming::SseStream<EventListResponse>, crate::error::OpencodeError> {
        self.client.get_stream("/event").await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::session::{UserMessage, UserMessageModel, UserMessageTime};

    // -- InstallationUpdated round-trip --

    #[test]
    fn installation_updated_round_trip() {
        let event = EventListResponse::InstallationUpdated {
            properties: InstallationUpdatedProps { version: "1.2.3".into() },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"installation.updated"#));
        assert!(json_str.contains(r#""version":"1.2.3"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    // -- MessageUpdated round-trip (with full Message) --

    #[test]
    fn message_updated_round_trip() {
        let msg = Message::User(Box::new(UserMessage {
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
        }));

        let event = EventListResponse::MessageUpdated {
            properties: MessageUpdatedProps { info: msg.clone() },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"message.updated"#));
        assert!(json_str.contains(r#""role":"user"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    // -- SessionError round-trip --

    #[test]
    fn session_error_round_trip() {
        use crate::resources::shared::{SessionError as SE, UnknownErrorData};

        let event = EventListResponse::SessionError {
            properties: SessionErrorProps {
                error: Some(SE::UnknownError {
                    data: UnknownErrorData { message: "something broke".into() },
                }),
                session_id: Some("sess_err_001".into()),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"session.error"#));
        assert!(json_str.contains(r#""name":"UnknownError"#));
        assert!(json_str.contains("something broke"));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn session_error_empty_round_trip() {
        let event = EventListResponse::SessionError {
            properties: SessionErrorProps { error: None, session_id: None },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        // Optional fields should be omitted (check for the key, not substring)
        assert!(!json_str.contains(r#""error""#));
        assert!(!json_str.contains(r#""sessionID""#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    // -- FileWatcherUpdated round-trip --

    #[test]
    fn file_watcher_updated_round_trip() {
        let event = EventListResponse::FileWatcherUpdated {
            properties: FileWatcherUpdatedProps {
                event: FileWatcherEvent::Add,
                file: "src/main.rs".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"file.watcher.updated"#));
        assert!(json_str.contains(r#""event":"add"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);

        // Also test the Change variant
        let event2 = EventListResponse::FileWatcherUpdated {
            properties: FileWatcherUpdatedProps {
                event: FileWatcherEvent::Change,
                file: "Cargo.toml".into(),
            },
        };
        let json_str2 = serde_json::to_string(&event2).unwrap();
        assert!(json_str2.contains(r#""event":"change"#));
        let back2: EventListResponse = serde_json::from_str(&json_str2).unwrap();
        assert_eq!(event2, back2);

        // Also test the Unlink variant
        let event3 = EventListResponse::FileWatcherUpdated {
            properties: FileWatcherUpdatedProps {
                event: FileWatcherEvent::Unlink,
                file: "old_file.rs".into(),
            },
        };
        let json_str3 = serde_json::to_string(&event3).unwrap();
        assert!(json_str3.contains(r#""event":"unlink"#));
        let back3: EventListResponse = serde_json::from_str(&json_str3).unwrap();
        assert_eq!(event3, back3);
    }

    // -- Deserialization from raw JSON --

    #[test]
    fn deserialize_permission_asked() {
        let raw = r#"{
            "type": "permission.asked",
            "properties": {
                "id": "perm_001",
                "sessionID": "sess_001",
                "title": "Run bash command"
            }
        }"#;
        let event: EventListResponse = serde_json::from_str(raw).unwrap();
        match &event {
            EventListResponse::PermissionAsked { properties } => {
                assert_eq!(properties["id"], "perm_001");
                assert_eq!(properties["sessionID"], "sess_001");
            }
            other => panic!("expected PermissionAsked, got {other:?}"),
        }
        // Round-trip
        let json_str = serde_json::to_string(&event).unwrap();
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn deserialize_permission_replied() {
        let raw = r#"{
            "type": "permission.replied",
            "properties": {
                "sessionID": "sess_001",
                "requestID": "req_001",
                "reply": "always"
            }
        }"#;
        let event: EventListResponse = serde_json::from_str(raw).unwrap();
        match &event {
            EventListResponse::PermissionReplied { properties } => {
                assert_eq!(properties.session_id, "sess_001");
                assert_eq!(properties.request_id, "req_001");
                assert_eq!(properties.reply, PermissionReply::Always);
            }
            other => panic!("expected PermissionReplied, got {other:?}"),
        }
    }

    // -- Missing event variant round-trips --

    #[test]
    fn lsp_client_diagnostics_round_trip() {
        let event = EventListResponse::LspClientDiagnostics {
            properties: LspClientDiagnosticsProps {
                path: "src/main.rs".into(),
                server_id: "rust-analyzer".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"lsp.client.diagnostics"#));
        assert!(json_str.contains(r#""serverID":"rust-analyzer"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn message_removed_round_trip() {
        let event = EventListResponse::MessageRemoved {
            properties: MessageRemovedProps {
                message_id: "msg_del_001".into(),
                session_id: "sess_001".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"message.removed"#));
        assert!(json_str.contains("messageID"));
        assert!(json_str.contains("sessionID"));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn message_part_updated_round_trip() {
        use crate::resources::session::{Part, TextPart};

        let event = EventListResponse::MessagePartUpdated {
            properties: MessagePartUpdatedProps {
                part: Part::Text(TextPart {
                    id: "p_upd_001".into(),
                    message_id: "msg_001".into(),
                    session_id: "sess_001".into(),
                    text: "updated text".into(),
                    synthetic: None,
                    time: None,
                }),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"message.part.updated"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn message_part_removed_round_trip() {
        let event = EventListResponse::MessagePartRemoved {
            properties: MessagePartRemovedProps {
                session_id: "sess_001".into(),
                message_id: "msg_001".into(),
                part_id: "p_del_001".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"message.part.removed"#));
        assert!(json_str.contains("sessionID"));
        assert!(json_str.contains("messageID"));
        assert!(json_str.contains("partID"));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn file_edited_round_trip() {
        let event = EventListResponse::FileEdited {
            properties: FileEditedProps { file: "src/lib.rs".into() },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"file.edited"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn session_updated_round_trip() {
        let event = EventListResponse::SessionUpdated {
            properties: SessionUpdatedProps {
                info: Session {
                    id: "sess_upd".into(),
                    slug: String::new(),
                    project_id: String::new(),
                    directory: String::new(),
                    time: crate::resources::session::SessionTime {
                        created: 1_700_000_000.0,
                        updated: 1_700_001_000.0,
                        compacting: None,
                        archived: None,
                    },
                    title: "Updated".into(),
                    version: "1".into(),
                    parent_id: None,
                    revert: None,
                    share: None,
                    summary: None,
                    permission: None,
                },
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"session.updated"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn session_deleted_round_trip() {
        let event = EventListResponse::SessionDeleted {
            properties: SessionDeletedProps {
                info: Session {
                    id: "sess_del".into(),
                    slug: String::new(),
                    project_id: String::new(),
                    directory: String::new(),
                    time: crate::resources::session::SessionTime {
                        created: 1_700_000_000.0,
                        updated: 1_700_000_000.0,
                        compacting: None,
                        archived: None,
                    },
                    title: "Deleted".into(),
                    version: "1".into(),
                    parent_id: None,
                    revert: None,
                    share: None,
                    summary: None,
                    permission: None,
                },
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"session.deleted"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn session_idle_round_trip() {
        let event = EventListResponse::SessionIdle {
            properties: SessionIdleProps { session_id: "sess_idle_001".into() },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"session.idle"#));
        assert!(json_str.contains("sessionID"));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn session_error_both_fields_null() {
        // Deserialize from JSON where both fields are explicitly null
        let raw = r#"{
            "type": "session.error",
            "properties": { "error": null, "sessionID": null }
        }"#;
        let event: EventListResponse = serde_json::from_str(raw).unwrap();
        match &event {
            EventListResponse::SessionError { properties } => {
                assert_eq!(properties.error, None);
                assert_eq!(properties.session_id, None);
            }
            other => panic!("expected SessionError, got {other:?}"),
        }
    }

    // -- New event variant tests --

    #[test]
    fn installation_update_available_round_trip() {
        let event = EventListResponse::InstallationUpdateAvailable {
            properties: InstallationUpdateAvailableProps { version: "2.0.0".into() },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"installation.update-available"#));
        assert!(json_str.contains(r#""version":"2.0.0"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn message_part_delta_round_trip() {
        let event = EventListResponse::MessagePartDelta {
            properties: MessagePartDeltaProps {
                session_id: "sess_001".into(),
                message_id: "msg_001".into(),
                part_id: "part_001".into(),
                field: "text".into(),
                delta: "hello ".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"message.part.delta"#));
        assert!(json_str.contains(r#""sessionID":"sess_001"#));
        assert!(json_str.contains(r#""delta":"hello "#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn server_connected_round_trip() {
        let event = EventListResponse::ServerConnected { properties: EmptyProps {} };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"server.connected"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn tui_toast_show_round_trip() {
        let event = EventListResponse::TuiToastShow {
            properties: TuiToastShowProps {
                title: Some("Heads up".into()),
                message: "Build succeeded".into(),
                variant: ToastVariant::Success,
                duration: Some(5.0),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"tui.toast.show"#));
        assert!(json_str.contains(r#""variant":"success"#));
        assert!(json_str.contains(r#""title":"Heads up"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);

        // Without optional fields
        let event2 = EventListResponse::TuiToastShow {
            properties: TuiToastShowProps {
                title: None,
                message: "Error occurred".into(),
                variant: ToastVariant::Error,
                duration: None,
            },
        };
        let json_str2 = serde_json::to_string(&event2).unwrap();
        assert!(!json_str2.contains(r#""title""#));
        assert!(!json_str2.contains(r#""duration""#));
        let back2: EventListResponse = serde_json::from_str(&json_str2).unwrap();
        assert_eq!(event2, back2);
    }

    #[test]
    fn todo_updated_round_trip() {
        let event = EventListResponse::TodoUpdated {
            properties: TodoUpdatedProps {
                session_id: "sess_001".into(),
                todos: vec![
                    Todo {
                        content: "Fix bug".into(),
                        status: "pending".into(),
                        priority: "high".into(),
                    },
                    Todo {
                        content: "Write docs".into(),
                        status: "done".into(),
                        priority: "low".into(),
                    },
                ],
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"todo.updated"#));
        assert!(json_str.contains("Fix bug"));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn worktree_ready_round_trip() {
        let event = EventListResponse::WorktreeReady {
            properties: WorktreeReadyProps {
                name: "feature-branch".into(),
                branch: "feat/new-feature".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"worktree.ready"#));
        assert!(json_str.contains(r#""name":"feature-branch"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn question_replied_round_trip() {
        let event = EventListResponse::QuestionReplied {
            properties: QuestionRepliedProps {
                session_id: "sess_001".into(),
                request_id: "req_001".into(),
                answers: vec![vec!["yes".into(), "no".into()]],
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"question.replied"#));
        assert!(json_str.contains(r#""sessionID":"sess_001"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn mcp_tools_changed_round_trip() {
        let event = EventListResponse::McpToolsChanged {
            properties: McpToolsChangedProps { server: "my-mcp-server".into() },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"mcp.tools.changed"#));
        assert!(json_str.contains(r#""server":"my-mcp-server"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn pty_created_round_trip() {
        let event = EventListResponse::PtyCreated {
            properties: PtyCreatedProps {
                info: Pty {
                    id: "pty_001".into(),
                    title: "shell".into(),
                    command: "/bin/zsh".into(),
                    args: vec!["-l".into()],
                    cwd: "/home/user".into(),
                    status: PtyStatus::Running,
                    pid: 12345.0,
                },
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"pty.created"#));
        assert!(json_str.contains(r#""status":"running"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn vcs_branch_updated_round_trip() {
        let event = EventListResponse::VcsBranchUpdated {
            properties: VcsBranchUpdatedProps { branch: Some("main".into()) },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"vcs.branch.updated"#));
        assert!(json_str.contains(r#""branch":"main"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);

        // Branch is optional
        let event2 = EventListResponse::VcsBranchUpdated {
            properties: VcsBranchUpdatedProps { branch: None },
        };
        let json_str2 = serde_json::to_string(&event2).unwrap();
        assert!(!json_str2.contains(r#""branch""#));
        let back2: EventListResponse = serde_json::from_str(&json_str2).unwrap();
        assert_eq!(event2, back2);
    }

    #[test]
    fn command_executed_round_trip() {
        let event = EventListResponse::CommandExecuted {
            properties: CommandExecutedProps {
                name: "test-cmd".into(),
                session_id: "sess_001".into(),
                arguments: "{}".into(),
                message_id: "msg_001".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"command.executed"#));
        assert!(json_str.contains(r#""sessionID":"sess_001"#));
        assert!(json_str.contains(r#""messageID":"msg_001"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn deserialize_project_updated_from_raw() {
        let raw = r#"{
            "type": "project.updated",
            "properties": {
                "properties": { "name": "my-project", "path": "/tmp/proj" }
            }
        }"#;
        let event: EventListResponse = serde_json::from_str(raw).unwrap();
        match &event {
            EventListResponse::ProjectUpdated { properties } => {
                assert_eq!(properties.properties["name"], "my-project");
            }
            other => panic!("expected ProjectUpdated, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_session_status_from_raw() {
        let raw = r#"{
            "type": "session.status",
            "properties": {
                "sessionID": "sess_001",
                "status": { "type": "running", "tool": "bash" }
            }
        }"#;
        let event: EventListResponse = serde_json::from_str(raw).unwrap();
        match &event {
            EventListResponse::SessionStatus { properties } => {
                assert_eq!(properties.session_id, "sess_001");
                assert_eq!(properties.status["type"], "running");
            }
            other => panic!("expected SessionStatus, got {other:?}"),
        }
    }
}
