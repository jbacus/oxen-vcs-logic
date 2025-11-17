use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Repository {
    pub id: Uuid,
    pub namespace: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: RepositoryVisibility,
    pub owner_id: Uuid,
    pub storage_path: String,
    pub default_branch: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_push_at: Option<DateTime<Utc>>,
    pub is_archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "repository_visibility", rename_all = "lowercase")]
pub enum RepositoryVisibility {
    Public,
    Private,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Commit {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub commit_hash: String,
    pub parent_hash: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub committer_name: String,
    pub committer_email: String,
    pub message: String,
    pub branch: String,
    pub created_at: DateTime<Utc>,
    pub bpm: Option<f64>,
    pub sample_rate: Option<i32>,
    pub key_signature: Option<String>,
    pub tags: Option<Vec<String>>,
}
