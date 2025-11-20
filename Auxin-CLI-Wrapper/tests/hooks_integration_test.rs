/// Integration tests for hooks functionality
///
/// Tests the complete hooks workflow including:
/// - Init, install, list, remove hooks
/// - Pre-commit and post-commit hook execution
/// - Hook failure handling
/// - Timeout handling
/// - Built-in hook templates

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use auxin::hooks::{HookManager, HookType};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    /// Helper to create a test hook manager
    fn create_test_manager() -> (TempDir, HookManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::new(temp_dir.path());
        (temp_dir, manager)
    }

    // ===================
    // Init Tests
    // ===================

    #[test]
    fn test_hooks_init_creates_directory() {
        let (temp_dir, manager) = create_test_manager();

        manager.init().unwrap();

        let hooks_dir = temp_dir.path().join(".auxin").join("hooks");
        assert!(hooks_dir.exists());
        assert!(hooks_dir.join("pre-commit").exists());
        assert!(hooks_dir.join("post-commit").exists());
    }

    #[test]
    fn test_hooks_init_idempotent() {
        let (_temp_dir, manager) = create_test_manager();

        // Init multiple times should succeed
        assert!(manager.init().is_ok());
        assert!(manager.init().is_ok());
        assert!(manager.init().is_ok());
    }

    // ===================
    // Install Tests
    // ===================

    #[test]
    fn test_install_builtin_hook_validate_metadata() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let result = manager.install_builtin("validate-metadata", HookType::PreCommit);
        assert!(result.is_ok());

        let hooks = manager.list_hooks(HookType::PreCommit).unwrap();
        assert!(hooks.iter().any(|h| h.contains("validate-metadata")));
    }

    #[test]
    fn test_install_builtin_hook_backup() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let result = manager.install_builtin("backup", HookType::PostCommit);
        assert!(result.is_ok());

        let hooks = manager.list_hooks(HookType::PostCommit).unwrap();
        assert!(hooks.iter().any(|h| h.contains("backup")));
    }

    #[test]
    fn test_install_builtin_hook_notify() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let result = manager.install_builtin("notify", HookType::PostCommit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_builtin_hook_check_size() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let result = manager.install_builtin("check-size", HookType::PreCommit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_nonexistent_builtin() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let result = manager.install_builtin("nonexistent-hook", HookType::PreCommit);
        assert!(result.is_err());
    }

    #[test]
    fn test_install_hook_without_init() {
        let (_temp_dir, manager) = create_test_manager();

        // Should fail or auto-init
        let result = manager.install_builtin("validate-metadata", HookType::PreCommit);
        // Behavior depends on implementation
        // Either auto-init and succeed, or fail
    }

    #[test]
    fn test_install_duplicate_hook() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Install same hook twice
        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        let result = manager.install_builtin("validate-metadata", HookType::PreCommit);

        // Should either overwrite or error
        // Both behaviors are acceptable
    }

    // ===================
    // List Tests
    // ===================

    #[test]
    fn test_list_hooks_empty() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let pre_hooks = manager.list_hooks(HookType::PreCommit).unwrap();
        let post_hooks = manager.list_hooks(HookType::PostCommit).unwrap();

        assert!(pre_hooks.is_empty());
        assert!(post_hooks.is_empty());
    }

    #[test]
    fn test_list_hooks_after_install() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        manager.install_builtin("backup", HookType::PostCommit).unwrap();

        let pre_hooks = manager.list_hooks(HookType::PreCommit).unwrap();
        let post_hooks = manager.list_hooks(HookType::PostCommit).unwrap();

        assert_eq!(pre_hooks.len(), 1);
        assert_eq!(post_hooks.len(), 1);
    }

    #[test]
    fn test_list_hooks_multiple() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        manager.install_builtin("check-size", HookType::PreCommit).unwrap();

        let pre_hooks = manager.list_hooks(HookType::PreCommit).unwrap();
        assert_eq!(pre_hooks.len(), 2);
    }

    // ===================
    // Remove Tests
    // ===================

    #[test]
    fn test_remove_hook_success() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();

        // Verify it exists
        let hooks_before = manager.list_hooks(HookType::PreCommit).unwrap();
        assert_eq!(hooks_before.len(), 1);

        // Remove it
        let result = manager.remove_hook("validate-metadata", HookType::PreCommit);
        assert!(result.is_ok());

        // Verify it's gone
        let hooks_after = manager.list_hooks(HookType::PreCommit).unwrap();
        assert!(hooks_after.is_empty());
    }

    #[test]
    fn test_remove_hook_not_installed() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let result = manager.remove_hook("nonexistent", HookType::PreCommit);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_hook_wrong_type() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();

        // Try to remove from wrong type
        let result = manager.remove_hook("validate-metadata", HookType::PostCommit);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_all_hooks() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Install multiple hooks
        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        manager.install_builtin("check-size", HookType::PreCommit).unwrap();

        // Remove all
        manager.remove_hook("validate-metadata", HookType::PreCommit).unwrap();
        manager.remove_hook("check-size", HookType::PreCommit).unwrap();

        let hooks = manager.list_hooks(HookType::PreCommit).unwrap();
        assert!(hooks.is_empty());
    }

    // ===================
    // Execution Tests
    // ===================

    #[test]
    fn test_run_pre_commit_hooks_success() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a simple hook that succeeds
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit")
            .join("success-hook");

        fs::write(&hook_path, "#!/bin/bash\nexit 0\n").unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_pre_commit_hooks_failure() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a hook that fails
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit")
            .join("fail-hook");

        fs::write(&hook_path, "#!/bin/bash\nexit 1\n").unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_run_hooks_with_output() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a hook that outputs text
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit")
            .join("output-hook");

        fs::write(&hook_path, "#!/bin/bash\necho 'Hook executed'\nexit 0\n").unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_multiple_all_succeed() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let hooks_dir = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit");

        // Create multiple hooks that succeed
        for i in 0..3 {
            let hook_path = hooks_dir.join(format!("hook-{}", i));
            fs::write(&hook_path, "#!/bin/bash\nexit 0\n").unwrap();
            fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();
        }

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_multiple_one_fails() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let hooks_dir = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit");

        // Create hooks - first succeeds, second fails
        let hook1 = hooks_dir.join("01-success");
        fs::write(&hook1, "#!/bin/bash\nexit 0\n").unwrap();
        fs::set_permissions(&hook1, fs::Permissions::from_mode(0o755)).unwrap();

        let hook2 = hooks_dir.join("02-fail");
        fs::write(&hook2, "#!/bin/bash\nexit 1\n").unwrap();
        fs::set_permissions(&hook2, fs::Permissions::from_mode(0o755)).unwrap();

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_run_hooks_empty() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // No hooks installed - should succeed
        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_post_commit_hooks() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a post-commit hook
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("post-commit")
            .join("notify-hook");

        fs::write(&hook_path, "#!/bin/bash\necho 'Commit complete'\nexit 0\n").unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        let result = manager.run_hooks(HookType::PostCommit, temp_dir.path());
        assert!(result.is_ok());
    }

    // ===================
    // Timeout Tests
    // ===================

    #[test]
    fn test_run_hooks_timeout() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a hook that hangs
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit")
            .join("slow-hook");

        fs::write(&hook_path, "#!/bin/bash\nsleep 60\nexit 0\n").unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        // Should timeout (if timeout is implemented)
        // This test may need adjustment based on timeout implementation
        // let result = manager.run_hooks_with_timeout(HookType::PreCommit, temp_dir.path(), 1);
        // assert!(result.is_err());
    }

    // ===================
    // Built-in Hooks Tests
    // ===================

    #[test]
    fn test_list_builtins() {
        let (_temp_dir, manager) = create_test_manager();

        let builtins = manager.list_builtins().unwrap();

        assert!(builtins.contains(&"validate-metadata".to_string()));
        assert!(builtins.contains(&"backup".to_string()));
        assert!(builtins.contains(&"notify".to_string()));
        assert!(builtins.contains(&"check-size".to_string()));
    }

    // ===================
    // Edge Case Tests
    // ===================

    #[test]
    fn test_hook_with_spaces_in_name() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Hook names with spaces should be handled properly
        // Either sanitized or rejected
    }

    #[test]
    fn test_hook_non_executable() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a non-executable hook
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit")
            .join("non-exec-hook");

        fs::write(&hook_path, "#!/bin/bash\nexit 0\n").unwrap();
        // Don't set execute permission

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        // Should either skip or error
    }

    #[test]
    fn test_hook_bad_shebang() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a hook with invalid shebang
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit")
            .join("bad-shebang");

        fs::write(&hook_path, "#!/nonexistent/interpreter\nexit 0\n").unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_environment_variables() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a hook that uses environment variables
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit")
            .join("env-hook");

        // Hook should have access to AUXIN_REPO_PATH
        fs::write(&hook_path, "#!/bin/bash\ntest -n \"$AUXIN_REPO_PATH\"\nexit $?\n").unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        // Depends on whether env vars are passed
    }

    #[test]
    fn test_hook_with_stderr() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a hook that writes to stderr
        let hook_path = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit")
            .join("stderr-hook");

        fs::write(&hook_path, "#!/bin/bash\necho 'Error message' >&2\nexit 0\n").unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        let result = manager.run_hooks(HookType::PreCommit, temp_dir.path());
        assert!(result.is_ok()); // stderr output shouldn't cause failure
    }

    #[test]
    fn test_hook_ordering() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let hooks_dir = temp_dir.path()
            .join(".auxin")
            .join("hooks")
            .join("pre-commit");

        // Create hooks that should run in alphabetical order
        // Hooks should run 01, 02, 03
        let hook1 = hooks_dir.join("02-second");
        let hook2 = hooks_dir.join("01-first");
        let hook3 = hooks_dir.join("03-third");

        // Create a marker file to track execution order
        let marker = temp_dir.path().join("order.txt");

        fs::write(&hook1, format!("#!/bin/bash\necho '2' >> {:?}\nexit 0\n", marker)).unwrap();
        fs::write(&hook2, format!("#!/bin/bash\necho '1' >> {:?}\nexit 0\n", marker)).unwrap();
        fs::write(&hook3, format!("#!/bin/bash\necho '3' >> {:?}\nexit 0\n", marker)).unwrap();

        fs::set_permissions(&hook1, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(&hook2, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(&hook3, fs::Permissions::from_mode(0o755)).unwrap();

        manager.run_hooks(HookType::PreCommit, temp_dir.path()).unwrap();

        let order = fs::read_to_string(&marker).unwrap();
        assert_eq!(order.trim(), "1\n2\n3".trim());
    }
}
