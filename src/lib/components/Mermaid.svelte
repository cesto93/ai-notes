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
        background: rgba(15, 23, 42, 0.03);
        padding: 30px;
        border-radius: 12px;
        overflow-x: auto;
        border: 1px solid var(--paper-border);

        --pele-bg: transparent;
        --pele-fg: #1e293b;
        --pele-muted: #64748b;
        --pele-line: #94a3b8;
        --pele-surface: rgba(226, 232, 240, 0.6);
        --pele-surface-alt: rgba(203, 213, 225, 0.7);
        --pele-border: #94a3b8;
        --pele-accent: #2563eb;
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
