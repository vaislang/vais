//! Plugin registry
//!
//! Manages loaded plugins and executes them in the correct order.

use crate::config::PluginsConfig;
use crate::loader::{load_plugin, LoadedPlugin};
use crate::traits::{
    ComplexityReport, DependencyGraph, Diagnostic, DiagnosticLevel, FormatConfig, OptLevel,
    PluginConfig, PluginType,
};
use std::env;
use std::path::{Path, PathBuf};
use vais_ast::Module;

/// Plugin registry that manages all loaded plugins
pub struct PluginRegistry {
    /// All loaded plugins
    plugins: Vec<LoadedPlugin>,
    /// Whether plugin loading is allowed (requires --allow-plugins flag)
    allow_plugins: bool,
}

impl PluginRegistry {
    /// Create an empty registry with plugins disabled by default
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            allow_plugins: false,
        }
    }

    /// Set whether plugin loading is allowed
    pub fn set_allow_plugins(&mut self, allow: bool) {
        self.allow_plugins = allow;
    }

    /// Create a new registry with plugin loading explicitly allowed
    pub fn new_with_plugins_allowed() -> Self {
        Self {
            plugins: Vec::new(),
            allow_plugins: true,
        }
    }

    /// Load plugins from a configuration
    pub fn load_from_config(&mut self, config: &PluginsConfig) -> Result<(), String> {
        // Load plugins from paths
        for path in &config.plugins.path {
            self.load_plugin_file(path, config)?;
        }

        // Load plugins by name from installed location
        for name in &config.plugins.enabled {
            self.load_plugin_by_name(name, config)?;
        }

        Ok(())
    }

    /// Get plugin search directories
    ///
    /// Returns directories where plugins may be installed:
    /// 1. ~/.vais/plugins/ (user plugins)
    /// 2. /usr/local/lib/vais/plugins/ (system plugins on Unix)
    /// 3. VAIS_PLUGIN_PATH environment variable (custom paths)
    fn plugin_search_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // User plugins directory: ~/.vais/plugins/
        if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
            let user_plugins = PathBuf::from(home).join(".vais").join("plugins");
            dirs.push(user_plugins);
        }

        // System plugins directory (Unix only)
        #[cfg(unix)]
        {
            dirs.push(PathBuf::from("/usr/local/lib/vais/plugins"));
            dirs.push(PathBuf::from("/usr/lib/vais/plugins"));
        }

        // Custom paths from VAIS_PLUGIN_PATH environment variable
        if let Ok(plugin_path) = env::var("VAIS_PLUGIN_PATH") {
            for path in plugin_path.split(':') {
                if !path.is_empty() {
                    dirs.push(PathBuf::from(path));
                }
            }
        }

        dirs
    }

    /// Get the platform-specific library extension
    fn library_extension() -> &'static str {
        #[cfg(target_os = "macos")]
        {
            "dylib"
        }
        #[cfg(target_os = "linux")]
        {
            "so"
        }
        #[cfg(target_os = "windows")]
        {
            "dll"
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            "so"
        }
    }

    /// Load a plugin by name from installed locations
    ///
    /// Searches for a plugin library file in standard installation directories.
    /// The library file should be named `lib{name}.{ext}` or `{name}.{ext}`.
    pub fn load_plugin_by_name(
        &mut self,
        name: &str,
        config: &PluginsConfig,
    ) -> Result<(), String> {
        let ext = Self::library_extension();
        let search_dirs = Self::plugin_search_dirs();

        // Possible filenames for the plugin
        let filenames = [
            format!("lib{}.{}", name, ext),
            format!("{}.{}", name, ext),
            format!("libvais_{}.{}", name, ext),
            format!("vais_{}.{}", name, ext),
        ];

        // Search for the plugin in all directories
        for dir in &search_dirs {
            if !dir.exists() {
                continue;
            }

            for filename in &filenames {
                let plugin_path = dir.join(filename);
                if plugin_path.exists() && plugin_path.is_file() {
                    return self.load_plugin_file(&plugin_path, config);
                }
            }
        }

        // Plugin not found - provide helpful error message
        let search_paths: Vec<_> = search_dirs
            .iter()
            .filter(|d| d.exists())
            .map(|d| d.display().to_string())
            .collect();

        let searched_msg = if search_paths.is_empty() {
            "No plugin directories found.".to_string()
        } else {
            format!("Searched in: {}", search_paths.join(", "))
        };

        Err(format!(
            "Plugin '{}' not found. {}\nExpected filename: lib{}.{} or {}.{}",
            name, searched_msg, name, ext, name, ext
        ))
    }

    /// Load a plugin from a file
    pub fn load_plugin_file(&mut self, path: &Path, config: &PluginsConfig) -> Result<(), String> {
        let mut loaded = load_plugin(path, self.allow_plugins)?;

        // Get plugin-specific config
        let plugin_name = loaded.plugin.info().name;
        let plugin_config = config
            .plugins
            .config
            .get(plugin_name)
            .map(|v| {
                let mut pc = PluginConfig::new();
                if let Some(table) = v.as_table() {
                    for (k, v) in table {
                        pc.values.insert(k.clone(), v.clone());
                    }
                }
                pc
            })
            .unwrap_or_default();

        // Initialize the plugin
        loaded
            .plugin
            .init(&plugin_config)
            .map_err(|e| format!("Failed to initialize plugin '{}': {}", plugin_name, e))?;

        self.plugins.push(loaded);
        Ok(())
    }

    /// Load a plugin from a path (without configuration)
    ///
    /// Returns the plugin info on success.
    pub fn load_from_path(&mut self, path: &Path) -> Result<crate::traits::PluginInfo, String> {
        let mut loaded = load_plugin(path, self.allow_plugins)?;
        let info = loaded.plugin.info();

        // Initialize with empty config
        loaded
            .plugin
            .init(&PluginConfig::new())
            .map_err(|e| format!("Failed to initialize plugin '{}': {}", info.name, e))?;

        let info_copy = crate::traits::PluginInfo {
            name: info.name,
            version: info.version,
            description: info.description,
        };

        self.plugins.push(loaded);
        Ok(info_copy)
    }

    /// Configure a plugin by name
    pub fn configure(&mut self, name: &str, config: &toml::Value) -> Result<(), String> {
        for loaded in &mut self.plugins {
            if loaded.plugin.info().name == name {
                let mut pc = PluginConfig::new();
                if let Some(table) = config.as_table() {
                    for (k, v) in table {
                        pc.values.insert(k.clone(), v.clone());
                    }
                }
                return loaded.plugin.init(&pc);
            }
        }
        Err(format!("Plugin '{}' not found", name))
    }

    /// Get the number of loaded plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if no plugins are loaded
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Get plugin names
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.plugin.info().name).collect()
    }

    /// Run all lint plugins on a module
    ///
    /// Returns all diagnostics from all lint plugins.
    pub fn run_lint(&self, module: &Module) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for loaded in &self.plugins {
            if let Some(lint) = loaded.as_lint() {
                diagnostics.extend(lint.check(module));
            }
        }

        diagnostics
    }

    /// Run all transform plugins on a module
    ///
    /// Transforms are applied in the order plugins were loaded.
    pub fn run_transform(&self, module: Module) -> Result<Module, String> {
        let mut result = module;

        for loaded in &self.plugins {
            if let Some(transform) = loaded.as_transform() {
                result = transform.transform(result)?;
            }
        }

        Ok(result)
    }

    /// Run all optimize plugins on LLVM IR
    ///
    /// Only runs plugins whose min_opt_level <= the given level.
    pub fn run_optimize(&self, ir: &str, level: OptLevel) -> Result<String, String> {
        let mut result = ir.to_string();

        for loaded in &self.plugins {
            if let Some(optimize) = loaded.as_optimize() {
                if optimize.min_opt_level() <= level {
                    result = optimize.optimize(&result)?;
                }
            }
        }

        Ok(result)
    }

    /// Run all codegen plugins
    ///
    /// Returns paths of all generated files.
    pub fn run_codegen(&self, module: &Module, output_dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();

        for loaded in &self.plugins {
            if let Some(codegen) = loaded.as_codegen() {
                files.extend(codegen.generate(module, output_dir)?);
            }
        }

        Ok(files)
    }

    /// Check if any diagnostics are errors
    pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
        diagnostics
            .iter()
            .any(|d| d.level == DiagnosticLevel::Error)
    }

    /// Run all formatter plugins on a module
    ///
    /// Returns the formatted source code. If multiple formatters are loaded,
    /// they are applied in sequence.
    pub fn run_format(&self, module: &Module, config: &FormatConfig) -> Result<String, String> {
        let mut result = String::new();
        let mut formatted = false;

        for loaded in &self.plugins {
            if let Some(formatter) = loaded.as_formatter() {
                result = formatter.format_module(module, config)?;
                formatted = true;
            }
        }

        if !formatted {
            return Err("No formatter plugins loaded".to_string());
        }

        Ok(result)
    }

    /// Run complexity analysis on a module
    ///
    /// Aggregates complexity reports from all analysis plugins.
    /// If no analysis plugins are loaded, returns an empty report.
    pub fn run_analysis_complexity(&self, module: &Module) -> ComplexityReport {
        let mut combined_report = ComplexityReport::new();

        for loaded in &self.plugins {
            if let Some(analysis) = loaded.as_analysis() {
                let report = analysis.analyze_complexity(module);

                // Aggregate function complexity
                for (name, complexity) in report.function_complexity {
                    combined_report.add_function(name, complexity);
                }

                // Aggregate suggestions
                for suggestion in report.suggestions {
                    combined_report.add_suggestion(suggestion);
                }
            }
        }

        combined_report
    }

    /// Run dependency analysis on a module
    ///
    /// Returns the dependency graph from the first analysis plugin found.
    /// If multiple analysis plugins are loaded, only the first one is used.
    /// If no analysis plugins are loaded, returns an empty graph.
    pub fn run_analysis_dependencies(&self, module: &Module) -> DependencyGraph {
        for loaded in &self.plugins {
            if let Some(analysis) = loaded.as_analysis() {
                return analysis.analyze_dependencies(module);
            }
        }

        DependencyGraph::new()
    }

    /// Get plugin counts by type
    pub fn counts_by_type(&self) -> (usize, usize, usize, usize, usize, usize) {
        let mut lint = 0;
        let mut transform = 0;
        let mut optimize = 0;
        let mut codegen = 0;
        let mut formatter = 0;
        let mut analysis = 0;

        for loaded in &self.plugins {
            match loaded.plugin_type {
                PluginType::Lint => lint += 1,
                PluginType::Transform => transform += 1,
                PluginType::Optimize => optimize += 1,
                PluginType::Codegen => codegen += 1,
                PluginType::Formatter => formatter += 1,
                PluginType::Analysis => analysis += 1,
            }
        }

        (lint, transform, optimize, codegen, formatter, analysis)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("plugin_count", &self.plugins.len())
            .field("allow_plugins", &self.allow_plugins)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = PluginRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_run_lint_empty() {
        let registry = PluginRegistry::new();
        let module = Module {
            items: vec![],
            modules_map: None,
        };
        let diagnostics = registry.run_lint(&module);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_run_transform_empty() {
        let registry = PluginRegistry::new();
        let module = Module {
            items: vec![],
            modules_map: None,
        };
        let result = registry.run_transform(module).unwrap();
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_run_optimize_empty() {
        let registry = PluginRegistry::new();
        let ir = "; test IR";
        let result = registry.run_optimize(ir, OptLevel::O2).unwrap();
        assert_eq!(result, ir);
    }

    #[test]
    fn test_has_errors() {
        let no_errors = vec![Diagnostic::warning("test"), Diagnostic::info("test")];
        assert!(!PluginRegistry::has_errors(&no_errors));

        let with_errors = vec![Diagnostic::warning("test"), Diagnostic::error("error")];
        assert!(PluginRegistry::has_errors(&with_errors));
    }

    #[test]
    fn test_library_extension() {
        let ext = PluginRegistry::library_extension();
        #[cfg(target_os = "macos")]
        assert_eq!(ext, "dylib");
        #[cfg(target_os = "linux")]
        assert_eq!(ext, "so");
        #[cfg(target_os = "windows")]
        assert_eq!(ext, "dll");
    }

    #[test]
    fn test_plugin_search_dirs_includes_home() {
        // Ensure HOME is set for the test
        let dirs = PluginRegistry::plugin_search_dirs();

        // Should include user plugins directory if HOME is set
        if env::var_os("HOME").is_some() || env::var_os("USERPROFILE").is_some() {
            assert!(dirs
                .iter()
                .any(|d| d.to_string_lossy().contains(".vais/plugins")));
        }
    }

    #[test]
    fn test_load_plugin_by_name_not_found() {
        let mut registry = PluginRegistry::new();
        let config = PluginsConfig::default();

        let result = registry.load_plugin_by_name("nonexistent_plugin", &config);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.contains("Plugin 'nonexistent_plugin' not found"));
        assert!(err.contains("Expected filename"));
    }

    #[test]
    fn test_plugin_search_dirs_with_env_var() {
        // Save original value
        let original = env::var("VAIS_PLUGIN_PATH").ok();

        // Set custom plugin path
        env::set_var("VAIS_PLUGIN_PATH", "/custom/path1:/custom/path2");

        let dirs = PluginRegistry::plugin_search_dirs();
        assert!(dirs.iter().any(|d| d == Path::new("/custom/path1")));
        assert!(dirs.iter().any(|d| d == Path::new("/custom/path2")));

        // Restore original value
        match original {
            Some(val) => env::set_var("VAIS_PLUGIN_PATH", val),
            None => env::remove_var("VAIS_PLUGIN_PATH"),
        }
    }

    #[test]
    fn test_run_format_empty() {
        use crate::traits::FormatConfig;

        let registry = PluginRegistry::new();
        let module = Module {
            items: vec![],
            modules_map: None,
        };
        let config = FormatConfig::new();

        let result = registry.run_format(&module, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No formatter plugins"));
    }

    #[test]
    fn test_run_analysis_complexity_empty() {
        let registry = PluginRegistry::new();
        let module = Module {
            items: vec![],
            modules_map: None,
        };

        let report = registry.run_analysis_complexity(&module);
        assert_eq!(report.overall_complexity, 0);
        assert!(report.function_complexity.is_empty());
        assert!(report.suggestions.is_empty());
    }

    #[test]
    fn test_run_analysis_dependencies_empty() {
        let registry = PluginRegistry::new();
        let module = Module {
            items: vec![],
            modules_map: None,
        };

        let graph = registry.run_analysis_dependencies(&module);
        assert!(graph.dependencies.is_empty());
        assert!(graph.dependents.is_empty());
        assert!(!graph.has_cycles());
    }

    #[test]
    fn test_counts_by_type_all_zeros() {
        let registry = PluginRegistry::new();
        let (lint, transform, optimize, codegen, formatter, analysis) = registry.counts_by_type();

        assert_eq!(lint, 0);
        assert_eq!(transform, 0);
        assert_eq!(optimize, 0);
        assert_eq!(codegen, 0);
        assert_eq!(formatter, 0);
        assert_eq!(analysis, 0);
    }
}
