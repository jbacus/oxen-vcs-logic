use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

use crate::error::{AppError, AppResult};

/// Project visibility settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

/// Project metadata stored in .oxen/project.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub owner_id: String,
    pub owner_username: String,
    pub visibility: Visibility,
    pub collaborators: Vec<String>, // User IDs
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProjectMetadata {
    /// Create new project metadata
    pub fn new(owner_id: String, owner_username: String, visibility: Visibility) -> Self {
        let now = Utc::now();
        Self {
            owner_id,
            owner_username,
            visibility,
            collaborators: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the path to the project metadata file
    fn metadata_path(repo_path: &Path) -> PathBuf {
        repo_path.join(".oxen").join("project.json")
    }

    /// Load project metadata from disk
    pub fn load(repo_path: &Path) -> AppResult<Self> {
        let path = Self::metadata_path(repo_path);

        if !path.exists() {
            return Err(AppError::NotFound(
                "Project metadata not found".to_string(),
            ));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| AppError::Internal(format!("Failed to read project metadata: {}", e)))?;

        let metadata: ProjectMetadata = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse project metadata: {}", e)))?;

        Ok(metadata)
    }

    /// Save project metadata to disk
    pub fn save(&self, repo_path: &Path) -> AppResult<()> {
        let path = Self::metadata_path(repo_path);

        // Ensure .oxen directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Internal(format!("Failed to create .oxen directory: {}", e)))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Internal(format!("Failed to serialize project metadata: {}", e)))?;

        std::fs::write(&path, content)
            .map_err(|e| AppError::Internal(format!("Failed to write project metadata: {}", e)))?;

        Ok(())
    }

    /// Check if a user is the owner
    pub fn is_owner(&self, user_id: &str) -> bool {
        self.owner_id == user_id
    }

    /// Check if a user is a collaborator
    pub fn is_collaborator(&self, user_id: &str) -> bool {
        self.collaborators.contains(&user_id.to_string())
    }

    /// Check if a user has write access (owner or collaborator)
    pub fn has_write_access(&self, user_id: &str) -> bool {
        self.is_owner(user_id) || self.is_collaborator(user_id)
    }

    /// Check if a user has read access
    pub fn has_read_access(&self, user_id: Option<&str>) -> bool {
        match self.visibility {
            Visibility::Public => true,
            Visibility::Private => {
                if let Some(uid) = user_id {
                    self.has_write_access(uid)
                } else {
                    false
                }
            }
        }
    }

    /// Add a collaborator
    pub fn add_collaborator(&mut self, user_id: String) -> AppResult<()> {
        if self.is_owner(&user_id) {
            return Err(AppError::BadRequest(
                "Owner is already a collaborator by default".to_string(),
            ));
        }

        if self.is_collaborator(&user_id) {
            return Err(AppError::BadRequest(
                "User is already a collaborator".to_string(),
            ));
        }

        self.collaborators.push(user_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove a collaborator
    pub fn remove_collaborator(&mut self, user_id: &str) -> AppResult<()> {
        if !self.is_collaborator(user_id) {
            return Err(AppError::BadRequest(
                "User is not a collaborator".to_string(),
            ));
        }

        self.collaborators.retain(|id| id != user_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update visibility
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
        self.updated_at = Utc::now();
    }
}

/// Authorization helper functions
pub struct ProjectAuth;

impl ProjectAuth {
    /// Check if user can read a repository
    pub fn can_read(repo_path: &Path, user_id: Option<&str>) -> AppResult<bool> {
        let metadata = ProjectMetadata::load(repo_path)?;
        Ok(metadata.has_read_access(user_id))
    }

    /// Check if user can write to a repository
    pub fn can_write(repo_path: &Path, user_id: &str) -> AppResult<bool> {
        let metadata = ProjectMetadata::load(repo_path)?;
        Ok(metadata.has_write_access(user_id))
    }

    /// Check if user is the owner
    pub fn is_owner(repo_path: &Path, user_id: &str) -> AppResult<bool> {
        let metadata = ProjectMetadata::load(repo_path)?;
        Ok(metadata.is_owner(user_id))
    }

    /// Require read access (returns error if denied)
    pub fn require_read(repo_path: &Path, user_id: Option<&str>) -> AppResult<()> {
        if !Self::can_read(repo_path, user_id)? {
            return Err(AppError::Forbidden(
                "You do not have read access to this repository".to_string(),
            ));
        }
        Ok(())
    }

    /// Require write access (returns error if denied)
    pub fn require_write(repo_path: &Path, user_id: &str) -> AppResult<()> {
        if !Self::can_write(repo_path, user_id)? {
            return Err(AppError::Forbidden(
                "You do not have write access to this repository".to_string(),
            ));
        }
        Ok(())
    }

    /// Require owner access (returns error if denied)
    pub fn require_owner(repo_path: &Path, user_id: &str) -> AppResult<()> {
        if !Self::is_owner(repo_path, user_id)? {
            return Err(AppError::Forbidden(
                "Only the repository owner can perform this action".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_metadata() -> ProjectMetadata {
        ProjectMetadata::new(
            "user-123".to_string(),
            "testuser".to_string(),
            Visibility::Private,
        )
    }

    #[test]
    fn test_new_project_metadata() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.owner_id, "user-123");
        assert_eq!(metadata.owner_username, "testuser");
        assert_eq!(metadata.visibility, Visibility::Private);
        assert!(metadata.collaborators.is_empty());
    }

    #[test]
    fn test_is_owner() {
        let metadata = create_test_metadata();
        assert!(metadata.is_owner("user-123"));
        assert!(!metadata.is_owner("user-456"));
    }

    #[test]
    fn test_add_collaborator() {
        let mut metadata = create_test_metadata();
        metadata.add_collaborator("user-456".to_string()).unwrap();
        assert!(metadata.is_collaborator("user-456"));
        assert_eq!(metadata.collaborators.len(), 1);
    }

    #[test]
    fn test_add_duplicate_collaborator() {
        let mut metadata = create_test_metadata();
        metadata.add_collaborator("user-456".to_string()).unwrap();
        let result = metadata.add_collaborator("user-456".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_add_owner_as_collaborator() {
        let mut metadata = create_test_metadata();
        let result = metadata.add_collaborator("user-123".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_collaborator() {
        let mut metadata = create_test_metadata();
        metadata.add_collaborator("user-456".to_string()).unwrap();
        metadata.remove_collaborator("user-456").unwrap();
        assert!(!metadata.is_collaborator("user-456"));
        assert!(metadata.collaborators.is_empty());
    }

    #[test]
    fn test_has_write_access() {
        let mut metadata = create_test_metadata();

        // Owner has write access
        assert!(metadata.has_write_access("user-123"));

        // Non-collaborator doesn't have write access
        assert!(!metadata.has_write_access("user-456"));

        // Collaborator has write access
        metadata.add_collaborator("user-456".to_string()).unwrap();
        assert!(metadata.has_write_access("user-456"));
    }

    #[test]
    fn test_has_read_access_public() {
        let mut metadata = create_test_metadata();
        metadata.set_visibility(Visibility::Public);

        // Anyone can read public repos
        assert!(metadata.has_read_access(None));
        assert!(metadata.has_read_access(Some("user-456")));
    }

    #[test]
    fn test_has_read_access_private() {
        let metadata = create_test_metadata(); // Private by default

        // Anonymous users cannot read private repos
        assert!(!metadata.has_read_access(None));

        // Non-collaborators cannot read private repos
        assert!(!metadata.has_read_access(Some("user-456")));

        // Owner can read
        assert!(metadata.has_read_access(Some("user-123")));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        std::fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        let mut metadata = create_test_metadata();
        metadata.add_collaborator("user-456".to_string()).unwrap();

        // Save
        metadata.save(&repo_path).unwrap();

        // Load
        let loaded = ProjectMetadata::load(&repo_path).unwrap();
        assert_eq!(loaded.owner_id, metadata.owner_id);
        assert_eq!(loaded.owner_username, metadata.owner_username);
        assert_eq!(loaded.visibility, metadata.visibility);
        assert_eq!(loaded.collaborators, metadata.collaborators);
    }

    #[test]
    fn test_set_visibility() {
        let mut metadata = create_test_metadata();
        assert_eq!(metadata.visibility, Visibility::Private);

        metadata.set_visibility(Visibility::Public);
        assert_eq!(metadata.visibility, Visibility::Public);
    }
}
