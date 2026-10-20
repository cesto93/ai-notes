<script lang="ts">
    import { Plus, ChevronRight, ChevronDown, Trash2, Settings as SettingsIcon, Edit3, RotateCw, FolderPlus, Search, X } from 'lucide-svelte';
    import { createDirectory, deleteNote, deleteDirectory, moveNote, renameNote, renameDirectory, refreshNotes, searchNotes } from '#lib/api.js';
    import type { NoteListResponse, SearchResult } from '#lib/types.js';

    let { 
        notes, 
        onSelectNote, 
        onNewNote, 
        onRefresh, 
        onToggleSettings 
    }: {
        notes: NoteListResponse,
        onSelectNote: (path: string) => void,
        onNewNote: (dir?: string) => void,
        onRefresh: () => void,
        onToggleSettings: () => void
    } = $props();
    
    let expandedDirs = $state(new Set());

    function toggleDir(dir: string) {
        if (expandedDirs.has(dir)) expandedDirs.delete(dir);
        else expandedDirs.add(dir);
        expandedDirs = new Set(expandedDirs); // trigger reactivity
    }

    async function handleRefresh() {
        await refreshNotes();
        onRefresh();
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

    let menu = $state<{ x: number, y: number, type: 'note' | 'dir' | 'root', target: any } | null>(null);

    function handleContextMenu(e: MouseEvent, type: 'note' | 'dir' | 'root', target: any = null) {
        e.preventDefault();
        e.stopPropagation();
        menu = { x: e.clientX, y: e.clientY, type, target };
    }

    async function handleRename() {
        if (!menu) return;
        const { type, target } = menu;
        const oldName = type === 'dir' ? target : target.noteFile.replace('.md', '').replace(/_/g, ' ');
        const newName = prompt(`Rename ${type === 'dir' ? 'directory' : 'note'}:`, oldName);
        
        if (newName && newName !== oldName) {
            if (type === 'dir') {
                await renameDirectory(target, newName);
            } else {
                const oldTitle = target.noteFile.replace('.md', '');
                await renameNote(oldTitle, target.directory, newName);
            }
            onRefresh();
        }
        menu = null;
    }

    async function handleDeleteFromMenu() {
        if (!menu) return;
        const { type, target } = menu;
        if (type === 'dir') {
            if (confirm(`Are you sure you want to delete the directory "${target}" and all its contents?`)) {
                await deleteDirectory(target);
                onRefresh();
            }
        } else {
            const title = target.noteFile.replace('.md', '');
            const displayName = title.replace(/_/g, ' ');
            if (confirm(`Are you sure you want to delete the note "${displayName}"?`)) {
                await deleteNote(title, target.directory);
                onRefresh();
            }
        }
        menu = null;
    }

    async function handleNewSubdir() {
        if (!menu || menu.type !== 'dir') return;
        const parentDir = menu.target;
        const name = prompt(`New subdirectory name in "${parentDir}":`);
        if (name && name.trim()) {
            const trimmedName = name.trim();
            const fullPath = parentDir ? `${parentDir}/${trimmedName}` : trimmedName;
            await createDirectory(fullPath);
            if (parentDir && !expandedDirs.has(parentDir)) {
                toggleDir(parentDir);
            }
            onRefresh();
        }
        menu = null;
    }

    async function handleNewRootDir() {
        menu = null;
        const name = prompt('New directory name:');
        if (name && name.trim()) {
            await createDirectory(name.trim());
            onRefresh();
        }
    }

    interface TreeNode {
        files: string[];
        children: Record<string, TreeNode>;
        fullPath: string;
    }

    function buildTree(directories: Record<string, string[]>): TreeNode {
        const root: TreeNode = { files: [], children: {}, fullPath: "" };
        
        for (const [path, files] of Object.entries(directories)) {
            const parts = path.split('/');
            let current = root;
            let currentPath = "";
            
            for (const part of parts) {
                currentPath = currentPath ? `${currentPath}/${part}` : part;
                if (!current.children[part]) {
                    current.children[part] = { files: [], children: {}, fullPath: currentPath };
                }
                current = current.children[part];
            }
            current.files = files;
        }
        return root;
    }

    let directoryTree = $derived(buildTree(notes.directories));

    // --- search ---
    let searchQuery = $state('');
    let searchResults = $state<SearchResult[]>([]);
    let searching = $state(false);
    let searchActive = $derived(searchQuery.trim().length > 0);

    let searchSeq = 0;
    let searchTimer: ReturnType<typeof setTimeout> | undefined;

    /** Debounced: wait for typing to settle before hitting the backend. */
    function handleSearchInput() {
        clearTimeout(searchTimer);
        const q = searchQuery.trim();
        if (!q) {
            clearSearch();
            return;
        }
        searchTimer = setTimeout(() => runSearch(q), 200);
    }

    async function runSearch(q: string) {
        const seq = ++searchSeq;
        searching = true;
        try {
            const results = await searchNotes(q);
            // Ignore stale responses so an older query can't overwrite a newer one.
            if (seq !== searchSeq) return;
            searchResults = results;
        } catch (err) {
            if (seq === searchSeq) searchResults = [];
        } finally {
            if (seq === searchSeq) searching = false;
        }
    }

    function clearSearch() {
        clearTimeout(searchTimer);
        searchSeq++; // invalidate any in-flight search
        searchQuery = '';
        searchResults = [];
        searching = false;
    }
</script>

{#snippet directoryNode(name: string, node: TreeNode, depth: number)}
    <div class="dir-item" style="margin-left: {depth * 12}px">
        <div 
            class="dir-toggle {dragOverDir === node.fullPath ? 'drag-over' : ''}" 
            onclick={() => toggleDir(node.fullPath)} 
            onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleDir(node.fullPath)}
            role="button" 
            tabindex="0"
            ondragover={(e) => handleDragOver(e, node.fullPath)}
            ondragleave={handleDragLeave}
            ondrop={(e) => handleDrop(e, node.fullPath)}
            oncontextmenu={(e) => handleContextMenu(e, 'dir', node.fullPath)}
        >
            {#if expandedDirs.has(node.fullPath)}
                <ChevronDown size={16} />
            {:else}
                <ChevronRight size={16} />
            {/if}
            <span class="dir-name">{name}</span>
        </div>
        
        {#if expandedDirs.has(node.fullPath)}
            <div class="dir-children fade-in">
                {#each Object.entries(node.children) as [childName, childNode]}
                    {@render directoryNode(childName, childNode, depth + 1)}
                {/each}

                {#each node.files as noteFile}
                    <div 
                        class="file-item" 
                        onclick={() => onSelectNote(`${node.fullPath}/${noteFile}`)} 
                        onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onSelectNote(`${node.fullPath}/${noteFile}`)}
                        role="button" 
                        tabindex="0"
                        draggable="true"
                        ondragstart={(e) => handleDragStart(e, noteFile.replace('.md', ''), node.fullPath)}
                        oncontextmenu={(e) => handleContextMenu(e, 'note', { noteFile, directory: node.fullPath })}
                    >
                        <span class="file-name">{noteFile.replace('.md', '').replace(/_/g, ' ')}</span>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
{/snippet}

<svelte:window 
    onclick={() => menu = null} 
    oncontextmenu={(e) => { if (menu) menu = null; }} 
/>

<aside class="sidebar glass-morphism">
    <div class="sidebar-header">
        <h2>AI Notes</h2>
        <div class="header-actions">
            <button class="icon-btn settings-btn" onclick={() => onToggleSettings()} title="Settings">
                <SettingsIcon size={18} />
            </button>
            <button class="icon-btn refresh-btn" onclick={handleRefresh} title="Refresh Notes">
                <RotateCw size={18} />
            </button>
        </div>
    </div>

    <div class="search-box">
        <Search size={16} />
        <input
            type="text"
            placeholder="Search notes…"
            bind:value={searchQuery}
            oninput={handleSearchInput}
            aria-label="Search notes"
        />
        {#if searchQuery}
            <button class="search-clear" onclick={clearSearch} title="Clear search">
                <X size={14} />
            </button>
        {/if}
    </div>

    <div class="sidebar-content" role="tree" aria-label="Notes" tabindex="-1" oncontextmenu={(e) => handleContextMenu(e, 'root')}>
        {#if searchActive}
            <div class="search-results" role="list" aria-label="Search results">
                {#if searchResults.length === 0}
                    <div class="search-empty">
                        {searching ? 'Searching…' : `No matching notes`}
                    </div>
                {:else}
                    {#each searchResults as result (result.path)}
                        <div
                            class="search-result"
                            role="button"
                            tabindex="0"
                            onclick={() => onSelectNote(result.path)}
                            onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onSelectNote(result.path)}
                        >
                            <div class="result-title">{result.title}</div>
                            {#if result.directory}
                                <div class="result-dir">{result.directory}</div>
                            {/if}
                            {#if result.tags.length > 0}
                                <div class="result-tags">
                                    {#each result.tags as tag}
                                        <span class="tag-pill">#{tag}</span>
                                    {/each}
                                </div>
                            {/if}
                            {#if result.match_content && result.snippet}
                                <div class="result-snippet">{result.snippet}</div>
                            {/if}
                        </div>
                    {/each}
                {/if}
            </div>
        {:else}
            <div class="dir-list">
                {#each Object.entries(directoryTree.children) as [name, node]}
                    {@render directoryNode(name, node, 0)}
                {/each}
            </div>

            {#if notes.files.length > 0}
                <div class="root-files-section">
                    <div 
                        class="file-list {dragOverDir === '' ? 'drag-over' : ''}"
                        role="region"
                        aria-label="Notes in root directory"
                        ondragover={(e) => handleDragOver(e, "")}
                        ondragleave={handleDragLeave}
                        ondrop={(e) => handleDrop(e, "")}
                    >
                        {#each notes.files as noteFile}
                            <div 
                                class="file-item" 
                                onclick={() => onSelectNote(noteFile)} 
                                onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onSelectNote(noteFile)}
                                role="button" 
                                tabindex="0"
                                draggable="true"
                                ondragstart={(e) => handleDragStart(e, noteFile.replace('.md', ''), "")}
                                oncontextmenu={(e) => handleContextMenu(e, 'note', { noteFile, directory: "" })}
                            >
                                <span class="file-name">{noteFile.replace('.md', '').replace(/_/g, ' ')}</span>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        {/if}
    </div>

</aside>

{#if menu}
    <div 
        role="menu"
        tabindex="-1"
        class="context-menu glass-morphism fade-in" 
        style="top: {menu.y}px; left: {menu.x}px"
        oncontextmenu={(e) => { e.preventDefault(); e.stopPropagation(); }}
    >
        {#if menu.type === 'dir'}
            <button class="menu-item" onclick={() => { if (menu) onNewNote(menu.target); menu = null; }}>
                <Plus size={16} class="icon-gap" />
                New Note
            </button>
            <button class="menu-item" onclick={handleNewSubdir}>
                <FolderPlus size={16} class="icon-gap" />
                New Subdirectory
            </button>
            <div class="menu-divider"></div>
        {:else if menu.type === 'root'}
            <button class="menu-item" onclick={() => { onNewNote(''); menu = null; }}>
                <Plus size={16} class="icon-gap" />
                New Note
            </button>
            <button class="menu-item" onclick={handleNewRootDir}>
                <FolderPlus size={16} class="icon-gap" />
                New Directory
            </button>
        {:else}
            <button class="menu-item" onclick={handleRename}>
                <Edit3 size={16} class="icon-gap" />
                Rename
            </button>
            <div class="menu-divider"></div>
            <button class="menu-item delete" onclick={handleDeleteFromMenu}>
                <Trash2 size={16} class="icon-gap" />
                Delete
            </button>
        {/if}
    </div>
{/if}

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

    .refresh-btn {
        background: var(--bg-secondary) !important;
        color: var(--text-dim) !important;
        box-shadow: none !important;
    }

    .refresh-btn:hover {
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

    :global(.icon-gap) {
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

    .file-name {
        flex: 1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .dir-toggle.drag-over,
    .file-list.drag-over {
        background: var(--glass);
        outline: 2px dashed var(--accent);
        outline-offset: -2px;
    }

    .context-menu {
        position: fixed;
        z-index: 1000;
        min-width: 160px;
        background: rgba(15, 23, 42, 0.95);
        backdrop-filter: blur(12px);
        border: 1px solid var(--border);
        border-radius: 12px;
        padding: 6px;
        box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
    }

    .menu-item {
        width: 100%;
        display: flex;
        align-items: center;
        padding: 8px 12px;
        border-radius: 8px;
        font-size: 0.9rem;
        color: var(--text-main);
        cursor: pointer;
        transition: all 0.2s;
    }

    .menu-item:hover {
        background: var(--glass);
    }

    .menu-item.delete {
        color: #ef4444;
    }

    .menu-item.delete:hover {
        background: rgba(239, 68, 68, 0.1);
    }

    .menu-divider {
        height: 1px;
        background: var(--border);
        margin: 4px;
    }

    .search-box {
        margin: 0 24px 12px;
        display: flex;
        align-items: center;
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        border-radius: 10px;
        padding: 0 8px;
        transition: all 0.2s;
    }

    .search-box:focus-within {
        border-color: var(--accent);
        background: rgba(255, 255, 255, 0.06);
    }

    .search-box :global(svg) {
        color: var(--text-dim);
        flex-shrink: 0;
    }

    .search-box input {
        flex: 1;
        min-width: 0;
        background: transparent;
        border: none;
        padding: 8px;
        font-size: 0.9rem;
        color: var(--text-main);
    }

    .search-box input:focus {
        border: none;
        background: transparent;
    }

    .search-clear {
        display: flex;
        align-items: center;
        padding: 4px;
        border-radius: 6px;
        color: var(--text-dim);
        flex-shrink: 0;
    }

    .search-clear:hover {
        color: white;
        background: var(--glass);
    }

    .search-results {
        padding: 0 4px;
    }

    .search-empty {
        padding: 16px 12px;
        color: var(--text-dim);
        font-size: 0.9rem;
        text-align: center;
    }

    .search-result {
        padding: 10px 12px;
        border-radius: 8px;
        margin-top: 4px;
        cursor: pointer;
        user-select: none;
    }

    .search-result:hover {
        background: var(--glass);
    }

    .result-title {
        color: var(--text-main);
        font-size: 0.9rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .result-dir {
        color: var(--text-dim);
        font-size: 0.75rem;
        margin-top: 2px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .result-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin-top: 6px;
    }

    .tag-pill {
        padding: 1px 6px;
        border-radius: 6px;
        background: var(--bg-secondary);
        border: 1px solid var(--border);
        color: var(--text-dim);
        font-size: 0.7rem;
    }

    .result-snippet {
        margin-top: 4px;
        color: var(--text-dim);
        font-size: 0.8rem;
        line-height: 1.3;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }
</style>
