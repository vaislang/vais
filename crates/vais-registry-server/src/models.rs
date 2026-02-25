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

/// Registry statistics
#[derive(Debug, Serialize)]
pub struct RegistryStats {
    pub total_packages: i64,
    pub total_downloads: i64,
    pub total_versions: i64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    // ========== DependencyKind tests ==========

    #[test]
    fn test_dependency_kind_default() {
        let kind: DependencyKind = Default::default();
        assert_eq!(kind, DependencyKind::Normal);
    }

    #[test]
    fn test_dependency_kind_serde_roundtrip() {
        for kind in [
            DependencyKind::Normal,
            DependencyKind::Dev,
            DependencyKind::Build,
        ] {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: DependencyKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn test_dependency_kind_serde_values() {
        assert_eq!(
            serde_json::to_string(&DependencyKind::Normal).unwrap(),
            "\"normal\""
        );
        assert_eq!(
            serde_json::to_string(&DependencyKind::Dev).unwrap(),
            "\"dev\""
        );
        assert_eq!(
            serde_json::to_string(&DependencyKind::Build).unwrap(),
            "\"build\""
        );
    }

    // ========== DependencySpec tests ==========

    #[test]
    fn test_dependency_spec_simple() {
        let spec = DependencySpec::Simple("^1.0".to_string());
        assert_eq!(spec.version_req(), "^1.0");
        assert!(spec.features().is_empty());
        assert!(!spec.is_optional());
        assert!(spec.target().is_none());
    }

    #[test]
    fn test_dependency_spec_detailed_full() {
        let spec = DependencySpec::Detailed {
            version: Some(">=2.0".to_string()),
            features: vec!["serde".to_string(), "async".to_string()],
            optional: true,
            target: Some("x86_64-unknown-linux-gnu".to_string()),
        };
        assert_eq!(spec.version_req(), ">=2.0");
        assert_eq!(spec.features(), vec!["serde", "async"]);
        assert!(spec.is_optional());
        assert_eq!(spec.target(), Some("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn test_dependency_spec_detailed_no_version() {
        let spec = DependencySpec::Detailed {
            version: None,
            features: vec![],
            optional: false,
            target: None,
        };
        assert_eq!(spec.version_req(), "*");
    }

    #[test]
    fn test_dependency_spec_serde_simple() {
        let json = r#""^1.0""#;
        let spec: DependencySpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.version_req(), "^1.0");
    }

    #[test]
    fn test_dependency_spec_serde_detailed() {
        let json = r#"{"version": "^1.0", "features": ["serde"], "optional": true}"#;
        let spec: DependencySpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.version_req(), "^1.0");
        assert_eq!(spec.features(), vec!["serde"]);
        assert!(spec.is_optional());
    }

    // ========== User serde tests ==========

    #[test]
    fn test_user_serialize_skips_password() {
        let user = User {
            id: Uuid::new_v4(),
            username: "alice".to_string(),
            password_hash: "secret_hash".to_string(),
            email: Some("alice@example.com".to_string()),
            is_admin: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(!json.contains("secret_hash"));
        assert!(json.contains("alice"));
        assert!(json.contains("alice@example.com"));
    }

    #[test]
    fn test_user_with_no_email() {
        let user = User {
            id: Uuid::new_v4(),
            username: "bob".to_string(),
            password_hash: "hash".to_string(),
            email: None,
            is_admin: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"is_admin\":true"));
    }

    // ========== ApiToken serde tests ==========

    #[test]
    fn test_api_token_serialize_skips_hash() {
        let token = ApiToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "deploy-token".to_string(),
            token_hash: "super_secret".to_string(),
            scopes: vec!["publish".to_string(), "yank".to_string()],
            expires_at: None,
            last_used_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(!json.contains("super_secret"));
        assert!(json.contains("deploy-token"));
        assert!(json.contains("publish"));
    }

    // ========== Scopes constants tests ==========

    #[test]
    fn test_scopes_constants() {
        assert_eq!(scopes::PUBLISH, "publish");
        assert_eq!(scopes::YANK, "yank");
        assert_eq!(scopes::ADMIN, "admin");
    }

    // ========== SearchQuery defaults tests ==========

    #[test]
    fn test_search_query_defaults() {
        let json = r#"{"q": "hello"}"#;
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.q, "hello");
        assert_eq!(query.limit, 20);
        assert_eq!(query.offset, 0);
        assert_eq!(query.sort, "downloads");
        assert!(query.category.is_none());
        assert!(query.keyword.is_none());
    }

    #[test]
    fn test_search_query_custom() {
        let json = r#"{"q": "test", "limit": 5, "offset": 10, "sort": "newest", "category": "web", "keyword": "async"}"#;
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.q, "test");
        assert_eq!(query.limit, 5);
        assert_eq!(query.offset, 10);
        assert_eq!(query.sort, "newest");
        assert_eq!(query.category, Some("web".to_string()));
        assert_eq!(query.keyword, Some("async".to_string()));
    }

    // ========== PublishRequest tests ==========

    #[test]
    fn test_publish_request_minimal() {
        let json = r#"{"name": "my-pkg", "version": "1.0.0"}"#;
        let req: PublishRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "my-pkg");
        assert_eq!(req.version, "1.0.0");
        assert!(req.description.is_none());
        assert!(req.keywords.is_empty());
        assert!(req.dependencies.is_empty());
        assert!(req.dev_dependencies.is_empty());
    }

    #[test]
    fn test_publish_request_full() {
        let json = r#"{
            "name": "my-pkg",
            "version": "2.0.0",
            "description": "A test package",
            "license": "MIT",
            "keywords": ["test", "vais"],
            "dependencies": {
                "foo": "^1.0",
                "bar": {"version": ">=2.0", "features": ["serde"]}
            }
        }"#;
        let req: PublishRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "my-pkg");
        assert_eq!(req.version, "2.0.0");
        assert_eq!(req.description, Some("A test package".to_string()));
        assert_eq!(req.license, Some("MIT".to_string()));
        assert_eq!(req.keywords, vec!["test", "vais"]);
        assert_eq!(req.dependencies.len(), 2);
        assert_eq!(req.dependencies["foo"].version_req(), "^1.0");
        assert_eq!(req.dependencies["bar"].version_req(), ">=2.0");
        assert_eq!(req.dependencies["bar"].features(), vec!["serde"]);
    }

    // ========== RegistryStats serialization ==========

    #[test]
    fn test_registry_stats_serialize() {
        let stats = RegistryStats {
            total_packages: 42,
            total_downloads: 1000,
            total_versions: 100,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("42"));
        assert!(json.contains("1000"));
        assert!(json.contains("100"));
    }

    // ========== Request/Response types ==========

    #[test]
    fn test_create_user_request() {
        let json = r#"{"username": "test", "password": "pass123"}"#;
        let req: CreateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "test");
        assert_eq!(req.password, "pass123");
        assert!(req.email.is_none());
    }

    #[test]
    fn test_login_request() {
        let json = r#"{"username": "test", "password": "pass"}"#;
        let req: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "test");
        assert_eq!(req.password, "pass");
    }

    #[test]
    fn test_create_token_request() {
        let json = r#"{"name": "ci-token", "scopes": ["publish"], "expires_in_days": 30}"#;
        let req: CreateTokenRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "ci-token");
        assert_eq!(req.scopes, vec!["publish"]);
        assert_eq!(req.expires_in_days, Some(30));
    }

    #[test]
    fn test_create_token_request_defaults() {
        let json = r#"{"name": "my-token"}"#;
        let req: CreateTokenRequest = serde_json::from_str(json).unwrap();
        assert!(req.scopes.is_empty());
        assert!(req.expires_in_days.is_none());
    }
}
