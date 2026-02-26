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
