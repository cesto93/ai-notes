const API_BASE = 'http://localhost:8000';

export async function fetchNotes() {
    const res = await fetch(`${API_BASE}/notes`);
    return res.json();
}

export async function fetchNote(path: string) {
    const res = await fetch(`${API_BASE}/note/${encodeURIComponent(path)}`);
    return res.json();
}

export async function saveNote(note: { title: string; content: string; directory: string }) {
    const res = await fetch(`${API_BASE}/note`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(note)
    });
    return res.json();
}

export async function updateNote(data: {
    old_title: string;
    old_directory: string;
    new_content: string;
    new_title: string;
    new_directory: string;
}) {
    const res = await fetch(`${API_BASE}/note`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data)
    });
    return res.json();
}

export async function moveNote(title: string, old_directory: string, new_directory: string) {
    const res = await fetch(`${API_BASE}/note/move`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, old_directory, new_directory })
    });
    return res.json();
}

export async function createDirectory(name: string) {
    const res = await fetch(`${API_BASE}/directory`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name })
    });
    return res.json();
}

export async function summarize(text: string) {
    const res = await fetch(`${API_BASE}/summarize`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ text })
    });
    return res.json();
}

export async function paraphrase(text: string) {
    const res = await fetch(`${API_BASE}/paraphrase`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ text })
    });
    return res.json();
}

export async function deleteNote(title: string, directory: string = "") {
    const dirParam = directory || "none";
    const res = await fetch(`${API_BASE}/note/${encodeURIComponent(dirParam)}/${encodeURIComponent(title)}`, {
        method: 'DELETE'
    });
    return res.json();
}

export async function deleteDirectory(name: string) {
    const res = await fetch(`${API_BASE}/directory/${encodeURIComponent(name)}`, {
        method: 'DELETE'
    });
    return res.json();
}

export async function fetchSettings() {
    const res = await fetch(`${API_BASE}/settings`);
    return res.json();
}

export async function updateSettings(settings: { provider: string; model: string }) {
    const res = await fetch(`${API_BASE}/settings`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings)
    });
    return res.json();
}
