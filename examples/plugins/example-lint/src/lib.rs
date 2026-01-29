//! Example Lint Plugin for Vais
//!
//! This plugin demonstrates how to create a lint plugin that checks
//! for specific patterns in Vais code.

use std::any::Any;
use vais_ast::{Item, Module};
use vais_plugin::{
    Diagnostic, LintPlugin, Plugin, PluginConfig, PluginInfo, PluginType,
};

/// Example lint plugin that checks for function naming conventions
pub struct NamingConventionLint {
    /// Maximum allowed function name length
    max_name_length: usize,
    /// Warn on single-character parameter names
    warn_short_params: bool,
}

impl NamingConventionLint {
    pub fn new() -> Self {
        Self {
            max_name_length: 50,
            warn_short_params: true,
        }
    }
}

impl Default for NamingConventionLint {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for NamingConventionLint {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "naming-convention",
            version: "0.1.0",
            description: "Checks function and parameter naming conventions",
        }
    }

    fn init(&mut self, config: &PluginConfig) -> Result<(), String> {
        if let Some(max_len) = config.get_integer("max_name_length") {
            self.max_name_length = max_len as usize;
        }
        if let Some(warn) = config.get_bool("warn_short_params") {
            self.warn_short_params = warn;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl LintPlugin for NamingConventionLint {
    fn check(&self, module: &Module) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for item in &module.items {
            if let Item::Function(func) = &item.node {
                let name = &func.name.node;

                // Check function name length
                if name.len() > self.max_name_length {
                    diagnostics.push(
                        Diagnostic::warning(format!(
                            "Function '{}' has a very long name ({} chars, max {})",
                            name,
                            name.len(),
                            self.max_name_length
                        ))
                        .with_span(func.name.span)
                        .with_help("Consider using a shorter, more descriptive name"),
                    );
                }

                // Check for snake_case naming convention
                if !is_snake_case(name) && !name.starts_with("__") {
                    diagnostics.push(
                        Diagnostic::warning(format!(
                            "Function '{}' should use snake_case naming",
                            name
                        ))
                        .with_span(func.name.span)
                        .with_help("Rename to use lowercase with underscores"),
                    );
                }

                // Check parameter names
                if self.warn_short_params {
                    for param in &func.params {
                        let param_name = &param.name.node;
                        if param_name.len() == 1 && !is_common_short_param(param_name) {
                            diagnostics.push(
                                Diagnostic::info(format!(
                                    "Parameter '{}' in function '{}' has a very short name",
                                    param_name, name
                                ))
                                .with_span(param.name.span)
                                .with_help("Consider using a more descriptive name"),
                            );
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// Check if a name follows snake_case convention
fn is_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }

    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            return false;
        }
        if c == '_' {
            // Double underscore is allowed, but not trailing/leading single underscore
            if let Some(&next) = chars.peek() {
                if next.is_uppercase() {
                    return false;
                }
            }
        }
    }
    true
}

/// Common single-letter parameter names that are acceptable
fn is_common_short_param(name: &str) -> bool {
    matches!(name, "n" | "i" | "j" | "k" | "x" | "y" | "z" | "a" | "b" | "c" | "s" | "t")
}

// Plugin export functions

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(NamingConventionLint::new()))
}

#[no_mangle]
pub extern "C" fn get_plugin_type() -> PluginType {
    PluginType::Lint
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_lint_plugin() -> *mut dyn LintPlugin {
    Box::into_raw(Box::new(NamingConventionLint::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_check() {
        assert!(is_snake_case("hello_world"));
        assert!(is_snake_case("test"));
        assert!(is_snake_case("my_function_name"));
        assert!(!is_snake_case("helloWorld"));
        assert!(!is_snake_case("HelloWorld"));
        assert!(!is_snake_case("myFunctionName"));
    }

    #[test]
    fn test_common_short_params() {
        assert!(is_common_short_param("n"));
        assert!(is_common_short_param("i"));
        assert!(is_common_short_param("x"));
        assert!(!is_common_short_param("q"));
        assert!(!is_common_short_param("ab"));
    }
}
