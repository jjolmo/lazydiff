<script lang="ts">
  import { diffStore } from '$lib/stores/diff.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let file = $derived(diffStore.selectedFileDiff);
  let summary = $derived(diffStore.selectedFileSummary);
  let hasSummary = $derived(!!summary);
  let isCurrentlySummarizing = $derived(
    diffStore.isSummarizing && diffStore.summarizingFile === file?.filename
  );

  let viewMode = $state<'semantic' | 'code'>('semantic');
  let transitioning = $state(false);

  function toggleView() {
    transitioning = true;
    setTimeout(() => {
      viewMode = viewMode === 'semantic' ? 'code' : 'semantic';
      setTimeout(() => { transitioning = false; }, 30);
    }, 200);
  }

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

  interface PatchHunk {
    header: string;
    lines: Array<{ type: 'add' | 'remove' | 'context'; text: string }>;
  }

  function parsePatchHunks(patch: string): PatchHunk[] {
    if (!patch) return [];
    const hunks: PatchHunk[] = [];
    let current: PatchHunk | null = null;

    for (const line of patch.split('\n')) {
      if (line.startsWith('@@')) {
        current = { header: line, lines: [] };
        hunks.push(current);
      } else if (current) {
        if (line.startsWith('+')) {
          current.lines.push({ type: 'add', text: line.slice(1) });
        } else if (line.startsWith('-')) {
          current.lines.push({ type: 'remove', text: line.slice(1) });
        } else if (line.length > 0) {
          current.lines.push({ type: 'context', text: line.startsWith(' ') ? line.slice(1) : line });
        }
      }
    }
    return hunks;
  }

  // Parse summary into bullet sections that can be mapped to hunks
  function parseSummaryBullets(text: string): string[] {
    if (!text) return [];
    return text.split('\n')
      .map(l => l.trim())
      .filter(l => l.startsWith('- ') || l.startsWith('* '))
      .map(l => l.replace(/^[-*]\s+/, ''));
  }
</script>

{#if file}
  <div class="viewer">
    <!-- Sticky header with file info and view toggle -->
    <div class="file-header">
      <span class="status-badge" style="color: {statusColor(file.status)}">
        {statusLabel(file.status)}
      </span>
      <span class="filename">{file.filename}</span>
      <span class="file-stats">
        <span class="add">+{file.additions}</span>
        <span class="del">-{file.deletions}</span>
      </span>

      <!-- View toggle button -->
      {#if hasSummary}
        <button class="view-toggle" onclick={toggleView} title="Switch between semantic and code view">
          <span class="toggle-track" class:code={viewMode === 'code'}>
            <span class="toggle-thumb"></span>
          </span>
          <span class="toggle-label">
            {viewMode === 'semantic' ? 'Semantic' : 'Code'}
          </span>
        </button>
      {/if}
    </div>

    <!-- Content area with transition -->
    <div class="diff-body" class:transitioning>
      {#if viewMode === 'semantic'}
        <!-- SEMANTIC VIEW -->
        {#if hasSummary}
          {@const hunks = parsePatchHunks(file.patch)}
          {@const bullets = parseSummaryBullets(summary!.summary)}
          <div class="semantic-view">
            {#each hunks as hunk, i}
              <div class="hunk-block">
                <div class="hunk-header">
                  <span class="hunk-location">{hunk.header.replace(/^@@\s*/, '').replace(/\s*@@.*/, '')}</span>
                </div>
                <div class="semantic-content">
                  {#if bullets[i]}
                    <div class="semantic-bullet">
                      <span class="bullet-icon">&#9889;</span>
                      <span class="bullet-text">{@html formatInline(bullets[i])}</span>
                    </div>
                  {:else if i === 0 && bullets.length > 0}
                    <!-- Show all bullets in first hunk if we can't map 1:1 -->
                    {#each bullets as bullet}
                      <div class="semantic-bullet">
                        <span class="bullet-icon">&#9889;</span>
                        <span class="bullet-text">{@html formatInline(bullet)}</span>
                      </div>
                    {/each}
                  {/if}
                  <!-- Collapsed code preview -->
                  <button class="code-peek" onclick={toggleView}>
                    <span class="peek-icon">&#60;/&#62;</span>
                    <span class="peek-lines">
                      {hunk.lines.filter(l => l.type === 'add').length} added,
                      {hunk.lines.filter(l => l.type === 'remove').length} removed
                    </span>
                  </button>
                </div>
              </div>
            {/each}

            <!-- If no hunks but we have summary, show full summary -->
            {#if hunks.length === 0}
              <div class="full-summary">
                <div class="summary-text">{@html formatSummary(summary!.summary)}</div>
              </div>
            {/if}

            <!-- If we had more bullets than hunks, show remaining -->
            {#if bullets.length > hunks.length && hunks.length > 0}
              <div class="extra-bullets">
                {#each bullets.slice(hunks.length) as bullet}
                  <div class="semantic-bullet">
                    <span class="bullet-icon">&#9889;</span>
                    <span class="bullet-text">{@html formatInline(bullet)}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else if isCurrentlySummarizing}
          <div class="loading-state">
            <span class="spinner"></span>
            Analyzing changes...
          </div>
        {:else if !settingsStore.hasClaudeKey}
          <div class="hint-state">
            Configure your Claude API key in Settings to get AI summaries.
            <br>Showing raw diff instead.
            <div class="raw-fallback">
              {@render rawDiff(file.patch)}
            </div>
          </div>
        {:else}
          <!-- No summary yet, show raw diff with hint -->
          <div class="hint-state">
            Click "Summarize All" to generate semantic view.
            <div class="raw-fallback">
              {@render rawDiff(file.patch)}
            </div>
          </div>
        {/if}

      {:else}
        <!-- CODE VIEW -->
        {@const hunks = parsePatchHunks(file.patch)}
        <div class="code-view">
          {#each hunks as hunk, i}
            <div class="hunk-block">
              <div class="hunk-header">
                <span class="hunk-location">{hunk.header.replace(/^@@\s*/, '').replace(/\s*@@.*/, '')}</span>
                <!-- If we have a matching semantic bullet, show it as a subtle label -->
                {#if hasSummary}
                  {@const bullets = parseSummaryBullets(summary!.summary)}
                  {#if bullets[i]}
                    <button class="semantic-peek" onclick={toggleView}>
                      <span class="peek-icon">&#9889;</span>
                      <span class="peek-text">{truncate(bullets[i], 60)}</span>
                    </button>
                  {/if}
                {/if}
              </div>
              <div class="diff-lines">
                {#each hunk.lines as line}
                  <div class="diff-line {line.type}">
                    <pre>{line.text}</pre>
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{:else}
  <div class="empty">
    <p>Select a file from the tree to view changes</p>
  </div>
{/if}

{#snippet rawDiff(patch: string)}
  {@const hunks = parsePatchHunks(patch)}
  {#each hunks as hunk}
    <div class="hunk-block" style="margin-top: 12px">
      <div class="hunk-header">
        <span class="hunk-location">{hunk.header.replace(/^@@\s*/, '').replace(/\s*@@.*/, '')}</span>
      </div>
      <div class="diff-lines">
        {#each hunk.lines as line}
          <div class="diff-line {line.type}">
            <pre>{line.text}</pre>
          </div>
        {/each}
      </div>
    </div>
  {/each}
{/snippet}

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

  function formatInline(text: string): string {
    return text
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/`(.+?)`/g, '<code>$1</code>');
  }

  function truncate(text: string, max: number): string {
    if (text.length <= max) return text;
    return text.slice(0, max) + '...';
  }
</script>

<style>
  .viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Header */
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
    flex-shrink: 0;
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
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-stats { font-size: 12px; flex-shrink: 0; }
  .add { color: var(--color-added-text); margin-right: 6px; }
  .del { color: var(--color-removed-text); }

  /* Toggle button */
  .view-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 20px;
    cursor: pointer;
    flex-shrink: 0;
    transition: border-color 0.2s;
  }
  .view-toggle:hover { border-color: var(--color-accent); }
  .toggle-track {
    width: 32px;
    height: 18px;
    border-radius: 9px;
    background: var(--color-accent);
    position: relative;
    transition: background 0.25s;
  }
  .toggle-track.code { background: var(--color-bg-hover); }
  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: white;
    transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }
  .toggle-track.code .toggle-thumb { transform: translateX(14px); }
  .toggle-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    min-width: 58px;
  }

  /* Diff body with transition */
  .diff-body {
    flex: 1;
    overflow-y: auto;
    transition: opacity 0.2s ease, transform 0.2s ease;
  }
  .diff-body.transitioning {
    opacity: 0;
    transform: translateY(6px);
  }

  /* Shared hunk styles */
  .hunk-block {
    border-bottom: 1px solid var(--color-border);
  }
  .hunk-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 16px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-text-muted);
  }
  .hunk-location { flex-shrink: 0; }

  /* Semantic view */
  .semantic-view { padding: 0; }
  .semantic-content {
    padding: 12px 16px;
  }
  .semantic-bullet {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px 0;
  }
  .semantic-bullet + .semantic-bullet {
    border-top: 1px solid var(--color-border);
  }
  .bullet-icon {
    font-size: 14px;
    flex-shrink: 0;
    margin-top: 1px;
  }
  .bullet-text {
    font-size: 13px;
    line-height: 1.6;
    color: var(--color-text-primary);
    user-select: text;
  }
  .bullet-text :global(code) {
    background: var(--color-bg-hover);
    padding: 1px 5px;
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .bullet-text :global(strong) { color: var(--color-accent); }

  .code-peek {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 8px;
    padding: 4px 10px;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text-muted);
    font-size: 11px;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }
  .code-peek:hover {
    color: var(--color-accent);
    border-color: var(--color-accent);
  }
  .peek-icon { font-family: var(--font-mono); font-size: 10px; }

  .full-summary {
    padding: 16px;
  }
  .full-summary :global(ul) { padding-left: 20px; margin: 4px 0; }
  .full-summary :global(li) { margin: 8px 0; font-size: 13px; line-height: 1.6; }
  .full-summary :global(code) {
    background: var(--color-bg-hover);
    padding: 1px 5px;
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .extra-bullets {
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
  }

  /* Code view */
  .code-view { padding: 0; }
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
  .diff-line.add { background: var(--color-added); color: var(--color-added-text); }
  .diff-line.remove { background: var(--color-removed); color: var(--color-removed-text); }
  .diff-line.context { color: var(--color-unchanged-text); }

  /* Semantic peek in code view */
  .semantic-peek {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    background: none;
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--color-text-muted);
    font-size: 11px;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.15s;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 400px;
  }
  .semantic-peek:hover {
    color: var(--color-accent);
    border-color: var(--color-accent);
    background: var(--color-bg-hover);
  }
  .semantic-peek .peek-icon { font-size: 11px; }
  .semantic-peek .peek-text { overflow: hidden; text-overflow: ellipsis; }

  /* States */
  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 40px;
    color: var(--color-text-secondary);
    font-size: 13px;
  }
  .hint-state {
    padding: 16px;
    color: var(--color-text-muted);
    font-size: 13px;
    font-style: italic;
  }
  .raw-fallback { margin-top: 8px; font-style: normal; }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    font-size: 14px;
  }
</style>
