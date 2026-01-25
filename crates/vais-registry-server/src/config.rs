//! Server configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Database file path
    #[serde(default = "default_database_path")]
    pub database_path: PathBuf,

    /// Package storage directory
    #[serde(default = "default_storage_path")]
    pub storage_path: PathBuf,

    /// Maximum upload size in bytes (default: 50MB)
    #[serde(default = "default_max_upload_size")]
    pub max_upload_size: usize,

    /// API token expiration in days (default: 365)
    #[serde(default = "default_token_expiration_days")]
    pub token_expiration_days: u32,

    /// Enable CORS for all origins
    #[serde(default)]
    pub cors_allow_all: bool,

    /// Allowed CORS origins
    #[serde(default)]
    pub cors_origins: Vec<String>,

    /// Enable request logging
    #[serde(default = "default_true")]
    pub enable_logging: bool,

    /// Admin username (for initial setup)
    pub admin_username: Option<String>,

    /// Admin password (for initial setup)
    pub admin_password: Option<String>,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_database_path() -> PathBuf {
    PathBuf::from("./data/registry.db")
}

fn default_storage_path() -> PathBuf {
    PathBuf::from("./data/packages")
}

fn default_max_upload_size() -> usize {
    50 * 1024 * 1024 // 50MB
}

fn default_token_expiration_days() -> u32 {
    365
}

fn default_true() -> bool {
    true
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            database_path: default_database_path(),
            storage_path: default_storage_path(),
            max_upload_size: default_max_upload_size(),
            token_expiration_days: default_token_expiration_days(),
            cors_allow_all: false,
            cors_origins: vec![],
            enable_logging: true,
            admin_username: None,
            admin_password: None,
        }
    }
}

impl ServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            host: std::env::var("VAIS_REGISTRY_HOST").unwrap_or_else(|_| default_host()),
            port: std::env::var("VAIS_REGISTRY_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_port),
            database_path: std::env::var("VAIS_REGISTRY_DB")
                .map(PathBuf::from)
                .unwrap_or_else(|_| default_database_path()),
            storage_path: std::env::var("VAIS_REGISTRY_STORAGE")
                .map(PathBuf::from)
                .unwrap_or_else(|_| default_storage_path()),
            max_upload_size: std::env::var("VAIS_REGISTRY_MAX_UPLOAD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_upload_size),
            token_expiration_days: std::env::var("VAIS_REGISTRY_TOKEN_EXPIRY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_token_expiration_days),
            cors_allow_all: std::env::var("VAIS_REGISTRY_CORS_ALL")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false),
            cors_origins: std::env::var("VAIS_REGISTRY_CORS_ORIGINS")
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default(),
            enable_logging: std::env::var("VAIS_REGISTRY_LOGGING")
                .map(|s| s != "false" && s != "0")
                .unwrap_or(true),
            admin_username: std::env::var("VAIS_REGISTRY_ADMIN_USER").ok(),
            admin_password: std::env::var("VAIS_REGISTRY_ADMIN_PASS").ok(),
        }
    }

    /// Load configuration from TOML file
    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Get the bind address
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
