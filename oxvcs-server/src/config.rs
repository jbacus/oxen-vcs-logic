use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub websocket_port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub s3_endpoint: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub max_upload_size: usize,
    pub deduplication_chunk_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            redis_url: std::env::var("REDIS_URL")
                .context("REDIS_URL must be set")?,
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("Invalid SERVER_PORT")?,
            websocket_port: std::env::var("WEBSOCKET_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .context("Invalid WEBSOCKET_PORT")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET must be set")?,
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .context("Invalid JWT_EXPIRATION_HOURS")?,
            s3_endpoint: std::env::var("S3_ENDPOINT")
                .context("S3_ENDPOINT must be set")?,
            s3_access_key: std::env::var("S3_ACCESS_KEY")
                .context("S3_ACCESS_KEY must be set")?,
            s3_secret_key: std::env::var("S3_SECRET_KEY")
                .context("S3_SECRET_KEY must be set")?,
            s3_bucket: std::env::var("S3_BUCKET")
                .context("S3_BUCKET must be set")?,
            s3_region: std::env::var("S3_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            max_upload_size: std::env::var("MAX_UPLOAD_SIZE")
                .unwrap_or_else(|_| "10737418240".to_string())
                .parse()
                .context("Invalid MAX_UPLOAD_SIZE")?,
            deduplication_chunk_size: std::env::var("DEDUPLICATION_CHUNK_SIZE")
                .unwrap_or_else(|_| "4194304".to_string())
                .parse()
                .context("Invalid DEDUPLICATION_CHUNK_SIZE")?,
        })
    }
}
