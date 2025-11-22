# End-to-End Test Specification with Real Oxen

**Status**: Pending implementation (blocked on full-oxen feature async refactoring)
**Priority**: High (required before v1.0)
**Est. Effort**: 2-3 days

## Current State

- Existing tests use `mock-oxen` feature (default)
- E2E tests exist at API level (`collaboration_e2e_tests.rs`)
- Real Oxen operations are mocked out
- Full integration requires `full-oxen` feature which is WIP

## Goal

Create comprehensive end-to-end tests that exercise:
1. Real Oxen VCS operations (not mocks)
2. Full request/response cycles
3. Multi-user collaboration scenarios
4. Large file handling (1GB+)
5. Network resilience

## Prerequisites

### 1. Complete async/await Refactoring

The `full-oxen` feature requires liboxen 0.38+ which uses async/await. Before E2E tests can run with real Oxen:

```rust
// Current (blocking):
impl RepositoryOps {
    pub fn clone(url: &str, path: &Path) -> Result<Self> {
        // Synchronous oxen operations
    }
}

// Needed (async):
impl RepositoryOps {
    pub async fn clone(url: &str, path: &Path) -> Result<Self> {
        // Async oxen operations using liboxen 0.38
    }
}
```

**Files to update**:
- `auxin-server/src/repo_full.rs` - Add async/await
- `auxin-server/src/api/repo_ops.rs` - Update handlers to use async repo methods
- All integration tests - Update to handle async operations

### 2. Test Environment Setup

```bash
# Install Oxen CLI
pip install oxenai

# Create test Oxen Hub account
# Set environment variables:
export OXEN_HUB_URL="https://hub.oxen.ai"
export OXEN_HUB_TOKEN="your_test_token"
export OXEN_TEST_REPO_URL="https://hub.oxen.ai/testuser/testrepo"
```

### 3. Test Data

Prepare test fixtures:
- Small test repository (~10MB)
- Large repository (1GB+ Logic Pro project)
- Repository with binary files
- Repository with many commits (100+)

## Test Scenarios

### Scenario 1: Basic Workflow with Real Oxen

```rust
#[actix_web::test]
#[cfg(feature = "full-oxen")]
async fn test_real_oxen_clone_push_pull() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let app = create_test_app(config).await;

    // 1. Clone from real Oxen Hub
    let clone_req = json!({
        "remote_url": env::var("OXEN_TEST_REPO_URL").unwrap()
    });

    let resp = test::TestRequest::post()
        .uri("/api/repos/testuser/testrepo/clone")
        .set_json(&clone_req)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 201);

    // 2. Verify .oxen directory structure
    let repo_path = temp_dir.path().join("testuser/testrepo");
    assert!(repo_path.join(".oxen").exists());
    assert!(repo_path.join(".oxen/HEAD").exists());

    // 3. Make local changes
    std::fs::write(repo_path.join("test.txt"), "Hello Oxen").unwrap();

    // 4. Commit (would need real oxen commit operation)
    // This requires implementing actual commit endpoint

    // 5. Push to remote
    let push_req = json!({
        "remote": "origin",
        "branch": "main"
    });

    let resp = test::TestRequest::post()
        .uri("/api/repos/testuser/testrepo/push")
        .set_json(&push_req)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 200);

    // 6. Pull from another instance (simulating 2nd user)
    let temp_dir2 = TempDir::new().unwrap();
    let config2 = test_config(&temp_dir2);
    let app2 = create_test_app(config2).await;

    // Clone to 2nd instance
    let resp = test::TestRequest::post()
        .uri("/api/repos/testuser2/testrepo/clone")
        .set_json(&clone_req)
        .send_request(&app2)
        .await;

    assert_eq!(resp.status(), 201);

    // Verify file exists in 2nd clone
    let repo_path2 = temp_dir2.path().join("testuser2/testrepo");
    assert!(repo_path2.join("test.txt").exists());

    // Cleanup
    // Delete test data from Oxen Hub if needed
}
```

### Scenario 2: Large File Upload (1GB+)

```rust
#[actix_web::test]
#[cfg(feature = "full-oxen")]
#[ignore] // Expensive test - run manually
async fn test_large_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);

    // Create 1GB test file
    let large_file = temp_dir.path().join("large.bin");
    create_test_file(&large_file, 1024 * 1024 * 1024); // 1GB

    // Initialize repository
    let repo = RepositoryOps::init(temp_dir.path()).await.unwrap();

    // Add large file
    repo.add(&large_file).await.unwrap();

    // Commit
    repo.commit("Add 1GB file").await.unwrap();

    // Push (should use chunked upload)
    let start = std::time::Instant::now();
    repo.push("origin", "main").await.unwrap();
    let duration = start.elapsed();

    println!("Large file push took: {:?}", duration);

    // Verify push succeeded
    // Clone to new location and verify file
    let clone_dir = TempDir::new().unwrap();
    let cloned = RepositoryOps::clone(&format!("file://{}", temp_dir.path().display()), clone_dir.path()).await.unwrap();

    assert!(clone_dir.path().join("large.bin").exists());
    assert_eq!(
        std::fs::metadata(clone_dir.path().join("large.bin")).unwrap().len(),
        1024 * 1024 * 1024
    );
}
```

### Scenario 3: Multi-User Collaboration

```rust
#[actix_web::test]
#[cfg(feature = "full-oxen")]
async fn test_multi_user_real_collaboration() {
    // User A creates repo and pushes
    let user_a_dir = TempDir::new().unwrap();
    let repo_a = RepositoryOps::init(&user_a_dir.path()).await.unwrap();

    std::fs::write(user_a_dir.path().join("track1.wav"), b"audio data").unwrap();
    repo_a.add("track1.wav").await.unwrap();
    repo_a.commit("Add track 1").await.unwrap();
    repo_a.push("origin", "main").await.unwrap();

    // User B clones
    let user_b_dir = TempDir::new().unwrap();
    let repo_b = RepositoryOps::clone(&repo_url, &user_b_dir.path()).await.unwrap();

    // User B acquires lock
    let lock = repo_b.acquire_lock("user_b", "machine_b", 1).await.unwrap();
    assert!(lock.lock_id.len() > 0);

    // User A tries to acquire lock (should fail)
    let result = repo_a.acquire_lock("user_a", "machine_a", 1).await;
    assert!(result.is_err());

    // User B makes changes
    std::fs::write(user_b_dir.path().join("track2.wav"), b"more audio").unwrap();
    repo_b.add("track2.wav").await.unwrap();
    repo_b.commit("Add track 2").await.unwrap();
    repo_b.push("origin", "main").await.unwrap();

    // User B releases lock
    repo_b.release_lock(&lock.lock_id).await.unwrap();

    // User A pulls changes
    repo_a.pull("origin", "main").await.unwrap();

    // Verify User A sees track2.wav
    assert!(user_a_dir.path().join("track2.wav").exists());
}
```

### Scenario 4: Network Resilience

```rust
#[actix_web::test]
#[cfg(feature = "full-oxen")]
async fn test_network_failure_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let repo = RepositoryOps::clone(&remote_url, &temp_dir.path()).await.unwrap();

    // Start large push
    std::fs::write(temp_dir.path().join("large.bin"), vec![0u8; 500 * 1024 * 1024]).unwrap(); // 500MB
    repo.add("large.bin").await.unwrap();
    repo.commit("Add large file").await.unwrap();

    // Push with simulated network interruption
    // This would require mocking network layer or using test proxy
    // For now, just verify push works end-to-end
    let result = repo.push("origin", "main").await;
    assert!(result.is_ok());

    // Verify can resume/retry push
    // (Implementation depends on Oxen's resume capabilities)
}
```

## Implementation Checklist

- [ ] Complete async/await refactoring in `repo_full.rs`
- [ ] Update all API handlers to use async repository operations
- [ ] Set up test Oxen Hub account and test repositories
- [ ] Create test data fixtures (small, medium, large repos)
- [ ] Implement Scenario 1: Basic workflow
- [ ] Implement Scenario 2: Large file handling
- [ ] Implement Scenario 3: Multi-user collaboration
- [ ] Implement Scenario 4: Network resilience
- [ ] Add performance benchmarks (time tracking for operations)
- [ ] Add assertions for Oxen-specific features (deduplication, versioning)
- [ ] Document test setup in CI/CD pipeline
- [ ] Create cleanup procedures for test data

## Running Tests

Once implemented:

```bash
# Run with full Oxen (requires liboxen installed)
cargo test --features full-oxen

# Run expensive tests
cargo test --features full-oxen -- --ignored

# Run specific scenario
cargo test --features full-oxen test_real_oxen_clone_push_pull
```

## Notes

- Tests require network access to Oxen Hub (or local Oxen server)
- Large file tests should be marked `#[ignore]` to avoid slowing down CI
- Consider using Docker to spin up local Oxen server for isolated testing
- Mock tests will remain for fast iteration; real E2E tests are for validation

---

**Next Steps**: Complete async refactoring, then implement tests in priority order (Scenario 1 → 2 → 3 → 4)
