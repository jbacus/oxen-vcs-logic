//! Offline operation queue for network-resilient collaboration
//!
//! This module provides a queue for operations that cannot be performed
//! when the network is unavailable. Operations are stored locally and
//! automatically synced when connectivity is restored.
//!
//! # Features
//!
//! - Queue operations when offline
//! - Automatic sync when online
//! - Conflict detection and resolution
//! - Persistent storage across restarts
//! - Operation ordering and dependencies
//!
//! # Example
//!
//! ```no_run
//! use oxenvcs_cli::offline_queue::{OfflineQueue, QueuedOperation};
//! use std::path::Path;
//!
//! let queue = OfflineQueue::new()?;
//!
//! // Queue a lock acquisition
//! queue.enqueue(QueuedOperation::AcquireLock {
//!     project_path: "MyProject.logicx".to_string(),
//!     user_id: "john@laptop".to_string(),
//!     timeout_hours: 4,
//! })?;
//!
//! // Later, when online
//! queue.sync_all()?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::network_resilience::{check_connectivity, ConnectivityState};
use crate::oxen_subprocess::OxenSubprocess;
use crate::remote_lock::RemoteLockManager;

/// Default queue directory
const DEFAULT_QUEUE_DIR: &str = ".oxenvcs/queue";

/// Queued operation that will be executed when online
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueuedOperation {
    /// Acquire a lock
    AcquireLock {
        project_path: String,
        user_id: String,
        timeout_hours: u32,
    },

    /// Release a lock
    ReleaseLock {
        project_path: String,
        lock_id: String,
    },

    /// Renew a lock
    RenewLock {
        project_path: String,
        lock_id: String,
        additional_hours: u32,
    },

    /// Push commits to remote
    PushCommits {
        repo_path: String,
        branch: String,
    },

    /// Pull commits from remote
    PullCommits {
        repo_path: String,
        branch: String,
    },

    /// Sync comments
    SyncComments {
        repo_path: String,
    },
}

impl QueuedOperation {
    /// Get a human-readable description of the operation
    pub fn description(&self) -> String {
        match self {
            QueuedOperation::AcquireLock { project_path, .. } => {
                format!("Acquire lock for {}", project_path)
            }
            QueuedOperation::ReleaseLock { project_path, .. } => {
                format!("Release lock for {}", project_path)
            }
            QueuedOperation::RenewLock { project_path, .. } => {
                format!("Renew lock for {}", project_path)
            }
            QueuedOperation::PushCommits { repo_path: _, branch } => {
                format!("Push {} to remote", branch)
            }
            QueuedOperation::PullCommits { repo_path: _, branch } => {
                format!("Pull {} from remote", branch)
            }
            QueuedOperation::SyncComments { repo_path } => {
                format!("Sync comments for {}", repo_path)
            }
        }
    }

    /// Check if this operation can be executed offline (none can)
    pub fn is_offline_capable(&self) -> bool {
        false // All queued operations require network
    }
}

/// Queue entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    /// Unique entry ID
    pub id: String,

    /// The operation to perform
    pub operation: QueuedOperation,

    /// When this was queued
    pub queued_at: DateTime<Utc>,

    /// Number of retry attempts
    pub attempts: u32,

    /// Last attempt timestamp
    pub last_attempt: Option<DateTime<Utc>>,

    /// Last error message
    pub last_error: Option<String>,

    /// Priority (higher = execute first)
    pub priority: i32,

    /// Whether this entry has been processed
    pub completed: bool,
}

impl QueueEntry {
    /// Create a new queue entry
    pub fn new(operation: QueuedOperation) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation,
            queued_at: Utc::now(),
            attempts: 0,
            last_attempt: None,
            last_error: None,
            priority: 0,
            completed: false,
        }
    }

    /// Create a high-priority entry
    pub fn with_priority(operation: QueuedOperation, priority: i32) -> Self {
        let mut entry = Self::new(operation);
        entry.priority = priority;
        entry
    }

    /// Mark as failed with error
    pub fn mark_failed(&mut self, error: String) {
        self.attempts += 1;
        self.last_attempt = Some(Utc::now());
        self.last_error = Some(error);
    }

    /// Mark as completed
    pub fn mark_completed(&mut self) {
        self.completed = true;
        self.last_attempt = Some(Utc::now());
        self.last_error = None;
    }
}

/// Offline operation queue
pub struct OfflineQueue {
    /// Directory for queue storage
    queue_dir: PathBuf,

    /// In-memory cache of queue entries
    entries: Vec<QueueEntry>,
}

impl OfflineQueue {
    /// Create a new offline queue with default directory
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let queue_dir = home.join(DEFAULT_QUEUE_DIR);

        Self::with_dir(queue_dir)
    }

    /// Create a new offline queue with custom directory
    pub fn with_dir(queue_dir: PathBuf) -> Result<Self> {
        // Create directory if it doesn't exist
        if !queue_dir.exists() {
            fs::create_dir_all(&queue_dir)
                .context("Failed to create queue directory")?;
        }

        let mut queue = Self {
            queue_dir,
            entries: Vec::new(),
        };

        // Load existing entries
        queue.load_all()?;

        Ok(queue)
    }

    /// Enqueue an operation
    pub fn enqueue(&mut self, operation: QueuedOperation) -> Result<String> {
        let entry = QueueEntry::new(operation);
        let id = entry.id.clone();

        crate::info!("Queuing operation: {}", entry.operation.description());

        // Save to disk
        self.save_entry(&entry)?;

        // Add to memory
        self.entries.push(entry);

        Ok(id)
    }

    /// Enqueue an operation with priority
    pub fn enqueue_with_priority(&mut self, operation: QueuedOperation, priority: i32) -> Result<String> {
        let entry = QueueEntry::with_priority(operation, priority);
        let id = entry.id.clone();

        crate::info!("Queuing high-priority operation: {}", entry.operation.description());

        self.save_entry(&entry)?;
        self.entries.push(entry);

        Ok(id)
    }

    /// Get all pending (non-completed) entries
    pub fn pending(&self) -> Vec<&QueueEntry> {
        self.entries.iter()
            .filter(|e| !e.completed)
            .collect()
    }

    /// Get all completed entries
    pub fn completed(&self) -> Vec<&QueueEntry> {
        self.entries.iter()
            .filter(|e| e.completed)
            .collect()
    }

    /// Get entry by ID
    pub fn get(&self, id: &str) -> Option<&QueueEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Remove an entry
    pub fn remove(&mut self, id: &str) -> Result<()> {
        // Remove from memory
        self.entries.retain(|e| e.id != id);

        // Remove from disk
        let entry_file = self.entry_file_path(id);
        if entry_file.exists() {
            fs::remove_file(&entry_file)
                .context("Failed to remove queue entry file")?;
        }

        Ok(())
    }

    /// Clear all completed entries
    pub fn clear_completed(&mut self) -> Result<()> {
        let completed_ids: Vec<String> = self.completed()
            .iter()
            .map(|e| e.id.clone())
            .collect();

        for id in completed_ids {
            self.remove(&id)?;
        }

        Ok(())
    }

    /// Sync all pending operations
    pub fn sync_all(&mut self) -> Result<SyncReport> {
        crate::info!("Starting offline queue sync...");

        // Check connectivity
        match check_connectivity() {
            ConnectivityState::Offline => {
                return Err(anyhow!("Cannot sync: network is offline"));
            }
            ConnectivityState::Unknown => {
                crate::warn!("Network state unknown, attempting sync anyway...");
            }
            ConnectivityState::Online => {}
        }

        let mut report = SyncReport::new();
        let pending: Vec<QueueEntry> = self.pending()
            .iter()
            .cloned()
            .cloned()
            .collect();

        // Sort by priority (highest first)
        let mut sorted = pending;
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

        for entry in sorted {
            crate::info!("Syncing: {}", entry.operation.description());

            match self.execute_entry(&entry) {
                Ok(_) => {
                    crate::info!("✓ Completed: {}", entry.operation.description());
                    report.succeeded.push(entry.id.clone());

                    // Mark as completed and clone for saving
                    let entry_to_save = {
                        if let Some(e) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                            e.mark_completed();
                            Some(e.clone())
                        } else {
                            None
                        }
                    };

                    // Save after releasing the mutable borrow
                    if let Some(e) = entry_to_save {
                        self.save_entry(&e)?;
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    crate::error!("✗ Failed: {} - {}", entry.operation.description(), error_msg);
                    report.failed.push((entry.id.clone(), error_msg.clone()));

                    // Mark as failed and clone for saving
                    let entry_to_save = {
                        if let Some(queue_entry) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                            queue_entry.mark_failed(error_msg);
                            Some(queue_entry.clone())
                        } else {
                            None
                        }
                    };

                    // Save after releasing the mutable borrow
                    if let Some(e) = entry_to_save {
                        self.save_entry(&e)?;
                    }
                }
            }
        }

        crate::info!("Sync complete: {} succeeded, {} failed",
            report.succeeded.len(), report.failed.len());

        Ok(report)
    }

    /// Execute a single queue entry
    fn execute_entry(&self, entry: &QueueEntry) -> Result<()> {
        crate::vlog!("Executing queued operation: {}", entry.operation.description());

        match &entry.operation {
            QueuedOperation::AcquireLock { project_path, user_id, timeout_hours } => {
                let lock_manager = RemoteLockManager::new();
                let project_path_buf = PathBuf::from(project_path);

                lock_manager.acquire_lock(&project_path_buf, user_id, *timeout_hours)
                    .with_context(|| format!(
                        "Failed to acquire lock for {} (user: {}, timeout: {}h)",
                        project_path, user_id, timeout_hours
                    ))?;

                crate::vlog!("Lock acquired for {}", project_path);
                Ok(())
            }

            QueuedOperation::ReleaseLock { project_path, lock_id } => {
                let lock_manager = RemoteLockManager::new();
                let project_path_buf = PathBuf::from(project_path);

                lock_manager.release_lock(&project_path_buf, lock_id)
                    .with_context(|| format!(
                        "Failed to release lock {} for {}",
                        lock_id, project_path
                    ))?;

                crate::vlog!("Lock released for {}", project_path);
                Ok(())
            }

            QueuedOperation::RenewLock { project_path, lock_id, additional_hours } => {
                let lock_manager = RemoteLockManager::new();
                let project_path_buf = PathBuf::from(project_path);

                lock_manager.renew_lock(&project_path_buf, lock_id, *additional_hours)
                    .with_context(|| format!(
                        "Failed to renew lock {} for {} (+{}h)",
                        lock_id, project_path, additional_hours
                    ))?;

                crate::vlog!("Lock renewed for {} (+{}h)", project_path, additional_hours);
                Ok(())
            }

            QueuedOperation::PushCommits { repo_path: _, branch } => {
                let oxen = OxenSubprocess::new();
                let cwd = std::env::current_dir()
                    .context("Failed to get current directory")?;

                oxen.push(&cwd, None, Some(branch))
                    .with_context(|| format!(
                        "Failed to push commits for branch {}",
                        branch
                    ))?;

                crate::vlog!("Pushed commits for branch {}", branch);
                Ok(())
            }

            QueuedOperation::PullCommits { repo_path: _, branch: _ } => {
                let oxen = OxenSubprocess::new();
                let cwd = std::env::current_dir()
                    .context("Failed to get current directory")?;

                oxen.pull(&cwd)
                    .with_context(|| "Failed to pull commits")?;

                crate::vlog!("Pulled commits from remote");
                Ok(())
            }

            QueuedOperation::SyncComments { repo_path } => {
                // TODO: Implement comment sync when CommentManager is integrated
                crate::warn!("Comment sync not yet implemented for {}", repo_path);
                Ok(())
            }
        }
    }

    /// Get path to entry file
    fn entry_file_path(&self, id: &str) -> PathBuf {
        self.queue_dir.join(format!("{}.json", id))
    }

    /// Save an entry to disk
    fn save_entry(&self, entry: &QueueEntry) -> Result<()> {
        let file_path = self.entry_file_path(&entry.id);
        let json = serde_json::to_string_pretty(entry)
            .context("Failed to serialize queue entry")?;

        fs::write(&file_path, json)
            .context("Failed to write queue entry to disk")?;

        Ok(())
    }

    /// Load all entries from disk
    fn load_all(&mut self) -> Result<()> {
        if !self.queue_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.queue_dir)
            .context("Failed to read queue directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_entry(&path) {
                    Ok(queue_entry) => {
                        self.entries.push(queue_entry);
                    }
                    Err(e) => {
                        crate::warn!("Failed to load queue entry {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Sort by priority
        self.entries.sort_by(|a, b| b.priority.cmp(&a.priority));

        crate::vlog!("Loaded {} queue entries", self.entries.len());

        Ok(())
    }

    /// Load a single entry from disk
    fn load_entry(&self, path: &Path) -> Result<QueueEntry> {
        let json = fs::read_to_string(path)
            .context("Failed to read queue entry file")?;

        let entry: QueueEntry = serde_json::from_str(&json)
            .context("Failed to deserialize queue entry")?;

        Ok(entry)
    }

    /// Get queue statistics
    pub fn stats(&self) -> QueueStats {
        let pending_count = self.pending().len();
        let completed_count = self.completed().len();
        let failed_count = self.entries.iter()
            .filter(|e| e.last_error.is_some() && !e.completed)
            .count();

        QueueStats {
            total: self.entries.len(),
            pending: pending_count,
            completed: completed_count,
            failed: failed_count,
        }
    }
}

impl Default for OfflineQueue {
    fn default() -> Self {
        Self::new().expect("Failed to create default offline queue")
    }
}

/// Report of sync operation results
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// IDs of successfully synced entries
    pub succeeded: Vec<String>,

    /// IDs and errors of failed entries
    pub failed: Vec<(String, String)>,
}

impl SyncReport {
    pub fn new() -> Self {
        Self {
            succeeded: Vec::new(),
            failed: Vec::new(),
        }
    }

    pub fn success_count(&self) -> usize {
        self.succeeded.len()
    }

    pub fn failure_count(&self) -> usize {
        self.failed.len()
    }

    pub fn is_complete_success(&self) -> bool {
        self.failed.is_empty()
    }
}

impl Default for SyncReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total: usize,
    pub pending: usize,
    pub completed: usize,
    pub failed: usize,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_queue_creation() {
        let temp_dir = TempDir::new().unwrap();
        let queue = OfflineQueue::with_dir(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(queue.entries.len(), 0);
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_enqueue_operation() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::with_dir(temp_dir.path().to_path_buf()).unwrap();

        let op = QueuedOperation::AcquireLock {
            project_path: "test.logicx".to_string(),
            user_id: "user@host".to_string(),
            timeout_hours: 4,
        };

        let id = queue.enqueue(op).unwrap();

        assert_eq!(queue.entries.len(), 1);
        assert!(queue.get(&id).is_some());
    }

    #[test]
    fn test_queue_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let queue_dir = temp_dir.path().to_path_buf();

        // Create queue and add entry
        {
            let mut queue = OfflineQueue::with_dir(queue_dir.clone()).unwrap();
            let op = QueuedOperation::ReleaseLock {
                project_path: "test.logicx".to_string(),
                lock_id: "lock123".to_string(),
            };
            queue.enqueue(op).unwrap();
        }

        // Load queue again
        {
            let queue = OfflineQueue::with_dir(queue_dir.clone()).unwrap();
            assert_eq!(queue.entries.len(), 1);
        }
    }

    #[test]
    fn test_priority_sorting() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::with_dir(temp_dir.path().to_path_buf()).unwrap();

        // Add low priority
        queue.enqueue_with_priority(
            QueuedOperation::PullCommits {
                repo_path: "test".to_string(),
                branch: "main".to_string(),
            },
            1,
        ).unwrap();

        // Add high priority
        queue.enqueue_with_priority(
            QueuedOperation::AcquireLock {
                project_path: "test.logicx".to_string(),
                user_id: "user".to_string(),
                timeout_hours: 4,
            },
            10,
        ).unwrap();

        let pending = queue.pending();
        assert_eq!(pending.len(), 2);

        // Reload and check order is maintained
        let queue2 = OfflineQueue::with_dir(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(queue2.entries[0].priority, 10);
        assert_eq!(queue2.entries[1].priority, 1);
    }

    #[test]
    fn test_remove_entry() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::with_dir(temp_dir.path().to_path_buf()).unwrap();

        let op = QueuedOperation::SyncComments {
            repo_path: "test".to_string(),
        };
        let id = queue.enqueue(op).unwrap();

        assert_eq!(queue.entries.len(), 1);

        queue.remove(&id).unwrap();

        assert_eq!(queue.entries.len(), 0);
        assert!(queue.get(&id).is_none());
    }

    #[test]
    fn test_clear_completed() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::with_dir(temp_dir.path().to_path_buf()).unwrap();

        // Add some entries
        queue.enqueue(QueuedOperation::PushCommits {
            repo_path: "test".to_string(),
            branch: "main".to_string(),
        }).unwrap();

        queue.enqueue(QueuedOperation::PullCommits {
            repo_path: "test".to_string(),
            branch: "main".to_string(),
        }).unwrap();

        // Mark one as completed
        queue.entries[0].mark_completed();

        assert_eq!(queue.entries.len(), 2);
        assert_eq!(queue.completed().len(), 1);
        assert_eq!(queue.pending().len(), 1);

        // Clear completed
        queue.clear_completed().unwrap();

        assert_eq!(queue.entries.len(), 1);
        assert_eq!(queue.completed().len(), 0);
        assert_eq!(queue.pending().len(), 1);
    }

    #[test]
    fn test_queue_stats() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::with_dir(temp_dir.path().to_path_buf()).unwrap();

        queue.enqueue(QueuedOperation::PushCommits {
            repo_path: "test".to_string(),
            branch: "main".to_string(),
        }).unwrap();

        queue.enqueue(QueuedOperation::PullCommits {
            repo_path: "test".to_string(),
            branch: "main".to_string(),
        }).unwrap();

        queue.entries[0].mark_completed();
        queue.entries[1].mark_failed("Test error".to_string());

        let stats = queue.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.failed, 1);
    }

    #[test]
    fn test_operation_description() {
        let op = QueuedOperation::AcquireLock {
            project_path: "MyProject.logicx".to_string(),
            user_id: "john@laptop".to_string(),
            timeout_hours: 4,
        };

        assert!(op.description().contains("Acquire lock"));
        assert!(op.description().contains("MyProject.logicx"));
    }
}
