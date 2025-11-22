use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// File-based distributed lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLock {
    pub lock_id: String,
    pub user: String,
    pub machine_id: String,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

impl FileLock {
    /// Acquire a lock for a repository
    pub fn acquire(
        repo_path: &Path,
        user: impl Into<String>,
        machine_id: impl Into<String>,
        timeout_hours: u64,
    ) -> Result<Self, std::io::Error> {
        let lock_path = repo_path.join(".oxen/locks/project.lock");

        // Check if lock already exists
        if lock_path.exists() {
            let existing = Self::read_from_file(&lock_path)?;

            // Check if lock is expired
            if existing.is_expired() {
                // Lock expired, can acquire
                fs::remove_file(&lock_path)?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!(
                        "Lock held by {} until {}",
                        existing.user, existing.expires_at
                    ),
                ));
            }
        }

        let now = Utc::now();
        let lock = Self {
            lock_id: uuid::Uuid::new_v4().to_string(),
            user: user.into(),
            machine_id: machine_id.into(),
            acquired_at: now,
            expires_at: now + Duration::hours(timeout_hours as i64),
            last_heartbeat: now,
        };

        lock.write_to_file(&lock_path)?;
        Ok(lock)
    }

    /// Release a lock
    pub fn release(repo_path: &Path, lock_id: &str) -> Result<(), std::io::Error> {
        let lock_path = repo_path.join(".oxen/locks/project.lock");

        if !lock_path.exists() {
            return Ok(()); // Already released
        }

        let existing = Self::read_from_file(&lock_path)?;

        if existing.lock_id != lock_id {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Cannot release lock owned by different user",
            ));
        }

        fs::remove_file(&lock_path)?;
        Ok(())
    }

    /// Update heartbeat for a lock
    pub fn heartbeat(repo_path: &Path, lock_id: &str) -> Result<Self, std::io::Error> {
        let lock_path = repo_path.join(".oxen/locks/project.lock");

        if !lock_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Lock not found",
            ));
        }

        let mut lock = Self::read_from_file(&lock_path)?;

        if lock.lock_id != lock_id {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Cannot update heartbeat for lock owned by different user",
            ));
        }

        lock.last_heartbeat = Utc::now();
        lock.write_to_file(&lock_path)?;

        Ok(lock)
    }

    /// Get current lock status
    pub fn status(repo_path: &Path) -> Result<Option<Self>, std::io::Error> {
        let lock_path = repo_path.join(".oxen/locks/project.lock");

        if !lock_path.exists() {
            return Ok(None);
        }

        let lock = Self::read_from_file(&lock_path)?;

        if lock.is_expired() {
            fs::remove_file(&lock_path)?;
            return Ok(None);
        }

        Ok(Some(lock))
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    fn read_from_file(path: &PathBuf) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    fn write_to_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_acquire_lock() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let lock = FileLock::acquire(repo_path, "user1", "machine1", 1).unwrap();

        assert_eq!(lock.user, "user1");
        assert_eq!(lock.machine_id, "machine1");
        assert!(!lock.is_expired());
    }

    #[test]
    fn test_acquire_lock_twice_fails() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        FileLock::acquire(repo_path, "user1", "machine1", 1).unwrap();

        // Try to acquire again
        let result = FileLock::acquire(repo_path, "user2", "machine2", 1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );
    }

    #[test]
    fn test_release_lock() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let lock = FileLock::acquire(repo_path, "user1", "machine1", 1).unwrap();
        FileLock::release(repo_path, &lock.lock_id).unwrap();

        // Lock should be released, can acquire again
        let lock2 = FileLock::acquire(repo_path, "user2", "machine2", 1).unwrap();
        assert_eq!(lock2.user, "user2");
    }

    #[test]
    fn test_release_wrong_lock_id_fails() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        FileLock::acquire(repo_path, "user1", "machine1", 1).unwrap();

        // Try to release with wrong lock ID
        let result = FileLock::release(repo_path, "wrong_id");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
    }

    #[test]
    fn test_release_nonexistent_lock() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Releasing non-existent lock should succeed
        let result = FileLock::release(repo_path, "any_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_heartbeat() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let lock = FileLock::acquire(repo_path, "user1", "machine1", 1).unwrap();
        let old_heartbeat = lock.last_heartbeat;

        // Sleep briefly to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        let updated = FileLock::heartbeat(repo_path, &lock.lock_id).unwrap();
        assert!(updated.last_heartbeat > old_heartbeat);
    }

    #[test]
    fn test_heartbeat_wrong_lock_id_fails() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        FileLock::acquire(repo_path, "user1", "machine1", 1).unwrap();

        let result = FileLock::heartbeat(repo_path, "wrong_id");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
    }

    #[test]
    fn test_heartbeat_nonexistent_lock_fails() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let result = FileLock::heartbeat(repo_path, "any_id");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_lock_status() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // No lock initially
        let status = FileLock::status(repo_path).unwrap();
        assert!(status.is_none());

        // Acquire lock
        let lock = FileLock::acquire(repo_path, "user1", "machine1", 1).unwrap();

        // Check status
        let status = FileLock::status(repo_path).unwrap();
        assert!(status.is_some());
        assert_eq!(status.unwrap().lock_id, lock.lock_id);

        // Release lock
        FileLock::release(repo_path, &lock.lock_id).unwrap();

        // Status should be None again
        let status = FileLock::status(repo_path).unwrap();
        assert!(status.is_none());
    }

    #[test]
    fn test_expired_lock_is_removed() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let lock_path = repo_path.join(".oxen/locks/project.lock");

        // Create an expired lock manually
        let now = Utc::now();
        let expired_lock = FileLock {
            lock_id: uuid::Uuid::new_v4().to_string(),
            user: "user1".to_string(),
            machine_id: "machine1".to_string(),
            acquired_at: now - Duration::hours(2),
            expires_at: now - Duration::hours(1), // Expired 1 hour ago
            last_heartbeat: now - Duration::hours(1),
        };

        expired_lock.write_to_file(&lock_path).unwrap();

        // Status check should remove expired lock
        let status = FileLock::status(repo_path).unwrap();
        assert!(status.is_none());

        // Should be able to acquire new lock
        let lock = FileLock::acquire(repo_path, "user2", "machine2", 1).unwrap();
        assert_eq!(lock.user, "user2");
    }

    #[test]
    fn test_lock_serialization() {
        let now = Utc::now();
        let lock = FileLock {
            lock_id: "test-id".to_string(),
            user: "testuser".to_string(),
            machine_id: "test-machine".to_string(),
            acquired_at: now,
            expires_at: now + Duration::hours(1),
            last_heartbeat: now,
        };

        let json = serde_json::to_string(&lock).unwrap();
        let deserialized: FileLock = serde_json::from_str(&json).unwrap();

        assert_eq!(lock.lock_id, deserialized.lock_id);
        assert_eq!(lock.user, deserialized.user);
        assert_eq!(lock.machine_id, deserialized.machine_id);
    }
}
