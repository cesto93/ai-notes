package models

type Note struct {
	Title     string `json:"title"`
	Content   string `json:"content"`
	Directory string `json:"directory"`
}

type NoteRequest struct {
	Content   string `json:"content"`
	Title     string `json:"title"`
	Directory string `json:"directory"`
}

type UpdateNoteRequest struct {
	OldTitle     string `json:"old_title"`
	OldDirectory string `json:"old_directory"`
	NewContent   string `json:"new_content"`
	NewTitle     string `json:"new_title"`
	NewDirectory string `json:"new_directory"`
}

type SummarizeRequest struct {
	Text string `json:"text"`
}

type DirectoryRequest struct {
	Name string `json:"name"`
}

type MoveNoteRequest struct {
	Title        string `json:"title"`
	OldDirectory string `json:"old_directory"`
	NewDirectory string `json:"new_directory"`
}

type SettingsRequest struct {
	Provider string `json:"provider"`
	Model    string `json:"model"`
}

type Settings struct {
	Provider string `json:"provider"`
	Model    string `json:"model"`
}

type NoteMetadata struct {
	Argument  string `json:"argument"`
	Directory string `json:"directory"`
}
