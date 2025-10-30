use crate::liboxen_stub as liboxen;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use liboxen::api;
use liboxen::command;
use liboxen::opts::AddOpts;
use std::path::{Path, PathBuf};

use crate::commit_metadata::CommitMetadata;
use crate::draft_manager::DraftManager;
use crate::ignore_template::generate_oxenignore;
use crate::logic_project::LogicProject;
use crate::{info, vlog};

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
        let logic_project =
            LogicProject::detect(path).context("Failed to detect Logic Pro project")?;

        info!("Detected Logic Pro project: {}", logic_project.name());
        vlog!("Project name: {}", logic_project.name());

        // Initialize Oxen repository
        vlog!("Step 2: Initializing Oxen repository...");
        let _repo =
            api::local::repositories::init(path).context("Failed to initialize Oxen repository")?;

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

        let draft_manager = DraftManager::new(path).context("Failed to create draft manager")?;

        draft_manager
            .initialize()
            .await
            .context("Failed to initialize draft branch")?;

        vlog!("Draft branch initialized successfully");
        vlog!("=== Initialization Complete ===");

        Ok(repo_instance)
    }

    /// Initializes a new Oxen repository (generic)
    pub async fn init(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let _repo =
            api::local::repositories::init(path).context("Failed to initialize Oxen repository")?;

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

        let mut commits =
            api::local::commits::list(&repo).context("Failed to get commit history")?;

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
    use std::fs;

    // Constructor tests

    #[test]
    fn test_new_with_relative_path() {
        let repo = OxenRepository::new("test/path");
        assert!(repo.path.to_string_lossy().contains("test"));
        assert!(repo.path.to_string_lossy().contains("path"));
    }

    #[test]
    fn test_new_with_absolute_path() {
        let repo = OxenRepository::new("/absolute/test/path");
        assert_eq!(repo.path, PathBuf::from("/absolute/test/path"));
    }

    #[test]
    fn test_new_with_pathbuf() {
        let path = PathBuf::from("/some/path");
        let repo = OxenRepository::new(path.clone());
        assert_eq!(repo.path, path);
    }

    #[test]
    fn test_new_with_str_slice() {
        let repo = OxenRepository::new("test");
        assert_eq!(repo.path, PathBuf::from("test"));
    }

    #[test]
    fn test_new_with_path_reference() {
        let path = Path::new("/test/reference");
        let repo = OxenRepository::new(path);
        assert_eq!(repo.path, PathBuf::from("/test/reference"));
    }

    #[test]
    fn test_new_empty_path() {
        let repo = OxenRepository::new("");
        assert_eq!(repo.path, PathBuf::from(""));
    }

    #[test]
    fn test_new_with_special_characters() {
        let repo = OxenRepository::new("/path/with spaces/and-dashes");
        assert_eq!(repo.path, PathBuf::from("/path/with spaces/and-dashes"));
    }

    #[test]
    fn test_new_with_unicode() {
        let repo = OxenRepository::new("/path/with/日本語");
        assert!(repo.path.to_string_lossy().contains("日本語"));
    }

    // Path handling tests

    #[test]
    fn test_multiple_repos_different_paths() {
        let repo1 = OxenRepository::new("/path1");
        let repo2 = OxenRepository::new("/path2");
        assert_ne!(repo1.path, repo2.path);
    }

    #[test]
    fn test_path_normalization() {
        let repo = OxenRepository::new("./test/../test");
        // PathBuf stores the path as-is, doesn't automatically normalize
        assert!(repo.path.to_string_lossy().contains("test"));
    }

    // Draft manager wrapper test

    #[test]
    fn test_draft_manager_returns_result() {
        let temp_dir = std::env::temp_dir().join("oxen_ops_test_draft");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let repo = OxenRepository::new(&temp_dir);
        let result = repo.draft_manager();

        // Should return Ok since DraftManager::new doesn't fail
        assert!(result.is_ok());

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_draft_manager_uses_repo_path() {
        let temp_dir = std::env::temp_dir().join("oxen_ops_test_draft2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let repo = OxenRepository::new(&temp_dir);
        let _draft = repo.draft_manager().unwrap();

        // Verify the draft manager can be created
        // (This tests the integration between OxenRepository and DraftManager)
        // Note: DraftManager doesn't expose repo_path publicly

        fs::remove_dir_all(&temp_dir).ok();
    }

    // Integration with ignore_template

    #[tokio::test]
    async fn test_generate_ignore() {
        let content = generate_oxenignore();
        assert!(content.contains("Bounces/"));
    }

    #[test]
    fn test_generate_ignore_has_logic_patterns() {
        let content = generate_oxenignore();
        assert!(content.contains("Bounces/"));
        assert!(content.contains("Freeze Files/"));
        assert!(content.contains("Autosave/"));
        assert!(content.contains(".DS_Store"));
    }

    // Error path tests (testing with invalid paths)

    #[test]
    #[ignore = "Requires real Oxen implementation - stub always returns success"]
    fn test_get_repo_with_nonexistent_path() {
        let repo = OxenRepository::new("/nonexistent/path/that/does/not/exist");
        let result = repo.get_repo();

        // Should return an error since the repository doesn't exist
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "Requires real Oxen implementation - stub always returns success"]
    fn test_get_repo_error_message() {
        let path = "/nonexistent/repo";
        let repo = OxenRepository::new(path);
        let result = repo.get_repo();

        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Repository not found"));
        assert!(err_msg.contains(path));
    }

    // Struct field access tests

    #[test]
    fn test_path_field_accessible() {
        let test_path = PathBuf::from("/test/path");
        let repo = OxenRepository::new(&test_path);
        assert_eq!(repo.path, test_path);
    }

    #[test]
    fn test_path_field_immutable() {
        let repo = OxenRepository::new("/test");
        let path_copy = repo.path.clone();
        assert_eq!(repo.path, path_copy);
    }

    // Testing async function signatures (compilation tests)

    #[tokio::test]
    async fn test_init_signature() {
        // This test verifies the init function signature compiles correctly
        // We don't expect it to succeed with the stub, but it validates the API
        let temp_dir = std::env::temp_dir().join("oxen_ops_test_init");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let _result = OxenRepository::init(&temp_dir).await;
        // With stub implementation, this may succeed or fail
        // The important part is that the function signature is correct

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_stage_changes_signature() {
        let repo = OxenRepository::new("/test");
        let files = vec![PathBuf::from("test.txt")];

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.stage_changes(files).await;
    }

    #[tokio::test]
    async fn test_stage_all_signature() {
        let repo = OxenRepository::new("/test");

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.stage_all().await;
    }

    #[tokio::test]
    async fn test_create_commit_signature() {
        let repo = OxenRepository::new("/test");
        let metadata = CommitMetadata::new("Test commit");

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.create_commit(metadata).await;
    }

    #[tokio::test]
    async fn test_get_history_signature() {
        let repo = OxenRepository::new("/test");

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.get_history(None).await;
        let _ = repo.get_history(Some(10)).await;
    }

    #[tokio::test]
    async fn test_restore_signature() {
        let repo = OxenRepository::new("/test");

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.restore("abc123").await;
    }

    #[tokio::test]
    async fn test_status_signature() {
        let repo = OxenRepository::new("/test");

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.status().await;
    }

    #[tokio::test]
    async fn test_has_changes_signature() {
        let repo = OxenRepository::new("/test");

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.has_changes().await;
    }

    #[tokio::test]
    async fn test_ensure_on_draft_branch_signature() {
        let repo = OxenRepository::new("/test");

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.ensure_on_draft_branch().await;
    }

    #[tokio::test]
    async fn test_auto_commit_signature() {
        let repo = OxenRepository::new("/test");
        let metadata = CommitMetadata::new("Auto commit");

        // This should fail because repo doesn't exist, but verifies signature
        let _ = repo.auto_commit(metadata).await;
    }

    // CommitMetadata integration tests

    #[test]
    fn test_commit_metadata_builder_integration() {
        let metadata = CommitMetadata::new("Test message")
            .with_bpm(120.0)
            .with_sample_rate(48000)
            .with_key_signature("C major");

        assert_eq!(metadata.message, "Test message");
        assert_eq!(metadata.bpm, Some(120.0));
        assert_eq!(metadata.sample_rate, Some(48000));
        assert_eq!(metadata.key_signature, Some("C major".to_string()));
    }

    #[test]
    fn test_commit_metadata_format_integration() {
        let metadata = CommitMetadata::new("Integration test").with_bpm(140.0);

        let formatted = metadata.format_commit_message();
        assert!(formatted.contains("Integration test"));
        assert!(formatted.contains("BPM: 140"));
    }

    // LogicProject integration tests

    #[test]
    fn test_logic_project_detect_integration() {
        // Test that detect returns proper error for invalid path
        let result = LogicProject::detect("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_logic_project_detect_not_a_project() {
        let temp_dir = std::env::temp_dir().join("not_a_logic_project");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_err());

        fs::remove_dir_all(&temp_dir).ok();
    }

    // Additional edge case tests

    #[test]
    fn test_new_with_current_dir() {
        let repo = OxenRepository::new(".");
        assert_eq!(repo.path, PathBuf::from("."));
    }

    #[test]
    fn test_new_with_parent_dir() {
        let repo = OxenRepository::new("..");
        assert_eq!(repo.path, PathBuf::from(".."));
    }

    #[test]
    fn test_new_with_home_tilde() {
        // Note: PathBuf doesn't expand ~ automatically
        let repo = OxenRepository::new("~/test");
        assert_eq!(repo.path, PathBuf::from("~/test"));
    }

    #[test]
    fn test_new_with_trailing_slash() {
        let repo = OxenRepository::new("/test/path/");
        assert!(repo.path.to_string_lossy().contains("test"));
    }

    #[test]
    fn test_new_with_multiple_slashes() {
        let repo = OxenRepository::new("/test//path///here");
        assert!(repo.path.to_string_lossy().contains("test"));
    }

    // Clone and Debug trait tests (if we add them in the future)

    #[test]
    fn test_repository_paths_can_be_compared() {
        let repo1 = OxenRepository::new("/same/path");
        let repo2 = OxenRepository::new("/same/path");
        assert_eq!(repo1.path, repo2.path);
    }

    #[test]
    fn test_repository_paths_inequality() {
        let repo1 = OxenRepository::new("/path1");
        let repo2 = OxenRepository::new("/path2");
        assert_ne!(repo1.path, repo2.path);
    }
}
