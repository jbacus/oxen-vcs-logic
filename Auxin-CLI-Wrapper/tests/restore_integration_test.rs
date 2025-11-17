/// Integration tests for restore command with short hash expansion
///
/// These tests require the `oxen` CLI to be installed:
///   pip3 install oxen-ai
///   or
///   cargo install oxen
///
/// To run these tests:
///   cargo test --test restore_integration_test -- --nocapture
#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use super::common::TestFixture;
    use auxin::OxenRepository;
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

    #[tokio::test]
    async fn test_restore_with_full_hash() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Initialize repository
        let _repo = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());
        fixture.add_text_file("file.txt", "version 1");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("First commit");
        let first_commit = repo.create_commit(metadata).await.unwrap();

        // Create second commit
        fixture.add_text_file("file.txt", "version 2");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("Second commit");
        repo.create_commit(metadata).await.unwrap();

        // Restore to first commit using full hash
        let result = repo.restore(&first_commit).await;
        assert!(result.is_ok(), "Restore with full hash should succeed");

        // Verify file was restored
        let content = fs::read_to_string(fixture.path().join("file.txt")).unwrap();
        assert_eq!(content, "version 1", "File should be restored to version 1");
    }

    #[tokio::test]
    async fn test_restore_with_short_hash() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Initialize repository
        let _init = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());
        fixture.add_text_file("file.txt", "version 1");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("First commit");
        let first_commit = repo.create_commit(metadata).await.unwrap();

        // Create second commit
        fixture.add_text_file("file.txt", "version 2");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("Second commit");
        repo.create_commit(metadata).await.unwrap();

        // Restore to first commit using short hash (first 8 characters)
        let short_hash = &first_commit[..8];
        let result = repo.restore(short_hash).await;
        assert!(
            result.is_ok(),
            "Restore with short hash should succeed: {:?}",
            result.err()
        );

        // Verify file was restored
        let content = fs::read_to_string(fixture.path().join("file.txt")).unwrap();
        assert_eq!(
            content, "version 1",
            "File should be restored to version 1 using short hash"
        );
    }

    #[tokio::test]
    async fn test_restore_with_very_short_hash() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Initialize repository
        let _init = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());
        fixture.add_text_file("file.txt", "version 1");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("First commit");
        let first_commit = repo.create_commit(metadata).await.unwrap();

        // Create second commit
        fixture.add_text_file("file.txt", "version 2");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("Second commit");
        repo.create_commit(metadata).await.unwrap();

        // Restore to first commit using very short hash (first 7 characters - Git standard)
        let short_hash = &first_commit[..7];
        let result = repo.restore(short_hash).await;
        assert!(
            result.is_ok(),
            "Restore with 7-char short hash should succeed: {:?}",
            result.err()
        );

        // Verify file was restored
        let content = fs::read_to_string(fixture.path().join("file.txt")).unwrap();
        assert_eq!(
            content, "version 1",
            "File should be restored to version 1 using 7-char short hash"
        );
    }

    #[tokio::test]
    async fn test_restore_with_invalid_hash() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Initialize repository
        let _init = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());
        fixture.add_text_file("file.txt", "version 1");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("First commit");
        repo.create_commit(metadata).await.unwrap();

        // Try to restore to non-existent short hash
        let result = repo.restore("invalid_hash").await;
        assert!(
            result.is_err(),
            "Restore with invalid hash should fail (was silently succeeding before fix)"
        );

        // Verify error message is helpful
        let err = result.unwrap_err();
        let err_msg = err.to_string().to_lowercase();
        assert!(
            err_msg.contains("no commit found") || err_msg.contains("not found"),
            "Error should mention commit not found: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_restore_with_nonexistent_short_hash() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Initialize repository
        let _init = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());
        fixture.add_text_file("file.txt", "version 1");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("First commit");
        repo.create_commit(metadata).await.unwrap();

        // Try to restore to a hash-like prefix that doesn't match any commit
        let result = repo.restore("ffffffff").await;
        assert!(
            result.is_err(),
            "Restore with non-existent short hash should fail"
        );

        // Verify error message
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("No commit found matching prefix"),
            "Error should mention no commit found: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_restore_preserves_untracked_files() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Initialize repository
        let _init = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());
        fixture.add_text_file("tracked.txt", "version 1");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("First commit");
        let first_commit = repo.create_commit(metadata).await.unwrap();

        // Create second commit
        fixture.add_text_file("tracked.txt", "version 2");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("Second commit");
        repo.create_commit(metadata).await.unwrap();

        // Create an untracked file
        fixture.add_text_file("untracked.txt", "should be preserved");

        // Restore to first commit
        let short_hash = &first_commit[..8];
        repo.restore(short_hash).await.unwrap();

        // Verify tracked file was restored
        let tracked = fs::read_to_string(fixture.path().join("tracked.txt")).unwrap();
        assert_eq!(tracked, "version 1", "Tracked file should be restored");

        // Verify untracked file still exists
        let untracked = fs::read_to_string(fixture.path().join("untracked.txt")).unwrap();
        assert_eq!(
            untracked, "should be preserved",
            "Untracked file should be preserved"
        );
    }

    #[tokio::test]
    async fn test_restore_multiple_times() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Initialize repository
        let _init = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());

        fixture.add_text_file("file.txt", "version 1");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("First commit");
        let commit1 = repo.create_commit(metadata).await.unwrap();

        fixture.add_text_file("file.txt", "version 2");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("Second commit");
        let commit2 = repo.create_commit(metadata).await.unwrap();

        fixture.add_text_file("file.txt", "version 3");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("Third commit");
        repo.create_commit(metadata).await.unwrap();

        // Restore to commit 1 using full hash (checkout changes context)
        repo.restore(&commit1).await.unwrap();
        let content = fs::read_to_string(fixture.path().join("file.txt")).unwrap();
        assert_eq!(content, "version 1");

        // Restore to commit 2 using full hash
        repo.restore(&commit2).await.unwrap();
        let content = fs::read_to_string(fixture.path().join("file.txt")).unwrap();
        assert_eq!(content, "version 2");

        // Restore back to commit 1 using full hash
        repo.restore(&commit1).await.unwrap();
        let content = fs::read_to_string(fixture.path().join("file.txt")).unwrap();
        assert_eq!(content, "version 1");
    }

    #[tokio::test]
    async fn test_restore_ambiguous_hash_would_fail() {
        skip_if_no_oxen!();

        let fixture = TestFixture::new();

        // Initialize repository
        let _init = OxenRepository::init(fixture.path()).await.unwrap();
        let repo = OxenRepository::new(fixture.path());

        // In practice, it's very unlikely to have ambiguous hashes with even 4-5 characters,
        // but if it happens, our code should error
        // This test documents expected behavior rather than testing a realistic scenario

        fixture.add_text_file("file1.txt", "content 1");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("Commit 1");
        repo.create_commit(metadata).await.unwrap();

        fixture.add_text_file("file2.txt", "content 2");
        repo.stage_all().await.unwrap();
        let metadata = auxin::CommitMetadata::new("Commit 2");
        repo.create_commit(metadata).await.unwrap();

        // Try using single character (very likely ambiguous if commits exist)
        // This should either work (if unique) or error (if ambiguous)
        let result = repo.restore("a").await;
        // We don't assert success or failure - just that it doesn't panic
        // If it fails, the error should mention ambiguity
        if let Err(e) = result {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("Ambiguous") || err_msg.contains("No commit found"),
                "Error should be clear about issue: {}",
                err_msg
            );
        }
    }
}
