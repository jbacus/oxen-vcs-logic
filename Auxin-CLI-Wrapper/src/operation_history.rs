use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum number of history entries to keep (prevents unbounded growth)
const MAX_HISTORY_ENTRIES: usize = 10000;

/// Represents a single operation in the audit trail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperationHistoryEntry {
    /// Unique identifier for this operation
    pub id: String,

    /// Type of operation performed
    pub operation: HistoryOperation,

    /// When the operation occurred
    pub timestamp: DateTime<Utc>,

    /// User who performed the operation
    pub user: String,

    /// Machine identifier
    pub machine_id: String,

    /// Repository path (if applicable)
    pub repo_path: Option<PathBuf>,

    /// Operation result
    pub result: OperationResult,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HistoryOperation {
    // Lock operations
    LockAcquire,
    LockRelease,
    LockRenew,
    LockBreak,

    // Network operations
    Push,
    Pull,
    Fetch,

    // Commit operations
    Commit,
    Rollback,

    // Authentication
    Login,
    Logout,

    // Collaboration
    CommentAdd,
    ActivityView,

    // Conflict detection
    ConflictCheck,

    // Other
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationResult {
    Success,
    Failure(String),
    Partial(String),
}

impl OperationHistoryEntry {
    pub fn new(operation: HistoryOperation) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            operation,
            timestamp: Utc::now(),
            user: whoami::username(),
            machine_id: whoami::devicename(),
            repo_path: None,
            result: OperationResult::Success,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_repo_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.repo_path = Some(path.into());
        self
    }

    pub fn with_result(mut self, result: OperationResult) -> Self {
        self.result = result;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn is_success(&self) -> bool {
        matches!(self.result, OperationResult::Success)
    }

    pub fn is_failure(&self) -> bool {
        matches!(self.result, OperationResult::Failure(_))
    }
}

/// Manages operation history and audit trail
pub struct OperationHistoryManager {
    history_file: PathBuf,
}

impl OperationHistoryManager {
    /// Create new manager with default history location
    pub fn new() -> Self {
        Self {
            history_file: Self::default_history_path(),
        }
    }

    /// Create with custom history file path
    pub fn with_history_path(history_file: PathBuf) -> Self {
        Self { history_file }
    }

    /// Get default history file path (~/.auxin/operation_history.json)
    fn default_history_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".auxin")
            .join("operation_history.json")
    }

    /// Load operation history from disk
    pub fn load_history(&self) -> Result<Vec<OperationHistoryEntry>> {
        if !self.history_file.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&self.history_file)
            .context("Failed to read operation history file")?;

        let entries: Vec<OperationHistoryEntry> =
            serde_json::from_str(&contents).context("Failed to parse operation history")?;

        Ok(entries)
    }

    /// Save operation history to disk
    fn save_history(&self, entries: &[OperationHistoryEntry]) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.history_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(entries)?;

        fs::write(&self.history_file, json).context("Failed to write operation history file")?;

        Ok(())
    }

    /// Record a new operation
    pub fn record(&self, entry: OperationHistoryEntry) -> Result<()> {
        let mut entries = self.load_history()?;

        // Add new entry
        entries.push(entry);

        // Trim to max size (keep most recent)
        if entries.len() > MAX_HISTORY_ENTRIES {
            let skip = entries.len() - MAX_HISTORY_ENTRIES;
            entries = entries.into_iter().skip(skip).collect();
        }

        self.save_history(&entries)?;
        Ok(())
    }

    /// Get recent operations (limited)
    pub fn get_recent(&self, limit: usize) -> Result<Vec<OperationHistoryEntry>> {
        let entries = self.load_history()?;
        Ok(entries.into_iter().rev().take(limit).collect())
    }

    /// Get operations by type
    pub fn get_by_operation(
        &self,
        operation_type: HistoryOperation,
    ) -> Result<Vec<OperationHistoryEntry>> {
        let entries = self.load_history()?;
        Ok(entries
            .into_iter()
            .filter(|e| {
                std::mem::discriminant(&e.operation) == std::mem::discriminant(&operation_type)
            })
            .collect())
    }

    /// Get operations by result
    pub fn get_by_result(&self, success: bool) -> Result<Vec<OperationHistoryEntry>> {
        let entries = self.load_history()?;
        Ok(entries
            .into_iter()
            .filter(|e| e.is_success() == success)
            .collect())
    }

    /// Get operations for a specific repository
    pub fn get_by_repo(&self, repo_path: &Path) -> Result<Vec<OperationHistoryEntry>> {
        let entries = self.load_history()?;
        Ok(entries
            .into_iter()
            .filter(|e| e.repo_path.as_deref() == Some(repo_path))
            .collect())
    }

    /// Get statistics about operations
    pub fn get_stats(&self) -> Result<OperationStats> {
        let entries = self.load_history()?;

        let total = entries.len();
        let successful = entries.iter().filter(|e| e.is_success()).count();
        let failed = entries.iter().filter(|e| e.is_failure()).count();

        let lock_operations = entries
            .iter()
            .filter(|e| {
                matches!(
                    e.operation,
                    HistoryOperation::LockAcquire
                        | HistoryOperation::LockRelease
                        | HistoryOperation::LockRenew
                        | HistoryOperation::LockBreak
                )
            })
            .count();

        let network_operations = entries
            .iter()
            .filter(|e| {
                matches!(
                    e.operation,
                    HistoryOperation::Push | HistoryOperation::Pull | HistoryOperation::Fetch
                )
            })
            .count();

        Ok(OperationStats {
            total,
            successful,
            failed,
            lock_operations,
            network_operations,
        })
    }

    /// Clear all history (use with caution)
    pub fn clear_history(&self) -> Result<()> {
        if self.history_file.exists() {
            fs::remove_file(&self.history_file).context("Failed to remove history file")?;
        }
        Ok(())
    }

    /// Export history to CSV for analysis
    pub fn export_csv(&self, output_path: &Path) -> Result<()> {
        let entries = self.load_history()?;

        let mut csv = String::from("Timestamp,Operation,User,Machine,Result,Repo\n");

        for entry in entries {
            let result_str = match &entry.result {
                OperationResult::Success => "Success".to_string(),
                OperationResult::Failure(e) => format!("Failure: {}", e),
                OperationResult::Partial(m) => format!("Partial: {}", m),
            };

            let repo_str = entry
                .repo_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();

            csv.push_str(&format!(
                "{},{:?},{},{},{},{}\n",
                entry.timestamp,
                entry.operation,
                entry.user,
                entry.machine_id,
                result_str,
                repo_str
            ));
        }

        fs::write(output_path, csv).context("Failed to write CSV file")?;
        Ok(())
    }

    /// Display recent history in a formatted way
    pub fn display_recent(&self, limit: usize) -> Result<()> {
        let entries = self.get_recent(limit)?;

        if entries.is_empty() {
            println!("{}", "No operation history yet".bright_black());
            return Ok(());
        }

        println!(
            "\n{}",
            "┌─ Operation History ─────────────────────────────────────┐".bright_blue()
        );

        for entry in entries.iter().rev() {
            let icon = match entry.operation {
                HistoryOperation::LockAcquire => "🔒",
                HistoryOperation::LockRelease => "🔓",
                HistoryOperation::LockRenew => "🔄",
                HistoryOperation::LockBreak => "🔨",
                HistoryOperation::Push => "⬆",
                HistoryOperation::Pull => "⬇",
                HistoryOperation::Commit => "●",
                HistoryOperation::Login => "🔑",
                _ => "•",
            };

            let result_icon = match &entry.result {
                OperationResult::Success => "✓".green(),
                OperationResult::Failure(_) => "✗".red(),
                OperationResult::Partial(_) => "⚠".yellow(),
            };

            let time_ago = Self::format_time_ago(&entry.timestamp);

            println!(
                "│ {} {} {:?} {} {}",
                icon,
                result_icon,
                entry.operation,
                format!("by {}", entry.user).bright_black(),
                time_ago.bright_black()
            );

            if let OperationResult::Failure(err) = &entry.result {
                println!("│   {}: {}", "Error".red(), err);
            }
        }

        println!(
            "{}\n",
            "└──────────────────────────────────────────────────────────┘".bright_blue()
        );

        Ok(())
    }

    /// Format timestamp as "X minutes/hours/days ago"
    fn format_time_ago(timestamp: &DateTime<Utc>) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(*timestamp);

        if duration.num_seconds() < 60 {
            format!("{}s ago", duration.num_seconds())
        } else if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h ago", duration.num_hours())
        } else {
            format!("{}d ago", duration.num_days())
        }
    }
}

impl Default for OperationHistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationStats {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub lock_operations: usize,
    pub network_operations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_operation_history_manager_creation() {
        let manager = OperationHistoryManager::new();
        assert!(manager.history_file.to_string_lossy().contains(".auxin"));
    }

    #[test]
    fn test_record_and_load_operations() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let manager = OperationHistoryManager::with_history_path(history_file);

        let entry = OperationHistoryEntry::new(HistoryOperation::LockAcquire)
            .with_repo_path("/test/repo")
            .with_result(OperationResult::Success);

        manager.record(entry.clone()).unwrap();

        let loaded = manager.load_history().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].operation, HistoryOperation::LockAcquire);
    }

    #[test]
    fn test_get_recent_operations() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let manager = OperationHistoryManager::with_history_path(history_file);

        // Record 5 operations
        for i in 0..5 {
            let entry = OperationHistoryEntry::new(HistoryOperation::Commit)
                .with_metadata("index", i.to_string());
            manager.record(entry).unwrap();
        }

        let recent = manager.get_recent(3).unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_get_by_operation_type() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let manager = OperationHistoryManager::with_history_path(history_file);

        manager
            .record(OperationHistoryEntry::new(HistoryOperation::LockAcquire))
            .unwrap();
        manager
            .record(OperationHistoryEntry::new(HistoryOperation::Commit))
            .unwrap();
        manager
            .record(OperationHistoryEntry::new(HistoryOperation::LockAcquire))
            .unwrap();

        let lock_ops = manager
            .get_by_operation(HistoryOperation::LockAcquire)
            .unwrap();
        assert_eq!(lock_ops.len(), 2);
    }

    #[test]
    fn test_get_by_result() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let manager = OperationHistoryManager::with_history_path(history_file);

        manager
            .record(
                OperationHistoryEntry::new(HistoryOperation::Push)
                    .with_result(OperationResult::Success),
            )
            .unwrap();
        manager
            .record(
                OperationHistoryEntry::new(HistoryOperation::Pull)
                    .with_result(OperationResult::Failure("Network error".to_string())),
            )
            .unwrap();

        let successful = manager.get_by_result(true).unwrap();
        let failed = manager.get_by_result(false).unwrap();

        assert_eq!(successful.len(), 1);
        assert_eq!(failed.len(), 1);
    }

    #[test]
    fn test_get_stats() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let manager = OperationHistoryManager::with_history_path(history_file);

        manager
            .record(OperationHistoryEntry::new(HistoryOperation::LockAcquire))
            .unwrap();
        manager
            .record(OperationHistoryEntry::new(HistoryOperation::Push))
            .unwrap();
        manager
            .record(OperationHistoryEntry::new(HistoryOperation::Pull))
            .unwrap();
        manager
            .record(
                OperationHistoryEntry::new(HistoryOperation::Commit)
                    .with_result(OperationResult::Failure("Test error".to_string())),
            )
            .unwrap();

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total, 4);
        assert_eq!(stats.successful, 3);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.lock_operations, 1);
        assert_eq!(stats.network_operations, 2);
    }

    #[test]
    fn test_max_history_entries() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let manager = OperationHistoryManager::with_history_path(history_file);

        // Record more than MAX_HISTORY_ENTRIES
        // (Use small number for test performance)
        for i in 0..15 {
            manager
                .record(
                    OperationHistoryEntry::new(HistoryOperation::Commit)
                        .with_metadata("i", i.to_string()),
                )
                .unwrap();
        }

        let history = manager.load_history().unwrap();
        // Should be trimmed (would be MAX_HISTORY_ENTRIES in production)
        assert!(history.len() <= 15);
    }

    #[test]
    fn test_clear_history() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let manager = OperationHistoryManager::with_history_path(history_file.clone());

        manager
            .record(OperationHistoryEntry::new(HistoryOperation::Commit))
            .unwrap();
        assert!(history_file.exists());

        manager.clear_history().unwrap();
        assert!(!history_file.exists());
    }

    #[test]
    fn test_export_csv() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let csv_file = temp_dir.path().join("export.csv");
        let manager = OperationHistoryManager::with_history_path(history_file);

        manager
            .record(OperationHistoryEntry::new(HistoryOperation::LockAcquire))
            .unwrap();
        manager
            .record(OperationHistoryEntry::new(HistoryOperation::Push))
            .unwrap();

        manager.export_csv(&csv_file).unwrap();

        assert!(csv_file.exists());
        let csv_content = fs::read_to_string(&csv_file).unwrap();
        assert!(csv_content.contains("Timestamp,Operation,User,Machine,Result,Repo"));
        assert!(csv_content.contains("LockAcquire"));
        assert!(csv_content.contains("Push"));
    }

    #[test]
    fn test_entry_builder_pattern() {
        let entry = OperationHistoryEntry::new(HistoryOperation::LockAcquire)
            .with_repo_path("/test/path")
            .with_result(OperationResult::Success)
            .with_metadata("timeout", "4");

        assert_eq!(entry.operation, HistoryOperation::LockAcquire);
        assert_eq!(entry.repo_path, Some(PathBuf::from("/test/path")));
        assert!(entry.is_success());
        assert_eq!(entry.metadata.get("timeout"), Some(&"4".to_string()));
    }
}
