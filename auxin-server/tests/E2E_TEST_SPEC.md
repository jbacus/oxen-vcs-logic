# End-to-End Test Specification with Real Oxen

**Status**: ✅ UNBLOCKED - Ready for implementation
**Previous Blocker**: Async refactoring (RESOLVED via auxin-oxen subprocess approach)
**Priority**: High (required before v0.3 Server Alpha)
**Est. Effort**: 1-2 days

## Recent Changes

**2025-11-22**: Created `auxin-oxen` shared crate using subprocess approach. This eliminates the need for async/await refactoring throughout the server. Real Oxen operations are now available via synchronous subprocess calls.

## Current State

- ✅ `auxin-oxen` crate provides subprocess-based Oxen integration
- ✅ Server `repo_full.rs` updated to use `OxenSubprocess`
- ✅ All Oxen operations (clone, push, pull, commit, etc.) are working
- ✅ 580 tests passing (auxin-oxen: 85, CLI: 426, server: 69)
- ⏳ E2E tests with real Oxen operations pending implementation

## Goal

Create comprehensive end-to-end tests that exercise:
1. Real Oxen VCS operations (not mocks)
2. Full request/response cycles via HTTP API
3. Multi-user collaboration scenarios
4. Large file handling (100MB+)
5. Lock contention and resolution

## Prerequisites

### 1. ~~Complete async/await Refactoring~~ ✅ NOT NEEDED

**Update**: We're using `auxin-oxen` subprocess approach which is synchronous. No async refactoring required.

Current implementation:
```rust
// repo_full.rs uses OxenSubprocess (synchronous)
impl RepositoryOps {
    pub fn clone(url: &str, path: impl AsRef<Path>) -> AppResult<Self> {
        let oxen = OxenSubprocess::new();
        oxen.clone(url, &path)?;
        Ok(Self { repo_path: path.as_ref().to_path_buf(), oxen })
    }
}
```

### 2. Test Environment Setup

```bash
# Install Oxen CLI (required for subprocess approach)
pip install oxenai

# Verify installation
oxen --version

# Optional: Create test Oxen Hub account for remote testing
# export OXEN_HUB_URL="https://hub.oxen.ai"
# export OXEN_HUB_TOKEN="your_test_token"
```

### 3. Test Data

For local testing (no network required):
- Use `file://` URLs to test clone/push/pull locally
- Create temporary test repositories
- Test with varying file sizes (1MB, 10MB, 100MB)

## Test Scenarios

### Scenario 1: Basic Workflow with Real Oxen (Local)

```rust
#[test]
fn test_real_oxen_init_add_commit() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");

    // 1. Initialize repository
    let repo = RepositoryOps::init(&repo_path).unwrap();

    // Verify .oxen directory structure
    assert!(repo_path.join(".oxen").exists());
    assert!(repo_path.join(".oxen/HEAD").exists());
    assert!(repo_path.join(".oxen/config.toml").exists());

    // 2. Create test file
    std::fs::write(repo_path.join("test.txt"), "Hello Oxen").unwrap();

    // 3. Add file
    repo.add(&[std::path::Path::new("test.txt")]).unwrap();

    // 4. Commit
    let commit_id = repo.commit("Add test file").unwrap();
    assert!(commit_id.len() >= 7); // Short hash

    // 5. Verify commit history
    let commits = repo.log(None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].message, "Add test file");
}
```

### Scenario 2: Clone and Pull Workflow

```rust
#[test]
fn test_real_oxen_clone_local() {
    // Create source repository
    let source_dir = TempDir::new().unwrap();
    let source_path = source_dir.path().join("source");
    let source_repo = RepositoryOps::init(&source_path).unwrap();

    // Add content to source
    std::fs::write(source_path.join("data.txt"), "source data").unwrap();
    source_repo.add(&[Path::new("data.txt")]).unwrap();
    source_repo.commit("Initial commit").unwrap();

    // Clone to destination
    let dest_dir = TempDir::new().unwrap();
    let dest_path = dest_dir.path().join("dest");
    let clone_url = format!("file://{}", source_path.display());

    let dest_repo = RepositoryOps::clone(&clone_url, &dest_path).unwrap();

    // Verify cloned content
    assert!(dest_path.join("data.txt").exists());
    let content = std::fs::read_to_string(dest_path.join("data.txt")).unwrap();
    assert_eq!(content, "source data");

    // Verify commit history
    let commits = dest_repo.log(None).unwrap();
    assert_eq!(commits.len(), 1);
}
```

### Scenario 3: Multi-User Collaboration with Locks

```rust
#[test]
fn test_lock_contention_real_oxen() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("collab-repo");
    let repo = RepositoryOps::init(&repo_path).unwrap();

    // User A acquires lock
    let lock_a = repo.acquire_lock("user_a", "machine_a", 1).unwrap();
    assert!(!lock_a.lock_id.is_empty());

    // User B tries to acquire lock (should fail)
    let result = repo.acquire_lock("user_b", "machine_b", 1);
    assert!(result.is_err());

    // Verify lock status
    let status = repo.lock_status().unwrap();
    assert!(status.is_some());
    assert_eq!(status.unwrap().user, "user_a");

    // User A releases lock
    repo.release_lock(&lock_a.lock_id).unwrap();

    // User B can now acquire lock
    let lock_b = repo.acquire_lock("user_b", "machine_b", 1).unwrap();
    assert!(!lock_b.lock_id.is_empty());
}
```

### Scenario 4: Branch Operations

```rust
#[test]
fn test_branch_operations_real_oxen() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("branch-repo");
    let repo = RepositoryOps::init(&repo_path).unwrap();

    // Create initial commit
    std::fs::write(repo_path.join("file.txt"), "content").unwrap();
    repo.add(&[Path::new("file.txt")]).unwrap();
    repo.commit("Initial commit").unwrap();

    // List branches (should have main)
    let branches = repo.list_branches().unwrap();
    assert!(branches.contains(&"main".to_string()));

    // Create new branch
    repo.create_branch("feature").unwrap();

    // List branches again
    let branches = repo.list_branches().unwrap();
    assert!(branches.contains(&"feature".to_string()));
    assert_eq!(branches.len(), 2);

    // Checkout feature branch
    repo.checkout("feature").unwrap();
    let current = repo.current_branch().unwrap();
    assert_eq!(current, "feature");
}
```

### Scenario 5: Large File Handling

```rust
#[test]
#[ignore] // Expensive test - run manually
fn test_large_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("large-repo");
    let repo = RepositoryOps::init(&repo_path).unwrap();

    // Create 100MB test file
    let large_file = repo_path.join("large.bin");
    let mut file = std::fs::File::create(&large_file).unwrap();
    file.write_all(&vec![0u8; 100 * 1024 * 1024]).unwrap();

    // Add and commit
    let start = std::time::Instant::now();
    repo.add(&[Path::new("large.bin")]).unwrap();
    repo.commit("Add 100MB file").unwrap();
    let duration = start.elapsed();

    println!("Large file add+commit took: {:?}", duration);

    // Verify commit
    let commits = repo.log(None).unwrap();
    assert_eq!(commits.len(), 1);

    // Clone to verify deduplication
    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("clone");
    let clone_url = format!("file://{}", repo_path.display());

    let cloned = RepositoryOps::clone(&clone_url, &clone_path).unwrap();
    assert!(clone_path.join("large.bin").exists());
}
```

### Scenario 6: HTTP API Integration Test

```rust
#[actix_web::test]
async fn test_api_with_real_oxen() {
    use actix_web::{test, App};
    use auxin_server::api::repo_ops;

    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .service(repo_ops::init_repository)
            .service(repo_ops::list_commits)
    ).await;

    // Initialize repository via API
    let req = test::TestRequest::post()
        .uri("/api/repos/testuser/testrepo/init")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // List commits (should be empty for new repo)
    let req = test::TestRequest::get()
        .uri("/api/repos/testuser/testrepo/commits")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let commits: Vec<CommitInfo> = test::read_body_json(resp).await;
    assert_eq!(commits.len(), 0);
}
```

## Implementation Checklist

- [x] ~~Complete async/await refactoring~~ Not needed with subprocess approach
- [x] RepositoryOps uses OxenSubprocess
- [ ] Implement Scenario 1: Basic init/add/commit workflow
- [ ] Implement Scenario 2: Clone and pull workflow
- [ ] Implement Scenario 3: Lock contention
- [ ] Implement Scenario 4: Branch operations
- [ ] Implement Scenario 5: Large file handling (marked `#[ignore]`)
- [ ] Implement Scenario 6: HTTP API integration
- [ ] Add helper functions for test setup/teardown
- [ ] Document running tests in README

## Running Tests

```bash
# Run all E2E tests with real Oxen
cargo test --test e2e_real_oxen

# Run specific test
cargo test --test e2e_real_oxen test_real_oxen_init_add_commit

# Run expensive tests (large files)
cargo test --test e2e_real_oxen -- --ignored --test-threads=1

# Run with verbose output
cargo test --test e2e_real_oxen -- --nocapture
```

## File Structure

Create new test file:
```
auxin-server/tests/
  ├── e2e_real_oxen.rs          # NEW: Real Oxen E2E tests
  ├── collaboration_e2e_tests.rs # Existing: Mock-based E2E
  ├── api_tests.rs               # Existing: API unit tests
  └── E2E_TEST_SPEC.md          # This file
```

## Notes

- Tests use local `file://` URLs for fast, network-free testing
- All tests create temporary directories (auto-cleaned)
- No external Oxen Hub account required for basic tests
- Large file tests marked `#[ignore]` to avoid slowing down CI
- Mock tests remain for fast iteration; real E2E tests for validation

---

**Status**: Ready to implement! The auxin-oxen refactoring removed the async blocker.
**Next Step**: Create `auxin-server/tests/e2e_real_oxen.rs` and implement scenarios 1-4.
