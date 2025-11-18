import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios from 'axios';
import {
  listRepositories,
  getRepository,
  createRepository,
  getCommits,
  listBranches,
  createBranch,
  getLockStatus,
  acquireLock,
  releaseLock,
  getMetadata,
  storeMetadata,
  login,
  register,
} from './api';

// Mock axios
vi.mock('axios', () => {
  const mockAxios = {
    create: vi.fn(() => mockAxios),
    get: vi.fn(),
    post: vi.fn(),
    delete: vi.fn(),
    interceptors: {
      request: { use: vi.fn() },
      response: { use: vi.fn() },
    },
  };
  return { default: mockAxios };
});

const mockedAxios = axios as any;

describe('API Client', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Repository endpoints', () => {
    it('lists repositories', async () => {
      const mockRepos = [
        { namespace: 'user', name: 'project1', path: '/path' },
        { namespace: 'user', name: 'project2', path: '/path2' },
      ];
      mockedAxios.get.mockResolvedValueOnce({ data: mockRepos });

      const response = await listRepositories();
      expect(response.data).toEqual(mockRepos);
      expect(mockedAxios.get).toHaveBeenCalledWith('/repos');
    });

    it('gets a repository by namespace and name', async () => {
      const mockRepo = { namespace: 'user', name: 'project', path: '/path' };
      mockedAxios.get.mockResolvedValueOnce({ data: mockRepo });

      const response = await getRepository('user', 'project');
      expect(response.data).toEqual(mockRepo);
      expect(mockedAxios.get).toHaveBeenCalledWith('/repos/user/project');
    });

    it('creates a repository', async () => {
      const mockRepo = { namespace: 'user', name: 'new-project', path: '/path' };
      mockedAxios.post.mockResolvedValueOnce({ data: mockRepo });

      const response = await createRepository('user', 'new-project', {
        description: 'Test project',
      });
      expect(response.data).toEqual(mockRepo);
      expect(mockedAxios.post).toHaveBeenCalledWith('/repos/user/new-project', {
        description: 'Test project',
      });
    });
  });

  describe('Commit endpoints', () => {
    it('gets commits for a repository', async () => {
      const mockCommits = [
        { id: 'abc123', message: 'Initial commit', timestamp: '2024-01-01' },
        { id: 'def456', message: 'Add feature', timestamp: '2024-01-02' },
      ];
      mockedAxios.get.mockResolvedValueOnce({ data: mockCommits });

      const response = await getCommits('user', 'project');
      expect(response.data).toEqual(mockCommits);
      expect(mockedAxios.get).toHaveBeenCalledWith('/repos/user/project/commits');
    });
  });

  describe('Branch endpoints', () => {
    it('lists branches', async () => {
      const mockBranches = [
        { name: 'main', commit_id: 'abc123' },
        { name: 'feature', commit_id: 'def456' },
      ];
      mockedAxios.get.mockResolvedValueOnce({ data: mockBranches });

      const response = await listBranches('user', 'project');
      expect(response.data).toEqual(mockBranches);
      expect(mockedAxios.get).toHaveBeenCalledWith('/repos/user/project/branches');
    });

    it('creates a branch', async () => {
      mockedAxios.post.mockResolvedValueOnce({ data: { success: true } });

      await createBranch('user', 'project', 'feature-branch', 'abc123');
      expect(mockedAxios.post).toHaveBeenCalledWith('/repos/user/project/branches', {
        name: 'feature-branch',
        from_commit: 'abc123',
      });
    });
  });

  describe('Lock endpoints', () => {
    it('gets lock status', async () => {
      const mockLock = {
        locked: true,
        holder: {
          user: 'john',
          machine_id: 'machine1',
          acquired_at: '2024-01-01T00:00:00Z',
          expires_at: '2024-01-01T08:00:00Z',
        },
      };
      mockedAxios.get.mockResolvedValueOnce({ data: mockLock });

      const response = await getLockStatus('user', 'project');
      expect(response.data).toEqual(mockLock);
      expect(mockedAxios.get).toHaveBeenCalledWith('/repos/user/project/locks/status');
    });

    it('acquires a lock', async () => {
      const mockLock = { locked: true, holder: { user: 'john' } };
      mockedAxios.post.mockResolvedValueOnce({ data: mockLock });

      const response = await acquireLock('user', 'project', { timeout_hours: 4 });
      expect(response.data).toEqual(mockLock);
      expect(mockedAxios.post).toHaveBeenCalledWith(
        '/repos/user/project/locks/acquire',
        { timeout_hours: 4 }
      );
    });

    it('releases a lock', async () => {
      mockedAxios.post.mockResolvedValueOnce({ data: { success: true } });

      await releaseLock('user', 'project');
      expect(mockedAxios.post).toHaveBeenCalledWith('/repos/user/project/locks/release');
    });
  });

  describe('Metadata endpoints', () => {
    it('gets metadata for a commit', async () => {
      const mockMetadata = {
        bpm: 120,
        sample_rate: 48000,
        key_signature: 'C Major',
        tags: ['rock', 'demo'],
      };
      mockedAxios.get.mockResolvedValueOnce({ data: mockMetadata });

      const response = await getMetadata('user', 'project', 'abc123');
      expect(response.data).toEqual(mockMetadata);
      expect(mockedAxios.get).toHaveBeenCalledWith('/repos/user/project/metadata/abc123');
    });

    it('stores metadata for a commit', async () => {
      const metadata = {
        bpm: 128,
        sample_rate: 44100,
        key_signature: 'G Major',
      };
      mockedAxios.post.mockResolvedValueOnce({ data: { success: true } });

      await storeMetadata('user', 'project', 'abc123', metadata);
      expect(mockedAxios.post).toHaveBeenCalledWith(
        '/repos/user/project/metadata/abc123',
        metadata
      );
    });
  });

  describe('Auth endpoints', () => {
    it('logs in a user', async () => {
      const mockResponse = {
        token: 'jwt-token',
        user: { id: '1', username: 'john', email: 'john@example.com' },
      };
      mockedAxios.post.mockResolvedValueOnce({ data: mockResponse });

      const response = await login('john@example.com', 'password123');
      expect(response.data).toEqual(mockResponse);
      expect(mockedAxios.post).toHaveBeenCalledWith('/auth/login', {
        email: 'john@example.com',
        password: 'password123',
      });
    });

    it('registers a new user', async () => {
      const mockResponse = {
        token: 'jwt-token',
        user: { id: '2', username: 'jane', email: 'jane@example.com' },
      };
      mockedAxios.post.mockResolvedValueOnce({ data: mockResponse });

      const response = await register('jane', 'jane@example.com', 'password123');
      expect(response.data).toEqual(mockResponse);
      expect(mockedAxios.post).toHaveBeenCalledWith('/auth/register', {
        username: 'jane',
        email: 'jane@example.com',
        password: 'password123',
      });
    });
  });
});
