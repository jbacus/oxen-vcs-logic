use anyhow::{anyhow, Context, Result};
use crate::liboxen_stub as liboxen;
use liboxen::api;
use liboxen::command;
use liboxen::opts::AddOpts;
use std::path::{Path, PathBuf};

use crate::commit_metadata::CommitMetadata;
use crate::ignore_template::generate_oxenignore;
use crate::logic_project::LogicProject;
use crate::draft_manager::DraftManager;
use crate::{vlog, info};

/// Wrapper for Oxen repository operations
pub struct OxenRepository {
    pub path: PathBuf,
}

impl OxenRepository {
    /// Creates a new OxenRepository instance
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Initializes a new Oxen repository for a Logic Pro project
    ///
    /// This will:
    /// 1. Detect if the path is a valid Logic Pro project
    /// 2. Initialize an Oxen repository
    /// 3. Create a .oxenignore file with Logic Pro-specific patterns
    pub async fn init_for_logic_project(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        vlog!("=== Initializing Logic Pro Project Repository ===");
        vlog!("Target path: {}", path.display());

        // Detect Logic Pro project
        vlog!("Step 1: Detecting Logic Pro project structure...");
        let logic_project = LogicProject::detect(path)
            .context("Failed to detect Logic Pro project")?;

        info!("Detected Logic Pro project: {}", logic_project.name());
        vlog!("Project name: {}", logic_project.name());

        // Initialize Oxen repository
        vlog!("Step 2: Initializing Oxen repository...");
        let repo = api::local::repositories::init(path)
            .context("Failed to initialize Oxen repository")?;

        info!("Initialized Oxen repository at: {}", path.display());

        // Create .oxenignore file
        vlog!("Step 3: Creating .oxenignore file...");
        let ignore_path = path.join(".oxenignore");
        vlog!("Ignore file path: {}", ignore_path.display());

        let ignore_content = generate_oxenignore();
        vlog!("Generated ignore patterns ({} bytes)", ignore_content.len());

        tokio::fs::write(&ignore_path, ignore_content)
            .await
            .context("Failed to write .oxenignore file")?;

        info!("Created .oxenignore file");

        // Create repository instance
        let repo_instance = Self {
            path: path.to_path_buf(),
        };

        // Initialize draft branch workflow
        vlog!("Step 4: Initializing draft branch workflow...");
        info!("Initializing draft branch workflow...");

        let draft_manager = DraftManager::new(path)
            .context("Failed to create draft manager")?;

        draft_manager.initialize().await
            .context("Failed to initialize draft branch")?;

        vlog!("Draft branch initialized successfully");
        vlog!("=== Initialization Complete ===");

        Ok(repo_instance)
    }

    /// Initializes a new Oxen repository (generic)
    pub async fn init(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let repo = api::local::repositories::init(path)
            .context("Failed to initialize Oxen repository")?;

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// Gets the repository instance
    pub fn get_repo(&self) -> Result<liboxen::model::LocalRepository> {
        api::local::repositories::get(&self.path)
            .ok_or_else(|| anyhow!("Repository not found at: {}", self.path.display()))
    }

    /// Stages changes to the repository
    ///
    /// This wraps `oxen add`
    pub async fn stage_changes(&self, files: Vec<PathBuf>) -> Result<()> {
        let repo = self.get_repo()?;

        for file in &files {
            println!("Staging: {}", file.display());
        }

        let opts = AddOpts {
            paths: files,
            is_remote: false,
            directory: None,
        };

        command::add(&repo, &opts)
            .await
            .context("Failed to stage changes")?;

        println!("Successfully staged changes");

        Ok(())
    }

    /// Stages all changes in the repository
    pub async fn stage_all(&self) -> Result<()> {
        let repo = self.get_repo()?;

        println!("Staging all changes...");

        let opts = AddOpts {
            paths: vec![self.path.clone()],
            is_remote: false,
            directory: None,
        };

        command::add(&repo, &opts)
            .await
            .context("Failed to stage all changes")?;

        println!("Successfully staged all changes");

        Ok(())
    }

    /// Creates a commit with metadata
    pub async fn create_commit(&self, metadata: CommitMetadata) -> Result<String> {
        let repo = self.get_repo()?;

        let message = metadata.format_commit_message();

        println!("Creating commit with message:\n{}", message);

        let commit = command::commit(&repo, &message)
            .await
            .context("Failed to create commit")?;

        println!("Commit created: {}", commit.id);

        Ok(commit.id)
    }

    /// Gets the commit history
    pub async fn get_history(&self, limit: Option<usize>) -> Result<Vec<liboxen::model::Commit>> {
        let repo = self.get_repo()?;

        let mut commits = api::local::commits::list(&repo)
            .context("Failed to get commit history")?;

        if let Some(limit) = limit {
            commits.truncate(limit);
        }

        Ok(commits)
    }

    /// Restores the repository to a specific commit
    pub async fn restore(&self, commit_id: &str) -> Result<()> {
        let repo = self.get_repo()?;

        println!("Restoring to commit: {}", commit_id);

        command::checkout(&repo, commit_id)
            .await
            .context("Failed to restore to commit")?;

        println!("Successfully restored to commit: {}", commit_id);

        Ok(())
    }

    /// Gets the status of the repository
    pub async fn status(&self) -> Result<liboxen::model::StagedData> {
        let repo = self.get_repo()?;

        let status = command::status(&repo)
            .await
            .context("Failed to get repository status")?;

        Ok(status)
    }

    /// Checks if the repository has uncommitted changes
    pub async fn has_changes(&self) -> Result<bool> {
        let status = self.status().await?;

        Ok(!status.staged_files.is_empty()
            || !status.staged_dirs.is_empty()
            || !status.untracked_files.is_empty()
            || !status.untracked_dirs.is_empty()
            || !status.modified_files.is_empty())
    }

    /// Get the draft manager for this repository
    pub fn draft_manager(&self) -> Result<DraftManager> {
        DraftManager::new(&self.path)
    }

    /// Ensure repository is on draft branch
    pub async fn ensure_on_draft_branch(&self) -> Result<()> {
        let draft = self.draft_manager()?;

        if !draft.is_on_draft_branch()? {
            draft.switch_to_draft().await?;
        }

        Ok(())
    }

    /// Create an auto-commit on the draft branch
    ///
    /// This is the primary method for daemon auto-commits
    pub async fn auto_commit(&self, metadata: CommitMetadata) -> Result<String> {
        let draft = self.draft_manager()?;

        // Stage all changes first
        self.stage_all().await?;

        // Create auto-commit on draft branch
        draft.auto_commit(metadata).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_ignore() {
        let content = generate_oxenignore();
        assert!(content.contains("Bounces/"));
    }
}
