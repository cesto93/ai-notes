use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub title: String,
    pub content: String,
    pub directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNoteRequest {
    pub old_title: String,
    pub old_directory: String,
    pub new_content: String,
    pub new_title: String,
    pub new_directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveNoteRequest {
    pub title: String,
    pub old_directory: String,
    pub new_directory: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub provider: String,
    pub model: String,
    /// Root folder where notes are stored. Must be set — no default fallback.
    #[serde(default)]
    pub notes_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteMetadata {
    pub argument: String,
    pub directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteListResponse {
    pub files: Vec<String>,
    pub directories: std::collections::HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    /// Note path relative to the notes root (same format as `get_note`).
    pub path: String,
    pub title: String,
    pub directory: String,
    /// Tags matched by the query (empty when the note matched on title/text only).
    pub tags: Vec<String>,
    /// Short context around the first content match; first line otherwise.
    pub snippet: String,
    pub match_title: bool,
    pub match_tags: bool,
    pub match_content: bool,
}
