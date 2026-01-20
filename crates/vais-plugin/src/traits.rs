//! Plugin trait definitions
//!
//! Defines the core traits that all plugins must implement.

use std::any::Any;
use std::path::Path;
use vais_ast::{Module, Span};

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: &'static str,
    /// Plugin version (semver)
    pub version: &'static str,
    /// Short description
    pub description: &'static str,
}

/// Plugin configuration passed during initialization
#[derive(Debug, Clone, Default)]
pub struct PluginConfig {
    /// Configuration values from vais-plugins.toml
    pub values: std::collections::HashMap<String, toml::Value>,
}

impl PluginConfig {
    /// Create an empty config
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a string value
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| v.as_str())
    }

    /// Get an integer value
    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.values.get(key).and_then(|v| v.as_integer())
    }

    /// Get a boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|v| v.as_bool())
    }
}

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// Hint message (lowest priority)
    Hint,
    /// Informational message
    Info,
    /// Warning (compilation continues)
    Warning,
    /// Error (compilation fails)
    Error,
}

/// A diagnostic message from a lint plugin
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level
    pub level: DiagnosticLevel,
    /// Error/warning message
    pub message: String,
    /// Source location (optional)
    pub span: Option<Span>,
    /// Help text (optional)
    pub help: Option<String>,
}

impl Diagnostic {
    /// Create a new warning diagnostic
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            span: None,
            help: None,
        }
    }

    /// Create a new error diagnostic
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            span: None,
            help: None,
        }
    }

    /// Create a new info diagnostic
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            message: message.into(),
            span: None,
            help: None,
        }
    }

    /// Set the span for this diagnostic
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Set the help text for this diagnostic
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Optimization level for optimize plugins
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    /// No optimization
    O0 = 0,
    /// Basic optimization
    O1 = 1,
    /// Standard optimization
    O2 = 2,
    /// Aggressive optimization
    O3 = 3,
}

/// Base trait for all plugins
///
/// All plugins must implement this trait to provide metadata and initialization.
pub trait Plugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> PluginInfo;

    /// Initialize the plugin with configuration
    ///
    /// Called once when the plugin is loaded. Configuration values
    /// come from the `[plugins.config]` section in vais-plugins.toml.
    fn init(&mut self, config: &PluginConfig) -> Result<(), String> {
        let _ = config;
        Ok(())
    }

    /// Return self as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Return self as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Lint plugin trait
///
/// Lint plugins check the AST for issues and return diagnostic messages.
/// They do not modify the AST.
///
/// # Example
///
/// ```ignore
/// impl LintPlugin for MyLinter {
///     fn check(&self, module: &Module) -> Vec<Diagnostic> {
///         // Check for issues and return diagnostics
///         vec![]
///     }
/// }
/// ```
pub trait LintPlugin: Plugin {
    /// Check the module and return diagnostics
    fn check(&self, module: &Module) -> Vec<Diagnostic>;
}

/// Transform plugin trait
///
/// Transform plugins modify the AST. They receive the AST and return
/// a modified version. Transforms run before type checking.
///
/// # Example
///
/// ```ignore
/// impl TransformPlugin for MyTransformer {
///     fn transform(&self, module: Module) -> Result<Module, String> {
///         // Modify and return the AST
///         Ok(module)
///     }
/// }
/// ```
pub trait TransformPlugin: Plugin {
    /// Transform the module AST
    fn transform(&self, module: Module) -> Result<Module, String>;
}

/// Optimize plugin trait
///
/// Optimize plugins modify the LLVM IR text. They run after code generation
/// and can apply custom optimization passes.
///
/// # Example
///
/// ```ignore
/// impl OptimizePlugin for MyOptimizer {
///     fn optimize(&self, ir: &str) -> Result<String, String> {
///         // Modify and return the IR
///         Ok(ir.to_string())
///     }
/// }
/// ```
pub trait OptimizePlugin: Plugin {
    /// Optimize the LLVM IR
    fn optimize(&self, ir: &str) -> Result<String, String>;

    /// Minimum optimization level at which this plugin runs
    ///
    /// Default is O2. Plugin will only run when compiler optimization
    /// level is >= this value.
    fn min_opt_level(&self) -> OptLevel {
        OptLevel::O2
    }
}

/// Codegen plugin trait
///
/// Codegen plugins generate additional output files from the AST.
/// They can be used for binding generation, documentation, etc.
///
/// # Example
///
/// ```ignore
/// impl CodegenPlugin for MyBindgen {
///     fn generate(&self, module: &Module, output_dir: &Path) -> Result<Vec<PathBuf>, String> {
///         // Generate files and return their paths
///         Ok(vec![])
///     }
/// }
/// ```
pub trait CodegenPlugin: Plugin {
    /// Generate additional files
    ///
    /// Returns the paths of generated files.
    fn generate(
        &self,
        module: &Module,
        output_dir: &Path,
    ) -> Result<Vec<std::path::PathBuf>, String>;
}

/// Plugin type enumeration for registry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum PluginType {
    Lint,
    Transform,
    Optimize,
    Codegen,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let diag = Diagnostic::warning("test warning")
            .with_span(Span::new(0, 10))
            .with_help("try this instead");

        assert_eq!(diag.level, DiagnosticLevel::Warning);
        assert_eq!(diag.message, "test warning");
        assert!(diag.span.is_some());
        assert_eq!(diag.help, Some("try this instead".to_string()));
    }

    #[test]
    fn test_plugin_config() {
        let mut config = PluginConfig::new();
        config.values.insert("max_complexity".to_string(), toml::Value::Integer(10));
        config.values.insert("enabled".to_string(), toml::Value::Boolean(true));
        config.values.insert("output".to_string(), toml::Value::String("test".to_string()));

        assert_eq!(config.get_integer("max_complexity"), Some(10));
        assert_eq!(config.get_bool("enabled"), Some(true));
        assert_eq!(config.get_string("output"), Some("test"));
    }

    #[test]
    fn test_opt_level_ordering() {
        assert!(OptLevel::O0 < OptLevel::O1);
        assert!(OptLevel::O1 < OptLevel::O2);
        assert!(OptLevel::O2 < OptLevel::O3);
    }
}
