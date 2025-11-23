use actix_web::{
    dev::ServiceRequest, error::ErrorUnauthorized, web, Error, HttpMessage, HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::info;

use auxin_config::Config;
use crate::error::{AppError, AppResult};

/// User role for access control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Full system access
    Admin,
    /// Can create repos, push/pull, upload bounces, acquire locks
    Producer,
    /// Read-only access to bounces in repos they're invited to
    Client,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Producer
    }
}

impl UserRole {
    /// Check if role can write to repositories
    pub fn can_write(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Producer)
    }

    /// Check if role can manage users and permissions
    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Check if role can upload/delete bounces
    pub fn can_manage_bounces(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Producer)
    }
}

/// User account stored in JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    #[serde(default)]
    pub role: UserRole,
    pub created_at: chrono::DateTime<Utc>,
}

/// User response (without password hash)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

/// Token data stored in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenData {
    user_id: String,
    username: String,
    expires_at: chrono::DateTime<Utc>,
}

/// Simple token-based authentication with user persistence
#[derive(Debug, Clone)]
pub struct AuthService {
    config: Config,
    // In-memory token store
    tokens: Arc<RwLock<HashMap<String, TokenData>>>,
    // In-memory user cache (backed by JSON file)
    users: Arc<RwLock<HashMap<String, User>>>,
}

/// Request/response types for auth endpoints
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

impl AuthService {
    pub fn new(config: Config) -> Self {
        let service = Self {
            config: config.clone(),
            tokens: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load users from disk on startup
        if let Err(e) = service.load_users() {
            info!("No existing users file or error loading: {}", e);
        }

        service
    }

    /// Get users file path
    fn users_file_path(&self) -> PathBuf {
        PathBuf::from(&self.config.server.sync_dir)
            .join(".auxin")
            .join("users.json")
    }

    /// Load users from JSON file
    fn load_users(&self) -> AppResult<()> {
        let path = self.users_file_path();
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| AppError::Internal(format!("Failed to read users file: {}", e)))?;

        let users: Vec<User> = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse users file: {}", e)))?;

        let mut user_map = self
            .users
            .write()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        for user in users {
            user_map.insert(user.id.clone(), user);
        }

        info!("Loaded {} users from disk", user_map.len());
        Ok(())
    }

    /// Save users to JSON file
    fn save_users(&self) -> AppResult<()> {
        let path = self.users_file_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Internal(format!("Failed to create directory: {}", e)))?;
        }

        let users = self
            .users
            .read()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        let user_list: Vec<&User> = users.values().collect();
        let content = serde_json::to_string_pretty(&user_list)
            .map_err(|e| AppError::Internal(format!("Failed to serialize users: {}", e)))?;

        std::fs::write(&path, content)
            .map_err(|e| AppError::Internal(format!("Failed to write users file: {}", e)))?;

        Ok(())
    }

    /// Register a new user
    pub fn register(&self, username: &str, email: &str, password: &str, role: Option<UserRole>) -> AppResult<User> {
        // Validate input
        if username.len() < 3 {
            return Err(AppError::BadRequest(
                "Username must be at least 3 characters".to_string(),
            ));
        }
        if password.len() < 8 {
            return Err(AppError::BadRequest(
                "Password must be at least 8 characters".to_string(),
            ));
        }
        if !email.contains('@') {
            return Err(AppError::BadRequest("Invalid email address".to_string()));
        }

        // Check if user already exists
        {
            let users = self
                .users
                .read()
                .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

            for user in users.values() {
                if user.email == email {
                    return Err(AppError::BadRequest("Email already registered".to_string()));
                }
                if user.username == username {
                    return Err(AppError::BadRequest("Username already taken".to_string()));
                }
            }
        }

        // Hash password
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

        // Create user
        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash: Some(password_hash),
            role: role.unwrap_or_default(),
            created_at: Utc::now(),
        };

        // Save user
        {
            let mut users = self
                .users
                .write()
                .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;
            users.insert(user.id.clone(), user.clone());
        }

        // Persist to disk
        self.save_users()?;

        info!("Registered new user: {}", username);
        Ok(user)
    }

    /// Login with email and password
    pub fn login(&self, email: &str, password: &str) -> AppResult<(String, User)> {
        let users = self
            .users
            .read()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        // Find user by email
        let user = users
            .values()
            .find(|u| u.email == email)
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        // Verify password
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or_else(|| AppError::Internal("User has no password".to_string()))?;

        let valid = verify(password, password_hash)
            .map_err(|e| AppError::Internal(format!("Password verification failed: {}", e)))?;

        if !valid {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        // Generate token
        let token = self.generate_token(&user.id, &user.username)?;

        info!("User logged in: {}", user.username);
        Ok((token, user.clone()))
    }

    /// Generate a new token for a user
    pub fn generate_token(&self, user_id: &str, username: &str) -> AppResult<String> {
        let token = format!("auxin_{}", uuid::Uuid::new_v4());
        let expires_at = Utc::now() + Duration::hours(self.config.server.auth_token_expiry_hours as i64);

        let token_data = TokenData {
            user_id: user_id.to_string(),
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

    /// Get user by token
    pub fn get_user_by_token(&self, token: &str) -> AppResult<User> {
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

        let users = self
            .users
            .read()
            .map_err(|_| AppError::Internal("Lock poisoned".to_string()))?;

        users
            .get(&token_data.user_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
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

// HTTP Handlers

/// POST /api/auth/register
pub async fn register(
    auth_service: web::Data<AuthService>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    let user = auth_service.register(&body.username, &body.email, &body.password, body.role)?;
    let token = auth_service.generate_token(&user.id, &user.username)?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// POST /api/auth/login
pub async fn login(
    auth_service: web::Data<AuthService>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let (token, user) = auth_service.login(&body.email, &body.password)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// POST /api/auth/logout
pub async fn logout(
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Extract token from Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                auth_service.revoke_token(token)?;
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

/// GET /api/auth/me
pub async fn me(
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Extract token from Authorization header
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("No authorization token".to_string()))?;

    let user = auth_service.get_user_by_token(token)?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

/// Middleware validator function for actix-web-httpauth
pub async fn validator(
    mut req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // Get AuthService from app data
    let auth_service = match req.app_data::<web::Data<AuthService>>() {
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

/// Extract authenticated username from request (for middleware)
pub fn get_authenticated_user(req: &ServiceRequest) -> Option<String> {
    req.extensions().get::<String>().cloned()
}

/// Extract authenticated user ID from HTTP request
pub fn get_user_id_from_request(
    req: &actix_web::HttpRequest,
    auth_service: &AuthService,
) -> AppResult<String> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("No authorization token".to_string()))?;

    let user = auth_service.get_user_by_token(token)?;
    Ok(user.id)
}

/// Extract optional authenticated user ID from HTTP request (for public endpoints)
pub fn get_optional_user_id_from_request(
    req: &actix_web::HttpRequest,
    auth_service: &AuthService,
) -> Option<String> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))?;

    auth_service.get_user_by_token(token).ok().map(|u| u.id)
}

/// Get user role from HTTP request
pub fn get_user_role_from_request(
    req: &actix_web::HttpRequest,
    auth_service: &AuthService,
) -> AppResult<UserRole> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("No authorization token".to_string()))?;

    let user = auth_service.get_user_by_token(token)?;
    Ok(user.role)
}

/// Check if user has required role
pub fn require_role(
    req: &actix_web::HttpRequest,
    auth_service: &AuthService,
    required_role: UserRole,
) -> AppResult<User> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("No authorization token".to_string()))?;

    let user = auth_service.get_user_by_token(token)?;

    match (user.role, required_role) {
        // Admin can do anything
        (UserRole::Admin, _) => Ok(user),
        // Exact role match
        (role, req_role) if role == req_role => Ok(user),
        // Producer can do what Client can
        (UserRole::Producer, UserRole::Client) => Ok(user),
        // Otherwise unauthorized
        _ => Err(AppError::Unauthorized(format!(
            "This action requires {:?} role or higher",
            required_role
        ))),
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
    fn test_register_user() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        let user = auth
            .register("testuser", "test@example.com", "password123", None)
            .unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::Producer); // Default role
        assert!(user.password_hash.is_some());
    }

    #[test]
    fn test_register_duplicate_email() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        auth.register("user1", "test@example.com", "password123", None)
            .unwrap();

        let result = auth.register("user2", "test@example.com", "password456", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_short_password() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        let result = auth.register("testuser", "test@example.com", "short", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_client_role() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        let user = auth
            .register("client", "client@example.com", "password123", Some(UserRole::Client))
            .unwrap();

        assert_eq!(user.role, UserRole::Client);
    }

    #[test]
    fn test_login() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        auth.register("testuser", "test@example.com", "password123", None)
            .unwrap();

        let (token, user) = auth.login("test@example.com", "password123").unwrap();
        assert!(token.starts_with("auxin_"));
        assert_eq!(user.username, "testuser");
    }

    #[test]
    fn test_login_wrong_password() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        auth.register("testuser", "test@example.com", "password123", None)
            .unwrap();

        let result = auth.login("test@example.com", "wrongpassword");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_user_by_token() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        let registered = auth
            .register("testuser", "test@example.com", "password123", None)
            .unwrap();

        let (token, _) = auth.login("test@example.com", "password123").unwrap();
        let user = auth.get_user_by_token(&token).unwrap();

        assert_eq!(user.id, registered.id);
        assert_eq!(user.username, "testuser");
    }

    #[test]
    fn test_generate_and_validate_token() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        let token = auth.generate_token("user-id", "testuser").unwrap();

        assert!(token.starts_with("auxin_"));

        let username = auth.validate_token(&token).unwrap();
        assert_eq!(username, "testuser");
    }

    #[test]
    fn test_invalid_token() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        let result = auth.validate_token("invalid_token");

        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_token() {
        let temp_dir = TempDir::new().unwrap();
        let auth = AuthService::new(test_config_with_dir(&temp_dir));
        let token = auth.generate_token("user-id", "testuser").unwrap();

        auth.revoke_token(&token).unwrap();

        let result = auth.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_expired() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = test_config_with_dir(&temp_dir);
        config.server.auth_token_expiry_hours = 0; // Tokens expire immediately

        let auth = AuthService::new(config);
        auth.generate_token("id1", "user1").unwrap();
        auth.generate_token("id2", "user2").unwrap();

        // Sleep briefly to ensure expiration
        std::thread::sleep(std::time::Duration::from_millis(100));

        let cleaned = auth.cleanup_expired().unwrap();
        assert_eq!(cleaned, 2);
    }
}