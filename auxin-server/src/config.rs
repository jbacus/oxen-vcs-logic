use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub sync_dir: String,
    pub host: String,
    pub port: u16,
    pub auth_token_secret: String,
    pub auth_token_expiry_hours: u64,
    pub enable_redis_locks: bool,
    pub enable_web_ui: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            sync_dir: std::env::var("SYNC_DIR").unwrap_or_else(|_| "/var/oxen/data".to_string()),
            host: std::env::var("OXEN_SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("OXEN_SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("Invalid OXEN_SERVER_PORT")?,
            auth_token_secret: std::env::var("AUTH_TOKEN_SECRET")
                .unwrap_or_else(|_| "dev_secret_change_in_production".to_string()),
            auth_token_expiry_hours: std::env::var("AUTH_TOKEN_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .context("Invalid AUTH_TOKEN_EXPIRY_HOURS")?,
            enable_redis_locks: std::env::var("ENABLE_REDIS_LOCKS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            enable_web_ui: std::env::var("ENABLE_WEB_UI")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            redis_url: std::env::var("REDIS_URL").ok(),
            database_url: std::env::var("DATABASE_URL").ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Use mutex to serialize config tests (they share environment)
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn clear_all_env_vars() {
        env::remove_var("SYNC_DIR");
        env::remove_var("OXEN_SERVER_HOST");
        env::remove_var("OXEN_SERVER_PORT");
        env::remove_var("AUTH_TOKEN_SECRET");
        env::remove_var("AUTH_TOKEN_EXPIRY_HOURS");
        env::remove_var("ENABLE_REDIS_LOCKS");
        env::remove_var("ENABLE_WEB_UI");
        env::remove_var("REDIS_URL");
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_defaults() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_all_env_vars();

        let config = Config::from_env().unwrap();

        assert_eq!(config.sync_dir, "/var/oxen/data");
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.auth_token_secret, "dev_secret_change_in_production");
        assert_eq!(config.auth_token_expiry_hours, 24);
        assert!(!config.enable_redis_locks);
        assert!(!config.enable_web_ui);
        assert!(config.redis_url.is_none());
        assert!(config.database_url.is_none());
    }

    #[test]
    fn test_config_from_env() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_all_env_vars();

        env::set_var("SYNC_DIR", "/custom/dir");
        env::set_var("OXEN_SERVER_HOST", "127.0.0.1");
        env::set_var("OXEN_SERVER_PORT", "8080");
        env::set_var("AUTH_TOKEN_SECRET", "custom_secret");
        env::set_var("AUTH_TOKEN_EXPIRY_HOURS", "48");
        env::set_var("ENABLE_REDIS_LOCKS", "true");
        env::set_var("ENABLE_WEB_UI", "true");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("DATABASE_URL", "postgres://localhost/auxin");

        let config = Config::from_env().unwrap();

        assert_eq!(config.sync_dir, "/custom/dir");
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.auth_token_secret, "custom_secret");
        assert_eq!(config.auth_token_expiry_hours, 48);
        assert!(config.enable_redis_locks);
        assert!(config.enable_web_ui);
        assert_eq!(config.redis_url, Some("redis://localhost:6379".to_string()));
        assert_eq!(
            config.database_url,
            Some("postgres://localhost/auxin".to_string())
        );

        clear_all_env_vars();
    }

    #[test]
    fn test_invalid_port() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_all_env_vars();

        env::set_var("OXEN_SERVER_PORT", "invalid");

        let result = Config::from_env();
        assert!(result.is_err());

        clear_all_env_vars();
    }

    #[test]
    fn test_invalid_expiry_hours() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_all_env_vars();

        env::set_var("AUTH_TOKEN_EXPIRY_HOURS", "not_a_number");

        let result = Config::from_env();
        assert!(result.is_err());

        clear_all_env_vars();
    }

    #[test]
    fn test_boolean_parsing() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_all_env_vars();

        env::set_var("ENABLE_REDIS_LOCKS", "yes"); // Invalid boolean, should default to false
        env::set_var("ENABLE_WEB_UI", "1"); // Invalid boolean, should default to false

        let config = Config::from_env().unwrap();

        // Invalid boolean strings default to false
        assert!(!config.enable_redis_locks);
        assert!(!config.enable_web_ui);

        clear_all_env_vars();
    }
}
