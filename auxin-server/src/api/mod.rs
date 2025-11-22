mod bounce_ops;
mod project_ops;
mod repo_ops;

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

use crate::auth::{get_optional_user_id_from_request, get_user_id_from_request, AuthService};
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::project::{ProjectMetadata, Visibility};
use crate::repo::RepositoryOps;

// Re-export API handlers
pub use repo_ops::{
    acquire_lock, clone_repository, create_branch, delete_branch, fetch_repository, get_activity,
    get_commits, get_metadata, get_status, heartbeat_lock, list_branches, lock_status,
    pull_repository, push_repository, release_lock, restore_commit, store_metadata,
};

pub use bounce_ops::{delete_bounce, get_bounce, get_bounce_audio, list_bounces, upload_bounce};

// File-based collaborator management (default)
#[cfg(not(feature = "web-ui"))]
pub use project_ops::{
    add_collaborator, list_collaborators, remove_collaborator, update_visibility,
};

// Database-backed project CRUD (web-ui feature)
#[cfg(feature = "web-ui")]
pub use project_ops::{
    create_project, delete_project, get_project, get_project_by_namespace, list_projects,
    update_project,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepoRequest {
    pub description: Option<String>,
    pub visibility: Option<String>, // "public" or "private"
}

#[derive(Debug, Serialize)]
pub struct RepositoryInfo {
    pub namespace: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub visibility: Option<String>,
}

/// List all repositories
pub async fn list_repositories(
    config: web::Data<Config>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse> {
    info!("Listing repositories from: {}", config.sync_dir);

    let user_id = get_optional_user_id_from_request(&req, &auth_service);
    let mut repositories = Vec::new();

    // Scan SYNC_DIR for repositories
    if let Ok(entries) = fs::read_dir(&config.sync_dir) {
        for namespace_entry in entries.flatten() {
            if let Ok(namespace_type) = namespace_entry.file_type() {
                if namespace_type.is_dir() {
                    let namespace = namespace_entry.file_name().to_string_lossy().to_string();

                    // Skip .auxin directory
                    if namespace == ".auxin" {
                        continue;
                    }

                    // Scan namespace for repos
                    if let Ok(repo_entries) = fs::read_dir(namespace_entry.path()) {
                        for repo_entry in repo_entries.flatten() {
                            if let Ok(repo_type) = repo_entry.file_type() {
                                if repo_type.is_dir() {
                                    let repo_name =
                                        repo_entry.file_name().to_string_lossy().to_string();

                                    // Check if .oxen directory exists
                                    let oxen_dir = repo_entry.path().join(".oxen");
                                    if oxen_dir.exists() {
                                        // Load project metadata
                                        let metadata =
                                            ProjectMetadata::load(&repo_entry.path()).ok();

                                        // Check if user has read access
                                        let has_access = metadata
                                            .as_ref()
                                            .map(|m| m.has_read_access(user_id.as_deref()))
                                            .unwrap_or(true); // Allow access if no metadata (backward compatibility)

                                        if has_access {
                                            repositories.push(RepositoryInfo {
                                                namespace: namespace.clone(),
                                                name: repo_name,
                                                path: repo_entry
                                                    .path()
                                                    .to_string_lossy()
                                                    .to_string(),
                                                description: None,
                                                owner: metadata
                                                    .as_ref()
                                                    .map(|m| m.owner_username.clone()),
                                                visibility: metadata.as_ref().map(|m| {
                                                    match m.visibility {
                                                        Visibility::Public => "public".to_string(),
                                                        Visibility::Private => {
                                                            "private".to_string()
                                                        }
                                                    }
                                                }),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Found {} accessible repositories", repositories.len());
    Ok(HttpResponse::Ok().json(repositories))
}

/// Create a new repository
pub async fn create_repository(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<CreateRepoRequest>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Creating repository: {}/{}", namespace, repo_name);

    // Require authentication
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    let user = auth_service.get_user_by_token(
        req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("No authorization token".to_string()))?,
    )?;

    // Validate namespace (prevent path traversal)
    if namespace.is_empty() || namespace.contains("..") || namespace.contains('/') {
        return Err(AppError::BadRequest("Invalid namespace".to_string()));
    }

    // Validate repository name (prevent path traversal)
    if repo_name.is_empty() || repo_name.contains("..") || repo_name.contains('/') {
        return Err(AppError::BadRequest("Invalid repository name".to_string()));
    }

    // Build repository path
    let repo_path = PathBuf::from(&config.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check if repository already exists
    if repo_path.exists() {
        return Err(AppError::BadRequest(
            "Repository already exists".to_string(),
        ));
    }

    // Initialize using liboxen
    let _repo = RepositoryOps::init(&repo_path)?;

    // Parse visibility
    let visibility = match body.visibility.as_deref() {
        Some("public") => Visibility::Public,
        Some("private") => Visibility::Private,
        None => Visibility::Public, // Default to public
        Some(v) => {
            return Err(AppError::BadRequest(format!(
                "Invalid visibility: {}. Must be 'public' or 'private'",
                v
            )));
        }
    };

    // Create project metadata
    let metadata = ProjectMetadata::new(user_id, user.username.clone(), visibility);
    metadata.save(&repo_path)?;

    info!(
        "Repository created successfully: {}/{} (owner: {})",
        namespace, repo_name, user.username
    );

    Ok(HttpResponse::Created().json(RepositoryInfo {
        namespace,
        name: repo_name,
        path: repo_path.to_string_lossy().to_string(),
        description: body.description.clone(),
        owner: Some(user.username),
        visibility: Some(match metadata.visibility {
            Visibility::Public => "public".to_string(),
            Visibility::Private => "private".to_string(),
        }),
    }))
}

/// Get repository information
pub async fn get_repository(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Getting repository: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    if !repo_path.join(".oxen").exists() {
        return Err(AppError::NotFound("Repository not found".to_string()));
    }

    // Check read access
    let user_id = get_optional_user_id_from_request(&req, &auth_service);
    let metadata = ProjectMetadata::load(&repo_path).ok();

    if let Some(ref m) = metadata {
        if !m.has_read_access(user_id.as_deref()) {
            return Err(AppError::Forbidden(
                "You do not have access to this repository".to_string(),
            ));
        }
    }

    Ok(HttpResponse::Ok().json(RepositoryInfo {
        namespace,
        name: repo_name,
        path: repo_path.to_string_lossy().to_string(),
        description: None,
        owner: metadata.as_ref().map(|m| m.owner_username.clone()),
        visibility: metadata.as_ref().map(|m| match m.visibility {
            Visibility::Public => "public".to_string(),
            Visibility::Private => "private".to_string(),
        }),
    }))
}
