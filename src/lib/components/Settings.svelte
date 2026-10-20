<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchSettings, updateSettings } from '#lib/api.js';

    let { onClose }: { onClose: () => void } = $props();

    let settings = $state({ provider: 'google', model: 'gemini-2.0-flash' });
    let saving = $state(false);

    const providers = [
        { id: 'google', name: 'Google (Gemini)' },
        { id: 'openrouter', name: 'OpenRouter' },
        { id: 'ollama', name: 'Ollama (Local)' },
        { id: 'llamacpp', name: 'llama.cpp (Local)' }
    ];

    const models: Record<string, string[]> = {
        google: ['gemini-2.5-flash', 'gemini-2.5-flash-lite', 'gemini-3-flash-preview', 'gemini-3.1-pro-preview'],
        openrouter: ['openai/gpt-4o-mini', 'anthropic/claude-3.5-sonnet', 'deepseek/deepseek-chat', 'meta-llama/llama-3.3-70b-instruct', 'google/gemini-2.0-flash-001'],
        ollama: ['ministral', 'granite4:tiny-h', 'olmo-3:7b-instruct'],
        llamacpp: ['llama-3.3-70b-instruct']
    };

    onMount(async () => {
        const current = await fetchSettings();
        // Reset to defaults if the saved provider is no longer supported
        if (!current.provider || !models[current.provider]) {
            settings.provider = 'google';
            settings.model = models.google[0];
            return;
        }
        settings.provider = current.provider;
        settings.model = current.model || models[current.provider][0];
    });

    async function handleSave() {
        saving = true;
        try {
            await updateSettings(settings);
            onClose();
        } finally {
            saving = false;
        }
    }
</script>

<div class="settings-modal fade-in">
    <div class="settings-content">
        <header>
            <h2>LLM Configuration</h2>
            <button class="close-btn" onclick={onClose}>✕</button>
        </header>

        <div class="form-group">
            <label for="provider">Provider</label>
            <select id="provider" bind:value={settings.provider} onchange={() => settings.model = models[settings.provider][0]}>
                {#each providers as provider}
                    <option value={provider.id}>{provider.name}</option>
                {/each}
            </select>
        </div>

        <div class="form-group">
            <label for="model">Model</label>
            {#if settings.provider === 'openrouter' || settings.provider === 'llamacpp'}
                <input
                    type="text"
                    id="model"
                    bind:value={settings.model}
                    placeholder={settings.provider === 'llamacpp' ? 'model name (e.g. llama-3.3-70b-instruct)' : 'vendor/model (e.g. openai/gpt-4o-mini)'}
                />
                {#if settings.provider === 'openrouter'}
                    <p class="hint">Use any OpenRouter model ID (vendor/model format). Ensure OPENROUTER_API_KEY is set in your .env file.</p>
                {:else}
                    <p class="hint">Ensure llama-server is running on localhost:8080 with your model loaded. The model name is informational — llama-server serves the model passed at startup.</p>
                {/if}
            {:else}
                <select id="model" bind:value={settings.model}>
                    {#each models[settings.provider] as model}
                        <option value={model}>{model}</option>
                    {/each}
                </select>
                {#if settings.provider === 'ollama'}
                    <p class="hint">Ensure Ollama is running locally and you have pulled the model.</p>
                {/if}
            {/if}
        </div>

        <div class="actions">
            <button class="cancel-btn" onclick={onClose}>Cancel</button>
            <button class="save-btn" onclick={handleSave} disabled={saving}>
                {saving ? 'Saving...' : 'Save Configuration'}
            </button>
        </div>
    </div>
</div>

<style>
    .settings-modal {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(8px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .settings-content {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: 16px;
        width: 100%;
        max-width: 450px;
        padding: 24px;
        box-shadow: var(--shadow-xl);
    }

    header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 24px;
    }

    h2 {
        margin: 0;
        font-size: 1.5rem;
        color: var(--text-bright);
    }

    .close-btn {
        background: none;
        border: none;
        color: var(--text-dim);
        font-size: 1.2rem;
        cursor: pointer;
    }

    .form-group {
        margin-bottom: 20px;
    }

    label {
        display: block;
        margin-bottom: 8px;
        font-weight: 500;
        color: var(--text-normal);
    }

    select,
    input[type='text'] {
        width: 100%;
        padding: 10px 12px;
        background: var(--bg-input);
        border: 1px solid var(--border);
        border-radius: 8px;
        color: var(--text-normal);
        font-size: 1rem;
        outline: none;
    }

    select:focus,
    input[type='text']:focus {
        border-color: var(--accent);
    }

    /* Native dropdowns inherit the OS palette; force readable colors */
    select option {
        background: #1a1a1a;
        color: var(--text-bright);
    }

    .hint {
        font-size: 0.85rem;
        color: var(--text-dim);
        margin-top: 8px;
        font-style: italic;
    }

    .actions {
        display: flex;
        justify-content: flex-end;
        gap: 12px;
        margin-top: 32px;
    }

    button {
        padding: 10px 20px;
        border-radius: 8px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }

    .cancel-btn {
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        color: var(--text-normal);
    }

    .save-btn {
        background: var(--accent);
        border: none;
        color: white;
    }

    .save-btn:hover {
        background: var(--accent-hover);
        transform: translateY(-1px);
    }

    .save-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
