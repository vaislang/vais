//! Plugin auto-discovery system
//!
//! Automatically discovers plugins from standard paths and environment variables.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{DynloadError, Result};
use crate::manifest::{PluginFormat, PluginManifest};

/// Source of a discovered plugin
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginSource {
    /// User plugins directory (~/.vais/plugins/)
    UserDir,
    /// System plugins directory (/usr/local/lib/vais/plugins/)
    SystemDir,
    /// Custom path from VAIS_PLUGIN_PATH
    CustomPath(PathBuf),
    /// Explicitly specified path
    Explicit(PathBuf),
}

impl PluginSource {
    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            PluginSource::UserDir => "User plugins directory".to_string(),
            PluginSource::SystemDir => "System plugins directory".to_string(),
            PluginSource::CustomPath(p) => format!("Custom path: {}", p.display()),
            PluginSource::Explicit(p) => format!("Explicit path: {}", p.display()),
        }
    }
}

/// Information about a discovered plugin
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Path to the plugin directory
    pub path: PathBuf,
    /// Source of discovery
    pub source: PluginSource,
    /// Entry point file path
    pub entry_path: PathBuf,
}

impl DiscoveredPlugin {
    /// Get the plugin name
    pub fn name(&self) -> &str {
        &self.manifest.plugin.name
    }

    /// Get the plugin version
    pub fn version(&self) -> &str {
        &self.manifest.plugin.version
    }

    /// Get the plugin format
    pub fn format(&self) -> PluginFormat {
        self.manifest.plugin.format
    }

    /// Check if plugin is compatible with the given Vais version
    pub fn is_compatible_with(&self, vais_version: &str) -> Result<bool> {
        self.manifest.is_compatible_with(vais_version)
    }
}

/// Configuration for plugin discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Search user directory
    pub search_user_dir: bool,
    /// Search system directories
    pub search_system_dirs: bool,
    /// Search VAIS_PLUGIN_PATH
    pub search_env_path: bool,
    /// Additional search paths
    pub additional_paths: Vec<PathBuf>,
    /// File extension filter (None = all supported)
    pub format_filter: Option<PluginFormat>,
    /// Vais version for compatibility check
    pub vais_version: Option<String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            search_user_dir: true,
            search_system_dirs: true,
            search_env_path: true,
            additional_paths: vec![],
            format_filter: None,
            vais_version: None,
        }
    }
}

impl DiscoveryConfig {
    /// Create a new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable user directory search
    pub fn without_user_dir(mut self) -> Self {
        self.search_user_dir = false;
        self
    }

    /// Disable system directory search
    pub fn without_system_dirs(mut self) -> Self {
        self.search_system_dirs = false;
        self
    }

    /// Disable environment path search
    pub fn without_env_path(mut self) -> Self {
        self.search_env_path = false;
        self
    }

    /// Add an additional search path
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.additional_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Filter by plugin format
    pub fn with_format(mut self, format: PluginFormat) -> Self {
        self.format_filter = Some(format);
        self
    }

    /// Set Vais version for compatibility checking
    pub fn with_vais_version(mut self, version: &str) -> Self {
        self.vais_version = Some(version.to_string());
        self
    }
}

/// Plugin discovery service
pub struct PluginDiscovery {
    /// Discovery configuration
    config: DiscoveryConfig,
    /// Cached discoveries
    cache: HashMap<String, DiscoveredPlugin>,
}

impl PluginDiscovery {
    /// Create a new plugin discovery service with default configuration
    pub fn new() -> Self {
        Self::with_config(DiscoveryConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }

    /// Get search directories based on configuration
    pub fn search_directories(&self) -> Vec<(PathBuf, PluginSource)> {
        let mut dirs = Vec::new();

        // User plugins directory: ~/.vais/plugins/
        if self.config.search_user_dir {
            if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
                let user_plugins = PathBuf::from(home).join(".vais").join("plugins");
                if user_plugins.exists() {
                    dirs.push((user_plugins, PluginSource::UserDir));
                }
            }
        }

        // System plugins directories (Unix only)
        if self.config.search_system_dirs {
            #[cfg(unix)]
            {
                let system_dirs = [
                    "/usr/local/lib/vais/plugins",
                    "/usr/lib/vais/plugins",
                    "/opt/vais/plugins",
                ];

                for dir in &system_dirs {
                    let path = PathBuf::from(dir);
                    if path.exists() {
                        dirs.push((path, PluginSource::SystemDir));
                    }
                }
            }

            // Windows system directories
            #[cfg(windows)]
            {
                if let Some(program_files) = env::var_os("ProgramFiles") {
                    let path = PathBuf::from(program_files).join("Vais").join("plugins");
                    if path.exists() {
                        dirs.push((path, PluginSource::SystemDir));
                    }
                }
            }
        }

        // VAIS_PLUGIN_PATH environment variable
        if self.config.search_env_path {
            if let Ok(plugin_path) = env::var("VAIS_PLUGIN_PATH") {
                let separator = if cfg!(windows) { ';' } else { ':' };
                for path in plugin_path.split(separator) {
                    if !path.is_empty() {
                        let path_buf = PathBuf::from(path);
                        if path_buf.exists() {
                            dirs.push((path_buf.clone(), PluginSource::CustomPath(path_buf)));
                        }
                    }
                }
            }
        }

        // Additional paths
        for path in &self.config.additional_paths {
            if path.exists() {
                dirs.push((path.clone(), PluginSource::Explicit(path.clone())));
            }
        }

        dirs
    }

    /// Scan all configured directories for plugins
    pub fn scan_all(&mut self) -> Result<Vec<DiscoveredPlugin>> {
        let mut plugins = Vec::new();
        let search_dirs = self.search_directories();

        for (dir, source) in search_dirs {
            if let Ok(discovered) = self.scan_directory(&dir, source) {
                plugins.extend(discovered);
            }
        }

        // Filter by format if specified
        if let Some(format) = self.config.format_filter {
            plugins.retain(|p| p.format() == format);
        }

        // Filter by version compatibility if specified
        if let Some(ref version) = self.config.vais_version {
            plugins.retain(|p| p.is_compatible_with(version).unwrap_or(true));
        }

        // Update cache
        for plugin in &plugins {
            self.cache.insert(plugin.name().to_string(), plugin.clone());
        }

        Ok(plugins)
    }

    /// Scan a specific directory for plugins
    pub fn scan_directory<P: AsRef<Path>>(
        &self,
        dir: P,
        source: PluginSource,
    ) -> Result<Vec<DiscoveredPlugin>> {
        let dir = dir.as_ref();
        let mut plugins = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(plugins);
        }

        // Read directory entries
        let entries = fs::read_dir(dir).map_err(|e| {
            DynloadError::DiscoveryError(format!(
                "Failed to read directory '{}': {}",
                dir.display(),
                e
            ))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Check for plugin.toml in subdirectory
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    if let Ok(plugin) =
                        self.load_plugin_from_manifest(&manifest_path, source.clone())
                    {
                        plugins.push(plugin);
                    }
                }
            } else if path.is_file() {
                // Check for standalone plugin files
                if let Some(plugin) = self.detect_standalone_plugin(&path, source.clone())? {
                    plugins.push(plugin);
                }
            }
        }

        Ok(plugins)
    }

    /// Load a plugin from its manifest file
    pub fn load_plugin_from_manifest<P: AsRef<Path>>(
        &self,
        manifest_path: P,
        source: PluginSource,
    ) -> Result<DiscoveredPlugin> {
        let manifest_path = manifest_path.as_ref();
        let manifest = PluginManifest::load(manifest_path)?;

        let plugin_dir = manifest_path
            .parent()
            .ok_or_else(|| DynloadError::ManifestError("Invalid manifest path".to_string()))?
            .to_path_buf();

        // Determine entry point
        let entry_path = if let Some(entry) = manifest.entry_point() {
            plugin_dir.join(entry)
        } else {
            // Try to find default entry point based on format
            match manifest.plugin.format {
                PluginFormat::Wasm => self.find_entry_file(&plugin_dir, &["wasm"])?,
                PluginFormat::Native => {
                    self.find_entry_file(&plugin_dir, &["so", "dylib", "dll"])?
                }
                PluginFormat::Vais => self.find_entry_file(&plugin_dir, &["vais"])?,
            }
        };

        Ok(DiscoveredPlugin {
            manifest,
            path: plugin_dir,
            source,
            entry_path,
        })
    }

    /// Detect a standalone plugin file (without manifest)
    fn detect_standalone_plugin(
        &self,
        path: &Path,
        source: PluginSource,
    ) -> Result<Option<DiscoveredPlugin>> {
        let ext = path.extension().and_then(|e| e.to_str());

        let format = match ext {
            Some("wasm") => PluginFormat::Wasm,
            Some("so") | Some("dylib") | Some("dll") => PluginFormat::Native,
            Some("vais") => PluginFormat::Vais,
            _ => return Ok(None),
        };

        // Create a minimal manifest for the standalone plugin
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut manifest = PluginManifest::default();
        manifest.plugin.name = name;
        manifest.plugin.version = "0.0.0".to_string();
        manifest.plugin.format = format;
        manifest.plugin.entry = Some(
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
        );

        Ok(Some(DiscoveredPlugin {
            manifest,
            path: path.parent().unwrap_or(Path::new(".")).to_path_buf(),
            source,
            entry_path: path.to_path_buf(),
        }))
    }

    /// Find an entry file with given extensions
    fn find_entry_file(&self, dir: &Path, extensions: &[&str]) -> Result<PathBuf> {
        for ext in extensions {
            let entries = fs::read_dir(dir).map_err(|e| {
                DynloadError::DiscoveryError(format!(
                    "Failed to read directory '{}': {}",
                    dir.display(),
                    e
                ))
            })?;

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                    return Ok(path);
                }
            }
        }

        Err(DynloadError::ManifestError(format!(
            "No entry file found in '{}'",
            dir.display()
        )))
    }

    /// Get a plugin by name from cache
    pub fn get_cached(&self, name: &str) -> Option<&DiscoveredPlugin> {
        self.cache.get(name)
    }

    /// Clear the plugin cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get all cached plugins
    pub fn cached_plugins(&self) -> Vec<&DiscoveredPlugin> {
        self.cache.values().collect()
    }

    /// Search for a specific plugin by name
    pub fn find_plugin(&mut self, name: &str) -> Result<Option<DiscoveredPlugin>> {
        // Check cache first
        if let Some(plugin) = self.cache.get(name) {
            return Ok(Some(plugin.clone()));
        }

        // Scan all directories
        let search_dirs = self.search_directories();

        for (dir, source) in search_dirs {
            // Check for plugin directory with matching name
            let plugin_dir = dir.join(name);
            if plugin_dir.exists() && plugin_dir.is_dir() {
                let manifest_path = plugin_dir.join("plugin.toml");
                if manifest_path.exists() {
                    let plugin = self.load_plugin_from_manifest(&manifest_path, source)?;
                    self.cache.insert(name.to_string(), plugin.clone());
                    return Ok(Some(plugin));
                }
            }

            // Check for standalone files with matching name
            for ext in ["wasm", "so", "dylib", "dll", "vais"] {
                let file_path = dir.join(format!("{}.{}", name, ext));
                if file_path.exists() {
                    if let Some(plugin) =
                        self.detect_standalone_plugin(&file_path, source.clone())?
                    {
                        self.cache.insert(name.to_string(), plugin.clone());
                        return Ok(Some(plugin));
                    }
                }

                // Also check lib prefix
                let file_path = dir.join(format!("lib{}.{}", name, ext));
                if file_path.exists() {
                    if let Some(plugin) =
                        self.detect_standalone_plugin(&file_path, source.clone())?
                    {
                        self.cache.insert(name.to_string(), plugin.clone());
                        return Ok(Some(plugin));
                    }
                }
            }
        }

        Ok(None)
    }
}

impl Default for PluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(config.search_user_dir);
        assert!(config.search_system_dirs);
        assert!(config.search_env_path);
        assert!(config.additional_paths.is_empty());
    }

    #[test]
    fn test_discovery_config_builder() {
        let config = DiscoveryConfig::new()
            .without_user_dir()
            .without_system_dirs()
            .with_path("/custom/path")
            .with_format(PluginFormat::Wasm)
            .with_vais_version("0.1.0");

        assert!(!config.search_user_dir);
        assert!(!config.search_system_dirs);
        assert_eq!(config.additional_paths.len(), 1);
        assert_eq!(config.format_filter, Some(PluginFormat::Wasm));
        assert_eq!(config.vais_version, Some("0.1.0".to_string()));
    }

    #[test]
    fn test_plugin_source_description() {
        assert!(PluginSource::UserDir.description().contains("User"));
        assert!(PluginSource::SystemDir.description().contains("System"));
        assert!(PluginSource::CustomPath(PathBuf::from("/test"))
            .description()
            .contains("Custom"));
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let discovery = PluginDiscovery::with_config(
            DiscoveryConfig::new()
                .without_user_dir()
                .without_system_dirs()
                .without_env_path()
                .with_path(temp_dir.path()),
        );

        let result = discovery.scan_directory(temp_dir.path(), PluginSource::UserDir);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_scan_directory_with_manifest() {
        let temp_dir = TempDir::new().unwrap();

        // Create a plugin directory with manifest
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        let manifest = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
format = "wasm"
entry = "plugin.wasm"
"#;

        fs::write(plugin_dir.join("plugin.toml"), manifest).unwrap();
        fs::write(plugin_dir.join("plugin.wasm"), "fake wasm").unwrap();

        let discovery = PluginDiscovery::new();
        let result = discovery.scan_directory(temp_dir.path(), PluginSource::UserDir);

        assert!(result.is_ok());
        let plugins = result.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name(), "test-plugin");
        assert_eq!(plugins[0].version(), "1.0.0");
    }

    #[test]
    fn test_detect_standalone_wasm_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("my-plugin.wasm");
        fs::write(&plugin_path, "fake wasm").unwrap();

        let discovery = PluginDiscovery::new();
        let result = discovery.detect_standalone_plugin(&plugin_path, PluginSource::UserDir);

        assert!(result.is_ok());
        let plugin = result.unwrap();
        assert!(plugin.is_some());

        let plugin = plugin.unwrap();
        assert_eq!(plugin.name(), "my-plugin");
        assert_eq!(plugin.format(), PluginFormat::Wasm);
    }

    #[test]
    fn test_find_plugin_not_found() {
        let mut discovery = PluginDiscovery::with_config(
            DiscoveryConfig::new()
                .without_user_dir()
                .without_system_dirs()
                .without_env_path(),
        );

        let result = discovery.find_plugin("nonexistent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_cache_operations() {
        let mut discovery = PluginDiscovery::new();
        assert!(discovery.cached_plugins().is_empty());

        // Manually add to cache for testing
        let manifest = PluginManifest::default();
        discovery.cache.insert(
            "test".to_string(),
            DiscoveredPlugin {
                manifest,
                path: PathBuf::from("/test"),
                source: PluginSource::UserDir,
                entry_path: PathBuf::from("/test/plugin.wasm"),
            },
        );

        assert!(discovery.get_cached("test").is_some());
        assert!(discovery.get_cached("other").is_none());
        assert_eq!(discovery.cached_plugins().len(), 1);

        discovery.clear_cache();
        assert!(discovery.cached_plugins().is_empty());
    }

    #[test]
    fn test_search_directories_with_env() {
        // Save original value
        let original = env::var("VAIS_PLUGIN_PATH").ok();

        // Create temp directory
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        // Set environment variable
        env::set_var("VAIS_PLUGIN_PATH", path);

        let discovery = PluginDiscovery::with_config(
            DiscoveryConfig::new()
                .without_user_dir()
                .without_system_dirs(),
        );

        let dirs = discovery.search_directories();
        assert!(dirs.iter().any(|(p, _)| p == temp_dir.path()));

        // Restore original value
        match original {
            Some(val) => env::set_var("VAIS_PLUGIN_PATH", val),
            None => env::remove_var("VAIS_PLUGIN_PATH"),
        }
    }
}
