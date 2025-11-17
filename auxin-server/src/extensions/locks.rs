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
                    format!("Lock held by {} until {}", existing.user, existing.expires_at),
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
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })
    }

    fn write_to_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        fs::write(path, content)?;
        Ok(())
    }
}
