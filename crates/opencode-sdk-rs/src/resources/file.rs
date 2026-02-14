//! File resource types and methods mirroring the JS SDK's `resources/file.ts`.

use serde::{Deserialize, Serialize};

use crate::{client::Opencode, error::OpencodeError};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The status of a file in the project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    /// The file was newly added.
    Added,
    /// The file was deleted.
    Deleted,
    /// The file was modified.
    Modified,
}

/// Information about a single file.
///
/// Named `FileInfo` instead of `File` to avoid collision with `std::fs::File`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileInfo {
    /// Number of lines added.
    pub added: i64,
    /// The file path.
    pub path: String,
    /// Number of lines removed.
    pub removed: i64,
    /// Current status of the file.
    pub status: FileStatus,
}

/// Query parameters for reading a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileReadParams {
    /// Path of the file to read.
    pub path: String,
}

/// Query parameters for listing files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileListParams {
    /// Path to list files from.
    pub path: String,
}

/// Response type for the file status endpoint.
pub type FileStatusResponse = Vec<FileInfo>;

/// The type of a node in a directory listing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileNodeType {
    /// A regular file.
    File,
    /// A directory.
    Directory,
}

/// A node in a directory listing returned by `GET /file`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileNode {
    /// The file or directory name.
    pub name: String,
    /// Relative path from the project root.
    pub path: String,
    /// Absolute path on disk.
    pub absolute: String,
    /// Whether this node is a file or a directory.
    #[serde(rename = "type")]
    pub node_type: FileNodeType,
    /// Whether this node is ignored (e.g. by `.gitignore`).
    pub ignored: bool,
}

/// The content type of a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileContentType {
    /// Plain text content.
    Text,
    /// Binary content (typically base64-encoded).
    Binary,
}

/// A single hunk in a structured patch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FilePatchHunk {
    /// Starting line number in the old file.
    pub old_start: f64,
    /// Number of lines in the old file.
    pub old_lines: f64,
    /// Starting line number in the new file.
    pub new_start: f64,
    /// Number of lines in the new file.
    pub new_lines: f64,
    /// The diff lines (prefixed with `+`, `-`, or ` `).
    pub lines: Vec<String>,
}

/// A structured patch describing changes between two file versions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FilePatch {
    /// The old file name.
    pub old_file_name: String,
    /// The new file name.
    pub new_file_name: String,
    /// The old file header (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_header: Option<String>,
    /// The new file header (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_header: Option<String>,
    /// The list of hunks in this patch.
    pub hunks: Vec<FilePatchHunk>,
    /// The index line (optional, e.g. git blob hashes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
}

/// The content of a file as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileContent {
    /// Whether the content is text or binary.
    #[serde(rename = "type")]
    pub content_type: FileContentType,
    /// The file content (plain text or base64-encoded binary).
    pub content: String,
    /// A unified diff string (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    /// A structured patch (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<FilePatch>,
    /// The encoding of the content (e.g. `"base64"` for binary files).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    /// The MIME type of the file (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Accessor for the `/file` endpoints.
pub struct FileResource<'a> {
    client: &'a Opencode,
}

impl<'a> FileResource<'a> {
    pub(crate) const fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    /// Read a file's content.
    ///
    /// `GET /file/content?path=<path>`
    pub async fn read(&self, params: &FileReadParams) -> Result<FileContent, OpencodeError> {
        self.client.get_with_query("/file/content", Some(params), None).await
    }

    /// List all files in the project directory tree.
    ///
    /// `GET /file?path=<path>`
    pub async fn list(
        &self,
        params: Option<&FileListParams>,
    ) -> Result<Vec<FileNode>, OpencodeError> {
        self.client.get_with_query("/file", params, None).await
    }

    /// Get the status of all files in the project.
    ///
    /// `GET /file/status`
    pub async fn status(&self) -> Result<FileStatusResponse, OpencodeError> {
        self.client.get("/file/status", None).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json;

    use super::*;

    #[test]
    fn file_status_round_trip() {
        for (variant, expected) in [
            (FileStatus::Added, "\"added\""),
            (FileStatus::Deleted, "\"deleted\""),
            (FileStatus::Modified, "\"modified\""),
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let parsed: FileStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn file_info_round_trip() {
        let info = FileInfo {
            added: 10,
            path: "src/main.rs".to_string(),
            removed: 3,
            status: FileStatus::Modified,
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: FileInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, info);
    }

    #[test]
    fn file_info_deserialize_from_js() {
        let json = r#"{
            "added": 5,
            "path": "README.md",
            "removed": 0,
            "status": "added"
        }"#;
        let info: FileInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.added, 5);
        assert_eq!(info.path, "README.md");
        assert_eq!(info.removed, 0);
        assert_eq!(info.status, FileStatus::Added);
    }

    #[test]
    fn file_read_params_round_trip() {
        let params = FileReadParams { path: "src/lib.rs".to_string() };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: FileReadParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, params);
    }

    #[test]
    fn file_status_response_round_trip() {
        let response: FileStatusResponse = vec![
            FileInfo { added: 1, path: "a.rs".to_string(), removed: 0, status: FileStatus::Added },
            FileInfo {
                added: 0,
                path: "b.rs".to_string(),
                removed: 10,
                status: FileStatus::Deleted,
            },
        ];
        let json = serde_json::to_string(&response).unwrap();
        let parsed: FileStatusResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, response);
    }

    // -- FileNodeType --

    #[test]
    fn file_node_type_round_trip() {
        for (variant, expected) in
            [(FileNodeType::File, "\"file\""), (FileNodeType::Directory, "\"directory\"")]
        {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let parsed: FileNodeType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    // -- FileNode --

    #[test]
    fn file_node_round_trip() {
        let node = FileNode {
            name: "main.rs".to_string(),
            path: "src/main.rs".to_string(),
            absolute: "/home/user/project/src/main.rs".to_string(),
            node_type: FileNodeType::File,
            ignored: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""type":"file""#));
        let parsed: FileNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, node);
    }

    #[test]
    fn file_node_deserialize_from_api() {
        let json = r#"{
            "name": "src",
            "path": "src",
            "absolute": "/home/user/project/src",
            "type": "directory",
            "ignored": true
        }"#;
        let node: FileNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.name, "src");
        assert_eq!(node.node_type, FileNodeType::Directory);
        assert!(node.ignored);
    }

    // -- FileContentType --

    #[test]
    fn file_content_type_round_trip() {
        for (variant, expected) in
            [(FileContentType::Text, "\"text\""), (FileContentType::Binary, "\"binary\"")]
        {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let parsed: FileContentType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    // -- FilePatchHunk --

    #[test]
    fn file_patch_hunk_round_trip() {
        let hunk = FilePatchHunk {
            old_start: 1.0,
            old_lines: 3.0,
            new_start: 1.0,
            new_lines: 4.0,
            lines: vec![
                " fn main() {".to_string(),
                "-    println!(\"old\");".to_string(),
                "+    println!(\"new\");".to_string(),
                "+    println!(\"extra\");".to_string(),
                " }".to_string(),
            ],
        };
        let json = serde_json::to_string(&hunk).unwrap();
        assert!(json.contains(r#""oldStart":1"#));
        assert!(json.contains(r#""newLines":4"#));
        let parsed: FilePatchHunk = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, hunk);
    }

    #[test]
    fn file_patch_hunk_deserialize_camel_case() {
        let json = r#"{
            "oldStart": 10,
            "oldLines": 2,
            "newStart": 10,
            "newLines": 3,
            "lines": [" a", "-b", "+c", "+d"]
        }"#;
        let hunk: FilePatchHunk = serde_json::from_str(json).unwrap();
        assert_eq!(hunk.old_start, 10.0);
        assert_eq!(hunk.new_lines, 3.0);
        assert_eq!(hunk.lines.len(), 4);
    }

    // -- FilePatch --

    #[test]
    fn file_patch_round_trip() {
        let patch = FilePatch {
            old_file_name: "a.rs".to_string(),
            new_file_name: "a.rs".to_string(),
            old_header: Some("old-header".to_string()),
            new_header: Some("new-header".to_string()),
            hunks: vec![FilePatchHunk {
                old_start: 1.0,
                old_lines: 1.0,
                new_start: 1.0,
                new_lines: 1.0,
                lines: vec!["-old".to_string(), "+new".to_string()],
            }],
            index: Some("abc123..def456".to_string()),
        };
        let json = serde_json::to_string(&patch).unwrap();
        assert!(json.contains(r#""oldFileName":"a.rs""#));
        let parsed: FilePatch = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, patch);
    }

    #[test]
    fn file_patch_optional_fields_omitted() {
        let patch = FilePatch {
            old_file_name: "x.rs".to_string(),
            new_file_name: "x.rs".to_string(),
            old_header: None,
            new_header: None,
            hunks: vec![],
            index: None,
        };
        let json = serde_json::to_string(&patch).unwrap();
        assert!(!json.contains("oldHeader"));
        assert!(!json.contains("newHeader"));
        assert!(!json.contains("index"));
        let parsed: FilePatch = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, patch);
    }

    // -- FileContent --

    #[test]
    fn file_content_text_round_trip() {
        let content = FileContent {
            content_type: FileContentType::Text,
            content: "fn main() {}".to_string(),
            diff: Some("--- a\n+++ b".to_string()),
            patch: None,
            encoding: None,
            mime_type: Some("text/x-rust".to_string()),
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains(r#""mimeType":"text/x-rust""#));
        assert!(!json.contains("encoding"));
        let parsed: FileContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, content);
    }

    #[test]
    fn file_content_binary_round_trip() {
        let content = FileContent {
            content_type: FileContentType::Binary,
            content: "aGVsbG8=".to_string(),
            diff: None,
            patch: None,
            encoding: Some("base64".to_string()),
            mime_type: Some("image/png".to_string()),
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""type":"binary""#));
        assert!(json.contains(r#""encoding":"base64""#));
        let parsed: FileContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, content);
    }

    #[test]
    fn file_content_with_patch_round_trip() {
        let content = FileContent {
            content_type: FileContentType::Text,
            content: "updated content".to_string(),
            diff: Some("@@ -1 +1 @@".to_string()),
            patch: Some(FilePatch {
                old_file_name: "lib.rs".to_string(),
                new_file_name: "lib.rs".to_string(),
                old_header: None,
                new_header: None,
                hunks: vec![FilePatchHunk {
                    old_start: 1.0,
                    old_lines: 1.0,
                    new_start: 1.0,
                    new_lines: 1.0,
                    lines: vec!["-old line".to_string(), "+new line".to_string()],
                }],
                index: None,
            }),
            encoding: None,
            mime_type: None,
        };
        let json = serde_json::to_string(&content).unwrap();
        let parsed: FileContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, content);
    }

    #[test]
    fn file_content_deserialize_minimal_from_api() {
        let json = r#"{"type": "text", "content": "hello world"}"#;
        let content: FileContent = serde_json::from_str(json).unwrap();
        assert_eq!(content.content_type, FileContentType::Text);
        assert_eq!(content.content, "hello world");
        assert!(content.diff.is_none());
        assert!(content.patch.is_none());
        assert!(content.encoding.is_none());
        assert!(content.mime_type.is_none());
    }
}
