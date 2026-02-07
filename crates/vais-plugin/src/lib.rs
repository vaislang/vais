//! Vais Plugin System
//!
//! Provides a plugin infrastructure for extending the Vais compiler.
//!
//! # Plugin Types
//!
//! - **Lint**: Check code for issues, return diagnostics
//! - **Transform**: Modify the AST before type checking
//! - **Optimize**: Apply custom LLVM IR optimizations
//! - **Codegen**: Generate additional output files
//!
//! # Creating a Plugin
//!
//! Plugins are shared libraries (.so/.dylib/.dll) that export two functions:
//!
//! ```ignore
//! use vais_plugin::{Plugin, PluginInfo, PluginType, LintPlugin};
//!
//! pub struct MyLintPlugin;
//!
//! impl Plugin for MyLintPlugin {
//!     fn info(&self) -> PluginInfo {
//!         PluginInfo {
//!             name: "my-lint",
//!             version: "0.1.0",
//!             description: "My custom lint plugin",
//!         }
//!     }
//!
//!     fn as_any(&self) -> &dyn std::any::Any { self }
//!     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
//! }
//!
//! impl LintPlugin for MyLintPlugin {
//!     fn check(&self, module: &vais_ast::Module) -> Vec<Diagnostic> {
//!         vec![]
//!     }
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn create_plugin() -> *mut dyn Plugin {
//!     Box::into_raw(Box::new(MyLintPlugin))
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn get_plugin_type() -> PluginType {
//!     PluginType::Lint
//! }
//! ```
//!
//! # Configuration
//!
//! Create a `vais-plugins.toml` file in your project:
//!
//! ```toml
//! [plugins]
//! path = ["./plugins/my-lint.dylib"]
//!
//! [plugins.config]
//! my-lint = { max_warnings = 10 }
//! ```

mod config;
mod loader;
mod registry;
mod traits;

pub use config::{find_config, load_default, PluginsConfig, PluginsSection};
pub use loader::{is_plugin_library, library_extension, load_plugin, LoadedPlugin};
pub use registry::PluginRegistry;
pub use traits::{
    AnalysisPlugin, CodegenPlugin, ComplexityReport, DependencyGraph, DependencyInfo, Diagnostic,
    DiagnosticLevel, FormatConfig, FormatterPlugin, LintPlugin, OptLevel, OptimizePlugin, Plugin,
    PluginConfig, PluginInfo, PluginType, TransformPlugin,
};

/// Re-export vais_ast for plugin authors
pub use vais_ast;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registry_workflow() {
        // Create empty registry
        let registry = PluginRegistry::new();
        assert!(registry.is_empty());

        // Run lint on empty module
        let module = vais_ast::Module { items: vec![], modules_map: None };
        let diagnostics = registry.run_lint(&module);
        assert!(diagnostics.is_empty());

        // Run transform on empty module
        let transformed = registry.run_transform(module).unwrap();
        assert!(transformed.items.is_empty());

        // Run optimize on IR
        let ir = "; test";
        let optimized = registry.run_optimize(ir, OptLevel::O2).unwrap();
        assert_eq!(optimized, ir);
    }

    #[test]
    fn test_diagnostic_creation() {
        let warning = Diagnostic::warning("test warning");
        assert_eq!(warning.level, DiagnosticLevel::Warning);
        assert_eq!(warning.message, "test warning");

        let error = Diagnostic::error("test error")
            .with_span(vais_ast::Span::new(0, 10))
            .with_help("try this");
        assert_eq!(error.level, DiagnosticLevel::Error);
        assert!(error.span.is_some());
        assert_eq!(error.help, Some("try this".to_string()));
    }
}
