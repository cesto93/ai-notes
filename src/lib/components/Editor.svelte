<script lang="ts">
    import { Save, X } from 'lucide-svelte';
    import { saveNote, updateNote } from '$lib/api';

    let { note, isNew = false, onSave, onCancel } = $props();
    
    let title = $state('');
    let content = $state('');
    let directory = $state('');
    let loading = $state(false);

    $effect(() => {
        title = note?.title || '';
        content = note?.content || '';
        directory = note?.directory || '';
    });

    async function handleSave() {
        if (!title || !content) return;
        loading = true;
        try {
            if (isNew) {
                await saveNote({ title, content, directory });
            } else {
                await updateNote({
                    old_title: note.title,
                    old_directory: note.directory,
                    new_title: title,
                    new_content: content,
                    new_directory: directory
                });
            }
            onSave();
        } finally {
            loading = false;
        }
    }
</script>

<div class="editor fade-in">
    <header class="editor-header">
        <div class="title-section">
            <h1>{isNew ? 'New Note' : 'Edit Note'}</h1>
            {#if directory}
                <span class="directory-tag">{directory}</span>
            {/if}
        </div>
        <div class="actions">
            <button class="cancel-btn" onclick={onCancel}>
                <X size={20} class="icon-gap" />
                Cancel
            </button>
            <button class="save-btn" onclick={handleSave} disabled={loading}>
                <Save size={20} class="icon-gap" />
                {loading ? 'Saving...' : 'Save Note'}
            </button>
        </div>
    </header>

    <div class="form">
        <div class="input-group">
            <label for="title">Title</label>
            <input id="title" bind:value={title} placeholder="Enter title..." />
        </div>
        
        <div class="input-group full-height">
            <label for="content">Content</label>
            <textarea id="content" bind:value={content} placeholder="Write your thoughts..."></textarea>
        </div>
    </div>
</div>

<style>
    .editor {
        flex: 1;
        display: flex;
        flex-direction: column;
        height: 100%;
        max-width: 900px;
        margin: 0 auto;
        padding: 40px 20px;
    }

    .editor-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 30px;
    }

    .title-section {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .directory-tag {
        align-self: flex-start;
        background: var(--glass);
        padding: 4px 12px;
        border-radius: 20px;
        font-size: 0.8rem;
        color: var(--accent);
        border: 1px solid var(--border);
    }

    .actions {
        display: flex;
        gap: 12px;
    }

    .save-btn {
        background: var(--accent);
        color: white;
        padding: 10px 20px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        font-weight: 600;
    }

    .save-btn:hover:not(:disabled) {
        background: var(--accent-hover);
        transform: translateY(-2px);
    }

    .cancel-btn {
        color: var(--text-dim);
        padding: 10px 20px;
    }

    .cancel-btn:hover {
        color: white;
    }

    .form {
        display: flex;
        flex-direction: column;
        gap: 20px;
        flex: 1;
    }

    .input-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    label {
        font-size: 0.8rem;
        text-transform: uppercase;
        color: var(--text-dim);
        font-weight: 600;
        letter-spacing: 0.05em;
    }

    .full-height {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    textarea {
        flex: 1;
        resize: none;
        font-size: 1.1rem;
        line-height: 1.6;
    }

    :global(.icon-gap) { margin-right: 8px; }
</style>
