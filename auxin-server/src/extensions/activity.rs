use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

use crate::error::{AppError, AppResult};

/// Activity event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Commit,
    LockAcquired,
    LockReleased,
    BranchCreated,
    UserJoined,
    Push,
    Pull,
}

/// Single activity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub activity_type: ActivityType,
    pub user: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Activity storage for a repository
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivityLog {
    pub activities: Vec<Activity>,
}

impl ActivityLog {
    /// Get activity log file path for a repository
    fn file_path(repo_path: &Path) -> std::path::PathBuf {
        repo_path.join(".oxen").join("activity.json")
    }

    /// Load activity log from disk
    pub fn load(repo_path: &Path) -> AppResult<Self> {
        let path = Self::file_path(repo_path);

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| AppError::Internal(format!("Failed to read activity log: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse activity log: {}", e)))
    }

    /// Save activity log to disk
    pub fn save(&self, repo_path: &Path) -> AppResult<()> {
        let path = Self::file_path(repo_path);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Internal(format!("Failed to create directory: {}", e)))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Internal(format!("Failed to serialize activity log: {}", e)))?;

        std::fs::write(&path, content)
            .map_err(|e| AppError::Internal(format!("Failed to write activity log: {}", e)))?;

        Ok(())
    }

    /// Add a new activity
    pub fn add(&mut self, activity: Activity) {
        self.activities.push(activity);
    }

    /// Get recent activities (most recent first)
    pub fn recent(&self, limit: usize) -> Vec<&Activity> {
        let mut activities: Vec<&Activity> = self.activities.iter().collect();
        activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        activities.into_iter().take(limit).collect()
    }

    /// Get activities by type
    pub fn by_type(&self, activity_type: ActivityType) -> Vec<&Activity> {
        self.activities
            .iter()
            .filter(|a| a.activity_type == activity_type)
            .collect()
    }

    /// Get activities by user
    pub fn by_user(&self, user: &str) -> Vec<&Activity> {
        self.activities
            .iter()
            .filter(|a| a.user == user)
            .collect()
    }
}

/// Log a new activity for a repository
pub fn log_activity(
    repo_path: &Path,
    activity_type: ActivityType,
    user: &str,
    message: &str,
    metadata: Option<serde_json::Value>,
) -> AppResult<Activity> {
    let mut log = ActivityLog::load(repo_path)?;

    let activity = Activity {
        id: uuid::Uuid::new_v4().to_string(),
        activity_type: activity_type.clone(),
        user: user.to_string(),
        message: message.to_string(),
        timestamp: Utc::now(),
        metadata,
    };

    log.add(activity.clone());
    log.save(repo_path)?;

    info!(
        "Logged activity: {:?} by {} - {}",
        activity_type, user, message
    );

    Ok(activity)
}

/// Get recent activities for a repository
pub fn get_activities(repo_path: &Path, limit: usize) -> AppResult<Vec<Activity>> {
    let log = ActivityLog::load(repo_path)?;
    Ok(log.recent(limit).into_iter().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_and_retrieve_activity() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create .oxen directory
        std::fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        // Log an activity
        let activity = log_activity(
            repo_path,
            ActivityType::Commit,
            "testuser",
            "Initial commit",
            None,
        )
        .unwrap();

        assert_eq!(activity.user, "testuser");
        assert_eq!(activity.message, "Initial commit");
        assert_eq!(activity.activity_type, ActivityType::Commit);

        // Retrieve activities
        let activities = get_activities(repo_path, 10).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].id, activity.id);
    }

    #[test]
    fn test_multiple_activities_ordering() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        // Log multiple activities
        log_activity(
            repo_path,
            ActivityType::Commit,
            "user1",
            "First commit",
            None,
        )
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        log_activity(
            repo_path,
            ActivityType::LockAcquired,
            "user2",
            "Acquired lock",
            None,
        )
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        log_activity(
            repo_path,
            ActivityType::Commit,
            "user1",
            "Second commit",
            None,
        )
        .unwrap();

        // Retrieve activities (most recent first)
        let activities = get_activities(repo_path, 10).unwrap();
        assert_eq!(activities.len(), 3);
        assert_eq!(activities[0].message, "Second commit");
        assert_eq!(activities[1].message, "Acquired lock");
        assert_eq!(activities[2].message, "First commit");
    }

    #[test]
    fn test_activity_with_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        let metadata = serde_json::json!({
            "commit_id": "abc123",
            "files_changed": 5
        });

        let activity = log_activity(
            repo_path,
            ActivityType::Commit,
            "testuser",
            "Added features",
            Some(metadata.clone()),
        )
        .unwrap();

        assert_eq!(activity.metadata.unwrap(), metadata);
    }

    #[test]
    fn test_limit_activities() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        // Log 5 activities
        for i in 0..5 {
            log_activity(
                repo_path,
                ActivityType::Commit,
                "user",
                &format!("Commit {}", i),
                None,
            )
            .unwrap();
        }

        // Request only 2
        let activities = get_activities(repo_path, 2).unwrap();
        assert_eq!(activities.len(), 2);
    }

    #[test]
    fn test_empty_activity_log() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        let activities = get_activities(repo_path, 10).unwrap();
        assert!(activities.is_empty());
    }

    #[test]
    fn test_filter_by_type() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        log_activity(repo_path, ActivityType::Commit, "user", "Commit", None).unwrap();
        log_activity(repo_path, ActivityType::LockAcquired, "user", "Lock", None).unwrap();
        log_activity(repo_path, ActivityType::Commit, "user", "Commit 2", None).unwrap();

        let log = ActivityLog::load(repo_path).unwrap();
        let commits = log.by_type(ActivityType::Commit);
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn test_filter_by_user() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        std::fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        log_activity(repo_path, ActivityType::Commit, "user1", "Commit 1", None).unwrap();
        log_activity(repo_path, ActivityType::Commit, "user2", "Commit 2", None).unwrap();
        log_activity(repo_path, ActivityType::Commit, "user1", "Commit 3", None).unwrap();

        let log = ActivityLog::load(repo_path).unwrap();
        let user1_activities = log.by_user("user1");
        assert_eq!(user1_activities.len(), 2);
    }
}
