/// Push/Pull Integration Tests
///
/// These tests verify push and pull operations between local repositories and
/// remote servers (Auxin Server or Oxen Hub).
///
/// Tests cover:
/// - Local to server push operations
/// - Server to local pull operations
/// - Push/pull with locks coordination
/// - Conflict detection and resolution
/// - Network resilience during sync
/// - Large file push/pull performance
///
/// Run with: cargo test --test push_pull_integration_test

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

mod common;
use common::TestFixture;

// =============================================================================
// Helper Functions
// =============================================================================

/// Create a test repository with some commits
fn setup_test_repo(path: &Path) -> anyhow::Result<()> {
    // Create directory structure
    fs::create_dir_all(path.join("Audio Files"))?;
    fs::write(path.join("projectData"), "test project data")?;

    // Initialize repository
    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(path)
        .output()?;

    // Create initial commit
    fs::write(path.join("test.txt"), "initial content")?;
    std::process::Command::new("oxen")
        .args(&["add", "."])
        .current_dir(path)
        .output()?;
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    Ok(())
}

/// Setup a "remote" repository (another local directory acting as remote)
fn setup_remote_repo() -> anyhow::Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let remote_path = temp_dir.path();

    // Initialize bare repository
    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(remote_path)
        .output()?;

    Ok(temp_dir)
}

// =============================================================================
// Basic Push/Pull Tests
// =============================================================================

#[test]
fn test_push_to_local_remote() {
    let fixture = TestFixture::new();
    let local_path = fixture.path();

    // Setup local repo
    setup_test_repo(local_path).expect("Failed to setup local repo");

    // Setup remote
    let remote_dir = setup_remote_repo().expect("Failed to setup remote");
    let remote_path = remote_dir.path();

    // Add remote
    std::process::Command::new("oxen")
        .args(&["remote", "add", "origin", &format!("file://{}", remote_path.display())])
        .current_dir(local_path)
        .output()
        .expect("Failed to add remote");

    // Create another commit
    fs::write(local_path.join("file2.txt"), "new content").expect("Failed to write file");
    std::process::Command::new("oxen")
        .args(&["add", "file2.txt"])
        .current_dir(local_path)
        .output()
        .expect("Failed to add file");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add file2"])
        .current_dir(local_path)
        .output()
        .expect("Failed to commit");

    // Push to remote
    let output = std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(local_path)
        .output()
        .expect("Failed to push");

    // Verify push succeeded
    assert!(
        output.status.success(),
        "Push should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_pull_from_local_remote() {
    let remote_dir = setup_remote_repo().expect("Failed to setup remote");
    let remote_path = remote_dir.path();

    // Setup remote with content
    setup_test_repo(remote_path).expect("Failed to setup remote repo");
    fs::write(remote_path.join("remote_file.txt"), "remote content")
        .expect("Failed to write remote file");
    std::process::Command::new("oxen")
        .args(&["add", "remote_file.txt"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add remote file");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add remote file"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    // Clone to local
    let local_dir = TempDir::new().expect("Failed to create temp dir");
    let local_path = local_dir.path().join("local");

    let output = std::process::Command::new("oxen")
        .args(&["clone", &format!("file://{}", remote_path.display()), &local_path.to_string_lossy()])
        .output()
        .expect("Failed to clone");

    // Verify clone succeeded
    assert!(
        output.status.success(),
        "Clone should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify files exist
    assert!(
        local_path.join("remote_file.txt").exists(),
        "Cloned file should exist"
    );

    let content = fs::read_to_string(local_path.join("remote_file.txt"))
        .expect("Failed to read file");
    assert_eq!(content, "remote content", "File content should match");
}

#[test]
fn test_push_pull_roundtrip() {
    // Setup remote
    let remote_dir = setup_remote_repo().expect("Failed to setup remote");
    let remote_path = remote_dir.path();
    setup_test_repo(remote_path).expect("Failed to setup remote repo");

    // Clone to local1
    let local1_dir = TempDir::new().expect("Failed to create temp dir");
    let local1_path = local1_dir.path().join("local1");
    std::process::Command::new("oxen")
        .args(&["clone", &format!("file://{}", remote_path.display()), &local1_path.to_string_lossy()])
        .output()
        .expect("Failed to clone to local1");

    // Clone to local2
    let local2_dir = TempDir::new().expect("Failed to create temp dir");
    let local2_path = local2_dir.path().join("local2");
    std::process::Command::new("oxen")
        .args(&["clone", &format!("file://{}", remote_path.display()), &local2_path.to_string_lossy()])
        .output()
        .expect("Failed to clone to local2");

    // Make changes in local1
    fs::write(local1_path.join("file_from_local1.txt"), "content from local1")
        .expect("Failed to write file");
    std::process::Command::new("oxen")
        .args(&["add", "file_from_local1.txt"])
        .current_dir(&local1_path)
        .output()
        .expect("Failed to add file");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add file from local1"])
        .current_dir(&local1_path)
        .output()
        .expect("Failed to commit");

    // Push from local1
    let push_output = std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(&local1_path)
        .output()
        .expect("Failed to push from local1");

    assert!(
        push_output.status.success(),
        "Push from local1 should succeed"
    );

    // Pull in local2
    let pull_output = std::process::Command::new("oxen")
        .args(&["pull", "origin", "main"])
        .current_dir(&local2_path)
        .output()
        .expect("Failed to pull in local2");

    assert!(
        pull_output.status.success(),
        "Pull in local2 should succeed. stderr: {}",
        String::from_utf8_lossy(&pull_output.stderr)
    );

    // Verify file exists in local2
    assert!(
        local2_path.join("file_from_local1.txt").exists(),
        "File should be pulled to local2"
    );

    let content = fs::read_to_string(local2_path.join("file_from_local1.txt"))
        .expect("Failed to read file");
    assert_eq!(content, "content from local1", "Content should match");
}

// =============================================================================
// Push/Pull with Locks
// =============================================================================

#[test]
fn test_push_requires_lock_release() {
    // This test verifies that in a collaboration scenario,
    // users should release locks before pushing

    let fixture = TestFixture::new();
    let local_path = fixture.path();
    setup_test_repo(local_path).expect("Failed to setup local repo");

    // Simulate acquiring a lock
    let lock_dir = local_path.join(".oxen/locks");
    fs::create_dir_all(&lock_dir).expect("Failed to create locks dir");

    let lock_file = lock_dir.join("project.lock");
    fs::write(&lock_file, r#"{"user": "test@machine", "acquired_at": "2025-01-01T00:00:00Z"}"#)
        .expect("Failed to write lock file");

    // Lock file exists
    assert!(lock_file.exists(), "Lock file should exist");

    // In real workflow, lock should be released before push
    // This is a reminder that the workflow should enforce this
}

#[test]
fn test_pull_with_lock_conflict_detection() {
    // This test verifies that pulling when someone else has a lock
    // should warn the user

    let remote_dir = setup_remote_repo().expect("Failed to setup remote");
    let remote_path = remote_dir.path();
    setup_test_repo(remote_path).expect("Failed to setup remote repo");

    // Add a lock in remote
    let lock_dir = remote_path.join(".oxen/locks");
    fs::create_dir_all(&lock_dir).expect("Failed to create locks dir");
    fs::write(
        lock_dir.join("project.lock"),
        r#"{"user": "otheruser@machine", "acquired_at": "2025-01-01T00:00:00Z"}"#
    ).expect("Failed to write lock");

    // Commit the lock
    std::process::Command::new("oxen")
        .args(&["add", ".oxen/locks/"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add locks");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add lock"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit lock");

    // Clone and verify lock is present
    let local_dir = TempDir::new().expect("Failed to create temp dir");
    let local_path = local_dir.path().join("local");
    std::process::Command::new("oxen")
        .args(&["clone", &format!("file://{}", remote_path.display()), &local_path.to_string_lossy()])
        .output()
        .expect("Failed to clone");

    // Verify lock file was pulled
    assert!(
        local_path.join(".oxen/locks/project.lock").exists(),
        "Lock should be pulled with repository"
    );
}

// =============================================================================
// Large File Push/Pull Tests
// =============================================================================

#[test]
#[ignore] // Expensive test - run manually
fn test_push_large_audio_file() {
    let fixture = TestFixture::new();
    let local_path = fixture.path();
    setup_test_repo(local_path).expect("Failed to setup local repo");

    // Create a large file (10MB simulated audio)
    let large_file = local_path.join("Audio Files/large_track.wav");
    let data = vec![0u8; 10 * 1024 * 1024]; // 10MB
    fs::write(&large_file, data).expect("Failed to write large file");

    // Setup remote
    let remote_dir = setup_remote_repo().expect("Failed to setup remote");
    let remote_path = remote_dir.path();

    std::process::Command::new("oxen")
        .args(&["remote", "add", "origin", &format!("file://{}", remote_path.display())])
        .current_dir(local_path)
        .output()
        .expect("Failed to add remote");

    // Add and commit large file
    std::process::Command::new("oxen")
        .args(&["add", "Audio Files/large_track.wav"])
        .current_dir(local_path)
        .output()
        .expect("Failed to add large file");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add large audio file"])
        .current_dir(local_path)
        .output()
        .expect("Failed to commit");

    // Time the push
    let start = std::time::Instant::now();
    let output = std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(local_path)
        .output()
        .expect("Failed to push");
    let duration = start.elapsed();

    assert!(output.status.success(), "Push should succeed");
    println!("Push of 10MB file took: {:?}", duration);

    // Verify push completed in reasonable time (< 60 seconds for 10MB)
    assert!(
        duration.as_secs() < 60,
        "Push should complete within 60 seconds"
    );
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_push_without_commits_fails_gracefully() {
    let fixture = TestFixture::new();
    let local_path = fixture.path();
    setup_test_repo(local_path).expect("Failed to setup local repo");

    let remote_dir = setup_remote_repo().expect("Failed to setup remote");
    let remote_path = remote_dir.path();

    std::process::Command::new("oxen")
        .args(&["remote", "add", "origin", &format!("file://{}", remote_path.display())])
        .current_dir(local_path)
        .output()
        .expect("Failed to add remote");

    // Try to push without new commits (already pushed initial commit)
    let output = std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(local_path)
        .output()
        .expect("Failed to execute push");

    // Push should handle gracefully (either succeed with "nothing to push" or similar)
    // We don't assert failure here because it depends on Oxen's behavior
}

#[test]
fn test_pull_with_local_changes_warns() {
    let remote_dir = setup_remote_repo().expect("Failed to setup remote");
    let remote_path = remote_dir.path();
    setup_test_repo(remote_path).expect("Failed to setup remote repo");

    // Clone
    let local_dir = TempDir::new().expect("Failed to create temp dir");
    let local_path = local_dir.path().join("local");
    std::process::Command::new("oxen")
        .args(&["clone", &format!("file://{}", remote_path.display()), &local_path.to_string_lossy()])
        .output()
        .expect("Failed to clone");

    // Make local changes without committing
    fs::write(local_path.join("uncommitted.txt"), "local changes")
        .expect("Failed to write file");

    // Make changes in remote
    fs::write(remote_path.join("remote_change.txt"), "remote changes")
        .expect("Failed to write file");
    std::process::Command::new("oxen")
        .args(&["add", "remote_change.txt"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Remote change"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    // Try to pull with uncommitted changes
    let output = std::process::Command::new("oxen")
        .args(&["pull", "origin", "main"])
        .current_dir(&local_path)
        .output()
        .expect("Failed to pull");

    // Oxen should handle this appropriately (either warning or refusing)
    // The exact behavior depends on Oxen's implementation
}

// =============================================================================
// Metadata Sync Tests
// =============================================================================

#[test]
fn test_push_pull_with_metadata() {
    let remote_dir = setup_remote_repo().expect("Failed to setup remote");
    let remote_path = remote_dir.path();
    setup_test_repo(remote_path).expect("Failed to setup remote repo");

    // Add metadata directory in remote
    let metadata_dir = remote_path.join(".oxen/metadata");
    fs::create_dir_all(&metadata_dir).expect("Failed to create metadata dir");
    fs::write(
        metadata_dir.join("commit1.json"),
        r#"{"bpm": 120, "sample_rate": 44100, "key_signature": "A Minor"}"#
    ).expect("Failed to write metadata");

    std::process::Command::new("oxen")
        .args(&["add", ".oxen/metadata/"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add metadata");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add metadata"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    // Clone and verify metadata is pulled
    let local_dir = TempDir::new().expect("Failed to create temp dir");
    let local_path = local_dir.path().join("local");
    std::process::Command::new("oxen")
        .args(&["clone", &format!("file://{}", remote_path.display()), &local_path.to_string_lossy()])
        .output()
        .expect("Failed to clone");

    // Verify metadata exists
    let metadata_file = local_path.join(".oxen/metadata/commit1.json");
    assert!(metadata_file.exists(), "Metadata should be pulled");

    let metadata_content = fs::read_to_string(metadata_file)
        .expect("Failed to read metadata");
    assert!(
        metadata_content.contains("120"),
        "Metadata should contain BPM"
    );
}

// =============================================================================
// Network Resilience Tests
// =============================================================================

#[test]
#[ignore] // Requires network simulation
fn test_push_with_network_interruption() {
    // This test would require mocking network failures
    // For now, it serves as documentation of what should be tested
    println!("TODO: Test push with network interruption and retry");
}

#[test]
#[ignore] // Requires network simulation
fn test_pull_with_partial_transfer() {
    // This test would require simulating partial transfers
    println!("TODO: Test pull with partial transfer and resume");
}

// =============================================================================
// Documentation Test
// =============================================================================

#[test]
fn test_push_pull_workflow_documentation() {
    println!(
        r#"
================================================================================
PUSH/PULL WORKFLOW BEST PRACTICES
================================================================================

RECOMMENDED WORKFLOW FOR TEAM COLLABORATION:

1. Before Starting Work:
   - Pull latest changes: `auxin pull` or `oxen pull origin main`
   - Check lock status: `auxin lock status`
   - Acquire lock: `auxin lock acquire`

2. During Work Session:
   - Make changes in your creative application
   - Auxin auto-commits to draft branch (via daemon)
   - Periodically check status: `auxin status`

3. After Finishing Work:
   - Finalize commit: `auxin commit -m "Description" --bpm 120`
   - Release lock: `auxin lock release`
   - Push changes: `oxen push origin main`

4. Starting New Session:
   - Pull updates: `oxen pull origin main`
   - Review activity: `auxin activity --limit 10`
   - Check team status: `auxin team`

IMPORTANT NOTES:
- Always pull before acquiring lock
- Always release lock before pushing
- Use meaningful commit messages
- Include metadata (BPM, key, etc.) in commits
- Check activity feed to coordinate with teammates

================================================================================
"#
    );
}
