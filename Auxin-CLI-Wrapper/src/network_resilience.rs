use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration as StdDuration;

/// Maximum number of retry attempts for network operations
const MAX_RETRIES: u32 = 4;

/// Initial backoff duration in milliseconds
const INITIAL_BACKOFF_MS: u64 = 2000;

/// Maximum backoff duration in milliseconds (16 seconds)
const MAX_BACKOFF_MS: u64 = 16000;

/// Connectivity state for network checks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityState {
    Online,
    Offline,
    Unknown,
}

/// Check current connectivity state
pub fn check_connectivity() -> ConnectivityState {
    if check_network_availability() {
        ConnectivityState::Online
    } else {
        ConnectivityState::Offline
    }
}

/// Retry policy for network operations
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_retries: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    verbose: bool,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_retries: u32, initial_backoff_ms: u64, max_backoff_ms: u64) -> Self {
        Self {
            max_retries,
            initial_backoff_ms,
            max_backoff_ms,
            verbose: false,
        }
    }

    /// Enable verbose logging
    pub fn set_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Execute an operation with retry logic
    pub fn execute<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let mut attempt = 0;
        let mut backoff_ms = self.initial_backoff_ms;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    if attempt > self.max_retries {
                        return Err(e);
                    }

                    if self.verbose {
                        crate::vlog!("Retry attempt {}/{}: {}", attempt, self.max_retries, e);
                    }

                    thread::sleep(StdDuration::from_millis(backoff_ms));
                    backoff_ms = (backoff_ms * 2).min(self.max_backoff_ms);
                }
            }
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(MAX_RETRIES, INITIAL_BACKOFF_MS, MAX_BACKOFF_MS)
    }
}

/// Represents a queued operation that failed due to network issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueuedOperation {
    /// Unique identifier for this operation
    pub id: String,

    /// Type of operation (push, pull, lock_acquire, etc.)
    pub operation_type: OperationType,

    /// Repository path
    pub repo_path: PathBuf,

    /// Additional operation-specific data
    pub data: OperationData,

    /// When this operation was first queued
    pub queued_at: DateTime<Utc>,

    /// Number of times this operation has been attempted
    pub attempt_count: u32,

    /// Last error message
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Push,
    Pull,
    LockAcquire,
    LockRelease,
    LockRenew,
    CommentSync,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperationData {
    /// Branch name (for push/pull)
    pub branch: Option<String>,

    /// Commit message (for commits)
    pub message: Option<String>,

    /// Lock timeout (for lock operations)
    pub timeout_hours: Option<u32>,

    /// Additional key-value data
    pub extra: std::collections::HashMap<String, String>,
}

impl OperationData {
    pub fn new() -> Self {
        Self {
            branch: None,
            message: None,
            timeout_hours: None,
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_timeout(mut self, hours: u32) -> Self {
        self.timeout_hours = Some(hours);
        self
    }
}

impl Default for OperationData {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages offline operation queue and network retry logic
pub struct NetworkResilienceManager {
    queue_file: PathBuf,
    operations: VecDeque<QueuedOperation>,
}

impl NetworkResilienceManager {
    /// Create a new NetworkResilienceManager with default queue location
    pub fn new() -> Self {
        let queue_file = Self::default_queue_path();
        Self {
            queue_file,
            operations: VecDeque::new(),
        }
    }

    /// Create with custom queue file path
    pub fn with_queue_path(queue_file: PathBuf) -> Self {
        Self {
            queue_file,
            operations: VecDeque::new(),
        }
    }

    /// Get default queue file path (~/.auxin/operation_queue.json)
    fn default_queue_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".auxin").join("operation_queue.json")
    }

    /// Load queued operations from disk
    pub fn load_queue(&mut self) -> Result<()> {
        if !self.queue_file.exists() {
            return Ok(());
        }

        let contents = fs::read_to_string(&self.queue_file)
            .context("Failed to read operation queue file")?;

        let ops: Vec<QueuedOperation> = serde_json::from_str(&contents)
            .context("Failed to parse operation queue")?;

        self.operations = ops.into();
        Ok(())
    }

    /// Save queued operations to disk
    pub fn save_queue(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.queue_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let ops: Vec<_> = self.operations.iter().collect();
        let json = serde_json::to_string_pretty(&ops)?;

        fs::write(&self.queue_file, json)
            .context("Failed to write operation queue file")?;

        Ok(())
    }

    /// Add an operation to the queue
    pub fn enqueue(&mut self, mut operation: QueuedOperation) -> Result<()> {
        operation.queued_at = Utc::now();
        operation.attempt_count = 0;
        self.operations.push_back(operation);
        self.save_queue()?;
        Ok(())
    }

    /// Get the next operation to retry
    pub fn dequeue(&mut self) -> Option<QueuedOperation> {
        let op = self.operations.pop_front();
        if op.is_some() {
            let _ = self.save_queue();
        }
        op
    }

    /// Peek at the next operation without removing it
    pub fn peek(&self) -> Option<&QueuedOperation> {
        self.operations.front()
    }

    /// Get number of queued operations
    pub fn queue_size(&self) -> usize {
        self.operations.len()
    }

    /// Clear all queued operations
    pub fn clear_queue(&mut self) -> Result<()> {
        self.operations.clear();
        self.save_queue()
    }

    /// Mark an operation as failed and re-queue if under retry limit
    pub fn mark_failed(&mut self, mut operation: QueuedOperation, error: String) -> Result<bool> {
        operation.attempt_count += 1;
        operation.last_error = Some(error);

        if operation.attempt_count < MAX_RETRIES {
            // Re-queue for retry
            self.operations.push_back(operation);
            self.save_queue()?;
            Ok(true) // Will retry
        } else {
            // Max retries exceeded, don't re-queue
            self.save_queue()?;
            Ok(false) // Won't retry
        }
    }

    /// Execute a network operation with retry logic
    pub fn execute_with_retry<F>(&self, mut operation: F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < MAX_RETRIES {
            match operation() {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    attempt += 1;

                    if attempt < MAX_RETRIES {
                        // Calculate exponential backoff
                        let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
                        let backoff_ms = backoff_ms.min(MAX_BACKOFF_MS);

                        crate::vlog!("Retry attempt {}/{}, waiting {}ms", attempt, MAX_RETRIES, backoff_ms);
                        thread::sleep(StdDuration::from_millis(backoff_ms));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Operation failed after {} retries", MAX_RETRIES)))
    }
}

impl Default for NetworkResilienceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect if an error is transient (retryable) or permanent
pub fn is_transient_error(error: &anyhow::Error) -> bool {
    let error_str = error.to_string().to_lowercase();

    // Network-related errors that are typically transient
    let transient_patterns = [
        "timeout",
        "connection refused",
        "connection reset",
        "broken pipe",
        "network unreachable",
        "temporary failure",
        "502",
        "503",
        "504",
        "try again",
    ];

    transient_patterns.iter().any(|pattern| error_str.contains(pattern))
}

/// Check if network is available by attempting to connect to Oxen Hub
pub fn check_network_availability() -> bool {
    use std::process::Command;

    // Try to ping Oxen Hub
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "2", "hub.oxen.ai"])
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_network_resilience_manager_creation() {
        let manager = NetworkResilienceManager::new();
        assert_eq!(manager.queue_size(), 0);
    }

    #[test]
    fn test_enqueue_and_dequeue() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("queue.json");
        let mut manager = NetworkResilienceManager::with_queue_path(queue_file);

        let op = QueuedOperation {
            id: "test-1".to_string(),
            operation_type: OperationType::Push,
            repo_path: PathBuf::from("/test/repo"),
            data: OperationData::new().with_branch("main"),
            queued_at: Utc::now(),
            attempt_count: 0,
            last_error: None,
        };

        manager.enqueue(op.clone()).unwrap();
        assert_eq!(manager.queue_size(), 1);

        let dequeued = manager.dequeue().unwrap();
        assert_eq!(dequeued.id, "test-1");
        assert_eq!(manager.queue_size(), 0);
    }

    #[test]
    fn test_queue_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("queue.json");

        // Create and enqueue
        {
            let mut manager = NetworkResilienceManager::with_queue_path(queue_file.clone());
            let op = QueuedOperation {
                id: "persist-test".to_string(),
                operation_type: OperationType::LockAcquire,
                repo_path: PathBuf::from("/test"),
                data: OperationData::new().with_timeout(4),
                queued_at: Utc::now(),
                attempt_count: 0,
                last_error: None,
            };
            manager.enqueue(op).unwrap();
        }

        // Load in new instance
        {
            let mut manager = NetworkResilienceManager::with_queue_path(queue_file);
            manager.load_queue().unwrap();
            assert_eq!(manager.queue_size(), 1);

            let op = manager.peek().unwrap();
            assert_eq!(op.id, "persist-test");
        }
    }

    #[test]
    fn test_mark_failed_retry_logic() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("queue.json");
        let mut manager = NetworkResilienceManager::with_queue_path(queue_file);

        let op = QueuedOperation {
            id: "retry-test".to_string(),
            operation_type: OperationType::Push,
            repo_path: PathBuf::from("/test"),
            data: OperationData::new(),
            queued_at: Utc::now(),
            attempt_count: 0,
            last_error: None,
        };

        // First 3 failures should re-queue
        let will_retry = manager.mark_failed(op.clone(), "Network error".to_string()).unwrap();
        assert!(will_retry);
        assert_eq!(manager.queue_size(), 1);

        // After MAX_RETRIES, should not re-queue
        let mut op_max = op.clone();
        op_max.attempt_count = MAX_RETRIES - 1;
        let will_retry = manager.mark_failed(op_max, "Network error".to_string()).unwrap();
        assert!(!will_retry);
    }

    #[test]
    fn test_is_transient_error() {
        let timeout_err = anyhow!("Connection timeout");
        assert!(is_transient_error(&timeout_err));

        let refused_err = anyhow!("Connection refused");
        assert!(is_transient_error(&refused_err));

        let auth_err = anyhow!("Authentication failed");
        assert!(!is_transient_error(&auth_err));
    }

    #[test]
    fn test_operation_data_builder() {
        let data = OperationData::new()
            .with_branch("main")
            .with_message("Test commit")
            .with_timeout(4);

        assert_eq!(data.branch, Some("main".to_string()));
        assert_eq!(data.message, Some("Test commit".to_string()));
        assert_eq!(data.timeout_hours, Some(4));
    }

    #[test]
    fn test_clear_queue() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("queue.json");
        let mut manager = NetworkResilienceManager::with_queue_path(queue_file);

        // Add multiple operations
        for i in 0..5 {
            let op = QueuedOperation {
                id: format!("op-{}", i),
                operation_type: OperationType::Push,
                repo_path: PathBuf::from("/test"),
                data: OperationData::new(),
                queued_at: Utc::now(),
                attempt_count: 0,
                last_error: None,
            };
            manager.enqueue(op).unwrap();
        }

        assert_eq!(manager.queue_size(), 5);

        manager.clear_queue().unwrap();
        assert_eq!(manager.queue_size(), 0);
    }
}
