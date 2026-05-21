<script lang="ts">
  import FileTree from '$lib/components/FileTree.svelte';
  import DiffViewer from '$lib/components/DiffViewer.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import Typeahead from '$lib/components/Typeahead.svelte';
  import { diffStore } from '$lib/stores/diff.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { open } from '@tauri-apps/plugin-dialog';

  let showSettings = $state(false);
  let showCostWarning = $state(false);
  let sidebarWidth = $state(280);
  let repoInputTimer: ReturnType<typeof setTimeout> | null = null;

  function handleSummarizeAll() {
    const fileCount = diffStore.diffResult?.files.length || 0;
    if (fileCount > 10) {
      showCostWarning = true;
    } else {
      diffStore.summarizeAll();
    }
  }

  function confirmSummarizeAll() {
    showCostWarning = false;
    diffStore.summarizeAll();
  }

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
  }

  function handleRepoInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    diffStore.ghRepo = val;
    // Debounce branch loading
    if (repoInputTimer) clearTimeout(repoInputTimer);
    repoInputTimer = setTimeout(() => {
      if (diffStore.parseRepo(val)) {
        diffStore.loadGitHubBranches();
      }
    }, 600);
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
          class="repo-input"
          type="text"
          placeholder="owner/repo or PR URL..."
          value={diffStore.ghRepo}
          oninput={handleRepoInput}
        />
        <Typeahead
          bind:value={diffStore.ghHead}
          items={diffStore.ghBranches}
          placeholder="branch..."
          label="head"
        />
        <Typeahead
          bind:value={diffStore.ghBase}
          items={diffStore.ghBranches}
          placeholder="trunk"
          label="base"
        />
        {#if diffStore.ghLoadingBranches}
          <span class="loading-branches">...</span>
        {/if}
      {:else}
        <button class="browse-btn" onclick={browsePath}>
          {diffStore.localPath || 'Select repo...'}
        </button>
        <Typeahead
          bind:value={diffStore.localBranch}
          items={diffStore.localBranches}
          placeholder="branch..."
          label="head"
        />
        <Typeahead
          bind:value={diffStore.localBase}
          items={diffStore.localBranches}
          placeholder="trunk"
          label="base"
        />
      {/if}

      <button
        class="fetch-btn"
        onclick={handleFetch}
        disabled={diffStore.isLoading}
      >
        {diffStore.isLoading ? 'Loading...' : 'Load Repo'}
      </button>
    </div>

    <div class="toolbar-right">
      {#if diffStore.diffResult && settingsStore.hasClaudeKey}
        <div class="style-toggle">
          <button
            class:active={diffStore.summaryStyle === 'human'}
            onclick={() => { diffStore.summaryStyle = 'human'; if (diffStore.selectedFile) diffStore.selectFile(diffStore.selectedFile); }}
          >Human</button>
          <button
            class:active={diffStore.summaryStyle === 'caveman'}
            onclick={() => { diffStore.summaryStyle = 'caveman'; if (diffStore.selectedFile) diffStore.selectFile(diffStore.selectedFile); }}
          >Caveman</button>
        </div>
        <button
          class="summarize-btn"
          onclick={handleSummarizeAll}
          disabled={diffStore.isSummarizing}
        >
          {#if diffStore.isSummarizing}
            Summarizing... ({Math.round(diffStore.progress)}%)
          {:else}
            &#9889; Summarize All ({diffStore.diffResult?.files.length})
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

  {#if diffStore.isLoading}
    <!-- Loading state -->
    <div class="loading-screen">
      <div class="loader">
        <div class="loader-ring"></div>
        <p class="loader-text">Fetching diff...</p>
      </div>
    </div>
  {:else if diffStore.diffResult}
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
  {:else}
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

{#if showCostWarning}
  <div class="warning-overlay" onclick={() => showCostWarning = false} role="presentation">
    <div class="warning-dialog" onclick={(e) => e.stopPropagation()} role="alertdialog">
      <div class="warning-icon">&#9888;</div>
      <h3>API Cost Warning</h3>
      <p>You're about to summarize <strong>{diffStore.diffResult?.files.length} files</strong> using the Claude API. Each file costs tokens — this could add up.</p>
      <div class="warning-actions">
        <button class="warning-cancel" onclick={() => showCostWarning = false}>Cancel</button>
        <button class="warning-confirm" onclick={confirmSummarizeAll}>Summarize All</button>
      </div>
    </div>
  </div>
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
  .repo-input {
    width: 180px;
    min-width: 140px;
    padding: 6px 10px;
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-primary);
    font-size: 12px;
    font-family: var(--font-mono);
    flex-shrink: 1;
  }
  .repo-input:focus { outline: none; border-color: var(--color-accent); }
  .loading-branches {
    font-size: 11px;
    color: var(--color-text-muted);
    flex-shrink: 0;
    animation: pulse 1s infinite;
  }
  @keyframes pulse { 50% { opacity: 0.3; } }
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
  .style-toggle {
    display: flex;
    background: var(--color-bg-primary);
    border-radius: 6px;
    overflow: hidden;
    border: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .style-toggle button {
    padding: 4px 10px;
    border: none;
    background: none;
    color: var(--color-text-muted);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .style-toggle button.active {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
    font-weight: 600;
  }
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

  /* Cost warning dialog */
  .warning-overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.6);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .warning-dialog {
    background: var(--color-bg-primary); border: 1px solid var(--color-border);
    border-radius: 12px; padding: 24px; max-width: 380px; text-align: center;
  }
  .warning-icon { font-size: 36px; margin-bottom: 8px; }
  .warning-dialog h3 { font-size: 16px; margin-bottom: 8px; color: var(--color-text-primary); }
  .warning-dialog p { font-size: 13px; color: var(--color-text-secondary); line-height: 1.5; margin-bottom: 20px; }
  .warning-actions { display: flex; gap: 10px; justify-content: center; }
  .warning-cancel {
    padding: 8px 20px; background: var(--color-bg-tertiary); border: 1px solid var(--color-border);
    border-radius: 6px; color: var(--color-text-secondary); font-size: 13px; cursor: pointer;
  }
  .warning-cancel:hover { background: var(--color-bg-hover); }
  .warning-confirm {
    padding: 8px 20px; background: var(--color-accent); border: none;
    border-radius: 6px; color: white; font-size: 13px; font-weight: 600; cursor: pointer;
  }
  .warning-confirm:hover { background: var(--color-accent-hover); }

  /* Loading screen */
  .loading-screen {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .loader {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
  }
  .loader-ring {
    width: 48px;
    height: 48px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: loader-spin 0.9s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  @keyframes loader-spin { to { transform: rotate(360deg); } }
  .loader-text {
    font-size: 13px;
    color: var(--color-text-secondary);
    animation: loader-pulse 1.5s ease-in-out infinite;
  }
  @keyframes loader-pulse { 50% { opacity: 0.4; } }
</style>
