<script lang="ts">
    import mermaid from 'mermaid';

    let { code }: { code: string } = $props();
    let container: HTMLDivElement | undefined = $state();

    mermaid.initialize({
        startOnLoad: false,
        theme: 'dark',
        securityLevel: 'loose',
    });

    async function renderChart(chartCode: string) {
        if (container && chartCode) {
            container.innerHTML = '';
            const id = `mermaid-${Math.random().toString(36).substring(2, 11)}`;
            try {
                const { svg } = await mermaid.render(id, chartCode);
                container.innerHTML = svg;
            } catch (e) {
                console.error('Mermaid rendering failed:', e);
                container.innerHTML = '<pre class="error">Failed to render mindmap</pre>';
            }
        }
    }

    $effect(() => {
        // Track `code` explicitly so the chart re-renders when it changes
        const current = code as string;
        if (current) {
            void renderChart(current);
        }
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

    :global(.error) {
        color: #ef4444;
        font-family: monospace;
    }
</style>
