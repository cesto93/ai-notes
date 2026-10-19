<script lang="ts">
    import { mount, type Mounted } from 'pele';

    let { code }: { code: string } = $props();
    let container: HTMLDivElement | undefined = $state();
    let error: string = $state('');
    let mounted: Mounted | undefined = $state();

    $effect(() => {
        // Track `code` explicitly so the chart re-renders when it changes
        const current = code as string;
        if (container && current) {
            error = '';
            mounted = mount(container, current, {
                onError: (e) => {
                    console.error('Pele rendering failed:', e);
                    error = e instanceof Error ? e.message : 'Failed to render diagram';
                },
            });
        }
        return () => {
            mounted?.destroy();
            mounted = undefined;
        };
    });
</script>

<div bind:this={container} class="pele-container"></div>
{#if error}
    <pre class="error">{error}</pre>
{/if}

<style>
    .pele-container {
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

        --pele-bg: transparent;
        --pele-fg: #e2e8f0;
        --pele-muted: #94a3b8;
        --pele-line: #cbd5e1;
        --pele-surface: rgba(30, 41, 59, 0.8);
        --pele-surface-alt: rgba(51, 65, 85, 0.8);
        --pele-border: #475569;
        --pele-accent: #60a5fa;
        --pele-font: sans-serif;
    }

    .pele-container :global(svg) {
        max-width: 100%;
        height: auto;
    }

    .error {
        color: #ef4444;
        font-family: monospace;
        margin: 8px 0 0;
    }
</style>
