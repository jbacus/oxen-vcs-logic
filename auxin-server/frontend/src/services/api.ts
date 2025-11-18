import axios from 'axios';
import type {
  Repository,
  CreateRepoRequest,
  Commit,
  Branch,
  LogicProMetadata,
  LockInfo,
  LockRequest,
  PushRequest,
  PullResponse,
} from '@/types';

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
});

// Health check
export const healthCheck = () => axios.get('/health');

// Repository endpoints
export const listRepositories = () =>
  api.get<Repository[]>('/repos');

export const getRepository = (namespace: string, name: string) =>
  api.get<Repository>(`/repos/${namespace}/${name}`);

export const createRepository = (
  namespace: string,
  name: string,
  data: CreateRepoRequest
) => api.post<Repository>(`/repos/${namespace}/${name}`, data);

// Commit endpoints
export const getCommits = (namespace: string, name: string) =>
  api.get<Commit[]>(`/repos/${namespace}/${name}/commits`);

// Branch endpoints
export const listBranches = (namespace: string, name: string) =>
  api.get<Branch[]>(`/repos/${namespace}/${name}/branches`);

export const createBranch = (
  namespace: string,
  name: string,
  branchName: string,
  fromCommit?: string
) => api.post(`/repos/${namespace}/${name}/branches`, {
  name: branchName,
  from_commit: fromCommit,
});

// Push/Pull endpoints
export const pushRepository = (
  namespace: string,
  name: string,
  data: PushRequest
) => api.post(`/repos/${namespace}/${name}/push`, data);

export const pullRepository = (namespace: string, name: string) =>
  api.post<PullResponse>(`/repos/${namespace}/${name}/pull`);

// Metadata endpoints (Auxin extension)
export const getMetadata = (
  namespace: string,
  name: string,
  commit: string
) => api.get<LogicProMetadata>(`/repos/${namespace}/${name}/metadata/${commit}`);

export const storeMetadata = (
  namespace: string,
  name: string,
  commit: string,
  metadata: LogicProMetadata
) => api.post(`/repos/${namespace}/${name}/metadata/${commit}`, metadata);

// Lock endpoints (Auxin extension)
export const getLockStatus = (namespace: string, name: string) =>
  api.get<LockInfo>(`/repos/${namespace}/${name}/locks/status`);

export const acquireLock = (
  namespace: string,
  name: string,
  request?: LockRequest
) => api.post<LockInfo>(`/repos/${namespace}/${name}/locks/acquire`, request);

export const releaseLock = (namespace: string, name: string) =>
  api.post(`/repos/${namespace}/${name}/locks/release`);

export const heartbeatLock = (namespace: string, name: string) =>
  api.post(`/repos/${namespace}/${name}/locks/heartbeat`);

export default api;
