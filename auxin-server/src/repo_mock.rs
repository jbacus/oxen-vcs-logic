// Mock implementations for Oxen VCS operations
// Used when liboxen is not available (e.g., macOS 26.x compilation issues)

use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::error::{AppError, AppResult};
use crate::extensions::{FileLock, LogicProMetadata};

/// Repository operations wrapper (mock implementation)
pub struct RepositoryOps {
    repo_path: PathBuf,
}

impl RepositoryOps {
    /// Open an existing repository
    pub fn open(repo_path: impl AsRef<Path>) -> AppResult<Self> {
        let repo_path = repo_path.as_ref().to_path_buf();

        // Verify .oxen directory exists
        if !repo_path.join(".oxen").exists() {
            return Err(AppError::NotFound("Repository not found".to_string()));
        }

        Ok(Self { repo_path })
    }

    /// Initialize a new repository
    pub fn init(repo_path: impl AsRef<Path>) -> AppResult<Self> {
        let repo_path = repo_path.as_ref().to_path_buf();

        info!("Initializing repository at: {:?} (mock mode)", repo_path);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to create repository directory: {}", e))
        })?;

        // Mock implementation: create minimal .oxen structure
        let oxen_dir = repo_path.join(".oxen");
        std::fs::create_dir_all(&oxen_dir).map_err(|e| {
            AppError::Internal(format!("Failed to create .oxen directory: {}", e))
        })?;

        // Create HEAD file
        std::fs::write(oxen_dir.join("HEAD"), "refs/heads/main\n").map_err(|e| {
            AppError::Internal(format!("Failed to create HEAD: {}", e))
        })?;

        // Create refs structure
        std::fs::create_dir_all(oxen_dir.join("refs/heads")).map_err(|e| {
            AppError::Internal(format!("Failed to create refs: {}", e))
        })?;

        // Create config
        std::fs::write(
            oxen_dir.join("config.toml"),
            "[repository]\nversion = \"0.2.0-mock\"\n",
        )
        .map_err(|e| AppError::Internal(format!("Failed to create config: {}", e)))?;

        // Create Auxin extension directories
        std::fs::create_dir_all(oxen_dir.join("metadata")).map_err(|e| {
            AppError::Internal(format!("Failed to create metadata directory: {}", e))
        })?;

        std::fs::create_dir_all(oxen_dir.join("locks")).map_err(|e| {
            AppError::Internal(format!("Failed to create locks directory: {}", e))
        })?;

        warn!("Using mock Oxen implementation - VCS operations will return NotImplemented");
        info!("Repository structure created successfully");

        Ok(Self { repo_path })
    }

    /// Add files to the staging area (mock)
    pub fn add(&self, _paths: &[impl AsRef<Path>]) -> AppResult<()> {
        Err(AppError::NotImplemented(
            "VCS add operation requires full-oxen feature".to_string(),
        ))
    }

    /// Commit staged changes (mock)
    pub fn commit(&self, _message: &str) -> AppResult<String> {
        Err(AppError::NotImplemented(
            "VCS commit operation requires full-oxen feature".to_string(),
        ))
    }

    /// Get commit history (mock)
    pub fn log(&self, _limit: Option<usize>) -> AppResult<Vec<CommitInfo>> {
        warn!("Mock VCS: Returning empty commit history");
        Ok(Vec::new())
    }

    /// Push to remote repository (mock)
    pub fn push(&self, _remote: &str, _branch: &str) -> AppResult<()> {
        Err(AppError::NotImplemented(
            "VCS push operation requires full-oxen feature".to_string(),
        ))
    }

    /// Pull from remote repository (mock)
    pub fn pull(&self, _remote: &str, _branch: &str) -> AppResult<()> {
        Err(AppError::NotImplemented(
            "VCS pull operation requires full-oxen feature".to_string(),
        ))
    }

    /// Clone a remote repository (mock)
    pub fn clone(_remote_url: &str, _dest_path: impl AsRef<Path>) -> AppResult<Self> {
        Err(AppError::NotImplemented(
            "VCS clone operation requires full-oxen feature".to_string(),
        ))
    }

    /// Get current branch name (mock)
    pub fn current_branch(&self) -> AppResult<String> {
        // Try to read HEAD file
        let head_path = self.repo_path.join(".oxen/HEAD");
        if head_path.exists() {
            let content = std::fs::read_to_string(&head_path)
                .map_err(|e| AppError::Internal(format!("Failed to read HEAD: {}", e)))?;

            if let Some(branch) = content.strip_prefix("refs/heads/") {
                return Ok(branch.trim().to_string());
            }
        }

        Ok("main".to_string())
    }

    /// List all branches (mock)
    pub fn list_branches(&self) -> AppResult<Vec<String>> {
        warn!("Mock VCS: Returning mock branch list");
        Ok(vec!["main".to_string()])
    }

    /// Create a new branch (mock)
    pub fn create_branch(&self, _branch_name: &str) -> AppResult<()> {
        Err(AppError::NotImplemented(
            "VCS branch creation requires full-oxen feature".to_string(),
        ))
    }

    /// Checkout a branch (mock)
    pub fn checkout(&self, _branch_name: &str) -> AppResult<()> {
        Err(AppError::NotImplemented(
            "VCS checkout operation requires full-oxen feature".to_string(),
        ))
    }

    /// Get repository path
    pub fn path(&self) -> &Path {
        &self.repo_path
    }

    // Auxin Extensions (these work in mock mode)

    /// Store Logic Pro metadata for a commit
    pub fn store_metadata(&self, commit_id: &str, metadata: &LogicProMetadata) -> AppResult<()> {
        let metadata_path = self
            .repo_path
            .join(".oxen")
            .join("metadata")
            .join(format!("{}.json", commit_id));

        let json = serde_json::to_string_pretty(metadata).map_err(|e| {
            AppError::Internal(format!("Failed to serialize metadata: {}", e))
        })?;

        std::fs::write(&metadata_path, json).map_err(|e| {
            AppError::Internal(format!("Failed to write metadata: {}", e))
        })?;

        info!("Metadata stored for commit: {}", commit_id);
        Ok(())
    }

    /// Retrieve Logic Pro metadata for a commit
    pub fn get_metadata(&self, commit_id: &str) -> AppResult<Option<LogicProMetadata>> {
        let metadata_path = self
            .repo_path
            .join(".oxen")
            .join("metadata")
            .join(format!("{}.json", commit_id));

        if !metadata_path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&metadata_path).map_err(|e| {
            AppError::Internal(format!("Failed to read metadata: {}", e))
        })?;

        let metadata = serde_json::from_str(&json).map_err(|e| {
            AppError::Internal(format!("Failed to parse metadata: {}", e))
        })?;

        Ok(Some(metadata))
    }

    /// Acquire lock for this repository
    pub fn acquire_lock(
        &self,
        user: &str,
        machine_id: &str,
        timeout_hours: u64,
    ) -> AppResult<FileLock> {
        FileLock::acquire(&self.repo_path, user, machine_id, timeout_hours).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                AppError::BadRequest(e.to_string())
            } else {
                AppError::Internal(format!("Failed to acquire lock: {}", e))
            }
        })
    }

    /// Release lock for this repository
    pub fn release_lock(&self, lock_id: &str) -> AppResult<()> {
        FileLock::release(&self.repo_path, lock_id).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::Unauthorized(e.to_string())
            } else {
                AppError::Internal(format!("Failed to release lock: {}", e))
            }
        })
    }

    /// Update lock heartbeat
    pub fn heartbeat_lock(&self, lock_id: &str) -> AppResult<FileLock> {
        FileLock::heartbeat(&self.repo_path, lock_id).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::Unauthorized(e.to_string())
            } else {
                AppError::Internal(format!("Failed to update heartbeat: {}", e))
            }
        })
    }

    /// Get lock status
    pub fn lock_status(&self) -> AppResult<Option<FileLock>> {
        FileLock::status(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to get lock status: {}", e))
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}
