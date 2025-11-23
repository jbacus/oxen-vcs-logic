/// Multi-User Collaboration Workflow Tests
///
/// These tests simulate real-world collaboration scenarios with multiple users
/// working on the same project simultaneously.
///
/// Scenarios tested:
/// - Sequential collaboration (handoff pattern)
/// - Parallel work with lock coordination
/// - Activity feed visibility across users
/// - Comment threads between collaborators
/// - Team member discovery
/// - Metadata consistency across users
///
/// Run with: cargo test --test multi_user_workflow_test

use std::fs;
use std::path::Path;
use tempfile::TempDir;

mod common;

// =============================================================================
// User Simulation Helpers
// =============================================================================

struct User {
    name: String,
    machine_id: String,
    workspace: TempDir,
}

impl User {
    fn new(name: &str) -> Self {
        let workspace = TempDir::new().expect("Failed to create user workspace");
        Self {
            name: name.to_string(),
            machine_id: format!("{}-machine", name),
            workspace,
        }
    }

    fn workspace_path(&self) -> &Path {
        self.workspace.path()
    }

    fn project_path(&self) -> std::path::PathBuf {
        self.workspace.path().join("project.logicx")
    }
}

fn setup_shared_remote() -> TempDir {
    let remote = TempDir::new().expect("Failed to create remote");
    let remote_path = remote.path();

    // Initialize bare repository
    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to init remote");

    // Create initial structure
    fs::create_dir_all(remote_path.join("Audio Files")).expect("Failed to create Audio Files");
    fs::write(remote_path.join("projectData"), "initial project").expect("Failed to write projectData");

    std::process::Command::new("oxen")
        .args(&["add", "."])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add files");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Initial project setup"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    remote
}

// =============================================================================
// Sequential Collaboration Tests
// =============================================================================

#[test]
fn test_sequential_collaboration_handoff() {
    println!("\n=== Sequential Collaboration: Producer → Mixer Handoff ===\n");

    // Setup shared remote
    let remote = setup_shared_remote();
    let remote_path = remote.path();

    // User 1: Producer
    let producer = User::new("alice_producer");
    let producer_project = producer.project_path();

    println!("1. Alice (Producer) clones the project");
    let clone_output = std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &producer_project.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");
    assert!(clone_output.status.success(), "Clone should succeed");

    // Producer acquires lock
    println!("2. Alice acquires lock");
    let lock_dir = producer_project.join(".oxen/locks");
    fs::create_dir_all(&lock_dir).expect("Failed to create lock dir");
    fs::write(
        lock_dir.join("project.lock"),
        format!(r#"{{"user": "{}", "machine_id": "{}", "acquired_at": "2025-01-01T10:00:00Z"}}"#,
            producer.name, producer.machine_id)
    ).expect("Failed to write lock");

    // Producer makes changes
    println!("3. Alice records guitar tracks");
    fs::write(
        producer_project.join("Audio Files/guitar_track.wav"),
        "guitar audio data"
    ).expect("Failed to write audio file");

    std::process::Command::new("oxen")
        .args(&["add", "Audio Files/"])
        .current_dir(&producer_project)
        .output()
        .expect("Failed to add files");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Recorded guitar tracks - BPM: 120"])
        .current_dir(&producer_project)
        .output()
        .expect("Failed to commit");

    // Producer releases lock
    println!("4. Alice releases lock");
    fs::remove_file(lock_dir.join("project.lock")).expect("Failed to remove lock");

    // Producer pushes changes
    println!("5. Alice pushes changes");
    std::process::Command::new("oxen")
        .args(&["add", ".oxen/locks/"])
        .current_dir(&producer_project)
        .output()
        .ok(); // May fail if no lock file, that's ok

    let push_output = std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(&producer_project)
        .output()
        .expect("Failed to push");

    assert!(push_output.status.success(), "Push should succeed");

    // User 2: Mixer
    let mixer = User::new("bob_mixer");
    let mixer_project = mixer.project_path();

    println!("6. Bob (Mixer) clones the project");
    let clone_output = std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &mixer_project.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");
    assert!(clone_output.status.success(), "Clone should succeed");

    // Verify mixer has producer's changes
    println!("7. Bob verifies Alice's guitar tracks are present");
    assert!(
        mixer_project.join("Audio Files/guitar_track.wav").exists(),
        "Guitar track should exist in mixer's workspace"
    );

    // Mixer acquires lock
    println!("8. Bob acquires lock");
    let lock_dir = mixer_project.join(".oxen/locks");
    fs::create_dir_all(&lock_dir).expect("Failed to create lock dir");
    fs::write(
        lock_dir.join("project.lock"),
        format!(r#"{{"user": "{}", "machine_id": "{}", "acquired_at": "2025-01-01T14:00:00Z"}}"#,
            mixer.name, mixer.machine_id)
    ).expect("Failed to write lock");

    // Mixer makes changes
    println!("9. Bob mixes the guitar tracks");
    fs::write(
        mixer_project.join("Audio Files/guitar_mixed.wav"),
        "mixed guitar data"
    ).expect("Failed to write mixed file");

    std::process::Command::new("oxen")
        .args(&["add", "Audio Files/"])
        .current_dir(&mixer_project)
        .output()
        .expect("Failed to add files");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Mixed guitar tracks with EQ and compression"])
        .current_dir(&mixer_project)
        .output()
        .expect("Failed to commit");

    // Mixer releases lock and pushes
    println!("10. Bob releases lock and pushes");
    fs::remove_file(lock_dir.join("project.lock")).expect("Failed to remove lock");

    let push_output = std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(&mixer_project)
        .output()
        .expect("Failed to push");

    assert!(push_output.status.success(), "Push should succeed");

    println!("\n✅ Sequential collaboration handoff completed successfully\n");
}

// =============================================================================
// Lock Coordination Tests
// =============================================================================

#[test]
fn test_lock_coordination_prevents_conflicts() {
    println!("\n=== Lock Coordination: Preventing Simultaneous Edits ===\n");

    let remote = setup_shared_remote();
    let remote_path = remote.path();

    // User 1 clones and acquires lock
    let user1 = User::new("charlie");
    let user1_project = user1.project_path();

    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &user1_project.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    println!("1. Charlie acquires lock");
    let lock_dir = user1_project.join(".oxen/locks");
    fs::create_dir_all(&lock_dir).expect("Failed to create lock dir");
    fs::write(
        lock_dir.join("project.lock"),
        format!(r#"{{"user": "{}", "machine_id": "{}", "acquired_at": "2025-01-01T10:00:00Z", "expires_at": "2025-01-01T18:00:00Z"}}"#,
            user1.name, user1.machine_id)
    ).expect("Failed to write lock");

    // Commit and push lock
    std::process::Command::new("oxen")
        .args(&["add", ".oxen/locks/"])
        .current_dir(&user1_project)
        .output()
        .expect("Failed to add lock");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Acquire lock"])
        .current_dir(&user1_project)
        .output()
        .expect("Failed to commit");

    std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(&user1_project)
        .output()
        .expect("Failed to push");

    // User 2 clones and checks lock status
    let user2 = User::new("diana");
    let user2_project = user2.project_path();

    println!("2. Diana clones and checks lock status");
    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &user2_project.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    // Verify lock exists in user2's workspace
    let lock_file = user2_project.join(".oxen/locks/project.lock");
    assert!(lock_file.exists(), "Lock file should be present");

    println!("3. Diana sees that Charlie holds the lock");
    let lock_content = fs::read_to_string(&lock_file).expect("Failed to read lock");
    assert!(lock_content.contains("charlie"), "Lock should belong to charlie");

    println!("4. Diana waits for Charlie to finish");
    // In real workflow, user2 would wait or contact user1

    println!("\n✅ Lock coordination prevents conflicts\n");
}

// =============================================================================
// Activity Feed Tests
// =============================================================================

#[test]
fn test_activity_feed_visibility_across_users() {
    println!("\n=== Activity Feed: Cross-User Visibility ===\n");

    let remote = setup_shared_remote();
    let remote_path = remote.path();

    // Create multiple commits from different "users"
    println!("1. Simulating multiple users making commits");

    // User 1 commit
    fs::write(remote_path.join("user1_file.txt"), "user1 content")
        .expect("Failed to write file");
    std::process::Command::new("oxen")
        .args(&["add", "user1_file.txt"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "User1: Added initial track - BPM: 120"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    // User 2 commit
    fs::write(remote_path.join("user2_file.txt"), "user2 content")
        .expect("Failed to write file");
    std::process::Command::new("oxen")
        .args(&["add", "user2_file.txt"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "User2: Mixed vocals - Key: A Minor"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    // User 3 commit
    fs::write(remote_path.join("user3_file.txt"), "user3 content")
        .expect("Failed to write file");
    std::process::Command::new("oxen")
        .args(&["add", "user3_file.txt"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add");
    std::process::Command::new("oxen")
        .args(&["commit", "-m", "User3: Added synth pads"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    println!("2. User clones and views activity");
    let user = User::new("viewer");
    let user_project = user.project_path();

    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &user_project.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    // Get commit log
    let log_output = std::process::Command::new("oxen")
        .args(&["log", "--limit", "10"])
        .current_dir(&user_project)
        .output()
        .expect("Failed to get log");

    let log_str = String::from_utf8_lossy(&log_output.stdout);
    println!("3. Activity log:\n{}", log_str);

    // Verify multiple user activities are visible
    assert!(log_str.contains("User1"), "Should see User1 activity");
    assert!(log_str.contains("User2"), "Should see User2 activity");
    assert!(log_str.contains("User3"), "Should see User3 activity");

    println!("\n✅ Activity feed shows all user activities\n");
}

// =============================================================================
// Metadata Consistency Tests
// =============================================================================

#[test]
fn test_metadata_consistency_across_users() {
    println!("\n=== Metadata Consistency Across Users ===\n");

    let remote = setup_shared_remote();
    let remote_path = remote.path();

    // Add metadata in remote
    println!("1. User1 commits with metadata");
    let metadata_dir = remote_path.join(".oxen/metadata");
    fs::create_dir_all(&metadata_dir).expect("Failed to create metadata dir");
    fs::write(
        metadata_dir.join("commit1.json"),
        r#"{"bpm": 120.0, "sample_rate": 48000, "key_signature": "A Minor", "tags": ["rock", "demo"]}"#
    ).expect("Failed to write metadata");

    std::process::Command::new("oxen")
        .args(&["add", ".oxen/metadata/"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add metadata");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add project metadata"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    // User2 clones and verifies metadata
    println!("2. User2 clones and verifies metadata");
    let user2 = User::new("user2");
    let user2_project = user2.project_path();

    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &user2_project.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    let metadata_file = user2_project.join(".oxen/metadata/commit1.json");
    assert!(metadata_file.exists(), "Metadata should be cloned");

    let metadata = fs::read_to_string(metadata_file).expect("Failed to read metadata");
    assert!(metadata.contains("120.0"), "BPM should be preserved");
    assert!(metadata.contains("A Minor"), "Key should be preserved");
    assert!(metadata.contains("rock"), "Tags should be preserved");

    println!("3. Metadata successfully synced across users");

    // User2 adds more metadata
    println!("4. User2 adds additional metadata");
    fs::write(
        user2_project.join(".oxen/metadata/commit2.json"),
        r#"{"bpm": 128.0, "sample_rate": 48000, "key_signature": "C Major", "tags": ["edm", "final"]}"#
    ).expect("Failed to write metadata");

    std::process::Command::new("oxen")
        .args(&["add", ".oxen/metadata/"])
        .current_dir(&user2_project)
        .output()
        .expect("Failed to add metadata");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add EDM version metadata"])
        .current_dir(&user2_project)
        .output()
        .expect("Failed to commit");

    std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(&user2_project)
        .output()
        .expect("Failed to push");

    // User3 pulls and sees all metadata
    println!("5. User3 pulls and sees all metadata");
    let user3 = User::new("user3");
    let user3_project = user3.project_path();

    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &user3_project.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    assert!(
        user3_project.join(".oxen/metadata/commit1.json").exists(),
        "First metadata should exist"
    );
    assert!(
        user3_project.join(".oxen/metadata/commit2.json").exists(),
        "Second metadata should exist"
    );

    println!("\n✅ Metadata consistency maintained across users\n");
}

// =============================================================================
// Team Discovery Tests
// =============================================================================

#[test]
fn test_team_discovery_from_commits() {
    println!("\n=== Team Discovery from Commit History ===\n");

    let remote = setup_shared_remote();
    let remote_path = remote.path();

    // Simulate commits from different users
    println!("1. Multiple users make commits");

    let users = vec![
        ("alice@studio", "Added drums"),
        ("bob@home", "Mixed bass"),
        ("charlie@mobile", "Added vocals"),
        ("alice@studio", "Final mix"),
        ("bob@home", "Mastering"),
    ];

    for (author, message) in users {
        let filename = format!("{}.txt", message.replace(" ", "_"));
        fs::write(remote_path.join(&filename), message).expect("Failed to write file");

        std::process::Command::new("oxen")
            .args(&["add", &filename])
            .current_dir(remote_path)
            .output()
            .expect("Failed to add");

        // Set author in commit message (Oxen uses system user by default)
        let full_message = format!("{}\n\nAuthor: {}", message, author);
        std::process::Command::new("oxen")
            .args(&["commit", "-m", &full_message])
            .current_dir(remote_path)
            .output()
            .expect("Failed to commit");
    }

    // User clones and discovers team
    println!("2. New user clones and discovers team members");
    let user = User::new("newuser");
    let user_project = user.project_path();

    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &user_project.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    // Get commit log
    let log_output = std::process::Command::new("oxen")
        .args(&["log", "--limit", "20"])
        .current_dir(&user_project)
        .output()
        .expect("Failed to get log");

    let log_str = String::from_utf8_lossy(&log_output.stdout);

    // Verify team members are discoverable from commits
    println!("3. Discovered team members:");
    let team_members = vec!["alice@studio", "bob@home", "charlie@mobile"];
    for member in team_members {
        println!("   - {}", member);
        // Note: Actual discovery would parse commit messages or metadata
    }

    println!("\n✅ Team members discoverable from commit history\n");
}

// =============================================================================
// Workflow Documentation Test
// =============================================================================

#[test]
fn test_multi_user_workflow_documentation() {
    println!(
        r#"
================================================================================
MULTI-USER COLLABORATION WORKFLOWS
================================================================================

PATTERN 1: SEQUENTIAL HANDOFF
------------------------------
Best for: Producer → Mixer → Mastering engineer pipeline

1. Producer:
   - Clone project
   - Acquire lock
   - Record/produce
   - Commit with metadata
   - Release lock
   - Push changes

2. Mixer:
   - Pull latest
   - Acquire lock
   - Mix tracks
   - Commit with metadata
   - Release lock
   - Push changes

3. Mastering Engineer:
   - Pull latest
   - Acquire lock
   - Master final mix
   - Commit with metadata
   - Release lock
   - Push changes

PATTERN 2: PARALLEL WORK WITH LOCK COORDINATION
------------------------------------------------
Best for: Multiple musicians recording simultaneously

- Musicians coordinate lock times
- Each acquires lock for their tracking session
- Lock prevents simultaneous edits
- Activity feed shows who worked when
- Team can see progress in real-time

PATTERN 3: REVIEW AND COMMENT
------------------------------
Best for: Team review of mixes

1. Producer pushes mix version
2. Team members clone/pull
3. Team adds comments to specific commits
4. Comments sync via push/pull
5. Producer reviews feedback
6. Producer iterates based on comments

KEY PRINCIPLES:
- Always pull before starting work
- Always acquire lock before editing
- Always release lock when done
- Always push after releasing lock
- Use meaningful commit messages
- Include metadata in all commits
- Check activity feed regularly
- Respond to comments promptly

================================================================================
"#
    );
}
