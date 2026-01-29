<script lang="ts">
    import { FileText, Folder, Plus, ChevronRight, ChevronDown, List as ListIcon, Trash2, Settings } from 'lucide-svelte';
    import { createDirectory, deleteNote, deleteDirectory, moveNote } from '$lib/api';

    let { notes, onSelectNote, onNewNote, onRefresh, onToggleSettings } = $props();
    
    let newDirName = $state('');
    let showDirInput = $state(false);
    let expandedDirs = $state(new Set());

    function toggleDir(dir: string) {
        if (expandedDirs.has(dir)) expandedDirs.delete(dir);
        else expandedDirs.add(dir);
        expandedDirs = new Set(expandedDirs); // trigger reactivity
    }

    async function handleCreateDir() {
        if (!newDirName.trim()) return;
        await createDirectory(newDirName);
        newDirName = '';
        showDirInput = false;
        onRefresh();
    }

    async function handleDeleteNote(e: MouseEvent, noteFile: string, directory: string = "") {
        e.stopPropagation();
        const title = noteFile.replace('.md', '');
        const displayName = title.replace(/_/g, ' ');
        if (confirm(`Are you sure you want to delete the note "${displayName}"?`)) {
            await deleteNote(title, directory);
            onRefresh();
        }
    }

    async function handleDeleteDir(e: MouseEvent, dir: string) {
        e.stopPropagation();
        if (confirm(`Are you sure you want to delete the directory "${dir}" and all its contents?`)) {
            await deleteDirectory(dir);
            onRefresh();
        }
    }

    let draggedNote = $state<{ title: string, directory: string } | null>(null);
    let dragOverDir = $state<string | null>(null);

    function handleDragStart(e: DragEvent, title: string, directory: string) {
        draggedNote = { title, directory };
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = 'move';
            e.dataTransfer.setData('text/plain', JSON.stringify({ title, directory }));
        }
    }

    async function handleDrop(e: DragEvent, targetDirectory: string) {
        e.preventDefault();
        dragOverDir = null;
        if (!draggedNote) {
            // Fallback for cross-window or other drag sources
            const data = e.dataTransfer?.getData('text/plain');
            if (data) {
                try {
                    draggedNote = JSON.parse(data);
                } catch (err) {
                    return;
                }
            }
        }

        if (draggedNote && draggedNote.directory !== targetDirectory) {
            await moveNote(draggedNote.title, draggedNote.directory, targetDirectory);
            onRefresh();
        }
        draggedNote = null;
    }

    function handleDragOver(e: DragEvent, dir: string | null = null) {
        e.preventDefault();
        dragOverDir = dir;
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = 'move';
        }
    }

    function handleDragLeave() {
        dragOverDir = null;
    }
</script>

<aside class="sidebar glass-morphism">
    <div class="sidebar-header">
        <h2>AI Notes</h2>
        <div class="header-actions">
            <button class="icon-btn settings-btn" onclick={onToggleSettings} title="Settings">
                <Settings size={18} />
            </button>
            <button class="icon-btn" onclick={onNewNote} title="New Note">
                <Plus size={20} />
            </button>
        </div>
    </div>

    <div class="sidebar-content">
        <div class="section">
            <div class="section-header">
                <span class="section-title">Directories</span>
                <button class="mini-btn" onclick={() => showDirInput = !showDirInput}>
                    <Plus size={14} />
                </button>
            </div>
            
            {#if showDirInput}
                <div class="dir-input fade-in">
                    <input 
                        bind:value={newDirName} 
                        placeholder="Name..." 
                        onkeydown={(e) => e.key === 'Enter' && handleCreateDir()}
                    />
                </div>
            {/if}

            <div class="dir-list">
                {#each Object.entries(notes.directories) as [dir, dirNotes]}
                    <div class="dir-item">
                        <div 
                            class="dir-toggle {dragOverDir === dir ? 'drag-over' : ''}" 
                            onclick={() => toggleDir(dir)} 
                            role="button" 
                            tabindex="0"
                            ondragover={(e) => handleDragOver(e, dir)}
                            ondragleave={handleDragLeave}
                            ondrop={(e) => handleDrop(e, dir)}
                        >
                            {#if expandedDirs.has(dir)}
                                <ChevronDown size={16} />
                            {:else}
                                <ChevronRight size={16} />
                            {/if}
                            <Folder size={16} class="icon-gap" />
                            <span class="dir-name">{dir}</span>
                            <button class="delete-btn" onclick={(e) => handleDeleteDir(e, dir)} title="Delete Directory">
                                <Trash2 size={14} />
                            </button>
                        </div>
                        
                        {#if expandedDirs.has(dir)}
                            <div class="dir-children fade-in">
                                <button class="file-item new-in-dir" onclick={() => onNewNote(dir)}>
                                    <Plus size={14} class="icon-gap" />
                                    <span>New Note</span>
                                </button>
                                {#each dirNotes as noteFile}
                                    <div 
                                        class="file-item" 
                                        onclick={() => onSelectNote(`${dir}/${noteFile}`)} 
                                        role="button" 
                                        tabindex="0"
                                        draggable="true"
                                        ondragstart={(e) => handleDragStart(e, noteFile.replace('.md', ''), dir)}
                                    >
                                        <FileText size={14} class="icon-gap" />
                                        <span class="file-name">{noteFile.replace('.md', '').replace(/_/g, ' ')}</span>
                                        <button class="delete-btn mini" onclick={(e) => handleDeleteNote(e, noteFile, dir)} title="Delete Note">
                                            <Trash2 size={12} />
                                        </button>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/each}
            </div>
        </div>

        <div class="section">
            <div class="section-header">
                <span class="section-title">All Notes</span>
            </div>
            <div 
                class="file-list {dragOverDir === '' ? 'drag-over' : ''}"
                ondragover={(e) => handleDragOver(e, "")}
                ondragleave={handleDragLeave}
                ondrop={(e) => handleDrop(e, "")}
            >
                {#each notes.files as noteFile}
                    <div 
                        class="file-item" 
                        onclick={() => onSelectNote(noteFile)} 
                        role="button" 
                        tabindex="0"
                        draggable="true"
                        ondragstart={(e) => handleDragStart(e, noteFile.replace('.md', ''), "")}
                    >
                        <FileText size={14} class="icon-gap" />
                        <span class="file-name">{noteFile.replace('.md', '').replace(/_/g, ' ')}</span>
                        <button class="delete-btn mini" onclick={(e) => handleDeleteNote(e, noteFile)} title="Delete Note">
                            <Trash2 size={12} />
                        </button>
                    </div>
                {/each}
            </div>
        </div>
    </div>
</aside>

<style>
    .sidebar {
        width: 280px;
        height: 100%;
        display: flex;
        flex-direction: column;
        border-right: 1px solid var(--border);
        background: var(--sidebar-bg);
    }

    .sidebar-header {
        padding: 24px;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .header-actions {
        display: flex;
        gap: 8px;
        align-items: center;
    }

    .settings-btn {
        background: var(--bg-secondary) !important;
        color: var(--text-dim) !important;
        box-shadow: none !important;
    }

    .settings-btn:hover {
        background: var(--glass) !important;
        color: white !important;
    }

    .icon-btn {
        background: var(--accent);
        color: white;
        border-radius: 8px;
        padding: 6px;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
    }

    .icon-btn:hover {
        background: var(--accent-hover);
        transform: scale(1.05);
    }

    .sidebar-content {
        flex: 1;
        overflow-y: auto;
        padding: 0 12px 24px;
    }

    .section {
        margin-bottom: 24px;
    }

    .section-header {
        padding: 8px 12px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
    }

    .section-title {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: var(--text-dim);
        font-weight: 600;
    }

    .mini-btn {
        color: var(--text-dim);
        padding: 2px;
        border-radius: 4px;
    }

    .mini-btn:hover {
        background: var(--glass);
        color: white;
    }

    .dir-input {
        padding: 0 12px 8px;
    }

    .dir-input input {
        width: 100%;
        padding: 6px 12px;
        font-size: 0.9rem;
    }

    .dir-item {
        margin-bottom: 4px;
    }

    .dir-toggle {
        width: 100%;
        display: flex;
        align-items: center;
        padding: 8px 12px;
        border-radius: 8px;
        font-size: 0.95rem;
        color: var(--text-main);
        cursor: pointer;
        user-select: none;
    }

    .dir-toggle:hover {
        background: var(--glass);
    }

    .icon-gap {
        margin-right: 10px;
        flex-shrink: 0;
    }

    .dir-name {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .dir-children {
        margin-left: 20px;
        border-left: 1px solid var(--border);
        padding-left: 8px;
    }

    .file-item {
        width: 100%;
        display: flex;
        align-items: center;
        padding: 8px 12px;
        border-radius: 8px;
        font-size: 0.9rem;
        color: var(--text-dim);
        margin-top: 2px;
        cursor: pointer;
        user-select: none;
    }

    .file-item:hover {
        background: var(--glass);
        color: white;
    }

    .new-in-dir {
        color: var(--accent);
        font-style: italic;
    }

    .file-name {
        flex: 1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .delete-btn {
        opacity: 0;
        color: var(--text-dim);
        padding: 4px;
        border-radius: 4px;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .delete-btn:hover {
        background: rgba(239, 68, 68, 0.1);
        color: #ef4444;
    }

    .dir-toggle:hover .delete-btn,
    .file-item:hover .delete-btn {
        opacity: 1;
    }

    .delete-btn.mini {
        padding: 2px;
    }

    .dir-toggle.drag-over,
    .file-list.drag-over {
        background: var(--glass);
        outline: 2px dashed var(--accent);
        outline-offset: -2px;
    }
</style>
