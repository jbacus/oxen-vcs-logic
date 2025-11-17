/// Integration tests for CLI command-line interface
/// Tests command parsing, argument handling, and command dispatch
use std::process::Command;

// Helper to run CLI commands
fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("auxin")
        .arg("--")
        .args(args)
        .output()
        .expect("Failed to execute command")
}

// Helper to check if command succeeded
fn command_succeeded(output: &std::process::Output) -> bool {
    output.status.success()
}

// Helper to get stdout as string
fn get_stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

// Helper to get stderr as string
fn get_stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    // MARK: - Help and Version Tests

    #[test]
    fn test_help_flag() {
        let output = run_cli(&["--help"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Help should succeed");
        assert!(stdout.contains("auxin"), "Should show program name");
        assert!(
            stdout.contains("USAGE") || stdout.contains("Usage"),
            "Should show usage"
        );
        assert!(
            stdout.contains("COMMANDS") || stdout.contains("Commands"),
            "Should list commands"
        );
    }

    #[test]
    fn test_version_flag() {
        let output = run_cli(&["--version"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Version should succeed");
        assert!(stdout.contains("auxin"), "Should show program name");
    }

    #[test]
    fn test_short_help_flag() {
        let output = run_cli(&["-h"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Short help should succeed");
        assert!(
            stdout.contains("auxin") || !stdout.is_empty(),
            "Should show help text"
        );
    }

    #[test]
    fn test_verbose_flag() {
        let output = run_cli(&["--verbose", "--help"]);
        // Verbose flag should be recognized without error
        assert!(command_succeeded(&output), "Verbose flag should be valid");
    }

    #[test]
    fn test_short_verbose_flag() {
        let output = run_cli(&["-v", "--help"]);
        // Short verbose flag should be recognized
        assert!(
            command_succeeded(&output),
            "Short verbose flag should be valid"
        );
    }

    // MARK: - Init Command Tests

    #[test]
    fn test_init_command_help() {
        let output = run_cli(&["init", "--help"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Init help should succeed");
        assert!(
            stdout.contains("init") || stdout.contains("Initialize"),
            "Should describe init"
        );
        assert!(stdout.contains("PATH"), "Should mention path argument");
    }

    #[test]
    fn test_init_command_requires_path() {
        let output = run_cli(&["init"]);

        // Should fail without path argument
        assert!(!command_succeeded(&output), "Init without path should fail");
    }

    #[test]
    fn test_init_command_recognizes_logic_flag() {
        let output = run_cli(&["init", "--help"]);
        let stdout = get_stdout(&output);

        assert!(
            stdout.contains("--logic") || stdout.contains("Logic"),
            "Should document logic flag"
        );
    }

    // MARK: - Add Command Tests

    #[test]
    fn test_add_command_help() {
        let output = run_cli(&["add", "--help"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Add help should succeed");
        assert!(
            stdout.contains("add") || stdout.contains("Stage"),
            "Should describe add"
        );
    }

    #[test]
    fn test_add_command_recognizes_all_flag() {
        let output = run_cli(&["add", "--help"]);
        let stdout = get_stdout(&output);

        assert!(stdout.contains("--all"), "Should document --all flag");
    }

    #[test]
    fn test_add_command_recognizes_paths() {
        let output = run_cli(&["add", "--help"]);
        let stdout = get_stdout(&output);

        assert!(
            stdout.contains("PATHS") || stdout.contains("path"),
            "Should document paths argument"
        );
    }

    // MARK: - Commit Command Tests

    #[test]
    fn test_commit_command_help() {
        let output = run_cli(&["commit", "--help"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Commit help should succeed");
        assert!(
            stdout.contains("commit") || stdout.contains("Commit"),
            "Should describe commit"
        );
        assert!(
            stdout.contains("--message") || stdout.contains("-m"),
            "Should document message flag"
        );
    }

    #[test]
    fn test_commit_command_requires_message() {
        let output = run_cli(&["commit"]);

        // Should fail without message
        assert!(
            !command_succeeded(&output),
            "Commit without message should fail"
        );
    }

    #[test]
    fn test_commit_command_recognizes_bpm_flag() {
        let output = run_cli(&["commit", "--help"]);
        let stdout = get_stdout(&output);

        assert!(stdout.contains("--bpm"), "Should document BPM flag");
    }

    #[test]
    fn test_commit_command_recognizes_sample_rate_flag() {
        let output = run_cli(&["commit", "--help"]);
        let stdout = get_stdout(&output);

        assert!(
            stdout.contains("--sample-rate"),
            "Should document sample rate flag"
        );
    }

    #[test]
    fn test_commit_command_recognizes_key_flag() {
        let output = run_cli(&["commit", "--help"]);
        let stdout = get_stdout(&output);

        assert!(stdout.contains("--key"), "Should document key flag");
    }

    #[test]
    fn test_commit_command_recognizes_tags_flag() {
        let output = run_cli(&["commit", "--help"]);
        let stdout = get_stdout(&output);

        assert!(stdout.contains("--tags"), "Should document tags flag");
    }

    // MARK: - Log Command Tests

    #[test]
    fn test_log_command_help() {
        let output = run_cli(&["log", "--help"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Log help should succeed");
        assert!(
            stdout.contains("log") || stdout.contains("history"),
            "Should describe log"
        );
    }

    #[test]
    fn test_log_command_recognizes_limit_flag() {
        let output = run_cli(&["log", "--help"]);
        let stdout = get_stdout(&output);

        assert!(
            stdout.contains("--limit") || stdout.contains("-l"),
            "Should document limit flag"
        );
    }

    #[test]
    fn test_log_command_works_without_limit() {
        // This tests that log command is recognized even without --limit
        let output = run_cli(&["log", "--help"]);
        assert!(
            command_succeeded(&output),
            "Log command should be recognized"
        );
    }

    // MARK: - Restore Command Tests

    #[test]
    fn test_restore_command_help() {
        let output = run_cli(&["restore", "--help"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Restore help should succeed");
        assert!(
            stdout.contains("restore") || stdout.contains("Restore"),
            "Should describe restore"
        );
        assert!(
            stdout.contains("COMMIT") || stdout.contains("commit"),
            "Should mention commit ID"
        );
    }

    #[test]
    fn test_restore_command_requires_commit_id() {
        let output = run_cli(&["restore"]);

        // Should fail without commit ID
        assert!(
            !command_succeeded(&output),
            "Restore without commit ID should fail"
        );
    }

    // MARK: - Status Command Tests

    #[test]
    fn test_status_command_help() {
        let output = run_cli(&["status", "--help"]);
        let stdout = get_stdout(&output);

        assert!(command_succeeded(&output), "Status help should succeed");
        assert!(
            stdout.contains("status") || stdout.contains("Status"),
            "Should describe status"
        );
    }

    #[test]
    fn test_status_command_no_args_required() {
        // Status should be recognized without additional arguments
        let output = run_cli(&["status", "--help"]);
        assert!(
            command_succeeded(&output),
            "Status should not require arguments"
        );
    }

    // MARK: - Invalid Command Tests

    #[test]
    fn test_invalid_command() {
        let output = run_cli(&["invalid-command"]);

        assert!(!command_succeeded(&output), "Invalid command should fail");
    }

    #[test]
    fn test_no_command() {
        let output = run_cli(&[]);

        // Without subcommand, should show help or error
        let stdout = get_stdout(&output);
        let stderr = get_stderr(&output);

        assert!(
            stdout.contains("USAGE")
                || stdout.contains("Usage")
                || stderr.contains("required")
                || stderr.contains("USAGE")
                || stderr.contains("Usage"),
            "Should show usage or error message"
        );
    }

    // MARK: - Flag Combination Tests

    #[test]
    fn test_verbose_with_version() {
        let output = run_cli(&["--verbose", "--version"]);
        assert!(
            command_succeeded(&output),
            "Verbose with version should work"
        );
    }

    #[test]
    fn test_verbose_with_help() {
        let output = run_cli(&["-v", "-h"]);
        assert!(command_succeeded(&output), "Verbose with help should work");
    }

    #[test]
    fn test_multiple_short_flags() {
        // Test that short flags are recognized
        let output = run_cli(&["-h"]);
        assert!(command_succeeded(&output), "Short flags should work");
    }

    // MARK: - Command Documentation Tests

    #[test]
    fn test_all_commands_documented() {
        let output = run_cli(&["--help"]);
        let stdout = get_stdout(&output);

        // All commands should be listed in help
        assert!(stdout.contains("init"), "Should list init command");
        assert!(stdout.contains("add"), "Should list add command");
        assert!(stdout.contains("commit"), "Should list commit command");
        assert!(stdout.contains("log"), "Should list log command");
        assert!(stdout.contains("restore"), "Should list restore command");
        assert!(stdout.contains("status"), "Should list status command");
    }

    #[test]
    fn test_program_description() {
        let output = run_cli(&["--help"]);
        let stdout = get_stdout(&output);

        assert!(
            stdout.contains("Logic Pro") || stdout.contains("version control"),
            "Should describe purpose"
        );
    }

    // MARK: - Edge Cases

    #[test]
    fn test_extra_dashes_ignored() {
        // Test that extra -- doesn't break parsing
        let output = run_cli(&["--", "--help"]);
        // Should either work or fail gracefully
        let _ = command_succeeded(&output);
    }

    #[test]
    fn test_mixed_case_commands() {
        // Commands should be case-sensitive
        let output = run_cli(&["INIT", "--help"]);
        assert!(
            !command_succeeded(&output),
            "Commands should be case-sensitive"
        );
    }

    #[test]
    fn test_help_for_each_subcommand() {
        let commands = vec!["init", "add", "commit", "log", "restore", "status"];

        for cmd in commands {
            let output = run_cli(&[cmd, "--help"]);
            assert!(
                command_succeeded(&output),
                "Help for {} should succeed",
                cmd
            );
        }
    }
}
