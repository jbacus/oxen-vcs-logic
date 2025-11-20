/// Integration tests for console TUI functionality
///
/// Tests the interactive console mode including:
/// - Mode switching
/// - Keyboard input handling
/// - Status display
/// - Console state management

#[cfg(test)]
mod tests {
    use auxin::{Console, ConsoleMode, DaemonStatus, LogLevel};
    use std::path::PathBuf;

    // ===================
    // Console Initialization Tests
    // ===================

    #[test]
    fn test_console_creation() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        // Console should initialize with empty activity log and unknown daemon status
        assert_eq!(console.activity_log.len(), 0);
        assert_eq!(console.daemon_status, DaemonStatus::Unknown);
    }

    #[test]
    fn test_console_initial_mode_is_normal() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        assert_eq!(console.mode, ConsoleMode::Normal);
    }

    #[test]
    fn test_console_should_not_quit_initially() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        assert!(!console.should_quit);
    }

    #[test]
    fn test_console_no_repo_status_initially() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        assert!(console.repo_status.is_none());
    }

    // ===================
    // Logging Tests
    // ===================

    #[test]
    fn test_console_log_entry() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.log(LogLevel::Info, "Test message");

        assert_eq!(console.activity_log.len(), 1);
        assert_eq!(console.activity_log[0].message, "Test message");
        assert_eq!(console.activity_log[0].level, LogLevel::Info);
    }

    #[test]
    fn test_console_log_multiple_entries() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.log(LogLevel::Info, "First message");
        console.log(LogLevel::Success, "Second message");
        console.log(LogLevel::Warning, "Third message");
        console.log(LogLevel::Error, "Fourth message");

        assert_eq!(console.activity_log.len(), 4);
        // Most recent should be first
        assert_eq!(console.activity_log[0].message, "Fourth message");
        assert_eq!(console.activity_log[3].message, "First message");
    }

    #[test]
    fn test_console_log_levels() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.log(LogLevel::Info, "Info");
        console.log(LogLevel::Success, "Success");
        console.log(LogLevel::Warning, "Warning");
        console.log(LogLevel::Error, "Error");

        assert_eq!(console.activity_log[3].level, LogLevel::Info);
        assert_eq!(console.activity_log[2].level, LogLevel::Success);
        assert_eq!(console.activity_log[1].level, LogLevel::Warning);
        assert_eq!(console.activity_log[0].level, LogLevel::Error);
    }

    #[test]
    fn test_console_log_pruning() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        // Add more than MAX_LOG_ENTRIES (100)
        for i in 0..150 {
            console.log(LogLevel::Info, format!("Message {}", i));
        }

        // Should be pruned to MAX_LOG_ENTRIES
        assert_eq!(console.activity_log.len(), 100);
        // Most recent should be "Message 149"
        assert_eq!(console.activity_log[0].message, "Message 149");
    }

    // ===================
    // Daemon Status Tests
    // ===================

    #[test]
    fn test_daemon_status_update_running() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.set_daemon_status(DaemonStatus::Running);

        assert_eq!(console.daemon_status, DaemonStatus::Running);
        // Should log the status change
        assert!(!console.activity_log.is_empty());
    }

    #[test]
    fn test_daemon_status_update_stopped() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.set_daemon_status(DaemonStatus::Stopped);

        assert_eq!(console.daemon_status, DaemonStatus::Stopped);
    }

    #[test]
    fn test_daemon_status_no_duplicate_logs() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.set_daemon_status(DaemonStatus::Running);
        let log_count_after_first = console.activity_log.len();

        // Setting to same status shouldn't add new log entry
        console.set_daemon_status(DaemonStatus::Running);

        assert_eq!(console.activity_log.len(), log_count_after_first);
    }

    #[test]
    fn test_daemon_status_transitions_log() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.set_daemon_status(DaemonStatus::Running);
        console.set_daemon_status(DaemonStatus::Stopped);
        console.set_daemon_status(DaemonStatus::Running);

        // Each transition should log
        assert!(console.activity_log.len() >= 3);
    }

    // ===================
    // Repository Status Tests
    // ===================

    #[test]
    fn test_repo_status_update() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.set_repo_status(2, 3, 1);

        assert!(console.repo_status.is_some());
        let status = console.repo_status.as_ref().unwrap();
        assert_eq!(status.staged, 2);
        assert_eq!(status.modified, 3);
        assert_eq!(status.untracked, 1);
    }

    #[test]
    fn test_repo_status_zero_values() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.set_repo_status(0, 0, 0);

        assert!(console.repo_status.is_some());
        let status = console.repo_status.as_ref().unwrap();
        assert_eq!(status.staged, 0);
        assert_eq!(status.modified, 0);
        assert_eq!(status.untracked, 0);
    }

    #[test]
    fn test_repo_status_large_values() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.set_repo_status(1000, 5000, 10000);

        let status = console.repo_status.as_ref().unwrap();
        assert_eq!(status.staged, 1000);
        assert_eq!(status.modified, 5000);
        assert_eq!(status.untracked, 10000);
    }

    // ===================
    // ConsoleMode Tests
    // ===================

    #[test]
    fn test_console_mode_enum_values() {
        // Ensure all modes exist
        let _ = ConsoleMode::Normal;
        let _ = ConsoleMode::CommitDialog;
        let _ = ConsoleMode::RestoreBrowser;
        let _ = ConsoleMode::Compare;
        let _ = ConsoleMode::Search;
        let _ = ConsoleMode::Hooks;
        let _ = ConsoleMode::Help;
    }

    #[test]
    fn test_console_mode_equality() {
        assert_eq!(ConsoleMode::Normal, ConsoleMode::Normal);
        assert_eq!(ConsoleMode::Help, ConsoleMode::Help);
        assert_ne!(ConsoleMode::Normal, ConsoleMode::Help);
    }

    #[test]
    fn test_console_mode_copy() {
        let mode = ConsoleMode::Search;
        let mode_copy = mode; // Copy
        assert_eq!(mode, mode_copy);
    }

    // ===================
    // DaemonStatus Tests
    // ===================

    #[test]
    fn test_daemon_status_enum_values() {
        let _ = DaemonStatus::Running;
        let _ = DaemonStatus::Stopped;
        let _ = DaemonStatus::Unknown;
    }

    #[test]
    fn test_daemon_status_equality() {
        assert_eq!(DaemonStatus::Running, DaemonStatus::Running);
        assert_ne!(DaemonStatus::Running, DaemonStatus::Stopped);
    }

    // ===================
    // LogLevel Tests
    // ===================

    #[test]
    fn test_log_level_enum_values() {
        let _ = LogLevel::Info;
        let _ = LogLevel::Success;
        let _ = LogLevel::Warning;
        let _ = LogLevel::Error;
    }

    #[test]
    fn test_log_level_equality() {
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Error);
    }

    // ===================
    // Edge Case Tests
    // ===================

    #[test]
    fn test_console_with_empty_path() {
        let console = Console::new(PathBuf::from(""));
        assert!(!console.should_quit);
    }

    #[test]
    fn test_console_with_unicode_path() {
        let console = Console::new(PathBuf::from("/projects/日本語/プロジェクト.logicx"));
        assert!(console.repo_status.is_none());
    }

    #[test]
    fn test_log_entry_with_empty_message() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.log(LogLevel::Info, "");
        assert_eq!(console.activity_log[0].message, "");
    }

    #[test]
    fn test_log_entry_with_unicode_message() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.log(LogLevel::Info, "日本語メッセージ 🎵");
        assert!(console.activity_log[0].message.contains("日本語"));
    }

    #[test]
    fn test_log_entry_with_very_long_message() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        let long_message = "A".repeat(10000);
        console.log(LogLevel::Info, &long_message);
        assert_eq!(console.activity_log[0].message.len(), 10000);
    }

    #[test]
    fn test_multiple_repo_status_updates() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        console.set_repo_status(1, 1, 1);
        console.set_repo_status(2, 2, 2);
        console.set_repo_status(3, 3, 3);

        // Should reflect latest values
        let status = console.repo_status.as_ref().unwrap();
        assert_eq!(status.staged, 3);
        assert_eq!(status.modified, 3);
        assert_eq!(status.untracked, 3);
    }

    // ===================
    // Field Access Tests
    // ===================

    #[test]
    fn test_console_activity_log_is_vec() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        let log_count = console.activity_log.len();
        assert_eq!(log_count, 0);
    }

    #[test]
    fn test_log_entry_has_timestamp() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.log(LogLevel::Info, "Test");

        // LogEntry should have a timestamp field
        let _timestamp = console.activity_log[0].timestamp;
    }
}
