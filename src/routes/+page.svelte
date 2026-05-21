<script lang="ts">
  import FileTree from '$lib/components/FileTree.svelte';
  import DiffViewer from '$lib/components/DiffViewer.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import { diffStore } from '$lib/stores/diff.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { open } from '@tauri-apps/plugin-dialog';

  let showSettings = $state(false);
  let sidebarWidth = $state(280);

  async function handleFetch() {
    if (diffStore.mode === 'github') {
      await diffStore.fetchGitHubDiff();
    } else {
      await diffStore.fetchLocalDiff();
    }
  }

  async function browsePath() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      diffStore.localPath = selected as string;
      await diffStore.loadBranches();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && showSettings) showSettings = false;
    if (e.key === 'Enter' && !showSettings && diffStore.inputUrl) handleFetch();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app">
  <!-- Toolbar -->
  <div class="toolbar">
    <div class="toolbar-left">
      <span class="app-title">LazyDiff</span>

      <div class="mode-toggle">
        <button
          class:active={diffStore.mode === 'github'}
          onclick={() => diffStore.mode = 'github'}
        >GitHub</button>
        <button
          class:active={diffStore.mode === 'local'}
          onclick={() => diffStore.mode = 'local'}
        >Local</button>
      </div>

      {#if diffStore.mode === 'github'}
        <input
          class="url-input"
          type="text"
          placeholder="Paste GitHub PR or branch URL..."
          bind:value={diffStore.inputUrl}
        />
      {:else}
        <button class="browse-btn" onclick={browsePath}>
          {diffStore.localPath || 'Select repo...'}
        </button>
        {#if diffStore.localBranches.length > 0}
          <select class="branch-select" bind:value={diffStore.localBranch}>
            <option value="">Select branch...</option>
            {#each diffStore.localBranches as branch}
              <option value={branch}>{branch}</option>
            {/each}
          </select>
        {/if}
      {/if}

      <button
        class="fetch-btn"
        onclick={handleFetch}
        disabled={diffStore.isLoading}
      >
        {diffStore.isLoading ? 'Loading...' : 'Fetch'}
      </button>
    </div>

    <div class="toolbar-right">
      {#if diffStore.diffResult && settingsStore.hasClaudeKey}
        <button
          class="summarize-btn"
          onclick={() => diffStore.summarizeAll()}
          disabled={diffStore.isSummarizing}
        >
          {#if diffStore.isSummarizing}
            Summarizing... ({Math.round(diffStore.progress)}%)
          {:else}
            &#9889; Summarize All
          {/if}
        </button>
      {/if}
      <button class="settings-btn" onclick={() => showSettings = true}>&#9881;</button>
    </div>
  </div>

  {#if diffStore.error}
    <div class="error-bar">
      <span>{diffStore.error}</span>
      <button onclick={() => diffStore.error = null}>&times;</button>
    </div>
  {/if}

  {#if diffStore.diffResult}
    <!-- Main content: tree + diff -->
    <div class="main">
      <div class="sidebar" style="width: {sidebarWidth}px">
        <div class="sidebar-header">
          <span class="file-count">{diffStore.diffResult.files.length} files changed</span>
          <span class="diff-stats">
            <span class="add">+{diffStore.diffResult.total_additions}</span>
            <span class="del">-{diffStore.diffResult.total_deletions}</span>
          </span>
        </div>
        <div class="tree-container">
          <FileTree nodes={diffStore.fileTree} />
        </div>
      </div>

      <div class="resize-handle"></div>

      <div class="content">
        <DiffViewer />
      </div>
    </div>
  {:else if !diffStore.isLoading}
    <!-- Landing -->
    <div class="landing">
      <div class="landing-content">
        <h1>LazyDiff</h1>
        <p>Paste a GitHub PR or branch URL, or select a local repo, to get a human-readable summary of code changes.</p>
        <div class="features">
          <div class="feature">
            <span class="feature-icon">&#128270;</span>
            <div>
              <strong>Smart Diff</strong>
              <p>Fetches and parses diffs from GitHub or local git repos</p>
            </div>
          </div>
          <div class="feature">
            <span class="feature-icon">&#9889;</span>
            <div>
              <strong>AI Summaries</strong>
              <p>Claude explains what changed in plain language</p>
            </div>
          </div>
          <div class="feature">
            <span class="feature-icon">&#128230;</span>
            <div>
              <strong>Cross-platform</strong>
              <p>Runs on Windows, macOS, and Linux</p>
            </div>
          </div>
        </div>
        {#if !settingsStore.hasClaudeKey}
          <button class="setup-btn" onclick={() => showSettings = true}>
            &#9881; Set up Claude API Key
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#if showSettings}
  <SettingsPanel onclose={() => showSettings = false} />
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* Toolbar */
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--color-bg-tertiary);
    border-bottom: 1px solid var(--color-border);
    gap: 8px;
    flex-shrink: 0;
  }
  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
  }
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .app-title {
    font-weight: 700;
    font-size: 14px;
    color: var(--color-accent);
    margin-right: 4px;
    flex-shrink: 0;
  }
  .mode-toggle {
    display: flex;
    background: var(--color-bg-primary);
    border-radius: 6px;
    overflow: hidden;
    border: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .mode-toggle button {
    padding: 5px 12px;
    border: none;
    background: none;
    color: var(--color-text-secondary);
    font-size: 12px;
    cursor: pointer;
  }
  .mode-toggle button.active {
    background: var(--color-accent);
    color: white;
  }
  .url-input {
    flex: 1;
    min-width: 200px;
    padding: 6px 12px;
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-primary);
    font-size: 12px;
  }
  .url-input:focus { outline: none; border-color: var(--color-accent); }
  .browse-btn {
    padding: 6px 12px;
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    font-size: 12px;
    cursor: pointer;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .branch-select {
    padding: 6px 8px;
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-primary);
    font-size: 12px;
  }
  .fetch-btn {
    padding: 6px 16px;
    background: var(--color-accent);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    flex-shrink: 0;
  }
  .fetch-btn:hover { background: var(--color-accent-hover); }
  .fetch-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .summarize-btn {
    padding: 6px 14px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-accent);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }
  .summarize-btn:hover { background: var(--color-bg-tertiary); }
  .summarize-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .settings-btn {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 18px;
    cursor: pointer;
    padding: 4px;
  }
  .settings-btn:hover { color: var(--color-text-primary); }

  /* Error */
  .error-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: var(--color-removed);
    color: var(--color-removed-text);
    font-size: 12px;
  }
  .error-bar button {
    background: none;
    border: none;
    color: var(--color-removed-text);
    font-size: 16px;
    cursor: pointer;
  }

  /* Main layout */
  .main {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .sidebar {
    display: flex;
    flex-direction: column;
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    flex-shrink: 0;
    overflow: hidden;
  }
  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    font-size: 11px;
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }
  .diff-stats .add { color: var(--color-added-text); margin-right: 4px; }
  .diff-stats .del { color: var(--color-removed-text); }
  .tree-container { flex: 1; overflow-y: auto; padding: 4px 0; }
  .resize-handle {
    width: 3px;
    cursor: col-resize;
    background: transparent;
    flex-shrink: 0;
  }
  .resize-handle:hover { background: var(--color-accent); }
  .content { flex: 1; overflow: hidden; }

  /* Landing */
  .landing {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .landing-content {
    text-align: center;
    max-width: 500px;
    padding: 40px;
  }
  .landing-content h1 {
    font-size: 36px;
    color: var(--color-accent);
    margin-bottom: 12px;
  }
  .landing-content > p {
    color: var(--color-text-secondary);
    font-size: 14px;
    line-height: 1.6;
    margin-bottom: 32px;
  }
  .features {
    display: flex;
    flex-direction: column;
    gap: 16px;
    text-align: left;
    margin-bottom: 24px;
  }
  .feature {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }
  .feature-icon { font-size: 24px; flex-shrink: 0; }
  .feature strong { font-size: 13px; display: block; margin-bottom: 2px; }
  .feature p { font-size: 12px; color: var(--color-text-secondary); margin: 0; }
  .setup-btn {
    padding: 10px 24px;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
  }
  .setup-btn:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }
</style>
