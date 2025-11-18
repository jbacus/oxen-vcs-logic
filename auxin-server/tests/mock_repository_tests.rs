// Integration tests for mock repository implementation
// These tests verify the mock-oxen feature works correctly

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

    let repo = result.unwrap();

    // Verify .oxen directory was created
    assert!(repo_path.join(".oxen").exists());
    assert!(repo_path.join(".oxen/HEAD").exists());
    assert!(repo_path.join(".oxen/refs/heads").exists());

    // Verify Auxin extensions
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
fn test_mock_vcs_operations_return_not_implemented() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Create a test file
    let test_file = repo_path.join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    // Try VCS operations - should return NotImplemented
    let add_result = repo.add(&[test_file.as_path()]);
    assert!(add_result.is_err());
    match add_result {
        Err(AppError::NotImplemented(_)) => {}, // Expected
        _ => panic!("Expected NotImplemented error for add"),
    }

    let commit_result = repo.commit("test commit");
    assert!(commit_result.is_err());
    match commit_result {
        Err(AppError::NotImplemented(_)) => {}, // Expected
        _ => panic!("Expected NotImplemented error for commit"),
    }

    let push_result = repo.push("origin", "main");
    assert!(push_result.is_err());
    match push_result {
        Err(AppError::NotImplemented(_)) => {}, // Expected
        _ => panic!("Expected NotImplemented error for push"),
    }

    let pull_result = repo.pull("origin", "main");
    assert!(pull_result.is_err());
    match pull_result {
        Err(AppError::NotImplemented(_)) => {}, // Expected
        _ => panic!("Expected NotImplemented error for pull"),
    }

    let branch_result = repo.create_branch("feature");
    assert!(branch_result.is_err());
    match branch_result {
        Err(AppError::NotImplemented(_)) => {}, // Expected
        _ => panic!("Expected NotImplemented error for create_branch"),
    }

    let checkout_result = repo.checkout("feature");
    assert!(checkout_result.is_err());
    match checkout_result {
        Err(AppError::NotImplemented(_)) => {}, // Expected
        _ => panic!("Expected NotImplemented error for checkout"),
    }
}

#[test]
fn test_mock_current_branch() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Current branch should work in mock mode
    let branch = repo.current_branch();
    assert!(branch.is_ok());
    assert_eq!(branch.unwrap(), "main");
}

#[test]
fn test_mock_list_branches() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // List branches should work in mock mode
    let branches = repo.list_branches();
    assert!(branches.is_ok());
    assert_eq!(branches.unwrap(), vec!["main"]);
}

#[test]
fn test_mock_log_returns_empty() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    let repo = RepositoryOps::init(repo_path).unwrap();

    // Log should return empty list in mock mode
    let commits = repo.log(None);
    assert!(commits.is_ok());
    assert_eq!(commits.unwrap().len(), 0);
}

#[test]
fn test_mock_clone_not_implemented() {
    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("cloned");

    let result = RepositoryOps::clone("https://example.com/repo.git", &dest_path);
    assert!(result.is_err());
    match result {
        Err(AppError::NotImplemented(_)) => {}, // Expected
        _ => panic!("Expected NotImplemented error for clone"),
    }
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
