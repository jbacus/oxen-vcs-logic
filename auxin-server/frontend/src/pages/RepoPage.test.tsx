import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RepoPage } from './RepoPage';
import * as api from '@/services/api';

// Mock the API module
vi.mock('@/services/api', () => ({
  getRepository: vi.fn(),
  getCommits: vi.fn(),
  getLockStatus: vi.fn(),
  acquireLock: vi.fn(),
  releaseLock: vi.fn(),
  heartbeatLock: vi.fn(),
  getMetadata: vi.fn(),
}));

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

const renderWithRouter = (ui: React.ReactElement, { route = '/testuser/test-project.logicx' } = {}) => {
  const queryClient = createTestQueryClient();
  window.history.pushState({}, 'Test page', route);

  return render(
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route path="/:namespace/:name" element={ui} />
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  );
};

describe('RepoPage - Clone Tab Integration', () => {
  const mockRepo = {
    namespace: 'testuser',
    name: 'test-project.logicx',
    path: '/repos/testuser/test-project.logicx',
    description: 'A test Logic Pro project',
  };

  const mockCommits = [
    {
      id: 'abc123',
      message: 'Initial commit',
      timestamp: '2024-01-01T00:00:00Z',
      author: 'Test User',
    },
  ];

  const mockLockInfo = {
    locked: false,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (api.getRepository as any).mockResolvedValue({ data: mockRepo });
    (api.getCommits as any).mockResolvedValue({ data: mockCommits });
    (api.getLockStatus as any).mockResolvedValue({ data: mockLockInfo });
  });

  describe('Tab Navigation', () => {
    it('should show Clone tab as the default active tab', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      // Clone tab should be active
      const cloneTab = screen.getByRole('tab', { name: /Clone/i });
      expect(cloneTab).toHaveClass('border-primary-600', 'text-primary-600');
    });

    it('should render Clone tab content by default', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      // Clone instructions should be visible
      expect(screen.getByText(/Clone this Logic Pro project/i)).toBeInTheDocument();
      expect(screen.getByText(/Auxin CLI \(Recommended\)/i)).toBeInTheDocument();
    });

    it('should switch to Commits tab when clicked', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const commitsTab = screen.getByRole('tab', { name: /Commits/i });
      fireEvent.click(commitsTab);

      await waitFor(() => {
        expect(screen.getByText('Initial commit')).toBeInTheDocument();
      });
    });

    it('should switch to Locks tab when clicked', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const locksTab = screen.getByRole('tab', { name: /Locks/i });
      fireEvent.click(locksTab);

      await waitFor(() => {
        expect(screen.getByText(/Lock Status/i)).toBeInTheDocument();
      });
    });

    it('should switch to Metadata tab when clicked', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const metadataTab = screen.getByRole('tab', { name: /Metadata/i });
      fireEvent.click(metadataTab);

      await waitFor(() => {
        expect(screen.getByText(/Select a commit from the Commits tab/i)).toBeInTheDocument();
      });
    });
  });

  describe('Clone Tab Content', () => {
    it('should pass correct props to CloneInstructions', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      // Check that the clone command contains correct namespace and name
      expect(screen.getByText(/auxin clone.*testuser\/test-project\.logicx/i)).toBeInTheDocument();
    });

    it('should use window.location.origin for server URL', async () => {
      const originalLocation = window.location;
      delete (window as any).location;
      window.location = { ...originalLocation, origin: 'https://auxin.example.com' } as any;

      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      expect(screen.getByText(/https:\/\/auxin\.example\.com/i)).toBeInTheDocument();

      window.location = originalLocation;
    });

    it('should allow toggling between Auxin and Oxen methods', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      // Initially shows auxin command
      expect(screen.getByText(/auxin clone/i)).toBeInTheDocument();

      // Switch to oxen
      const oxenButton = screen.getByRole('button', { name: /Use Oxen CLI to clone/i });
      fireEvent.click(oxenButton);

      await waitFor(() => {
        expect(screen.getByText(/oxen clone/i)).toBeInTheDocument();
      });
    });

    it('should show copy button for clone command', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const copyButtons = screen.getAllByRole('button', { name: /Copy to clipboard/i });
      expect(copyButtons.length).toBeGreaterThan(0);
    });

    it('should show detailed instructions when expanded', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const summary = screen.getByText(/Show detailed clone instructions/i);
      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.getByText(/Prerequisites/i)).toBeInTheDocument();
        expect(screen.getByText(/Step by Step/i)).toBeInTheDocument();
      });
    });
  });

  describe('Tab Persistence', () => {
    it('should remember tab selection when switching between tabs', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      // Switch to Commits tab
      const commitsTab = screen.getByRole('tab', { name: /Commits/i });
      fireEvent.click(commitsTab);

      await waitFor(() => {
        expect(screen.getByText('Initial commit')).toBeInTheDocument();
      });

      // Switch back to Clone tab
      const cloneTab = screen.getByRole('tab', { name: /Clone/i });
      fireEvent.click(cloneTab);

      await waitFor(() => {
        expect(screen.getByText(/Clone this Logic Pro project/i)).toBeInTheDocument();
      });
    });
  });

  describe('Repository Information Display', () => {
    it('should display repository namespace and name', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });
    });

    it('should display repository description', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('A test Logic Pro project')).toBeInTheDocument();
      });
    });

    it('should not display description if not provided', async () => {
      (api.getRepository as any).mockResolvedValue({
        data: { ...mockRepo, description: undefined },
      });

      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      expect(screen.queryByText('A test Logic Pro project')).not.toBeInTheDocument();
    });
  });

  describe('Error Handling', () => {
    it('should show error message when repository fails to load', async () => {
      (api.getRepository as any).mockRejectedValue(new Error('Not found'));

      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText(/Repository not found/i)).toBeInTheDocument();
      });
    });

    it('should show loading state while fetching repository', () => {
      (api.getRepository as any).mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve({ data: mockRepo }), 100))
      );

      renderWithRouter(<RepoPage />);

      expect(screen.getByText(/Loading repository/i)).toBeInTheDocument();
    });
  });

  describe('Back Navigation', () => {
    it('should show back button to repositories list', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const backButton = screen.getByRole('link', { name: /Back to repositories/i });
      expect(backButton).toHaveAttribute('href', '/');
    });
  });

  describe('Tab Icons', () => {
    it('should show Download icon for Clone tab', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const cloneTab = screen.getByRole('tab', { name: /Clone/i });
      const icon = cloneTab.querySelector('svg');
      expect(icon).toBeInTheDocument();
    });

    it('should show GitBranch icon for Commits tab', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const commitsTab = screen.getByRole('tab', { name: /Commits/i });
      const icon = commitsTab.querySelector('svg');
      expect(icon).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA roles for tabs', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      expect(screen.getByRole('tablist')).toBeInTheDocument();
      expect(screen.getAllByRole('tab').length).toBe(4);
    });

    it('should have ARIA selected attribute on active tab', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const cloneTab = screen.getByRole('tab', { name: /Clone/i });
      expect(cloneTab).toHaveAttribute('aria-selected', 'true');
    });

    it('should have ARIA controls attribute on tabs', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const cloneTab = screen.getByRole('tab', { name: /Clone/i });
      expect(cloneTab).toHaveAttribute('aria-controls', 'clone-panel');
    });

    it('should have proper ARIA label for tablist', async () => {
      renderWithRouter(<RepoPage />);

      await waitFor(() => {
        expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      });

      const tablist = screen.getByRole('tablist');
      expect(tablist).toHaveAttribute('aria-label', 'Repository sections');
    });
  });
});
