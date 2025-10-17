use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use validator::Validate;

// ===== User Models =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
}

// ===== Token Models =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiToken {
    pub token_id: Uuid,
    pub user_id: Uuid,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    #[serde(default = "default_token_expiry")]
    pub expires_in_days: i64,
}

fn default_token_expiry() -> i64 {
    30
}

#[derive(Debug, Serialize)]
pub struct CreateTokenResponse {
    pub token: String,
    pub token_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// ===== Package Models =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Package {
    pub package_id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub description: Option<String>,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PackageVersion {
    pub version_id: Uuid,
    pub package_id: Uuid,
    pub version: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: String,
    pub repository: Option<String>,
    pub dependencies: serde_json::Value, // JSON object
    pub dev_dependencies: serde_json::Value, // JSON object
    pub tarball_url: String,
    pub checksum: String,
    pub size_bytes: i64,
    pub published_at: DateTime<Utc>,
    pub yanked: bool,
    pub yanked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PublishRequest {
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    #[validate(length(min = 1))]
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub package_id: Uuid,
    pub name: String,
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub download_url: String,
    pub checksum: String,
}

#[derive(Debug, Serialize)]
pub struct PackageResponse {
    pub name: String,
    pub description: Option<String>,
    pub latest_version: String,
    pub versions: Vec<String>,
    pub owner: OwnerInfo,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub downloads_total: i64,
    pub downloads_last_month: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct OwnerInfo {
    pub username: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: String,
    pub repository: Option<String>,
    pub dependencies: serde_json::Value,
    pub dev_dependencies: serde_json::Value,
    pub published_at: DateTime<Utc>,
    pub download_url: String,
    pub checksum: String,
    pub size_bytes: i64,
    pub yanked: bool,
}

// ===== Search Models =====

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub downloads: i64,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ===== Statistics Models =====

#[derive(Debug, Serialize)]
pub struct GlobalStats {
    pub total_packages: i64,
    pub total_versions: i64,
    pub total_downloads: i64,
    pub total_users: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PackageStats {
    pub name: String,
    pub downloads: DownloadStats,
    pub versions_count: i64,
    pub dependents_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DownloadStats {
    pub total: i64,
    pub last_week: i64,
    pub last_month: i64,
    pub last_year: i64,
}

// ===== Download Tracking =====

#[derive(Debug, Clone, FromRow)]
pub struct Download {
    pub download_id: Uuid,
    pub package_id: Uuid,
    pub version_id: Uuid,
    pub downloaded_at: DateTime<Utc>,
    pub ip_hash: String,
}

// ===== Package Owners =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PackageOwner {
    pub package_id: Uuid,
    pub user_id: Uuid,
    pub role: String, // "owner" or "maintainer"
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddOwnerRequest {
    pub username: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct OwnerResponse {
    pub username: String,
    pub user_id: Uuid,
    pub role: String,
}

// ===== Error Response =====

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(
        error: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: Some(details),
        }
    }
}
