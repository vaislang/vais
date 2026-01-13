//! Module resolution for Vais
//!
//! Handles loading and merging modules from external files.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use vais_ast::{Item, Program, UseDef};

use crate::error::ParseError;
use crate::parse;

/// Module resolver
pub struct ModuleResolver {
    /// Base directory for resolving relative imports
    base_dir: PathBuf,
    /// Already loaded modules (to prevent cycles)
    loaded: HashSet<PathBuf>,
    /// Standard library path (if set)
    std_lib_path: Option<PathBuf>,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            loaded: HashSet::new(),
            std_lib_path: None,
        }
    }

    /// Set the standard library path
    pub fn with_std_lib(mut self, path: impl Into<PathBuf>) -> Self {
        self.std_lib_path = Some(path.into());
        self
    }

    /// Resolve all imports in a program and return a merged program
    pub fn resolve(&mut self, program: Program) -> Result<Program, ParseError> {
        let mut merged_items = Vec::new();
        let new_span = program.span;

        // First pass: collect all use statements and load modules
        for item in &program.items {
            if let Item::Use(use_def) = item {
                let imported = self.load_module(use_def)?;
                for imp_item in imported.items {
                    // Only import functions and type definitions
                    match &imp_item {
                        Item::Function(func) => {
                            // Check if function is public or if we're importing specific items
                            if func.is_pub || self.should_import(&func.name, use_def) {
                                merged_items.push(imp_item);
                            }
                        }
                        Item::TypeDef(typedef) => {
                            if typedef.is_pub || self.should_import(&typedef.name, use_def) {
                                merged_items.push(imp_item);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Second pass: add all items from the original program (except Use statements)
        for item in program.items {
            match item {
                Item::Use(_) | Item::Module(_) => {
                    // Skip use and mod statements in the output
                }
                _ => {
                    merged_items.push(item);
                }
            }
        }

        Ok(Program {
            items: merged_items,
            span: new_span,
        })
    }

    /// Check if a specific item should be imported based on use statement
    fn should_import(&self, name: &str, use_def: &UseDef) -> bool {
        // Star import: import all public items
        if use_def.star {
            return true;
        }
        match &use_def.items {
            Some(items) => items.iter().any(|i| i == name),
            None => true, // Import all public items (default behavior)
        }
    }

    /// Load a module from its path
    fn load_module(&mut self, use_def: &UseDef) -> Result<Program, ParseError> {
        let module_path = self.resolve_path(&use_def.path)?;

        // Check for cycles
        let canonical = module_path.canonicalize().map_err(|e| ParseError::ModuleNotFound {
            path: use_def.path.join("."),
            reason: format!("Failed to resolve path: {}", e),
            span: use_def.span,
        })?;

        if self.loaded.contains(&canonical) {
            // Already loaded, return empty program
            return Ok(Program {
                items: Vec::new(),
                span: use_def.span,
            });
        }
        self.loaded.insert(canonical.clone());

        // Read the file
        let source = fs::read_to_string(&module_path).map_err(|e| ParseError::ModuleNotFound {
            path: use_def.path.join("."),
            reason: format!("Failed to read file: {}", e),
            span: use_def.span,
        })?;

        // Parse the module
        let program = parse(&source).map_err(|e| ParseError::ModuleError {
            path: use_def.path.join("."),
            error: Box::new(e),
            span: use_def.span,
        })?;

        // Recursively resolve imports in the loaded module
        let old_base = self.base_dir.clone();
        if let Some(parent) = module_path.parent() {
            self.base_dir = parent.to_path_buf();
        }

        let resolved = self.resolve(program)?;

        self.base_dir = old_base;

        Ok(resolved)
    }

    /// Resolve module path to a file path
    fn resolve_path(&self, path: &[String]) -> Result<PathBuf, ParseError> {
        if path.is_empty() {
            return Err(ParseError::InvalidSyntax {
                message: "Empty module path".to_string(),
                span: vais_lexer::Span::default(),
            });
        }

        // Try standard library first (if path starts with "std")
        if path[0] == "std" {
            if let Some(std_path) = &self.std_lib_path {
                let mut file_path = std_path.clone();
                for component in &path[1..] {
                    file_path.push(component);
                }
                file_path.set_extension("vais");
                if file_path.exists() {
                    return Ok(file_path);
                }
            }
        }

        // Try relative path
        let mut file_path = self.base_dir.clone();
        for component in path {
            file_path.push(component);
        }
        file_path.set_extension("vais");

        if file_path.exists() {
            return Ok(file_path);
        }

        // Try without extension (might be a directory with mod.vais)
        file_path.set_extension("");
        file_path.push("mod.vais");
        if file_path.exists() {
            return Ok(file_path);
        }

        Err(ParseError::ModuleNotFound {
            path: path.join("."),
            reason: format!(
                "Could not find module at '{}' or '{}'",
                self.base_dir.join(path.join("/")).with_extension("vais").display(),
                self.base_dir.join(path.join("/")).join("mod.vais").display()
            ),
            span: vais_lexer::Span::default(),
        })
    }
}

/// Convenience function to resolve modules in a program
pub fn resolve_modules(
    program: Program,
    base_dir: impl Into<PathBuf>,
) -> Result<Program, ParseError> {
    let mut resolver = ModuleResolver::new(base_dir);
    resolver.resolve(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_path_resolution() {
        // Test that path building works correctly
        let resolver = ModuleResolver::new("/base/dir");
        assert!(!resolver.loaded.contains(&PathBuf::from("/base/dir")));
    }

    #[test]
    fn test_should_import() {
        let resolver = ModuleResolver::new("/base");

        // Test with specific items
        let use_def = UseDef {
            path: vec!["math".to_string()],
            items: Some(vec!["add".to_string(), "mul".to_string()]),
            alias: None,
            star: false,
            span: vais_lexer::Span::default(),
        };
        assert!(resolver.should_import("add", &use_def));
        assert!(resolver.should_import("mul", &use_def));
        assert!(!resolver.should_import("div", &use_def));

        // Test without specific items (import all)
        let use_def_all = UseDef {
            path: vec!["math".to_string()],
            items: None,
            alias: None,
            star: false,
            span: vais_lexer::Span::default(),
        };
        assert!(resolver.should_import("anything", &use_def_all));

        // Test star import
        let use_def_star = UseDef {
            path: vec!["math".to_string()],
            items: None,
            alias: None,
            star: true,
            span: vais_lexer::Span::default(),
        };
        assert!(resolver.should_import("any_function", &use_def_star));
    }
}
