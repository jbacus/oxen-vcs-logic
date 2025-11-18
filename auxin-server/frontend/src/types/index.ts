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
  name: string;
  path: string;
  type: 'file' | 'dir';
  size?: number;
  hash?: string;
  children?: FileEntry[];
}

// Auth types
export interface User {
  id: string;
  username: string;
  email: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

// Activity types
export interface Activity {
  id: string;
  type: 'commit' | 'lock_acquired' | 'lock_released' | 'branch_created' | 'user_joined';
  user: string;
  description: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

export interface PushRequest {
  branch?: string;
  commits?: Commit[];
}

export interface PullResponse {
  commits: Commit[];
  branch: string;
}
