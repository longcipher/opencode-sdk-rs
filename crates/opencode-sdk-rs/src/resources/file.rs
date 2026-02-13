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

/// The type of content returned when reading a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileReadType {
    /// Raw file content.
    Raw,
    /// A patch / diff.
    Patch,
}

/// Response from reading a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileReadResponse {
    /// The file content (raw text or patch).
    pub content: String,
    /// The type of content returned.
    #[serde(rename = "type")]
    pub file_type: FileReadType,
}

/// Query parameters for reading a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileReadParams {
    /// Path of the file to read.
    pub path: String,
}

/// Response type for the file status endpoint.
pub type FileStatusResponse = Vec<FileInfo>;

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
    /// `GET /file?path=<path>`
    pub async fn read(&self, params: &FileReadParams) -> Result<FileReadResponse, OpencodeError> {
        self.client.get_with_query("/file", Some(params), None).await
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
    fn file_read_type_round_trip() {
        for (variant, expected) in
            [(FileReadType::Raw, "\"raw\""), (FileReadType::Patch, "\"patch\"")]
        {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let parsed: FileReadType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn file_read_response_round_trip() {
        let resp =
            FileReadResponse { content: "fn main() {}".to_string(), file_type: FileReadType::Raw };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(r#""type":"raw""#));
        let parsed: FileReadResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, resp);
    }

    #[test]
    fn file_read_response_deserialize_from_js() {
        let json = r#"{"content": "diff --git a/file", "type": "patch"}"#;
        let resp: FileReadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content, "diff --git a/file");
        assert_eq!(resp.file_type, FileReadType::Patch);
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
}
