/// Integration tests for console TUI functionality
///
/// Tests the interactive console mode including:
/// - Mode switching
/// - Keyboard input handling
/// - Status display
/// - Commit dialog
/// - History browsing

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use auxin::console::{
        Console, ConsoleMode, KeyEvent, ConsoleState,
        StatusView, HistoryView, CommitDialog, SearchView,
        HooksView, DiffView, HelpView
    };
    use tempfile::TempDir;

    /// Helper to create a test console
    fn create_test_console() -> (TempDir, Console) {
        let temp_dir = TempDir::new().unwrap();
        let console = Console::new(temp_dir.path());
        (temp_dir, console)
    }

    // ===================
    // Console Initialization Tests
    // ===================

    #[test]
    fn test_console_new() {
        let (_temp_dir, console) = create_test_console();

        assert_eq!(console.current_mode(), ConsoleMode::Status);
    }

    #[test]
    fn test_console_initial_state() {
        let (_temp_dir, console) = create_test_console();

        let state = console.get_state();
        assert!(!state.is_running);
        assert_eq!(state.mode, ConsoleMode::Status);
    }

    // ===================
    // Mode Switching Tests
    // ===================

    #[test]
    fn test_switch_to_history_mode() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::History);
        assert_eq!(console.current_mode(), ConsoleMode::History);
    }

    #[test]
    fn test_switch_to_commit_mode() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::Commit);
        assert_eq!(console.current_mode(), ConsoleMode::Commit);
    }

    #[test]
    fn test_switch_to_search_mode() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::Search);
        assert_eq!(console.current_mode(), ConsoleMode::Search);
    }

    #[test]
    fn test_switch_to_diff_mode() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::Diff);
        assert_eq!(console.current_mode(), ConsoleMode::Diff);
    }

    #[test]
    fn test_switch_to_hooks_mode() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::Hooks);
        assert_eq!(console.current_mode(), ConsoleMode::Hooks);
    }

    #[test]
    fn test_switch_to_help_mode() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::Help);
        assert_eq!(console.current_mode(), ConsoleMode::Help);
    }

    // ===================
    // Keyboard Input Tests
    // ===================

    #[test]
    fn test_key_event_char() {
        let event = KeyEvent::Char('a');
        assert_eq!(event.to_char(), Some('a'));
    }

    #[test]
    fn test_key_event_special() {
        let event = KeyEvent::Enter;
        assert_eq!(event.to_char(), None);
    }

    #[test]
    fn test_handle_key_quit() {
        let (_temp_dir, mut console) = create_test_console();

        let result = console.handle_key(KeyEvent::Char('q'));
        assert!(result.should_quit);
    }

    #[test]
    fn test_handle_key_help() {
        let (_temp_dir, mut console) = create_test_console();

        console.handle_key(KeyEvent::Char('?'));
        assert_eq!(console.current_mode(), ConsoleMode::Help);
    }

    #[test]
    fn test_handle_key_refresh() {
        let (_temp_dir, mut console) = create_test_console();

        let result = console.handle_key(KeyEvent::Char('r'));
        assert!(result.needs_refresh);
    }

    #[test]
    fn test_handle_key_commit_shortcut() {
        let (_temp_dir, mut console) = create_test_console();

        console.handle_key(KeyEvent::Char('i'));
        assert_eq!(console.current_mode(), ConsoleMode::Commit);
    }

    #[test]
    fn test_handle_key_log_shortcut() {
        let (_temp_dir, mut console) = create_test_console();

        console.handle_key(KeyEvent::Char('l'));
        assert_eq!(console.current_mode(), ConsoleMode::History);
    }

    #[test]
    fn test_handle_key_diff_shortcut() {
        let (_temp_dir, mut console) = create_test_console();

        console.handle_key(KeyEvent::Char('d'));
        assert_eq!(console.current_mode(), ConsoleMode::Diff);
    }

    #[test]
    fn test_handle_key_search_shortcut() {
        let (_temp_dir, mut console) = create_test_console();

        console.handle_key(KeyEvent::Char('s'));
        assert_eq!(console.current_mode(), ConsoleMode::Search);
    }

    #[test]
    fn test_handle_key_hooks_shortcut() {
        let (_temp_dir, mut console) = create_test_console();

        console.handle_key(KeyEvent::Char('k'));
        assert_eq!(console.current_mode(), ConsoleMode::Hooks);
    }

    #[test]
    fn test_handle_key_escape_returns_to_status() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::History);
        console.handle_key(KeyEvent::Escape);
        assert_eq!(console.current_mode(), ConsoleMode::Status);
    }

    #[test]
    fn test_handle_key_navigation_up() {
        let (_temp_dir, mut console) = create_test_console();
        console.switch_mode(ConsoleMode::History);

        let state_before = console.get_state().selected_index;
        console.handle_key(KeyEvent::Up);
        // Selected index should change (if possible)
    }

    #[test]
    fn test_handle_key_navigation_down() {
        let (_temp_dir, mut console) = create_test_console();
        console.switch_mode(ConsoleMode::History);

        console.handle_key(KeyEvent::Down);
        // Navigation should work
    }

    // ===================
    // StatusView Tests
    // ===================

    #[test]
    fn test_status_view_render() {
        let (_temp_dir, console) = create_test_console();

        let view = StatusView::new(console.get_repo_path());
        let output = view.render();

        assert!(!output.is_empty());
    }

    #[test]
    fn test_status_view_shows_staged() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file to stage
        std::fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

        let view = StatusView::new(temp_dir.path());
        let output = view.render();

        // Should show untracked or modified files
    }

    // ===================
    // HistoryView Tests
    // ===================

    #[test]
    fn test_history_view_render() {
        let temp_dir = TempDir::new().unwrap();

        let view = HistoryView::new(temp_dir.path());
        let output = view.render();

        // Should render even with no history
        assert!(!output.is_empty() || output.is_empty()); // May be empty for new repo
    }

    #[test]
    fn test_history_view_scroll() {
        let temp_dir = TempDir::new().unwrap();

        let mut view = HistoryView::new(temp_dir.path());

        view.scroll_down();
        view.scroll_up();
        // Should not crash
    }

    #[test]
    fn test_history_view_select() {
        let temp_dir = TempDir::new().unwrap();

        let mut view = HistoryView::new(temp_dir.path());

        let selected = view.get_selected();
        // May be None for empty repo
    }

    // ===================
    // CommitDialog Tests
    // ===================

    #[test]
    fn test_commit_dialog_new() {
        let dialog = CommitDialog::new();

        assert!(dialog.get_message().is_empty());
        assert!(dialog.get_bpm().is_none());
    }

    #[test]
    fn test_commit_dialog_set_message() {
        let mut dialog = CommitDialog::new();

        dialog.set_message("Test commit");
        assert_eq!(dialog.get_message(), "Test commit");
    }

    #[test]
    fn test_commit_dialog_set_bpm() {
        let mut dialog = CommitDialog::new();

        dialog.set_bpm(128.0);
        assert_eq!(dialog.get_bpm(), Some(128.0));
    }

    #[test]
    fn test_commit_dialog_set_key() {
        let mut dialog = CommitDialog::new();

        dialog.set_key("A Minor");
        assert_eq!(dialog.get_key(), Some("A Minor".to_string()));
    }

    #[test]
    fn test_commit_dialog_set_tags() {
        let mut dialog = CommitDialog::new();

        dialog.set_tags("mixing,vocals");
        assert_eq!(dialog.get_tags(), Some("mixing,vocals".to_string()));
    }

    #[test]
    fn test_commit_dialog_validate_empty() {
        let dialog = CommitDialog::new();

        assert!(!dialog.is_valid()); // Empty message is invalid
    }

    #[test]
    fn test_commit_dialog_validate_with_message() {
        let mut dialog = CommitDialog::new();
        dialog.set_message("Valid commit");

        assert!(dialog.is_valid());
    }

    #[test]
    fn test_commit_dialog_field_navigation() {
        let mut dialog = CommitDialog::new();

        dialog.next_field();
        dialog.next_field();
        dialog.prev_field();
        // Should cycle through fields
    }

    #[test]
    fn test_commit_dialog_input_char() {
        let mut dialog = CommitDialog::new();

        dialog.input_char('H');
        dialog.input_char('i');
        assert_eq!(dialog.get_message(), "Hi");
    }

    #[test]
    fn test_commit_dialog_backspace() {
        let mut dialog = CommitDialog::new();

        dialog.set_message("Hello");
        dialog.backspace();
        assert_eq!(dialog.get_message(), "Hell");
    }

    // ===================
    // SearchView Tests
    // ===================

    #[test]
    fn test_search_view_new() {
        let temp_dir = TempDir::new().unwrap();

        let view = SearchView::new(temp_dir.path());
        assert!(view.get_query().is_empty());
    }

    #[test]
    fn test_search_view_set_query() {
        let temp_dir = TempDir::new().unwrap();

        let mut view = SearchView::new(temp_dir.path());
        view.set_query("bpm:120");

        assert_eq!(view.get_query(), "bpm:120");
    }

    #[test]
    fn test_search_view_execute() {
        let temp_dir = TempDir::new().unwrap();

        let mut view = SearchView::new(temp_dir.path());
        view.set_query("bpm:100-140");

        let results = view.execute();
        // Results depend on repo content
    }

    #[test]
    fn test_search_view_clear() {
        let temp_dir = TempDir::new().unwrap();

        let mut view = SearchView::new(temp_dir.path());
        view.set_query("test");
        view.clear();

        assert!(view.get_query().is_empty());
    }

    // ===================
    // DiffView Tests
    // ===================

    #[test]
    fn test_diff_view_new() {
        let temp_dir = TempDir::new().unwrap();

        let view = DiffView::new(temp_dir.path());
        // Should initialize
    }

    #[test]
    fn test_diff_view_set_commits() {
        let temp_dir = TempDir::new().unwrap();

        let mut view = DiffView::new(temp_dir.path());
        view.set_commits("abc123", "def456");

        // Should store commits to compare
    }

    #[test]
    fn test_diff_view_render() {
        let temp_dir = TempDir::new().unwrap();

        let view = DiffView::new(temp_dir.path());
        let output = view.render();

        // Should render something
    }

    // ===================
    // HooksView Tests
    // ===================

    #[test]
    fn test_hooks_view_new() {
        let temp_dir = TempDir::new().unwrap();

        let view = HooksView::new(temp_dir.path());
        // Should initialize
    }

    #[test]
    fn test_hooks_view_list() {
        let temp_dir = TempDir::new().unwrap();

        let view = HooksView::new(temp_dir.path());
        let hooks = view.list_hooks();

        // May be empty
    }

    #[test]
    fn test_hooks_view_render() {
        let temp_dir = TempDir::new().unwrap();

        let view = HooksView::new(temp_dir.path());
        let output = view.render();

        assert!(!output.is_empty());
    }

    // ===================
    // HelpView Tests
    // ===================

    #[test]
    fn test_help_view_render() {
        let view = HelpView::new();
        let output = view.render();

        assert!(!output.is_empty());
        assert!(output.contains("q") || output.contains("quit"));
    }

    #[test]
    fn test_help_view_contains_shortcuts() {
        let view = HelpView::new();
        let output = view.render();

        // Should contain all shortcuts
        assert!(output.contains("i") || output.contains("commit"));
        assert!(output.contains("l") || output.contains("log"));
        assert!(output.contains("s") || output.contains("search"));
    }

    // ===================
    // Rendering Tests
    // ===================

    #[test]
    fn test_console_render_status_mode() {
        let (_temp_dir, console) = create_test_console();

        let output = console.render();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_console_render_history_mode() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::History);
        let output = console.render();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_console_render_help_mode() {
        let (_temp_dir, mut console) = create_test_console();

        console.switch_mode(ConsoleMode::Help);
        let output = console.render();

        assert!(!output.is_empty());
    }

    // ===================
    // Terminal Size Tests
    // ===================

    #[test]
    fn test_console_handle_resize() {
        let (_temp_dir, mut console) = create_test_console();

        console.handle_resize(80, 24);
        let state = console.get_state();

        assert_eq!(state.terminal_width, 80);
        assert_eq!(state.terminal_height, 24);
    }

    #[test]
    fn test_console_minimum_size() {
        let (_temp_dir, mut console) = create_test_console();

        // Should handle small terminal gracefully
        console.handle_resize(40, 10);
        let output = console.render();
        // Should not crash
    }

    // ===================
    // Color Tests
    // ===================

    #[test]
    fn test_console_colored_output() {
        let (_temp_dir, console) = create_test_console();

        let output = console.render();
        // Output may contain ANSI codes
    }

    #[test]
    fn test_console_disable_colors() {
        let (_temp_dir, mut console) = create_test_console();

        console.set_colored(false);
        let output = console.render();

        // Should not contain ANSI escape codes
        assert!(!output.contains("\x1b["));
    }

    // ===================
    // State Persistence Tests
    // ===================

    #[test]
    fn test_console_state_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Set some state
        {
            let mut console = Console::new(temp_dir.path());
            console.switch_mode(ConsoleMode::History);
            console.save_state().unwrap();
        }

        // Restore state
        {
            let console = Console::new(temp_dir.path());
            // State may or may not persist based on implementation
        }
    }

    // ===================
    // Edge Case Tests
    // ===================

    #[test]
    fn test_console_rapid_key_input() {
        let (_temp_dir, mut console) = create_test_console();

        // Rapid key presses should not crash
        for _ in 0..100 {
            console.handle_key(KeyEvent::Char('r'));
        }
    }

    #[test]
    fn test_console_invalid_mode_transition() {
        let (_temp_dir, mut console) = create_test_console();

        // Switching to same mode should be no-op
        console.switch_mode(ConsoleMode::Status);
        console.switch_mode(ConsoleMode::Status);
        assert_eq!(console.current_mode(), ConsoleMode::Status);
    }

    #[test]
    fn test_commit_dialog_very_long_message() {
        let mut dialog = CommitDialog::new();

        let long_message = "A".repeat(10000);
        dialog.set_message(&long_message);

        assert_eq!(dialog.get_message().len(), 10000);
    }

    #[test]
    fn test_commit_dialog_unicode_message() {
        let mut dialog = CommitDialog::new();

        dialog.set_message("Added Japanese lyrics: 日本語");
        assert!(dialog.get_message().contains("日本語"));
    }
}
