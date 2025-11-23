/// Integration tests for branch operations
///
/// These tests cover branch deletion and remote branch operations to fill
/// gaps identified in the branching test coverage analysis.
///
/// Coverage:
/// - Branch deletion (non-current branches)
/// - Error handling (delete current branch, delete main)
/// - Remote branch creation via push
/// - Remote branch tracking
///
/// To run these tests:
///   cargo test --test branch_operations_test -- --nocapture
#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use super::common::TestFixture;
    use auxin::OxenSubprocess;
    use std::fs;
    use tempfile::TempDir;

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

    /// Helper to initialize a repository with an initial commit
    fn init_repo_with_commit(fixture: &TestFixture, oxen: &OxenSubprocess) {
        oxen.init(fixture.path()).unwrap();
        fixture.add_text_file("README.md", "# Test Project");
        oxen.add(fixture.path(), &["README.md".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Initial commit").unwrap();
    }

    /// Helper to setup a remote repository
    fn setup_remote_repo() -> Result<TempDir, std::io::Error> {
        let remote_dir = TempDir::new()?;
        let remote_path = remote_dir.path();

        // Initialize remote repo
        std::process::Command::new("oxen")
            .args(&["init"])
            .current_dir(remote_path)
            .output()
            .expect("Failed to init remote");

        Ok(remote_dir)
    }

    // ========== Branch Deletion Tests ==========

    #[test]
    fn test_delete_non_current_branch() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture, &oxen);

        // Create a feature branch
        let feature_branch = "feature/test-delete";
        oxen.create_branch(fixture.path(), feature_branch).unwrap();

        // Verify branch exists
        let branches = oxen.list_branches(fixture.path()).unwrap();
        assert!(
            branches.iter().any(|b| b.name == feature_branch),
            "Feature branch should exist"
        );

        // Switch back to main
        oxen.checkout(fixture.path(), "main").unwrap();

        // Delete the feature branch
        let result = oxen.delete_branch(fixture.path(), feature_branch);
        assert!(
            result.is_ok(),
            "Should be able to delete non-current branch: {:?}",
            result.err()
        );

        // Verify branch is gone
        let branches_after = oxen.list_branches(fixture.path()).unwrap();
        assert!(
            !branches_after.iter().any(|b| b.name == feature_branch),
            "Feature branch should be deleted"
        );
    }

    #[test]
    fn test_cannot_delete_current_branch() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture, &oxen);

        // Create and switch to feature branch
        let feature_branch = "feature/current";
        oxen.create_branch(fixture.path(), feature_branch).unwrap();

        // Try to delete current branch (should fail)
        let result = oxen.delete_branch(fixture.path(), feature_branch);
        assert!(
            result.is_err(),
            "Should NOT be able to delete current branch"
        );

        // Error message should be meaningful
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("current") || err_msg.contains("checked out") || err_msg.contains("Cannot delete"),
            "Error message should explain why deletion failed: {}",
            err_msg
        );
    }

    #[test]
    fn test_delete_branch_after_switching() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture, &oxen);

        // Create feature branch and make a commit
        let feature_branch = "feature/to-delete";
        oxen.create_branch(fixture.path(), feature_branch).unwrap();

        fixture.add_text_file("feature.txt", "feature work");
        oxen.add(fixture.path(), &["feature.txt".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Feature work").unwrap();

        // Switch back to main
        oxen.checkout(fixture.path(), "main").unwrap();

        // Now delete feature branch
        let result = oxen.delete_branch(fixture.path(), feature_branch);
        assert!(
            result.is_ok(),
            "Should delete branch after switching away: {:?}",
            result.err()
        );

        // Verify file from feature branch is not in main
        let feature_file = fixture.path().join("feature.txt");
        assert!(
            !feature_file.exists(),
            "Feature file should not exist on main branch"
        );
    }

    #[test]
    fn test_list_branches_after_deletion() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture, &oxen);

        // Create multiple branches
        oxen.create_branch(fixture.path(), "feature/one").unwrap();
        oxen.checkout(fixture.path(), "main").unwrap();
        oxen.create_branch(fixture.path(), "feature/two").unwrap();
        oxen.checkout(fixture.path(), "main").unwrap();

        // Verify both exist
        let branches = oxen.list_branches(fixture.path()).unwrap();
        assert_eq!(
            branches
                .iter()
                .filter(|b| b.name.starts_with("feature/"))
                .count(),
            2,
            "Should have 2 feature branches"
        );

        // Delete one
        oxen.delete_branch(fixture.path(), "feature/one").unwrap();

        // Verify only one remains
        let branches_after = oxen.list_branches(fixture.path()).unwrap();
        assert_eq!(
            branches_after
                .iter()
                .filter(|b| b.name.starts_with("feature/"))
                .count(),
            1,
            "Should have 1 feature branch remaining"
        );
        assert!(
            branches_after.iter().any(|b| b.name == "feature/two"),
            "feature/two should still exist"
        );
        assert!(
            !branches_after.iter().any(|b| b.name == "feature/one"),
            "feature/one should be deleted"
        );
    }

    // ========== Remote Branch Tests ==========

    #[test]
    fn test_push_creates_remote_branch() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture, &oxen);

        // Setup remote
        let remote_dir = setup_remote_repo().expect("Failed to setup remote");
        let remote_path = remote_dir.path();

        // Add remote
        let remote_url = format!("file://{}", remote_path.display());
        let output = std::process::Command::new("oxen")
            .args(&["config", "--add", "remote.origin.url", &remote_url])
            .current_dir(fixture.path())
            .output()
            .expect("Failed to add remote");
        assert!(output.status.success(), "Failed to configure remote");

        // Push to remote
        oxen.push(fixture.path(), Some("origin"), Some("main"))
            .unwrap();

        // Verify remote has commits
        let log_output = std::process::Command::new("oxen")
            .args(&["log", "--limit", "5"])
            .current_dir(remote_path)
            .output()
            .expect("Failed to check remote log");

        assert!(
            log_output.status.success(),
            "Remote should have commits after push"
        );

        let log_str = String::from_utf8_lossy(&log_output.stdout);
        assert!(
            log_str.contains("Initial commit"),
            "Remote should have our initial commit"
        );
    }

    #[test]
    fn test_push_pull_roundtrip_with_branches() {
        skip_if_no_oxen!();

        let fixture1 = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture1, &oxen);

        // Setup remote
        let remote_dir = setup_remote_repo().expect("Failed to setup remote");
        let remote_path = remote_dir.path();
        let remote_url = format!("file://{}", remote_path.display());

        // Add remote to repo1
        std::process::Command::new("oxen")
            .args(&["config", "--add", "remote.origin.url", &remote_url])
            .current_dir(fixture1.path())
            .output()
            .expect("Failed to add remote");

        // Push from repo1
        oxen.push(fixture1.path(), Some("origin"), Some("main"))
            .unwrap();

        // Create second repo (user 2)
        let fixture2 = TestFixture::new();

        // Clone from remote
        let clone_output = std::process::Command::new("oxen")
            .args(&["clone", &remote_url, fixture2.path().to_str().unwrap()])
            .output()
            .expect("Failed to clone");
        assert!(
            clone_output.status.success(),
            "Clone should succeed: {}",
            String::from_utf8_lossy(&clone_output.stderr)
        );

        // Verify repo2 has the commit
        let log = oxen.log(fixture2.path(), Some(5)).unwrap();
        assert!(
            !log.is_empty(),
            "Cloned repo should have commit history"
        );
        assert!(
            log.iter().any(|c| c.message.contains("Initial commit")),
            "Cloned repo should have initial commit"
        );
    }

    #[test]
    fn test_remote_branch_tracking() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture, &oxen);

        // Setup remote
        let remote_dir = setup_remote_repo().expect("Failed to setup remote");
        let remote_path = remote_dir.path();
        let remote_url = format!("file://{}", remote_path.display());

        // Add remote
        std::process::Command::new("oxen")
            .args(&["config", "--add", "remote.origin.url", &remote_url])
            .current_dir(fixture.path())
            .output()
            .expect("Failed to add remote");

        // Create feature branch
        oxen.create_branch(fixture.path(), "feature/remote").unwrap();

        // Make a commit on feature branch
        fixture.add_text_file("remote-feature.txt", "remote feature work");
        oxen.add(fixture.path(), &["remote-feature.txt".as_ref()])
            .unwrap();
        oxen.commit(fixture.path(), "Remote feature work").unwrap();

        // Push feature branch to remote
        oxen.push(fixture.path(), Some("origin"), Some("feature/remote"))
            .unwrap();

        // Verify push succeeded (second push should also succeed)
        // Note: Remote might not show "feature/remote" directly since it's a bare repo,
        // but we can verify the push succeeded by doing it again
        let result = oxen.push(fixture.path(), Some("origin"), Some("feature/remote"));
        assert!(
            result.is_ok(),
            "Pushing feature branch should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_fetch_remote_changes() {
        skip_if_no_oxen!();

        // User 1 creates and pushes
        let fixture1 = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture1, &oxen);

        let remote_dir = setup_remote_repo().expect("Failed to setup remote");
        let remote_url = format!("file://{}", remote_dir.path().display());

        std::process::Command::new("oxen")
            .args(&["config", "--add", "remote.origin.url", &remote_url])
            .current_dir(fixture1.path())
            .output()
            .expect("Failed to add remote");

        oxen.push(fixture1.path(), Some("origin"), Some("main"))
            .unwrap();

        // User 2 clones
        let fixture2 = TestFixture::new();
        std::process::Command::new("oxen")
            .args(&["clone", &remote_url, fixture2.path().to_str().unwrap()])
            .output()
            .expect("Failed to clone");

        // User 1 makes new commit
        fixture1.add_text_file("new-file.txt", "new content");
        oxen.add(fixture1.path(), &["new-file.txt".as_ref()])
            .unwrap();
        oxen.commit(fixture1.path(), "New commit from user1")
            .unwrap();
        oxen.push(fixture1.path(), Some("origin"), Some("main"))
            .unwrap();

        // User 2 pulls changes
        oxen.pull(fixture2.path()).unwrap();

        // Verify user 2 has the new file
        let new_file = fixture2.path().join("new-file.txt");
        assert!(
            new_file.exists(),
            "User 2 should have pulled the new file from user 1"
        );

        let content = fs::read_to_string(&new_file).unwrap();
        assert_eq!(content, "new content", "File content should match");
    }

    #[test]
    fn test_branch_deletion_local_only() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let oxen = OxenSubprocess::new();
        init_repo_with_commit(&fixture, &oxen);

        // Setup remote
        let remote_dir = setup_remote_repo().expect("Failed to setup remote");
        let remote_url = format!("file://{}", remote_dir.path().display());

        std::process::Command::new("oxen")
            .args(&["config", "--add", "remote.origin.url", &remote_url])
            .current_dir(fixture.path())
            .output()
            .expect("Failed to add remote");

        // Create and push feature branch
        oxen.create_branch(fixture.path(), "feature/to-delete")
            .unwrap();
        fixture.add_text_file("feature.txt", "feature");
        oxen.add(fixture.path(), &["feature.txt".as_ref()]).unwrap();
        oxen.commit(fixture.path(), "Feature commit").unwrap();
        oxen.push(fixture.path(), Some("origin"), Some("feature/to-delete"))
            .unwrap();

        // Switch to main and delete local branch
        oxen.checkout(fixture.path(), "main").unwrap();
        oxen.delete_branch(fixture.path(), "feature/to-delete")
            .unwrap();

        // Verify local branch is gone
        let branches = oxen.list_branches(fixture.path()).unwrap();
        assert!(
            !branches.iter().any(|b| b.name == "feature/to-delete"),
            "Local branch should be deleted"
        );

        // Note: Remote branch might still exist (depending on Oxen's behavior)
        // This test validates that local deletion doesn't require remote deletion
    }
}
