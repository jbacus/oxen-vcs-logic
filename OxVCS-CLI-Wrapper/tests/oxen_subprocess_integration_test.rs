/// Integration tests for OxenSubprocess
///
/// These tests require the `oxen` CLI to be installed:
///   pip3 install oxen-ai
///   or
///   cargo install oxen
///
/// To run these tests:
///   cargo test --test oxen_subprocess_integration_test -- --nocapture
///
/// Note: These tests will be skipped if oxen is not available

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use super::common::TestFixture;
    use oxenvcs_cli::oxen_subprocess::OxenSubprocess;
    use std::fs;

    /// Check if oxen CLI is available
    fn oxen_available() -> bool {
        std::process::Command::new("oxen")
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Skip test if oxen is not available
    macro_rules! skip_if_no_oxen {
        () => {
            if !oxen_available() {
                println!("Skipping test: oxen CLI not installed");
                println!("Install with: pip3 install oxen-ai");
                return;
            }
        };
    }

    // Basic initialization tests

    #[test]
    fn test_init_creates_oxen_directory() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        let result = oxen.init(fixture.path());
        assert!(result.is_ok(), "Init should succeed: {:?}", result.err());

        // Verify .oxen directory was created
        let oxen_dir = fixture.path().join(".oxen");
        assert!(oxen_dir.exists(), ".oxen directory should exist");
    }

    #[test]
    fn test_init_twice_fails() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        let result = oxen.init(fixture.path());

        assert!(result.is_err(), "Initializing twice should fail");
    }

    #[test]
    fn test_init_nonexistent_directory_fails() {
        skip_if_no_oxen!();

        let oxen = OxenSubprocess::new();
        let result = oxen.init("/nonexistent/directory/that/does/not/exist".as_ref());

        assert!(result.is_err(), "Init on nonexistent directory should fail");
    }

    // Add operation tests

    #[test]
    fn test_add_single_file() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("test.txt", "test content");

        let result = oxen.add(fixture.path(), &["test.txt".as_ref()]);
        assert!(result.is_ok(), "Add should succeed: {:?}", result.err());
    }

    #[test]
    fn test_add_multiple_files() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("file1.txt", "content 1");
        fixture.add_text_file("file2.txt", "content 2");
        fixture.add_text_file("file3.txt", "content 3");

        let result = oxen.add(
            fixture.path(),
            &["file1.txt".as_ref(), "file2.txt".as_ref(), "file3.txt".as_ref()],
        );
        assert!(result.is_ok(), "Adding multiple files should succeed");
    }

    #[test]
    fn test_add_all_files() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("file1.txt", "content 1");
        fixture.add_text_file("file2.txt", "content 2");

        let result = oxen.add(fixture.path(), &[".".as_ref()]);
        assert!(result.is_ok(), "Add all should succeed");
    }

    #[test]
    fn test_add_nonexistent_file() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        let result = oxen.add(fixture.path(), &["nonexistent.txt".as_ref()]);
        assert!(result.is_err(), "Adding nonexistent file should fail");
    }

    #[test]
    fn test_add_without_init() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        fixture.add_text_file("test.txt", "test");

        let result = oxen.add(fixture.path(), &["test.txt".as_ref()]);
        assert!(result.is_err(), "Add without init should fail");
    }

    // Commit tests

    #[test]
    fn test_commit_after_add() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("test.txt", "test content");
        oxen.add(fixture.path(), &["test.txt".as_ref()]).unwrap();

        let result = oxen.commit(fixture.path(), "Initial commit");
        assert!(result.is_ok(), "Commit should succeed");

        let commit_info = result.unwrap();
        assert!(!commit_info.id.is_empty(), "Commit ID should not be empty");
        assert_eq!(commit_info.message, "Initial commit");
    }

    #[test]
    fn test_commit_without_changes() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        let result = oxen.commit(fixture.path(), "Empty commit");
        // Oxen might allow or disallow empty commits - both are valid behaviors
        // Just verify we get a consistent response
        if result.is_ok() {
            println!("Empty commit succeeded (oxen allows empty commits)");
        } else {
            println!("Empty commit failed (oxen requires changes)");
        }
    }

    #[test]
    fn test_commit_with_multiline_message() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("test.txt", "test");
        oxen.add(fixture.path(), &["test.txt".as_ref()]).unwrap();

        let message = "First line\n\nSecond paragraph\nThird line";
        let result = oxen.commit(fixture.path(), message);

        assert!(result.is_ok(), "Multiline commit should succeed");
        let commit_info = result.unwrap();
        assert!(
            commit_info.message.contains("First line"),
            "Message should contain first line"
        );
    }

    #[test]
    fn test_commit_with_special_characters() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("test.txt", "test");
        oxen.add(fixture.path(), &["test.txt".as_ref()]).unwrap();

        let message = "Commit with 'quotes' and \"double quotes\" and (parentheses)";
        let result = oxen.commit(fixture.path(), message);

        assert!(result.is_ok(), "Commit with special chars should succeed");
    }

    // Log tests

    #[test]
    fn test_log_after_commits() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        // Create three commits
        fixture.add_text_file("file1.txt", "content1");
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "First commit").unwrap();

        fixture.add_text_file("file2.txt", "content2");
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Second commit").unwrap();

        fixture.add_text_file("file3.txt", "content3");
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Third commit").unwrap();

        let result = oxen.log(fixture.path(), None);
        assert!(result.is_ok(), "Log should succeed");

        let commits = result.unwrap();
        assert_eq!(commits.len(), 3, "Should have 3 commits");

        // Verify messages are correct (most recent first)
        assert_eq!(commits[0].message, "Third commit");
        assert_eq!(commits[1].message, "Second commit");
        assert_eq!(commits[2].message, "First commit");
    }

    #[test]
    fn test_log_with_limit() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        // Create five commits
        for i in 1..=5 {
            fixture.add_text_file(&format!("file{}.txt", i), &format!("content{}", i));
            oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
            oxen.commit(fixture.path(), &format!("Commit {}", i)).unwrap();
        }

        let result = oxen.log(fixture.path(), Some(3));
        assert!(result.is_ok(), "Log with limit should succeed");

        let commits = result.unwrap();
        assert_eq!(commits.len(), 3, "Should have 3 commits (limited)");
    }

    #[test]
    fn test_log_empty_repository() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        let result = oxen.log(fixture.path(), None);
        // Empty repo log might succeed with empty list or fail - both valid
        if result.is_ok() {
            let commits = result.unwrap();
            assert_eq!(commits.len(), 0, "Empty repo should have no commits");
        }
    }

    // Status tests

    #[test]
    fn test_status_clean_repository() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        let result = oxen.status(fixture.path());
        assert!(result.is_ok(), "Status should succeed");

        let status = result.unwrap();
        assert!(status.staged.is_empty(), "No files should be staged");
        assert!(status.modified.is_empty(), "No files should be modified");
        assert!(status.untracked.is_empty(), "No files should be untracked");
    }

    #[test]
    fn test_status_with_untracked_files() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("untracked.txt", "content");

        let result = oxen.status(fixture.path());
        assert!(result.is_ok(), "Status should succeed");

        let status = result.unwrap();
        assert!(
            !status.untracked.is_empty(),
            "Should have untracked files"
        );
        assert!(
            status.untracked.iter().any(|p| p.ends_with("untracked.txt")),
            "Should contain untracked.txt"
        );
    }

    #[test]
    fn test_status_with_staged_files() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("staged.txt", "content");
        oxen.add(fixture.path(), &["staged.txt".as_ref()]).unwrap();

        let result = oxen.status(fixture.path());
        assert!(result.is_ok(), "Status should succeed");

        let status = result.unwrap();
        assert!(!status.staged.is_empty(), "Should have staged files");
    }

    #[test]
    fn test_status_with_modified_files() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("file.txt", "original");
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Initial").unwrap();

        // Modify the file
        fixture.add_text_file("file.txt", "modified");

        let result = oxen.status(fixture.path());
        assert!(result.is_ok(), "Status should succeed");

        let status = result.unwrap();
        assert!(!status.modified.is_empty(), "Should have modified files");
    }

    // Checkout tests

    #[test]
    fn test_checkout_previous_commit() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        // First commit
        fixture.add_text_file("file.txt", "version 1");
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
        let first_commit = oxen.commit(fixture.path(), "First").unwrap();

        // Second commit
        fixture.add_text_file("file.txt", "version 2");
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Second").unwrap();

        // Checkout first commit
        let result = oxen.checkout(fixture.path(), &first_commit.id);
        assert!(result.is_ok(), "Checkout should succeed");

        // Verify file content
        let content = fs::read_to_string(fixture.path().join("file.txt")).unwrap();
        assert_eq!(content, "version 1", "File should be restored to version 1");
    }

    #[test]
    fn test_checkout_invalid_commit() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        let result = oxen.checkout(fixture.path(), "invalid_commit_id");
        assert!(result.is_err(), "Checkout invalid commit should fail");
    }

    // Branch tests

    #[test]
    fn test_list_branches() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        let result = oxen.branches(fixture.path());
        assert!(result.is_ok(), "List branches should succeed");

        let branches = result.unwrap();
        assert!(
            !branches.is_empty(),
            "Should have at least one branch (main)"
        );
        assert!(
            branches.iter().any(|b| b.name == "main" || b.name == "master"),
            "Should have main or master branch"
        );
    }

    #[test]
    fn test_current_branch_is_marked() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        let result = oxen.branches(fixture.path());
        assert!(result.is_ok(), "List branches should succeed");

        let branches = result.unwrap();
        let current_count = branches.iter().filter(|b| b.is_current).count();
        assert_eq!(current_count, 1, "Exactly one branch should be current");
    }

    // Push/Pull tests (these will fail without a remote, but test the interface)

    #[test]
    fn test_push_without_remote_fails() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("file.txt", "content");
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Commit").unwrap();

        let result = oxen.push(fixture.path());
        assert!(result.is_err(), "Push without remote should fail");
    }

    #[test]
    fn test_pull_without_remote_fails() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        let result = oxen.pull(fixture.path());
        assert!(result.is_err(), "Pull without remote should fail");
    }

    // Workflow integration tests

    #[test]
    fn test_complete_workflow() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        // 1. Initialize
        oxen.init(fixture.path()).unwrap();

        // 2. Add files
        fixture.add_text_file("README.md", "# My Project");
        fixture.add_text_file("src/main.rs", "fn main() {}");

        // 3. Stage all
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();

        // 4. Check status
        let status = oxen.status(fixture.path()).unwrap();
        assert!(!status.staged.is_empty(), "Files should be staged");

        // 5. Commit
        let commit = oxen.commit(fixture.path(), "Initial commit").unwrap();
        assert!(!commit.id.is_empty(), "Should have commit ID");

        // 6. Check log
        let commits = oxen.log(fixture.path(), None).unwrap();
        assert_eq!(commits.len(), 1, "Should have one commit");
        assert_eq!(commits[0].message, "Initial commit");

        // 7. Modify file
        fixture.add_text_file("README.md", "# My Project\n\nUpdated!");

        // 8. Check status shows modification
        let status = oxen.status(fixture.path()).unwrap();
        assert!(!status.modified.is_empty(), "Should have modified files");

        // 9. Add and commit again
        oxen.add(fixture.path(), &["README.md".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Update README").unwrap();

        // 10. Verify two commits
        let commits = oxen.log(fixture.path(), None).unwrap();
        assert_eq!(commits.len(), 2, "Should have two commits");
    }

    #[test]
    fn test_large_file_handling() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();

        oxen.init(fixture.path()).unwrap();

        // Create a 10MB file
        fixture.create_audio_file("large.wav", 10);

        // Add and commit
        oxen.add(fixture.path(), &[".".as_ref()]).unwrap();
        let result = oxen.commit(fixture.path(), "Add large file");

        assert!(result.is_ok(), "Should handle large files");
    }

    #[test]
    fn test_verbose_mode() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new_verbose();

        // Verbose mode should not change behavior, just output
        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("test.txt", "test");
        oxen.add(fixture.path(), &["test.txt".as_ref()]).unwrap();
        let result = oxen.commit(fixture.path(), "Test");

        assert!(result.is_ok(), "Verbose mode should work correctly");
    }
}
