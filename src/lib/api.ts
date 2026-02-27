import { invoke } from '@tauri-apps/api/core';
import type { Note, NoteListResponse, Settings } from './types';

export async function fetchNotes(): Promise<NoteListResponse> {
    return await invoke('list_notes');
}

export async function fetchNote(path: string): Promise<Note> {
    return await invoke('get_note', { path });
}

export async function saveNote(note: { title: string; content: string; directory: string }) {
    return await invoke('save_note', {
        content: note.content,
        title: note.title,
        directory: note.directory
    });
}

export async function updateNote(data: {
    old_title: string;
    old_directory: string;
    new_content: string;
    new_title: string;
    new_directory: string;
}) {
    return await invoke('update_note', { req: data });
}

export async function renameNote(oldTitle: string, directory: string, newTitle: string) {
    return await invoke('rename_note', { oldTitle, directory, newTitle });
}

export async function moveNote(title: string, old_directory: string, new_directory: string) {
    return await invoke('move_note', {
        req: { title, old_directory, new_directory }
    });
}

export async function createDirectory(name: string) {
    return await invoke('create_directory', { name });
}

export async function summarize(text: string): Promise<string> {
    return await invoke('summarize', { text });
}

export async function paraphrase(text: string): Promise<string> {
    return await invoke('paraphrase', { text });
}

export async function deleteNote(title: string, directory: string = "") {
    return await invoke('delete_note', { title, directory });
}

export async function deleteDirectory(name: string) {
    return await invoke('delete_directory', { name });
}

export async function renameDirectory(oldName: string, newName: string) {
    return await invoke('rename_directory', { oldName, newName });
}

export async function fetchSettings(): Promise<Settings> {
    return await invoke('get_settings');
}

export async function updateSettings(settings: { provider: string; model: string }) {
    return await invoke('save_settings', {
        provider: settings.provider,
        model: settings.model
    });
}

export async function mindmap(text: string): Promise<string> {
    return await invoke('mindmap', { text });
}
