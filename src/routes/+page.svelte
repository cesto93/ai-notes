<script lang="ts">
    import { onMount } from 'svelte';
    import Sidebar from '$lib/components/Sidebar.svelte';
    import Editor from '$lib/components/Editor.svelte';
    import Viewer from '$lib/components/Viewer.svelte';
    import Settings from '$lib/components/Settings.svelte';
    import { fetchNotes, fetchNote } from '$lib/api';
    import type { Note, NoteListResponse } from '$lib/types';

    let notes = $state<NoteListResponse>({ files: [], directories: {} });
    let selectedNote = $state<Note | null>(null);
    let viewMode = $state<'empty' | 'viewer' | 'editor' | 'new'>('empty');
    let currentDir = $state(''); 
    let showSettings = $state(false);

    async function loadNotes() {
        notes = await fetchNotes();
    }

    async function handleSelectNote(path: string) {
        const fullNote = await fetchNote(path);
        selectedNote = fullNote;
        viewMode = 'viewer';
    }

    function handleNewNote(dir = '') {
        selectedNote = { title: '', content: '', directory: dir };
        currentDir = dir;
        viewMode = 'new';
    }

    function handleEditNote() {
        viewMode = 'editor';
    }

    function handleSaveSuccess() {
        loadNotes();
        viewMode = 'empty';
        selectedNote = null;
    }

    onMount(() => {
        loadNotes();
    });
</script>

<div class="main-layout">
    <Sidebar 
        {notes} 
        onSelectNote={handleSelectNote} 
        onNewNote={handleNewNote} 
        onRefresh={loadNotes} 
        onToggleSettings={() => showSettings = !showSettings}
    />
    
    {#if showSettings}
        <Settings onClose={() => showSettings = false} />
    {/if}
    
    <main class="content-area">
        {#if viewMode === 'empty'}
            <div class="empty-state fade-in">
                <div class="empty-icon">📝</div>
                <h2>Select a note to read</h2>
                <p>Or create a new one to start capturing your thoughts.</p>
                <button class="primary-btn" onclick={() => handleNewNote()}>Create Note</button>
            </div>
        {:else if viewMode === 'viewer'}
            <Viewer note={selectedNote} onEdit={handleEditNote} />
        {:else if viewMode === 'editor' || viewMode === 'new'}
            {#key selectedNote}
                <Editor 
                    note={selectedNote} 
                    isNew={viewMode === 'new'} 
                    onSave={handleSaveSuccess} 
                    onCancel={() => viewMode = viewMode === 'new' ? 'empty' : 'viewer'}
                />
            {/key}
        {/if}
    </main>
</div>

<style>
    .main-layout {
        display: flex;
        width: 100%;
        height: 100%;
    }

    .content-area {
        flex: 1;
        overflow: hidden;
        background: radial-gradient(circle at top right, rgba(59, 130, 246, 0.05), transparent);
    }

    .empty-state {
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        text-align: center;
        color: var(--text-dim);
    }

    .empty-icon {
        font-size: 4rem;
        margin-bottom: 20px;
        opacity: 0.5;
    }

    .primary-btn {
        margin-top: 24px;
        background: var(--accent);
        color: white;
        padding: 12px 24px;
        border-radius: 8px;
        font-weight: 600;
        box-shadow: 0 4px 14px rgba(59, 130, 246, 0.2);
    }

    .primary-btn:hover {
        background: var(--accent-hover);
        transform: translateY(-2px);
    }
</style>
