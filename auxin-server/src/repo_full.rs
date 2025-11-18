use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

#[cfg(feature = "full-oxen")]
use liboxen::api;

use crate::error::{AppError, AppResult};
use crate::extensions::{FileLock, LogicProMetadata};

/// Repository operations wrapper using liboxen
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

        info!("Initializing repository at: {:?}", repo_path);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to create repository directory: {}", e))
        })?;

        #[cfg(feature = "full-oxen")]
        {
            // Initialize using liboxen
            api::local::repositories::init(&repo_path).map_err(|e| {
                AppError::Internal(format!("Failed to initialize repository: {}", e))
            })?;
        }

        #[cfg(feature = "mock-oxen")]
        {
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

            warn!("Using mock Oxen implementation - full VCS operations not available");
        }

        // Create Auxin extension directories
        let oxen_dir = repo_path.join(".oxen");
        std::fs::create_dir_all(oxen_dir.join("metadata")).map_err(|e| {
            AppError::Internal(format!("Failed to create metadata directory: {}", e))
        })?;

        std::fs::create_dir_all(oxen_dir.join("locks")).map_err(|e| {
            AppError::Internal(format!("Failed to create locks directory: {}", e))
        })?;

        info!("Repository initialized successfully");
        Ok(Self { repo_path })
    }

    /// Add files to the staging area
    pub fn add(&self, paths: &[impl AsRef<Path>]) -> AppResult<()> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        for path in paths {
            let full_path = if path.as_ref().is_absolute() {
                path.as_ref().to_path_buf()
            } else {
                self.repo_path.join(path)
            };

            debug!("Adding file: {:?}", full_path);

            api::local::staging::add(&repo, &full_path).map_err(|e| {
                AppError::Internal(format!("Failed to add file {:?}: {}", path.as_ref(), e))
            })?;
        }

        Ok(())
    }

    /// Commit staged changes
    pub fn commit(&self, message: &str) -> AppResult<String> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        info!("Creating commit: {}", message);

        let commit = api::local::commits::commit(&repo, message).map_err(|e| {
            AppError::Internal(format!("Failed to create commit: {}", e))
        })?;

        let commit_id = commit.id.clone();
        info!("Commit created: {}", commit_id);

        Ok(commit_id)
    }

    /// Get commit history
    pub fn log(&self, limit: Option<usize>) -> AppResult<Vec<CommitInfo>> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        let commits = api::local::commits::list(&repo).map_err(|e| {
            AppError::Internal(format!("Failed to get commit history: {}", e))
        })?;

        let mut result: Vec<CommitInfo> = commits
            .into_iter()
            .map(|c| CommitInfo {
                id: c.id,
                message: c.message,
                author: c.author.name,
                timestamp: c.timestamp.to_rfc3339(),
            })
            .collect();

        if let Some(limit) = limit {
            result.truncate(limit);
        }

        Ok(result)
    }

    /// Push to remote repository
    pub fn push(&self, remote: &str, branch: &str) -> AppResult<()> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        info!("Pushing to remote: {} (branch: {})", remote, branch);

        api::local::branches::push_remote_branch(&repo, remote, branch, branch).map_err(|e| {
            AppError::Internal(format!("Failed to push: {}", e))
        })?;

        info!("Push completed successfully");
        Ok(())
    }

    /// Pull from remote repository
    pub fn pull(&self, remote: &str, branch: &str) -> AppResult<()> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        info!("Pulling from remote: {} (branch: {})", remote, branch);

        api::local::branches::pull_remote_branch(&repo, remote, branch).map_err(|e| {
            AppError::Internal(format!("Failed to pull: {}", e))
        })?;

        info!("Pull completed successfully");
        Ok(())
    }

    /// Clone a remote repository
    pub fn clone(remote_url: &str, dest_path: impl AsRef<Path>) -> AppResult<Self> {
        let dest_path = dest_path.as_ref();

        info!("Cloning repository from: {} to: {:?}", remote_url, dest_path);

        api::local::repositories::clone(remote_url, dest_path).map_err(|e| {
            AppError::Internal(format!("Failed to clone repository: {}", e))
        })?;

        info!("Clone completed successfully");
        Ok(Self {
            repo_path: dest_path.to_path_buf(),
        })
    }

    /// Get current branch name
    pub fn current_branch(&self) -> AppResult<String> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        let branch = api::local::branches::current_branch(&repo).map_err(|e| {
            AppError::Internal(format!("Failed to get current branch: {}", e))
        })?;

        Ok(branch.name)
    }

    /// List all branches
    pub fn list_branches(&self) -> AppResult<Vec<String>> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        let branches = api::local::branches::list(&repo).map_err(|e| {
            AppError::Internal(format!("Failed to list branches: {}", e))
        })?;

        Ok(branches.into_iter().map(|b| b.name).collect())
    }

    /// Create a new branch
    pub fn create_branch(&self, branch_name: &str) -> AppResult<()> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        info!("Creating branch: {}", branch_name);

        api::local::branches::create_from_head(&repo, branch_name).map_err(|e| {
            AppError::Internal(format!("Failed to create branch: {}", e))
        })?;

        Ok(())
    }

    /// Checkout a branch
    pub fn checkout(&self, branch_name: &str) -> AppResult<()> {
        let repo = api::local::repositories::get(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to open repository: {}", e))
        })?;

        info!("Checking out branch: {}", branch_name);

        api::local::branches::checkout(&repo, branch_name).map_err(|e| {
            AppError::Internal(format!("Failed to checkout branch: {}", e))
        })?;

        Ok(())
    }

    /// Get repository path
    pub fn path(&self) -> &Path {
        &self.repo_path
    }

    // Auxin Extensions

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

        debug!("Metadata stored for commit: {}", commit_id);
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
