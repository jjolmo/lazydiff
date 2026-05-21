<script lang="ts">
  import type { FileTreeNode } from '$lib/types';
  import { diffStore } from '$lib/stores/diff.svelte';

  let { nodes = [], depth = 0 }: { nodes: FileTreeNode[]; depth?: number } = $props();

  function toggleDir(node: FileTreeNode) {
    node.expanded = !node.expanded;
  }

  function statusColor(status?: string): string {
    switch (status) {
      case 'added': return 'var(--color-added-text)';
      case 'removed': return 'var(--color-removed-text)';
      case 'modified': return 'var(--color-accent)';
      default: return 'var(--color-text-secondary)';
    }
  }

  function statusIcon(status?: string): string {
    switch (status) {
      case 'added': return 'A';
      case 'removed': return 'D';
      case 'modified': return 'M';
      case 'renamed': return 'R';
      default: return '';
    }
  }
</script>

{#each nodes as node}
  {#if node.type === 'directory'}
    <button
      class="tree-item dir"
      style="padding-left: {12 + depth * 16}px"
      onclick={() => toggleDir(node)}
    >
      <span class="arrow" class:expanded={node.expanded}>&#9654;</span>
      <span class="icon">&#128193;</span>
      <span class="name">{node.name}</span>
    </button>
    {#if node.expanded && node.children}
      <svelte:self nodes={node.children} depth={depth + 1} />
    {/if}
  {:else}
    <button
      class="tree-item file"
      class:selected={diffStore.selectedFile === node.path}
      style="padding-left: {12 + depth * 16}px"
      onclick={() => diffStore.selectFile(node.path)}
    >
      <span class="name">{node.name}</span>
      <span class="badge" style="color: {statusColor(node.status)}">{statusIcon(node.status)}</span>
      {#if node.additions || node.deletions}
        <span class="stats">
          {#if node.additions}<span class="add">+{node.additions}</span>{/if}
          {#if node.deletions}<span class="del">-{node.deletions}</span>{/if}
        </span>
      {/if}
    </button>
  {/if}
{/each}

<style>
  .tree-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 8px;
    border: none;
    background: none;
    color: var(--color-text-primary);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    white-space: nowrap;
  }
  .tree-item:hover { background: var(--color-bg-hover); }
  .tree-item.selected { background: var(--color-bg-selected); }
  .arrow {
    font-size: 8px;
    transition: transform 0.15s;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }
  .arrow.expanded { transform: rotate(90deg); }
  .icon { font-size: 14px; flex-shrink: 0; }
  .name { overflow: hidden; text-overflow: ellipsis; flex: 1; }
  .badge { font-size: 10px; font-weight: 700; flex-shrink: 0; }
  .stats { font-size: 10px; flex-shrink: 0; margin-left: auto; }
  .add { color: var(--color-added-text); margin-right: 4px; }
  .del { color: var(--color-removed-text); }
</style>
