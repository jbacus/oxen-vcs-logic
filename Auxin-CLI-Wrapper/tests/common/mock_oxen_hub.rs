// Mock Oxen Hub for integration testing without real network access
//
// This module provides a mock implementation of Oxen Hub that can be used
// for testing collaboration features locally without network connectivity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Mock Oxen Hub state
#[derive(Debug, Clone)]
pub struct MockOxenHub {
    /// Registered users
    users: Arc<Mutex<HashMap<String, MockUser>>>,

    /// Repositories
    repositories: Arc<Mutex<HashMap<String, MockRepository>>>,

    /// Active locks
    locks: Arc<Mutex<HashMap<String, MockLock>>>,

    /// Authentication tokens
    tokens: Arc<Mutex<HashMap<String, String>>>, // token -> username
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockUser {
    pub username: String,
    pub api_key: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRepository {
    pub name: String,
    pub owner: String,
    pub is_private: bool,
    pub commits: Vec<MockCommit>,
    pub branches: HashMap<String, String>, // branch_name -> commit_hash
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockCommit {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockLock {
    pub lock_id: String,
    pub project_path: String,
    pub locked_by: String,
    pub machine_id: String,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

impl Default for MockOxenHub {
    fn default() -> Self {
        Self::new()
    }
}

impl MockOxenHub {
    /// Create a new mock Oxen Hub
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            repositories: Arc::new(Mutex::new(HashMap::new())),
            locks: Arc::new(Mutex::new(HashMap::new())),
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // =========================================================================
    // User Management
    // =========================================================================

    /// Register a test user
    pub fn register_user(&self, username: &str, api_key: &str, email: &str) {
        let user = MockUser {
            username: username.to_string(),
            api_key: api_key.to_string(),
            email: email.to_string(),
        };

        self.users
            .lock()
            .unwrap()
            .insert(username.to_string(), user);
    }

    /// Authenticate user and return token
    pub fn authenticate(&self, username: &str, api_key: &str) -> Result<String, String> {
        let users = self.users.lock().unwrap();

        if let Some(user) = users.get(username) {
            if user.api_key == api_key {
                let token = format!("token_{}", uuid::Uuid::new_v4());
                drop(users); // Release lock before acquiring another

                self.tokens
                    .lock()
                    .unwrap()
                    .insert(token.clone(), username.to_string());
                return Ok(token);
            }
        }

        Err("Invalid credentials".to_string())
    }

    /// Verify token is valid
    pub fn verify_token(&self, token: &str) -> Result<String, String> {
        self.tokens
            .lock()
            .unwrap()
            .get(token)
            .cloned()
            .ok_or_else(|| "Invalid token".to_string())
    }

    /// Revoke authentication token
    pub fn revoke_token(&self, token: &str) {
        self.tokens.lock().unwrap().remove(token);
    }

    // =========================================================================
    // Repository Management
    // =========================================================================

    /// Create a new repository
    pub fn create_repository(&self, name: &str, owner: &str, is_private: bool) {
        let repo = MockRepository {
            name: name.to_string(),
            owner: owner.to_string(),
            is_private,
            commits: Vec::new(),
            branches: HashMap::new(),
        };

        let repo_key = format!("{}/{}", owner, name);
        self.repositories.lock().unwrap().insert(repo_key, repo);
    }

    /// Add a commit to a repository
    pub fn add_commit(
        &self,
        repo_name: &str,
        author: &str,
        message: &str,
        metadata: HashMap<String, String>,
    ) -> String {
        let commit_hash = format!(
            "commit_{}",
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        let commit = MockCommit {
            hash: commit_hash.clone(),
            author: author.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            metadata,
        };

        let mut repos = self.repositories.lock().unwrap();
        if let Some(repo) = repos.get_mut(repo_name) {
            repo.commits.push(commit);
        }

        commit_hash
    }

    /// Get repository commits
    pub fn get_commits(&self, repo_name: &str) -> Vec<MockCommit> {
        self.repositories
            .lock()
            .unwrap()
            .get(repo_name)
            .map(|repo| repo.commits.clone())
            .unwrap_or_default()
    }

    // =========================================================================
    // Lock Management
    // =========================================================================

    /// Acquire a lock
    pub fn acquire_lock(
        &self,
        project_path: &str,
        locked_by: &str,
        machine_id: &str,
        timeout_hours: u32,
    ) -> Result<MockLock, String> {
        let mut locks = self.locks.lock().unwrap();

        // Check if lock already exists
        if let Some(existing_lock) = locks.get(project_path) {
            if Utc::now() < existing_lock.expires_at {
                return Err(format!(
                    "Project locked by {} until {}",
                    existing_lock.locked_by, existing_lock.expires_at
                ));
            }
        }

        // Create new lock
        let now = Utc::now();
        let lock = MockLock {
            lock_id: uuid::Uuid::new_v4().to_string(),
            project_path: project_path.to_string(),
            locked_by: locked_by.to_string(),
            machine_id: machine_id.to_string(),
            acquired_at: now,
            expires_at: now + chrono::Duration::hours(timeout_hours as i64),
            last_heartbeat: now,
        };

        locks.insert(project_path.to_string(), lock.clone());
        Ok(lock)
    }

    /// Release a lock
    pub fn release_lock(&self, project_path: &str, lock_id: &str) -> Result<(), String> {
        let mut locks = self.locks.lock().unwrap();

        if let Some(lock) = locks.get(project_path) {
            if lock.lock_id != lock_id {
                return Err("Lock ID mismatch".to_string());
            }
            locks.remove(project_path);
            Ok(())
        } else {
            Err("No lock exists for this project".to_string())
        }
    }

    /// Get lock status
    pub fn get_lock(&self, project_path: &str) -> Option<MockLock> {
        self.locks.lock().unwrap().get(project_path).cloned()
    }

    /// Renew a lock (update heartbeat and expiration)
    pub fn renew_lock(
        &self,
        project_path: &str,
        lock_id: &str,
        additional_hours: u32,
    ) -> Result<MockLock, String> {
        let mut locks = self.locks.lock().unwrap();

        if let Some(lock) = locks.get_mut(project_path) {
            if lock.lock_id != lock_id {
                return Err("Lock ID mismatch".to_string());
            }

            let now = Utc::now();
            lock.last_heartbeat = now;
            lock.expires_at = now + chrono::Duration::hours(additional_hours as i64);

            Ok(lock.clone())
        } else {
            Err("No lock exists for this project".to_string())
        }
    }

    /// Force break a lock
    pub fn force_break_lock(&self, project_path: &str) -> Result<(), String> {
        let mut locks = self.locks.lock().unwrap();

        if locks.remove(project_path).is_some() {
            Ok(())
        } else {
            Err("No lock exists for this project".to_string())
        }
    }

    // =========================================================================
    // Testing Utilities
    // =========================================================================

    /// Simulate network delay (for testing)
    pub fn simulate_delay(&self, milliseconds: u64) {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
    }

    /// Simulate network failure
    pub fn simulate_network_failure(&self) -> Result<(), String> {
        Err("Network connection failed".to_string())
    }

    /// Get all active locks (for debugging)
    pub fn get_all_locks(&self) -> Vec<MockLock> {
        self.locks.lock().unwrap().values().cloned().collect()
    }

    /// Clear all locks (for test cleanup)
    pub fn clear_all_locks(&self) {
        self.locks.lock().unwrap().clear();
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        self.users.lock().unwrap().len()
    }

    /// Get repository count
    pub fn repository_count(&self) -> usize {
        self.repositories.lock().unwrap().len()
    }
}

// =============================================================================
// Test Helpers
// =============================================================================

/// Create a test mock hub with default test data
pub fn create_test_hub() -> MockOxenHub {
    let hub = MockOxenHub::new();

    // Register test users
    hub.register_user("testuser1", "test_api_key_1", "user1@example.com");
    hub.register_user("testuser2", "test_api_key_2", "user2@example.com");
    hub.register_user("producer", "producer_key", "producer@studio.com");
    hub.register_user("mixer", "mixer_key", "mixer@studio.com");

    // Create test repository
    hub.create_repository("test-project", "testuser1", false);

    // Add some test commits
    let mut metadata1 = HashMap::new();
    metadata1.insert("bpm".to_string(), "120".to_string());
    metadata1.insert("key".to_string(), "C Major".to_string());
    hub.add_commit(
        "testuser1/test-project",
        "testuser1",
        "Initial commit",
        metadata1,
    );

    let mut metadata2 = HashMap::new();
    metadata2.insert("bpm".to_string(), "128".to_string());
    metadata2.insert("tags".to_string(), "drums,tracking".to_string());
    hub.add_commit(
        "testuser1/test-project",
        "testuser1",
        "Added drums",
        metadata2,
    );

    hub
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_hub_creation() {
        let hub = MockOxenHub::new();
        assert_eq!(hub.user_count(), 0);
        assert_eq!(hub.repository_count(), 0);
    }

    #[test]
    fn test_user_registration_and_auth() {
        let hub = MockOxenHub::new();

        // Register user
        hub.register_user("testuser", "test_key", "test@example.com");
        assert_eq!(hub.user_count(), 1);

        // Authenticate with valid credentials
        let token = hub
            .authenticate("testuser", "test_key")
            .expect("Auth should succeed");
        assert!(!token.is_empty());

        // Verify token
        let username = hub.verify_token(&token).expect("Token should be valid");
        assert_eq!(username, "testuser");

        // Authenticate with invalid credentials
        let result = hub.authenticate("testuser", "wrong_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_lock_acquire_release() {
        let hub = MockOxenHub::new();

        // Acquire lock
        let lock = hub
            .acquire_lock("project.logicx", "user1@machine1", "machine1", 4)
            .expect("Lock acquire should succeed");

        assert_eq!(lock.project_path, "project.logicx");
        assert_eq!(lock.locked_by, "user1@machine1");

        // Check lock status
        let status = hub.get_lock("project.logicx");
        assert!(status.is_some());
        assert_eq!(status.unwrap().lock_id, lock.lock_id);

        // Release lock
        hub.release_lock("project.logicx", &lock.lock_id)
            .expect("Lock release should succeed");

        // Verify lock removed
        let status = hub.get_lock("project.logicx");
        assert!(status.is_none());
    }

    #[test]
    fn test_lock_collision() {
        let hub = MockOxenHub::new();

        // User 1 acquires lock
        let lock1 = hub
            .acquire_lock("project.logicx", "user1@machine1", "machine1", 4)
            .expect("First lock should succeed");

        // User 2 tries to acquire (should fail)
        let result = hub.acquire_lock("project.logicx", "user2@machine2", "machine2", 4);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("locked by user1@machine1"));

        // User 1 releases
        hub.release_lock("project.logicx", &lock1.lock_id)
            .expect("Release should succeed");

        // User 2 can now acquire
        let lock2 = hub
            .acquire_lock("project.logicx", "user2@machine2", "machine2", 4)
            .expect("Second lock should succeed after release");

        assert_eq!(lock2.locked_by, "user2@machine2");
    }

    #[test]
    fn test_lock_renewal() {
        let hub = MockOxenHub::new();

        // Acquire lock
        let lock = hub
            .acquire_lock("project.logicx", "user1@machine1", "machine1", 2)
            .expect("Lock acquire should succeed");

        let original_expiration = lock.expires_at;

        // Renew lock
        let renewed = hub
            .renew_lock("project.logicx", &lock.lock_id, 4)
            .expect("Renewal should succeed");

        assert!(renewed.expires_at > original_expiration);
        assert!(renewed.last_heartbeat > lock.last_heartbeat);
    }

    #[test]
    fn test_force_break_lock() {
        let hub = MockOxenHub::new();

        // User 1 acquires lock
        hub.acquire_lock("project.logicx", "user1@machine1", "machine1", 4)
            .expect("Lock acquire should succeed");

        // Admin force breaks
        hub.force_break_lock("project.logicx")
            .expect("Force break should succeed");

        // Verify lock removed
        assert!(hub.get_lock("project.logicx").is_none());

        // Another user can now acquire
        hub.acquire_lock("project.logicx", "user2@machine2", "machine2", 4)
            .expect("Lock should be available after force break");
    }

    #[test]
    fn test_repository_commits() {
        let hub = create_test_hub();

        // Get commits
        let commits = hub.get_commits("testuser1/test-project");

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].message, "Initial commit");
        assert_eq!(commits[0].metadata.get("bpm"), Some(&"120".to_string()));
        assert_eq!(commits[1].message, "Added drums");
    }

    #[test]
    fn test_test_hub_creation() {
        let hub = create_test_hub();

        assert_eq!(hub.user_count(), 4);
        assert_eq!(hub.repository_count(), 1);

        let commits = hub.get_commits("testuser1/test-project");
        assert_eq!(commits.len(), 2);
    }
}
