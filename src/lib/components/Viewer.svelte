<script lang="ts">
    import { Edit, Sparkles, Wand2, Network } from 'lucide-svelte';
    import { summarize, paraphrase, mindmap } from '$lib/api';
    import { marked } from 'marked';
    import Mermaid from './Mermaid.svelte';

    let { note, onEdit } = $props();
    
    let aiResult = $state('');
    let isMindmap = $state(false);
    let loading = $state(false);

    let htmlContent = $derived(marked.parse(note.content || ''));

    async function handleSummarize() {
        loading = true;
        aiResult = '';
        isMindmap = false;
        try {
            const data = await summarize(note.content);
            aiResult = data;
        } catch (e) {
            aiResult = `Error: ${e}`;
        } finally {
            loading = false;
        }
    }

    async function handleParaphrase() {
        loading = true;
        aiResult = '';
        isMindmap = false;
        try {
            const data = await paraphrase(note.content);
            aiResult = data;
        } catch (e) {
            aiResult = `Error: ${e}`;
        } finally {
            loading = false;
        }
    }

    async function handleMindmap() {
        loading = true;
        aiResult = '';
        isMindmap = true;
        try {
            const data = await mindmap(note.content);
            aiResult = data;
        } catch (e) {
            aiResult = `Error: ${e}`;
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
            <button class="action-btn mindmap" onclick={handleMindmap} disabled={loading}>
                <Network size={18} class="icon-gap" />
                Mindmap
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
                <div class="ai-content">
                    {#if isMindmap}
                        <Mermaid code={aiResult} />
                    {:else}
                        {@html marked.parse(aiResult)}
                    {/if}
                </div>
            </div>
        {/if}

        {#if loading}
            <div class="loader">
                <div class="spinner"></div>
                <span>Thinking...</span>
            </div>
        {/if}

        <article class="note-body">
            {@html htmlContent}
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
    .mindmap { color: #8b5cf6; }

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
        color: var(--text-main);
        line-height: 1.6;
    }

    .note-body :global(h1), 
    .note-body :global(h2), 
    .note-body :global(h3) {
        margin-top: 1.5em;
        margin-bottom: 0.5em;
        color: var(--text-main);
    }

    .note-body :global(p) {
        margin-bottom: 1em;
    }

    .note-body :global(ul), 
    .note-body :global(ol) {
        margin-bottom: 1em;
        padding-left: 1.5em;
    }

    .note-body :global(li) {
        margin-bottom: 0.5em;
    }

    .note-body :global(code) {
        background: var(--glass);
        padding: 2px 6px;
        border-radius: 4px;
        font-family: 'Fira Code', monospace;
        font-size: 0.9em;
    }

    .note-body :global(pre) {
        background: #1e293b;
        padding: 16px;
        border-radius: 8px;
        overflow-x: auto;
        margin-bottom: 1em;
    }

    .note-body :global(pre code) {
        background: transparent;
        padding: 0;
        color: #e2e8f0;
    }

    .note-body :global(blockquote) {
        border-left: 4px solid var(--accent);
        padding-left: 16px;
        font-style: italic;
        color: var(--text-dim);
        margin: 1.5em 0;
    }

    .note-body :global(img) {
        max-width: 100%;
        border-radius: 8px;
        margin: 1em 0;
    }

    .note-body :global(table) {
        width: 100%;
        border-collapse: collapse;
        margin-bottom: 1em;
    }

    .note-body :global(th), 
    .note-body :global(td) {
        border: 1px solid var(--border);
        padding: 8px 12px;
        text-align: left;
    }

    .note-body :global(th) {
        background: var(--glass);
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
        margin-bottom: 12px;
    }

    .ai-content {
        font-size: 0.95rem;
        color: var(--text-main);
    }

    .ai-content :global(p) {
        margin-bottom: 0.5em;
    }

    .ai-content :global(p:last-child) {
        margin-bottom: 0;
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
