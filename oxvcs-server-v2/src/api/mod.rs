use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

use crate::config::Config;
use crate::error::{AppError, AppResult};

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

    // Validate repository name
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

    // Create repository directory
    fs::create_dir_all(&repo_path).map_err(|e| {
        error!("Failed to create repository directory: {}", e);
        AppError::Internal("Failed to create repository".to_string())
    })?;

    // Initialize .oxen directory structure (minimal for now)
    let oxen_dir = repo_path.join(".oxen");
    fs::create_dir_all(oxen_dir.join("history")).map_err(|e| {
        error!("Failed to create .oxen/history: {}", e);
        AppError::Internal("Failed to initialize repository".to_string())
    })?;

    fs::create_dir_all(oxen_dir.join("refs/heads")).map_err(|e| {
        error!("Failed to create .oxen/refs/heads: {}", e);
        AppError::Internal("Failed to initialize repository".to_string())
    })?;

    fs::create_dir_all(oxen_dir.join("tree")).map_err(|e| {
        error!("Failed to create .oxen/tree: {}", e);
        AppError::Internal("Failed to initialize repository".to_string())
    })?;

    fs::create_dir_all(oxen_dir.join("versions")).map_err(|e| {
        error!("Failed to create .oxen/versions: {}", e);
        AppError::Internal("Failed to initialize repository".to_string())
    })?;

    // OxVCS extensions
    fs::create_dir_all(oxen_dir.join("metadata")).map_err(|e| {
        error!("Failed to create .oxen/metadata: {}", e);
        AppError::Internal("Failed to initialize repository".to_string())
    })?;

    fs::create_dir_all(oxen_dir.join("locks")).map_err(|e| {
        error!("Failed to create .oxen/locks: {}", e);
        AppError::Internal("Failed to initialize repository".to_string())
    })?;

    // Write config.toml
    let config_content = format!(
        "[repository]\nname = \"{}\"\nnamespace = \"{}\"\n",
        repo_name, namespace
    );
    fs::write(oxen_dir.join("config.toml"), config_content).map_err(|e| {
        error!("Failed to write config.toml: {}", e);
        AppError::Internal("Failed to initialize repository".to_string())
    })?;

    // Write HEAD (default to main branch)
    fs::write(oxen_dir.join("HEAD"), "ref: refs/heads/main\n").map_err(|e| {
        error!("Failed to write HEAD: {}", e);
        AppError::Internal("Failed to initialize repository".to_string())
    })?;

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
