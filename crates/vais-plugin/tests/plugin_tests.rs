//! Comprehensive tests for the Vais plugin system
//!
//! Tests cover:
//! - PluginRegistry creation and initialization
//! - PluginsConfig parsing (TOML configuration)
//! - Plugin loading and execution
//! - Error handling (invalid paths, malformed config)

use std::any::Any;
use std::path::PathBuf;
use vais_ast::{Module, Span};
use vais_plugin::{
    AnalysisPlugin, ComplexityReport, DependencyGraph, DependencyInfo, Diagnostic,
    DiagnosticLevel, FormatConfig, FormatterPlugin, LintPlugin, OptLevel, Plugin, PluginConfig,
    PluginInfo, PluginRegistry, PluginsConfig, PluginsSection, TransformPlugin,
};

// ============================================================================
// Mock Plugin Implementations for Testing
// ============================================================================

/// Mock lint plugin for testing
struct MockLintPlugin {
    name: &'static str,
    warnings_to_emit: usize,
}

impl MockLintPlugin {
    fn new(name: &'static str, warnings: usize) -> Self {
        Self {
            name,
            warnings_to_emit: warnings,
        }
    }
}

impl Plugin for MockLintPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name,
            version: "0.1.0",
            description: "Mock lint plugin for testing",
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl LintPlugin for MockLintPlugin {
    fn check(&self, module: &Module) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Emit configured number of warnings
        for i in 0..self.warnings_to_emit {
            diagnostics.push(
                Diagnostic::warning(format!("Mock warning {} from {}", i, self.name))
                    .with_span(Span::new(0, 10))
                    .with_help("This is a test warning"),
            );
        }

        // Check for functions without bodies (as an example lint)
        for item in &module.items {
            if let vais_ast::Item::Function(f) = &item.node {
                if matches!(f.body, vais_ast::FunctionBody::Block(ref stmts) if stmts.is_empty()) {
                    diagnostics.push(
                        Diagnostic::warning(format!("Function '{}' has empty body", f.name.node))
                            .with_span(item.span)
                    );
                }
            }
        }

        diagnostics
    }
}

/// Mock transform plugin for testing
struct MockTransformPlugin {
    name: &'static str,
    should_fail: bool,
}

impl MockTransformPlugin {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            should_fail: false,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

impl Plugin for MockTransformPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name,
            version: "0.1.0",
            description: "Mock transform plugin for testing",
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl TransformPlugin for MockTransformPlugin {
    fn transform(&self, module: Module) -> Result<Module, String> {
        if self.should_fail {
            return Err(format!("Transform failed in {}", self.name));
        }

        // Just return the module unchanged (in real plugin, would transform AST)
        Ok(module)
    }
}

/// Mock formatter plugin for testing
struct MockFormatterPlugin {
    name: &'static str,
}

impl MockFormatterPlugin {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl Plugin for MockFormatterPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name,
            version: "0.1.0",
            description: "Mock formatter plugin for testing",
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl FormatterPlugin for MockFormatterPlugin {
    fn format_module(&self, module: &Module, config: &FormatConfig) -> Result<String, String> {
        // Simple mock formatter that just returns a formatted string
        let mut output = String::new();
        output.push_str(&format!("// Formatted by {} with indent_size={}\n",
            self.name, config.indent_size));
        output.push_str(&format!("// Module with {} items\n", module.items.len()));
        Ok(output)
    }
}

/// Mock analysis plugin for testing
struct MockAnalysisPlugin {
    name: &'static str,
}

impl MockAnalysisPlugin {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl Plugin for MockAnalysisPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name,
            version: "0.1.0",
            description: "Mock analysis plugin for testing",
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl AnalysisPlugin for MockAnalysisPlugin {
    fn analyze_complexity(&self, module: &Module) -> ComplexityReport {
        let mut report = ComplexityReport::new();

        // Analyze functions in the module
        for item in &module.items {
            if let vais_ast::Item::Function(f) = &item.node {
                // Simple complexity: count statements in function body
                let complexity = match &f.body {
                    vais_ast::FunctionBody::Block(stmts) => stmts.len(),
                    vais_ast::FunctionBody::Expr(_) => 1,
                };
                report.add_function(f.name.node.clone(), complexity);

                if complexity > 5 {
                    report.add_suggestion(format!(
                        "Function '{}' has complexity {}, consider refactoring",
                        f.name.node, complexity
                    ));
                }
            }
        }

        report
    }

    fn analyze_dependencies(&self, module: &Module) -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        // Analyze use statements (imports) in the module
        for item in &module.items {
            if let vais_ast::Item::Use(use_stmt) = &item.node {
                // Extract the string values from Spanned<String>
                let path_parts: Vec<String> = use_stmt.path.iter()
                    .map(|s| s.node.clone())
                    .collect();

                let dep = DependencyInfo {
                    name: path_parts.join("::"),
                    external: path_parts.first().map(|s| s.as_str() == "std").unwrap_or(false),
                    span: Some(item.span),
                };
                graph.add_dependency(dep);
            }
        }

        graph
    }
}

// ============================================================================
// PluginRegistry Tests
// ============================================================================

#[test]
fn test_plugin_registry_creation() {
    let registry = PluginRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert_eq!(registry.plugin_names().len(), 0);
}

#[test]
fn test_plugin_registry_default() {
    let registry = PluginRegistry::default();
    assert!(registry.is_empty());
}

#[test]
fn test_empty_registry_operations() {
    let registry = PluginRegistry::new();
    let module = Module { items: vec![] };

    // Running lint on empty registry should return no diagnostics
    let diagnostics = registry.run_lint(&module);
    assert!(diagnostics.is_empty());

    // Running transform on empty registry should return unchanged module
    let transformed = registry.run_transform(module.clone()).unwrap();
    assert_eq!(transformed.items.len(), 0);

    // Running optimize on empty registry should return unchanged IR
    let ir = "; LLVM IR test";
    let optimized = registry.run_optimize(ir, OptLevel::O2).unwrap();
    assert_eq!(optimized, ir);
}

#[test]
fn test_plugin_registry_counts_by_type() {
    let registry = PluginRegistry::new();
    let (lint, transform, optimize, codegen, formatter, analysis) = registry.counts_by_type();

    assert_eq!(lint, 0);
    assert_eq!(transform, 0);
    assert_eq!(optimize, 0);
    assert_eq!(codegen, 0);
    assert_eq!(formatter, 0);
    assert_eq!(analysis, 0);
}

// ============================================================================
// PluginsConfig Tests - TOML Parsing
// ============================================================================

#[test]
fn test_parse_empty_config() {
    let config = PluginsConfig::parse("").unwrap();
    assert!(config.is_empty());
    assert_eq!(config.plugins.path.len(), 0);
    assert_eq!(config.plugins.enabled.len(), 0);
}

#[test]
fn test_parse_config_with_paths() {
    let toml = r#"
[plugins]
path = ["./plugins/lint.dylib", "./plugins/optimizer.so"]
"#;

    let config = PluginsConfig::parse(toml).unwrap();
    assert!(!config.is_empty());
    assert_eq!(config.plugins.path.len(), 2);
    assert_eq!(config.plugins.path[0], PathBuf::from("./plugins/lint.dylib"));
    assert_eq!(config.plugins.path[1], PathBuf::from("./plugins/optimizer.so"));
}

#[test]
fn test_parse_config_with_enabled() {
    let toml = r#"
[plugins]
enabled = ["vais-lint", "vais-format"]
"#;

    let config = PluginsConfig::parse(toml).unwrap();
    assert!(!config.is_empty());
    assert_eq!(config.plugins.enabled.len(), 2);
    assert_eq!(config.plugins.enabled[0], "vais-lint");
    assert_eq!(config.plugins.enabled[1], "vais-format");
}

#[test]
fn test_parse_config_with_plugin_config() {
    let toml = r#"
[plugins]
path = ["./plugins/lint.dylib"]

[plugins.config]
lint = { max_complexity = 15, enabled = true }
formatter = { indent_size = 2, line_length = 80 }
"#;

    let config = PluginsConfig::parse(toml).unwrap();
    assert_eq!(config.plugins.config.len(), 2);

    // Check lint config
    let lint_config = config.plugins.config.get("lint").unwrap();
    assert_eq!(
        lint_config.get("max_complexity").and_then(|v| v.as_integer()),
        Some(15)
    );
    assert_eq!(
        lint_config.get("enabled").and_then(|v| v.as_bool()),
        Some(true)
    );

    // Check formatter config
    let formatter_config = config.plugins.config.get("formatter").unwrap();
    assert_eq!(
        formatter_config.get("indent_size").and_then(|v| v.as_integer()),
        Some(2)
    );
    assert_eq!(
        formatter_config.get("line_length").and_then(|v| v.as_integer()),
        Some(80)
    );
}

#[test]
fn test_parse_config_full_example() {
    let toml = r#"
[plugins]
path = [
    "./plugins/example-lint.dylib",
    "./plugins/example-optimizer.so"
]
enabled = ["vais-lint-complexity", "vais-bindgen"]

[plugins.config]
example-lint = { max_warnings = 10, strict_mode = false }
example-optimizer = { level = "aggressive", inline_threshold = 100 }
vais-bindgen = { output_dir = "./bindings", language = "python" }
"#;

    let config = PluginsConfig::parse(toml).unwrap();

    // Check paths
    assert_eq!(config.plugins.path.len(), 2);

    // Check enabled
    assert_eq!(config.plugins.enabled.len(), 2);
    assert!(config.plugins.enabled.contains(&"vais-lint-complexity".to_string()));
    assert!(config.plugins.enabled.contains(&"vais-bindgen".to_string()));

    // Check plugin-specific configs
    assert_eq!(config.plugins.config.len(), 3);

    let lint_cfg = config.plugins.config.get("example-lint").unwrap();
    assert_eq!(lint_cfg.get("max_warnings").and_then(|v| v.as_integer()), Some(10));
    assert_eq!(lint_cfg.get("strict_mode").and_then(|v| v.as_bool()), Some(false));

    let bindgen_cfg = config.plugins.config.get("vais-bindgen").unwrap();
    assert_eq!(
        bindgen_cfg.get("language").and_then(|v| v.as_str()),
        Some("python")
    );
}

#[test]
fn test_parse_invalid_toml() {
    let invalid_toml = r#"
[plugins
path = "missing bracket"
"#;

    let result = PluginsConfig::parse(invalid_toml);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid config"));
}

#[test]
fn test_parse_malformed_structure() {
    let toml = r#"
[plugins]
path = 123  # Should be array, not integer
"#;

    let result = PluginsConfig::parse(toml);
    assert!(result.is_err());
}

#[test]
fn test_config_empty() {
    let config = PluginsConfig::empty();
    assert!(config.is_empty());
}

#[test]
fn test_config_is_empty() {
    let empty = PluginsConfig::empty();
    assert!(empty.is_empty());

    let with_path = PluginsConfig {
        plugins: PluginsSection {
            path: vec![PathBuf::from("test.dylib")],
            enabled: vec![],
            config: Default::default(),
        },
    };
    assert!(!with_path.is_empty());

    let with_enabled = PluginsConfig {
        plugins: PluginsSection {
            path: vec![],
            enabled: vec!["test-plugin".to_string()],
            config: Default::default(),
        },
    };
    assert!(!with_enabled.is_empty());
}

// ============================================================================
// Diagnostic Tests
// ============================================================================

#[test]
fn test_diagnostic_creation() {
    let warning = Diagnostic::warning("test warning");
    assert_eq!(warning.level, DiagnosticLevel::Warning);
    assert_eq!(warning.message, "test warning");
    assert!(warning.span.is_none());
    assert!(warning.help.is_none());

    let error = Diagnostic::error("test error");
    assert_eq!(error.level, DiagnosticLevel::Error);
    assert_eq!(error.message, "test error");

    let info = Diagnostic::info("test info");
    assert_eq!(info.level, DiagnosticLevel::Info);
    assert_eq!(info.message, "test info");
}

#[test]
fn test_diagnostic_builder() {
    let diagnostic = Diagnostic::warning("warning message")
        .with_span(Span::new(10, 20))
        .with_help("try this instead");

    assert_eq!(diagnostic.level, DiagnosticLevel::Warning);
    assert_eq!(diagnostic.message, "warning message");
    assert_eq!(diagnostic.span, Some(Span::new(10, 20)));
    assert_eq!(diagnostic.help, Some("try this instead".to_string()));
}

#[test]
fn test_has_errors_detection() {
    let no_errors = vec![
        Diagnostic::warning("warn1"),
        Diagnostic::info("info1"),
    ];
    assert!(!PluginRegistry::has_errors(&no_errors));

    let with_errors = vec![
        Diagnostic::warning("warn1"),
        Diagnostic::error("error1"),
        Diagnostic::info("info1"),
    ];
    assert!(PluginRegistry::has_errors(&with_errors));

    let only_errors = vec![
        Diagnostic::error("error1"),
        Diagnostic::error("error2"),
    ];
    assert!(PluginRegistry::has_errors(&only_errors));
}

// ============================================================================
// PluginConfig Tests
// ============================================================================

#[test]
fn test_plugin_config_creation() {
    let config = PluginConfig::new();
    assert!(config.values.is_empty());
}

#[test]
fn test_plugin_config_getters() {
    let mut config = PluginConfig::new();

    config.values.insert("max_complexity".to_string(), toml::Value::Integer(10));
    config.values.insert("enabled".to_string(), toml::Value::Boolean(true));
    config.values.insert("output_format".to_string(), toml::Value::String("json".to_string()));

    assert_eq!(config.get_integer("max_complexity"), Some(10));
    assert_eq!(config.get_bool("enabled"), Some(true));
    assert_eq!(config.get_string("output_format"), Some("json"));

    // Test missing keys
    assert_eq!(config.get_integer("nonexistent"), None);
    assert_eq!(config.get_bool("nonexistent"), None);
    assert_eq!(config.get_string("nonexistent"), None);

    // Test type mismatches
    assert_eq!(config.get_integer("enabled"), None); // bool, not int
    assert_eq!(config.get_bool("max_complexity"), None); // int, not bool
}

// ============================================================================
// OptLevel Tests
// ============================================================================

#[test]
fn test_opt_level_ordering() {
    assert!(OptLevel::O0 < OptLevel::O1);
    assert!(OptLevel::O1 < OptLevel::O2);
    assert!(OptLevel::O2 < OptLevel::O3);

    assert!(OptLevel::O0 <= OptLevel::O0);
    assert!(OptLevel::O3 >= OptLevel::O2);
}

#[test]
fn test_opt_level_equality() {
    assert_eq!(OptLevel::O0, OptLevel::O0);
    assert_eq!(OptLevel::O3, OptLevel::O3);
    assert_ne!(OptLevel::O1, OptLevel::O2);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_load_nonexistent_plugin() {
    let mut registry = PluginRegistry::new();
    let result = registry.load_from_path(PathBuf::from("/nonexistent/plugin.dylib").as_path());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to load plugin"));
}

#[test]
fn test_load_from_config_with_invalid_path() {
    let mut registry = PluginRegistry::new();

    let config = PluginsConfig {
        plugins: PluginsSection {
            path: vec![PathBuf::from("/invalid/path/plugin.dylib")],
            enabled: vec![],
            config: Default::default(),
        },
    };

    let result = registry.load_from_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_configure_nonexistent_plugin() {
    let mut registry = PluginRegistry::new();

    let config = toml::Value::Table(Default::default());
    let result = registry.configure("nonexistent-plugin", &config);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_plugin_info() {
    let plugin = MockLintPlugin::new("test-lint", 0);
    let info = plugin.info();

    assert_eq!(info.name, "test-lint");
    assert_eq!(info.version, "0.1.0");
    assert!(!info.description.is_empty());
}

#[test]
fn test_plugin_as_any() {
    let mut plugin = MockLintPlugin::new("test-lint", 0);

    // Test immutable downcast
    let any_ref = plugin.as_any();
    assert!(any_ref.downcast_ref::<MockLintPlugin>().is_some());

    // Test mutable downcast
    let any_mut = plugin.as_any_mut();
    assert!(any_mut.downcast_mut::<MockLintPlugin>().is_some());
}

#[test]
fn test_plugin_default_init() {
    let mut plugin = MockLintPlugin::new("test-lint", 0);
    let config = PluginConfig::new();

    // Default init should succeed
    let result = plugin.init(&config);
    assert!(result.is_ok());
}

// ============================================================================
// FormatterPlugin Tests
// ============================================================================

#[test]
fn test_formatter_plugin_basic() {
    let formatter = MockFormatterPlugin::new("test-formatter");
    let module = Module { items: vec![] };
    let config = FormatConfig::new();

    let result = formatter.format_module(&module, &config);
    assert!(result.is_ok());

    let formatted = result.unwrap();
    assert!(formatted.contains("test-formatter"));
    assert!(formatted.contains("indent_size=4"));
    assert!(formatted.contains("0 items"));
}

#[test]
fn test_formatter_plugin_with_custom_config() {
    let formatter = MockFormatterPlugin::new("test-formatter");
    let module = Module { items: vec![] };
    let mut config = FormatConfig::new();
    config.indent_size = 2;
    config.line_length = 80;
    config.use_tabs = true;

    let result = formatter.format_module(&module, &config);
    assert!(result.is_ok());

    let formatted = result.unwrap();
    assert!(formatted.contains("indent_size=2"));
}

#[test]
fn test_format_config_from_plugin_config() {
    let mut plugin_config = PluginConfig::new();
    plugin_config.values.insert(
        "indent_size".to_string(),
        toml::Value::Integer(2),
    );
    plugin_config.values.insert(
        "line_length".to_string(),
        toml::Value::Integer(120),
    );
    plugin_config.values.insert(
        "use_tabs".to_string(),
        toml::Value::Boolean(true),
    );

    let format_config = FormatConfig::from_plugin_config(&plugin_config);
    assert_eq!(format_config.indent_size, 2);
    assert_eq!(format_config.line_length, 120);
    assert!(format_config.use_tabs);
}

// ============================================================================
// AnalysisPlugin Tests
// ============================================================================

#[test]
fn test_analysis_plugin_complexity_empty_module() {
    let analyzer = MockAnalysisPlugin::new("test-analyzer");
    let module = Module { items: vec![] };

    let report = analyzer.analyze_complexity(&module);
    assert_eq!(report.overall_complexity, 0);
    assert!(report.function_complexity.is_empty());
    assert!(report.suggestions.is_empty());
}

#[test]
fn test_analysis_plugin_dependencies_empty_module() {
    let analyzer = MockAnalysisPlugin::new("test-analyzer");
    let module = Module { items: vec![] };

    let graph = analyzer.analyze_dependencies(&module);
    assert_eq!(graph.dependency_count(), 0);
    assert_eq!(graph.external_dependency_count(), 0);
    assert!(!graph.has_cycles());
}

#[test]
fn test_complexity_report_builder() {
    let mut report = ComplexityReport::new();
    assert_eq!(report.overall_complexity, 0);

    report.add_function("main".to_string(), 10);
    report.add_function("helper".to_string(), 5);
    assert_eq!(report.overall_complexity, 15);
    assert_eq!(report.function_complexity.len(), 2);

    report.add_suggestion("Reduce complexity in main()".to_string());
    assert_eq!(report.suggestions.len(), 1);

    let (name, &complexity) = report.max_complexity_function().unwrap();
    assert_eq!(name, "main");
    assert_eq!(complexity, 10);
}

#[test]
fn test_dependency_graph_builder() {
    let mut graph = DependencyGraph::new();

    graph.add_dependency(DependencyInfo {
        name: "std::io".to_string(),
        external: true,
        span: None,
    });
    graph.add_dependency(DependencyInfo {
        name: "local::util".to_string(),
        external: false,
        span: Some(Span::new(0, 20)),
    });

    assert_eq!(graph.dependency_count(), 2);
    assert_eq!(graph.external_dependency_count(), 1);

    graph.add_dependent("other::module".to_string());
    assert_eq!(graph.dependents.len(), 1);

    graph.add_cycle(vec!["A".to_string(), "B".to_string(), "A".to_string()]);
    assert!(graph.has_cycles());
}

#[test]
fn test_plugin_info_formatter() {
    let plugin = MockFormatterPlugin::new("test-formatter");
    let info = plugin.info();

    assert_eq!(info.name, "test-formatter");
    assert_eq!(info.version, "0.1.0");
    assert!(info.description.contains("formatter"));
}

#[test]
fn test_plugin_info_analysis() {
    let plugin = MockAnalysisPlugin::new("test-analyzer");
    let info = plugin.info();

    assert_eq!(info.name, "test-analyzer");
    assert_eq!(info.version, "0.1.0");
    assert!(info.description.contains("analysis"));
}
