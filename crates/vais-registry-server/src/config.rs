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
                .or_else(|_| std::env::var("PORT")) // Fly.io sets PORT
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.database_path, PathBuf::from("./data/registry.db"));
        assert_eq!(config.storage_path, PathBuf::from("./data/packages"));
        assert_eq!(config.max_upload_size, 50 * 1024 * 1024);
        assert_eq!(config.token_expiration_days, 365);
        assert!(!config.cors_allow_all);
        assert!(config.cors_origins.is_empty());
        assert!(config.enable_logging);
        assert!(config.admin_username.is_none());
        assert!(config.admin_password.is_none());
    }

    #[test]
    fn test_bind_addr() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_addr(), "0.0.0.0:3000");
    }

    #[test]
    fn test_bind_addr_custom() {
        let mut config = ServerConfig::default();
        config.host = "127.0.0.1".to_string();
        config.port = 8080;
        assert_eq!(config.bind_addr(), "127.0.0.1:8080");
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = ServerConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: ServerConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.host, config.host);
        assert_eq!(parsed.port, config.port);
        assert_eq!(parsed.max_upload_size, config.max_upload_size);
    }

    #[test]
    fn test_config_serde_with_custom_values() {
        let toml_str = r#"
            host = "localhost"
            port = 9090
            max_upload_size = 1024
            token_expiration_days = 7
            cors_allow_all = true
            enable_logging = false
        "#;
        let config: ServerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 9090);
        assert_eq!(config.max_upload_size, 1024);
        assert_eq!(config.token_expiration_days, 7);
        assert!(config.cors_allow_all);
        assert!(!config.enable_logging);
    }

    #[test]
    fn test_config_from_file_not_found() {
        let result = ServerConfig::from_file(std::path::Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_clone() {
        let config = ServerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.host, cloned.host);
        assert_eq!(config.port, cloned.port);
    }
}
