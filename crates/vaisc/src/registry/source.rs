//! Registry source configuration
//!
//! Defines where packages can be fetched from.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Registry source type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RegistrySource {
    /// HTTP/HTTPS registry
    Http {
        /// Base URL of the registry
        url: String,
        /// Optional authentication token
        #[serde(default)]
        token: Option<String>,
    },
    /// Local filesystem registry
    Local {
        /// Path to the local registry
        path: PathBuf,
    },
    /// Git repository as registry
    Git {
        /// Git repository URL
        url: String,
        /// Branch or tag
        #[serde(default = "default_branch")]
        branch: String,
    },
}

fn default_branch() -> String {
    "main".to_string()
}

impl RegistrySource {
    /// Create an HTTP registry source
    pub fn http(url: impl Into<String>) -> Self {
        Self::Http {
            url: url.into(),
            token: None,
        }
    }

    /// Create an HTTP registry source with token
    pub fn http_with_token(url: impl Into<String>, token: impl Into<String>) -> Self {
        Self::Http {
            url: url.into(),
            token: Some(token.into()),
        }
    }

    /// Create a local registry source
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self::Local { path: path.into() }
    }

    /// Create a git registry source
    pub fn git(url: impl Into<String>) -> Self {
        Self::Git {
            url: url.into(),
            branch: default_branch(),
        }
    }

    /// Get a display name for this source
    pub fn name(&self) -> String {
        match self {
            Self::Http { url, .. } => {
                // Extract domain from URL
                url.trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .split('/')
                    .next()
                    .unwrap_or("http")
                    .to_string()
            }
            Self::Local { path } => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("local")
                .to_string(),
            Self::Git { url, .. } => url
                .rsplit('/')
                .next()
                .unwrap_or("git")
                .trim_end_matches(".git")
                .to_string(),
        }
    }

    /// Check if this is the default registry
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Http { url, .. } if url.contains("vais.dev"))
    }
}

impl Default for RegistrySource {
    fn default() -> Self {
        // Default to the official Vais registry (future)
        Self::Http {
            url: "https://registry.vais.dev".to_string(),
            token: None,
        }
    }
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryConfig {
    /// Default registry for package lookups
    #[serde(default)]
    pub default: RegistrySource,
    /// Additional registries
    #[serde(default)]
    pub registries: Vec<NamedRegistry>,
}

/// Named registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedRegistry {
    /// Registry name
    pub name: String,
    /// Registry source
    pub source: RegistrySource,
}

impl RegistryConfig {
    /// Create config with only a default registry
    pub fn with_default(source: RegistrySource) -> Self {
        Self {
            default: source,
            registries: Vec::new(),
        }
    }

    /// Add a named registry
    pub fn add_registry(&mut self, name: impl Into<String>, source: RegistrySource) {
        self.registries.push(NamedRegistry {
            name: name.into(),
            source,
        });
    }

    /// Get a registry by name (or default if name is None)
    pub fn get(&self, name: Option<&str>) -> &RegistrySource {
        if let Some(name) = name {
            self.registries
                .iter()
                .find(|r| r.name == name)
                .map(|r| &r.source)
                .unwrap_or(&self.default)
        } else {
            &self.default
        }
    }

    /// Load from TOML string
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_source_name() {
        let http = RegistrySource::http("https://registry.vais.dev/api");
        assert_eq!(http.name(), "registry.vais.dev");

        let local = RegistrySource::local("/home/user/my-registry");
        assert_eq!(local.name(), "my-registry");

        let git = RegistrySource::git("https://github.com/vais-lang/packages.git");
        assert_eq!(git.name(), "packages");
    }

    #[test]
    fn test_registry_config() {
        let mut config = RegistryConfig::default();
        config.add_registry("local", RegistrySource::local("/tmp/registry"));

        let default = config.get(None);
        assert!(matches!(default, RegistrySource::Http { .. }));

        let local = config.get(Some("local"));
        assert!(matches!(local, RegistrySource::Local { .. }));
    }

    #[test]
    fn test_registry_source_http() {
        let src = RegistrySource::http("https://example.com");
        match &src {
            RegistrySource::Http { url, token } => {
                assert_eq!(url, "https://example.com");
                assert!(token.is_none());
            }
            _ => panic!("Expected Http variant"),
        }
    }

    #[test]
    fn test_registry_source_http_with_token() {
        let src = RegistrySource::http_with_token("https://example.com", "my-token");
        match &src {
            RegistrySource::Http { url, token } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(token.as_deref(), Some("my-token"));
            }
            _ => panic!("Expected Http variant"),
        }
    }

    #[test]
    fn test_registry_source_local() {
        let src = RegistrySource::local("/my/registry");
        match &src {
            RegistrySource::Local { path } => {
                assert_eq!(path, &PathBuf::from("/my/registry"));
            }
            _ => panic!("Expected Local variant"),
        }
    }

    #[test]
    fn test_registry_source_git() {
        let src = RegistrySource::git("https://github.com/org/repo.git");
        match &src {
            RegistrySource::Git { url, branch } => {
                assert_eq!(url, "https://github.com/org/repo.git");
                assert_eq!(branch, "main");
            }
            _ => panic!("Expected Git variant"),
        }
    }

    #[test]
    fn test_registry_source_name_http() {
        let src = RegistrySource::http("https://registry.vais.dev/api/v1");
        assert_eq!(src.name(), "registry.vais.dev");
    }

    #[test]
    fn test_registry_source_name_http_without_scheme() {
        let src = RegistrySource::http("http://localhost:8080/api");
        assert_eq!(src.name(), "localhost:8080");
    }

    #[test]
    fn test_registry_source_name_local() {
        let src = RegistrySource::local("/home/user/my-registry");
        assert_eq!(src.name(), "my-registry");
    }

    #[test]
    fn test_registry_source_name_git() {
        let src = RegistrySource::git("https://github.com/org/vais-packages.git");
        assert_eq!(src.name(), "vais-packages");
    }

    #[test]
    fn test_registry_source_name_git_no_extension() {
        let src = RegistrySource::git("https://github.com/org/packages");
        assert_eq!(src.name(), "packages");
    }

    #[test]
    fn test_registry_source_is_default() {
        let default = RegistrySource::default();
        assert!(default.is_default());

        let custom = RegistrySource::http("https://custom.com");
        assert!(!custom.is_default());
    }

    #[test]
    fn test_registry_source_default() {
        let src = RegistrySource::default();
        match &src {
            RegistrySource::Http { url, token } => {
                assert!(url.contains("vais.dev"));
                assert!(token.is_none());
            }
            _ => panic!("Expected Http variant"),
        }
    }

    #[test]
    fn test_registry_config_with_default() {
        let config = RegistryConfig::with_default(RegistrySource::local("/tmp"));
        assert!(matches!(config.get(None), RegistrySource::Local { .. }));
        assert!(config.registries.is_empty());
    }

    #[test]
    fn test_registry_config_get_nonexistent_falls_back() {
        let config = RegistryConfig::default();
        let src = config.get(Some("nonexistent"));
        // Falls back to default
        assert!(matches!(src, RegistrySource::Http { .. }));
    }

    #[test]
    fn test_registry_config_add_multiple() {
        let mut config = RegistryConfig::default();
        config.add_registry("local", RegistrySource::local("/tmp/local"));
        config.add_registry("staging", RegistrySource::http("https://staging.vais.dev"));

        assert_eq!(config.registries.len(), 2);
        assert!(matches!(
            config.get(Some("local")),
            RegistrySource::Local { .. }
        ));
        assert!(matches!(
            config.get(Some("staging")),
            RegistrySource::Http { .. }
        ));
    }
}
