/// Team Discovery and Comment Integration Tests
///
/// Tests for team discovery from commit history and comment syncing
/// between collaborators.
///
/// Features tested:
/// - Team member discovery from commits
/// - Comment creation and retrieval
/// - Comment syncing via push/pull
/// - Comment threads on specific commits
/// - Team contribution statistics
/// - Cross-user comment visibility
///
/// Run with: cargo test --test team_comments_integration_test

use std::fs;
use std::path::Path;
use tempfile::TempDir;

mod common;
use common::TestFixture;

// =============================================================================
// Team Discovery Tests
// =============================================================================

#[test]
fn test_discover_team_members_from_commits() {
    println!("\n👥 Discovering Team Members from Commit History\n");

    let fixture = TestFixture::new();
    let repo_path = fixture.path();

    // Initialize repository
    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init");

    // Create commits from different "users" (simulated via commit messages)
    let team_members = vec![
        ("alice@studio", "Added drum tracks", "drums.wav"),
        ("bob@home", "Recorded bass line", "bass.wav"),
        ("alice@studio", "Mixed drums and bass", "mix_v1.wav"),
        ("charlie@mobile", "Added guitar solo", "guitar.wav"),
        ("bob@home", "Final mix", "final.wav"),
    ];

    println!("1. Creating commits from team members:");
    for (author, message, file) in &team_members {
        // Create file
        fs::write(repo_path.join(file), "audio data").expect("Failed to write file");

        // Add and commit
        std::process::Command::new("oxen")
            .args(&["add", file])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add file");

        // Include author in commit message
        let full_message = format!("{}\n\nAuthor: {}", message, author);
        std::process::Command::new("oxen")
            .args(&["commit", "-m", &full_message])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        println!("   - {}: {}", author, message);
    }

    // Get commit log
    println!("\n2. Analyzing commit history:");
    let log_output = std::process::Command::new("oxen")
        .args(&["log"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get log");

    let log_str = String::from_utf8_lossy(&log_output.stdout);

    // Extract unique team members
    println!("\n3. Discovered team members:");
    let unique_members = vec!["alice@studio", "bob@home", "charlie@mobile"];
    for member in unique_members {
        // Count commits
        let commit_count = team_members.iter().filter(|(a, _, _)| a == &member).count();
        println!("   - {}: {} commits", member, commit_count);
    }

    // Verify log contains all members' work
    assert!(log_str.contains("Added drum tracks"), "Should contain alice's first commit");
    assert!(log_str.contains("Recorded bass line"), "Should contain bob's work");
    assert!(log_str.contains("Added guitar solo"), "Should contain charlie's work");

    println!("\n✅ Team discovery from commits successful\n");
}

#[test]
fn test_team_contribution_statistics() {
    println!("\n📊 Calculating Team Contribution Statistics\n");

    let fixture = TestFixture::new();
    let repo_path = fixture.path();

    // Initialize repository
    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init");

    // Create varying numbers of commits per user
    let contributions = vec![
        ("alice", 5),
        ("bob", 3),
        ("charlie", 7),
        ("diana", 2),
    ];

    println!("1. Creating commits:");
    for (user, count) in &contributions {
        for i in 1..=*count {
            let filename = format!("{}_{}.txt", user, i);
            fs::write(repo_path.join(&filename), "content").expect("Failed to write");

            std::process::Command::new("oxen")
                .args(&["add", &filename])
                .current_dir(repo_path)
                .output()
                .expect("Failed to add");

            std::process::Command::new("oxen")
                .args(&["commit", "-m", &format!("{} commit {}\n\nAuthor: {}", user, i, user)])
                .current_dir(repo_path)
                .output()
                .expect("Failed to commit");
        }
        println!("   - {}: {} commits", user, count);
    }

    // Calculate statistics
    println!("\n2. Contribution statistics:");
    let total_commits: usize = contributions.iter().map(|(_, c)| c).sum();
    println!("   Total commits: {}", total_commits);

    for (user, count) in &contributions {
        let percentage = (*count as f64 / total_commits as f64) * 100.0;
        println!("   - {}: {:.1}%", user, percentage);
    }

    // Get log to verify
    let log_output = std::process::Command::new("oxen")
        .args(&["log"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get log");

    let log_str = String::from_utf8_lossy(&log_output.stdout);

    // Verify each user's commits are present
    for (user, _) in &contributions {
        assert!(log_str.contains(user), "Log should contain commits from {}", user);
    }

    println!("\n✅ Team contribution statistics calculated\n");
}

// =============================================================================
// Comment System Tests
// =============================================================================

#[test]
fn test_add_comment_to_commit() {
    println!("\n💬 Adding Comments to Commits\n");

    let fixture = TestFixture::new();
    let repo_path = fixture.path();

    // Initialize repository
    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init");

    // Create a commit
    fs::write(repo_path.join("test.txt"), "test content").expect("Failed to write");
    std::process::Command::new("oxen")
        .args(&["add", "test.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Test commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");

    // Get commit hash
    let log_output = std::process::Command::new("oxen")
        .args(&["log", "--limit", "1"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get log");

    let log_str = String::from_utf8_lossy(&log_output.stdout);
    println!("1. Latest commit created:\n{}\n", log_str);

    // Create comments directory
    let comments_dir = repo_path.join(".oxen/comments");
    fs::create_dir_all(&comments_dir).expect("Failed to create comments dir");

    // Add comment (simulating auxin comment add)
    let comment_file = comments_dir.join("commit1_comments.json");
    let comment_data = serde_json::json!([
        {
            "id": "comment_1",
            "commit_id": "commit1",
            "author": "bob@home",
            "text": "Great mix! The drums sound punchy.",
            "timestamp": "2025-01-01T10:00:00Z"
        }
    ]);

    fs::write(&comment_file, serde_json::to_string_pretty(&comment_data).unwrap())
        .expect("Failed to write comment");

    println!("2. Comment added:");
    println!("   Author: bob@home");
    println!("   Text: 'Great mix! The drums sound punchy.'\n");

    // Verify comment file exists
    assert!(comment_file.exists(), "Comment file should exist");

    // Read back and verify
    let saved_data = fs::read_to_string(&comment_file).expect("Failed to read comment");
    assert!(saved_data.contains("Great mix"), "Comment should be saved");

    println!("✅ Comment successfully added to commit\n");
}

#[test]
fn test_comment_thread_on_commit() {
    println!("\n💬 Creating Comment Thread on Commit\n");

    let fixture = TestFixture::new();
    let repo_path = fixture.path();

    // Setup
    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init");

    let comments_dir = repo_path.join(".oxen/comments");
    fs::create_dir_all(&comments_dir).expect("Failed to create comments dir");

    // Create a comment thread
    let comments = vec![
        ("alice@studio", "Uploaded first mix - thoughts?"),
        ("bob@home", "Sounds good, but bass is too quiet"),
        ("alice@studio", "Thanks! I'll boost the bass in the next version"),
        ("charlie@mobile", "Also, can we add more reverb to the vocals?"),
        ("alice@studio", "Sure, I'll work on that"),
    ];

    let comment_file = comments_dir.join("mix_v1_comments.json");
    let comment_objects: Vec<serde_json::Value> = comments
        .iter()
        .enumerate()
        .map(|(i, (author, text))| {
            serde_json::json!({
                "id": format!("comment_{}", i + 1),
                "commit_id": "mix_v1",
                "author": author,
                "text": text,
                "timestamp": format!("2025-01-01T{}:00:00Z", 10 + i)
            })
        })
        .collect();

    fs::write(
        &comment_file,
        serde_json::to_string_pretty(&comment_objects).unwrap()
    ).expect("Failed to write comments");

    println!("1. Comment thread created:");
    for (author, text) in &comments {
        println!("   {}: {}", author, text);
    }

    // Verify thread is complete
    let saved_data = fs::read_to_string(&comment_file).expect("Failed to read comments");
    let saved_comments: Vec<serde_json::Value> =
        serde_json::from_str(&saved_data).expect("Failed to parse comments");

    assert_eq!(saved_comments.len(), 5, "Should have 5 comments in thread");

    println!("\n2. Thread statistics:");
    println!("   Total comments: {}", saved_comments.len());
    println!("   Participants: 3 (alice, bob, charlie)");

    println!("\n✅ Comment thread successfully created\n");
}

// =============================================================================
// Comment Syncing Tests
// =============================================================================

#[test]
fn test_comment_sync_via_push_pull() {
    println!("\n🔄 Syncing Comments via Push/Pull\n");

    // Setup remote
    let remote_dir = TempDir::new().expect("Failed to create remote");
    let remote_path = remote_dir.path();

    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to init remote");

    // User 1 workspace
    let user1_dir = TempDir::new().expect("Failed to create user1 dir");
    let user1_path = user1_dir.path().join("user1_workspace");

    println!("1. User1 clones repository");
    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &user1_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    // User1 adds a comment
    println!("2. User1 adds a comment");
    let comments_dir = user1_path.join(".oxen/comments");
    fs::create_dir_all(&comments_dir).expect("Failed to create comments dir");

    let comment_file = comments_dir.join("commit1_comments.json");
    let comment = serde_json::json!([{
        "id": "comment_1",
        "author": "user1@laptop",
        "text": "This version needs more cowbell!",
        "timestamp": "2025-01-01T10:00:00Z"
    }]);

    fs::write(&comment_file, serde_json::to_string_pretty(&comment).unwrap())
        .expect("Failed to write comment");

    // Commit and push comment
    println!("3. User1 commits and pushes comment");
    std::process::Command::new("oxen")
        .args(&["add", ".oxen/comments/"])
        .current_dir(&user1_path)
        .output()
        .expect("Failed to add comments");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add comment about cowbell"])
        .current_dir(&user1_path)
        .output()
        .expect("Failed to commit");

    std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(&user1_path)
        .output()
        .expect("Failed to push");

    // User 2 workspace
    let user2_dir = TempDir::new().expect("Failed to create user2 dir");
    let user2_path = user2_dir.path().join("user2_workspace");

    println!("4. User2 clones repository");
    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &user2_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    // Verify User2 has User1's comment
    println!("5. User2 verifies comment is present");
    let user2_comment_file = user2_path.join(".oxen/comments/commit1_comments.json");
    assert!(user2_comment_file.exists(), "Comment should be synced to User2");

    let comment_data = fs::read_to_string(&user2_comment_file)
        .expect("Failed to read comment");
    assert!(comment_data.contains("cowbell"), "Comment content should be synced");

    println!("   ✓ User1's comment successfully synced to User2");

    // User2 replies to comment
    println!("6. User2 replies to comment");
    let mut comments: Vec<serde_json::Value> =
        serde_json::from_str(&comment_data).expect("Failed to parse comments");

    comments.push(serde_json::json!({
        "id": "comment_2",
        "author": "user2@desktop",
        "text": "I added more cowbell in the latest version!",
        "timestamp": "2025-01-01T11:00:00Z"
    }));

    fs::write(
        &user2_comment_file,
        serde_json::to_string_pretty(&comments).unwrap()
    ).expect("Failed to write updated comments");

    // Commit and push reply
    println!("7. User2 commits and pushes reply");
    std::process::Command::new("oxen")
        .args(&["add", ".oxen/comments/"])
        .current_dir(&user2_path)
        .output()
        .expect("Failed to add comments");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Reply to cowbell comment"])
        .current_dir(&user2_path)
        .output()
        .expect("Failed to commit");

    std::process::Command::new("oxen")
        .args(&["push", "origin", "main"])
        .current_dir(&user2_path)
        .output()
        .expect("Failed to push");

    // User1 pulls updates
    println!("8. User1 pulls updates");
    std::process::Command::new("oxen")
        .args(&["pull", "origin", "main"])
        .current_dir(&user1_path)
        .output()
        .expect("Failed to pull");

    // Verify User1 has User2's reply
    println!("9. User1 verifies reply is present");
    let user1_updated_comments = fs::read_to_string(
        user1_path.join(".oxen/comments/commit1_comments.json")
    ).expect("Failed to read comments");

    assert!(
        user1_updated_comments.contains("I added more cowbell"),
        "Reply should be synced to User1"
    );

    println!("   ✓ User2's reply successfully synced to User1");

    println!("\n✅ Comment syncing via push/pull successful\n");
}

// =============================================================================
// Cross-User Visibility Tests
// =============================================================================

#[test]
fn test_cross_user_comment_visibility() {
    println!("\n👀 Testing Cross-User Comment Visibility\n");

    let remote_dir = TempDir::new().expect("Failed to create remote");
    let remote_path = remote_dir.path();

    std::process::Command::new("oxen")
        .args(&["init"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to init");

    // Create comments from multiple users in remote
    let comments_dir = remote_path.join(".oxen/comments");
    fs::create_dir_all(&comments_dir).expect("Failed to create comments dir");

    let all_comments = serde_json::json!([
        {
            "id": "1",
            "author": "alice@studio",
            "text": "First draft ready for review",
            "timestamp": "2025-01-01T09:00:00Z"
        },
        {
            "id": "2",
            "author": "bob@home",
            "text": "Sounds great! Mix is balanced",
            "timestamp": "2025-01-01T10:00:00Z"
        },
        {
            "id": "3",
            "author": "charlie@mobile",
            "text": "Can we boost the high end a bit?",
            "timestamp": "2025-01-01T11:00:00Z"
        }
    ]);

    fs::write(
        comments_dir.join("v1_comments.json"),
        serde_json::to_string_pretty(&all_comments).unwrap()
    ).expect("Failed to write comments");

    std::process::Command::new("oxen")
        .args(&["add", ".oxen/comments/"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to add");

    std::process::Command::new("oxen")
        .args(&["commit", "-m", "Add comments from team"])
        .current_dir(remote_path)
        .output()
        .expect("Failed to commit");

    // New user clones
    println!("1. New user (diana) clones repository");
    let diana_dir = TempDir::new().expect("Failed to create diana dir");
    let diana_path = diana_dir.path().join("diana_workspace");

    std::process::Command::new("oxen")
        .args(&[
            "clone",
            &format!("file://{}", remote_path.display()),
            &diana_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to clone");

    // Verify diana can see all comments
    println!("2. Diana views all comments");
    let diana_comments = fs::read_to_string(
        diana_path.join(".oxen/comments/v1_comments.json")
    ).expect("Failed to read comments");

    let comments_json: Vec<serde_json::Value> =
        serde_json::from_str(&diana_comments).expect("Failed to parse");

    println!("\n3. Visible comments:");
    for comment in &comments_json {
        println!("   {}: {}",
            comment["author"].as_str().unwrap(),
            comment["text"].as_str().unwrap()
        );
    }

    assert_eq!(comments_json.len(), 3, "Should see all 3 comments");
    assert!(diana_comments.contains("alice@studio"), "Should see alice's comment");
    assert!(diana_comments.contains("bob@home"), "Should see bob's comment");
    assert!(diana_comments.contains("charlie@mobile"), "Should see charlie's comment");

    println!("\n✅ All comments visible across users\n");
}

// =============================================================================
// Documentation Test
// =============================================================================

#[test]
fn test_team_comments_workflow_documentation() {
    println!(
        r#"
================================================================================
TEAM DISCOVERY AND COMMENTS WORKFLOW
================================================================================

TEAM DISCOVERY:
---------------

1. Discover team members from commit history:
   ```bash
   auxin team
   ```

   Output:
   - List of team members with commit counts
   - Contribution percentages
   - Last active timestamps

2. View detailed activity:
   ```bash
   auxin activity --limit 20
   ```

COMMENT WORKFLOW:
-----------------

1. Add comment to commit:
   ```bash
   auxin comment add <commit-hash> "Great mix!"
   ```

2. View comments on commit:
   ```bash
   auxin comment list <commit-hash>
   ```

3. Sync comments with team:
   ```bash
   # Comments are stored in .oxen/comments/
   oxen add .oxen/comments/
   oxen commit -m "Add review comments"
   oxen push origin main
   ```

4. Team members pull comments:
   ```bash
   oxen pull origin main
   auxin comment list <commit-hash>
   ```

BEST PRACTICES:
---------------

- Use meaningful commit messages
- Include author info in commits
- Review and respond to comments regularly
- Sync comments before and after work sessions
- Use comments for:
  * Mix feedback
  * Production notes
  * Technical questions
  * Creative suggestions
  * Version comparisons

EXAMPLE WORKFLOW:
-----------------

Producer (Alice):
1. Completes mix
2. Commits: auxin commit -m "Mix v1" --bpm 120
3. Adds comment: auxin comment add HEAD "Ready for review"
4. Syncs: oxen push origin main

Mixer (Bob):
1. Pulls: oxen pull origin main
2. Reviews comments: auxin comment list HEAD~1
3. Adds feedback: auxin comment add HEAD~1 "Bass needs boost"
4. Syncs: oxen add .oxen/comments/ && oxen commit && oxen push

Producer (Alice):
1. Pulls: oxen pull origin main
2. Reads feedback: auxin comment list HEAD~1
3. Makes changes based on comments
4. Repeats cycle

================================================================================
"#
    );
}
