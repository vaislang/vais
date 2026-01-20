//! Plugin configuration file parsing
//!
//! Parses `vais-plugins.toml` configuration files.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginsConfig {
    /// Plugins section
    #[serde(default)]
    pub plugins: PluginsSection,
}

/// Plugins section of the config
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginsSection {
    /// Paths to plugin libraries
    #[serde(default)]
    pub path: Vec<PathBuf>,

    /// Names of installed plugins to enable
    #[serde(default)]
    pub enabled: Vec<String>,

    /// Per-plugin configuration
    #[serde(default)]
    pub config: HashMap<String, toml::Value>,
}

impl PluginsConfig {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read config file '{}': {}", path.display(), e))?;

        Self::parse(&content)
    }

    /// Parse configuration from a string
    pub fn parse(content: &str) -> Result<Self, String> {
        toml::from_str(content).map_err(|e| format!("Invalid config: {}", e))
    }

    /// Create an empty configuration
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if no plugins are configured
    pub fn is_empty(&self) -> bool {
        self.plugins.path.is_empty() && self.plugins.enabled.is_empty()
    }
}

/// Find the plugin configuration file
///
/// Searches in the current directory and parent directories for:
/// - `vais-plugins.toml`
/// - `.vais-plugins.toml`
pub fn find_config() -> Option<PathBuf> {
    let names = ["vais-plugins.toml", ".vais-plugins.toml"];
    let mut dir = std::env::current_dir().ok()?;

    loop {
        for name in &names {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
        if !dir.pop() {
            break;
        }
    }

    None
}

/// Load configuration from the default location
///
/// Returns an empty config if no config file is found.
pub fn load_default() -> PluginsConfig {
    find_config()
        .and_then(|path| PluginsConfig::load(&path).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let config = PluginsConfig::parse("").unwrap();
        assert!(config.is_empty());
    }

    #[test]
    fn test_parse_plugins_section() {
        let toml = r#"
[plugins]
path = ["./plugins/lint.dylib", "./plugins/optimizer.dylib"]
enabled = ["vais-lint-complexity"]
"#;

        let config = PluginsConfig::parse(toml).unwrap();
        assert_eq!(config.plugins.path.len(), 2);
        assert_eq!(config.plugins.enabled.len(), 1);
    }

    #[test]
    fn test_parse_plugin_config() {
        let toml = r#"
[plugins]
path = ["./plugins/lint.dylib"]

[plugins.config]
lint = { max_complexity = 10 }
optimizer = { level = "aggressive" }
"#;

        let config = PluginsConfig::parse(toml).unwrap();
        assert_eq!(config.plugins.config.len(), 2);

        let lint_config = config.plugins.config.get("lint").unwrap();
        assert_eq!(
            lint_config.get("max_complexity").and_then(|v| v.as_integer()),
            Some(10)
        );
    }

    #[test]
    fn test_parse_invalid() {
        let result = PluginsConfig::parse("invalid [ toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_empty() {
        let empty = PluginsConfig::empty();
        assert!(empty.is_empty());

        let with_path = PluginsConfig {
            plugins: PluginsSection {
                path: vec![PathBuf::from("test.dylib")],
                ..Default::default()
            },
        };
        assert!(!with_path.is_empty());

        let with_enabled = PluginsConfig {
            plugins: PluginsSection {
                enabled: vec!["test".to_string()],
                ..Default::default()
            },
        };
        assert!(!with_enabled.is_empty());
    }
}
