<script lang="ts">
    import { Save, X } from 'lucide-svelte';
    import { saveNote, updateNote } from '#lib/api.js';

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
            <input 
                type="text" 
                class="title-input" 
                bind:value={title} 
                placeholder="Note Title..."
            />
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
        <div class="input-group full-height">
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
        padding: 40px 48px;
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
        flex: 1;
    }

    .title-input {
        font-size: 2.5rem;
        font-weight: 700;
        background: transparent;
        border: none;
        color: var(--paper-text);
        outline: none;
        padding: 0;
        margin: 0;
        width: 100%;
    }

    .title-input::placeholder {
        color: var(--paper-text-dim);
        opacity: 0.5;
    }

    .directory-tag {
        align-self: flex-start;
        background: rgba(59, 130, 246, 0.1);
        padding: 4px 12px;
        border-radius: 20px;
        font-size: 0.8rem;
        color: var(--accent);
        border: 1px solid rgba(59, 130, 246, 0.25);
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
        color: var(--paper-text-dim);
        padding: 10px 20px;
    }

    .cancel-btn:hover {
        color: var(--paper-text);
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
        background: transparent;
        border: none;
        color: var(--paper-text);
    }

    textarea::placeholder {
        color: var(--paper-text-dim);
    }

    :global(.icon-gap) { margin-right: 8px; }
</style>
