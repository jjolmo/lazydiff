export interface FileDiff {
  filename: string;
  status: 'added' | 'removed' | 'modified' | 'renamed';
  additions: number;
  deletions: number;
  patch: string;
}

export interface DiffResult {
  base: string;
  head: string;
  files: FileDiff[];
  total_additions: number;
  total_deletions: number;
}

export interface AiSummary {
  filename: string;
  summary: string;
}

export interface FileTreeNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  status?: string;
  additions?: number;
  deletions?: number;
  children?: FileTreeNode[];
  expanded?: boolean;
}
