// Integration tests for mock repository implementation
// These tests verify the mock-oxen feature works correctly
// Note: The mock implementation uses the Oxen CLI when available,
// or creates a minimal directory structure when it's not.

use auxin_server::error::AppError;
use auxin_server::repo::RepositoryOps;
use tempfile::TempDir;

#[test]
fn test_mock_repo_init() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize repository
    let result = RepositoryOps::init(repo_path);
    assert!(result.is_ok(), "Failed to initialize repository");

    let _repo = result.unwrap();

    // Verify basic .oxen directory structure was created
    // (minimal structure is always created, CLI adds more if available)
    assert!(repo_path.join(".oxen").exists());

    // Verify Auxin extensions (always created)
    assert!(repo_path.join(".oxen/metadata").exists());
    assert!(repo_path.join(".oxen/locks").exists());
}

#[test]
fn test_mock_repo_open() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize first
    RepositoryOps::init(repo_path).unwrap();

    // Then open
    let result = RepositoryOps::open(repo_path);
    assert!(result.is_ok(), "Failed to open repository");
}

#[test]
fn test_mock_repo_open_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("nonexistent");

    // Try to open non-existent repo
    let result = RepositoryOps::open(&repo_path);
    assert!(result.is_err());

    match result {
        Err(AppError::NotFound(_)) => {}, // Expected
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_mock_vcs_operations_require_cli() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Create a test file
    let test_file = repo_path.join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    // VCS operations require CLI - they will return Internal error if CLI not available
    // When CLI is available, they may succeed or fail depending on repo state
    // This test just verifies the operations don't panic
    let _add_result = repo.add(&[test_file.as_path()]);
    let _commit_result = repo.commit("test commit");
    let _push_result = repo.push("origin", "main");
    let _pull_result = repo.pull("origin", "main");
    let _branch_result = repo.create_branch("feature");
    let _checkout_result = repo.checkout("feature");

    // If we get here without panic, the operations handled properly
}

#[test]
fn test_mock_current_branch() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Current branch requires CLI - just verify it doesn't panic
    let _branch = repo.current_branch();
}

#[test]
fn test_mock_list_branches() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // List branches requires CLI - just verify it doesn't panic
    let _branches = repo.list_branches();
}

#[test]
fn test_mock_log() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Log requires CLI - just verify it doesn't panic
    let _commits = repo.log(None);
}

#[test]
fn test_mock_clone_requires_cli() {
    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("cloned");

    // Clone requires CLI - will fail without it
    let result = RepositoryOps::clone("https://example.com/repo.git", &dest_path);
    // Just verify it returns an error (Internal when CLI not available)
    assert!(result.is_err());
}

#[test]
fn test_auxin_extensions_work_in_mock() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Auxin extensions should work in mock mode
    use auxin_server::extensions::LogicProMetadata;

    let metadata = LogicProMetadata {
        bpm: Some(120.0),
        sample_rate: Some(48000),
        key_signature: Some("C Major".to_string()),
        tags: vec![],
    };

    // Store metadata
    let store_result = repo.store_metadata("test-commit-id", &metadata);
    assert!(store_result.is_ok(), "Failed to store metadata: {:?}", store_result);

    // Retrieve metadata
    let get_result = repo.get_metadata("test-commit-id");
    assert!(get_result.is_ok());
    let retrieved = get_result.unwrap();
    assert!(retrieved.is_some());

    let retrieved_metadata = retrieved.unwrap();
    assert_eq!(retrieved_metadata.bpm, Some(120.0));
    assert_eq!(retrieved_metadata.sample_rate, Some(48000));
    assert_eq!(retrieved_metadata.key_signature, Some("C Major".to_string()));
}

#[test]
fn test_auxin_locks_work_in_mock() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Lock operations should work in mock mode
    let lock_result = repo.acquire_lock("test-user", "test-machine", 24);
    assert!(lock_result.is_ok(), "Failed to acquire lock: {:?}", lock_result);

    let lock = lock_result.unwrap();
    assert_eq!(lock.user, "test-user");
    assert_eq!(lock.machine_id, "test-machine");

    // Check lock status
    let status = repo.lock_status();
    assert!(status.is_ok());
    assert!(status.unwrap().is_some());

    // Release lock
    let release_result = repo.release_lock(&lock.lock_id);
    assert!(release_result.is_ok());

    // Verify lock is released
    let status_after = repo.lock_status();
    assert!(status_after.is_ok());
    assert!(status_after.unwrap().is_none());
}

#[test]
fn test_mock_repo_path() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Verify path() returns correct path
    assert_eq!(repo.path(), repo_path);
}
