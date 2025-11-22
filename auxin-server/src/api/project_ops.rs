use actix_web::{web, HttpResponse};
use tracing::info;

use crate::error::AppResult;

// File-based collaborator management (default, for authentication system)
#[cfg(not(feature = "web-ui"))]
mod collab_management {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;

    use crate::auth::{get_user_id_from_request, AuthService};
    use crate::config::Config;
    use crate::error::AppError;
    use crate::project::{ProjectAuth, ProjectMetadata, Visibility};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AddCollaboratorRequest {
        pub user_id: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct UpdateVisibilityRequest {
        pub visibility: String, // "public" or "private"
    }

    #[derive(Debug, Serialize)]
    pub struct CollaboratorInfo {
        pub user_id: String,
    }

    /// Add a collaborator to a repository
    pub async fn add_collaborator(
        config: web::Data<Config>,
        path: web::Path<(String, String)>,
        body: web::Json<AddCollaboratorRequest>,
        auth_service: web::Data<AuthService>,
        req: actix_web::HttpRequest,
    ) -> AppResult<HttpResponse> {
        let (namespace, repo_name) = path.into_inner();
        info!("Adding collaborator to: {}/{}", namespace, repo_name);

        // Get authenticated user
        let user_id = get_user_id_from_request(&req, &auth_service)?;

        let repo_path = PathBuf::from(&config.sync_dir)
            .join(&namespace)
            .join(&repo_name);

        if !repo_path.join(".oxen").exists() {
            return Err(AppError::NotFound("Repository not found".to_string()));
        }

        // Only owner can add collaborators
        ProjectAuth::require_owner(&repo_path, &user_id)?;

        // Load metadata
        let mut metadata = ProjectMetadata::load(&repo_path)?;

        // Add collaborator
        metadata.add_collaborator(body.user_id.clone())?;
        metadata.save(&repo_path)?;

        info!(
            "Added collaborator {} to {}/{}",
            body.user_id, namespace, repo_name
        );

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Collaborator added",
            "user_id": body.user_id
        })))
    }

    /// Remove a collaborator from a repository
    pub async fn remove_collaborator(
        config: web::Data<Config>,
        path: web::Path<(String, String, String)>,
        auth_service: web::Data<AuthService>,
        req: actix_web::HttpRequest,
    ) -> AppResult<HttpResponse> {
        let (namespace, repo_name, collaborator_id) = path.into_inner();
        info!(
            "Removing collaborator {} from: {}/{}",
            collaborator_id, namespace, repo_name
        );

        // Get authenticated user
        let user_id = get_user_id_from_request(&req, &auth_service)?;

        let repo_path = PathBuf::from(&config.sync_dir)
            .join(&namespace)
            .join(&repo_name);

        if !repo_path.join(".oxen").exists() {
            return Err(AppError::NotFound("Repository not found".to_string()));
        }

        // Only owner can remove collaborators
        ProjectAuth::require_owner(&repo_path, &user_id)?;

        // Load metadata
        let mut metadata = ProjectMetadata::load(&repo_path)?;

        // Remove collaborator
        metadata.remove_collaborator(&collaborator_id)?;
        metadata.save(&repo_path)?;

        info!(
            "Removed collaborator {} from {}/{}",
            collaborator_id, namespace, repo_name
        );

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Collaborator removed"
        })))
    }

    /// List collaborators for a repository
    pub async fn list_collaborators(
        config: web::Data<Config>,
        path: web::Path<(String, String)>,
        auth_service: web::Data<AuthService>,
        req: actix_web::HttpRequest,
    ) -> AppResult<HttpResponse> {
        let (namespace, repo_name) = path.into_inner();
        info!("Listing collaborators for: {}/{}", namespace, repo_name);

        // Get authenticated user (optional)
        let user_id = crate::auth::get_optional_user_id_from_request(&req, &auth_service);

        let repo_path = PathBuf::from(&config.sync_dir)
            .join(&namespace)
            .join(&repo_name);

        if !repo_path.join(".oxen").exists() {
            return Err(AppError::NotFound("Repository not found".to_string()));
        }

        // Check read access
        ProjectAuth::require_read(&repo_path, user_id.as_deref())?;

        // Load metadata
        let metadata = ProjectMetadata::load(&repo_path)?;

        let collaborators: Vec<CollaboratorInfo> = metadata
            .collaborators
            .iter()
            .map(|id| CollaboratorInfo {
                user_id: id.clone(),
            })
            .collect();

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "owner_id": metadata.owner_id,
            "owner_username": metadata.owner_username,
            "collaborators": collaborators
        })))
    }

    /// Update repository visibility
    pub async fn update_visibility(
        config: web::Data<Config>,
        path: web::Path<(String, String)>,
        body: web::Json<UpdateVisibilityRequest>,
        auth_service: web::Data<AuthService>,
        req: actix_web::HttpRequest,
    ) -> AppResult<HttpResponse> {
        let (namespace, repo_name) = path.into_inner();
        info!("Updating visibility for: {}/{}", namespace, repo_name);

        // Get authenticated user
        let user_id = get_user_id_from_request(&req, &auth_service)?;

        let repo_path = PathBuf::from(&config.sync_dir)
            .join(&namespace)
            .join(&repo_name);

        if !repo_path.join(".oxen").exists() {
            return Err(AppError::NotFound("Repository not found".to_string()));
        }

        // Only owner can change visibility
        ProjectAuth::require_owner(&repo_path, &user_id)?;

        // Parse visibility
        let visibility = match body.visibility.as_str() {
            "public" => Visibility::Public,
            "private" => Visibility::Private,
            v => {
                return Err(AppError::BadRequest(format!(
                    "Invalid visibility: {}. Must be 'public' or 'private'",
                    v
                )));
            }
        };

        // Load metadata
        let mut metadata = ProjectMetadata::load(&repo_path)?;

        // Update visibility
        metadata.set_visibility(visibility);
        metadata.save(&repo_path)?;

        info!(
            "Updated visibility for {}/{} to {}",
            namespace, repo_name, body.visibility
        );

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "visibility": body.visibility
        })))
    }
}

// Re-export file-based collaborator management functions
#[cfg(not(feature = "web-ui"))]
pub use collab_management::*;

// Database-backed project CRUD operations (web-ui feature)
#[cfg(feature = "web-ui")]
mod db_projects {
    use super::*;
    use actix_web::web;
    use sqlx::SqlitePool;

    use crate::models::{CreateProjectRequest, Project, UpdateProjectRequest};

    /// Create a new project
    pub async fn create_project(
        pool: web::Data<SqlitePool>,
        req: web::Json<CreateProjectRequest>,
    ) -> AppResult<HttpResponse> {
        info!("Creating project: {}/{}", req.namespace, req.name);

        let project = Project::create(&pool, req.into_inner()).await?;

        Ok(HttpResponse::Created().json(project))
    }

    /// List all projects
    pub async fn list_projects(pool: web::Data<SqlitePool>) -> AppResult<HttpResponse> {
        info!("Listing all projects");

        let projects = Project::list(&pool).await?;

        Ok(HttpResponse::Ok().json(projects))
    }

    /// Get a project by ID
    pub async fn get_project(
        pool: web::Data<SqlitePool>,
        path: web::Path<String>,
    ) -> AppResult<HttpResponse> {
        let id = path.into_inner();
        info!("Getting project: {}", id);

        let project = Project::get_by_id(&pool, &id).await?;

        Ok(HttpResponse::Ok().json(project))
    }

    /// Get a project by namespace and name
    pub async fn get_project_by_namespace(
        pool: web::Data<SqlitePool>,
        path: web::Path<(String, String)>,
    ) -> AppResult<HttpResponse> {
        let (namespace, name) = path.into_inner();
        info!("Getting project: {}/{}", namespace, name);

        let project = Project::get_by_namespace_and_name(&pool, &namespace, &name).await?;

        Ok(HttpResponse::Ok().json(project))
    }

    /// Update a project
    pub async fn update_project(
        pool: web::Data<SqlitePool>,
        path: web::Path<String>,
        req: web::Json<UpdateProjectRequest>,
    ) -> AppResult<HttpResponse> {
        let id = path.into_inner();
        info!("Updating project: {}", id);

        let project = Project::update(&pool, &id, req.into_inner()).await?;

        Ok(HttpResponse::Ok().json(project))
    }

    /// Delete a project
    pub async fn delete_project(
        pool: web::Data<SqlitePool>,
        path: web::Path<String>,
    ) -> AppResult<HttpResponse> {
        let id = path.into_inner();
        info!("Deleting project: {}", id);

        Project::delete(&pool, &id).await?;

        Ok(HttpResponse::NoContent().finish())
    }
}

// Re-export database-backed project CRUD functions
#[cfg(feature = "web-ui")]
pub use db_projects::*;
