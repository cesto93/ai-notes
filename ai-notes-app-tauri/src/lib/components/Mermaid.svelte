<script lang="ts">
    import { onMount } from 'svelte';
    import mermaid from 'mermaid';

    let { code } = $props();
    let container: HTMLDivElement;

    mermaid.initialize({
        startOnLoad: false,
        theme: 'dark',
        securityLevel: 'loose',
    });

    async function renderChart() {
        if (container && code) {
            container.innerHTML = '';
            const id = `mermaid-${Math.random().toString(36).substr(2, 9)}`;
            try {
                const { svg } = await mermaid.render(id, code);
                container.innerHTML = svg;
            } catch (e) {
                console.error('Mermaid rendering failed:', e);
                container.innerHTML = '<pre class="error">Failed to render mindmap</pre>';
            }
        }
    }

    $effect(() => {
        renderChart();
    });
</script>

<div bind:this={container} class="mermaid-container"></div>

<style>
    .mermaid-container {
        width: 100%;
        min-height: 400px;
        display: flex;
        justify-content: center;
        align-items: center;
        background: rgba(15, 23, 42, 0.5);
        padding: 30px;
        border-radius: 12px;
        overflow-x: auto;
        border: 1px solid var(--border);
    }

    .mermaid-container :global(svg) {
        max-width: 100%;
        height: auto;
    }

    .error {
        color: #ef4444;
        font-family: monospace;
    }
</style>
