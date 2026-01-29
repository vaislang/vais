//! Data models for the registry server

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API token for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Token scopes
pub mod scopes {
    pub const PUBLISH: &str = "publish";
    pub const YANK: &str = "yank";
    pub const ADMIN: &str = "admin";
}

/// Package metadata stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub downloads: i64,
}

/// Package version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    pub id: Uuid,
    pub package_id: Uuid,
    pub version: String,
    pub checksum: String,
    pub size: i64,
    pub yanked: bool,
    pub readme: Option<String>,
    pub published_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub downloads: i64,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: Uuid,
    pub version_id: Uuid,
    pub name: String,
    pub version_req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub target: Option<String>,
    pub kind: DependencyKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum DependencyKind {
    #[default]
    Normal,
    Dev,
    Build,
}


/// Package owner (for shared ownership)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageOwner {
    pub package_id: Uuid,
    pub user_id: Uuid,
    pub added_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Request types

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CreateTokenResponse {
    pub id: Uuid,
    pub token: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, DependencySpec>,
    pub readme: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Simple(String),
    Detailed {
        version: Option<String>,
        #[serde(default)]
        features: Vec<String>,
        #[serde(default)]
        optional: bool,
        target: Option<String>,
    },
}

impl DependencySpec {
    pub fn version_req(&self) -> &str {
        match self {
            DependencySpec::Simple(v) => v,
            DependencySpec::Detailed { version, .. } => version.as_deref().unwrap_or("*"),
        }
    }

    pub fn features(&self) -> Vec<String> {
        match self {
            DependencySpec::Simple(_) => vec![],
            DependencySpec::Detailed { features, .. } => features.clone(),
        }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            DependencySpec::Simple(_) => false,
            DependencySpec::Detailed { optional, .. } => *optional,
        }
    }

    pub fn target(&self) -> Option<&str> {
        match self {
            DependencySpec::Simple(_) => None,
            DependencySpec::Detailed { target, .. } => target.as_deref(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    /// Sort order: "downloads" (default), "newest", "name", "relevance"
    #[serde(default = "default_sort")]
    pub sort: String,
    /// Filter by category
    pub category: Option<String>,
    /// Filter by keyword
    pub keyword: Option<String>,
}

fn default_limit() -> usize {
    20
}

fn default_sort() -> String {
    "downloads".to_string()
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub packages: Vec<PackageSearchEntry>,
    pub total: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<CategoryCount>,
}

/// Category with package count for faceted search
#[derive(Debug, Serialize)]
pub struct CategoryCount {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct PackageSearchEntry {
    pub name: String,
    pub description: Option<String>,
    pub latest_version: String,
    pub downloads: i64,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

/// Response for package info
#[derive(Debug, Serialize)]
pub struct PackageInfo {
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub downloads: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub versions: Vec<VersionInfo>,
    pub owners: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub checksum: String,
    pub size: i64,
    pub yanked: bool,
    pub downloads: i64,
    pub created_at: DateTime<Utc>,
    pub dependencies: Vec<DependencyInfo>,
}

#[derive(Debug, Serialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version_req: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub kind: DependencyKind,
}

/// Index format (compatible with existing client)
#[derive(Debug, Serialize)]
pub struct IndexEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub versions: Vec<IndexVersionEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct IndexVersionEntry {
    pub version: String,
    pub checksum: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, IndexDependency>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub yanked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct IndexDependency {
    pub req: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}
