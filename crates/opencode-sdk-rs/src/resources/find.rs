//! Find resource types and methods mirroring the JS SDK's `resources/find.ts`.

use serde::{Deserialize, Serialize};

use crate::{client::Opencode, error::OpencodeError};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A position in a text document (zero-based line and character offset).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    /// Zero-based character offset on the line.
    pub character: i64,
    /// Zero-based line number.
    pub line: i64,
}

/// A range in a text document represented by a start and end position.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Range {
    /// The range's end position.
    pub end: Position,
    /// The range's start position.
    pub start: Position,
}

/// The location of a symbol, including its URI and range.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolLocation {
    /// The range within the document.
    pub range: Range,
    /// The URI of the document.
    pub uri: String,
}

/// Information about a symbol found in the workspace.
///
/// Named `SymbolInfo` instead of `Symbol` to avoid collision with `std::symbol`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolInfo {
    /// The kind of symbol (e.g. function, class, variable).
    pub kind: i64,
    /// The location of the symbol.
    pub location: SymbolLocation,
    /// The name of the symbol.
    pub name: String,
}

/// A text match value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextMatch {
    /// The matched text.
    pub text: String,
}

/// A sub-match within a text search result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Submatch {
    /// End byte offset of the sub-match.
    pub end: i64,
    /// The matched text.
    #[serde(rename = "match")]
    pub match_info: TextMatch,
    /// Start byte offset of the sub-match.
    pub start: i64,
}

/// Lines context for a text search result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lines {
    /// The text of the matched line(s).
    pub text: String,
}

/// Path information for a text search result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathInfo {
    /// The file path as text.
    pub text: String,
}

/// A single item in a text search response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindTextResponseItem {
    /// Absolute byte offset of the match in the file.
    pub absolute_offset: i64,
    /// One-based line number of the match.
    pub line_number: i64,
    /// The matched line(s).
    pub lines: Lines,
    /// The file path.
    pub path: PathInfo,
    /// Sub-matches within this result.
    pub submatches: Vec<Submatch>,
}

/// Response from searching for files by name.
pub type FindFilesResponse = Vec<String>;

/// Response from searching for symbols.
pub type FindSymbolsResponse = Vec<SymbolInfo>;

/// Response from searching for text in files.
pub type FindTextResponse = Vec<FindTextResponseItem>;

// ---------------------------------------------------------------------------
// Params
// ---------------------------------------------------------------------------

/// Query parameters for searching files by name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindFilesParams {
    /// The file name query.
    pub query: String,
}

/// Query parameters for searching symbols.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindSymbolsParams {
    /// The symbol name query.
    pub query: String,
}

/// Query parameters for searching text in files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindTextParams {
    /// The text pattern to search for.
    pub pattern: String,
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Accessor for the `/find` endpoints.
pub struct FindResource<'a> {
    client: &'a Opencode,
}

impl<'a> FindResource<'a> {
    pub(crate) const fn new(client: &'a Opencode) -> Self {
        Self { client }
    }

    /// Search for files by name.
    ///
    /// `GET /find/file?query=<query>`
    pub async fn files(
        &self,
        params: &FindFilesParams,
    ) -> Result<FindFilesResponse, OpencodeError> {
        self.client.get_with_query("/find/file", Some(params), None).await
    }

    /// Search for symbols by name.
    ///
    /// `GET /find/symbol?query=<query>`
    pub async fn symbols(
        &self,
        params: &FindSymbolsParams,
    ) -> Result<FindSymbolsResponse, OpencodeError> {
        self.client.get_with_query("/find/symbol", Some(params), None).await
    }

    /// Search for text in files.
    ///
    /// `GET /find?pattern=<pattern>`
    pub async fn text(&self, params: &FindTextParams) -> Result<FindTextResponse, OpencodeError> {
        self.client.get_with_query("/find", Some(params), None).await
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
    fn symbol_info_round_trip() {
        let symbol = SymbolInfo {
            kind: 12,
            location: SymbolLocation {
                range: Range {
                    end: Position { character: 20, line: 10 },
                    start: Position { character: 5, line: 10 },
                },
                uri: "file:///src/main.rs".to_owned(),
            },
            name: "my_function".to_owned(),
        };

        let json = serde_json::to_string(&symbol).unwrap();
        let deserialized: SymbolInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(symbol, deserialized);

        // Verify specific JSON structure
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["kind"], 12);
        assert_eq!(value["name"], "my_function");
        assert_eq!(value["location"]["uri"], "file:///src/main.rs");
        assert_eq!(value["location"]["range"]["start"]["line"], 10);
        assert_eq!(value["location"]["range"]["start"]["character"], 5);
        assert_eq!(value["location"]["range"]["end"]["character"], 20);
    }

    #[test]
    fn find_text_response_item_round_trip() {
        let item = FindTextResponseItem {
            absolute_offset: 1024,
            line_number: 42,
            lines: Lines { text: "    let x = 42;".to_owned() },
            path: PathInfo { text: "src/main.rs".to_owned() },
            submatches: vec![Submatch {
                end: 15,
                match_info: TextMatch { text: "42".to_owned() },
                start: 13,
            }],
        };

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: FindTextResponseItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);

        // Verify the `match` field is serialised with its original name
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["absolute_offset"], 1024);
        assert_eq!(value["line_number"], 42);
        assert_eq!(value["lines"]["text"], "    let x = 42;");
        assert_eq!(value["path"]["text"], "src/main.rs");
        assert_eq!(value["submatches"][0]["match"]["text"], "42");
        assert_eq!(value["submatches"][0]["start"], 13);
        assert_eq!(value["submatches"][0]["end"], 15);
    }

    #[test]
    fn find_text_response_item_deserialize_match_field() {
        // Ensure we can deserialize from JSON where the field is called "match"
        let json = r#"{
            "absolute_offset": 0,
            "line_number": 1,
            "lines": { "text": "hello world" },
            "path": { "text": "test.txt" },
            "submatches": [{
                "end": 5,
                "match": { "text": "hello" },
                "start": 0
            }]
        }"#;

        let item: FindTextResponseItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.submatches[0].match_info.text, "hello");
    }

    #[test]
    fn find_files_params_serialize() {
        let params = FindFilesParams { query: "main.rs".to_owned() };
        let json = serde_json::to_string(&params).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["query"], "main.rs");
    }

    #[test]
    fn find_symbols_params_serialize() {
        let params = FindSymbolsParams { query: "MyStruct".to_owned() };
        let json = serde_json::to_string(&params).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["query"], "MyStruct");
    }

    #[test]
    fn find_text_params_serialize() {
        let params = FindTextParams { pattern: "TODO".to_owned() };
        let json = serde_json::to_string(&params).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["pattern"], "TODO");
    }

    #[test]
    fn find_symbols_response_round_trip() {
        let response: FindSymbolsResponse = vec![SymbolInfo {
            kind: 5,
            location: SymbolLocation {
                range: Range {
                    end: Position { character: 10, line: 0 },
                    start: Position { character: 0, line: 0 },
                },
                uri: "file:///lib.rs".to_owned(),
            },
            name: "Foo".to_owned(),
        }];

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: FindSymbolsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn find_files_response_round_trip() {
        let response: FindFilesResponse = vec!["src/main.rs".to_owned(), "src/lib.rs".to_owned()];

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: FindFilesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }

    // -- Edge cases --

    #[test]
    fn find_text_response_item_empty_submatches() {
        let item = FindTextResponseItem {
            absolute_offset: 0,
            line_number: 1,
            lines: Lines { text: "no matches here".to_owned() },
            path: PathInfo { text: "test.txt".to_owned() },
            submatches: vec![],
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: FindTextResponseItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);
        assert!(deserialized.submatches.is_empty());
    }

    #[test]
    fn find_text_response_empty_vec() {
        let response: FindTextResponse = vec![];
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, "[]");
        let deserialized: FindTextResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn find_files_response_empty() {
        let response: FindFilesResponse = vec![];
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, "[]");
        let deserialized: FindFilesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn find_symbols_response_empty() {
        let response: FindSymbolsResponse = vec![];
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, "[]");
        let deserialized: FindSymbolsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}
