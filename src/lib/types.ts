export interface Note {
    title: string;
    content: string;
    directory: string;
}

export interface NoteListResponse {
    files: string[];
    directories: Record<string, string[]>;
}

export interface Settings {
    provider: string;
    model: string;
    /** Root folder where notes are stored. Must be set — the app has no default location. */
    notes_dir: string;
}
