use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::error::{AppError, AppResult};
use crate::extensions::{FileLock, LogicProMetadata};

// Import liboxen modules
use liboxen::error::OxenError;
use liboxen::model::{Branch, Commit, LocalRepository};
use liboxen::repositories;

/// Repository operations wrapper using liboxen 0.38+
pub struct RepositoryOps {
    repo: LocalRepository,
}

impl RepositoryOps {
    /// Open an existing repository
    pub fn open(repo_path: impl AsRef<Path>) -> AppResult<Self> {
        let repo_path = repo_path.as_ref();

        // Verify .oxen directory exists
        if !repo_path.join(".oxen").exists() {
            return Err(AppError::NotFound("Repository not found".to_string()));
        }

        // Load repository
        let repo = LocalRepository::from_dir(repo_path)
            .map_err(|e| AppError::Internal(format!("Failed to open repository: {}", e)))?;

        Ok(Self { repo })
    }

    /// Initialize a new repository
    pub fn init(repo_path: impl AsRef<Path>) -> AppResult<Self> {
        let repo_path = repo_path.as_ref();

        info!("Initializing repository at: {:?}", repo_path);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to create repository directory: {}", e))
        })?;

        // Initialize using liboxen
        let repo = repositories::init(repo_path)
            .map_err(|e| AppError::Internal(format!("Failed to initialize repository: {}", e)))?;

        // Create Auxin extension directories
        let oxen_dir = repo_path.join(".oxen");
        std::fs::create_dir_all(oxen_dir.join("metadata")).map_err(|e| {
            AppError::Internal(format!("Failed to create metadata directory: {}", e))
        })?;

        std::fs::create_dir_all(oxen_dir.join("locks")).map_err(|e| {
            AppError::Internal(format!("Failed to create locks directory: {}", e))
        })?;

        info!("Repository initialized successfully");
        Ok(Self { repo })
    }

    /// Add files to the staging area
    pub fn add(&self, paths: &[impl AsRef<Path>]) -> AppResult<()> {
        // Note: liboxen 0.38's add is async, but we're in a sync context
        // We need to spawn a runtime to handle this
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::Internal(format!("Failed to create runtime: {}", e)))?;

        for path in paths {
            let full_path = if path.as_ref().is_absolute() {
                path.as_ref().to_path_buf()
            } else {
                self.repo.path.join(path)
            };

            debug!("Adding file: {:?}", full_path);

            runtime
                .block_on(repositories::add(&self.repo, &full_path))
                .map_err(|e| {
                    AppError::Internal(format!("Failed to add file {:?}: {}", path.as_ref(), e))
                })?;
        }

        Ok(())
    }

    /// Commit staged changes
    pub fn commit(&self, message: &str) -> AppResult<String> {
        info!("Creating commit: {}", message);

        let commit = repositories::commit(&self.repo, message)
            .map_err(|e| AppError::Internal(format!("Failed to create commit: {}", e)))?;

        let commit_id = commit.id.clone();
        info!("Commit created: {}", commit_id);

        Ok(commit_id)
    }

    /// Get commit history
    pub fn log(&self, limit: Option<usize>) -> AppResult<Vec<CommitInfo>> {
        let commits = repositories::commits::list(&self.repo)
            .map_err(|e| AppError::Internal(format!("Failed to get commit history: {}", e)))?;

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
        info!("Pushing to remote: {} (branch: {})", remote, branch);

        repositories::push(&self.repo, remote, branch)
            .map_err(|e| AppError::Internal(format!("Failed to push: {}", e)))?;

        info!("Push completed successfully");
        Ok(())
    }

    /// Pull from remote repository
    pub fn pull(&self, remote: &str, branch: &str) -> AppResult<()> {
        info!("Pulling from remote: {} (branch: {})", remote, branch);

        repositories::pull(&self.repo, remote, branch)
            .map_err(|e| AppError::Internal(format!("Failed to pull: {}", e)))?;

        info!("Pull completed successfully");
        Ok(())
    }

    /// Clone a remote repository
    pub fn clone(remote_url: &str, dest_path: impl AsRef<Path>) -> AppResult<Self> {
        let dest_path = dest_path.as_ref();

        info!("Cloning repository from: {} to: {:?}", remote_url, dest_path);

        let repo = repositories::clone(remote_url, dest_path)
            .map_err(|e| AppError::Internal(format!("Failed to clone repository: {}", e)))?;

        info!("Clone completed successfully");
        Ok(Self { repo })
    }

    /// Get current branch name
    pub fn current_branch(&self) -> AppResult<String> {
        let branch = repositories::branches::current_branch(&self.repo)
            .map_err(|e| AppError::Internal(format!("Failed to get current branch: {}", e)))?;

        Ok(branch
            .map(|b| b.name)
            .unwrap_or_else(|| "main".to_string()))
    }

    /// List all branches
    pub fn list_branches(&self) -> AppResult<Vec<String>> {
        let branches = repositories::branches::list(&self.repo)
            .map_err(|e| AppError::Internal(format!("Failed to list branches: {}", e)))?;

        Ok(branches.into_iter().map(|b| b.name).collect())
    }

    /// Create a new branch
    pub fn create_branch(&self, branch_name: &str) -> AppResult<()> {
        info!("Creating branch: {}", branch_name);

        repositories::branches::create_from_head(&self.repo, branch_name)
            .map_err(|e| AppError::Internal(format!("Failed to create branch: {}", e)))?;

        Ok(())
    }

    /// Checkout a branch
    pub fn checkout(&self, branch_name: &str) -> AppResult<()> {
        info!("Checking out branch: {}", branch_name);

        repositories::checkout(&self.repo, branch_name)
            .map_err(|e| AppError::Internal(format!("Failed to checkout branch: {}", e)))?;

        Ok(())
    }

    /// Get repository path
    pub fn path(&self) -> &Path {
        &self.repo.path
    }

    // Auxin Extensions

    /// Store Logic Pro metadata for a commit
    pub fn store_metadata(&self, commit_id: &str, metadata: &LogicProMetadata) -> AppResult<()> {
        let metadata_path = self
            .repo
            .path
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
            .repo
            .path
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
        FileLock::acquire(&self.repo.path, user, machine_id, timeout_hours).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                AppError::BadRequest(e.to_string())
            } else {
                AppError::Internal(format!("Failed to acquire lock: {}", e))
            }
        })
    }

    /// Release lock for this repository
    pub fn release_lock(&self, lock_id: &str) -> AppResult<()> {
        FileLock::release(&self.repo.path, lock_id).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::Unauthorized(e.to_string())
            } else {
                AppError::Internal(format!("Failed to release lock: {}", e))
            }
        })
    }

    /// Update lock heartbeat
    pub fn heartbeat_lock(&self, lock_id: &str) -> AppResult<FileLock> {
        FileLock::heartbeat(&self.repo.path, lock_id).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::Unauthorized(e.to_string())
            } else {
                AppError::Internal(format!("Failed to update heartbeat: {}", e))
            }
        })
    }

    /// Get lock status
    pub fn lock_status(&self) -> AppResult<Option<FileLock>> {
        FileLock::status(&self.repo.path).map_err(|e| {
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
