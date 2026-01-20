//! Plugin registry
//!
//! Manages loaded plugins and executes them in the correct order.

use crate::config::PluginsConfig;
use crate::loader::{load_plugin, LoadedPlugin};
use crate::traits::{Diagnostic, DiagnosticLevel, OptLevel, PluginConfig, PluginType};
use std::path::{Path, PathBuf};
use vais_ast::Module;

/// Plugin registry that manages all loaded plugins
pub struct PluginRegistry {
    /// All loaded plugins
    plugins: Vec<LoadedPlugin>,
}

impl PluginRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Load plugins from a configuration
    pub fn load_from_config(&mut self, config: &PluginsConfig) -> Result<(), String> {
        // Load plugins from paths
        for path in &config.plugins.path {
            self.load_plugin_file(path, config)?;
        }

        // TODO: Load plugins by name from installed location
        // for name in &config.plugins.enabled {
        //     self.load_plugin_by_name(name, config)?;
        // }

        Ok(())
    }

    /// Load a plugin from a file
    pub fn load_plugin_file(&mut self, path: &Path, config: &PluginsConfig) -> Result<(), String> {
        let mut loaded = load_plugin(path)?;

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
        let mut loaded = load_plugin(path)?;
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
        self.plugins
            .iter()
            .map(|p| p.plugin.info().name)
            .collect()
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

    /// Get plugin counts by type
    pub fn counts_by_type(&self) -> (usize, usize, usize, usize) {
        let mut lint = 0;
        let mut transform = 0;
        let mut optimize = 0;
        let mut codegen = 0;

        for loaded in &self.plugins {
            match loaded.plugin_type {
                PluginType::Lint => lint += 1,
                PluginType::Transform => transform += 1,
                PluginType::Optimize => optimize += 1,
                PluginType::Codegen => codegen += 1,
            }
        }

        (lint, transform, optimize, codegen)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
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
        let module = Module { items: vec![] };
        let diagnostics = registry.run_lint(&module);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_run_transform_empty() {
        let registry = PluginRegistry::new();
        let module = Module { items: vec![] };
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
        let no_errors = vec![
            Diagnostic::warning("test"),
            Diagnostic::info("test"),
        ];
        assert!(!PluginRegistry::has_errors(&no_errors));

        let with_errors = vec![
            Diagnostic::warning("test"),
            Diagnostic::error("error"),
        ];
        assert!(PluginRegistry::has_errors(&with_errors));
    }
}
