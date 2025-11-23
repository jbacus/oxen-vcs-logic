//! Repository access control API operations

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::auth::{require_role, AuthService, UserRole};
use crate::error::{AppError, AppResult};
use crate::repo_access::RepoAccessService;

/// Request to grant repository access
#[derive(Debug, Deserialize)]
pub struct GrantAccessRequest {
    pub user_id: String,
}

/// Request to revoke repository access
#[derive(Debug, Deserialize)]
pub struct RevokeAccessRequest {
    pub user_id: String,
}

/// Response for access operations
#[derive(Debug, Serialize)]
pub struct AccessResponse {
    pub message: String,
}

/// Response for listing access
#[derive(Debug, Serialize)]
pub struct AccessListResponse {
    pub users: Vec<String>,
}

/// Grant access to a repository for a user
/// POST /api/repos/{namespace}/{name}/access/grant
/// Requires Producer or Admin role
pub async fn grant_access(
    path: web::Path<(String, String)>,
    body: web::Json<GrantAccessRequest>,
    auth_service: web::Data<AuthService>,
    repo_access: web::Data<RepoAccessService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();

    // Require Producer or Admin role
    let user = require_role(&req, &auth_service, UserRole::Producer)?;
    info!(
        "User {} granting access to {}/{} for user {}",
        user.username, namespace, repo_name, body.user_id
    );

    repo_access.grant_access(&namespace, &repo_name, &body.user_id)?;

    Ok(HttpResponse::Ok().json(AccessResponse {
        message: format!(
            "Access granted to {}/{} for user {}",
            namespace, repo_name, body.user_id
        ),
    }))
}

/// Revoke access to a repository for a user
/// POST /api/repos/{namespace}/{name}/access/revoke
/// Requires Producer or Admin role
pub async fn revoke_access(
    path: web::Path<(String, String)>,
    body: web::Json<RevokeAccessRequest>,
    auth_service: web::Data<AuthService>,
    repo_access: web::Data<RepoAccessService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();

    // Require Producer or Admin role
    let user = require_role(&req, &auth_service, UserRole::Producer)?;
    info!(
        "User {} revoking access to {}/{} for user {}",
        user.username, namespace, repo_name, body.user_id
    );

    repo_access.revoke_access(&namespace, &repo_name, &body.user_id)?;

    Ok(HttpResponse::Ok().json(AccessResponse {
        message: format!(
            "Access revoked to {}/{} for user {}",
            namespace, repo_name, body.user_id
        ),
    }))
}

/// List users with access to a repository
/// GET /api/repos/{namespace}/{name}/access
/// Requires Producer or Admin role
pub async fn list_access(
    path: web::Path<(String, String)>,
    auth_service: web::Data<AuthService>,
    repo_access: web::Data<RepoAccessService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();

    // Require Producer or Admin role
    require_role(&req, &auth_service, UserRole::Producer)?;

    let users = repo_access.list_repo_users(&namespace, &repo_name)?;

    Ok(HttpResponse::Ok().json(AccessListResponse { users }))
}
