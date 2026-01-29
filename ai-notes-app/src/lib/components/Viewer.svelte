<script lang="ts">
    import { Edit, Sparkles, Wand2 } from 'lucide-svelte';
    import { summarize, paraphrase } from '$lib/api';

    let { note, onEdit } = $props();
    
    let aiResult = $state('');
    let loading = $state(false);

    async function handleSummarize() {
        loading = true;
        aiResult = '';
        try {
            const data = await summarize(note.content);
            aiResult = data.result;
        } finally {
            loading = false;
        }
    }

    async function handleParaphrase() {
        loading = true;
        aiResult = '';
        try {
            const data = await paraphrase(note.content);
            aiResult = data.result;
        } finally {
            loading = false;
        }
    }
</script>

<div class="viewer fade-in">
    <header class="viewer-header">
        <div class="title-section">
            <h1>{note.title}</h1>
            {#if note.directory}
                <span class="directory-tag">{note.directory}</span>
            {/if}
        </div>
        
        <div class="actions">
            <button class="action-btn summarize" onclick={handleSummarize} disabled={loading}>
                <Sparkles size={18} class="icon-gap" />
                Summarize
            </button>
            <button class="action-btn paraphrase" onclick={handleParaphrase} disabled={loading}>
                <Wand2 size={18} class="icon-gap" />
                Paraphrase
            </button>
            <button class="edit-btn" onclick={onEdit}>
                <Edit size={18} />
            </button>
        </div>
    </header>

    <div class="content-scroll">
        {#if aiResult}
            <div class="ai-box fade-in">
                <div class="ai-header">
                    <Sparkles size={14} />
                    <span>AI Insight</span>
                    <button class="close-ai" onclick={() => aiResult = ''}>&times;</button>
                </div>
                <p>{aiResult}</p>
            </div>
        {/if}

        {#if loading}
            <div class="loader">
                <div class="spinner"></div>
                <span>Thinking...</span>
            </div>
        {/if}

        <article class="note-body">
            {note.content}
        </article>
    </div>
</div>

<style>
    .viewer {
        flex: 1;
        display: flex;
        flex-direction: column;
        height: 100%;
        max-width: 900px;
        margin: 0 auto;
        padding: 40px 20px;
    }

    .viewer-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 40px;
    }

    h1 {
        font-size: 2.5rem;
        margin-bottom: 8px;
    }

    .directory-tag {
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
        align-items: center;
    }

    .action-btn {
        display: flex;
        align-items: center;
        padding: 8px 16px;
        border-radius: 8px;
        font-size: 0.9rem;
        font-weight: 500;
        border: 1px solid var(--border);
    }

    .action-btn:hover:not(:disabled) {
        background: var(--glass);
        transform: translateY(-2px);
    }

    .action-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .summarize { color: #f59e0b; }
    .paraphrase { color: #10b981; }

    .edit-btn {
        padding: 10px;
        border-radius: 50%;
        background: var(--glass);
        color: var(--text-dim);
    }

    .edit-btn:hover {
        background: var(--accent);
        color: white;
    }

    .content-scroll {
        flex: 1;
        overflow-y: auto;
        padding-right: 10px;
    }

    .note-body {
        font-size: 1.1rem;
        white-space: pre-wrap;
        color: var(--text-main);
    }

    .ai-box {
        background: rgba(59, 130, 246, 0.1);
        border-left: 4px solid var(--accent);
        padding: 16px;
        border-radius: 8px;
        margin-bottom: 30px;
        position: relative;
    }

    .ai-header {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 0.8rem;
        text-transform: uppercase;
        color: var(--accent);
        margin-bottom: 8px;
    }

    .close-ai {
        position: absolute;
        top: 8px;
        right: 12px;
        font-size: 1.2rem;
        color: var(--text-dim);
    }

    .loader {
        display: flex;
        align-items: center;
        gap: 12px;
        color: var(--accent);
        margin-bottom: 20px;
    }

    .spinner {
        width: 20px;
        height: 20px;
        border: 2px solid var(--glass);
        border-top-color: var(--accent);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    .icon-gap { margin-right: 8px; }
</style>
