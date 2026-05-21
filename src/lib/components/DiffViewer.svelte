<script lang="ts">
  import { diffStore } from '$lib/stores/diff.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let file = $derived(diffStore.selectedFileDiff);
  let summary = $derived(diffStore.selectedFileSummary);
  let hasSummary = $derived(!!summary);
  let isCurrentlySummarizing = $derived(
    diffStore.isSummarizing && diffStore.summarizingFile === file?.filename
  );

  function statusLabel(status: string): string {
    switch (status) {
      case 'added': return 'New file';
      case 'removed': return 'Deleted';
      case 'modified': return 'Modified';
      case 'renamed': return 'Renamed';
      default: return status;
    }
  }

  function statusColor(status: string): string {
    switch (status) {
      case 'added': return 'var(--color-added-text)';
      case 'removed': return 'var(--color-removed-text)';
      default: return 'var(--color-accent)';
    }
  }

  function parsePatch(patch: string): Array<{ type: 'header' | 'add' | 'remove' | 'context'; text: string }> {
    if (!patch) return [];
    return patch.split('\n').map(line => {
      if (line.startsWith('@@')) return { type: 'header' as const, text: line };
      if (line.startsWith('+')) return { type: 'add' as const, text: line.slice(1) };
      if (line.startsWith('-')) return { type: 'remove' as const, text: line.slice(1) };
      return { type: 'context' as const, text: line.startsWith(' ') ? line.slice(1) : line };
    }).filter(l => l.text !== '' || l.type === 'header');
  }
</script>

{#if file}
  <div class="viewer">
    <div class="file-header">
      <span class="status-badge" style="color: {statusColor(file.status)}">
        {statusLabel(file.status)}
      </span>
      <span class="filename">{file.filename}</span>
      <span class="file-stats">
        <span class="add">+{file.additions}</span>
        <span class="del">-{file.deletions}</span>
      </span>
    </div>

    <!-- AI Summary section -->
    <div class="summary-section">
      {#if hasSummary}
        <div class="summary-content">
          <div class="summary-header">
            <span class="summary-icon">&#9889;</span>
            <span>AI Summary</span>
          </div>
          <div class="summary-text">{@html formatSummary(summary!.summary)}</div>
        </div>
      {:else if isCurrentlySummarizing}
        <div class="summary-loading">
          <span class="spinner"></span>
          Analyzing changes...
        </div>
      {:else if !settingsStore.hasClaudeKey}
        <div class="summary-hint">
          Configure your Claude API key in Settings to get AI summaries
        </div>
      {/if}
    </div>

    <!-- Raw diff -->
    <div class="diff-content">
      <div class="diff-header">Raw Diff</div>
      <div class="diff-lines">
        {#each parsePatch(file.patch) as line}
          <div class="diff-line {line.type}">
            <pre>{line.text}</pre>
          </div>
        {/each}
      </div>
    </div>
  </div>
{:else}
  <div class="empty">
    <p>Select a file from the tree to view changes</p>
  </div>
{/if}

<script lang="ts" module>
  function formatSummary(text: string): string {
    return text
      .replace(/^- /gm, '<li>')
      .replace(/\n<li>/g, '</li>\n<li>')
      .replace(/^<li>/, '<ul><li>')
      .replace(/<\/li>$/, '</li></ul>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/`(.+?)`/g, '<code>$1</code>');
  }
</script>

<style>
  .viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }
  .file-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    background: var(--color-bg-tertiary);
    border-bottom: 1px solid var(--color-border);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .status-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
    background: var(--color-bg-hover);
  }
  .filename {
    font-family: var(--font-mono);
    font-size: 13px;
    flex: 1;
  }
  .file-stats { font-size: 12px; }
  .add { color: var(--color-added-text); margin-right: 6px; }
  .del { color: var(--color-removed-text); }

  .summary-section {
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }
  .summary-content {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 12px 16px;
  }
  .summary-header {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
    font-size: 13px;
    margin-bottom: 8px;
    color: var(--color-accent);
  }
  .summary-icon { font-size: 16px; }
  .summary-text {
    font-size: 13px;
    line-height: 1.6;
    color: var(--color-text-primary);
    user-select: text;
  }
  .summary-text :global(ul) { padding-left: 20px; margin: 4px 0; }
  .summary-text :global(li) { margin: 4px 0; }
  .summary-text :global(code) {
    background: var(--color-bg-hover);
    padding: 1px 5px;
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .summary-loading {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--color-text-secondary);
    font-size: 12px;
  }
  .summary-hint {
    color: var(--color-text-muted);
    font-size: 12px;
    font-style: italic;
  }
  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .diff-content { flex: 1; }
  .diff-header {
    padding: 6px 16px;
    font-size: 11px;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }
  .diff-lines { font-family: var(--font-mono); font-size: 12px; }
  .diff-line {
    padding: 0 16px;
    user-select: text;
  }
  .diff-line pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .diff-line.header {
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
    padding: 6px 16px;
    margin-top: 4px;
    font-style: italic;
  }
  .diff-line.add { background: var(--color-added); color: var(--color-added-text); }
  .diff-line.remove { background: var(--color-removed); color: var(--color-removed-text); }
  .diff-line.context { color: var(--color-unchanged-text); }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    font-size: 14px;
  }
</style>
