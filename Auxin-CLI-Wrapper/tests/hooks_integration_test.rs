/// Integration tests for hooks functionality
///
/// Tests the complete hooks workflow including:
/// - Init, install, list, remove hooks
/// - Pre-commit and post-commit hook execution
/// - Hook failure handling
/// - Built-in hook templates

#[cfg(test)]
mod tests {
    use auxin::hooks::{HookManager, HookType};
    use auxin::CommitMetadata;
    use std::fs;
    #[cfg(unix)]
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

        let hooks_dir = temp_dir.path().join(".oxen").join("hooks");
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

    #[test]
    fn test_hooks_init_creates_readme() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let readme_path = temp_dir.path().join(".oxen").join("hooks").join("README.md");
        assert!(readme_path.exists());
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

        let hooks = manager.list_hooks_by_type(HookType::PreCommit).unwrap();
        assert!(hooks.iter().any(|h| h.contains("validate-metadata")));
    }

    #[test]
    fn test_install_builtin_hook_backup() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let result = manager.install_builtin("backup", HookType::PostCommit);
        assert!(result.is_ok());

        let hooks = manager.list_hooks_by_type(HookType::PostCommit).unwrap();
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
    fn test_install_builtin_hook_check_file_sizes() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let result = manager.install_builtin("check-file-sizes", HookType::PreCommit);
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
    fn test_install_duplicate_hook() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Install same hook twice
        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        let result = manager.install_builtin("validate-metadata", HookType::PreCommit);

        // Should overwrite (succeed)
        assert!(result.is_ok());
    }

    // ===================
    // List Tests
    // ===================

    #[test]
    fn test_list_hooks_empty() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let pre_hooks = manager.list_hooks_by_type(HookType::PreCommit).unwrap();
        let post_hooks = manager.list_hooks_by_type(HookType::PostCommit).unwrap();

        assert!(pre_hooks.is_empty());
        assert!(post_hooks.is_empty());
    }

    #[test]
    fn test_list_hooks_after_install() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        manager.install_builtin("backup", HookType::PostCommit).unwrap();

        let pre_hooks = manager.list_hooks_by_type(HookType::PreCommit).unwrap();
        let post_hooks = manager.list_hooks_by_type(HookType::PostCommit).unwrap();

        assert_eq!(pre_hooks.len(), 1);
        assert_eq!(post_hooks.len(), 1);
    }

    #[test]
    fn test_list_hooks_multiple() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        manager.install_builtin("check-file-sizes", HookType::PreCommit).unwrap();

        let pre_hooks = manager.list_hooks_by_type(HookType::PreCommit).unwrap();
        assert_eq!(pre_hooks.len(), 2);
    }

    #[test]
    fn test_list_all_hooks() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        manager.install_builtin("backup", HookType::PostCommit).unwrap();

        let all_hooks = manager.list_hooks().unwrap();
        assert_eq!(all_hooks.len(), 2);

        // Check that tuples contain correct types
        assert!(all_hooks.iter().any(|(t, n)| *t == HookType::PreCommit && n == "validate-metadata"));
        assert!(all_hooks.iter().any(|(t, n)| *t == HookType::PostCommit && n == "backup"));
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
        let hooks_before = manager.list_hooks_by_type(HookType::PreCommit).unwrap();
        assert_eq!(hooks_before.len(), 1);

        // Remove it
        let result = manager.remove_hook("validate-metadata", HookType::PreCommit);
        assert!(result.is_ok());

        // Verify it's gone
        let hooks_after = manager.list_hooks_by_type(HookType::PreCommit).unwrap();
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
        manager.install_builtin("check-file-sizes", HookType::PreCommit).unwrap();

        // Remove all
        manager.remove_hook("validate-metadata", HookType::PreCommit).unwrap();
        manager.remove_hook("check-file-sizes", HookType::PreCommit).unwrap();

        let hooks = manager.list_hooks_by_type(HookType::PreCommit).unwrap();
        assert!(hooks.is_empty());
    }

    // ===================
    // Execution Tests
    // ===================

    #[test]
    #[cfg(unix)]
    fn test_run_pre_commit_hooks_success() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a simple hook that succeeds
        let hook_path = temp_dir.path()
            .join(".oxen")
            .join("hooks")
            .join("pre-commit")
            .join("success-hook");

        fs::write(&hook_path, "#!/bin/bash\nexit 0\n").unwrap();
        let mut perms = fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).unwrap();

        let metadata = CommitMetadata::new("Test commit");
        let result = manager.run_hooks(HookType::PreCommit, &metadata);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Hook should succeed
    }

    #[test]
    #[cfg(unix)]
    fn test_run_pre_commit_hooks_failure() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a hook that fails
        let hook_path = temp_dir.path()
            .join(".oxen")
            .join("hooks")
            .join("pre-commit")
            .join("fail-hook");

        fs::write(&hook_path, "#!/bin/bash\nexit 1\n").unwrap();
        let mut perms = fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).unwrap();

        let metadata = CommitMetadata::new("Test commit");
        let result = manager.run_hooks(HookType::PreCommit, &metadata);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Hook should fail
    }

    #[test]
    fn test_run_hooks_no_hooks() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // No hooks installed
        let metadata = CommitMetadata::new("Test commit");
        let result = manager.run_hooks(HookType::PreCommit, &metadata);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should succeed with no hooks
    }

    #[test]
    fn test_run_hooks_no_init() {
        let (_temp_dir, manager) = create_test_manager();

        // No init called
        let metadata = CommitMetadata::new("Test commit");
        let result = manager.run_hooks(HookType::PreCommit, &metadata);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should succeed (no hooks directory)
    }

    // ===================
    // HookType Tests
    // ===================

    #[test]
    fn test_hook_type_dir_name() {
        assert_eq!(HookType::PreCommit.dir_name(), "pre-commit");
        assert_eq!(HookType::PostCommit.dir_name(), "post-commit");
    }

    #[test]
    fn test_hook_type_equality() {
        assert_eq!(HookType::PreCommit, HookType::PreCommit);
        assert_eq!(HookType::PostCommit, HookType::PostCommit);
        assert_ne!(HookType::PreCommit, HookType::PostCommit);
    }

    // ===================
    // Built-in Hooks Tests
    // ===================

    #[test]
    fn test_list_builtins() {
        let builtins = HookManager::list_builtins();

        assert!(!builtins.is_empty());

        // Check for expected built-ins
        let names: Vec<&str> = builtins.iter().map(|h| h.name.as_str()).collect();
        assert!(names.contains(&"validate-metadata"));
        assert!(names.contains(&"check-file-sizes"));
        assert!(names.contains(&"notify"));
        assert!(names.contains(&"backup"));
    }

    #[test]
    fn test_builtin_hook_has_description() {
        let builtins = HookManager::list_builtins();

        for hook in builtins {
            assert!(!hook.description.is_empty(), "Hook {} has empty description", hook.name);
        }
    }

    #[test]
    fn test_builtin_hook_types() {
        let builtins = HookManager::list_builtins();

        // Check that we have both pre and post commit hooks
        let pre_commit_count = builtins.iter().filter(|h| h.hook_type == HookType::PreCommit).count();
        let post_commit_count = builtins.iter().filter(|h| h.hook_type == HookType::PostCommit).count();

        assert!(pre_commit_count > 0, "No pre-commit hooks");
        assert!(post_commit_count > 0, "No post-commit hooks");
    }

    // ===================
    // Edge Case Tests
    // ===================

    #[test]
    fn test_hook_with_special_characters_in_name() {
        let (_temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Hook names with special characters should be handled
        // (though this is an edge case that may not be supported)
    }

    #[test]
    fn test_hooks_directory_permissions() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        let hooks_dir = temp_dir.path().join(".oxen").join("hooks");
        let metadata = fs::metadata(&hooks_dir).unwrap();
        assert!(metadata.is_dir());
    }

    #[test]
    #[cfg(unix)]
    fn test_installed_hook_is_executable() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();

        let hook_path = temp_dir.path()
            .join(".oxen")
            .join("hooks")
            .join("pre-commit")
            .join("validate-metadata");

        let metadata = fs::metadata(&hook_path).unwrap();
        let permissions = metadata.permissions();

        // Check that the executable bit is set
        assert!(permissions.mode() & 0o111 != 0, "Hook is not executable");
    }

    // ===================
    // Environment Variable Tests
    // ===================

    #[test]
    #[cfg(unix)]
    fn test_hook_receives_environment_variables() {
        let (temp_dir, manager) = create_test_manager();
        manager.init().unwrap();

        // Create a hook that outputs environment variables
        let hook_path = temp_dir.path()
            .join(".oxen")
            .join("hooks")
            .join("pre-commit")
            .join("env-test");

        fs::write(&hook_path, r#"#!/bin/bash
if [ -z "$AUXIN_MESSAGE" ]; then
    exit 1
fi
exit 0
"#).unwrap();
        let mut perms = fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).unwrap();

        let metadata = CommitMetadata::new("Test message");
        let result = manager.run_hooks(HookType::PreCommit, &metadata);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should succeed
    }
}
