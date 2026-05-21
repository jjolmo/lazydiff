import { invoke } from '@tauri-apps/api/core';
import type { DiffResult, FileDiff, AiSummary, FileTreeNode } from '$lib/types';
import { settingsStore } from './settings.svelte';

class DiffStore {
  // State
  diffResult = $state<DiffResult | null>(null);
  summaries = $state<Map<string, AiSummary>>(new Map());
  selectedFile = $state<string | null>(null);
  isLoading = $state(false);
  isSummarizing = $state(false);
  summarizingFile = $state<string | null>(null);
  error = $state<string | null>(null);
  inputUrl = $state('');
  mode = $state<'github' | 'local'>('github');
  localPath = $state('');
  localBranch = $state('');
  localBranches = $state<string[]>([]);

  get fileTree(): FileTreeNode[] {
    if (!this.diffResult) return [];
    return buildFileTree(this.diffResult.files);
  }

  get selectedFileDiff(): FileDiff | null {
    if (!this.selectedFile || !this.diffResult) return null;
    return this.diffResult.files.find(f => f.filename === this.selectedFile) || null;
  }

  get selectedFileSummary(): AiSummary | null {
    if (!this.selectedFile) return null;
    return this.summaries.get(this.selectedFile) || null;
  }

  get progress(): number {
    if (!this.diffResult) return 0;
    return (this.summaries.size / this.diffResult.files.length) * 100;
  }

  async fetchGitHubDiff() {
    if (!this.inputUrl.trim()) return;
    this.isLoading = true;
    this.error = null;
    this.summaries = new Map();
    try {
      this.diffResult = await invoke<DiffResult>('fetch_github_diff', { url: this.inputUrl });
      if (this.diffResult && this.diffResult.files.length > 0) {
        this.selectedFile = this.diffResult.files[0].filename;
      }
    } catch (e: any) {
      this.error = e.toString();
    } finally {
      this.isLoading = false;
    }
  }

  async fetchLocalDiff() {
    if (!this.localPath || !this.localBranch) return;
    this.isLoading = true;
    this.error = null;
    this.summaries = new Map();
    try {
      this.diffResult = await invoke<DiffResult>('fetch_local_diff', {
        repoPath: this.localPath,
        branch: this.localBranch
      });
      if (this.diffResult && this.diffResult.files.length > 0) {
        this.selectedFile = this.diffResult.files[0].filename;
      }
    } catch (e: any) {
      this.error = e.toString();
    } finally {
      this.isLoading = false;
    }
  }

  async loadBranches() {
    if (!this.localPath) return;
    try {
      this.localBranches = await invoke<string[]>('list_branches', { repoPath: this.localPath });
    } catch {
      this.localBranches = [];
    }
  }

  async summarizeAll() {
    if (!this.diffResult || !settingsStore.hasClaudeKey) return;
    this.isSummarizing = true;
    this.error = null;

    try {
      // Summarize in batches of 3
      const files = this.diffResult.files;
      for (let i = 0; i < files.length; i += 3) {
        const batch = files.slice(i, i + 3);
        this.summarizingFile = batch[0].filename;
        const results = await invoke<AiSummary[]>('summarize_with_claude', {
          apiKey: settingsStore.claudeApiKey,
          fileDiffs: batch
        });
        const newMap = new Map(this.summaries);
        for (const s of results) {
          newMap.set(s.filename, s);
        }
        this.summaries = newMap;
      }
    } catch (e: any) {
      this.error = e.toString();
    } finally {
      this.isSummarizing = false;
      this.summarizingFile = null;
    }
  }

  selectFile(filename: string) {
    this.selectedFile = filename;
  }

  reset() {
    this.diffResult = null;
    this.summaries = new Map();
    this.selectedFile = null;
    this.error = null;
  }
}

function buildFileTree(files: FileDiff[]): FileTreeNode[] {
  const root: FileTreeNode[] = [];

  for (const file of files) {
    const parts = file.filename.split('/');
    let current = root;

    for (let i = 0; i < parts.length; i++) {
      const name = parts[i];
      const isFile = i === parts.length - 1;
      const path = parts.slice(0, i + 1).join('/');

      const existing = current.find(n => n.name === name);
      if (existing) {
        if (!isFile && existing.children) {
          current = existing.children;
        }
      } else {
        const node: FileTreeNode = {
          name,
          path,
          type: isFile ? 'file' : 'directory',
          status: isFile ? file.status : undefined,
          additions: isFile ? file.additions : undefined,
          deletions: isFile ? file.deletions : undefined,
          children: isFile ? undefined : [],
          expanded: true
        };
        current.push(node);
        if (!isFile && node.children) {
          current = node.children;
        }
      }
    }
  }

  return sortTree(root);
}

function sortTree(nodes: FileTreeNode[]): FileTreeNode[] {
  nodes.sort((a, b) => {
    if (a.type !== b.type) return a.type === 'directory' ? -1 : 1;
    return a.name.localeCompare(b.name);
  });
  for (const node of nodes) {
    if (node.children) sortTree(node.children);
  }
  return nodes;
}

export const diffStore = new DiffStore();
