/// Integration tests for draft branch workflow
///
/// These tests require the `oxen` CLI to be installed:
///   pip3 install oxen-ai
///   or
///   cargo install oxen
///
/// To run these tests:
///   cargo test --test draft_manager_integration_test -- --nocapture
#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use super::common::TestFixture;
    use auxin::{CommitMetadata, DraftManager, OxenRepository};

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

    /// Helper function to initialize a repository with an initial commit
    /// (Oxen requires HEAD to exist before creating branches)
    async fn init_repo_with_commit(fixture: &TestFixture) -> OxenRepository {
        let _init = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());
        fixture.add_text_file("README.md", "# Test Project");
        repo.stage_all().await.unwrap();
        let metadata = CommitMetadata::new("Initial commit");
        repo.create_commit(metadata).await.unwrap();
        repo
    }

    #[tokio::test]
    async fn test_draft_manager_initialization() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let _repo = init_repo_with_commit(&fixture).await;

        // Create draft manager and initialize
        let draft_manager = DraftManager::new(fixture.path()).unwrap();
        draft_manager.initialize().await.unwrap();

        // Verify draft branch was created
        assert!(
            draft_manager.draft_branch_exists().unwrap(),
            "Draft branch should exist after initialization"
        );

        // Verify we're on draft branch
        assert!(
            draft_manager.is_on_draft_branch().unwrap(),
            "Should be on draft branch after initialization"
        );

        let current = draft_manager.current_branch().unwrap();
        assert_eq!(
            current,
            DraftManager::DEFAULT_DRAFT_BRANCH,
            "Current branch should be draft"
        );
    }

    #[tokio::test]
    async fn test_draft_manager_auto_commit() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let repo = init_repo_with_commit(&fixture).await;

        // Initialize draft manager
        let draft_manager = DraftManager::new(fixture.path()).unwrap();
        draft_manager.initialize().await.unwrap();

        // Create and commit a file
        fixture.add_text_file("test.txt", "initial content");
        repo.stage_all().await.unwrap();
        let metadata = CommitMetadata::new("Test auto-commit");
        let commit_id = draft_manager.auto_commit(metadata).await.unwrap();

        // Verify commit was created
        assert!(!commit_id.is_empty(), "Commit ID should not be empty");

        // Verify we're still on draft branch
        assert!(
            draft_manager.is_on_draft_branch().unwrap(),
            "Should still be on draft branch after auto-commit"
        );

        // Verify commit count increased
        let count = draft_manager.draft_commit_count().unwrap();
        assert!(
            count >= 1,
            "Draft commit count should be at least 1 (got {})",
            count
        );
    }

    #[tokio::test]
    async fn test_draft_manager_switch_branches() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let _repo = init_repo_with_commit(&fixture).await;

        // Initialize draft manager
        let draft_manager = DraftManager::new(fixture.path()).unwrap();
        draft_manager.initialize().await.unwrap();

        // Should start on draft branch
        assert_eq!(
            draft_manager.current_branch().unwrap(),
            DraftManager::DEFAULT_DRAFT_BRANCH
        );

        // Switch to main
        draft_manager.switch_to_main().await.unwrap();
        assert_eq!(
            draft_manager.current_branch().unwrap(),
            DraftManager::MAIN_BRANCH
        );
        assert!(!draft_manager.is_on_draft_branch().unwrap());

        // Switch back to draft
        draft_manager.switch_to_draft().await.unwrap();
        assert_eq!(
            draft_manager.current_branch().unwrap(),
            DraftManager::DEFAULT_DRAFT_BRANCH
        );
        assert!(draft_manager.is_on_draft_branch().unwrap());
    }

    #[tokio::test]
    async fn test_draft_manager_multiple_commits() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let repo = init_repo_with_commit(&fixture).await;

        // Initialize draft manager
        let draft_manager = DraftManager::new(fixture.path()).unwrap();
        draft_manager.initialize().await.unwrap();

        // Create multiple commits
        for i in 1..=3 {
            fixture.add_text_file(&format!("file{}.txt", i), &format!("content {}", i));
            repo.stage_all().await.unwrap();
            let metadata = CommitMetadata::new(&format!("Commit {}", i));
            draft_manager.auto_commit(metadata).await.unwrap();
        }

        // Verify commit count
        let count = draft_manager.draft_commit_count().unwrap();
        assert!(
            count >= 3,
            "Should have at least 3 commits on draft branch (got {})",
            count
        );

        // Verify files exist
        for i in 1..=3 {
            let file_path = fixture.path().join(format!("file{}.txt", i));
            assert!(file_path.exists(), "File {} should exist", i);
        }
    }

    #[tokio::test]
    async fn test_draft_manager_reset_to_main() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let repo = init_repo_with_commit(&fixture).await;

        // Initialize draft manager
        let draft_manager = DraftManager::new(fixture.path()).unwrap();
        draft_manager.initialize().await.unwrap();

        // Create a commit on draft branch
        fixture.add_text_file("draft_file.txt", "draft content");
        repo.stage_all().await.unwrap();
        let metadata = CommitMetadata::new("Draft commit");
        draft_manager.auto_commit(metadata).await.unwrap();

        // Verify draft branch has commits
        let count_before = draft_manager.draft_commit_count().unwrap();
        assert!(count_before >= 1, "Should have commits before reset");

        // Reset draft to main
        draft_manager.reset_to_main().await.unwrap();

        // Verify we're back on draft branch
        assert!(
            draft_manager.is_on_draft_branch().unwrap(),
            "Should be on draft branch after reset"
        );

        // Note: Draft file should be gone after reset since we reset the branch
        // The draft branch is now a fresh copy of main
        let draft_file_path = fixture.path().join("draft_file.txt");
        assert!(
            !draft_file_path.exists(),
            "Draft file should be gone after reset"
        );
    }

    #[tokio::test]
    async fn test_draft_manager_stats() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let repo = init_repo_with_commit(&fixture).await;

        // Initialize draft manager
        let draft_manager = DraftManager::new(fixture.path()).unwrap();
        draft_manager.initialize().await.unwrap();

        // Get initial stats
        let stats = draft_manager.get_stats().unwrap();
        assert!(stats.is_on_draft, "Should be on draft branch");
        assert_eq!(stats.current_branch, DraftManager::DEFAULT_DRAFT_BRANCH);
        assert_eq!(stats.draft_branch_name, DraftManager::DEFAULT_DRAFT_BRANCH);
        assert_eq!(stats.max_commits, DraftManager::DEFAULT_MAX_COMMITS);

        // Create a commit
        fixture.add_text_file("test.txt", "content");
        repo.stage_all().await.unwrap();
        let metadata = CommitMetadata::new("Test");
        draft_manager.auto_commit(metadata).await.unwrap();

        // Get updated stats
        let stats = draft_manager.get_stats().unwrap();
        assert!(
            stats.commit_count >= 1,
            "Commit count should be at least 1"
        );

        // Test print (should not panic)
        stats.print();
    }

    #[tokio::test]
    async fn test_draft_manager_custom_config() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let _repo = init_repo_with_commit(&fixture).await;

        // Create draft manager with custom config
        let custom_branch = "custom-draft";
        let custom_max = 50;
        let draft_manager =
            DraftManager::with_config(fixture.path(), custom_branch.to_string(), custom_max)
                .unwrap();

        draft_manager.initialize().await.unwrap();

        // Verify custom configuration
        assert!(
            draft_manager.draft_branch_exists().unwrap(),
            "Custom draft branch should exist"
        );
        assert_eq!(draft_manager.current_branch().unwrap(), custom_branch);

        let stats = draft_manager.get_stats().unwrap();
        assert_eq!(stats.draft_branch_name, custom_branch);
        assert_eq!(stats.max_commits, custom_max);
    }

    #[tokio::test]
    async fn test_draft_manager_auto_switch_to_draft() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let repo = init_repo_with_commit(&fixture).await;

        // Initialize draft manager
        let draft_manager = DraftManager::new(fixture.path()).unwrap();
        draft_manager.initialize().await.unwrap();

        // Switch to main
        draft_manager.switch_to_main().await.unwrap();
        assert!(!draft_manager.is_on_draft_branch().unwrap());

        // Auto-commit should switch to draft automatically
        fixture.add_text_file("test.txt", "content");
        repo.stage_all().await.unwrap();
        let metadata = CommitMetadata::new("Auto-commit from main");
        draft_manager.auto_commit(metadata).await.unwrap();

        // Verify we're now on draft branch
        assert!(
            draft_manager.is_on_draft_branch().unwrap(),
            "auto_commit should switch to draft branch automatically"
        );
    }

    #[tokio::test]
    async fn test_draft_manager_prune_if_needed() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();
        let _repo = init_repo_with_commit(&fixture).await;

        // Initialize draft manager
        let draft_manager = DraftManager::new(fixture.path()).unwrap();
        draft_manager.initialize().await.unwrap();

        // Call prune_if_needed (should do nothing since we haven't exceeded max)
        let result = draft_manager.prune_if_needed().await;
        assert!(
            result.is_ok(),
            "prune_if_needed should succeed even when not needed"
        );

        // Note: Testing actual pruning would require creating 100+ commits,
        // which is too slow for a unit test. The pruning logic is a placeholder anyway.
    }

    #[tokio::test]
    async fn test_draft_manager_fails_without_repo() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Try to create draft manager without initializing repo
        let result = DraftManager::new(fixture.path());
        assert!(
            result.is_err(),
            "DraftManager::new should fail without initialized repository"
        );

        if let Err(err) = result {
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("Repository not found")
                    || err_msg.contains("Run 'auxin init'"),
                "Error message should mention repository not found: {}",
                err_msg
            );
        }
    }
}
