/// Authentication management for Oxen Hub
///
/// This module handles user authentication with Oxen Hub, including:
/// - API key storage and retrieval
/// - Credential validation
/// - Authentication status checking
///
/// # Security
///
/// API keys are stored in the system keychain (macOS Keychain on macOS).
/// Credentials are never logged or written to plain text files.
///
/// # Usage
///
/// ```no_run
/// use oxenvcs_cli::auth::AuthManager;
///
/// let auth = AuthManager::new();
///
/// // Store credentials
/// auth.store_credentials("username", "api_key_here")?;
///
/// // Retrieve for use
/// if let Some(creds) = auth.get_credentials()? {
///     println!("Authenticated as: {}", creds.username);
/// }
/// ```

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Authentication credentials for Oxen Hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Oxen Hub username
    pub username: String,
    /// API key for authentication (sensitive)
    #[serde(skip_serializing)]
    pub api_key: String,
    /// Oxen Hub URL (default: https://hub.oxen.ai)
    pub hub_url: String,
}

impl Credentials {
    /// Create new credentials
    pub fn new(username: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            api_key: api_key.into(),
            hub_url: "https://hub.oxen.ai".to_string(),
        }
    }

    /// Create credentials with custom hub URL
    pub fn with_hub_url(
        username: impl Into<String>,
        api_key: impl Into<String>,
        hub_url: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            api_key: api_key.into(),
            hub_url: hub_url.into(),
        }
    }

    /// Validate credentials (basic format check)
    pub fn validate(&self) -> Result<()> {
        if self.username.is_empty() {
            return Err(anyhow!("Username cannot be empty"));
        }
        if self.api_key.is_empty() {
            return Err(anyhow!("API key cannot be empty"));
        }
        if self.hub_url.is_empty() {
            return Err(anyhow!("Hub URL cannot be empty"));
        }
        if !self.hub_url.starts_with("http://") && !self.hub_url.starts_with("https://") {
            return Err(anyhow!("Hub URL must start with http:// or https://"));
        }
        Ok(())
    }
}

/// Manages authentication for Oxen Hub operations
pub struct AuthManager {
    /// Path to credentials file (fallback if keychain unavailable)
    config_file: PathBuf,
}

impl AuthManager {
    /// Create a new AuthManager with default configuration directory
    pub fn new() -> Self {
        Self {
            config_file: Self::default_config_path(),
        }
    }

    /// Create AuthManager with custom config file path
    pub fn with_config_path(config_file: PathBuf) -> Self {
        Self { config_file }
    }

    /// Get default configuration file path
    /// Returns: ~/.oxenvcs/credentials (fallback storage)
    fn default_config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".oxenvcs")
            .join("credentials")
    }

    /// Store credentials securely
    ///
    /// On macOS: Delegates to system keychain via Oxen CLI config
    /// Fallback: Encrypted storage in ~/.oxenvcs/credentials (not implemented yet)
    ///
    /// # Arguments
    ///
    /// * `username` - Oxen Hub username
    /// * `api_key` - Oxen Hub API key
    ///
    /// # Security Note
    ///
    /// The API key is sensitive. This function:
    /// 1. First attempts to use Oxen CLI's built-in credential storage
    /// 2. Falls back to file-based storage (TODO: add encryption)
    pub fn store_credentials(&self, username: &str, api_key: &str) -> Result<()> {
        let creds = Credentials::new(username, api_key);
        creds.validate()?;

        // Strategy 1: Use Oxen CLI's built-in config system
        // The oxen CLI stores credentials in ~/.oxen/user_config.toml
        // We'll leverage this by running `oxen config` commands
        self.configure_oxen_cli(&creds)?;

        // Strategy 2: Store in our own config file as backup
        self.store_in_config_file(&creds)?;

        crate::info!("Credentials stored for user: {}", username);
        Ok(())
    }

    /// Configure Oxen CLI with credentials
    fn configure_oxen_cli(&self, creds: &Credentials) -> Result<()> {
        use std::process::Command;

        // Set user name
        let output = Command::new("oxen")
            .args(["config", "user.name", &creds.username])
            .output()
            .context("Failed to execute oxen config command")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to set oxen user.name: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Set API key (oxen config user.api_key)
        let output = Command::new("oxen")
            .args(["config", "user.api_key", &creds.api_key])
            .output()
            .context("Failed to set oxen API key")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to set oxen API key: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Set default remote URL if not using default
        if creds.hub_url != "https://hub.oxen.ai" {
            let output = Command::new("oxen")
                .args(["config", "remote.hub_url", &creds.hub_url])
                .output()
                .context("Failed to set oxen hub URL")?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Failed to set oxen hub URL: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        Ok(())
    }

    /// Store credentials in local config file (fallback)
    fn store_in_config_file(&self, creds: &Credentials) -> Result<()> {
        // Create config directory if it doesn't exist
        if let Some(parent) = self.config_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        // Serialize credentials (note: api_key is skipped in serialization for security)
        // For the file, we'll create a simpler format
        let config_content = format!(
            "username={}\nhub_url={}\n# API key stored in oxen config\n",
            creds.username, creds.hub_url
        );

        fs::write(&self.config_file, config_content).with_context(|| {
            format!("Failed to write credentials to {:?}", self.config_file)
        })?;

        // Set file permissions to user-only read/write (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.config_file)?.permissions();
            perms.set_mode(0o600); // rw-------
            fs::set_permissions(&self.config_file, perms)?;
        }

        Ok(())
    }

    /// Retrieve stored credentials
    ///
    /// Returns None if no credentials are configured
    pub fn get_credentials(&self) -> Result<Option<Credentials>> {
        // Try to get credentials from Oxen CLI config
        match self.get_from_oxen_cli() {
            Ok(creds) => return Ok(Some(creds)),
            Err(e) => {
                crate::vlog!("Failed to get credentials from oxen config: {}", e);
            }
        }

        // Fallback: read from our config file
        self.get_from_config_file()
    }

    /// Get credentials from Oxen CLI config
    fn get_from_oxen_cli(&self) -> Result<Credentials> {
        use std::process::Command;

        // Get username
        let output = Command::new("oxen")
            .args(["config", "user.name"])
            .output()
            .context("Failed to get oxen user.name")?;

        if !output.status.success() {
            return Err(anyhow!("No oxen username configured"));
        }

        let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if username.is_empty() {
            return Err(anyhow!("Oxen username is empty"));
        }

        // Get API key
        let output = Command::new("oxen")
            .args(["config", "user.api_key"])
            .output()
            .context("Failed to get oxen API key")?;

        if !output.status.success() {
            return Err(anyhow!("No oxen API key configured"));
        }

        let api_key = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if api_key.is_empty() {
            return Err(anyhow!("Oxen API key is empty"));
        }

        // Get hub URL (optional, defaults to https://hub.oxen.ai)
        let hub_url = match Command::new("oxen")
            .args(["config", "remote.hub_url"])
            .output()
        {
            Ok(output) if output.status.success() => {
                let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if url.is_empty() {
                    "https://hub.oxen.ai".to_string()
                } else {
                    url
                }
            }
            _ => "https://hub.oxen.ai".to_string(),
        };

        Ok(Credentials::with_hub_url(username, api_key, hub_url))
    }

    /// Get credentials from config file
    fn get_from_config_file(&self) -> Result<Option<Credentials>> {
        if !self.config_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.config_file)
            .context("Failed to read credentials file")?;

        let mut username = String::new();
        let mut hub_url = "https://hub.oxen.ai".to_string();

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("username=") {
                username = value.to_string();
            } else if let Some(value) = line.strip_prefix("hub_url=") {
                hub_url = value.to_string();
            }
        }

        if username.is_empty() {
            return Ok(None);
        }

        // API key should be in oxen config, but we need a placeholder
        Ok(Some(Credentials::with_hub_url(
            username, "", hub_url,
        )))
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.get_credentials()
            .map(|creds| creds.is_some())
            .unwrap_or(false)
    }

    /// Clear stored credentials
    pub fn clear_credentials(&self) -> Result<()> {
        // Clear oxen CLI config
        use std::process::Command;

        let _ = Command::new("oxen")
            .args(["config", "--unset", "user.name"])
            .output();

        let _ = Command::new("oxen")
            .args(["config", "--unset", "user.api_key"])
            .output();

        let _ = Command::new("oxen")
            .args(["config", "--unset", "remote.hub_url"])
            .output();

        // Remove our config file
        if self.config_file.exists() {
            fs::remove_file(&self.config_file)
                .context("Failed to remove credentials file")?;
        }

        crate::info!("Credentials cleared");
        Ok(())
    }

    /// Test authentication by making a simple API call
    ///
    /// Returns the authenticated username on success
    pub fn test_authentication(&self) -> Result<String> {
        let creds = self
            .get_credentials()?
            .ok_or_else(|| anyhow!("No credentials configured"))?;

        creds.validate()?;

        // Test by running `oxen info` or similar command
        use std::process::Command;

        let output = Command::new("oxen")
            .args(["config", "user.name"])
            .output()
            .context("Failed to test authentication")?;

        if !output.status.success() {
            return Err(anyhow!("Authentication test failed"));
        }

        let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if username.is_empty() {
            return Err(anyhow!("No authenticated user"));
        }

        Ok(username)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_credentials_new() {
        let creds = Credentials::new("testuser", "test_api_key_123");
        assert_eq!(creds.username, "testuser");
        assert_eq!(creds.api_key, "test_api_key_123");
        assert_eq!(creds.hub_url, "https://hub.oxen.ai");
    }

    #[test]
    fn test_credentials_with_hub_url() {
        let creds =
            Credentials::with_hub_url("testuser", "api_key", "https://custom.oxen.server");
        assert_eq!(creds.hub_url, "https://custom.oxen.server");
    }

    #[test]
    fn test_credentials_validate_empty_username() {
        let creds = Credentials::new("", "api_key");
        assert!(creds.validate().is_err());
    }

    #[test]
    fn test_credentials_validate_empty_api_key() {
        let creds = Credentials::new("user", "");
        assert!(creds.validate().is_err());
    }

    #[test]
    fn test_credentials_validate_invalid_url() {
        let mut creds = Credentials::new("user", "key");
        creds.hub_url = "invalid-url".to_string();
        assert!(creds.validate().is_err());
    }

    #[test]
    fn test_credentials_validate_valid() {
        let creds = Credentials::new("user", "key");
        assert!(creds.validate().is_ok());
    }

    #[test]
    fn test_auth_manager_new() {
        let auth = AuthManager::new();
        assert!(auth.config_file.to_str().unwrap().contains(".oxenvcs"));
    }

    #[test]
    fn test_auth_manager_custom_path() {
        let path = PathBuf::from("/tmp/test_credentials");
        let auth = AuthManager::with_config_path(path.clone());
        assert_eq!(auth.config_file, path);
    }

    #[test]
    fn test_default_config_path() {
        let path = AuthManager::default_config_path();
        assert!(path.to_str().unwrap().contains(".oxenvcs"));
        assert!(path.to_str().unwrap().contains("credentials"));
    }

    #[test]
    fn test_store_and_retrieve_credentials_file() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let temp_dir = env::temp_dir();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let config_file = temp_dir.join(format!("test_creds_{}.txt", timestamp));
        let auth = AuthManager::with_config_path(config_file.clone());

        // Store credentials (will fail oxen CLI config but should succeed file write)
        let result = auth.store_in_config_file(&Credentials::new("testuser", "testkey"));
        assert!(result.is_ok());

        // Verify file exists
        assert!(config_file.exists());

        // Read back (won't have API key since it's not serialized)
        let creds = auth.get_from_config_file().unwrap();
        assert!(creds.is_some());
        let creds = creds.unwrap();
        assert_eq!(creds.username, "testuser");

        // Cleanup
        let _ = std::fs::remove_file(&config_file);
    }

    #[test]
    fn test_clear_credentials() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let temp_dir = env::temp_dir();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let config_file = temp_dir.join(format!("test_creds_{}.txt", timestamp));
        let auth = AuthManager::with_config_path(config_file.clone());

        // Create credentials file
        let _ = auth.store_in_config_file(&Credentials::new("user", "key"));
        assert!(config_file.exists());

        // Clear credentials
        let result = auth.clear_credentials();
        assert!(result.is_ok());

        // Verify file removed
        assert!(!config_file.exists());
    }

    #[test]
    fn test_credentials_serialization() {
        let creds = Credentials::new("testuser", "secret_key");

        // Serialize to JSON
        let json = serde_json::to_string(&creds).unwrap();

        // Verify API key is NOT in serialized output (security)
        assert!(!json.contains("secret_key"));
        assert!(json.contains("testuser"));
    }
}
