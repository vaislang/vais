//! JavaScript target compilation.
//!
//! When `--target js` is specified, this module handles the entire compilation
//! pipeline: parse → type check → macro expand → JS codegen → write .js output.

use crate::configure_type_checker;
use crate::error_formatter;
use crate::imports::load_module_with_imports_internal;
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use vais_ast::Item;
use vais_codegen_js::{JsCodeGenerator, JsConfig};
use vais_macro::{collect_macros, expand_macros, process_derives, MacroRegistry};
use vais_plugin::PluginRegistry;
use vais_query::QueryDatabase;

/// Configuration for JS target compilation
pub(crate) struct JsBuildConfig {
    /// Enable tree shaking (dead code elimination)
    pub tree_shake: bool,
    /// Generate source maps
    pub source_map: bool,
    /// Enable minification (compact output)
    pub minify: bool,
}

impl Default for JsBuildConfig {
    fn default() -> Self {
        Self {
            tree_shake: true,
            source_map: false,
            minify: false,
        }
    }
}

/// Build a .vais file to JavaScript output.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_build_js(
    input: &PathBuf,
    output: Option<PathBuf>,
    verbose: bool,
    plugins: &PluginRegistry,
    js_config: &JsBuildConfig,
) -> Result<(), String> {
    let start = std::time::Instant::now();

    // Read source
    let main_source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    // Initialize query database
    let mut query_db = QueryDatabase::new();
    query_db.set_cfg_values(std::collections::HashMap::new());

    // Parse and resolve imports
    let parse_start = std::time::Instant::now();
    let mut loaded_modules: HashSet<PathBuf> = HashSet::new();
    let mut loading_stack: Vec<PathBuf> = Vec::new();
    let merged_ast = load_module_with_imports_internal(
        input,
        &mut loaded_modules,
        &mut loading_stack,
        verbose,
        &main_source,
        &query_db,
    )?;
    let parse_time = parse_start.elapsed();

    if verbose {
        println!(
            "  {} total items (including imports)",
            merged_ast.items.len()
        );
        println!(
            "  {} Parse time: {:.3}s",
            "⏱".cyan(),
            parse_time.as_secs_f64()
        );
    }

    // Run lint plugins
    if !plugins.is_empty() {
        let diagnostics = plugins.run_lint(&merged_ast);
        if !diagnostics.is_empty() {
            crate::utils::print_plugin_diagnostics(&diagnostics, &main_source, input);
            let has_errors = diagnostics
                .iter()
                .any(|d| d.level == vais_plugin::DiagnosticLevel::Error);
            if has_errors {
                return Err("Plugin lint check failed".to_string());
            }
        }
    }

    // Run transform plugins
    let transformed_ast = if !plugins.is_empty() {
        plugins
            .run_transform(merged_ast)
            .map_err(|e| format!("Plugin transform error: {}", e))?
    } else {
        merged_ast
    };

    // Macro expansion
    let mut macro_registry = MacroRegistry::new();
    collect_macros(&transformed_ast, &mut macro_registry);
    let macro_expanded_ast = expand_macros(transformed_ast, &macro_registry)
        .map_err(|e| format!("Macro expansion error: {}", e))?;
    let mut final_ast = macro_expanded_ast;
    process_derives(&mut final_ast).map_err(|e| format!("Derive macro error: {}", e))?;

    if verbose {
        let macro_count = macro_registry.macros_count();
        if macro_count > 0 {
            println!("  {} {} macro(s) expanded", "Macros:".cyan(), macro_count);
        }
    }

    // Type check
    let typecheck_start = std::time::Instant::now();
    let mut checker = vais_types::TypeChecker::new();
    configure_type_checker(&mut checker);

    // Calculate imported item count
    let input_canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
    if let Ok(original_ast) = query_db.parse(&input_canonical) {
        let original_non_use_count = original_ast
            .items
            .iter()
            .filter(|item| !matches!(item.node, Item::Use(_)))
            .count();
        let imported_count = final_ast.items.len().saturating_sub(original_non_use_count);
        if imported_count > 0 {
            checker.set_imported_item_count(imported_count);
        }
    }

    if let Err(e) = checker.check_module(&final_ast) {
        return Err(error_formatter::format_type_error(&e, &main_source, input));
    }
    let typecheck_time = typecheck_start.elapsed();

    if verbose {
        println!("  {}", "Type check passed".green());
        println!(
            "  {} Type check time: {:.3}s",
            "⏱".cyan(),
            typecheck_time.as_secs_f64()
        );
    }

    // Tree shaking (before codegen)
    let codegen_ast = if js_config.tree_shake {
        let shaken = vais_codegen_js::tree_shaking::TreeShaker::shake(&final_ast);
        let removed = final_ast.items.len() - shaken.items.len();
        if verbose && removed > 0 {
            println!(
                "  {} Removed {} unreachable item(s)",
                "Tree shaking:".cyan(),
                removed
            );
        }
        shaken
    } else {
        final_ast.clone()
    };

    // JS code generation
    let codegen_start = std::time::Instant::now();
    let config = JsConfig {
        use_const_let: true,
        emit_jsdoc: true,
        indent: if js_config.minify {
            String::new()
        } else {
            "  ".to_string()
        },
        target: "es2020".to_string(),
    };
    let mut generator = JsCodeGenerator::with_config(config);
    let js_output = generator
        .generate_module(&codegen_ast)
        .map_err(|e| format!("JS codegen error: {}", e))?;
    let codegen_time = codegen_start.elapsed();

    if verbose {
        println!(
            "  {} JS codegen time: {:.3}s",
            "⏱".cyan(),
            codegen_time.as_secs_f64()
        );
    }

    // Determine output path
    let out_path = output.unwrap_or_else(|| input.with_extension("js"));

    // Generate source map if requested
    if js_config.source_map {
        let source_file = input
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("input.vais");
        let mut source_map = vais_codegen_js::SourceMap::new(
            source_file,
            out_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("output.js"),
        );
        // Add basic mapping (line 1 col 0 → source line 1 col 0)
        source_map.add_mapping(0, 0, 0, 0);

        let map_path = out_path.with_extension("js.map");
        let map_json = source_map.to_json();
        fs::write(&map_path, &map_json)
            .map_err(|e| format!("Cannot write source map '{}': {}", map_path.display(), e))?;

        // Append source map reference to JS output
        let map_filename = map_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("output.js.map");
        let js_with_sourcemap = format!(
            "{}//# sourceMappingURL={}\n",
            js_output, map_filename
        );
        fs::write(&out_path, js_with_sourcemap)
            .map_err(|e| format!("Cannot write '{}': {}", out_path.display(), e))?;

        if verbose {
            println!(
                "{} {} (source map: {})",
                "Wrote".green().bold(),
                out_path.display(),
                map_path.display()
            );
        }
    } else {
        fs::write(&out_path, &js_output)
            .map_err(|e| format!("Cannot write '{}': {}", out_path.display(), e))?;
    }

    let total_time = start.elapsed();

    // Print output info
    let file_size = fs::metadata(&out_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let size_str = if file_size > 1024 * 1024 {
        format!("{:.1} MB", file_size as f64 / (1024.0 * 1024.0))
    } else if file_size > 1024 {
        format!("{:.1} KB", file_size as f64 / 1024.0)
    } else {
        format!("{} bytes", file_size)
    };

    if verbose {
        println!(
            "\n{} {} ({}, {:.3}s)",
            "✓".green().bold(),
            out_path.display(),
            size_str,
            total_time.as_secs_f64()
        );
    } else {
        println!("{}", out_path.display());
    }

    Ok(())
}

/// Check if a target string refers to the JavaScript target.
pub(crate) fn is_js_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "js" | "javascript" | "esm" | "es2020"
    )
}
