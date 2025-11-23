//! Repository access control for managing user permissions
//!
//! This module handles granting and revoking access to repositories for specific users.
//! Particularly useful for Client users who need read-only access to specific repos.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::info;

use auxin_config::Config;
use crate::auth::UserRole;
use crate::error::{AppError, AppResult};

/// Repository access entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoAccess {
    pub user_id: String,
    pub namespace: String,
    pub repo_name: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
    pub granted_by: String,
}

/// Service for managing repository access control
#[derive(Debug, Clone)]
pub struct RepoAccessService {
    config: Config,
    // Map of "namespace/repo_name" -> Set of user_ids
    access_map: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

/// Request to grant repository access
#[derive(Debug, Deserialize)]
pub struct GrantAccessRequest {
    pub user_id: String,
}

impl RepoAccessService {
    pub fn new(config: Config) -> Self {
        let service = Self {
            config: config.clone(),
            access_map: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load access control from disk on startup
        if let Err(e) = service.load_access() {
            info!("No existing access control file or error loading: {}", e);
        }

        service
    }

    /// Get access control file path
    fn access_file_path(&self) -> PathBuf {
        PathBuf::from(&self.config.server.sync_dir)
            .join(".auxin")
            .join("repo_access.json")
    }

    /// Load access control from JSON file
    fn load_access(&self) -> AppResult<()> {
        let path = self.access_file_path();
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::Internal(format!("Failed to read access file: {}", e)))?;

        let access_list: Vec<RepoAccess> = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse access file: {}", e)))?;

        let mut access_map = self
            .access_map
            .write()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        for access in access_list {
            let key = format!("{}/{}", access.namespace, access.repo_name);
            access_map
                .entry(key)
                .or_insert_with(Vec::new)
                .push(access.user_id);
        }

        info!("Loaded {} repository access entries", access_map.len());
        Ok(())
    }

    /// Save access control to JSON file
    fn save_access(&self) -> AppResult<()> {
        let path = self.access_file_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Internal(format!("Failed to create directory: {}", e)))?;
        }

        let access_map = self
            .access_map
            .read()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        // Convert map to list of RepoAccess entries
        let mut access_list = Vec::new();
        for (repo_key, user_ids) in access_map.iter() {
            let parts: Vec<&str> = repo_key.split('/').collect();
            if parts.len() != 2 {
                continue;
            }
            for user_id in user_ids {
                access_list.push(RepoAccess {
                    user_id: user_id.clone(),
                    namespace: parts[0].to_string(),
                    repo_name: parts[1].to_string(),
                    granted_at: chrono::Utc::now(),
                    granted_by: "system".to_string(),
                });
            }
        }

        let content = serde_json::to_string_pretty(&access_list)
            .map_err(|e| AppError::Internal(format!("Failed to serialize access: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| AppError::Internal(format!("Failed to write access file: {}", e)))?;

        Ok(())
    }

    /// Grant access to a repository for a user
    pub fn grant_access(
        &self,
        namespace: &str,
        repo_name: &str,
        user_id: &str,
    ) -> AppResult<()> {
        let key = format!("{}/{}", namespace, repo_name);

        {
            let mut access_map = self
                .access_map
                .write()
                .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

            let users = access_map.entry(key.clone()).or_insert_with(Vec::new);

            if !users.contains(&user_id.to_string()) {
                users.push(user_id.to_string());
            }
        }

        self.save_access()?;

        info!("Granted access to {}/{} for user {}", namespace, repo_name, user_id);
        Ok(())
    }

    /// Revoke access to a repository for a user
    pub fn revoke_access(
        &self,
        namespace: &str,
        repo_name: &str,
        user_id: &str,
    ) -> AppResult<()> {
        let key = format!("{}/{}", namespace, repo_name);

        {
            let mut access_map = self
                .access_map
                .write()
                .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

            if let Some(users) = access_map.get_mut(&key) {
                users.retain(|id| id != user_id);
                if users.is_empty() {
                    access_map.remove(&key);
                }
            }
        }

        self.save_access()?;

        info!("Revoked access to {}/{} for user {}", namespace, repo_name, user_id);
        Ok(())
    }

    /// Check if a user has access to a repository
    /// Admin and Producer roles have access to all repos
    /// Client role needs explicit access grant
    pub fn has_access(
        &self,
        namespace: &str,
        repo_name: &str,
        user_id: &str,
        user_role: UserRole,
    ) -> AppResult<bool> {
        // Admin and Producer have access to all repos
        if matches!(user_role, UserRole::Admin | UserRole::Producer) {
            return Ok(true);
        }

        // Client needs explicit access
        let key = format!("{}/{}", namespace, repo_name);

        let access_map = self
            .access_map
            .read()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        Ok(access_map
            .get(&key)
            .map(|users| users.contains(&user_id.to_string()))
            .unwrap_or(false))
    }

    /// List all repositories a user has access to
    pub fn list_user_repos(&self, user_id: &str, user_role: UserRole) -> AppResult<Vec<String>> {
        // Admin and Producer can see all repos (handled elsewhere)
        if matches!(user_role, UserRole::Admin | UserRole::Producer) {
            return Ok(Vec::new()); // Return empty - caller will show all repos
        }

        let access_map = self
            .access_map
            .read()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        let repos: Vec<String> = access_map
            .iter()
            .filter(|(_, users)| users.contains(&user_id.to_string()))
            .map(|(repo_key, _)| repo_key.clone())
            .collect();

        Ok(repos)
    }

    /// List all users with access to a repository
    pub fn list_repo_users(&self, namespace: &str, repo_name: &str) -> AppResult<Vec<String>> {
        let key = format!("{}/{}", namespace, repo_name);

        let access_map = self
            .access_map
            .read()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        Ok(access_map.get(&key).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config_with_dir(dir: &TempDir) -> Config {
        let mut config = Config::default();
        config.server.sync_dir = dir.path().to_string_lossy().to_string();
        config
    }

    #[test]
    fn test_grant_access() {
        let temp_dir = TempDir::new().unwrap();
        let service = RepoAccessService::new(test_config_with_dir(&temp_dir));

        service.grant_access("namespace", "repo", "user123").unwrap();

        let has_access = service
            .has_access("namespace", "repo", "user123", UserRole::Client)
            .unwrap();
        assert!(has_access);
    }

    #[test]
    fn test_revoke_access() {
        let temp_dir = TempDir::new().unwrap();
        let service = RepoAccessService::new(test_config_with_dir(&temp_dir));

        service.grant_access("namespace", "repo", "user123").unwrap();
        service.revoke_access("namespace", "repo", "user123").unwrap();

        let has_access = service
            .has_access("namespace", "repo", "user123", UserRole::Client)
            .unwrap();
        assert!(!has_access);
    }

    #[test]
    fn test_admin_has_access_to_all() {
        let temp_dir = TempDir::new().unwrap();
        let service = RepoAccessService::new(test_config_with_dir(&temp_dir));

        // Admin should have access even without explicit grant
        let has_access = service
            .has_access("namespace", "repo", "admin123", UserRole::Admin)
            .unwrap();
        assert!(has_access);
    }

    #[test]
    fn test_producer_has_access_to_all() {
        let temp_dir = TempDir::new().unwrap();
        let service = RepoAccessService::new(test_config_with_dir(&temp_dir));

        // Producer should have access even without explicit grant
        let has_access = service
            .has_access("namespace", "repo", "producer123", UserRole::Producer)
            .unwrap();
        assert!(has_access);
    }

    #[test]
    fn test_list_user_repos() {
        let temp_dir = TempDir::new().unwrap();
        let service = RepoAccessService::new(test_config_with_dir(&temp_dir));

        service.grant_access("ns1", "repo1", "client123").unwrap();
        service.grant_access("ns2", "repo2", "client123").unwrap();

        let repos = service.list_user_repos("client123", UserRole::Client).unwrap();
        assert_eq!(repos.len(), 2);
        assert!(repos.contains(&"ns1/repo1".to_string()));
        assert!(repos.contains(&"ns2/repo2".to_string()));
    }
}
