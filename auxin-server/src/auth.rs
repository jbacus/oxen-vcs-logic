use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::config::Config;
use crate::error::{AppError, AppResult};

/// Simple token-based authentication
/// In production, use proper JWT with signing or integrate with Oxen Hub
#[derive(Debug, Clone)]
pub struct AuthService {
    config: Config,
    // Simple in-memory token store (replace with Redis in production)
    tokens: Arc<RwLock<HashMap<String, TokenData>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenData {
    username: String,
    expires_at: chrono::DateTime<Utc>,
}

impl AuthService {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new token for a user
    pub fn generate_token(&self, username: &str) -> AppResult<String> {
        let token = format!("auxin_{}", uuid::Uuid::new_v4());
        let expires_at = Utc::now() + Duration::hours(self.config.auth_token_expiry_hours as i64);

        let token_data = TokenData {
            username: username.to_string(),
            expires_at,
        };

        self.tokens
            .write()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?
            .insert(token.clone(), token_data);

        Ok(token)
    }

    /// Validate a token and return the username
    pub fn validate_token(&self, token: &str) -> AppResult<String> {
        let tokens = self
            .tokens
            .read()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        let token_data = tokens
            .get(token)
            .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;

        if token_data.expires_at < Utc::now() {
            return Err(AppError::Unauthorized("Token expired".to_string()));
        }

        Ok(token_data.username.clone())
    }

    /// Revoke a token
    pub fn revoke_token(&self, token: &str) -> AppResult<()> {
        self.tokens
            .write()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?
            .remove(token);

        Ok(())
    }

    /// Clean up expired tokens
    pub fn cleanup_expired(&self) -> AppResult<usize> {
        let mut tokens = self
            .tokens
            .write()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        let now = Utc::now();
        let initial_count = tokens.len();

        tokens.retain(|_, data| data.expires_at > now);

        Ok(initial_count - tokens.len())
    }
}

/// Middleware validator function for actix-web-httpauth
pub async fn validator(
    mut req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // Get AuthService from app data
    let auth_service = match req.app_data::<actix_web::web::Data<AuthService>>() {
        Some(service) => service,
        None => {
            return Err((ErrorUnauthorized("Auth service not configured"), req));
        }
    };

    // Validate token
    match auth_service.validate_token(credentials.token()) {
        Ok(username) => {
            // Store username in request extensions for use in handlers
            req.extensions_mut().insert(username);
            Ok(req)
        }
        Err(e) => Err((ErrorUnauthorized(e.to_string()), req)),
    }
}

/// Extract authenticated username from request
pub fn get_authenticated_user(req: &ServiceRequest) -> Option<String> {
    req.extensions().get::<String>().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            sync_dir: "/tmp/test".to_string(),
            host: "127.0.0.1".to_string(),
            port: 3000,
            auth_token_secret: "test_secret".to_string(),
            auth_token_expiry_hours: 24,
            enable_redis_locks: false,
            enable_web_ui: false,
            redis_url: None,
            database_url: None,
        }
    }

    #[test]
    fn test_generate_and_validate_token() {
        let auth = AuthService::new(test_config());
        let token = auth.generate_token("testuser").unwrap();

        assert!(token.starts_with("auxin_"));

        let username = auth.validate_token(&token).unwrap();
        assert_eq!(username, "testuser");
    }

    #[test]
    fn test_invalid_token() {
        let auth = AuthService::new(test_config());
        let result = auth.validate_token("invalid_token");

        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_token() {
        let auth = AuthService::new(test_config());
        let token = auth.generate_token("testuser").unwrap();

        auth.revoke_token(&token).unwrap();

        let result = auth.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_expired() {
        let mut config = test_config();
        config.auth_token_expiry_hours = 0; // Tokens expire immediately

        let auth = AuthService::new(config);
        auth.generate_token("user1").unwrap();
        auth.generate_token("user2").unwrap();

        // Sleep briefly to ensure expiration
        std::thread::sleep(std::time::Duration::from_millis(100));

        let cleaned = auth.cleanup_expired().unwrap();
        assert_eq!(cleaned, 2);
    }
}
