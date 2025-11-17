// Integration tests for collaboration features
//
// These tests require:
// - Real Oxen Hub account and API key
// - Network connectivity to hub.oxen.ai
// - Test repository configured
//
// Run with: cargo test --test collaboration_integration_test -- --ignored --test-threads=1
//
// Note: These tests are marked #[ignore] by default to prevent running in CI.
// Run explicitly when you have access to Oxen Hub.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// Helper to check if integration tests should run
fn should_run_integration_tests() -> bool {
    env::var("RUN_INTEGRATION_TESTS").is_ok()
}

// Helper to get test credentials from environment
fn get_test_credentials() -> Option<(String, String)> {
    let username = env::var("OXEN_TEST_USERNAME").ok()?;
    let api_key = env::var("OXEN_TEST_API_KEY").ok()?;
    Some((username, api_key))
}

// Helper to run CLI command
fn run_cli_command(args: &[&str], working_dir: Option<&PathBuf>) -> Result<String, String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--");
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| format!("Failed to execute: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// =============================================================================
// Authentication Integration Tests
// =============================================================================

#[test]
#[ignore] // Run with: cargo test test_auth_login_flow -- --ignored
fn test_auth_login_flow() {
    if !should_run_integration_tests() {
        println!("Skipping integration test. Set RUN_INTEGRATION_TESTS=1 to run.");
        return;
    }

    let (username, api_key) = get_test_credentials()
        .expect("OXEN_TEST_USERNAME and OXEN_TEST_API_KEY must be set");

    // Test 1: Login
    println!("Testing auth login...");
    // Note: This requires interactive input, so we'll test the non-interactive path
    // TODO: Implement non-interactive login for testing

    // Test 2: Status check
    println!("Testing auth status...");
    let status_output = run_cli_command(&["auth", "status"], None)
        .expect("Auth status should succeed");

    assert!(
        status_output.contains(&username),
        "Status should show username"
    );

    // Test 3: Connection test
    println!("Testing auth test...");
    let test_output = run_cli_command(&["auth", "test"], None)
        .expect("Auth test should succeed");

    assert!(
        test_output.contains("success") || test_output.contains("Success"),
        "Connection test should succeed"
    );

    println!("✅ Authentication integration tests passed");
}

#[test]
#[ignore]
fn test_auth_logout_flow() {
    if !should_run_integration_tests() {
        return;
    }

    // Test logout
    println!("Testing auth logout...");
    let logout_output = run_cli_command(&["auth", "logout"], None)
        .expect("Auth logout should succeed");

    assert!(
        logout_output.contains("success") || logout_output.contains("logged out"),
        "Logout should succeed"
    );

    // Verify credentials removed
    let status_output = run_cli_command(&["auth", "status"], None)
        .expect("Auth status should still work");

    assert!(
        status_output.contains("Not Authenticated"),
        "Should show not authenticated after logout"
    );

    println!("✅ Logout flow test passed");
}

// =============================================================================
// Remote Lock Integration Tests
// =============================================================================

#[test]
#[ignore] // Run with: cargo test test_lock_acquire_release -- --ignored
fn test_lock_acquire_release() {
    if !should_run_integration_tests() {
        return;
    }

    let test_dir = TempDir::new().expect("Failed to create temp dir");
    let test_repo = test_dir.path();

    // Setup: Initialize test repository
    println!("Setting up test repository...");
    setup_test_repo(test_repo);

    // Test 1: Check lock status (should be no lock)
    println!("Checking initial lock status...");
    let status_output = run_cli_command(&["lock", "status"], Some(&test_repo.to_path_buf()))
        .expect("Lock status should succeed");

    assert!(
        status_output.contains("No lock") || status_output.contains("unlocked"),
        "Should show no lock initially"
    );

    // Test 2: Acquire lock
    println!("Acquiring lock...");
    let acquire_output = run_cli_command(
        &["lock", "acquire", "--timeout", "4"],
        Some(&test_repo.to_path_buf())
    ).expect("Lock acquire should succeed");

    assert!(
        acquire_output.contains("acquired") || acquire_output.contains("Lock ID"),
        "Should show lock acquired"
    );

    // Test 3: Check status (should show locked)
    println!("Checking lock status after acquisition...");
    let status_output = run_cli_command(&["lock", "status"], Some(&test_repo.to_path_buf()))
        .expect("Lock status should succeed");

    assert!(
        status_output.contains("Locked") || status_output.contains("●"),
        "Should show locked status"
    );

    // Test 4: Release lock
    println!("Releasing lock...");
    let release_output = run_cli_command(&["lock", "release"], Some(&test_repo.to_path_buf()))
        .expect("Lock release should succeed");

    assert!(
        release_output.contains("released") || release_output.contains("success"),
        "Should show lock released"
    );

    // Test 5: Verify lock released
    println!("Verifying lock released...");
    let status_output = run_cli_command(&["lock", "status"], Some(&test_repo.to_path_buf()))
        .expect("Lock status should succeed");

    assert!(
        status_output.contains("No lock") || status_output.contains("unlocked"),
        "Should show no lock after release"
    );

    println!("✅ Lock acquire/release test passed");
}

#[test]
#[ignore]
fn test_lock_collision() {
    if !should_run_integration_tests() {
        return;
    }

    // This test requires two separate processes/machines
    // For now, document the manual test procedure
    println!("
    ⚠️  Lock collision test requires manual execution:

    Machine A:
    1. cd test-project.logicx
    2. auxin lock acquire --timeout 4
    3. Verify success

    Machine B:
    1. cd test-project.logicx
    2. oxen pull origin locks
    3. auxin lock acquire --timeout 4
    4. EXPECTED: Failure - lock held by Machine A

    Machine A:
    1. auxin lock release

    Machine B:
    1. auxin lock acquire --timeout 4
    2. EXPECTED: Success - lock now available
    ");

    // TODO: Implement when we have multi-process test harness
}

#[test]
#[ignore]
fn test_lock_expiration() {
    if !should_run_integration_tests() {
        return;
    }

    let test_dir = TempDir::new().expect("Failed to create temp dir");
    let test_repo = test_dir.path();

    println!("Setting up test repository...");
    setup_test_repo(test_repo);

    // Acquire lock with very short timeout (for testing)
    // Note: Minimum timeout in production is 1 hour, but we can test the logic
    println!("Acquiring lock with short timeout...");
    run_cli_command(
        &["lock", "acquire", "--timeout", "1"],
        Some(&test_repo.to_path_buf())
    ).expect("Lock acquire should succeed");

    // Manually edit the lock file to expire it (simulate time passing)
    println!("Simulating lock expiration...");
    // TODO: Implement helper to manually expire lock in test

    // Try to acquire from "another user"
    println!("Attempting to acquire expired lock...");
    // TODO: This requires simulating different user

    println!("⚠️  Lock expiration test requires manual execution - see test code for steps");
}

#[test]
#[ignore]
fn test_lock_force_break() {
    if !should_run_integration_tests() {
        return;
    }

    let test_dir = TempDir::new().expect("Failed to create temp dir");
    let test_repo = test_dir.path();

    println!("Setting up test repository...");
    setup_test_repo(test_repo);

    // Acquire lock
    println!("Acquiring lock...");
    run_cli_command(
        &["lock", "acquire", "--timeout", "4"],
        Some(&test_repo.to_path_buf())
    ).expect("Lock acquire should succeed");

    // Try to break without --force (should fail)
    println!("Attempting to break without --force...");
    let break_result = run_cli_command(&["lock", "break"], Some(&test_repo.to_path_buf()));

    assert!(
        break_result.is_err() && break_result.unwrap_err().contains("--force"),
        "Should require --force flag"
    );

    // Break with --force
    println!("Breaking lock with --force...");
    let break_output = run_cli_command(
        &["lock", "break", "--force"],
        Some(&test_repo.to_path_buf())
    ).expect("Lock break should succeed");

    assert!(
        break_output.contains("break") || break_output.contains("removed"),
        "Should show lock broken"
    );

    // Verify lock removed
    let status_output = run_cli_command(&["lock", "status"], Some(&test_repo.to_path_buf()))
        .expect("Lock status should succeed");

    assert!(
        status_output.contains("No lock"),
        "Should show no lock after break"
    );

    println!("✅ Lock force break test passed");
}

// =============================================================================
// Collaboration Feature Integration Tests
// =============================================================================

#[test]
#[ignore]
fn test_activity_feed() {
    if !should_run_integration_tests() {
        return;
    }

    let test_dir = TempDir::new().expect("Failed to create temp dir");
    let test_repo = test_dir.path();

    println!("Setting up test repository with commits...");
    setup_test_repo(test_repo);
    create_test_commits(test_repo);

    // Test activity feed
    println!("Testing activity feed...");
    let activity_output = run_cli_command(
        &["activity", "--limit", "10"],
        Some(&test_repo.to_path_buf())
    ).expect("Activity command should succeed");

    // Verify output contains commit information
    assert!(
        activity_output.contains("●") || activity_output.contains("Commit"),
        "Should show commit activities"
    );

    // Check for metadata
    assert!(
        activity_output.contains("BPM") || activity_output.contains("120"),
        "Should show BPM metadata"
    );

    println!("✅ Activity feed test passed");
}

#[test]
#[ignore]
fn test_team_discovery() {
    if !should_run_integration_tests() {
        return;
    }

    let test_dir = TempDir::new().expect("Failed to create temp dir");
    let test_repo = test_dir.path();

    println!("Setting up test repository with multiple contributors...");
    setup_test_repo(test_repo);
    create_test_commits(test_repo);

    // Test team discovery
    println!("Testing team discovery...");
    let team_output = run_cli_command(&["team"], Some(&test_repo.to_path_buf()))
        .expect("Team command should succeed");

    // Verify output contains team information
    assert!(
        team_output.contains("commit") || team_output.contains("Team"),
        "Should show team members"
    );

    // Check for contribution stats
    assert!(
        team_output.contains("%") || team_output.contains("█"),
        "Should show contribution percentages/bars"
    );

    println!("✅ Team discovery test passed");
}

#[test]
#[ignore]
fn test_comment_system() {
    if !should_run_integration_tests() {
        return;
    }

    let test_dir = TempDir::new().expect("Failed to create temp dir");
    let test_repo = test_dir.path();

    println!("Setting up test repository...");
    setup_test_repo(test_repo);
    create_test_commits(test_repo);

    // Get a commit hash to comment on
    let log_output = run_cli_command(
        &["log", "--limit", "1"],
        Some(&test_repo.to_path_buf())
    ).expect("Log command should succeed");

    // Extract commit hash (basic parsing)
    // TODO: Implement proper hash extraction
    let commit_hash = "abc123"; // Placeholder

    // Add comment
    println!("Adding comment...");
    let comment_output = run_cli_command(
        &["comment", "add", commit_hash, "Great work on this mix!"],
        Some(&test_repo.to_path_buf())
    ).expect("Comment add should succeed");

    assert!(
        comment_output.contains("success") || comment_output.contains("added"),
        "Should show comment added"
    );

    // List comments
    println!("Listing comments...");
    let list_output = run_cli_command(
        &["comment", "list", commit_hash],
        Some(&test_repo.to_path_buf())
    ).expect("Comment list should succeed");

    assert!(
        list_output.contains("Great work"),
        "Should show the comment we added"
    );

    // Verify comment file exists
    let comment_file = test_repo.join(".oxen").join("comments").join(format!("{}.json", commit_hash));
    assert!(comment_file.exists(), "Comment file should exist");

    println!("✅ Comment system test passed");
}

// =============================================================================
// End-to-End Workflow Tests
// =============================================================================

#[test]
#[ignore]
fn test_complete_collaboration_workflow() {
    if !should_run_integration_tests() {
        return;
    }

    println!("
    ⚠️  Complete workflow test requires manual execution with 2 users:

    === User A (Producer) ===
    1. cd MyProject.logicx
    2. oxen pull origin main
    3. auxin lock acquire --timeout 8
    4. # Make changes to project
    5. auxin add --all
    6. auxin commit -m 'Recorded vocals' --bpm 120 --tags recording
    7. oxen push origin main
    8. auxin lock release

    === User B (Mixer) ===
    1. auxin activity --limit 10  # See A's work
    2. auxin team                 # Check team stats
    3. oxen pull origin main            # Get A's changes
    4. auxin lock acquire --timeout 4
    5. # Make changes
    6. auxin add --all
    7. auxin commit -m 'Mixed vocals' --bpm 120 --tags mixing
    8. auxin comment add <commit> 'Great vocal take!'
    9. oxen add .oxen/comments/ && oxen commit -m 'Add comment'
    10. oxen push origin main
    11. auxin lock release

    === User A ===
    1. oxen pull origin main
    2. auxin activity --limit 10  # See B's work
    3. auxin comment list <commit> # See B's comment

    Expected: No conflicts, all changes synced, comments visible
    ");

    // TODO: Implement when we have multi-machine test harness
}

#[test]
#[ignore]
fn test_large_project_performance() {
    if !should_run_integration_tests() {
        return;
    }

    println!("
    ⚠️  Large project test should be run manually:

    1. Create Logic Pro project with ~5GB of audio files
    2. Initialize: auxin init --logic .
    3. Add: auxin add --all
    4. Commit: auxin commit -m 'Initial' --bpm 120
    5. Push: oxen push origin main (record time)
    6. Clone on another machine: oxen clone <url> (record time)
    7. Lock, modify, commit, push (record times)

    Record metrics:
    - Initial push: ___ minutes
    - Clone time: ___ minutes
    - Lock ops: ___ seconds
    - Pull after changes: ___ minutes

    Expected: All operations <10 minutes, no timeouts
    ");

    // TODO: Implement automated large project test
}

// =============================================================================
// Test Helper Functions
// =============================================================================

fn setup_test_repo(repo_path: &std::path::Path) {
    // Create .logicx directory structure
    fs::create_dir_all(repo_path.join("Audio Files")).expect("Failed to create Audio Files dir");
    fs::create_dir_all(repo_path.join("Alternatives")).expect("Failed to create Alternatives dir");

    // Create projectData file
    fs::write(repo_path.join("projectData"), b"mock project data").expect("Failed to create projectData");

    // Initialize OxVCS
    run_cli_command(&["init", "--logic", "."], Some(&repo_path.to_path_buf()))
        .expect("Failed to initialize repo");

    // TODO: Add remote repository configuration
    // This requires test repo URL from environment
    if let Ok(test_repo_url) = env::var("OXEN_TEST_REPO_URL") {
        // Configure remote
        Command::new("oxen")
            .args(&["remote", "add", "origin", &test_repo_url])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add remote");
    }
}

fn create_test_commits(repo_path: &std::path::Path) {
    // Create some test commits with metadata

    // Commit 1
    fs::write(repo_path.join("Audio Files/track1.wav"), b"audio data 1")
        .expect("Failed to create test file");

    run_cli_command(&["add", "--all"], Some(&repo_path.to_path_buf()))
        .expect("Failed to add files");

    run_cli_command(
        &["commit", "-m", "First track", "--bpm", "120", "--key", "C Major"],
        Some(&repo_path.to_path_buf())
    ).expect("Failed to commit");

    // Commit 2
    fs::write(repo_path.join("Audio Files/track2.wav"), b"audio data 2")
        .expect("Failed to create test file");

    run_cli_command(&["add", "--all"], Some(&repo_path.to_path_buf()))
        .expect("Failed to add files");

    run_cli_command(
        &["commit", "-m", "Added drums", "--bpm", "128", "--tags", "drums,tracking"],
        Some(&repo_path.to_path_buf())
    ).expect("Failed to commit");

    // Commit 3
    fs::write(repo_path.join("Audio Files/track3.wav"), b"audio data 3")
        .expect("Failed to create test file");

    run_cli_command(&["add", "--all"], Some(&repo_path.to_path_buf()))
        .expect("Failed to add files");

    run_cli_command(
        &["commit", "-m", "Mixed", "--bpm", "128", "--key", "C Major", "--tags", "mixing,final"],
        Some(&repo_path.to_path_buf())
    ).expect("Failed to commit");
}

// =============================================================================
// Test Documentation
// =============================================================================

#[test]
fn test_integration_setup_instructions() {
    println!("
================================================================================
INTEGRATION TEST SETUP INSTRUCTIONS
================================================================================

These integration tests require access to Oxen Hub and cannot run in standard
CI environments. Follow these steps to run them:

1. PREREQUISITES
   - macOS 14.0+ with Xcode 15+
   - Oxen CLI installed: pip install oxen-ai
   - OxVCS CLI built: cargo build --release
   - Oxen Hub account with API key
   - Test repository created on hub.oxen.ai

2. ENVIRONMENT VARIABLES
   export RUN_INTEGRATION_TESTS=1
   export OXEN_TEST_USERNAME='your-username'
   export OXEN_TEST_API_KEY='your-api-key'
   export OXEN_TEST_REPO_URL='https://hub.oxen.ai/username/test-repo'

3. RUN TESTS
   # Run all integration tests (single-threaded to avoid conflicts)
   cargo test --test collaboration_integration_test -- --ignored --test-threads=1

   # Run specific test
   cargo test test_auth_login_flow -- --ignored
   cargo test test_lock_acquire_release -- --ignored
   cargo test test_activity_feed -- --ignored

4. MANUAL TESTS
   Some tests require multiple machines/users and must be run manually:
   - test_lock_collision (2 users)
   - test_complete_collaboration_workflow (2 users)
   - test_large_project_performance (large project)

5. REPORT RESULTS
   For each test, document:
   - ✅ PASS / ❌ FAIL / ⚠️ PARTIAL
   - Execution time
   - Any bugs or unexpected behavior
   - Performance metrics

See INTEGRATION_TEST_PLAN.md for detailed test procedures.
================================================================================
    ");
}
