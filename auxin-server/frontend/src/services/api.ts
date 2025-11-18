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
  AuthResponse,
  User,
  Activity,
  FileEntry,
} from '@/types';

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
});

// Auth interceptor - add token to requests
api.interceptors.request.use((config) => {
  const authData = localStorage.getItem('auxin-auth');
  if (authData) {
    try {
      const parsed = JSON.parse(authData);
      if (parsed.state?.token) {
        config.headers.Authorization = `Bearer ${parsed.state.token}`;
      }
    } catch {
      // Invalid JSON in localStorage, ignore
    }
  }
  return config;
});

// Error interceptor - handle 401 redirects
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('auxin-auth');
      if (!window.location.pathname.startsWith('/login')) {
        window.location.href = '/login';
      }
    }
    return Promise.reject(error);
  }
);

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

// Auth endpoints
export const login = (email: string, password: string) =>
  api.post<AuthResponse>('/auth/login', { email, password });

export const register = (username: string, email: string, password: string) =>
  api.post<AuthResponse>('/auth/register', { username, email, password });

export const logout = () => api.post('/auth/logout');

export const getCurrentUser = () => api.get<User>('/auth/me');

// Activity endpoint
export const getActivity = (namespace: string, name: string, limit: number = 20) =>
  api.get<Activity[]>(`/repos/${namespace}/${name}/activity?limit=${limit}`);

// File tree endpoint
export const getFileTree = (namespace: string, name: string, commit: string) =>
  api.get<FileEntry[]>(`/repos/${namespace}/${name}/tree/${commit}`);

// Alias for branch listing (used by BranchManager)
export const getBranches = listBranches;

// Delete branch
export const deleteBranch = (namespace: string, name: string, branch: string) =>
  api.delete(`/repos/${namespace}/${name}/branches/${branch}`);

export default api;
