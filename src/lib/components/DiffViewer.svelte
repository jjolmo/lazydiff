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

  // Parse structured summary
  let parsed = $derived.by(() => {
    if (!summary) return null;
    const text = summary.summary;
    const role = text.match(/^ROLE:\s*(.+)$/m)?.[1]?.trim() || '';
    const calls = text.match(/^CALLS:\s*(.+)$/m)?.[1]?.trim() || '';
    const calledBy = text.match(/^CALLED_BY:\s*(.+)$/m)?.[1]?.trim() || '';
    const changesMatch = text.match(/CHANGES:\n([\s\S]*)/);
    const bullets = changesMatch
      ? changesMatch[1].split('\n').map(l => l.trim()).filter(l => l.startsWith('- ')).map(l => l.slice(2))
      : text.split('\n').map(l => l.trim()).filter(l => l.startsWith('- ')).map(l => l.slice(2));
    const callsList = calls && calls.toLowerCase() !== 'nothing' ? calls.split(',').map(s => s.trim()).filter(Boolean) : [];
    const calledByList = calledBy && calledBy.toLowerCase() !== 'unknown' ? calledBy.split(',').map(s => s.trim()).filter(Boolean) : [];
    return { role, calls: callsList, calledBy: calledByList, bullets };
  });

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
        if (line.startsWith('+')) current.lines.push({ type: 'add', text: line.slice(1) });
        else if (line.startsWith('-')) current.lines.push({ type: 'remove', text: line.slice(1) });
        else if (line.length > 0) current.lines.push({ type: 'context', text: line.startsWith(' ') ? line.slice(1) : line });
      }
    }
    return hunks;
  }

  function shortName(filename: string): string {
    const parts = filename.split('/');
    return parts[parts.length - 1];
  }
</script>

{#if file}
  <div class="viewer">
    <!-- Sticky header -->
    <div class="file-header">
      <span class="status-badge" style="color: {statusColor(file.status)}">
        {statusLabel(file.status)}
      </span>
      <span class="filename">{file.filename}</span>
      <span class="file-stats">
        <span class="add">+{file.additions}</span>
        <span class="del">-{file.deletions}</span>
      </span>
      {#if hasSummary}
        <button class="view-toggle" onclick={toggleView}>
          <span class="toggle-track" class:code={viewMode === 'code'}>
            <span class="toggle-thumb"></span>
          </span>
          <span class="toggle-label">{viewMode === 'semantic' ? 'Semantic' : 'Code'}</span>
        </button>
      {/if}
    </div>

    <!-- LOADING STATE — full area spinner -->
    {#if isCurrentlySummarizing && !hasSummary}
      <div class="summarizing-overlay">
        <div class="summarize-loader">
          <div class="loader-ring"></div>
          <p class="loader-text">Analyzing {shortName(file.filename)}...</p>
        </div>
      </div>
    {:else}
      <!-- Content with transition -->
      <div class="diff-body" class:transitioning>
        {#if viewMode === 'semantic' && hasSummary && parsed}
          <!-- SEMANTIC VIEW -->
          <div class="semantic-view">
            <!-- Role description -->
            {#if parsed.role}
              <div class="role-bar">
                <span class="role-label">This file</span>
                <span class="role-text">{parsed.role}</span>
              </div>
            {/if}

            <!-- Flow diagram -->
            {#if parsed.calledBy.length > 0 || parsed.calls.length > 0}
              <div class="flow-diagram">
                <div class="flow-col flow-left">
                  {#if parsed.calledBy.length > 0}
                    <div class="flow-header">Called by</div>
                    {#each parsed.calledBy as caller}
                      <div class="flow-node caller">{caller}</div>
                    {/each}
                  {/if}
                </div>
                <div class="flow-col flow-center">
                  <div class="flow-arrow-in" class:hidden={parsed.calledBy.length === 0}></div>
                  <div class="flow-node current">{shortName(file.filename)}</div>
                  <div class="flow-arrow-out" class:hidden={parsed.calls.length === 0}></div>
                </div>
                <div class="flow-col flow-right">
                  {#if parsed.calls.length > 0}
                    <div class="flow-header">Calls</div>
                    {#each parsed.calls as dep}
                      <div class="flow-node dep">{dep}</div>
                    {/each}
                  {/if}
                </div>
              </div>
            {/if}

            <!-- Change bullets -->
            {#if parsed.bullets.length > 0}
              <div class="changes-section">
                <div class="changes-header">Changes</div>
                {#each parsed.bullets as bullet}
                  <div class="semantic-bullet">
                    <span class="bullet-icon">&#9889;</span>
                    <span class="bullet-text">{@html formatInline(bullet)}</span>
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Peek to code -->
            <div class="peek-bar">
              <button class="code-peek" onclick={toggleView}>
                <span class="peek-icon">&#60;/&#62;</span>
                View raw diff
              </button>
            </div>
          </div>

        {:else if viewMode === 'semantic' && !hasSummary}
          <!-- No summary, show raw diff with hint -->
          <div class="hint-banner">
            {#if !settingsStore.hasClaudeKey}
              Configure your Claude API key in Settings to get AI summaries.
            {:else}
              Waiting for summary...
            {/if}
          </div>
          {@render rawDiffView(file.patch)}

        {:else}
          <!-- CODE VIEW -->
          {@const hunks = parsePatchHunks(file.patch)}
          <div class="code-view">
            {#each hunks as hunk, i}
              <div class="hunk-block">
                <div class="hunk-header">
                  <span class="hunk-location">{hunk.header.replace(/^@@\s*/, '').replace(/\s*@@.*/, '')}</span>
                  {#if hasSummary && parsed && parsed.bullets[i]}
                    <button class="semantic-peek" onclick={toggleView}>
                      <span class="peek-icon">&#9889;</span>
                      <span class="peek-text">{truncate(parsed.bullets[i], 60)}</span>
                    </button>
                  {/if}
                </div>
                <div class="diff-lines">
                  {#each hunk.lines as line}
                    <div class="diff-line {line.type}"><pre>{line.text}</pre></div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{:else}
  <div class="empty">
    <p>Select a file from the tree to view changes</p>
  </div>
{/if}

{#snippet rawDiffView(patch: string)}
  {@const hunks = parsePatchHunks(patch)}
  <div class="code-view">
    {#each hunks as hunk}
      <div class="hunk-block">
        <div class="hunk-header">
          <span class="hunk-location">{hunk.header.replace(/^@@\s*/, '').replace(/\s*@@.*/, '')}</span>
        </div>
        <div class="diff-lines">
          {#each hunk.lines as line}
            <div class="diff-line {line.type}"><pre>{line.text}</pre></div>
          {/each}
        </div>
      </div>
    {/each}
  </div>
{/snippet}

<script lang="ts" module>
  function formatInline(text: string): string {
    return text
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/`(.+?)`/g, '<code>$1</code>');
  }
  function truncate(text: string, max: number): string {
    return text.length <= max ? text : text.slice(0, max) + '...';
  }
</script>

<style>
  .viewer { display: flex; flex-direction: column; height: 100%; overflow: hidden; }

  /* Header */
  .file-header {
    display: flex; align-items: center; gap: 10px; padding: 10px 16px;
    background: var(--color-bg-tertiary); border-bottom: 1px solid var(--color-border);
    position: sticky; top: 0; z-index: 1; flex-shrink: 0;
  }
  .status-badge { font-size: 11px; font-weight: 600; padding: 2px 8px; border-radius: 4px; background: var(--color-bg-hover); }
  .filename { font-family: var(--font-mono); font-size: 13px; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .file-stats { font-size: 12px; flex-shrink: 0; }
  .add { color: var(--color-added-text); margin-right: 6px; }
  .del { color: var(--color-removed-text); }

  /* Toggle */
  .view-toggle {
    display: flex; align-items: center; gap: 8px; padding: 4px 10px;
    background: var(--color-bg-primary); border: 1px solid var(--color-border);
    border-radius: 20px; cursor: pointer; flex-shrink: 0; transition: border-color 0.2s;
  }
  .view-toggle:hover { border-color: var(--color-accent); }
  .toggle-track { width: 32px; height: 18px; border-radius: 9px; background: var(--color-accent); position: relative; transition: background 0.25s; }
  .toggle-track.code { background: var(--color-bg-hover); }
  .toggle-thumb { position: absolute; top: 2px; left: 2px; width: 14px; height: 14px; border-radius: 50%; background: white; transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1); }
  .toggle-track.code .toggle-thumb { transform: translateX(14px); }
  .toggle-label { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); min-width: 58px; }

  /* Loading overlay */
  .summarizing-overlay {
    flex: 1; display: flex; align-items: center; justify-content: center;
  }
  .summarize-loader { display: flex; flex-direction: column; align-items: center; gap: 16px; }
  .loader-ring {
    width: 40px; height: 40px;
    border: 3px solid var(--color-border); border-top-color: var(--color-accent);
    border-radius: 50%; animation: spin 0.9s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  .loader-text { font-size: 13px; color: var(--color-text-secondary); animation: pulse 1.5s ease-in-out infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes pulse { 50% { opacity: 0.4; } }

  /* Body transition */
  .diff-body { flex: 1; overflow-y: auto; transition: opacity 0.2s ease, transform 0.2s ease; }
  .diff-body.transitioning { opacity: 0; transform: translateY(6px); }

  /* Role bar */
  .role-bar {
    display: flex; align-items: baseline; gap: 8px; padding: 10px 16px;
    background: var(--color-bg-secondary); border-bottom: 1px solid var(--color-border);
  }
  .role-label { font-size: 11px; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.5px; flex-shrink: 0; }
  .role-text { font-size: 13px; color: var(--color-text-primary); line-height: 1.4; }

  /* Flow diagram */
  .flow-diagram {
    display: flex; align-items: center; justify-content: center; gap: 0;
    padding: 16px; border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-primary);
  }
  .flow-col { display: flex; flex-direction: column; align-items: center; gap: 6px; min-width: 80px; }
  .flow-left { align-items: flex-end; }
  .flow-right { align-items: flex-start; }
  .flow-center { align-items: center; flex-shrink: 0; position: relative; }
  .flow-header { font-size: 10px; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 2px; }
  .flow-node {
    padding: 4px 12px; border-radius: 6px; font-size: 12px; font-weight: 500;
    white-space: nowrap; max-width: 160px; overflow: hidden; text-overflow: ellipsis;
  }
  .flow-node.caller { background: var(--color-bg-tertiary); color: var(--color-text-secondary); border: 1px solid var(--color-border); }
  .flow-node.current { background: var(--color-accent); color: white; font-weight: 600; padding: 6px 16px; font-size: 13px; }
  .flow-node.dep { background: var(--color-bg-tertiary); color: var(--color-text-secondary); border: 1px solid var(--color-border); }
  .flow-arrow-in, .flow-arrow-out {
    width: 40px; height: 2px; background: var(--color-border); position: relative; margin: 4px 0;
  }
  .flow-arrow-in::after, .flow-arrow-out::after {
    content: ''; position: absolute; top: -4px; width: 0; height: 0;
    border-top: 5px solid transparent; border-bottom: 5px solid transparent;
  }
  .flow-arrow-in::after { right: -1px; border-left: 6px solid var(--color-accent); }
  .flow-arrow-out::after { right: -1px; border-left: 6px solid var(--color-border); }
  .hidden { visibility: hidden; }

  /* Changes */
  .changes-section { padding: 0; }
  .changes-header {
    padding: 8px 16px; font-size: 11px; color: var(--color-text-muted);
    text-transform: uppercase; letter-spacing: 0.5px;
    background: var(--color-bg-secondary); border-bottom: 1px solid var(--color-border);
  }
  .semantic-bullet { display: flex; align-items: flex-start; gap: 10px; padding: 10px 16px; border-bottom: 1px solid var(--color-border); }
  .bullet-icon { font-size: 14px; flex-shrink: 0; margin-top: 1px; }
  .bullet-text { font-size: 13px; line-height: 1.6; color: var(--color-text-primary); user-select: text; }
  .bullet-text :global(code) { background: var(--color-bg-hover); padding: 1px 5px; border-radius: 3px; font-family: var(--font-mono); font-size: 12px; }
  .bullet-text :global(strong) { color: var(--color-accent); }

  .peek-bar { padding: 10px 16px; }
  .code-peek {
    display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px;
    background: var(--color-bg-tertiary); border: 1px solid var(--color-border);
    border-radius: 4px; color: var(--color-text-muted); font-size: 11px;
    cursor: pointer; transition: color 0.15s, border-color 0.15s;
  }
  .code-peek:hover { color: var(--color-accent); border-color: var(--color-accent); }
  .peek-icon { font-family: var(--font-mono); font-size: 10px; }

  /* Hint */
  .hint-banner { padding: 10px 16px; font-size: 12px; color: var(--color-text-muted); font-style: italic; border-bottom: 1px solid var(--color-border); }

  /* Code view */
  .code-view { padding: 0; }
  .hunk-block { border-bottom: 1px solid var(--color-border); }
  .hunk-header {
    display: flex; align-items: center; gap: 12px; padding: 6px 16px;
    background: var(--color-bg-secondary); border-bottom: 1px solid var(--color-border);
    font-family: var(--font-mono); font-size: 11px; color: var(--color-text-muted);
  }
  .hunk-location { flex-shrink: 0; }
  .diff-lines { font-family: var(--font-mono); font-size: 12px; }
  .diff-line { padding: 0 16px; user-select: text; }
  .diff-line pre { margin: 0; white-space: pre-wrap; word-break: break-all; }
  .diff-line.add { background: var(--color-added); color: var(--color-added-text); }
  .diff-line.remove { background: var(--color-removed); color: var(--color-removed-text); }
  .diff-line.context { color: var(--color-unchanged-text); }

  .semantic-peek {
    display: inline-flex; align-items: center; gap: 5px; padding: 2px 8px;
    background: none; border: 1px solid transparent; border-radius: 4px;
    color: var(--color-text-muted); font-size: 11px; font-family: inherit;
    cursor: pointer; transition: all 0.15s; overflow: hidden; text-overflow: ellipsis;
    white-space: nowrap; max-width: 400px;
  }
  .semantic-peek:hover { color: var(--color-accent); border-color: var(--color-accent); background: var(--color-bg-hover); }
  .semantic-peek .peek-text { overflow: hidden; text-overflow: ellipsis; }

  .empty { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--color-text-muted); font-size: 14px; }
</style>
