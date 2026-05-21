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
  mode = $state<'github' | 'local'>('github');

  // GitHub fields
  inputUrl = $state('');
  ghRepo = $state('');
  ghHead = $state('');
  ghBase = $state('trunk');
  ghBranches = $state<string[]>([]);
  ghLoadingBranches = $state(false);

  // Local fields
  localPath = $state('');
  localBranch = $state('');
  localBase = $state('');
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

  // Parse "owner/repo" from URL or direct input
  parseRepo(input: string): { owner: string; repo: string } | null {
    const trimmed = input.trim().replace(/\/$/, '');
    // Direct owner/repo format
    const directMatch = trimmed.match(/^([a-zA-Z0-9_.-]+)\/([a-zA-Z0-9_.-]+)$/);
    if (directMatch) return { owner: directMatch[1], repo: directMatch[2] };
    // GitHub URL: extract owner/repo
    const urlMatch = trimmed.match(/github\.com\/([a-zA-Z0-9_.-]+)\/([a-zA-Z0-9_.-]+)/);
    if (urlMatch) return { owner: urlMatch[1], repo: urlMatch[2] };
    return null;
  }

  async loadGitHubBranches() {
    const parsed = this.parseRepo(this.ghRepo);
    if (!parsed) {
      this.ghBranches = [];
      return;
    }
    this.ghLoadingBranches = true;
    try {
      this.ghBranches = await invoke<string[]>('fetch_github_branches', {
        owner: parsed.owner,
        repo: parsed.repo
      });
    } catch {
      this.ghBranches = [];
    } finally {
      this.ghLoadingBranches = false;
    }
  }

  async fetchGitHubDiff() {
    // If there's a full URL with /pull/ or /compare/, use the URL parser
    if (this.ghRepo.includes('/pull/') || this.ghRepo.includes('/compare/')) {
      return this.fetchGitHubDiffByUrl(this.ghRepo);
    }

    const parsed = this.parseRepo(this.ghRepo);
    if (!parsed || !this.ghHead) return;

    this.isLoading = true;
    this.error = null;
    this.summaries = new Map();
    try {
      this.diffResult = await invoke<DiffResult>('fetch_github_compare', {
        owner: parsed.owner,
        repo: parsed.repo,
        base: this.ghBase || 'main',
        head: this.ghHead
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

  async fetchGitHubDiffByUrl(url: string) {
    this.isLoading = true;
    this.error = null;
    this.summaries = new Map();
    try {
      this.diffResult = await invoke<DiffResult>('fetch_github_diff', { url });
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
