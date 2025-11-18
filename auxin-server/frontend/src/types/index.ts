// API Types for Auxin Server

export interface Repository {
  namespace: string;
  name: string;
  path: string;
  description?: string;
}

export interface CreateRepoRequest {
  description?: string;
}

export interface Commit {
  id: string;
  message: string;
  author?: string;
  timestamp: string;
  parent_ids?: string[];
}

export interface Branch {
  name: string;
  commit_id: string;
}

export interface LogicProMetadata {
  bpm?: number;
  sample_rate?: number;
  key_signature?: string;
  tags?: string[];
  custom?: Record<string, any>;
}

export interface LockInfo {
  locked: boolean;
  holder?: {
    user: string;
    machine_id: string;
    acquired_at: string;
    expires_at: string;
  };
}

export interface LockRequest {
  timeout_hours?: number;
}

export interface FileEntry {
  path: string;
  size: number;
  is_dir: boolean;
  hash?: string;
}

export interface PushRequest {
  branch?: string;
  commits?: Commit[];
}

export interface PullResponse {
  commits: Commit[];
  branch: string;
}
