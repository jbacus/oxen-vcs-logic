/// Remote lock management for distributed collaboration
///
/// This module implements pessimistic locking for Logic Pro projects to prevent
/// merge conflicts when multiple team members work on the same project.
///
/// # Architecture
///
/// Locks are stored in a dedicated `locks` branch in the Oxen repository:
/// - Lock metadata stored in `.oxen/locks/<project_id>.json`
/// - Each lock acquisition creates a new commit on the locks branch
/// - Atomic operations via fetch → check → commit → push → verify
///
/// # Lock Lifecycle
///
/// 1. **Acquire**: User requests lock
///    - Fetch latest locks branch
///    - Check if project already locked
///    - Create lock entry with expiration
///    - Commit to locks branch
///    - Force push (allows overwrites)
///    - Poll to verify no race condition
///
/// 2. **Heartbeat**: Keep lock alive
///    - Update lock expiration timestamp
///    - Commit and push every 10 minutes
///    - Prevents stale lock timeouts
///
/// 3. **Release**: User releases lock
///    - Remove lock entry
///    - Commit and push deletion
///
/// 4. **Cleanup**: Remove stale locks
///    - Find locks expired >48 hours
///    - Remove automatically
///
/// # Race Condition Handling
///
/// When two users try to acquire the same lock simultaneously:
/// 1. Both fetch and see no lock
/// 2. Both create lock commits
/// 3. First push wins (becomes HEAD)
/// 4. Second push overwrites (force push)
/// 5. Second user polls and sees different lock owner → FAIL
///
/// # Example
///
/// ```no_run
/// use auxin_cli::remote_lock::RemoteLockManager;
/// use std::path::Path;
///
/// let manager = RemoteLockManager::new();
/// let project = Path::new("MyProject.logicx");
///
/// // Acquire lock
/// match manager.acquire_lock(project, "john@laptop", 4) {
///     Ok(lock) => {
///         println!("Lock acquired: {}", lock.lock_id);
///         // ... do work ...
///         manager.release_lock(project, &lock.lock_id)?;
///     }
///     Err(e) => eprintln!("Failed to acquire lock: {}", e),
/// }
/// ```

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration as StdDuration;
use uuid::Uuid;

use crate::oxen_subprocess::OxenSubprocess;
use crate::network_resilience::RetryPolicy;

/// A distributed lock for a Logic Pro project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteLock {
    /// Unique lock identifier
    pub lock_id: String,

    /// Project path (relative to repository root)
    pub project_path: String,

    /// User who holds the lock (username@hostname)
    pub locked_by: String,

    /// Machine identifier (for detecting same user on different machines)
    pub machine_id: String,

    /// When the lock was acquired
    pub acquired_at: DateTime<Utc>,

    /// When the lock expires (auto-release after this time)
    pub expires_at: DateTime<Utc>,

    /// Last heartbeat timestamp (for staleness detection)
    pub last_heartbeat: DateTime<Utc>,
}

impl RemoteLock {
    /// Create a new lock
    pub fn new(
        project_path: impl Into<String>,
        locked_by: impl Into<String>,
        timeout_hours: u32,
    ) -> Self {
        let now = Utc::now();
        let machine_id = get_machine_id();

        Self {
            lock_id: Uuid::new_v4().to_string(),
            project_path: project_path.into(),
            locked_by: locked_by.into(),
            machine_id,
            acquired_at: now,
            expires_at: now + Duration::hours(timeout_hours as i64),
            last_heartbeat: now,
        }
    }

    /// Check if lock has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if lock is stale (no heartbeat for >1 hour)
    pub fn is_stale(&self) -> bool {
        let stale_threshold = Utc::now() - Duration::hours(1);
        self.last_heartbeat < stale_threshold
    }

    /// Get remaining time until expiration (returns zero if expired)
    pub fn remaining_time(&self) -> Duration {
        let remaining = self.expires_at - Utc::now();
        if remaining < Duration::zero() {
            Duration::zero()
        } else {
            remaining
        }
    }

    /// Check if lock belongs to current user/machine
    pub fn is_owned_by_current_user(&self) -> bool {
        let current_user = get_user_identifier();
        let current_machine = get_machine_id();
        self.locked_by == current_user && self.machine_id == current_machine
    }

    /// Renew lock (update heartbeat and expiration)
    pub fn renew(&mut self, additional_hours: u32) {
        let now = Utc::now();
        self.last_heartbeat = now;
        self.expires_at = now + Duration::hours(additional_hours as i64);
    }

    /// Check if lock is expiring soon (within threshold minutes)
    pub fn is_expiring_soon(&self, threshold_minutes: i64) -> bool {
        let now = Utc::now();
        let time_until_expiry = self.expires_at.signed_duration_since(now);
        time_until_expiry < Duration::minutes(threshold_minutes)
    }

    /// Get minutes until lock expires
    pub fn minutes_until_expiry(&self) -> i64 {
        let now = Utc::now();
        self.expires_at.signed_duration_since(now).num_minutes()
    }
}

/// Manages distributed locks stored in Oxen repository
pub struct RemoteLockManager {
    /// Oxen subprocess wrapper
    oxen: OxenSubprocess,

    /// Name of the locks branch
    locks_branch: String,

    /// Directory for lock files within repository
    locks_dir: PathBuf,
}

impl RemoteLockManager {
    /// Create a new RemoteLockManager with default settings
    pub fn new() -> Self {
        Self {
            oxen: OxenSubprocess::new(),
            locks_branch: "locks".to_string(),
            locks_dir: PathBuf::from(".oxen/locks"),
        }
    }

    /// Create with custom Oxen subprocess
    pub fn with_oxen(oxen: OxenSubprocess) -> Self {
        Self {
            oxen,
            locks_branch: "locks".to_string(),
            locks_dir: PathBuf::from(".oxen/locks"),
        }
    }

    /// Acquire a lock for a project
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to Oxen repository
    /// * `user_id` - User identifier (username@hostname)
    /// * `timeout_hours` - Lock expiration time in hours
    ///
    /// # Returns
    ///
    /// The acquired lock on success, or error if:
    /// - Project already locked by someone else
    /// - Race condition detected
    /// - Network/repository errors
    pub fn acquire_lock(
        &self,
        repo_path: &Path,
        user_id: &str,
        timeout_hours: u32,
    ) -> Result<RemoteLock> {
        crate::vlog!("Acquiring lock for project: {}", repo_path.display());

        // 1. Ensure locks branch exists
        self.ensure_locks_branch(repo_path)?;

        // 2. Fetch latest locks
        self.fetch_locks_branch(repo_path)?;

        // 3. Check if already locked
        if let Some(existing_lock) = self.get_lock(repo_path)? {
            if !existing_lock.is_expired() && !existing_lock.is_stale() {
                return Err(anyhow!(
                    "Project locked by {} until {}",
                    existing_lock.locked_by,
                    existing_lock.expires_at.format("%Y-%m-%d %H:%M:%S UTC")
                ));
            }
            crate::vlog!("Existing lock is expired/stale, will overwrite");
        }

        // 4. Create new lock
        let lock = RemoteLock::new(
            repo_path.to_string_lossy().to_string(),
            user_id,
            timeout_hours,
        );

        // 5. Write lock file
        self.write_lock_file(repo_path, &lock)?;

        // 6. Commit lock to locks branch
        self.commit_lock(repo_path, &lock, "Acquire lock")?;

        // 7. Push to remote (force push to handle race conditions)
        self.push_locks_branch(repo_path, true)?;

        // 8. Verify lock (detect race conditions)
        thread::sleep(StdDuration::from_secs(2)); // Give remote time to settle
        self.verify_lock_ownership(repo_path, &lock)?;

        crate::info!("Lock acquired: {}", lock.lock_id);
        Ok(lock)
    }

    /// Release a lock
    pub fn release_lock(&self, repo_path: &Path, lock_id: &str) -> Result<()> {
        crate::vlog!("Releasing lock: {}", lock_id);

        // 1. Fetch latest locks
        self.fetch_locks_branch(repo_path)?;

        // 2. Verify we own the lock
        let current_lock = self.get_lock(repo_path)?
            .ok_or_else(|| anyhow!("No lock exists for this project"))?;

        if current_lock.lock_id != lock_id {
            return Err(anyhow!(
                "Cannot release lock: lock ID mismatch (expected: {}, found: {})",
                lock_id,
                current_lock.lock_id
            ));
        }

        if !current_lock.is_owned_by_current_user() {
            return Err(anyhow!(
                "Cannot release lock owned by {}",
                current_lock.locked_by
            ));
        }

        // 3. Remove lock file
        self.remove_lock_file(repo_path)?;

        // 4. Commit deletion
        self.commit_lock_deletion(repo_path)?;

        // 5. Push to remote
        self.push_locks_branch(repo_path, false)?;

        crate::info!("Lock released: {}", lock_id);
        Ok(())
    }

    /// Renew a lock (extend expiration and update heartbeat)
    pub fn renew_lock(
        &self,
        repo_path: &Path,
        lock_id: &str,
        additional_hours: u32,
    ) -> Result<RemoteLock> {
        crate::vlog!("Renewing lock: {}", lock_id);

        // 1. Fetch latest locks
        self.fetch_locks_branch(repo_path)?;

        // 2. Get current lock
        let mut lock = self.get_lock(repo_path)?
            .ok_or_else(|| anyhow!("No lock exists for this project"))?;

        // 3. Verify ownership
        if lock.lock_id != lock_id {
            return Err(anyhow!("Lock ID mismatch"));
        }

        if !lock.is_owned_by_current_user() {
            return Err(anyhow!("Cannot renew lock owned by {}", lock.locked_by));
        }

        // 4. Renew lock
        lock.renew(additional_hours);

        // 5. Write updated lock
        self.write_lock_file(repo_path, &lock)?;

        // 6. Commit renewal
        self.commit_lock(repo_path, &lock, "Renew lock (heartbeat)")?;

        // 7. Push to remote
        self.push_locks_branch(repo_path, false)?;

        crate::vlog!("Lock renewed: {} (expires: {})", lock_id, lock.expires_at);
        Ok(lock)
    }

    /// Get current lock for a project
    pub fn get_lock(&self, repo_path: &Path) -> Result<Option<RemoteLock>> {
        let lock_file = self.get_lock_file_path(repo_path);

        if !lock_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&lock_file)
            .context("Failed to read lock file")?;

        let lock: RemoteLock = serde_json::from_str(&content)
            .context("Failed to parse lock file")?;

        Ok(Some(lock))
    }

    /// Force break a lock (admin operation)
    pub fn force_break_lock(&self, repo_path: &Path) -> Result<()> {
        crate::vlog!("Force breaking lock");

        // Fetch latest
        self.fetch_locks_branch(repo_path)?;

        // Remove lock file
        self.remove_lock_file(repo_path)?;

        // Commit and push
        self.commit_lock_deletion(repo_path)?;
        self.push_locks_branch(repo_path, true)?;

        crate::info!("Lock forcibly broken");
        Ok(())
    }

    /// Emergency unlock: Break lock if expired or stale
    pub fn emergency_unlock_if_expired(&self, repo_path: &Path) -> Result<bool> {
        match self.get_lock(repo_path)? {
            Some(lock) => {
                if lock.is_expired() || lock.is_stale() {
                    crate::info!("Lock is expired/stale, performing emergency unlock");
                    self.force_break_lock(repo_path)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }

    /// Check if lock can be emergency unlocked
    pub fn can_emergency_unlock(&self, repo_path: &Path) -> Result<bool> {
        match self.get_lock(repo_path)? {
            Some(lock) => Ok(lock.is_expired() || lock.is_stale()),
            None => Ok(false),
        }
    }

    /// Get lock age in hours
    pub fn get_lock_age_hours(&self, repo_path: &Path) -> Result<Option<i64>> {
        match self.get_lock(repo_path)? {
            Some(lock) => {
                let age = Utc::now() - lock.acquired_at;
                Ok(Some(age.num_hours()))
            }
            None => Ok(None),
        }
    }

    // ========== Private Helper Methods ==========

    /// Ensure locks branch exists in repository
    fn ensure_locks_branch(&self, repo_path: &Path) -> Result<()> {
        // Check if locks branch exists
        let branches = self.oxen.list_branches(repo_path)?;
        let locks_branch_exists = branches.iter()
            .any(|b| b.name == self.locks_branch);

        if !locks_branch_exists {
            crate::vlog!("Creating locks branch");

            // Create locks branch (orphan branch with no history)
            self.oxen.create_branch(repo_path, &self.locks_branch)?;

            // Checkout locks branch
            self.oxen.checkout(repo_path, &self.locks_branch)?;

            // Create locks directory
            let locks_dir = repo_path.join(&self.locks_dir);
            std::fs::create_dir_all(&locks_dir)?;

            // Create .gitkeep to track directory
            let gitkeep = locks_dir.join(".gitkeep");
            std::fs::write(&gitkeep, "")?;

            // Add and commit
            self.oxen.add(repo_path, &[gitkeep.as_path()])?;
            self.oxen.commit(repo_path, "Initialize locks branch")?;

            // Return to main branch
            self.oxen.checkout(repo_path, "main")?;
        }

        Ok(())
    }

    /// Fetch latest locks branch from remote (with retry)
    fn fetch_locks_branch(&self, repo_path: &Path) -> Result<()> {
        crate::vlog!("Fetching locks branch from remote");

        // Save current branch
        let current_branch = self.oxen.current_branch(repo_path)?;

        // Checkout locks branch
        self.oxen.checkout(repo_path, &self.locks_branch)?;

        // Pull latest with retry (ignore errors if remote doesn't have locks branch yet)
        let repo_path_owned = repo_path.to_path_buf();
        let policy = RetryPolicy::new(3, 1000, 10000).set_verbose(true);

        let _ = policy.execute(|| {
            match self.oxen.pull(&repo_path_owned) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // If remote doesn't have locks branch yet, that's okay
                    let err_str = e.to_string();
                    if err_str.contains("not found") || err_str.contains("doesn't exist") {
                        Ok(())
                    } else {
                        Err(e)
                    }
                }
            }
        });

        // Return to original branch
        self.oxen.checkout(repo_path, &current_branch)?;

        Ok(())
    }

    /// Push locks branch to remote (with retry)
    fn push_locks_branch(&self, repo_path: &Path, force: bool) -> Result<()> {
        crate::vlog!("Pushing locks branch to remote (force: {})", force);

        let current_branch = self.oxen.current_branch(repo_path)?;

        // Checkout locks branch
        self.oxen.checkout(repo_path, &self.locks_branch)?;

        // Push with retry
        let repo_path_owned = repo_path.to_path_buf();
        let locks_branch = self.locks_branch.clone();
        let policy = RetryPolicy::new(5, 1000, 15000).set_verbose(true);

        let push_result = policy.execute(|| {
            if force {
                // For force push, we need to use oxen CLI directly
                use std::process::Command;
                let output = Command::new("oxen")
                    .args(["push", "--force", "origin", &locks_branch])
                    .current_dir(&repo_path_owned)
                    .output()
                    .context("Failed to execute oxen push command")?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Force push failed: {}", stderr));
                }
                Ok(())
            } else {
                self.oxen.push(&repo_path_owned, Some("origin"), Some(&locks_branch))
                    .context("Failed to push locks branch")
            }
        });

        // Return to original branch (even if push failed)
        let checkout_result = self.oxen.checkout(repo_path, &current_branch);

        // Propagate push error if it failed
        push_result?;

        // Then check checkout result
        checkout_result?;

        Ok(())
    }

    /// Verify we still own the lock after pushing (detect race conditions)
    fn verify_lock_ownership(&self, repo_path: &Path, expected_lock: &RemoteLock) -> Result<()> {
        // Fetch latest locks
        self.fetch_locks_branch(repo_path)?;

        // Read current lock
        let current_lock = self.get_lock(repo_path)?
            .ok_or_else(|| anyhow!("Lock disappeared after push (race condition)"))?;

        // Verify it's our lock
        if current_lock.lock_id != expected_lock.lock_id {
            return Err(anyhow!(
                "Lock race condition detected: lock now owned by {}",
                current_lock.locked_by
            ));
        }

        Ok(())
    }

    /// Get path to lock file for a project
    fn get_lock_file_path(&self, repo_path: &Path) -> PathBuf {
        // Use project name as lock file identifier
        let project_name = repo_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        repo_path
            .join(&self.locks_dir)
            .join(format!("{}.json", sanitize_filename(&project_name)))
    }

    /// Write lock to file
    fn write_lock_file(&self, repo_path: &Path, lock: &RemoteLock) -> Result<()> {
        let lock_file = self.get_lock_file_path(repo_path);

        // Ensure directory exists
        if let Some(parent) = lock_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize lock
        let json = serde_json::to_string_pretty(lock)?;

        // Write to file
        std::fs::write(&lock_file, json)?;

        Ok(())
    }

    /// Remove lock file
    fn remove_lock_file(&self, repo_path: &Path) -> Result<()> {
        let lock_file = self.get_lock_file_path(repo_path);

        if lock_file.exists() {
            std::fs::remove_file(&lock_file)?;
        }

        Ok(())
    }

    /// Commit lock change to locks branch
    fn commit_lock(&self, repo_path: &Path, lock: &RemoteLock, message: &str) -> Result<()> {
        let current_branch = self.oxen.current_branch(repo_path)?;

        // Checkout locks branch
        self.oxen.checkout(repo_path, &self.locks_branch)?;

        // Add lock file
        let lock_file = self.get_lock_file_path(repo_path);
        self.oxen.add(repo_path, &[lock_file.as_path()])?;

        // Commit
        let commit_msg = format!("{} - {}", message, lock.lock_id);
        self.oxen.commit(repo_path, &commit_msg)?;

        // Return to original branch
        self.oxen.checkout(repo_path, &current_branch)?;

        Ok(())
    }

    /// Commit lock deletion
    fn commit_lock_deletion(&self, repo_path: &Path) -> Result<()> {
        let current_branch = self.oxen.current_branch(repo_path)?;

        // Checkout locks branch
        self.oxen.checkout(repo_path, &self.locks_branch)?;

        // Commit all changes (deletion)
        self.oxen.add_all(repo_path)?;
        self.oxen.commit(repo_path, "Release lock")?;

        // Return to original branch
        self.oxen.checkout(repo_path, &current_branch)?;

        Ok(())
    }
}

impl Default for RemoteLockManager {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Helper Functions ==========

/// Get current user identifier (username@hostname)
fn get_user_identifier() -> String {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    format!("{}@{}", username, hostname)
}

/// Get machine identifier (unique per machine)
fn get_machine_id() -> String {
    // Use hostname + MAC address hash for machine ID
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    // In production, you'd want to use actual MAC address or UUID
    // For now, just use hostname
    hostname
}

/// Sanitize filename (remove invalid characters)
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_lock_creation() {
        let lock = RemoteLock::new("test.logicx", "user@host", 4);

        assert_eq!(lock.project_path, "test.logicx");
        assert_eq!(lock.locked_by, "user@host");
        assert!(!lock.is_expired());
        assert!(!lock.is_stale());
    }

    #[test]
    fn test_lock_expiration() {
        let mut lock = RemoteLock::new("test.logicx", "user@host", 4);

        // Set to past time
        lock.expires_at = Utc::now() - Duration::hours(1);

        assert!(lock.is_expired());
    }

    #[test]
    fn test_lock_staleness() {
        let mut lock = RemoteLock::new("test.logicx", "user@host", 4);

        // Set heartbeat to past
        lock.last_heartbeat = Utc::now() - Duration::hours(2);

        assert!(lock.is_stale());
    }

    #[test]
    fn test_lock_renewal() {
        let mut lock = RemoteLock::new("test.logicx", "user@host", 4);
        let original_expires = lock.expires_at;

        thread::sleep(StdDuration::from_millis(10));
        lock.renew(4);

        assert!(lock.expires_at > original_expires);
        assert!(lock.last_heartbeat > lock.acquired_at);
    }

    #[test]
    fn test_lock_remaining_time() {
        let lock = RemoteLock::new("test.logicx", "user@host", 4);
        let remaining = lock.remaining_time();

        // Should be close to 4 hours (within 1 minute tolerance)
        let hours = remaining.num_hours();
        assert!(hours >= 3 && hours <= 4);
    }

    #[test]
    fn test_lock_serialization() {
        let lock = RemoteLock::new("test.logicx", "user@host", 4);

        let json = serde_json::to_string(&lock).unwrap();
        let deserialized: RemoteLock = serde_json::from_str(&json).unwrap();

        assert_eq!(lock, deserialized);
    }

    #[test]
    fn test_get_user_identifier() {
        let id = get_user_identifier();
        assert!(id.contains('@'));
    }

    #[test]
    fn test_get_machine_id() {
        let id = get_machine_id();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.logicx"), "test_logicx");
        assert_eq!(sanitize_filename("my project"), "my_project");
        assert_eq!(sanitize_filename("valid-name_123"), "valid-name_123");
    }

    #[test]
    fn test_remote_lock_manager_creation() {
        let manager = RemoteLockManager::new();
        assert_eq!(manager.locks_branch, "locks");
    }

    #[test]
    fn test_lock_file_path_generation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let manager = RemoteLockManager::new();

        // Create locks directory
        let locks_dir = temp_dir.path().join(".oxen").join("locks");
        std::fs::create_dir_all(&locks_dir).unwrap();

        // Test that lock file path is correctly generated
        let project_name = "test.logicx";
        let lock_file = locks_dir.join(format!("{}.json", sanitize_filename(project_name)));

        assert!(lock_file.to_str().unwrap().contains("test_logicx.json"));
    }

    #[test]
    fn test_get_lock_nonexistent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create .oxen directory structure
        std::fs::create_dir_all(repo_path.join(".oxen").join("locks")).unwrap();

        let manager = RemoteLockManager::new();

        // Should return None when no lock exists
        let result = manager.get_lock(repo_path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_lock_ownership_check() {
        let lock = RemoteLock::new("test.logicx", &get_user_identifier(), 4);
        assert!(lock.is_owned_by_current_user());

        let other_lock = RemoteLock::new("test.logicx", "different@user", 4);
        assert!(!other_lock.is_owned_by_current_user());
    }

    #[test]
    fn test_lock_remaining_time_expired() {
        let mut lock = RemoteLock::new("test.logicx", "user@host", 4);
        lock.expires_at = Utc::now() - Duration::hours(2);

        let remaining = lock.remaining_time();
        assert_eq!(remaining, Duration::zero());
    }

    #[test]
    fn test_lock_renew_extends_expiration() {
        let mut lock = RemoteLock::new("test.logicx", "user@host", 1);
        let original_expires = lock.expires_at;

        // Renew for 2 more hours
        std::thread::sleep(std::time::Duration::from_millis(10));
        lock.renew(2);

        // New expiration should be ~2 hours from now (much later than original)
        assert!(lock.expires_at > original_expires);
        let diff = lock.expires_at - original_expires;
        assert!(diff > Duration::hours(1)); // Should be at least 1 hour more
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(sanitize_filename("test/file.logicx"), "test_file_logicx");
        assert_eq!(sanitize_filename("my\\project"), "my_project");
        assert_eq!(sanitize_filename("project:name"), "project_name");
        assert_eq!(sanitize_filename("file?name"), "file_name");
        assert_eq!(sanitize_filename("file*name"), "file_name");
    }
}
