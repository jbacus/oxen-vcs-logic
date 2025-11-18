mod bounce_ops;
mod repo_ops;

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::repo::RepositoryOps;

// Re-export API handlers
pub use repo_ops::{
    acquire_lock, create_branch, get_commits, get_metadata, heartbeat_lock,
    list_branches, lock_status, pull_repository, push_repository,
    release_lock, store_metadata,
};

pub use bounce_ops::{
    delete_bounce, get_bounce, get_bounce_audio, list_bounces, upload_bounce,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepoRequest {
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RepositoryInfo {
    pub namespace: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
}

/// List all repositories
pub async fn list_repositories(config: web::Data<Config>) -> Result<HttpResponse> {
    info!("Listing repositories from: {}", config.sync_dir);

    let mut repositories = Vec::new();

    // Scan SYNC_DIR for repositories
    if let Ok(entries) = fs::read_dir(&config.sync_dir) {
        for namespace_entry in entries.flatten() {
            if let Ok(namespace_type) = namespace_entry.file_type() {
                if namespace_type.is_dir() {
                    let namespace = namespace_entry.file_name().to_string_lossy().to_string();

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
                                        repositories.push(RepositoryInfo {
                                            namespace: namespace.clone(),
                                            name: repo_name,
                                            path: repo_entry.path().to_string_lossy().to_string(),
                                            description: None,
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

    info!("Found {} repositories", repositories.len());
    Ok(HttpResponse::Ok().json(repositories))
}

/// Create a new repository
pub async fn create_repository(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    req: web::Json<CreateRepoRequest>,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Creating repository: {}/{}", namespace, repo_name);

    // Validate namespace (prevent path traversal)
    if namespace.is_empty() || namespace.contains("..") || namespace.contains('/') {
        return Err(AppError::BadRequest(
            "Invalid namespace".to_string(),
        ));
    }

    // Validate repository name (prevent path traversal)
    if repo_name.is_empty() || repo_name.contains("..") || repo_name.contains('/') {
        return Err(AppError::BadRequest(
            "Invalid repository name".to_string(),
        ));
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

    info!("Repository created successfully: {}/{}", namespace, repo_name);

    Ok(HttpResponse::Created().json(RepositoryInfo {
        namespace,
        name: repo_name,
        path: repo_path.to_string_lossy().to_string(),
        description: req.description.clone(),
    }))
}

/// Get repository information
pub async fn get_repository(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Getting repository: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    if !repo_path.join(".oxen").exists() {
        return Err(AppError::NotFound("Repository not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(RepositoryInfo {
        namespace,
        name: repo_name,
        path: repo_path.to_string_lossy().to_string(),
        description: None,
    }))
}
