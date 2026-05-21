<script lang="ts">
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { invoke } from '@tauri-apps/api/core';

  let { onclose }: { onclose: () => void } = $props();

  let activeTab = $state('api');
  let apiKey = $state(settingsStore.claudeApiKey);
  let showKey = $state(false);
  let saveMessage = $state('');
  let updateInfo = $state<any>(null);
  let checkingUpdate = $state(false);
  let desktopMessage = $state('');

  async function saveApiKey() {
    await settingsStore.setSetting('claude_api_key', apiKey);
    saveMessage = 'Saved!';
    setTimeout(() => saveMessage = '', 2000);
  }

  async function checkUpdates() {
    checkingUpdate = true;
    try {
      updateInfo = await invoke('check_for_updates');
    } catch (e: any) {
      updateInfo = { error: e.toString() };
    } finally {
      checkingUpdate = false;
    }
  }

  async function createDesktopEntry() {
    try {
      const path = await invoke<string>('create_desktop_entry');
      desktopMessage = `Created: ${path}`;
    } catch (e: any) {
      desktopMessage = e.toString();
    }
  }

  const tabs = [
    { id: 'api', label: 'Claude API', icon: '&#9889;' },
    { id: 'general', label: 'General', icon: '&#9881;' },
    { id: 'about', label: 'About', icon: '&#9432;' },
  ];
</script>

<div class="overlay" onclick={onclose} role="presentation">
  <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog">
    <div class="sidebar">
      <div class="sidebar-title">Settings</div>
      {#each tabs as tab}
        <button
          class="tab-btn"
          class:active={activeTab === tab.id}
          onclick={() => activeTab = tab.id}
        >
          <span class="tab-icon">{@html tab.icon}</span>
          {tab.label}
        </button>
      {/each}
    </div>

    <div class="content">
      <button class="close-btn" onclick={onclose}>&times;</button>

      {#if activeTab === 'api'}
        <h2>Claude API Configuration</h2>
        <p class="description">
          LazyDiff uses Claude to generate human-readable summaries of code changes.
          You need an Anthropic API key to use this feature.
        </p>

        <div class="steps">
          <h3>How to get your API key:</h3>
          <ol>
            <li>Go to <strong>console.anthropic.com</strong></li>
            <li>Sign in or create an account</li>
            <li>Navigate to <strong>API Keys</strong> in the sidebar</li>
            <li>Click <strong>Create Key</strong> and copy it</li>
            <li>Paste it below</li>
          </ol>
        </div>

        <div class="field">
          <label for="api-key">API Key</label>
          <div class="input-row">
            <input
              id="api-key"
              type={showKey ? 'text' : 'password'}
              bind:value={apiKey}
              placeholder="sk-ant-..."
            />
            <button class="icon-btn" onclick={() => showKey = !showKey}>
              {showKey ? '&#128065;' : '&#128064;'}
            </button>
          </div>
          <div class="actions">
            <button class="primary-btn" onclick={saveApiKey}>Save</button>
            {#if saveMessage}
              <span class="save-msg">{saveMessage}</span>
            {/if}
          </div>
        </div>

      {:else if activeTab === 'general'}
        <h2>General</h2>
        <div class="field">
          <h3>Desktop Entry (Linux)</h3>
          <p class="description">Create a .desktop entry so LazyDiff appears in your app launcher.</p>
          <button class="primary-btn" onclick={createDesktopEntry}>Create Desktop Entry</button>
          {#if desktopMessage}
            <p class="save-msg" style="margin-top: 8px">{desktopMessage}</p>
          {/if}
        </div>

      {:else if activeTab === 'about'}
        <h2>LazyDiff</h2>
        <p class="description">Human-readable PR diff viewer powered by AI</p>
        <div class="about-info">
          <div class="badge-row">
            <span class="tech-badge">Tauri v2</span>
            <span class="tech-badge">SvelteKit</span>
            <span class="tech-badge">Claude API</span>
          </div>
          <p class="version">Version {__APP_VERSION__}</p>
        </div>
        <div class="field">
          <button class="primary-btn" onclick={checkUpdates} disabled={checkingUpdate}>
            {checkingUpdate ? 'Checking...' : 'Check for Updates'}
          </button>
          {#if updateInfo}
            {#if updateInfo.error}
              <p class="error">{updateInfo.error}</p>
            {:else if updateInfo.has_update}
              <p class="update-available">New version available: v{updateInfo.latest_version}</p>
            {:else}
              <p class="save-msg">You're up to date!</p>
            {/if}
          {/if}
        </div>
        <div class="field">
          <a href="https://github.com/jjolmo/lazydiff" class="link" target="_blank">
            GitHub Repository
          </a>
        </div>
      {/if}
    </div>
  </div>
</div>

<script lang="ts" module>
  declare const __APP_VERSION__: string;
</script>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    display: flex;
    width: 700px;
    height: 500px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    overflow: hidden;
  }
  .sidebar {
    width: 180px;
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    padding: 16px 0;
    flex-shrink: 0;
  }
  .sidebar-title {
    padding: 0 16px 12px;
    font-weight: 600;
    font-size: 14px;
    color: var(--color-text-primary);
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 8px;
  }
  .tab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 16px;
    border: none;
    background: none;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
  }
  .tab-btn:hover { background: var(--color-bg-hover); }
  .tab-btn.active { background: var(--color-bg-selected); color: var(--color-text-primary); }
  .tab-icon { font-size: 14px; }
  .content {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
    position: relative;
  }
  .close-btn {
    position: absolute;
    top: 12px;
    right: 16px;
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 20px;
    cursor: pointer;
  }
  .close-btn:hover { color: var(--color-text-primary); }
  h2 { font-size: 18px; margin-bottom: 8px; }
  h3 { font-size: 14px; margin-bottom: 6px; color: var(--color-text-primary); }
  .description { color: var(--color-text-secondary); font-size: 13px; margin-bottom: 16px; line-height: 1.5; }
  .steps {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 12px 16px;
    margin-bottom: 16px;
  }
  .steps ol { padding-left: 20px; color: var(--color-text-secondary); font-size: 13px; line-height: 1.8; }
  .field { margin-bottom: 16px; }
  label { display: block; font-size: 12px; color: var(--color-text-secondary); margin-bottom: 6px; }
  .input-row { display: flex; gap: 6px; }
  input {
    flex: 1;
    padding: 8px 12px;
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-primary);
    font-size: 13px;
    font-family: var(--font-mono);
  }
  input:focus { outline: none; border-color: var(--color-accent); }
  .icon-btn {
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    cursor: pointer;
    padding: 8px;
    font-size: 14px;
  }
  .actions { display: flex; align-items: center; gap: 10px; margin-top: 10px; }
  .primary-btn {
    padding: 8px 20px;
    background: var(--color-accent);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }
  .primary-btn:hover { background: var(--color-accent-hover); }
  .primary-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .save-msg { color: var(--color-added-text); font-size: 12px; }
  .error { color: var(--color-removed-text); font-size: 12px; margin-top: 8px; }
  .update-available { color: var(--color-accent); font-size: 13px; margin-top: 8px; }
  .about-info { margin-bottom: 16px; }
  .badge-row { display: flex; gap: 8px; margin-bottom: 8px; }
  .tech-badge {
    padding: 3px 10px;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-size: 11px;
    color: var(--color-text-secondary);
  }
  .version { font-size: 12px; color: var(--color-text-muted); }
  .link { color: var(--color-accent); font-size: 13px; text-decoration: none; }
  .link:hover { text-decoration: underline; }
</style>
