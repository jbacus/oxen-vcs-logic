import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CloneInstructions } from './CloneInstructions';

describe('CloneInstructions', () => {
  const defaultProps = {
    namespace: 'testuser',
    name: 'test-project.logicx',
    serverUrl: 'http://localhost:3000',
  };

  // Mock clipboard API
  const mockClipboard = {
    writeText: vi.fn(() => Promise.resolve()),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    Object.assign(navigator, {
      clipboard: mockClipboard,
    });
  });

  describe('Rendering', () => {
    it('should render the component with default auxin method', () => {
      render(<CloneInstructions {...defaultProps} />);

      expect(screen.getByText(/Clone this Logic Pro project/i)).toBeInTheDocument();
      expect(screen.getByText(/Auxin CLI \(Recommended\)/i)).toBeInTheDocument();
      expect(screen.getByText(/Oxen CLI/i)).toBeInTheDocument();
    });

    it('should detect Logic Pro project type from .logicx extension', () => {
      render(<CloneInstructions {...defaultProps} />);

      expect(screen.getByText(/Clone this Logic Pro project/i)).toBeInTheDocument();
    });

    it('should detect SketchUp project type from .skp extension', () => {
      render(<CloneInstructions {...defaultProps} name="model.skp" />);

      expect(screen.getByText(/Clone this SketchUp project/i)).toBeInTheDocument();
    });

    it('should detect Blender project type from .blend extension', () => {
      render(<CloneInstructions {...defaultProps} name="scene.blend" />);

      expect(screen.getByText(/Clone this Blender project/i)).toBeInTheDocument();
    });

    it('should use generic project type for unknown extensions', () => {
      render(<CloneInstructions {...defaultProps} name="generic-repo" />);

      expect(screen.getByText(/Clone this project project/i)).toBeInTheDocument();
    });

    it('should display auxin clone command by default', () => {
      render(<CloneInstructions {...defaultProps} />);

      const expectedCommand = `auxin clone ${defaultProps.serverUrl}/${defaultProps.namespace}/${defaultProps.name} ${defaultProps.name}`;
      expect(screen.getByText(expectedCommand)).toBeInTheDocument();
    });

    it('should use provided serverUrl', () => {
      const customUrl = 'https://auxin.example.com';
      render(<CloneInstructions {...defaultProps} serverUrl={customUrl} />);

      expect(screen.getByText(new RegExp(customUrl))).toBeInTheDocument();
    });
  });

  describe('Method Toggle', () => {
    it('should switch to oxen method when oxen button is clicked', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const oxenButton = screen.getByRole('button', { name: /Use Oxen CLI to clone/i });
      fireEvent.click(oxenButton);

      await waitFor(() => {
        const expectedCommand = `oxen clone ${defaultProps.serverUrl}/${defaultProps.namespace}/${defaultProps.name} ${defaultProps.name}`;
        expect(screen.getByText(expectedCommand)).toBeInTheDocument();
      });
    });

    it('should toggle back to auxin method', async () => {
      render(<CloneInstructions {...defaultProps} />);

      // Switch to oxen
      const oxenButton = screen.getByRole('button', { name: /Use Oxen CLI to clone/i });
      fireEvent.click(oxenButton);

      // Switch back to auxin
      const auxinButton = screen.getByRole('button', { name: /Use Auxin CLI to clone/i });
      fireEvent.click(auxinButton);

      await waitFor(() => {
        const expectedCommand = `auxin clone ${defaultProps.serverUrl}/${defaultProps.namespace}/${defaultProps.name} ${defaultProps.name}`;
        expect(screen.getByText(expectedCommand)).toBeInTheDocument();
      });
    });

    it('should apply active styles to selected method button', () => {
      render(<CloneInstructions {...defaultProps} />);

      const auxinButton = screen.getByRole('button', { name: /Use Auxin CLI to clone/i });
      expect(auxinButton).toHaveClass('bg-primary-600', 'text-white');
    });
  });

  describe('Copy to Clipboard', () => {
    it('should copy auxin clone command to clipboard', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const copyButtons = screen.getAllByRole('button', { name: /Copy to clipboard/i });
      fireEvent.click(copyButtons[0]);

      await waitFor(() => {
        const expectedCommand = `auxin clone ${defaultProps.serverUrl}/${defaultProps.namespace}/${defaultProps.name} ${defaultProps.name}`;
        expect(mockClipboard.writeText).toHaveBeenCalledWith(expectedCommand);
      });
    });

    it('should copy oxen clone command when oxen method is selected', async () => {
      render(<CloneInstructions {...defaultProps} />);

      // Switch to oxen method
      const oxenButton = screen.getByRole('button', { name: /Use Oxen CLI to clone/i });
      fireEvent.click(oxenButton);

      const copyButtons = screen.getAllByRole('button', { name: /Copy to clipboard/i });
      fireEvent.click(copyButtons[0]);

      await waitFor(() => {
        const expectedCommand = `oxen clone ${defaultProps.serverUrl}/${defaultProps.namespace}/${defaultProps.name} ${defaultProps.name}`;
        expect(mockClipboard.writeText).toHaveBeenCalledWith(expectedCommand);
      });
    });

    it('should handle clipboard write failure gracefully', async () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      mockClipboard.writeText.mockRejectedValueOnce(new Error('Clipboard error'));

      render(<CloneInstructions {...defaultProps} />);

      const copyButtons = screen.getAllByRole('button', { name: /Copy to clipboard/i });
      fireEvent.click(copyButtons[0]);

      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to copy:', expect.any(Error));
      });

      consoleErrorSpy.mockRestore();
    });
  });

  describe('Detailed Instructions', () => {
    it('should expand detailed instructions when summary is clicked', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const summary = screen.getByText(/Show detailed clone instructions/i);
      expect(screen.queryByText(/Prerequisites/i)).not.toBeInTheDocument();

      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.getByText(/Prerequisites/i)).toBeInTheDocument();
        expect(screen.getByText(/Step by Step/i)).toBeInTheDocument();
      });
    });

    it('should show auxin-specific prerequisites when auxin method is selected', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const summary = screen.getByText(/Show detailed clone instructions/i);
      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.getByText(/Install Auxin CLI: Follow the installation guide/i)).toBeInTheDocument();
      });
    });

    it('should show oxen prerequisites when oxen method is selected', async () => {
      render(<CloneInstructions {...defaultProps} />);

      // Switch to oxen
      const oxenButton = screen.getByRole('button', { name: /Use Oxen CLI to clone/i });
      fireEvent.click(oxenButton);

      const summary = screen.getByText(/Show detailed clone instructions/i);
      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.getByText(/pip install oxen-ai/i)).toBeInTheDocument();
      });
    });

    it('should display alternative URLs section', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const summary = screen.getByText(/Show detailed clone instructions/i);
      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.getByText(/Alternative Clone URLs/i)).toBeInTheDocument();
        expect(screen.getByText(/HTTP URL \(current\):/i)).toBeInTheDocument();
        expect(screen.getByText(/File URL \(for local network shares\):/i)).toBeInTheDocument();
      });
    });

    it('should show auxin post-clone instructions', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const summary = screen.getByText(/Show detailed clone instructions/i);
      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.getByText(/After Cloning/i)).toBeInTheDocument();
        expect(screen.getByText(/auxin log/i)).toBeInTheDocument();
        expect(screen.getByText(/auxin status/i)).toBeInTheDocument();
        expect(screen.getByText(/auxin lock acquire/i)).toBeInTheDocument();
      });
    });

    it('should not show post-clone instructions for oxen method', async () => {
      render(<CloneInstructions {...defaultProps} />);

      // Switch to oxen
      const oxenButton = screen.getByRole('button', { name: /Use Oxen CLI to clone/i });
      fireEvent.click(oxenButton);

      const summary = screen.getByText(/Show detailed clone instructions/i);
      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.queryByText(/After Cloning/i)).not.toBeInTheDocument();
      });
    });
  });

  describe('Info Message', () => {
    it('should show info message for auxin method', () => {
      render(<CloneInstructions {...defaultProps} />);

      expect(screen.getByText(/Auxin CLI provides enhanced features/i)).toBeInTheDocument();
    });

    it('should hide info message for oxen method', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const oxenButton = screen.getByRole('button', { name: /Use Oxen CLI to clone/i });
      fireEvent.click(oxenButton);

      await waitFor(() => {
        expect(screen.queryByText(/Auxin CLI provides enhanced features/i)).not.toBeInTheDocument();
      });
    });

    it('should mention project type in info message', () => {
      render(<CloneInstructions {...defaultProps} />);

      expect(screen.getByText(/for Logic Pro projects/i)).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels for buttons', () => {
      render(<CloneInstructions {...defaultProps} />);

      expect(screen.getByRole('button', { name: /Use Auxin CLI to clone/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Use Oxen CLI to clone/i })).toBeInTheDocument();
    });

    it('should have ARIA label for clone command', () => {
      render(<CloneInstructions {...defaultProps} />);

      const codeElements = screen.getAllByLabelText(/Clone command/i);
      expect(codeElements.length).toBeGreaterThan(0);
    });

    it('should have proper heading structure', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const summary = screen.getByText(/Show detailed clone instructions/i);
      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.getByText(/Prerequisites/i)).toBeInTheDocument();
        expect(screen.getByText(/Step by Step/i)).toBeInTheDocument();
        expect(screen.getByText(/Alternative Clone URLs/i)).toBeInTheDocument();
      });
    });
  });

  describe('URL Generation', () => {
    it('should generate correct HTTP URL', () => {
      render(<CloneInstructions {...defaultProps} />);

      const expectedUrl = `${defaultProps.serverUrl}/${defaultProps.namespace}/${defaultProps.name}`;
      expect(screen.getByText(new RegExp(expectedUrl))).toBeInTheDocument();
    });

    it('should handle serverUrl with trailing slash', () => {
      render(<CloneInstructions {...defaultProps} serverUrl="http://localhost:3000/" />);

      // Should not have double slashes
      expect(screen.queryByText(/\/\//)).not.toBeInTheDocument();
    });

    it('should show file URL pattern in detailed instructions', async () => {
      render(<CloneInstructions {...defaultProps} />);

      const summary = screen.getByText(/Show detailed clone instructions/i);
      fireEvent.click(summary);

      await waitFor(() => {
        expect(screen.getByText(/file:\/\/\/path\/to\/server\/repos/i)).toBeInTheDocument();
      });
    });
  });
});
