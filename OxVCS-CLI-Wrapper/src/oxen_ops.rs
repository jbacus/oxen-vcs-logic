use anyhow::{anyhow, Context, Result};
use liboxen::api;
use liboxen::command;
use liboxen::opts::AddOpts;
use std::path::{Path, PathBuf};

use crate::commit_metadata::CommitMetadata;
use crate::ignore_template::generate_oxenignore;
use crate::logic_project::LogicProject;

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

        // Detect Logic Pro project
        let logic_project = LogicProject::detect(path)
            .context("Failed to detect Logic Pro project")?;

        println!("Detected Logic Pro project: {}", logic_project.name());

        // Initialize Oxen repository
        let repo = api::local::repositories::init(path)
            .context("Failed to initialize Oxen repository")?;

        println!("Initialized Oxen repository at: {}", path.display());

        // Create .oxenignore file
        let ignore_path = path.join(".oxenignore");
        let ignore_content = generate_oxenignore();

        tokio::fs::write(&ignore_path, ignore_content)
            .await
            .context("Failed to write .oxenignore file")?;

        println!("Created .oxenignore file");

        Ok(Self {
            path: path.to_path_buf(),
        })
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
