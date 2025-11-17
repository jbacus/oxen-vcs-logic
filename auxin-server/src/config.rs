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
            sync_dir: std::env::var("SYNC_DIR")
                .unwrap_or_else(|_| "/var/oxen/data".to_string()),
            host: std::env::var("OXEN_SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
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
