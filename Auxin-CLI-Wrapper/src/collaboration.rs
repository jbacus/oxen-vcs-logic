/// Collaboration features for team-based workflows
///
/// This module provides activity feeds, team member discovery, and
/// commit commenting to enable GitHub-like collaboration for Logic Pro projects.
///
/// # Features
///
/// - **Activity Feed**: Timeline of recent project activity
/// - **Team Discovery**: Find collaborators from commit history
/// - **Comments**: Add comments to commits for discussion
///
/// # Example
///
/// ```no_run
/// # fn main() -> anyhow::Result<()> {
/// use auxin::collaboration::ActivityFeed;
/// use std::path::Path;
///
/// let feed = ActivityFeed::new();
/// let activities = feed.get_recent_activity(Path::new("."), 10)?;
///
/// for activity in activities {
///     println!("{}: {} - {}", activity.timestamp, activity.author, activity.message);
/// }
/// # Ok(())
/// # }
/// ```
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{CommitInfo, OxenSubprocess};

/// A project activity entry (commit, lock, comment, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Activity ID (typically commit hash)
    pub id: String,

    /// Activity type
    pub activity_type: ActivityType,

    /// Author of the activity
    pub author: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Activity message/description
    pub message: String,

    /// Additional metadata (e.g., BPM, sample rate)
    pub metadata: HashMap<String, String>,
}

/// Type of activity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    /// Commit to repository
    Commit,
    /// Lock acquired
    LockAcquired,
    /// Lock released
    LockReleased,
    /// Comment added
    Comment,
    /// Branch created
    BranchCreated,
}

impl ActivityType {
    pub fn icon(&self) -> &'static str {
        match self {
            ActivityType::Commit => "●",
            ActivityType::LockAcquired => "🔒",
            ActivityType::LockReleased => "🔓",
            ActivityType::Comment => "💬",
            ActivityType::BranchCreated => "⎇",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActivityType::Commit => "Commit",
            ActivityType::LockAcquired => "Lock Acquired",
            ActivityType::LockReleased => "Lock Released",
            ActivityType::Comment => "Comment",
            ActivityType::BranchCreated => "Branch Created",
        }
    }
}

/// Manages activity feed for a project
pub struct ActivityFeed {
    oxen: OxenSubprocess,
}

impl ActivityFeed {
    /// Create a new ActivityFeed
    pub fn new() -> Self {
        Self {
            oxen: OxenSubprocess::new(),
        }
    }

    /// Get recent activity for a project
    ///
    /// Returns up to `limit` recent activities, sorted by timestamp (newest first)
    pub fn get_recent_activity(&self, repo_path: &Path, limit: usize) -> Result<Vec<Activity>> {
        // Get recent commits
        let commits = self
            .oxen
            .log(repo_path, Some(limit))
            .context("Failed to fetch commit log")?;

        let mut activities = Vec::new();

        for commit in commits {
            let activity = self.commit_to_activity(&commit)?;
            activities.push(activity);
        }

        Ok(activities)
    }

    /// Get activity for a specific time range
    pub fn get_activity_since(
        &self,
        repo_path: &Path,
        since: DateTime<Utc>,
    ) -> Result<Vec<Activity>> {
        // For now, get all recent commits and filter
        // In a production system, this would use git log --since
        let all_activities = self.get_recent_activity(repo_path, 100)?;

        Ok(all_activities
            .into_iter()
            .filter(|a| a.timestamp >= since)
            .collect())
    }

    /// Convert commit to activity
    fn commit_to_activity(&self, commit: &CommitInfo) -> Result<Activity> {
        // Parse commit message to extract metadata
        let (message, metadata) = self.parse_commit_message(&commit.message);

        // Try to extract author from commit message
        // In real implementation, would use git log --format to get author
        let author =
            extract_author_from_message(&commit.message).unwrap_or_else(|| "unknown".to_string());

        Ok(Activity {
            id: commit.id.clone(),
            activity_type: ActivityType::Commit,
            author,
            timestamp: Utc::now(), // TODO: Parse from commit
            message,
            metadata,
        })
    }

    /// Parse commit message to extract message and metadata
    fn parse_commit_message(&self, full_message: &str) -> (String, HashMap<String, String>) {
        let mut message = String::new();
        let mut metadata = HashMap::new();

        for line in full_message.lines() {
            let trimmed = line.trim();

            // Check if line is metadata (key: value format)
            if let Some((key, value)) = parse_metadata_line(trimmed) {
                metadata.insert(key.to_string(), value.to_string());
            } else if !trimmed.is_empty() && message.is_empty() {
                // First non-empty, non-metadata line is the message
                message = trimmed.to_string();
            }
        }

        (message, metadata)
    }
}

impl Default for ActivityFeed {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages team member discovery
pub struct TeamManager {
    oxen: OxenSubprocess,
}

impl TeamManager {
    /// Create a new TeamManager
    pub fn new() -> Self {
        Self {
            oxen: OxenSubprocess::new(),
        }
    }

    /// Discover team members from commit history
    pub fn discover_team_members(&self, repo_path: &Path) -> Result<Vec<TeamMember>> {
        // Get commit history
        let commits = self
            .oxen
            .log(repo_path, Some(100))
            .context("Failed to fetch commit log")?;

        let mut members_map: HashMap<String, TeamMember> = HashMap::new();

        for commit in commits {
            if let Some(author) = extract_author_from_message(&commit.message) {
                members_map
                    .entry(author.clone())
                    .and_modify(|m| m.commit_count += 1)
                    .or_insert_with(|| TeamMember {
                        name: author,
                        commit_count: 1,
                        last_active: Utc::now(), // TODO: Parse from commit
                    });
            }
        }

        let mut members: Vec<TeamMember> = members_map.into_values().collect();

        // Sort by commit count (most active first)
        members.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

        Ok(members)
    }
}

impl Default for TeamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A team member working on the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    /// Member name (username@hostname)
    pub name: String,

    /// Number of commits
    pub commit_count: usize,

    /// Last activity timestamp
    pub last_active: DateTime<Utc>,
}

/// Manages comments on commits
pub struct CommentManager {
    /// Comments are stored in .oxen/comments/<commit_hash>.json
    comments_dir: String,
}

impl CommentManager {
    /// Create a new CommentManager
    pub fn new() -> Self {
        Self {
            comments_dir: ".oxen/comments".to_string(),
        }
    }

    /// Add a comment to a commit
    pub fn add_comment(
        &self,
        repo_path: &Path,
        commit_id: &str,
        author: &str,
        text: &str,
    ) -> Result<Comment> {
        let comment = Comment {
            id: uuid::Uuid::new_v4().to_string(),
            commit_id: commit_id.to_string(),
            author: author.to_string(),
            text: text.to_string(),
            created_at: Utc::now(),
        };

        // Create comments directory
        let comments_dir = repo_path.join(&self.comments_dir);
        std::fs::create_dir_all(&comments_dir)?;

        // Read existing comments
        let comment_file = comments_dir.join(format!("{}.json", commit_id));
        let mut comments = if comment_file.exists() {
            let data = std::fs::read_to_string(&comment_file)?;
            serde_json::from_str::<Vec<Comment>>(&data)?
        } else {
            Vec::new()
        };

        // Add new comment
        comments.push(comment.clone());

        // Write back
        let json = serde_json::to_string_pretty(&comments)?;
        std::fs::write(&comment_file, json)?;

        Ok(comment)
    }

    /// Get comments for a commit
    pub fn get_comments(&self, repo_path: &Path, commit_id: &str) -> Result<Vec<Comment>> {
        let comment_file = repo_path
            .join(&self.comments_dir)
            .join(format!("{}.json", commit_id));

        if !comment_file.exists() {
            return Ok(Vec::new());
        }

        let data = std::fs::read_to_string(&comment_file)?;
        let comments = serde_json::from_str(&data)?;

        Ok(comments)
    }

    /// Get all comments for the repository
    pub fn get_all_comments(&self, repo_path: &Path) -> Result<Vec<Comment>> {
        let comments_dir = repo_path.join(&self.comments_dir);

        if !comments_dir.exists() {
            return Ok(Vec::new());
        }

        let mut all_comments = Vec::new();

        for entry in std::fs::read_dir(&comments_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let data = std::fs::read_to_string(&path)?;
                let comments: Vec<Comment> = serde_json::from_str(&data)?;
                all_comments.extend(comments);
            }
        }

        // Sort by timestamp (newest first)
        all_comments.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(all_comments)
    }
}

impl Default for CommentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A comment on a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID
    pub id: String,

    /// Commit this comment is on
    pub commit_id: String,

    /// Author of the comment
    pub author: String,

    /// Comment text
    pub text: String,

    /// When comment was created
    pub created_at: DateTime<Utc>,
}

// ========== Helper Functions ==========

/// Parse metadata line (e.g., "BPM: 120")
fn parse_metadata_line(line: &str) -> Option<(&str, &str)> {
    if let Some(pos) = line.find(':') {
        let key = line[..pos].trim();
        let value = line[pos + 1..].trim();

        // Only consider as metadata if key is uppercase or known metadata
        if key.chars().all(|c| c.is_uppercase() || c == ' ')
            || matches!(
                key,
                "BPM" | "Sample Rate" | "Key" | "Tempo" | "Time Signature"
            )
        {
            return Some((key, value));
        }
    }

    None
}

/// Extract author from commit message
/// Tries to find author in message, falls back to "unknown"
fn extract_author_from_message(message: &str) -> Option<String> {
    for line in message.lines() {
        if line.trim().starts_with("Author:") {
            let author = line.trim().strip_prefix("Author:")?.trim();
            return Some(author.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_type_icon() {
        assert_eq!(ActivityType::Commit.icon(), "●");
        assert_eq!(ActivityType::LockAcquired.icon(), "🔒");
        assert_eq!(ActivityType::Comment.icon(), "💬");
    }

    #[test]
    fn test_activity_type_label() {
        assert_eq!(ActivityType::Commit.label(), "Commit");
        assert_eq!(ActivityType::LockReleased.label(), "Lock Released");
    }

    #[test]
    fn test_parse_metadata_line() {
        assert_eq!(parse_metadata_line("BPM: 120"), Some(("BPM", "120")));
        assert_eq!(
            parse_metadata_line("Sample Rate: 48000Hz"),
            Some(("Sample Rate", "48000Hz"))
        );
        assert_eq!(parse_metadata_line("Not metadata"), None);
    }

    #[test]
    fn test_extract_author_from_message() {
        let message = "Commit message\n\nAuthor: john@laptop";
        assert_eq!(
            extract_author_from_message(message),
            Some("john@laptop".to_string())
        );

        assert_eq!(extract_author_from_message("No author here"), None);
    }

    #[test]
    fn test_activity_feed_creation() {
        let _feed = ActivityFeed::new();
        assert!(true); // Just verify it compiles
    }

    #[test]
    fn test_team_manager_creation() {
        let _manager = TeamManager::new();
        assert!(true);
    }

    #[test]
    fn test_comment_manager_creation() {
        let manager = CommentManager::new();
        assert_eq!(manager.comments_dir, ".oxen/comments");
    }

    #[test]
    fn test_add_and_get_comments() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create comments directory
        let comments_dir = repo_path.join(".oxen").join("comments");
        std::fs::create_dir_all(&comments_dir).unwrap();

        let manager = CommentManager::new();
        let commit_id = "abc123";
        let author = "test@user";
        let text = "Great work on this commit!";

        // Add a comment
        let result = manager.add_comment(repo_path, commit_id, author, text);
        assert!(result.is_ok());

        let comment = result.unwrap();
        assert_eq!(comment.author, author);
        assert_eq!(comment.text, text);
        assert_eq!(comment.commit_id, commit_id);

        // Retrieve comments
        let comments = manager.get_comments(repo_path, commit_id).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, text);
    }

    #[test]
    fn test_multiple_comments_on_commit() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen").join("comments")).unwrap();

        let manager = CommentManager::new();
        let commit_id = "def456";

        // Add multiple comments
        manager
            .add_comment(repo_path, commit_id, "user1@host", "First comment")
            .unwrap();
        manager
            .add_comment(repo_path, commit_id, "user2@host", "Second comment")
            .unwrap();
        manager
            .add_comment(repo_path, commit_id, "user3@host", "Third comment")
            .unwrap();

        // Get all comments
        let comments = manager.get_comments(repo_path, commit_id).unwrap();
        assert_eq!(comments.len(), 3);
        assert_eq!(comments[0].text, "First comment");
        assert_eq!(comments[1].text, "Second comment");
        assert_eq!(comments[2].text, "Third comment");
    }

    #[test]
    fn test_get_comments_nonexistent_commit() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen").join("comments")).unwrap();

        let manager = CommentManager::new();

        // Should return empty vec for non-existent commit
        let comments = manager.get_comments(repo_path, "nonexistent").unwrap();
        assert_eq!(comments.len(), 0);
    }

    #[test]
    fn test_activity_creation_from_metadata() {
        let activity = Activity {
            id: "abc123".to_string(),
            activity_type: ActivityType::Commit,
            author: "john@laptop".to_string(),
            timestamp: Utc::now(),
            message: "Added drums track".to_string(),
            metadata: HashMap::from([
                ("BPM".to_string(), "128".to_string()),
                ("Sample Rate".to_string(), "48000Hz".to_string()),
            ]),
        };

        assert_eq!(activity.activity_type.icon(), "●");
        assert_eq!(activity.metadata.get("BPM").unwrap(), "128");
        assert!(activity.message.contains("drums"));
    }

    #[test]
    fn test_team_member_stats() {
        let member = TeamMember {
            name: "alice@studio".to_string(),
            commit_count: 15,
            last_active: Utc::now(),
        };

        assert_eq!(member.name, "alice@studio");
        assert_eq!(member.commit_count, 15);
    }

    #[test]
    fn test_parse_metadata_empty_line() {
        assert_eq!(parse_metadata_line(""), None);
        assert_eq!(parse_metadata_line("   "), None);
    }

    #[test]
    fn test_extract_author_multiline() {
        let message = "First line\nSecond line\n\nAuthor: bob@machine\nSome other text";
        assert_eq!(
            extract_author_from_message(message),
            Some("bob@machine".to_string())
        );
    }

    #[test]
    fn test_comment_id_uniqueness() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen").join("comments")).unwrap();

        let manager = CommentManager::new();
        let commit_id = "test123";

        let comment1 = manager
            .add_comment(repo_path, commit_id, "user@host", "Comment 1")
            .unwrap();

        // Wait a tiny bit to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(2));

        let comment2 = manager
            .add_comment(repo_path, commit_id, "user@host", "Comment 2")
            .unwrap();

        // Comment IDs should be unique
        assert_ne!(comment1.id, comment2.id);
    }
}
