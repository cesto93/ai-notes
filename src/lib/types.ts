export interface Note {
    title: string;
    content: string;
    directory: string;
}

export interface NoteListResponse {
    files: string[];
    directories: Record<string, string[]>;
}

export interface SearchResult {
    /** Note path relative to the notes root, same format as get_note(). */
    path: string;
    title: string;
    directory: string;
    /** Tags matched by the query (empty when the note matched on title/text only). */
    tags: string[];
    /** Short context around the first content match; first plain line otherwise. */
    snippet: string;
    match_title: boolean;
    match_tags: boolean;
    match_content: boolean;
}

export interface Settings {
    provider: string;
    model: string;
    /** Root folder where notes are stored. Must be set — the app has no default location. */
    notes_dir: string;
}
