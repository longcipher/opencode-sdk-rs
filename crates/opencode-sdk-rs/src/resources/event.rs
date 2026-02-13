//! Event resource types and the `EventResource` struct.
//!
//! Events are delivered via Server-Sent Events (SSE).  The [`EventResource`]
//! will expose a streaming `list()` method once SSE support is wired up
//! (Task 3.1).  For now only the data types are defined.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    session::{Message, Part, Session},
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
    /// An installation was updated to a new version.
    #[serde(rename = "installation.updated")]
    InstallationUpdated {
        /// Payload.
        properties: InstallationUpdatedProps,
    },

    /// LSP client diagnostics were received.
    #[serde(rename = "lsp.client.diagnostics")]
    LspClientDiagnostics {
        /// Payload.
        properties: LspClientDiagnosticsProps,
    },

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

    /// A message part was removed.
    #[serde(rename = "message.part.removed")]
    MessagePartRemoved {
        /// Payload.
        properties: MessagePartRemovedProps,
    },

    /// A storage key was written.
    #[serde(rename = "storage.write")]
    StorageWrite {
        /// Payload.
        properties: StorageWriteProps,
    },

    /// A permission was updated.
    #[serde(rename = "permission.updated")]
    PermissionUpdated {
        /// Payload.
        properties: PermissionUpdatedProps,
    },

    /// A file was edited.
    #[serde(rename = "file.edited")]
    FileEdited {
        /// Payload.
        properties: FileEditedProps,
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

    /// A session became idle.
    #[serde(rename = "session.idle")]
    SessionIdle {
        /// Payload.
        properties: SessionIdleProps,
    },

    /// A session encountered an error.
    #[serde(rename = "session.error")]
    SessionError {
        /// Payload.
        properties: SessionErrorProps,
    },

    /// A file-watcher event was received.
    #[serde(rename = "file.watcher.updated")]
    FileWatcherUpdated {
        /// Payload.
        properties: FileWatcherUpdatedProps,
    },

    /// An IDE was installed.
    #[serde(rename = "ide.installed")]
    IdeInstalled {
        /// Payload.
        properties: IdeInstalledProps,
    },
}

// ---------------------------------------------------------------------------
// Property structs
// ---------------------------------------------------------------------------

/// Properties for [`EventListResponse::InstallationUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallationUpdatedProps {
    /// New version string.
    pub version: String,
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

/// Properties for [`EventListResponse::MessagePartRemoved`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessagePartRemovedProps {
    /// Message the part belonged to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// ID of the removed part.
    #[serde(rename = "partID")]
    pub part_id: String,
}

/// Properties for [`EventListResponse::StorageWrite`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageWriteProps {
    /// Storage key.
    pub key: String,
    /// Optional storage content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

/// Properties for [`EventListResponse::PermissionUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionUpdatedProps {
    /// Permission identifier.
    pub id: String,
    /// Arbitrary metadata map.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Session the permission belongs to.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Timestamps.
    pub time: PermissionTime,
    /// Human-readable title.
    pub title: String,
}

/// Timestamp container for [`PermissionUpdatedProps`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionTime {
    /// Unix epoch seconds when the permission was created.
    pub created: f64,
}

/// Properties for [`EventListResponse::FileEdited`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEditedProps {
    /// The edited file path.
    pub file: String,
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

/// Properties for [`EventListResponse::SessionIdle`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionIdleProps {
    /// The idle session's ID.
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

/// Properties for [`EventListResponse::SessionError`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionErrorProps {
    /// The error, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SessionError>,
    /// The session ID, if available.
    #[serde(rename = "sessionID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Kind of file-watcher event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileWatcherEvent {
    /// A file was renamed.
    #[serde(rename = "rename")]
    Rename,
    /// A file was changed.
    #[serde(rename = "change")]
    Change,
}

/// Properties for [`EventListResponse::FileWatcherUpdated`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileWatcherUpdatedProps {
    /// The kind of file-system event.
    pub event: FileWatcherEvent,
    /// The affected file path.
    pub file: String,
}

/// Properties for [`EventListResponse::IdeInstalled`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdeInstalledProps {
    /// The IDE identifier.
    pub ide: String,
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
    use serde_json::json;

    use super::*;
    use crate::resources::session::{UserMessage, UserMessageTime};

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
        let msg = Message::User(UserMessage {
            id: "msg_u001".into(),
            session_id: "sess_001".into(),
            time: UserMessageTime { created: 1_700_000_100.0 },
        });

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
                event: FileWatcherEvent::Rename,
                file: "src/main.rs".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"file.watcher.updated"#));
        assert!(json_str.contains(r#""event":"rename"#));
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
    }

    // -- StorageWrite round-trip --

    #[test]
    fn storage_write_round_trip() {
        let event = EventListResponse::StorageWrite {
            properties: StorageWriteProps {
                key: "my-key".into(),
                content: Some(json!({"nested": true, "count": 42})),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"storage.write"#));
        assert!(json_str.contains(r#""key":"my-key"#));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn storage_write_no_content_round_trip() {
        let event = EventListResponse::StorageWrite {
            properties: StorageWriteProps { key: "empty-key".into(), content: None },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(!json_str.contains("content"));
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
    }

    // -- Deserialization from raw JSON --

    #[test]
    fn deserialize_from_raw_json() {
        let raw = r#"{
            "type": "ide.installed",
            "properties": { "ide": "vscode" }
        }"#;
        let event: EventListResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(
            event,
            EventListResponse::IdeInstalled {
                properties: IdeInstalledProps { ide: "vscode".into() }
            }
        );
    }

    #[test]
    fn deserialize_permission_updated() {
        let raw = r#"{
            "type": "permission.updated",
            "properties": {
                "id": "perm_001",
                "metadata": {"tool": "bash"},
                "sessionID": "sess_001",
                "time": {"created": 1700000000.0},
                "title": "Run bash command"
            }
        }"#;
        let event: EventListResponse = serde_json::from_str(raw).unwrap();
        match &event {
            EventListResponse::PermissionUpdated { properties } => {
                assert_eq!(properties.id, "perm_001");
                assert_eq!(properties.session_id, "sess_001");
                assert_eq!(properties.title, "Run bash command");
                assert_eq!(properties.time.created, 1_700_000_000.0);
                assert_eq!(properties.metadata.get("tool"), Some(&json!("bash")));
            }
            other => panic!("expected PermissionUpdated, got {other:?}"),
        }
        // Round-trip
        let json_str = serde_json::to_string(&event).unwrap();
        let back: EventListResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event, back);
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
                message_id: "msg_001".into(),
                part_id: "p_del_001".into(),
            },
        };
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains(r#""type":"message.part.removed"#));
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
                    time: crate::resources::session::SessionTime {
                        created: 1_700_000_000.0,
                        updated: 1_700_001_000.0,
                    },
                    title: "Updated".into(),
                    version: "1".into(),
                    parent_id: None,
                    revert: None,
                    share: None,
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
                    time: crate::resources::session::SessionTime {
                        created: 1_700_000_000.0,
                        updated: 1_700_000_000.0,
                    },
                    title: "Deleted".into(),
                    version: "1".into(),
                    parent_id: None,
                    revert: None,
                    share: None,
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
    fn storage_write_null_content() {
        // Deserialize from JSON where content is explicitly null
        let raw = r#"{
            "type": "storage.write",
            "properties": { "key": "k", "content": null }
        }"#;
        let event: EventListResponse = serde_json::from_str(raw).unwrap();
        match &event {
            EventListResponse::StorageWrite { properties } => {
                assert_eq!(properties.key, "k");
                assert_eq!(properties.content, None);
            }
            other => panic!("expected StorageWrite, got {other:?}"),
        }
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
}
