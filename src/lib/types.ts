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
}
