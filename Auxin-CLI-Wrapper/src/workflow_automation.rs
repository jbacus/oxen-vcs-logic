use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration as StdDuration;

use crate::operation_history::{
    HistoryOperation, OperationHistoryEntry, OperationHistoryManager, OperationResult,
};
use crate::remote_lock::RemoteLockManager;

/// Configuration for automated workflows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowConfig {
    /// Enable automatic lock renewal
    pub auto_renew_locks: bool,

    /// How often to check for lock renewal (minutes)
    pub lock_check_interval_minutes: u64,

    /// Renew lock when this much time remains (minutes)
    pub lock_renew_threshold_minutes: u64,

    /// Enable automatic pull on startup
    pub auto_pull_on_startup: bool,

    /// Enable automatic push after commit
    pub auto_push_after_commit: bool,

    /// Enable confirmation prompts for destructive operations
    pub confirm_destructive_operations: bool,

    /// Enable dry-run mode (preview without executing)
    pub dry_run_mode: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            auto_renew_locks: true,
            lock_check_interval_minutes: 15,
            lock_renew_threshold_minutes: 60,
            auto_pull_on_startup: false,
            auto_push_after_commit: false,
            confirm_destructive_operations: true,
            dry_run_mode: false,
        }
    }
}

impl WorkflowConfig {
    /// Load configuration from file
    pub fn load(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(config_path).context("Failed to read workflow config")?;

        let config: WorkflowConfig =
            serde_json::from_str(&contents).context("Failed to parse workflow config")?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, config_path: &Path) -> Result<()> {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, json).context("Failed to write workflow config")?;

        Ok(())
    }

    /// Get default config path
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".auxin")
            .join("workflow_config.json")
    }
}

/// Manages automated workflows
pub struct WorkflowAutomation {
    config: WorkflowConfig,
    lock_manager: RemoteLockManager,
    history_manager: OperationHistoryManager,
}

impl WorkflowAutomation {
    pub fn new() -> Self {
        let config = WorkflowConfig::load(&WorkflowConfig::default_path()).unwrap_or_default();

        Self {
            config,
            lock_manager: RemoteLockManager::new(),
            history_manager: OperationHistoryManager::new(),
        }
    }

    pub fn with_config(config: WorkflowConfig) -> Self {
        Self {
            config,
            lock_manager: RemoteLockManager::new(),
            history_manager: OperationHistoryManager::new(),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: WorkflowConfig) -> Result<()> {
        config.save(&WorkflowConfig::default_path())?;
        self.config = config;
        Ok(())
    }

    /// Check if lock needs renewal and renew if necessary
    pub fn check_and_renew_lock(&self, repo_path: &Path) -> Result<bool> {
        if !self.config.auto_renew_locks {
            return Ok(false);
        }

        let lock = match self.lock_manager.get_lock(repo_path)? {
            Some(lock) => lock,
            None => return Ok(false), // No lock to renew
        };

        // Only renew if we own the lock
        if !lock.is_owned_by_current_user() {
            return Ok(false);
        }

        // Check if lock is close to expiring
        let now = Utc::now();
        let time_until_expiry = lock.expires_at.signed_duration_since(now);
        let threshold = Duration::minutes(self.config.lock_renew_threshold_minutes as i64);

        if time_until_expiry < threshold {
            crate::vlog!(
                "Lock expiring in {} minutes, renewing...",
                time_until_expiry.num_minutes()
            );

            if !self.config.dry_run_mode {
                let lock_id = lock.lock_id.clone();
                self.lock_manager.renew_lock(repo_path, &lock_id, 4)?;

                // Record renewal in history
                let entry = OperationHistoryEntry::new(HistoryOperation::LockRenew)
                    .with_repo_path(repo_path)
                    .with_result(OperationResult::Success)
                    .with_metadata("auto_renewed", "true")
                    .with_metadata("lock_id", &lock_id);

                let _ = self.history_manager.record(entry);
            } else {
                crate::vlog!("DRY RUN: Would renew lock");
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Run lock renewal daemon (blocks indefinitely)
    pub fn run_lock_renewal_daemon(&self, repo_path: &Path) -> Result<()> {
        crate::info!(
            "Starting lock renewal daemon (checking every {} minutes)",
            self.config.lock_check_interval_minutes
        );

        loop {
            match self.check_and_renew_lock(repo_path) {
                Ok(renewed) => {
                    if renewed {
                        crate::info!("Lock renewed successfully");
                    }
                }
                Err(e) => {
                    crate::error!("Failed to renew lock: {}", e);
                }
            }

            thread::sleep(StdDuration::from_secs(
                self.config.lock_check_interval_minutes * 60,
            ));
        }
    }

    /// Confirm a destructive operation with the user
    pub fn confirm_destructive_operation(&self, operation_name: &str) -> Result<bool> {
        if !self.config.confirm_destructive_operations || self.config.dry_run_mode {
            if self.config.dry_run_mode {
                println!(
                    "{}",
                    format!("DRY RUN: Would execute {}", operation_name).yellow()
                );
                return Ok(false);
            }
            return Ok(true);
        }

        use dialoguer::Confirm;

        let confirmed = Confirm::new()
            .with_prompt(format!("Are you sure you want to {}?", operation_name))
            .default(false)
            .interact()?;

        Ok(confirmed)
    }

    /// Suggest next action based on repository state
    pub fn suggest_next_action(&self, repo_path: &Path) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();

        // Check lock status
        match self.lock_manager.get_lock(repo_path) {
            Ok(Some(lock)) => {
                if lock.is_owned_by_current_user() {
                    if lock.is_expiring_soon(60) {
                        suggestions.push(format!(
                            "{} Lock expires in {} minutes - consider running 'auxin lock renew'",
                            "⚠".yellow(),
                            lock.minutes_until_expiry()
                        ));
                    }
                } else {
                    suggestions.push(format!(
                        "{} Project is locked by {} - cannot edit until lock is released",
                        "🔒".red(),
                        lock.locked_by
                    ));
                }
            }
            Ok(None) => {
                suggestions.push(format!(
                    "{} No lock held - run 'auxin lock acquire' before editing",
                    "💡",
                ));
            }
            Err(_) => {}
        }

        // Check recent operation history
        if let Ok(recent) = self.history_manager.get_recent(5) {
            let failed_ops: Vec<_> = recent.iter().filter(|e| e.is_failure()).collect();

            if !failed_ops.is_empty() {
                suggestions.push(format!(
                    "{} {} recent operations failed - check history with 'auxin history'",
                    "⚠".yellow(),
                    failed_ops.len()
                ));
            }
        }

        Ok(suggestions)
    }

    /// Display workflow suggestions to user
    pub fn display_suggestions(&self, repo_path: &Path) -> Result<()> {
        let suggestions = self.suggest_next_action(repo_path)?;

        if suggestions.is_empty() {
            println!("{}", "✓ All good! No suggestions at this time.".green());
            return Ok(());
        }

        println!(
            "\n{}",
            "┌─ Suggestions ───────────────────────────────────────────┐".bright_blue()
        );
        for suggestion in suggestions {
            println!("│ {}", suggestion);
        }
        println!(
            "{}\n",
            "└──────────────────────────────────────────────────────────┘".bright_blue()
        );

        Ok(())
    }

    /// Execute pre-commit checks
    pub fn pre_commit_checks(&self, repo_path: &Path) -> Result<bool> {
        crate::vlog!("Running pre-commit checks...");

        // Check if we have the lock
        match self.lock_manager.get_lock(repo_path)? {
            Some(lock) => {
                if !lock.is_owned_by_current_user() {
                    crate::error!("Cannot commit: project is locked by {}", lock.locked_by);
                    return Ok(false);
                }
            }
            None => {
                crate::warn!("No lock held - committing anyway but consider acquiring lock");
            }
        }

        crate::vlog!("Pre-commit checks passed");
        Ok(true)
    }

    /// Execute post-commit actions
    pub fn post_commit_actions(&self, _repo_path: &Path, commit_id: &str) -> Result<()> {
        crate::vlog!("Running post-commit actions...");

        // Record commit in history
        let entry = OperationHistoryEntry::new(HistoryOperation::Commit)
            .with_repo_path(_repo_path)
            .with_result(OperationResult::Success)
            .with_metadata("commit_id", commit_id);

        self.history_manager.record(entry)?;

        if self.config.auto_push_after_commit {
            crate::info!("Auto-push enabled - push your changes with 'oxen push origin main'");
            // Note: Actual push would require oxen subprocess integration
        }

        Ok(())
    }
}

impl Default for WorkflowAutomation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workflow_config_default() {
        let config = WorkflowConfig::default();
        assert!(config.auto_renew_locks);
        assert!(config.confirm_destructive_operations);
        assert!(!config.dry_run_mode);
    }

    #[test]
    fn test_workflow_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("workflow.json");

        let mut config = WorkflowConfig::default();
        config.auto_renew_locks = false;
        config.dry_run_mode = true;

        config.save(&config_file).unwrap();

        let loaded = WorkflowConfig::load(&config_file).unwrap();
        assert_eq!(loaded, config);
        assert!(!loaded.auto_renew_locks);
        assert!(loaded.dry_run_mode);
    }

    #[test]
    fn test_workflow_automation_creation() {
        // Use with_config to test default values (new() loads from file which may differ)
        let automation = WorkflowAutomation::with_config(WorkflowConfig::default());
        assert!(automation.config().auto_renew_locks);
    }

    #[test]
    fn test_workflow_automation_with_config() {
        let mut config = WorkflowConfig::default();
        config.lock_check_interval_minutes = 30;

        let automation = WorkflowAutomation::with_config(config);
        assert_eq!(automation.config().lock_check_interval_minutes, 30);
    }

    #[test]
    fn test_update_config() {
        let mut automation = WorkflowAutomation::new();
        let mut new_config = WorkflowConfig::default();
        new_config.auto_renew_locks = false;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workflow.json");
        new_config.save(&config_path).unwrap();

        automation.update_config(new_config.clone()).unwrap();
        assert!(!automation.config().auto_renew_locks);
    }

    #[test]
    fn test_dry_run_mode() {
        let mut config = WorkflowConfig::default();
        config.dry_run_mode = true;
        config.confirm_destructive_operations = true;

        let automation = WorkflowAutomation::with_config(config);

        // Dry run should not require confirmation
        let confirmed = automation
            .confirm_destructive_operation("test operation")
            .unwrap();
        assert!(!confirmed);
    }

    #[test]
    fn test_pre_commit_checks() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create minimal repo structure
        fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        let automation = WorkflowAutomation::new();

        // Should pass (with warning) when no lock exists
        let result = automation.pre_commit_checks(repo_path).unwrap();
        assert!(result);
    }

    #[test]
    fn test_post_commit_actions() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let automation = WorkflowAutomation::new();

        let result = automation.post_commit_actions(repo_path, "abc123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_suggest_next_action() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create minimal repo structure
        fs::create_dir_all(repo_path.join(".oxen")).unwrap();

        let automation = WorkflowAutomation::new();

        let suggestions = automation.suggest_next_action(repo_path).unwrap();
        // Should suggest acquiring lock when no lock exists
        assert!(!suggestions.is_empty());
    }
}
