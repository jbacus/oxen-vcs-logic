/// Write-Ahead Logging (WAL) for crash recovery
///
/// This module implements a transaction log that records intent BEFORE
/// executing critical operations, enabling recovery of interrupted operations.
///
/// # Architecture
///
/// 1. **Pre-operation**: Log intent with operation details
/// 2. **Execute**: Perform the actual operation
/// 3. **Post-operation**: Mark as completed or failed
/// 4. **Recovery**: On startup, check for incomplete operations and replay/cleanup
///
/// # Example
///
/// ```no_run
/// use auxin_cli::write_ahead_log::{WriteAheadLog, WalOperation};
/// use std::path::Path;
///
/// let wal = WriteAheadLog::new();
/// let repo = Path::new("/path/to/repo");
///
/// // Log intent before operation
/// let entry_id = wal.log_intent(WalOperation::Commit {
///     repo_path: repo.to_path_buf(),
///     message: "My commit".to_string(),
/// })?;
///
/// // Execute operation
/// let result = perform_commit(repo, "My commit");
///
/// // Mark completion
/// match result {
///     Ok(_) => wal.mark_completed(&entry_id)?,
///     Err(e) => wal.mark_failed(&entry_id, &e.to_string())?,
/// }
/// ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Maximum age of WAL entries before cleanup (24 hours)
const WAL_ENTRY_MAX_AGE_HOURS: i64 = 24;

/// WAL operation types that need crash protection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalOperation {
    /// Commit operation
    Commit {
        repo_path: PathBuf,
        message: String,
    },
    /// Push operation
    Push {
        repo_path: PathBuf,
        remote: String,
        branch: String,
    },
    /// Lock acquire operation
    LockAcquire {
        repo_path: PathBuf,
        user_id: String,
        timeout_hours: u32,
    },
    /// Lock release operation
    LockRelease {
        repo_path: PathBuf,
        lock_id: String,
    },
    /// File staging operation
    StageFiles {
        repo_path: PathBuf,
        files: Vec<PathBuf>,
    },
}

/// Status of a WAL entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalStatus {
    /// Operation intent logged, not yet started
    Pending,
    /// Operation in progress
    InProgress,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed(String),
    /// Operation was recovered/replayed
    Recovered,
}

/// A single WAL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    /// Unique entry identifier
    pub id: String,
    /// Operation details
    pub operation: WalOperation,
    /// Current status
    pub status: WalStatus,
    /// When the entry was created
    pub created_at: DateTime<Utc>,
    /// When the entry was last updated
    pub updated_at: DateTime<Utc>,
    /// User who initiated the operation
    pub user: String,
    /// Machine identifier
    pub machine_id: String,
    /// Number of recovery attempts
    pub recovery_attempts: u32,
}

impl WalEntry {
    /// Create a new WAL entry
    pub fn new(operation: WalOperation) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            operation,
            status: WalStatus::Pending,
            created_at: now,
            updated_at: now,
            user: whoami::username(),
            machine_id: whoami::devicename(),
            recovery_attempts: 0,
        }
    }

    /// Check if entry is incomplete (needs recovery)
    pub fn is_incomplete(&self) -> bool {
        matches!(self.status, WalStatus::Pending | WalStatus::InProgress)
    }

    /// Check if entry is old enough to be cleaned up
    pub fn is_stale(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.created_at);
        age.num_hours() > WAL_ENTRY_MAX_AGE_HOURS
    }

    /// Get a human-readable description of the operation
    pub fn description(&self) -> String {
        match &self.operation {
            WalOperation::Commit { repo_path, message } => {
                format!("Commit '{}' in {}", message, repo_path.display())
            }
            WalOperation::Push { repo_path, remote, branch } => {
                format!("Push {} to {}/{}", repo_path.display(), remote, branch)
            }
            WalOperation::LockAcquire { repo_path, user_id, timeout_hours } => {
                format!("Lock {} by {} for {}h", repo_path.display(), user_id, timeout_hours)
            }
            WalOperation::LockRelease { repo_path, lock_id } => {
                format!("Release lock {} in {}", lock_id, repo_path.display())
            }
            WalOperation::StageFiles { repo_path, files } => {
                format!("Stage {} files in {}", files.len(), repo_path.display())
            }
        }
    }
}

/// Write-ahead log manager
pub struct WriteAheadLog {
    /// Path to the WAL file
    wal_file: PathBuf,
}

impl WriteAheadLog {
    /// Create a new WAL manager with default location
    pub fn new() -> Self {
        Self {
            wal_file: Self::default_wal_path(),
        }
    }

    /// Create with custom WAL file path
    pub fn with_path(wal_file: PathBuf) -> Self {
        Self { wal_file }
    }

    /// Get default WAL file path (~/.auxin/wal.json)
    fn default_wal_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".auxin").join("wal.json")
    }

    /// Load WAL entries from disk
    fn load_entries(&self) -> Result<Vec<WalEntry>> {
        if !self.wal_file.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&self.wal_file)
            .context("Failed to read WAL file")?;

        if contents.trim().is_empty() {
            return Ok(Vec::new());
        }

        let entries: Vec<WalEntry> = serde_json::from_str(&contents)
            .context("Failed to parse WAL entries")?;

        Ok(entries)
    }

    /// Save WAL entries to disk
    fn save_entries(&self, entries: &[WalEntry]) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.wal_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(entries)?;

        // Write atomically using temp file
        let temp_file = self.wal_file.with_extension("tmp");
        fs::write(&temp_file, &json)
            .context("Failed to write WAL temp file")?;
        fs::rename(&temp_file, &self.wal_file)
            .context("Failed to rename WAL file")?;

        Ok(())
    }

    /// Log intent to perform an operation (call BEFORE executing)
    pub fn log_intent(&self, operation: WalOperation) -> Result<String> {
        let mut entries = self.load_entries()?;

        let entry = WalEntry::new(operation);
        let entry_id = entry.id.clone();

        entries.push(entry);
        self.save_entries(&entries)?;

        Ok(entry_id)
    }

    /// Mark operation as in progress
    pub fn mark_in_progress(&self, entry_id: &str) -> Result<()> {
        let mut entries = self.load_entries()?;

        if let Some(entry) = entries.iter_mut().find(|e| e.id == entry_id) {
            entry.status = WalStatus::InProgress;
            entry.updated_at = Utc::now();
        }

        self.save_entries(&entries)
    }

    /// Mark operation as completed successfully
    pub fn mark_completed(&self, entry_id: &str) -> Result<()> {
        let mut entries = self.load_entries()?;

        if let Some(entry) = entries.iter_mut().find(|e| e.id == entry_id) {
            entry.status = WalStatus::Completed;
            entry.updated_at = Utc::now();
        }

        self.save_entries(&entries)
    }

    /// Mark operation as failed
    pub fn mark_failed(&self, entry_id: &str, error: &str) -> Result<()> {
        let mut entries = self.load_entries()?;

        if let Some(entry) = entries.iter_mut().find(|e| e.id == entry_id) {
            entry.status = WalStatus::Failed(error.to_string());
            entry.updated_at = Utc::now();
        }

        self.save_entries(&entries)
    }

    /// Get all incomplete (pending/in-progress) entries
    pub fn get_incomplete_entries(&self) -> Result<Vec<WalEntry>> {
        let entries = self.load_entries()?;
        Ok(entries.into_iter().filter(|e| e.is_incomplete()).collect())
    }

    /// Get entry by ID
    pub fn get_entry(&self, entry_id: &str) -> Result<Option<WalEntry>> {
        let entries = self.load_entries()?;
        Ok(entries.into_iter().find(|e| e.id == entry_id))
    }

    /// Check if there are any incomplete operations that need recovery
    pub fn needs_recovery(&self) -> Result<bool> {
        let incomplete = self.get_incomplete_entries()?;
        Ok(!incomplete.is_empty())
    }

    /// Mark entry as recovered
    pub fn mark_recovered(&self, entry_id: &str) -> Result<()> {
        let mut entries = self.load_entries()?;

        if let Some(entry) = entries.iter_mut().find(|e| e.id == entry_id) {
            entry.status = WalStatus::Recovered;
            entry.updated_at = Utc::now();
            entry.recovery_attempts += 1;
        }

        self.save_entries(&entries)
    }

    /// Increment recovery attempt counter
    pub fn increment_recovery_attempts(&self, entry_id: &str) -> Result<u32> {
        let mut entries = self.load_entries()?;

        let attempts = if let Some(entry) = entries.iter_mut().find(|e| e.id == entry_id) {
            entry.recovery_attempts += 1;
            entry.updated_at = Utc::now();
            entry.recovery_attempts
        } else {
            0
        };

        self.save_entries(&entries)?;
        Ok(attempts)
    }

    /// Clean up old completed/failed entries
    pub fn cleanup(&self) -> Result<usize> {
        let mut entries = self.load_entries()?;
        let original_count = entries.len();

        // Keep incomplete entries and recent completed/failed entries
        entries.retain(|e| {
            e.is_incomplete() || !e.is_stale()
        });

        let removed = original_count - entries.len();
        self.save_entries(&entries)?;

        Ok(removed)
    }

    /// Clear all WAL entries (use with caution)
    pub fn clear(&self) -> Result<()> {
        if self.wal_file.exists() {
            fs::remove_file(&self.wal_file)?;
        }
        Ok(())
    }

    /// Get statistics about WAL entries
    pub fn get_stats(&self) -> Result<WalStats> {
        let entries = self.load_entries()?;

        let total = entries.len();
        let pending = entries.iter().filter(|e| matches!(e.status, WalStatus::Pending)).count();
        let in_progress = entries.iter().filter(|e| matches!(e.status, WalStatus::InProgress)).count();
        let completed = entries.iter().filter(|e| matches!(e.status, WalStatus::Completed)).count();
        let failed = entries.iter().filter(|e| matches!(e.status, WalStatus::Failed(_))).count();
        let recovered = entries.iter().filter(|e| matches!(e.status, WalStatus::Recovered)).count();

        Ok(WalStats {
            total,
            pending,
            in_progress,
            completed,
            failed,
            recovered,
        })
    }

    /// Display WAL status
    pub fn display_status(&self) -> Result<()> {
        use colored::Colorize;

        let stats = self.get_stats()?;
        let incomplete = self.get_incomplete_entries()?;

        println!("\n{}", "┌─ Write-Ahead Log Status ─────────────────────┐".bright_blue());
        println!("│ Total entries: {}", stats.total);
        println!("│ Pending: {}", stats.pending);
        println!("│ In progress: {}", stats.in_progress);
        println!("│ Completed: {}", stats.completed);
        println!("│ Failed: {}", stats.failed);
        println!("│ Recovered: {}", stats.recovered);

        if !incomplete.is_empty() {
            println!("│");
            println!("│ {} Incomplete operations:", "⚠".yellow());
            for entry in &incomplete {
                println!("│   • {}", entry.description());
            }
        }

        println!("{}\n", "└──────────────────────────────────────────────┘".bright_blue());

        Ok(())
    }
}

impl Default for WriteAheadLog {
    fn default() -> Self {
        Self::new()
    }
}

/// WAL statistics
#[derive(Debug, Clone)]
pub struct WalStats {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
    pub recovered: usize,
}

/// Recovery manager for replaying incomplete WAL entries
pub struct WalRecoveryManager {
    wal: WriteAheadLog,
    max_recovery_attempts: u32,
}

impl WalRecoveryManager {
    /// Create a new recovery manager
    pub fn new() -> Self {
        Self {
            wal: WriteAheadLog::new(),
            max_recovery_attempts: 3,
        }
    }

    /// Create with custom WAL
    pub fn with_wal(wal: WriteAheadLog) -> Self {
        Self {
            wal,
            max_recovery_attempts: 3,
        }
    }

    /// Check and recover any incomplete operations
    pub fn check_and_recover(&self) -> Result<RecoveryReport> {
        let incomplete = self.wal.get_incomplete_entries()?;

        if incomplete.is_empty() {
            return Ok(RecoveryReport {
                entries_found: 0,
                recovered: 0,
                failed: 0,
                skipped: 0,
            });
        }

        let mut report = RecoveryReport {
            entries_found: incomplete.len(),
            recovered: 0,
            failed: 0,
            skipped: 0,
        };

        for entry in incomplete {
            if entry.recovery_attempts >= self.max_recovery_attempts {
                // Too many attempts, mark as failed
                self.wal.mark_failed(&entry.id, "Max recovery attempts exceeded")?;
                report.skipped += 1;
                continue;
            }

            match self.recover_entry(&entry) {
                Ok(true) => {
                    self.wal.mark_recovered(&entry.id)?;
                    report.recovered += 1;
                }
                Ok(false) => {
                    // Recovery not needed or not possible
                    self.wal.increment_recovery_attempts(&entry.id)?;
                    report.skipped += 1;
                }
                Err(e) => {
                    self.wal.mark_failed(&entry.id, &e.to_string())?;
                    report.failed += 1;
                }
            }
        }

        Ok(report)
    }

    /// Attempt to recover a single entry
    fn recover_entry(&self, entry: &WalEntry) -> Result<bool> {
        use colored::Colorize;

        println!("{} Attempting to recover: {}", "⚠".yellow(), entry.description());

        match &entry.operation {
            WalOperation::Commit { repo_path, message: _ } => {
                // Check if commit was actually created
                // If repo has uncommitted changes, the commit failed
                let oxen = crate::oxen_subprocess::OxenSubprocess::new();
                let status = oxen.status(repo_path)?;

                if status.staged_files.is_empty() && status.modified_files.is_empty() {
                    // No pending changes, commit likely succeeded
                    println!("  {} Commit appears to have succeeded", "✓".green());
                    Ok(true)
                } else {
                    // There are still changes, commit may have failed
                    println!("  {} Found uncommitted changes, commit may have failed", "!".yellow());
                    Ok(false)
                }
            }
            WalOperation::Push { repo_path, remote, branch } => {
                // Check if local is ahead of remote
                let oxen = crate::oxen_subprocess::OxenSubprocess::new();
                // For now, just mark as needing manual check
                println!("  {} Push to {}/{} needs manual verification", "!".yellow(), remote, branch);
                let _ = repo_path; // Suppress unused warning
                Ok(false)
            }
            WalOperation::LockAcquire { repo_path, user_id: _, timeout_hours: _ } => {
                // Check if lock was actually acquired
                let manager = crate::remote_lock::RemoteLockManager::new();
                if let Some(lock) = manager.get_lock(repo_path)? {
                    if lock.is_owned_by_current_user() {
                        println!("  {} Lock was acquired successfully", "✓".green());
                        return Ok(true);
                    }
                }
                println!("  {} Lock was not acquired", "✗".red());
                Ok(false)
            }
            WalOperation::LockRelease { repo_path, lock_id: _ } => {
                // Check if lock still exists
                let manager = crate::remote_lock::RemoteLockManager::new();
                if manager.get_lock(repo_path)?.is_none() {
                    println!("  {} Lock was released successfully", "✓".green());
                    return Ok(true);
                }
                println!("  {} Lock still exists", "!".yellow());
                Ok(false)
            }
            WalOperation::StageFiles { repo_path: _, files: _ } => {
                // Staging can be re-run safely
                println!("  {} Stage operation can be safely re-run", "i".blue());
                Ok(false)
            }
        }
    }
}

impl Default for WalRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Report from recovery operation
#[derive(Debug, Clone)]
pub struct RecoveryReport {
    pub entries_found: usize,
    pub recovered: usize,
    pub failed: usize,
    pub skipped: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_wal() -> (WriteAheadLog, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let wal_file = temp_dir.path().join("wal.json");
        let wal = WriteAheadLog::with_path(wal_file);
        (wal, temp_dir)
    }

    #[test]
    fn test_wal_log_intent() {
        let (wal, _temp) = create_test_wal();

        let entry_id = wal.log_intent(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "Test commit".to_string(),
        }).unwrap();

        assert!(!entry_id.is_empty());

        let entry = wal.get_entry(&entry_id).unwrap().unwrap();
        assert_eq!(entry.status, WalStatus::Pending);
    }

    #[test]
    fn test_wal_mark_in_progress() {
        let (wal, _temp) = create_test_wal();

        let entry_id = wal.log_intent(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "Test".to_string(),
        }).unwrap();

        wal.mark_in_progress(&entry_id).unwrap();

        let entry = wal.get_entry(&entry_id).unwrap().unwrap();
        assert_eq!(entry.status, WalStatus::InProgress);
    }

    #[test]
    fn test_wal_mark_completed() {
        let (wal, _temp) = create_test_wal();

        let entry_id = wal.log_intent(WalOperation::Push {
            repo_path: PathBuf::from("/test/repo"),
            remote: "origin".to_string(),
            branch: "main".to_string(),
        }).unwrap();

        wal.mark_completed(&entry_id).unwrap();

        let entry = wal.get_entry(&entry_id).unwrap().unwrap();
        assert_eq!(entry.status, WalStatus::Completed);
    }

    #[test]
    fn test_wal_mark_failed() {
        let (wal, _temp) = create_test_wal();

        let entry_id = wal.log_intent(WalOperation::LockAcquire {
            repo_path: PathBuf::from("/test/repo"),
            user_id: "user@host".to_string(),
            timeout_hours: 4,
        }).unwrap();

        wal.mark_failed(&entry_id, "Network error").unwrap();

        let entry = wal.get_entry(&entry_id).unwrap().unwrap();
        assert!(matches!(entry.status, WalStatus::Failed(_)));
    }

    #[test]
    fn test_wal_get_incomplete() {
        let (wal, _temp) = create_test_wal();

        // Create some entries with different statuses
        let id1 = wal.log_intent(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "Test 1".to_string(),
        }).unwrap();

        let id2 = wal.log_intent(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "Test 2".to_string(),
        }).unwrap();

        wal.mark_completed(&id1).unwrap();
        // id2 stays pending

        let incomplete = wal.get_incomplete_entries().unwrap();
        assert_eq!(incomplete.len(), 1);
        assert_eq!(incomplete[0].id, id2);
    }

    #[test]
    fn test_wal_needs_recovery() {
        let (wal, _temp) = create_test_wal();

        assert!(!wal.needs_recovery().unwrap());

        wal.log_intent(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "Test".to_string(),
        }).unwrap();

        assert!(wal.needs_recovery().unwrap());
    }

    #[test]
    fn test_wal_stats() {
        let (wal, _temp) = create_test_wal();

        let id1 = wal.log_intent(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "Test 1".to_string(),
        }).unwrap();

        let id2 = wal.log_intent(WalOperation::Push {
            repo_path: PathBuf::from("/test/repo"),
            remote: "origin".to_string(),
            branch: "main".to_string(),
        }).unwrap();

        wal.mark_completed(&id1).unwrap();
        wal.mark_failed(&id2, "Error").unwrap();

        let stats = wal.get_stats().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.pending, 0);
    }

    #[test]
    fn test_wal_clear() {
        let (wal, _temp) = create_test_wal();

        wal.log_intent(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "Test".to_string(),
        }).unwrap();

        wal.clear().unwrap();

        let entries = wal.load_entries().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_wal_entry_description() {
        let entry = WalEntry::new(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "My commit".to_string(),
        });

        let desc = entry.description();
        assert!(desc.contains("Commit"));
        assert!(desc.contains("My commit"));
    }

    #[test]
    fn test_wal_entry_is_incomplete() {
        let mut entry = WalEntry::new(WalOperation::Commit {
            repo_path: PathBuf::from("/test"),
            message: "Test".to_string(),
        });

        assert!(entry.is_incomplete()); // Pending

        entry.status = WalStatus::InProgress;
        assert!(entry.is_incomplete());

        entry.status = WalStatus::Completed;
        assert!(!entry.is_incomplete());

        entry.status = WalStatus::Failed("Error".to_string());
        assert!(!entry.is_incomplete());
    }

    #[test]
    fn test_wal_recovery_attempts() {
        let (wal, _temp) = create_test_wal();

        let entry_id = wal.log_intent(WalOperation::Commit {
            repo_path: PathBuf::from("/test/repo"),
            message: "Test".to_string(),
        }).unwrap();

        let attempts = wal.increment_recovery_attempts(&entry_id).unwrap();
        assert_eq!(attempts, 1);

        let attempts = wal.increment_recovery_attempts(&entry_id).unwrap();
        assert_eq!(attempts, 2);
    }
}
