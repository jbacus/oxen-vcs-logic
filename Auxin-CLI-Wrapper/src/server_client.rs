//! HTTP client for auxin-server integration
//!
//! This module provides a client for communicating with auxin-server
//! for repository management, locks, and metadata operations.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for server connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server URL (e.g., "http://localhost:3000")
    pub url: String,

    /// Optional authentication token
    pub token: Option<String>,

    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3000".to_string(),
            token: None,
            timeout_secs: 30,
        }
    }
}

/// Repository information from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub namespace: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
}

/// Commit information from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub author: Option<String>,
    pub timestamp: String,
    pub parent_ids: Option<Vec<String>>,
}

/// Branch information from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit_id: String,
    pub is_head: Option<bool>,
}

/// Lock information from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub locked: bool,
    pub lock: Option<LockHolder>,
}

/// Lock holder details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockHolder {
    pub lock_id: String,
    pub user: String,
    pub machine_id: String,
    pub acquired_at: String,
    pub expires_at: String,
    pub last_heartbeat: String,
}

/// Lock acquire request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockAcquireRequest {
    pub user: String,
    pub machine_id: String,
    pub timeout_hours: u32,
}

/// Lock release request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockReleaseRequest {
    pub lock_id: String,
    pub user: String,
    pub machine_id: String,
}

/// Logic Pro metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicProMetadata {
    pub bpm: Option<f64>,
    pub sample_rate: Option<u32>,
    pub key_signature: Option<String>,
    pub tags: Option<Vec<String>>,
    pub custom: Option<serde_json::Value>,
}

/// Create repository request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepoRequest {
    pub description: Option<String>,
}

/// HTTP client for auxin-server
pub struct AuxinServerClient {
    agent: ureq::Agent,
    config: ServerConfig,
}

impl AuxinServerClient {
    /// Create a new client with the given configuration
    pub fn new(config: ServerConfig) -> Result<Self> {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(config.timeout_secs))
            .timeout_write(Duration::from_secs(config.timeout_secs))
            .user_agent("auxin-cli/0.2.0")
            .build();

        Ok(Self { agent, config })
    }

    /// Create a client with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(ServerConfig::default())
    }

    /// Create a client for a specific server URL
    pub fn with_url(url: &str) -> Result<Self> {
        let config = ServerConfig {
            url: url.to_string(),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Get the base URL for API requests
    fn api_url(&self, path: &str) -> String {
        format!("{}/api{}", self.config.url.trim_end_matches('/'), path)
    }

    /// Make a GET request with optional auth
    fn get(&self, url: &str) -> ureq::Request {
        let req = self.agent.get(url);
        if let Some(ref token) = self.config.token {
            req.set("Authorization", &format!("Bearer {}", token))
        } else {
            req
        }
    }

    /// Make a POST request with optional auth
    fn post(&self, url: &str) -> ureq::Request {
        let req = self.agent.post(url);
        if let Some(ref token) = self.config.token {
            req.set("Authorization", &format!("Bearer {}", token))
        } else {
            req
        }
    }

    // ========== Health & Status ==========

    /// Check server health
    pub fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.config.url.trim_end_matches('/'));
        match self.agent.get(&url).call() {
            Ok(resp) => Ok(resp.status() == 200),
            Err(_) => Ok(false),
        }
    }

    // ========== Repository Operations ==========

    /// List all repositories
    pub fn list_repositories(&self) -> Result<Vec<Repository>> {
        let url = self.api_url("/repos");
        let response = self.get(&url).call()
            .map_err(|e| anyhow!("Failed to list repositories: {}", e))?;

        response.into_json()
            .context("Failed to parse repository list")
    }

    /// Get a specific repository
    pub fn get_repository(&self, namespace: &str, name: &str) -> Result<Repository> {
        let url = self.api_url(&format!("/repos/{}/{}", namespace, name));
        let response = self.get(&url).call()
            .map_err(|e| anyhow!("Failed to get repository: {}", e))?;

        response.into_json()
            .context("Failed to parse repository")
    }

    /// Create a new repository
    pub fn create_repository(&self, namespace: &str, name: &str, description: Option<&str>) -> Result<Repository> {
        let url = self.api_url(&format!("/repos/{}/{}", namespace, name));
        let body = CreateRepoRequest {
            description: description.map(|s| s.to_string()),
        };

        let response = self.post(&url)
            .send_json(&body)
            .map_err(|e| anyhow!("Failed to create repository: {}", e))?;

        response.into_json()
            .context("Failed to parse created repository")
    }

    // ========== Commit Operations ==========

    /// Get commits for a repository
    pub fn get_commits(&self, namespace: &str, name: &str) -> Result<Vec<Commit>> {
        let url = self.api_url(&format!("/repos/{}/{}/commits", namespace, name));
        match self.get(&url).call() {
            Ok(response) => {
                response.into_json()
                    .context("Failed to parse commits")
            }
            Err(ureq::Error::Status(501, _)) => {
                // VCS not implemented in mock mode
                Ok(vec![])
            }
            Err(e) => Err(anyhow!("Failed to get commits: {}", e)),
        }
    }

    // ========== Branch Operations ==========

    /// Get branches for a repository
    pub fn get_branches(&self, namespace: &str, name: &str) -> Result<Vec<Branch>> {
        let url = self.api_url(&format!("/repos/{}/{}/branches", namespace, name));
        match self.get(&url).call() {
            Ok(response) => {
                response.into_json()
                    .context("Failed to parse branches")
            }
            Err(ureq::Error::Status(501, _)) => {
                // VCS not implemented in mock mode
                Ok(vec![])
            }
            Err(e) => Err(anyhow!("Failed to get branches: {}", e)),
        }
    }

    // ========== Lock Operations ==========

    /// Get lock status for a repository
    pub fn get_lock_status(&self, namespace: &str, name: &str) -> Result<LockInfo> {
        let url = self.api_url(&format!("/repos/{}/{}/locks/status", namespace, name));
        let response = self.get(&url).call()
            .map_err(|e| anyhow!("Failed to get lock status: {}", e))?;

        response.into_json()
            .context("Failed to parse lock status")
    }

    /// Acquire a lock on a repository
    pub fn acquire_lock(&self, namespace: &str, name: &str, user: &str, machine_id: &str, timeout_hours: u32) -> Result<LockHolder> {
        let url = self.api_url(&format!("/repos/{}/{}/locks/acquire", namespace, name));
        let body = LockAcquireRequest {
            user: user.to_string(),
            machine_id: machine_id.to_string(),
            timeout_hours,
        };

        let response = self.post(&url)
            .send_json(&body)
            .map_err(|e| anyhow!("Failed to acquire lock: {}", e))?;

        response.into_json()
            .context("Failed to parse lock response")
    }

    /// Release a lock on a repository
    pub fn release_lock(&self, namespace: &str, name: &str, lock_id: &str, user: &str, machine_id: &str) -> Result<()> {
        let url = self.api_url(&format!("/repos/{}/{}/locks/release", namespace, name));
        let body = LockReleaseRequest {
            lock_id: lock_id.to_string(),
            user: user.to_string(),
            machine_id: machine_id.to_string(),
        };

        self.post(&url)
            .send_json(&body)
            .map_err(|e| anyhow!("Failed to release lock: {}", e))?;

        Ok(())
    }

    /// Send heartbeat to extend lock
    pub fn heartbeat_lock(&self, namespace: &str, name: &str) -> Result<()> {
        let url = self.api_url(&format!("/repos/{}/{}/locks/heartbeat", namespace, name));
        self.post(&url).call()
            .map_err(|e| anyhow!("Failed to send heartbeat: {}", e))?;

        Ok(())
    }

    // ========== Metadata Operations ==========

    /// Get metadata for a commit
    pub fn get_metadata(&self, namespace: &str, name: &str, commit_id: &str) -> Result<LogicProMetadata> {
        let url = self.api_url(&format!("/repos/{}/{}/metadata/{}", namespace, name, commit_id));
        let response = self.get(&url).call()
            .map_err(|e| anyhow!("Failed to get metadata: {}", e))?;

        response.into_json()
            .context("Failed to parse metadata")
    }

    /// Store metadata for a commit
    pub fn store_metadata(&self, namespace: &str, name: &str, commit_id: &str, metadata: &LogicProMetadata) -> Result<()> {
        let url = self.api_url(&format!("/repos/{}/{}/metadata/{}", namespace, name, commit_id));
        self.post(&url)
            .send_json(metadata)
            .map_err(|e| anyhow!("Failed to store metadata: {}", e))?;

        Ok(())
    }
}

/// Get current user identifier
pub fn get_user_identifier() -> String {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    format!("{}@{}", username, hostname)
}

/// Get machine identifier
pub fn get_machine_id() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.url, "http://localhost:3000");
        assert_eq!(config.timeout_secs, 30);
        assert!(config.token.is_none());
    }

    #[test]
    fn test_api_url_generation() {
        let client = AuxinServerClient::with_url("http://localhost:3000").unwrap();
        assert_eq!(client.api_url("/repos"), "http://localhost:3000/api/repos");

        let client = AuxinServerClient::with_url("http://localhost:3000/").unwrap();
        assert_eq!(client.api_url("/repos"), "http://localhost:3000/api/repos");
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
    fn test_lock_request_serialization() {
        let request = LockAcquireRequest {
            user: "john@laptop".to_string(),
            machine_id: "laptop".to_string(),
            timeout_hours: 8,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("john@laptop"));
        assert!(json.contains("timeout_hours"));
    }
}
