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

/// Format configuration for formatter plugins
#[derive(Debug, Clone, Default)]
pub struct FormatConfig {
    /// Indentation size in spaces
    pub indent_size: usize,
    /// Maximum line length
    pub line_length: usize,
    /// Use tabs instead of spaces
    pub use_tabs: bool,
    /// Additional configuration values
    pub values: std::collections::HashMap<String, toml::Value>,
}

impl FormatConfig {
    /// Create a new format config with defaults
    pub fn new() -> Self {
        Self {
            indent_size: 4,
            line_length: 100,
            use_tabs: false,
            values: std::collections::HashMap::new(),
        }
    }

    /// Create from PluginConfig
    pub fn from_plugin_config(config: &PluginConfig) -> Self {
        let mut fmt_config = Self::new();

        if let Some(indent) = config.get_integer("indent_size") {
            fmt_config.indent_size = indent as usize;
        }
        if let Some(line_len) = config.get_integer("line_length") {
            fmt_config.line_length = line_len as usize;
        }
        if let Some(tabs) = config.get_bool("use_tabs") {
            fmt_config.use_tabs = tabs;
        }

        fmt_config.values = config.values.clone();
        fmt_config
    }
}

/// Complexity analysis report
#[derive(Debug, Clone)]
pub struct ComplexityReport {
    /// Overall module complexity score
    pub overall_complexity: usize,
    /// Function-level complexity scores
    pub function_complexity: std::collections::HashMap<String, usize>,
    /// Suggestions for reducing complexity
    pub suggestions: Vec<String>,
}

impl ComplexityReport {
    /// Create a new empty complexity report
    pub fn new() -> Self {
        Self {
            overall_complexity: 0,
            function_complexity: std::collections::HashMap::new(),
            suggestions: Vec::new(),
        }
    }

    /// Add a function complexity score
    pub fn add_function(&mut self, name: String, complexity: usize) {
        self.function_complexity.insert(name, complexity);
        self.overall_complexity += complexity;
    }

    /// Add a suggestion
    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }

    /// Get the highest complexity function
    pub fn max_complexity_function(&self) -> Option<(&String, &usize)> {
        self.function_complexity.iter().max_by_key(|(_, &v)| v)
    }
}

impl Default for ComplexityReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Dependency information for a module
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    /// Module name
    pub name: String,
    /// Whether this is an external dependency
    pub external: bool,
    /// Import location span (if available)
    pub span: Option<Span>,
}

/// Dependency graph for a module
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    /// Direct dependencies
    pub dependencies: Vec<DependencyInfo>,
    /// Modules that depend on this module (reverse dependencies)
    pub dependents: Vec<String>,
    /// Circular dependency chains (if any)
    pub cycles: Vec<Vec<String>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dep: DependencyInfo) {
        self.dependencies.push(dep);
    }

    /// Add a dependent
    pub fn add_dependent(&mut self, name: String) {
        self.dependents.push(name);
    }

    /// Add a circular dependency cycle
    pub fn add_cycle(&mut self, cycle: Vec<String>) {
        self.cycles.push(cycle);
    }

    /// Check if there are circular dependencies
    pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }

    /// Get count of direct dependencies
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    /// Get count of external dependencies
    pub fn external_dependency_count(&self) -> usize {
        self.dependencies.iter().filter(|d| d.external).count()
    }
}

/// Formatter plugin trait
///
/// Formatter plugins format source code according to style rules.
///
/// # Example
///
/// ```ignore
/// impl FormatterPlugin for MyFormatter {
///     fn format_module(&self, module: &Module, config: &FormatConfig) -> Result<String, String> {
///         // Format the module and return formatted source code
///         Ok("fn main() {}\n".to_string())
///     }
/// }
/// ```
pub trait FormatterPlugin: Plugin {
    /// Format a module and return the formatted source code
    fn format_module(&self, module: &Module, config: &FormatConfig) -> Result<String, String>;
}

/// Analysis plugin trait
///
/// Analysis plugins analyze code without modifying it and return reports.
///
/// # Example
///
/// ```ignore
/// impl AnalysisPlugin for MyAnalyzer {
///     fn analyze_complexity(&self, module: &Module) -> ComplexityReport {
///         // Analyze and return complexity report
///         ComplexityReport::new()
///     }
///
///     fn analyze_dependencies(&self, module: &Module) -> DependencyGraph {
///         // Analyze and return dependency graph
///         DependencyGraph::new()
///     }
/// }
/// ```
pub trait AnalysisPlugin: Plugin {
    /// Analyze module complexity
    ///
    /// Returns a report containing complexity metrics for the module,
    /// including per-function complexity scores and suggestions for improvement.
    fn analyze_complexity(&self, module: &Module) -> ComplexityReport;

    /// Analyze module dependencies
    ///
    /// Returns a dependency graph showing all imports, their sources,
    /// and any circular dependency issues.
    fn analyze_dependencies(&self, module: &Module) -> DependencyGraph;
}

/// Plugin type enumeration for registry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum PluginType {
    Lint,
    Transform,
    Optimize,
    Codegen,
    Formatter,
    Analysis,
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
        config
            .values
            .insert("max_complexity".to_string(), toml::Value::Integer(10));
        config
            .values
            .insert("enabled".to_string(), toml::Value::Boolean(true));
        config.values.insert(
            "output".to_string(),
            toml::Value::String("test".to_string()),
        );

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

    #[test]
    fn test_format_config_defaults() {
        let config = FormatConfig::new();
        assert_eq!(config.indent_size, 4);
        assert_eq!(config.line_length, 100);
        assert!(!config.use_tabs);
        assert!(config.values.is_empty());
    }

    #[test]
    fn test_format_config_from_plugin_config() {
        let mut plugin_config = PluginConfig::new();
        plugin_config
            .values
            .insert("indent_size".to_string(), toml::Value::Integer(2));
        plugin_config
            .values
            .insert("line_length".to_string(), toml::Value::Integer(80));
        plugin_config
            .values
            .insert("use_tabs".to_string(), toml::Value::Boolean(true));

        let fmt_config = FormatConfig::from_plugin_config(&plugin_config);
        assert_eq!(fmt_config.indent_size, 2);
        assert_eq!(fmt_config.line_length, 80);
        assert!(fmt_config.use_tabs);
    }

    #[test]
    fn test_complexity_report() {
        let mut report = ComplexityReport::new();
        assert_eq!(report.overall_complexity, 0);
        assert!(report.function_complexity.is_empty());
        assert!(report.suggestions.is_empty());

        report.add_function("main".to_string(), 5);
        report.add_function("helper".to_string(), 3);
        assert_eq!(report.overall_complexity, 8);
        assert_eq!(report.function_complexity.len(), 2);

        report.add_suggestion("Reduce complexity in main()".to_string());
        assert_eq!(report.suggestions.len(), 1);

        let (name, &complexity) = report.max_complexity_function().unwrap();
        assert_eq!(name, "main");
        assert_eq!(complexity, 5);
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();
        assert!(graph.dependencies.is_empty());
        assert!(graph.dependents.is_empty());
        assert!(!graph.has_cycles());

        graph.add_dependency(DependencyInfo {
            name: "std::io".to_string(),
            external: true,
            span: None,
        });
        assert_eq!(graph.dependency_count(), 1);
        assert_eq!(graph.external_dependency_count(), 1);

        graph.add_dependency(DependencyInfo {
            name: "local::module".to_string(),
            external: false,
            span: Some(Span::new(0, 10)),
        });
        assert_eq!(graph.dependency_count(), 2);
        assert_eq!(graph.external_dependency_count(), 1);

        graph.add_dependent("other::module".to_string());
        assert_eq!(graph.dependents.len(), 1);

        graph.add_cycle(vec!["A".to_string(), "B".to_string(), "A".to_string()]);
        assert!(graph.has_cycles());
        assert_eq!(graph.cycles.len(), 1);
    }

    #[test]
    fn test_plugin_type_variants() {
        assert_eq!(PluginType::Lint, PluginType::Lint);
        assert_ne!(PluginType::Lint, PluginType::Transform);
        assert_ne!(PluginType::Formatter, PluginType::Analysis);
    }
}
