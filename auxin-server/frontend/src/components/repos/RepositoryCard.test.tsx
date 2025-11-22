import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { RepositoryCard } from './RepositoryCard';
import type { Repository } from '@/types';

// Wrapper for components that use react-router
const RouterWrapper = ({ children }: { children: React.ReactNode }) => (
  <BrowserRouter>{children}</BrowserRouter>
);

describe('RepositoryCard', () => {
  const mockRepo: Repository = {
    namespace: 'testuser',
    name: 'test-project.logicx',
    path: '/repos/testuser/test-project.logicx',
    description: 'A test Logic Pro project',
  };

  // Mock clipboard API
  const mockClipboard = {
    writeText: vi.fn(() => Promise.resolve()),
  };

  // Mock window.location.origin
  const originalLocation = window.location;

  beforeEach(() => {
    vi.clearAllMocks();
    Object.assign(navigator, {
      clipboard: mockClipboard,
    });

    // Mock window.location.origin
    delete (window as any).location;
    window.location = { ...originalLocation, origin: 'http://localhost:3000' } as any;
  });

  afterEach(() => {
    window.location = originalLocation;
  });

  describe('Rendering', () => {
    it('should render repository card with basic information', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      expect(screen.getByText('A test Logic Pro project')).toBeInTheDocument();
      expect(screen.getByText('/repos/testuser/test-project.logicx')).toBeInTheDocument();
    });

    it('should render repository card without description', () => {
      const repoWithoutDesc = { ...mockRepo, description: undefined };
      render(<RepositoryCard repo={repoWithoutDesc} />, { wrapper: RouterWrapper });

      expect(screen.getByText('testuser/test-project.logicx')).toBeInTheDocument();
      expect(screen.queryByText('A test Logic Pro project')).not.toBeInTheDocument();
    });

    it('should render download/clone button', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      expect(screen.getByRole('button', { name: /Show clone command/i })).toBeInTheDocument();
    });

    it('should render link to repository page', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const link = screen.getByRole('link', { name: /View repository/i });
      expect(link).toHaveAttribute('href', '/testuser/test-project.logicx');
    });

    it('should render folder icon', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const icon = screen.getByLabelText(/View repository/i).querySelector('svg');
      expect(icon).toBeInTheDocument();
    });
  });

  describe('Clone Tooltip', () => {
    it('should not show clone tooltip by default', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      expect(screen.queryByText(/Clone command:/i)).not.toBeInTheDocument();
    });

    it('should show clone tooltip when download button is clicked', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        expect(screen.getByText(/Clone command:/i)).toBeInTheDocument();
      });
    });

    it('should display correct clone command in tooltip', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        const expectedCommand = 'auxin clone http://localhost:3000/testuser/test-project.logicx test-project.logicx';
        expect(screen.getByText(expectedCommand)).toBeInTheDocument();
      });
    });

    it('should hide clone tooltip when close button is clicked', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      // Open tooltip
      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        expect(screen.getByText(/Clone command:/i)).toBeInTheDocument();
      });

      // Close tooltip
      const closeButton = screen.getByRole('button', { name: /Close clone tooltip/i });
      fireEvent.click(closeButton);

      await waitFor(() => {
        expect(screen.queryByText(/Clone command:/i)).not.toBeInTheDocument();
      });
    });

    it('should toggle tooltip visibility on multiple clicks', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });

      // Open
      fireEvent.click(downloadButton);
      await waitFor(() => {
        expect(screen.getByText(/Clone command:/i)).toBeInTheDocument();
      });

      // Close
      fireEvent.click(downloadButton);
      await waitFor(() => {
        expect(screen.queryByText(/Clone command:/i)).not.toBeInTheDocument();
      });

      // Open again
      fireEvent.click(downloadButton);
      await waitFor(() => {
        expect(screen.getByText(/Clone command:/i)).toBeInTheDocument();
      });
    });

    it('should show hint about more clone options', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        expect(screen.getByText(/Click the repository for more clone options/i)).toBeInTheDocument();
      });
    });
  });

  describe('Copy to Clipboard', () => {
    it('should copy clone command to clipboard', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      // Open tooltip
      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        expect(screen.getByText(/Clone command:/i)).toBeInTheDocument();
      });

      // Click copy button
      const copyButton = screen.getByRole('button', { name: /Copy to clipboard/i });
      fireEvent.click(copyButton);

      await waitFor(() => {
        const expectedCommand = 'auxin clone http://localhost:3000/testuser/test-project.logicx test-project.logicx';
        expect(mockClipboard.writeText).toHaveBeenCalledWith(expectedCommand);
      });
    });

    it('should show checkmark after successful copy', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      // Open tooltip
      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      // Click copy button
      const copyButton = screen.getByRole('button', { name: /Copy to clipboard/i });
      fireEvent.click(copyButton);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /Copied to clipboard/i })).toBeInTheDocument();
      });
    });

    it('should auto-close tooltip after successful copy', async () => {
      vi.useFakeTimers();

      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      // Open tooltip
      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      // Click copy button
      const copyButton = screen.getByRole('button', { name: /Copy to clipboard/i });
      fireEvent.click(copyButton);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /Copied to clipboard/i })).toBeInTheDocument();
      });

      // Fast-forward 2 seconds
      vi.advanceTimersByTime(2000);

      await waitFor(() => {
        expect(screen.queryByText(/Clone command:/i)).not.toBeInTheDocument();
      });

      vi.useRealTimers();
    });

    it('should handle clipboard write failure gracefully', async () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      mockClipboard.writeText.mockRejectedValueOnce(new Error('Clipboard error'));

      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      // Open tooltip
      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      // Click copy button
      const copyButton = screen.getByRole('button', { name: /Copy to clipboard/i });
      fireEvent.click(copyButton);

      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to copy:', expect.any(Error));
      });

      consoleErrorSpy.mockRestore();
    });
  });

  describe('Event Handling', () => {
    it('should prevent navigation when clicking download button', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      const clickEvent = new MouseEvent('click', { bubbles: true });
      const preventDefaultSpy = vi.spyOn(clickEvent, 'preventDefault');
      const stopPropagationSpy = vi.spyOn(clickEvent, 'stopPropagation');

      downloadButton.dispatchEvent(clickEvent);

      expect(preventDefaultSpy).toHaveBeenCalled();
      expect(stopPropagationSpy).toHaveBeenCalled();
    });

    it('should prevent navigation when clicking copy button', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      // Open tooltip
      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        expect(screen.getByText(/Clone command:/i)).toBeInTheDocument();
      });

      const copyButton = screen.getByRole('button', { name: /Copy to clipboard/i });
      const clickEvent = new MouseEvent('click', { bubbles: true });
      const preventDefaultSpy = vi.spyOn(clickEvent, 'preventDefault');
      const stopPropagationSpy = vi.spyOn(clickEvent, 'stopPropagation');

      copyButton.dispatchEvent(clickEvent);

      expect(preventDefaultSpy).toHaveBeenCalled();
      expect(stopPropagationSpy).toHaveBeenCalled();
    });
  });

  describe('URL Generation', () => {
    it('should use window.location.origin for clone URL', async () => {
      window.location = { ...originalLocation, origin: 'https://auxin.example.com' } as any;

      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        expect(screen.getByText(/https:\/\/auxin\.example\.com/i)).toBeInTheDocument();
      });
    });

    it('should include full path in clone command', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        expect(screen.getByText(/testuser\/test-project\.logicx/i)).toBeInTheDocument();
      });
    });

    it('should use project name as destination', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        const command = screen.getByText(/auxin clone/i).textContent;
        expect(command).toContain('test-project.logicx');
      });
    });
  });

  describe('Styling', () => {
    it('should have hover effects on card', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const link = screen.getByRole('link', { name: /View repository/i });
      expect(link).toHaveClass('hover:shadow-md', 'hover:border-primary-200');
    });

    it('should have transition classes', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const link = screen.getByRole('link', { name: /View repository/i });
      expect(link).toHaveClass('transition-all', 'duration-200');
    });

    it('should apply primary color to download button', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      expect(downloadButton).toHaveClass('text-primary-600');
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels for buttons', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      expect(screen.getByLabelText(/View repository/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Show clone command/i)).toBeInTheDocument();
    });

    it('should have ARIA hidden on icons', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const icons = screen.getAllByRole('link')[0].querySelectorAll('[aria-hidden="true"]');
      expect(icons.length).toBeGreaterThan(0);
    });

    it('should have proper title attribute on download button', () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      expect(downloadButton).toHaveAttribute('title', 'Clone repository');
    });

    it('should show tooltip with proper ARIA labels', async () => {
      render(<RepositoryCard repo={mockRepo} />, { wrapper: RouterWrapper });

      const downloadButton = screen.getByRole('button', { name: /Show clone command/i });
      fireEvent.click(downloadButton);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /Close clone tooltip/i })).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /Copy to clipboard/i })).toBeInTheDocument();
      });
    });
  });
});
