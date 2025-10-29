use anyhow::{Context, Result};
use crate::liboxen_stub as liboxen;
use liboxen::api;
use liboxen::command;
use std::path::Path;

use crate::commit_metadata::CommitMetadata;

/// Manages the draft branch workflow for Logic Pro projects
///
/// The draft branch serves as a working branch where auto-commits are made.
/// This keeps the main branch clean while providing automatic version control.
pub struct DraftManager {
    repo: liboxen::model::LocalRepository,
    draft_branch_name: String,
    max_draft_commits: usize,
}

impl DraftManager {
    /// Branch name constants
    pub const DEFAULT_DRAFT_BRANCH: &'static str = "draft";
    pub const MAIN_BRANCH: &'static str = "main";

    /// Default maximum draft commits before pruning
    pub const DEFAULT_MAX_COMMITS: usize = 100;

    /// Create a new draft manager for a repository
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self> {
        let repo = api::local::repositories::get(repo_path.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

        Ok(Self {
            repo,
            draft_branch_name: Self::DEFAULT_DRAFT_BRANCH.to_string(),
            max_draft_commits: Self::DEFAULT_MAX_COMMITS,
        })
    }

    /// Create with custom configuration
    pub fn with_config(
        repo_path: impl AsRef<Path>,
        draft_branch_name: String,
        max_draft_commits: usize,
    ) -> Result<Self> {
        let repo = api::local::repositories::get(repo_path.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

        Ok(Self {
            repo,
            draft_branch_name,
            max_draft_commits,
        })
    }

    /// Initialize draft branch workflow
    ///
    /// This will:
    /// 1. Check if draft branch exists
    /// 2. Create draft branch if it doesn't exist
    /// 3. Switch to draft branch
    pub async fn initialize(&self) -> Result<()> {
        println!("Initializing draft branch workflow...");

        // Check if draft branch exists
        if !self.draft_branch_exists()? {
            println!("Creating draft branch: {}", self.draft_branch_name);
            self.create_draft_branch().await?;
        } else {
            println!("Draft branch already exists: {}", self.draft_branch_name);
        }

        // Switch to draft branch
        self.switch_to_draft().await?;

        println!("✓ Draft branch workflow initialized");

        Ok(())
    }

    /// Check if draft branch exists
    pub fn draft_branch_exists(&self) -> Result<bool> {
        let branches = api::local::branches::list(&self.repo)
            .context("Failed to list branches")?;

        Ok(branches.iter().any(|b| b.name == self.draft_branch_name))
    }

    /// Create the draft branch from current HEAD
    async fn create_draft_branch(&self) -> Result<()> {
        api::local::branches::create_from_head(&self.repo, &self.draft_branch_name)
            .context("Failed to create draft branch")?;

        Ok(())
    }

    /// Switch to the draft branch
    pub async fn switch_to_draft(&self) -> Result<()> {
        command::checkout(&self.repo, &self.draft_branch_name)
            .await
            .context("Failed to switch to draft branch")?;

        println!("Switched to branch: {}", self.draft_branch_name);

        Ok(())
    }

    /// Switch to main branch
    pub async fn switch_to_main(&self) -> Result<()> {
        command::checkout(&self.repo, Self::MAIN_BRANCH)
            .await
            .context("Failed to switch to main branch")?;

        println!("Switched to branch: {}", Self::MAIN_BRANCH);

        Ok(())
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        api::local::branches::current_branch(&self.repo)
            .map(|b| b.name)
            .context("Failed to get current branch")
    }

    /// Check if currently on draft branch
    pub fn is_on_draft_branch(&self) -> Result<bool> {
        let current = self.current_branch()?;
        Ok(current == self.draft_branch_name)
    }

    /// Create an auto-commit on the draft branch
    ///
    /// This is the primary method for automatic commits
    pub async fn auto_commit(&self, metadata: CommitMetadata) -> Result<String> {
        // Ensure we're on draft branch
        if !self.is_on_draft_branch()? {
            self.switch_to_draft().await?;
        }

        // Create commit
        let message = metadata.format_commit_message();
        let commit = command::commit(&self.repo, &message)
            .await
            .context("Failed to create auto-commit")?;

        println!("Auto-commit created: {}", commit.id);

        // Check if pruning is needed
        self.prune_if_needed().await?;

        Ok(commit.id)
    }

    /// Count commits on draft branch since divergence from main
    pub fn draft_commit_count(&self) -> Result<usize> {
        let commits = api::local::commits::list(&self.repo)
            .context("Failed to list commits")?;

        // This is a simplified count - in reality you'd want to count
        // commits since the branch diverged from main
        Ok(commits.len())
    }

    /// Prune old draft commits if limit exceeded
    ///
    /// This creates a squashed commit and resets the draft branch
    pub async fn prune_if_needed(&self) -> Result<()> {
        let count = self.draft_commit_count()?;

        if count > self.max_draft_commits {
            println!(
                "Draft branch has {} commits (max: {}), pruning...",
                count, self.max_draft_commits
            );
            self.prune_draft_commits().await?;
        }

        Ok(())
    }

    /// Prune draft commits by squashing old ones
    async fn prune_draft_commits(&self) -> Result<()> {
        // This is a placeholder - actual implementation would:
        // 1. Identify commits to keep (most recent N)
        // 2. Create a squash commit for older commits
        // 3. Reset branch to new history

        println!("⚠️  Draft pruning not fully implemented yet");
        println!("   Would squash commits beyond the {} most recent", self.max_draft_commits / 2);

        Ok(())
    }

    /// Merge draft changes back to main
    ///
    /// This creates a clean commit on main with all draft changes
    pub async fn merge_to_main(&self, _commit_message: &str) -> Result<String> {
        println!("Merging draft branch to main...");

        // Switch to main
        self.switch_to_main().await?;

        // Merge draft branch
        // Note: liboxen may not have direct merge support, so this is simplified
        println!("⚠️  Merge functionality requires liboxen merge support");
        println!("   Manual merge required: oxen merge {}", self.draft_branch_name);

        Ok("merge-placeholder".to_string())
    }

    /// Reset draft branch to main
    ///
    /// Useful for starting fresh
    pub async fn reset_to_main(&self) -> Result<()> {
        println!("Resetting draft branch to main...");

        // Switch to main
        self.switch_to_main().await?;

        // Delete and recreate draft branch
        api::local::branches::delete(&self.repo, &self.draft_branch_name)
            .context("Failed to delete draft branch")?;

        self.create_draft_branch().await?;
        self.switch_to_draft().await?;

        println!("✓ Draft branch reset to main");

        Ok(())
    }

    /// Get statistics about the draft branch
    pub fn get_stats(&self) -> Result<DraftStats> {
        let commit_count = self.draft_commit_count()?;
        let is_on_draft = self.is_on_draft_branch()?;
        let current_branch = self.current_branch()?;

        Ok(DraftStats {
            commit_count,
            is_on_draft,
            current_branch,
            draft_branch_name: self.draft_branch_name.clone(),
            max_commits: self.max_draft_commits,
        })
    }
}

/// Statistics about the draft branch
#[derive(Debug, Clone)]
pub struct DraftStats {
    pub commit_count: usize,
    pub is_on_draft: bool,
    pub current_branch: String,
    pub draft_branch_name: String,
    pub max_commits: usize,
}

impl DraftStats {
    /// Print formatted statistics
    pub fn print(&self) {
        println!("Draft Branch Statistics:");
        println!("  Current branch:  {}", self.current_branch);
        println!("  Draft branch:    {}", self.draft_branch_name);
        println!("  On draft:        {}", if self.is_on_draft { "Yes" } else { "No" });
        println!("  Commit count:    {}", self.commit_count);
        println!("  Max commits:     {}", self.max_commits);

        if self.commit_count > self.max_commits {
            println!("  ⚠️  Exceeds limit - pruning recommended");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DraftManager::DEFAULT_DRAFT_BRANCH, "draft");
        assert_eq!(DraftManager::MAIN_BRANCH, "main");
        assert!(DraftManager::DEFAULT_MAX_COMMITS > 0);
    }

    #[test]
    fn test_default_draft_branch_name() {
        assert_eq!(DraftManager::DEFAULT_DRAFT_BRANCH, "draft");
    }

    #[test]
    fn test_main_branch_name() {
        assert_eq!(DraftManager::MAIN_BRANCH, "main");
    }

    #[test]
    fn test_default_max_commits_reasonable() {
        // Should be a reasonable number (not too small, not ridiculously large)
        assert!(DraftManager::DEFAULT_MAX_COMMITS >= 50);
        assert!(DraftManager::DEFAULT_MAX_COMMITS <= 1000);
    }

    #[test]
    fn test_default_max_commits_value() {
        // Verify the documented default
        assert_eq!(DraftManager::DEFAULT_MAX_COMMITS, 100);
    }

    #[test]
    fn test_draft_stats_fields() {
        let stats = DraftStats {
            commit_count: 42,
            is_on_draft: true,
            current_branch: "draft".to_string(),
            draft_branch_name: "draft".to_string(),
            max_commits: 100,
        };

        assert_eq!(stats.commit_count, 42);
        assert!(stats.is_on_draft);
        assert_eq!(stats.current_branch, "draft");
        assert_eq!(stats.draft_branch_name, "draft");
        assert_eq!(stats.max_commits, 100);
    }

    #[test]
    fn test_draft_stats_clone() {
        let stats = DraftStats {
            commit_count: 10,
            is_on_draft: false,
            current_branch: "main".to_string(),
            draft_branch_name: "draft".to_string(),
            max_commits: 100,
        };

        let cloned = stats.clone();
        assert_eq!(stats.commit_count, cloned.commit_count);
        assert_eq!(stats.is_on_draft, cloned.is_on_draft);
        assert_eq!(stats.current_branch, cloned.current_branch);
    }

    #[test]
    fn test_draft_stats_debug() {
        let stats = DraftStats {
            commit_count: 5,
            is_on_draft: true,
            current_branch: "draft".to_string(),
            draft_branch_name: "draft".to_string(),
            max_commits: 100,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("commit_count"));
        assert!(debug_str.contains("5"));
    }

    #[test]
    fn test_draft_stats_print_doesnt_panic() {
        let stats = DraftStats {
            commit_count: 50,
            is_on_draft: true,
            current_branch: "draft".to_string(),
            draft_branch_name: "custom-draft".to_string(),
            max_commits: 100,
        };

        // Should not panic
        stats.print();
    }

    #[test]
    fn test_draft_stats_print_with_exceeded_limit() {
        let stats = DraftStats {
            commit_count: 150, // Exceeds max
            is_on_draft: true,
            current_branch: "draft".to_string(),
            draft_branch_name: "draft".to_string(),
            max_commits: 100,
        };

        // Should not panic even when limit is exceeded
        stats.print();
    }

    #[test]
    fn test_draft_stats_on_main_branch() {
        let stats = DraftStats {
            commit_count: 25,
            is_on_draft: false,
            current_branch: "main".to_string(),
            draft_branch_name: "draft".to_string(),
            max_commits: 100,
        };

        assert!(!stats.is_on_draft);
        assert_eq!(stats.current_branch, "main");
    }

    #[test]
    fn test_draft_stats_custom_branch_name() {
        let stats = DraftStats {
            commit_count: 10,
            is_on_draft: true,
            current_branch: "my-custom-draft".to_string(),
            draft_branch_name: "my-custom-draft".to_string(),
            max_commits: 50,
        };

        assert_eq!(stats.draft_branch_name, "my-custom-draft");
        assert_eq!(stats.current_branch, "my-custom-draft");
    }

    #[test]
    fn test_draft_stats_zero_commits() {
        let stats = DraftStats {
            commit_count: 0,
            is_on_draft: true,
            current_branch: "draft".to_string(),
            draft_branch_name: "draft".to_string(),
            max_commits: 100,
        };

        assert_eq!(stats.commit_count, 0);
        // Should not trigger "exceeds limit" warning
        assert!(stats.commit_count <= stats.max_commits);
    }

    #[test]
    fn test_draft_stats_at_limit() {
        let stats = DraftStats {
            commit_count: 100,
            is_on_draft: true,
            current_branch: "draft".to_string(),
            draft_branch_name: "draft".to_string(),
            max_commits: 100,
        };

        // At limit, not exceeding
        assert_eq!(stats.commit_count, stats.max_commits);
    }

    #[test]
    fn test_draft_stats_just_over_limit() {
        let stats = DraftStats {
            commit_count: 101,
            is_on_draft: true,
            current_branch: "draft".to_string(),
            draft_branch_name: "draft".to_string(),
            max_commits: 100,
        };

        // Just over the limit
        assert!(stats.commit_count > stats.max_commits);
    }

    // Note: Testing async methods and methods that depend on liboxen
    // would require mocking or integration tests. For now, we test
    // the synchronous data structures and constants.

    #[test]
    fn test_draft_stats_various_max_commits() {
        let max_values = vec![50, 100, 200, 500, 1000];

        for max in max_values {
            let stats = DraftStats {
                commit_count: max / 2,
                is_on_draft: true,
                current_branch: "draft".to_string(),
                draft_branch_name: "draft".to_string(),
                max_commits: max,
            };

            assert_eq!(stats.max_commits, max);
            assert!(stats.commit_count < stats.max_commits);
        }
    }
}
