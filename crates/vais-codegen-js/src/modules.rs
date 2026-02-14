//! Module system code generation: Vais Use → JavaScript ESM import/export

use crate::{JsCodeGenerator, Result};
use std::collections::HashMap;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use vais_ast::*;

impl JsCodeGenerator {
    /// Generate JavaScript import statement from Vais Use item
    pub(crate) fn generate_use(&self, use_item: &Use) -> Result<String> {
        if use_item.path.is_empty() {
            return Ok(String::new());
        }

        // Convert path segments to module path
        let module_path = self.use_path_to_module(&use_item.path);
        let module_name = use_item
            .path
            .last()
            .map(|s| s.node.as_str())
            .unwrap_or("module");

        // Check if there's an alias
        if let Some(alias) = &use_item.alias {
            // U module::path as alias → import * as alias from './path.js';
            Ok(format!("import * as {} from {module_path};\n", alias.node))
        } else {
            // U module::path → import * as path from './path.js';
            Ok(format!("import * as {module_name} from {module_path};\n"))
        }
    }

    /// Convert Vais path segments to JavaScript module path
    /// `path::to::module` → `'./to/module.js'`
    fn use_path_to_module(&self, path: &[Spanned<String>]) -> String {
        if path.is_empty() {
            return "'./module.js'".to_string();
        }

        // Join path segments with '/' (skip first if it's the package name)
        let segments: Vec<&str> = path.iter().map(|s| s.node.as_str()).collect();

        // For single segment or simple paths, use ./ prefix
        let module_path = if segments.len() == 1 {
            format!("./{}.js", segments[0])
        } else {
            // Skip first segment (package name), join rest with /
            format!("./{}.js", segments[1..].join("/"))
        };

        format!("'{module_path}'")
    }

    /// Generate barrel export file (index.js) listing all public exports
    /// Returns content for index.js that re-exports all public items
    pub fn generate_barrel_export(&self, modules: &[String]) -> String {
        let mut output = String::new();
        output.push_str("// Auto-generated barrel export\n\n");

        for module in modules {
            let module_file = if module.ends_with(".js") {
                module.clone()
            } else {
                format!("{module}.js")
            };
            output.push_str(&format!("export * from './{module_file}';\n"));
        }

        output
    }

    /// Generate separate .js files for each module in modules_map
    /// Returns HashMap<filename, js_content>
    pub fn generate_module_to_files(&mut self, module: &Module) -> Result<HashMap<String, String>> {
        let mut files = HashMap::new();

        if let Some(ref modules_map) = module.modules_map {
            // Generate per-module files
            for (path, item_indices) in modules_map {
                let mut module_gen = JsCodeGenerator::with_config(self.config.clone());
                let mut output = String::new();

                // Register all types first (for forward references)
                for &idx in item_indices {
                    if let Some(item) = module.items.get(idx) {
                        module_gen.register_item(&item.node)?;
                    }
                }

                // Generate code for items in this module
                for &idx in item_indices {
                    if let Some(item) = module.items.get(idx) {
                        let js = module_gen.generate_item(&item.node)?;
                        if !js.is_empty() {
                            output.push_str(&js);
                            output.push('\n');
                        }
                    }
                }

                // Append helper functions if any
                if !module_gen.helpers.is_empty() {
                    output.push('\n');
                    for helper in &module_gen.helpers {
                        output.push_str(helper);
                        output.push('\n');
                    }
                }

                // Convert PathBuf to filename
                let filename = path_to_js_filename(path);
                files.insert(filename, output);
            }
        } else {
            // Single file mode - generate everything in one file
            let output = self.generate_module(module)?;
            files.insert("index.js".to_string(), output);
        }

        Ok(files)
    }
}

/// Convert PathBuf to JavaScript filename
/// `/path/to/module.vais` → `module.js`
fn path_to_js_filename(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("{s}.js"))
        .unwrap_or_else(|| "module.js".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_use(path_parts: &[&str], alias: Option<&str>) -> Use {
        Use {
            path: path_parts
                .iter()
                .map(|s| Spanned::new(s.to_string(), Span::new(0, s.len())))
                .collect(),
            alias: alias.map(|s| Spanned::new(s.to_string(), Span::new(0, s.len()))),
            items: None,
        }
    }

    #[test]
    fn test_single_module_import() {
        let gen = JsCodeGenerator::new();
        let use_item = make_use(&["module"], None);
        let result = gen.generate_use(&use_item).unwrap();
        assert_eq!(result, "import * as module from './module.js';\n");
    }

    #[test]
    fn test_nested_module_import() {
        let gen = JsCodeGenerator::new();
        let use_item = make_use(&["path", "to", "module"], None);
        let result = gen.generate_use(&use_item).unwrap();
        assert_eq!(result, "import * as module from './to/module.js';\n");
    }

    #[test]
    fn test_module_import_with_alias() {
        let gen = JsCodeGenerator::new();
        let use_item = make_use(&["std", "collections", "HashMap"], Some("Map"));
        let result = gen.generate_use(&use_item).unwrap();
        assert_eq!(result, "import * as Map from './collections/HashMap.js';\n");
    }

    #[test]
    fn test_empty_use() {
        let gen = JsCodeGenerator::new();
        let use_item = Use {
            path: vec![],
            alias: None,
            items: None,
        };
        let result = gen.generate_use(&use_item).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_barrel_export_generation() {
        let gen = JsCodeGenerator::new();
        let modules = vec![
            "module1".to_string(),
            "module2".to_string(),
            "utils".to_string(),
        ];
        let result = gen.generate_barrel_export(&modules);
        assert!(result.contains("export * from './module1.js';"));
        assert!(result.contains("export * from './module2.js';"));
        assert!(result.contains("export * from './utils.js';"));
        assert!(result.contains("// Auto-generated barrel export"));
    }

    #[test]
    fn test_path_to_js_filename() {
        assert_eq!(
            path_to_js_filename(Path::new("/path/to/module.vais")),
            "module.js"
        );
        assert_eq!(path_to_js_filename(Path::new("module.vais")), "module.js");
        assert_eq!(path_to_js_filename(Path::new("module")), "module.js");
    }

    #[test]
    fn test_generate_module_to_files_single_mode() {
        let mut gen = JsCodeGenerator::new();
        let module = Module {
            items: vec![Spanned::new(
                Item::Function(Function {
                    name: Spanned::new("test".to_string(), Span::new(0, 4)),
                    generics: vec![],
                    params: vec![],
                    ret_type: None,
                    body: FunctionBody::Expr(Box::new(Spanned::new(
                        Expr::Int(42),
                        Span::new(0, 2),
                    ))),
                    is_pub: true,
                    is_async: false,
                    attributes: vec![],
            where_clause: vec![],
                }),
                Span::new(0, 10),
            )],
            modules_map: None,
        };

        let result = gen.generate_module_to_files(&module).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("index.js"));
        let content = result.get("index.js").unwrap();
        assert!(content.contains("export function test()"));
    }

    #[test]
    fn test_generate_module_to_files_multi_mode() {
        let mut gen = JsCodeGenerator::new();
        let mut modules_map = HashMap::new();
        modules_map.insert(PathBuf::from("mod1.vais"), vec![0]);
        modules_map.insert(PathBuf::from("mod2.vais"), vec![1]);

        let module = Module {
            items: vec![
                Spanned::new(
                    Item::Function(Function {
                        name: Spanned::new("func1".to_string(), Span::new(0, 5)),
                        generics: vec![],
                        params: vec![],
                        ret_type: None,
                        body: FunctionBody::Expr(Box::new(Spanned::new(
                            Expr::Int(1),
                            Span::new(0, 1),
                        ))),
                        is_pub: true,
                        is_async: false,
                        attributes: vec![],
            where_clause: vec![],
                    }),
                    Span::new(0, 10),
                ),
                Spanned::new(
                    Item::Function(Function {
                        name: Spanned::new("func2".to_string(), Span::new(0, 5)),
                        generics: vec![],
                        params: vec![],
                        ret_type: None,
                        body: FunctionBody::Expr(Box::new(Spanned::new(
                            Expr::Int(2),
                            Span::new(0, 1),
                        ))),
                        is_pub: false,
                        is_async: false,
                        attributes: vec![],
            where_clause: vec![],
                    }),
                    Span::new(0, 10),
                ),
            ],
            modules_map: Some(modules_map),
        };

        let result = gen.generate_module_to_files(&module).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("mod1.js"));
        assert!(result.contains_key("mod2.js"));

        let mod1 = result.get("mod1.js").unwrap();
        assert!(mod1.contains("export function func1()"));

        let mod2 = result.get("mod2.js").unwrap();
        assert!(mod2.contains("function func2()"));
        assert!(!mod2.contains("export function func2()"));
    }
}
