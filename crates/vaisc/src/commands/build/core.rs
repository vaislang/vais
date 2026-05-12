//! Core build command implementation.
//!
//! Orchestrates the compilation pipeline: parse, macro expand, type check,
//! codegen, optimize, and link. Delegates to `parallel` and `serial` submodules
//! for the actual codegen paths.

use crate::configure_type_checker;
use crate::error_formatter;
use crate::imports::{load_module_with_imports_internal, load_module_with_imports_parallel};
use crate::incremental;
use crate::utils::{print_plugin_diagnostics, print_suggested_fixes};

/// Phase 17.H1: FNV-1a 32 hash of a source path for use as TC file_id.
/// Non-zero — 0 is reserved for synthetic spans.
fn phase17_fnv1a_file_id(s: &str) -> u32 {
    const FNV_OFFSET: u32 = 0x811c_9dc5;
    const FNV_PRIME: u32 = 0x0100_0193;
    let mut hash = FNV_OFFSET;
    for byte in s.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    if hash == 0 {
        1
    } else {
        hash
    }
}
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use vais_ast::{
    Expr, FunctionBody, GenericParamKind, IfElse, Item, MatchArm, Module, Pattern, Span, Spanned,
    Stmt, Type, VariantFields,
};
use vais_codegen::TargetTriple;
use vais_macro::{collect_macros, expand_macros, process_derives, MacroRegistry};
use vais_plugin::{DiagnosticLevel, PluginRegistry};
use vais_query::QueryDatabase;
use vais_types::TypeChecker;

/// Per-phase compilation timing profile.
/// Used by `--profile` flag to display detailed pipeline breakdown.
#[derive(Debug, Clone, Default)]
pub struct CompileProfile {
    pub parse_ms: f64,
    pub macro_ms: f64,
    pub typecheck_ms: f64,
    pub codegen_ms: f64,
    pub optimize_ms: f64,
    pub clang_ms: f64,
    pub total_ms: f64,
}

impl CompileProfile {
    /// Print a detailed profile table with percentages.
    pub fn print(&self) {
        let total = self.total_ms.max(0.001); // avoid division by zero
        println!("\n{}", "=== Compilation Profile ===".cyan().bold());
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Parse + Import",
            self.parse_ms,
            self.parse_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Macro Expand",
            self.macro_ms,
            self.macro_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Type Check",
            self.typecheck_ms,
            self.typecheck_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Codegen",
            self.codegen_ms,
            self.codegen_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "IR Optimize",
            self.optimize_ms,
            self.optimize_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Clang Link",
            self.clang_ms,
            self.clang_ms / total * 100.0
        );
        println!("  {}", "─".repeat(40));
        println!("  {:<16} {:>8.1}ms", "Total", self.total_ms);
    }
}

/// Wrapper around cmd_build that optionally prints timing information
#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_build_with_timing(
    input: &PathBuf,
    output: Option<PathBuf>,
    emit_ir: bool,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    time: bool,
    profile: bool,
    plugins: &PluginRegistry,
    target: TargetTriple,
    force_rebuild: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    hot: bool,
    lto_mode: vais_codegen::optimize::LtoMode,
    pgo_mode: vais_codegen::optimize::PgoMode,
    coverage_mode: vais_codegen::optimize::CoverageMode,
    suggest_fixes: bool,
    parallel_config: Option<vais_codegen::parallel::ParallelConfig>,
    use_inkwell: bool,
    per_module: bool,
    cache_limit: u64,
) -> Result<(), String> {
    use std::time::Instant;

    let start = Instant::now();
    let mut compile_profile = CompileProfile::default();
    let result = cmd_build(
        input,
        output,
        emit_ir,
        opt_level,
        debug,
        verbose,
        plugins,
        target,
        force_rebuild,
        gc,
        gc_threshold,
        hot,
        lto_mode,
        pgo_mode,
        coverage_mode,
        suggest_fixes,
        parallel_config,
        use_inkwell,
        per_module,
        cache_limit,
        if profile {
            Some(&mut compile_profile)
        } else {
            None
        },
    );
    let elapsed = start.elapsed();

    if time {
        println!(
            "\n{} Total compilation time: {:.3}s",
            "⏱".cyan().bold(),
            elapsed.as_secs_f64()
        );
    }

    if profile {
        compile_profile.total_ms = elapsed.as_secs_f64() * 1000.0;
        compile_profile.print();
    }

    result
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::result_large_err)]
pub(crate) fn cmd_build(
    input: &PathBuf,
    output: Option<PathBuf>,
    emit_ir: bool,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    plugins: &PluginRegistry,
    target: TargetTriple,
    force_rebuild: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    hot: bool,
    lto_mode: vais_codegen::optimize::LtoMode,
    pgo_mode: vais_codegen::optimize::PgoMode,
    coverage_mode: vais_codegen::optimize::CoverageMode,
    suggest_fixes: bool,
    parallel_config: Option<vais_codegen::parallel::ParallelConfig>,
    use_inkwell: bool,
    per_module: bool,
    cache_limit: u64,
    mut profile_out: Option<&mut CompileProfile>,
) -> Result<(), String> {
    use incremental::{get_cache_dir, CompilationOptions, IncrementalCache};

    // Initialize incremental compilation cache.
    // The cache is always initialized (for metadata tracking) but skip-on-hit
    // behaviour is controlled by the `--force-rebuild` flag (default: incremental enabled).
    let cache_dir = get_cache_dir(input);
    let mut cache = IncrementalCache::new(cache_dir).ok();

    if verbose {
        if let Some(ref c) = cache {
            println!(
                "{} incremental cache at {}",
                "Incremental:".cyan().bold(),
                c.cache_dir().display()
            );
        }
    }

    // Set compilation options for cache validity checking
    if let Some(ref mut c) = cache {
        c.set_compilation_options(CompilationOptions {
            opt_level,
            debug,
            target_triple: target.triple_str().to_string(),
        });
    }

    // Check if we can skip compilation (only when not forcing rebuild)
    if !force_rebuild {
        if let Some(ref mut c) = cache {
            if let Some(result) = check_cache_skip(c, input, &output, emit_ir, verbose, &target)? {
                return result;
            }
        }
    } else if verbose {
        println!("{} (--force-rebuild)", "Full rebuild".yellow().bold());
    }

    // Initialize parallel compilation if requested
    let use_parallel = parallel_config.is_some();
    if let Some(ref config) = parallel_config {
        config.init_thread_pool()?;
        if verbose {
            println!(
                "{} Parallel compilation enabled ({} threads)",
                "⚡".cyan().bold(),
                config.effective_threads()
            );
        }
    }

    // Read source for error reporting
    let main_source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    // Step 19 P1 (2026-05-07): emit single-char keyword deprecation
    // warnings. The lex is a pre-pass that does not affect parser output;
    // the parser will lex independently. Cost is small (Phase 129 baseline:
    // 50K LOC ≈ 3.4ms lex), and suppressed via VAIS_SUPPRESS_SINGLE_CHAR_WARN=1.
    if let Ok((_tokens, deprecation_warnings)) = vais_lexer::tokenize_with_warnings(&main_source) {
        crate::utils::emit_deprecation_warnings(
            &deprecation_warnings,
            &input.display().to_string(),
        );
    }
    // Note: if lex fails here, the parser will report the same error
    // shortly with a richer span. Don't double-report.

    // Initialize query database for memoized parsing
    let mut query_db = QueryDatabase::new();

    // Set cfg values from target triple for conditional compilation
    let mut cfg = target.cfg_values();

    // Inject feature flags into cfg values (set by `vaisc pkg build --features`)
    if let Ok(features_str) = std::env::var("VAIS_FEATURES") {
        for feat in features_str.split(',') {
            let feat = feat.trim();
            if !feat.is_empty() {
                cfg.insert(format!("feature:{}", feat), feat.to_string());
            }
        }
    }

    query_db.set_cfg_values(cfg);

    // Parse main file and resolve imports
    let parse_start = std::time::Instant::now();
    let mut loaded_modules: HashSet<PathBuf> = HashSet::new();
    let mut loading_stack: Vec<PathBuf> = Vec::new();
    let merged_ast = if use_parallel {
        load_module_with_imports_parallel(
            input,
            &mut loaded_modules,
            verbose,
            &main_source,
            &query_db,
            input.parent().map(|p| p as &Path),
        )?
    } else {
        load_module_with_imports_internal(
            input,
            &mut loaded_modules,
            &mut loading_stack,
            verbose,
            &main_source,
            &query_db,
            input.parent().map(|p| p as &Path),
        )?
    };
    let parse_time = parse_start.elapsed();
    if let Some(ref mut p) = profile_out {
        p.parse_ms = parse_time.as_secs_f64() * 1000.0;
    }

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
            print_plugin_diagnostics(&diagnostics, &main_source, input);

            // Check if any errors (not just warnings)
            let has_errors = diagnostics
                .iter()
                .any(|d| d.level == DiagnosticLevel::Error);
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

    // Macro expansion phase
    let macro_start = std::time::Instant::now();
    let mut macro_registry = MacroRegistry::new();

    // Register builtin panic! macro: panic!("msg") => __panic("msg")
    {
        use vais_ast::{
            Delimiter, MacroDef, MacroPattern, MacroPatternElement, MacroRule, MacroTemplate,
            MacroTemplateElement, MacroToken, MetaVarKind, Span, Spanned,
        };
        macro_registry.register(MacroDef {
            name: Spanned::new("panic".to_string(), Span::new(0, 5)),
            rules: vec![MacroRule {
                pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                    name: "msg".to_string(),
                    kind: MetaVarKind::Expr,
                }]),
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::Token(MacroToken::Ident("__panic".to_string())),
                    MacroTemplateElement::Group {
                        delimiter: Delimiter::Paren,
                        content: vec![MacroTemplateElement::MetaVar("msg".to_string())],
                    },
                ]),
            }],
            is_pub: false,
        });
    }

    collect_macros(&transformed_ast, &mut macro_registry);

    let macro_expanded_ast = expand_macros(transformed_ast, &macro_registry)
        .map_err(|e| format!("Macro expansion error: {}", e))?;

    let mut final_ast = macro_expanded_ast;
    process_derives(&mut final_ast).map_err(|e| format!("Derive macro error: {}", e))?;
    stamp_module_file_ids_for_typecheck(&mut final_ast);
    let macro_time = macro_start.elapsed();
    if let Some(ref mut p) = profile_out {
        p.macro_ms = macro_time.as_secs_f64() * 1000.0;
    }

    if verbose {
        let macro_count = macro_registry.macros_count();
        if macro_count > 0 {
            println!("  {} {} macro(s) expanded", "Macros:".cyan(), macro_count);
        }
    }

    // Type check
    let typecheck_start = std::time::Instant::now();
    let checker = run_type_check(
        &final_ast,
        &query_db,
        input,
        &main_source,
        suggest_fixes,
        force_rebuild,
        use_parallel,
        verbose,
        &mut cache,
    )?;
    let typecheck_time = typecheck_start.elapsed();
    if let Some(ref mut p) = profile_out {
        p.typecheck_ms = typecheck_time.as_secs_f64() * 1000.0;
    }

    // Print ownership warnings if any
    let ownership_warnings: Vec<_> = checker
        .get_warnings()
        .iter()
        .filter(|w| w.starts_with("[ownership]"))
        .collect();
    if !ownership_warnings.is_empty() {
        for w in &ownership_warnings {
            eprintln!("{} {}", "warning:".yellow().bold(), w);
        }
    }

    if verbose {
        println!("  {}", "Type check passed".green());
        println!(
            "  {} Type check time: {:.3}s",
            "⏱".cyan(),
            typecheck_time.as_secs_f64()
        );
    }

    // MIR borrow checking (when --strict-borrow is enabled)
    if crate::get_strict_borrow() {
        run_mir_borrow_check(&final_ast, verbose)?;
    }

    // Per-module codegen path
    // VAIS_SINGLE_MODULE=1 forces single-module codegen (deprecated — per-module now supports generics)
    let force_single = std::env::var("VAIS_SINGLE_MODULE").is_ok_and(|v| v == "1");
    if force_single && verbose {
        eprintln!("warning: VAIS_SINGLE_MODULE=1 is deprecated — per-module codegen now supports cross-module generics");
    }
    let use_per_module = !force_single
        && (per_module || final_ast.modules_map.as_ref().is_some_and(|m| m.len() > 1));
    if use_per_module {
        if let Some(ref mmap) = final_ast.modules_map {
            if mmap.len() > 1 {
                let input_canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());

                if emit_ir {
                    return super::parallel::run_per_module_emit_ir(
                        &final_ast,
                        &checker,
                        &target,
                        input,
                        &input_canonical,
                        &output,
                        debug,
                        opt_level,
                        verbose,
                        gc,
                        gc_threshold,
                        &main_source,
                        mmap,
                        &loaded_modules,
                        &mut cache,
                    );
                }

                return super::parallel::run_per_module_compile(
                    &final_ast,
                    &checker,
                    &target,
                    input,
                    &input_canonical,
                    output,
                    opt_level,
                    debug,
                    verbose,
                    gc,
                    gc_threshold,
                    &main_source,
                    &loaded_modules,
                    &mut cache,
                );
            }
        }
    }

    // Single-module codegen path
    let raw_ir = super::serial::generate_ir_single_module(
        &final_ast,
        &checker,
        &target,
        input,
        &main_source,
        debug,
        verbose,
        gc,
        gc_threshold,
        use_inkwell,
        use_parallel,
        &mut profile_out,
    )?;

    super::serial::optimize_and_output(
        &raw_ir,
        &final_ast,
        input,
        output,
        emit_ir,
        opt_level,
        debug,
        verbose,
        &target,
        hot,
        &lto_mode,
        &pgo_mode,
        &coverage_mode,
        plugins,
        use_parallel,
        &loaded_modules,
        &mut cache,
        cache_limit,
        &mut profile_out,
    )
}

// ============================================================================
// Helper functions extracted from cmd_build for readability
// ============================================================================

fn stamp_module_file_ids_for_typecheck(module: &mut Module) {
    let Some(modules_map) = module.modules_map.as_ref() else {
        return;
    };

    let mut item_file_ids = vec![None; module.items.len()];
    let mut module_entries: Vec<_> = modules_map.iter().collect();
    module_entries.sort_by(|a, b| a.0.cmp(b.0));

    for (module_path, item_indices) in module_entries {
        let file_id = phase17_fnv1a_file_id(&module_path.to_string_lossy());
        for &idx in item_indices {
            if let Some(slot) = item_file_ids.get_mut(idx) {
                slot.get_or_insert(file_id);
            }
        }
    }

    for (item, file_id) in module.items.iter_mut().zip(item_file_ids) {
        if let Some(file_id) = file_id {
            stamp_spanned_item(item, file_id);
        }
    }
}

fn stamp_span(span: &mut Span, file_id: u32) {
    span.file_id = file_id;
}

fn stamp_spanned<T>(spanned: &mut Spanned<T>, file_id: u32) {
    stamp_span(&mut spanned.span, file_id);
}

fn stamp_attributes(attributes: &mut [vais_ast::Attribute], file_id: u32) {
    for attribute in attributes {
        if let Some(expr) = attribute.expr.as_mut() {
            stamp_expr(expr, file_id);
        }
    }
}

fn stamp_generic_params(generics: &mut [vais_ast::GenericParam], file_id: u32) {
    for generic in generics {
        stamp_spanned(&mut generic.name, file_id);
        for bound in &mut generic.bounds {
            stamp_spanned(bound, file_id);
        }
        match &mut generic.kind {
            GenericParamKind::Type { bounds } => {
                for bound in bounds {
                    stamp_spanned(bound, file_id);
                }
            }
            GenericParamKind::Const { ty } => stamp_type(ty, file_id),
            GenericParamKind::Lifetime { .. } => {}
        }
    }
}

fn stamp_where_clause(where_clause: &mut [vais_ast::WherePredicate], file_id: u32) {
    for predicate in where_clause {
        stamp_spanned(&mut predicate.ty, file_id);
        for bound in &mut predicate.bounds {
            stamp_spanned(bound, file_id);
        }
    }
}

fn stamp_spanned_item(item: &mut Spanned<Item>, file_id: u32) {
    stamp_spanned(item, file_id);
    match &mut item.node {
        Item::Function(function) => stamp_function(function, file_id),
        Item::Struct(struct_def) => {
            stamp_spanned(&mut struct_def.name, file_id);
            stamp_generic_params(&mut struct_def.generics, file_id);
            for field in &mut struct_def.fields {
                stamp_field(field, file_id);
            }
            for method in &mut struct_def.methods {
                stamp_spanned(method, file_id);
                stamp_function(&mut method.node, file_id);
            }
            stamp_attributes(&mut struct_def.attributes, file_id);
            stamp_where_clause(&mut struct_def.where_clause, file_id);
        }
        Item::Enum(enum_def) => {
            stamp_spanned(&mut enum_def.name, file_id);
            stamp_generic_params(&mut enum_def.generics, file_id);
            for variant in &mut enum_def.variants {
                stamp_spanned(&mut variant.name, file_id);
                stamp_variant_fields(&mut variant.fields, file_id);
            }
            stamp_attributes(&mut enum_def.attributes, file_id);
        }
        Item::Union(union_def) => {
            stamp_spanned(&mut union_def.name, file_id);
            stamp_generic_params(&mut union_def.generics, file_id);
            for field in &mut union_def.fields {
                stamp_field(field, file_id);
            }
        }
        Item::TypeAlias(type_alias) => {
            stamp_spanned(&mut type_alias.name, file_id);
            stamp_generic_params(&mut type_alias.generics, file_id);
            stamp_type(&mut type_alias.ty, file_id);
        }
        Item::TraitAlias(trait_alias) => {
            stamp_spanned(&mut trait_alias.name, file_id);
            stamp_generic_params(&mut trait_alias.generics, file_id);
            for bound in &mut trait_alias.bounds {
                stamp_spanned(bound, file_id);
            }
        }
        Item::Use(use_stmt) => stamp_use(use_stmt, file_id),
        Item::Trait(trait_def) => {
            stamp_spanned(&mut trait_def.name, file_id);
            stamp_generic_params(&mut trait_def.generics, file_id);
            for super_trait in &mut trait_def.super_traits {
                stamp_spanned(super_trait, file_id);
            }
            for associated_type in &mut trait_def.associated_types {
                stamp_associated_type(associated_type, file_id);
            }
            for method in &mut trait_def.methods {
                stamp_trait_method(method, file_id);
            }
            stamp_where_clause(&mut trait_def.where_clause, file_id);
        }
        Item::Impl(impl_block) => {
            stamp_type(&mut impl_block.target_type, file_id);
            if let Some(trait_name) = &mut impl_block.trait_name {
                stamp_spanned(trait_name, file_id);
            }
            stamp_generic_params(&mut impl_block.generics, file_id);
            for associated_type in &mut impl_block.associated_types {
                stamp_spanned(&mut associated_type.name, file_id);
                stamp_type(&mut associated_type.ty, file_id);
            }
            for method in &mut impl_block.methods {
                stamp_spanned(method, file_id);
                stamp_function(&mut method.node, file_id);
            }
        }
        Item::Macro(macro_def) => {
            stamp_spanned(&mut macro_def.name, file_id);
        }
        Item::ExternBlock(extern_block) => {
            for function in &mut extern_block.functions {
                stamp_spanned(&mut function.name, file_id);
                stamp_params(&mut function.params, file_id);
                if let Some(ret_type) = &mut function.ret_type {
                    stamp_type(ret_type, file_id);
                }
                stamp_attributes(&mut function.attributes, file_id);
            }
        }
        Item::Const(const_def) => {
            stamp_spanned(&mut const_def.name, file_id);
            stamp_type(&mut const_def.ty, file_id);
            stamp_expr(&mut const_def.value, file_id);
            stamp_attributes(&mut const_def.attributes, file_id);
        }
        Item::Global(global_def) => {
            stamp_spanned(&mut global_def.name, file_id);
            stamp_type(&mut global_def.ty, file_id);
            stamp_expr(&mut global_def.value, file_id);
        }
        Item::Error { .. } => {}
    }
}

fn stamp_use(use_stmt: &mut vais_ast::Use, file_id: u32) {
    for segment in &mut use_stmt.path {
        stamp_spanned(segment, file_id);
    }
    if let Some(alias) = &mut use_stmt.alias {
        stamp_spanned(alias, file_id);
    }
    if let Some(items) = &mut use_stmt.items {
        for item in items {
            stamp_spanned(item, file_id);
        }
    }
}

fn stamp_function(function: &mut vais_ast::Function, file_id: u32) {
    stamp_spanned(&mut function.name, file_id);
    stamp_generic_params(&mut function.generics, file_id);
    stamp_params(&mut function.params, file_id);
    if let Some(ret_type) = &mut function.ret_type {
        stamp_type(ret_type, file_id);
    }
    stamp_function_body(&mut function.body, file_id);
    stamp_attributes(&mut function.attributes, file_id);
    stamp_where_clause(&mut function.where_clause, file_id);
}

fn stamp_params(params: &mut [vais_ast::Param], file_id: u32) {
    for param in params {
        stamp_spanned(&mut param.name, file_id);
        stamp_type(&mut param.ty, file_id);
        if let Some(default_value) = param.default_value.as_mut() {
            stamp_expr(default_value, file_id);
        }
    }
}

fn stamp_function_body(body: &mut FunctionBody, file_id: u32) {
    match body {
        FunctionBody::Expr(expr) => stamp_expr(expr, file_id),
        FunctionBody::Block(stmts) => stamp_stmts(stmts, file_id),
    }
}

fn stamp_field(field: &mut vais_ast::Field, file_id: u32) {
    stamp_spanned(&mut field.name, file_id);
    stamp_type(&mut field.ty, file_id);
}

fn stamp_variant_fields(fields: &mut VariantFields, file_id: u32) {
    match fields {
        VariantFields::Unit => {}
        VariantFields::Tuple(types) => {
            for ty in types {
                stamp_type(ty, file_id);
            }
        }
        VariantFields::Struct(fields) => {
            for field in fields {
                stamp_field(field, file_id);
            }
        }
    }
}

fn stamp_associated_type(associated_type: &mut vais_ast::AssociatedType, file_id: u32) {
    stamp_spanned(&mut associated_type.name, file_id);
    stamp_generic_params(&mut associated_type.generics, file_id);
    for bound in &mut associated_type.bounds {
        stamp_spanned(bound, file_id);
    }
    if let Some(default_ty) = &mut associated_type.default {
        stamp_type(default_ty, file_id);
    }
}

fn stamp_trait_method(method: &mut vais_ast::TraitMethod, file_id: u32) {
    stamp_spanned(&mut method.name, file_id);
    stamp_generic_params(&mut method.generics, file_id);
    stamp_params(&mut method.params, file_id);
    if let Some(ret_type) = &mut method.ret_type {
        stamp_type(ret_type, file_id);
    }
    if let Some(default_body) = &mut method.default_body {
        stamp_function_body(default_body, file_id);
    }
}

fn stamp_type(ty: &mut Spanned<Type>, file_id: u32) {
    stamp_spanned(ty, file_id);
    match &mut ty.node {
        Type::Named { generics, .. } => {
            for generic in generics {
                stamp_type(generic, file_id);
            }
        }
        Type::FnPtr { params, ret, .. } | Type::Fn { params, ret } => {
            for param in params {
                stamp_type(param, file_id);
            }
            stamp_type(ret, file_id);
        }
        Type::Array(inner)
        | Type::Optional(inner)
        | Type::Result(inner)
        | Type::Pointer(inner)
        | Type::Ref(inner)
        | Type::RefMut(inner)
        | Type::Slice(inner)
        | Type::SliceMut(inner)
        | Type::RefLifetime { inner, .. }
        | Type::RefMutLifetime { inner, .. }
        | Type::Linear(inner)
        | Type::Affine(inner) => stamp_type(inner, file_id),
        Type::ConstArray { element, .. } => stamp_type(element, file_id),
        Type::Map(key, value) => {
            stamp_type(key, file_id);
            stamp_type(value, file_id);
        }
        Type::Tuple(types) => {
            for ty in types {
                stamp_type(ty, file_id);
            }
        }
        Type::DynTrait { generics, .. } => {
            for generic in generics {
                stamp_type(generic, file_id);
            }
        }
        Type::Associated { base, generics, .. } => {
            stamp_type(base, file_id);
            for generic in generics {
                stamp_type(generic, file_id);
            }
        }
        Type::Dependent {
            base, predicate, ..
        } => {
            stamp_type(base, file_id);
            stamp_expr(predicate, file_id);
        }
        Type::Unit | Type::Infer => {}
    }
}

fn stamp_stmts(stmts: &mut [Spanned<Stmt>], file_id: u32) {
    for stmt in stmts {
        stamp_stmt(stmt, file_id);
    }
}

fn stamp_stmt(stmt: &mut Spanned<Stmt>, file_id: u32) {
    stamp_spanned(stmt, file_id);
    match &mut stmt.node {
        Stmt::Let {
            name, ty, value, ..
        } => {
            stamp_spanned(name, file_id);
            if let Some(ty) = ty {
                stamp_type(ty, file_id);
            }
            stamp_expr(value, file_id);
        }
        Stmt::LetDestructure { pattern, value, .. } => {
            stamp_pattern(pattern, file_id);
            stamp_expr(value, file_id);
        }
        Stmt::Expr(expr) | Stmt::Defer(expr) => stamp_expr(expr, file_id),
        Stmt::Return(expr) | Stmt::Break(expr) => {
            if let Some(expr) = expr {
                stamp_expr(expr, file_id);
            }
        }
        Stmt::Continue | Stmt::Error { .. } => {}
    }
}

fn stamp_expr(expr: &mut Spanned<Expr>, file_id: u32) {
    stamp_spanned(expr, file_id);
    match &mut expr.node {
        Expr::StringInterp(parts) => {
            for part in parts {
                if let vais_ast::StringInterpPart::Expr(expr) = part {
                    stamp_expr(expr, file_id);
                }
            }
        }
        Expr::Binary { left, right, .. } => {
            stamp_expr(left, file_id);
            stamp_expr(right, file_id);
        }
        Expr::Unary { expr, .. }
        | Expr::Await(expr)
        | Expr::Try(expr)
        | Expr::Unwrap(expr)
        | Expr::Spread(expr)
        | Expr::Ref(expr)
        | Expr::Deref(expr)
        | Expr::Yield(expr)
        | Expr::Comptime { body: expr }
        | Expr::Old(expr)
        | Expr::Assume(expr) => stamp_expr(expr, file_id),
        Expr::Ternary { cond, then, else_ } => {
            stamp_expr(cond, file_id);
            stamp_expr(then, file_id);
            stamp_expr(else_, file_id);
        }
        Expr::If { cond, then, else_ } => {
            stamp_expr(cond, file_id);
            stamp_stmts(then, file_id);
            if let Some(else_) = else_ {
                stamp_if_else(else_, file_id);
            }
        }
        Expr::Loop {
            pattern,
            iter,
            body,
        } => {
            if let Some(pattern) = pattern {
                stamp_pattern(pattern, file_id);
            }
            if let Some(iter) = iter {
                stamp_expr(iter, file_id);
            }
            stamp_stmts(body, file_id);
        }
        Expr::While { condition, body } => {
            stamp_expr(condition, file_id);
            stamp_stmts(body, file_id);
        }
        Expr::Match { expr, arms } => {
            stamp_expr(expr, file_id);
            for arm in arms {
                stamp_match_arm(arm, file_id);
            }
        }
        Expr::Call { func, args } => {
            stamp_expr(func, file_id);
            for arg in args {
                stamp_expr(arg, file_id);
            }
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            stamp_expr(receiver, file_id);
            stamp_spanned(method, file_id);
            for arg in args {
                stamp_expr(arg, file_id);
            }
        }
        Expr::StaticMethodCall {
            type_name,
            method,
            args,
        } => {
            stamp_spanned(type_name, file_id);
            stamp_spanned(method, file_id);
            for arg in args {
                stamp_expr(arg, file_id);
            }
        }
        Expr::Field { expr, field } => {
            stamp_expr(expr, file_id);
            stamp_spanned(field, file_id);
        }
        Expr::TupleFieldAccess { expr, .. } => stamp_expr(expr, file_id),
        Expr::Index { expr, index } => {
            stamp_expr(expr, file_id);
            stamp_expr(index, file_id);
        }
        Expr::Array(items) | Expr::Tuple(items) => {
            for item in items {
                stamp_expr(item, file_id);
            }
        }
        Expr::StructLit { name, fields, .. } => {
            stamp_spanned(name, file_id);
            for (field, value) in fields {
                stamp_spanned(field, file_id);
                stamp_expr(value, file_id);
            }
        }
        Expr::Range { start, end, .. } => {
            if let Some(start) = start {
                stamp_expr(start, file_id);
            }
            if let Some(end) = end {
                stamp_expr(end, file_id);
            }
        }
        Expr::Block(stmts) => stamp_stmts(stmts, file_id),
        Expr::MapLit(pairs) => {
            for (key, value) in pairs {
                stamp_expr(key, file_id);
                stamp_expr(value, file_id);
            }
        }
        Expr::Cast { expr, ty } => {
            stamp_expr(expr, file_id);
            stamp_type(ty, file_id);
        }
        Expr::Assign { target, value } | Expr::AssignOp { target, value, .. } => {
            stamp_expr(target, file_id);
            stamp_expr(value, file_id);
        }
        Expr::Lambda { params, body, .. } => {
            stamp_params(params, file_id);
            stamp_expr(body, file_id);
        }
        Expr::MacroInvoke(invoke) => {
            stamp_spanned(&mut invoke.name, file_id);
        }
        Expr::Assert { condition, message } => {
            stamp_expr(condition, file_id);
            if let Some(message) = message {
                stamp_expr(message, file_id);
            }
        }
        Expr::EnumAccess { data, .. } => {
            if let Some(data) = data {
                stamp_expr(data, file_id);
            }
        }
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Unit
        | Expr::Ident(_)
        | Expr::SelfCall
        | Expr::Error { .. } => {}
    }
}

fn stamp_if_else(else_: &mut IfElse, file_id: u32) {
    match else_ {
        IfElse::ElseIf(cond, stmts, next) => {
            stamp_expr(cond, file_id);
            stamp_stmts(stmts, file_id);
            if let Some(next) = next {
                stamp_if_else(next, file_id);
            }
        }
        IfElse::Else(stmts) => stamp_stmts(stmts, file_id),
    }
}

fn stamp_match_arm(arm: &mut MatchArm, file_id: u32) {
    stamp_pattern(&mut arm.pattern, file_id);
    if let Some(guard) = &mut arm.guard {
        stamp_expr(guard, file_id);
    }
    stamp_expr(&mut arm.body, file_id);
}

fn stamp_pattern(pattern: &mut Spanned<Pattern>, file_id: u32) {
    stamp_spanned(pattern, file_id);
    match &mut pattern.node {
        Pattern::Tuple(patterns)
        | Pattern::Variant {
            fields: patterns, ..
        }
        | Pattern::Or(patterns) => {
            for pattern in patterns {
                stamp_pattern(pattern, file_id);
            }
        }
        Pattern::Struct { name, fields, .. } => {
            stamp_spanned(name, file_id);
            for (field, nested) in fields {
                stamp_spanned(field, file_id);
                if let Some(nested) = nested {
                    stamp_pattern(nested, file_id);
                }
            }
        }
        Pattern::Range { start, end, .. } => {
            if let Some(start) = start {
                stamp_pattern(start, file_id);
            }
            if let Some(end) = end {
                stamp_pattern(end, file_id);
            }
        }
        Pattern::Alias { pattern, .. } => stamp_pattern(pattern, file_id),
        Pattern::Wildcard | Pattern::Ident(_) | Pattern::Literal(_) => {}
    }
}

/// Check if we can skip compilation due to cache hit.
/// Returns Some(Ok(())) if skip is valid, None if compilation is needed.
fn check_cache_skip(
    c: &mut incremental::IncrementalCache,
    input: &Path,
    output: &Option<PathBuf>,
    emit_ir: bool,
    verbose: bool,
    target: &TargetTriple,
) -> Result<Option<Result<(), String>>, String> {
    match c.detect_changes_with_stats(input) {
        Ok((dirty_set, incr_stats)) => {
            if dirty_set.is_empty() {
                if verbose {
                    println!(
                        "{} {} (no changes detected)",
                        "Skipping".cyan().bold(),
                        input.display()
                    );
                    println!(
                        "  Cache hit rate: {:.1}% ({}/{} files)",
                        incr_stats.hit_rate(),
                        incr_stats.cache_hits,
                        incr_stats.files_checked
                    );
                    if incr_stats.signature_hits > 0 {
                        println!(
                            "  Signature hits: {} (dependents skipped)",
                            incr_stats.signature_hits
                        );
                    }
                    println!("  Cache check: {}ms", incr_stats.total_check_time_ms);
                }
                // Still need to output the binary path if not emit_ir
                if !emit_ir {
                    let default_ext = match target {
                        TargetTriple::Wasm32Unknown
                        | TargetTriple::WasiPreview1
                        | TargetTriple::WasiPreview2 => "wasm",
                        _ => "",
                    };
                    let bin_path = output
                        .clone()
                        .unwrap_or_else(|| input.with_extension(default_ext));
                    if bin_path.exists() {
                        if !verbose {
                            println!("{}", bin_path.display());
                        }
                        return Ok(Some(Ok(())));
                    }
                } else {
                    let ir_path = output.clone().unwrap_or_else(|| input.with_extension("ll"));
                    if ir_path.exists() {
                        if !verbose {
                            println!("{}", ir_path.display());
                        }
                        return Ok(Some(Ok(())));
                    }
                }
            } else if verbose {
                println!(
                    "{} {} file(s) changed",
                    "Rebuilding".yellow().bold(),
                    dirty_set.count()
                );
                println!(
                    "  Cache: {}/{} hits ({:.1}%), {} misses, check {}ms",
                    incr_stats.cache_hits,
                    incr_stats.files_checked,
                    incr_stats.hit_rate(),
                    incr_stats.cache_misses,
                    incr_stats.total_check_time_ms
                );
                // Show miss reasons for verbose debugging
                for (file, reasons) in &incr_stats.miss_reasons {
                    let reason_strs: Vec<&str> = reasons
                        .iter()
                        .map(|r| match r {
                            incremental::CacheMissReason::NewFile => "new",
                            incremental::CacheMissReason::ContentHashChanged => "content changed",
                            incremental::CacheMissReason::SignatureChanged => "signature changed",
                            incremental::CacheMissReason::DependencyChanged(_) => {
                                "dependency changed"
                            }
                            incremental::CacheMissReason::OptionsChanged => "options changed",
                            incremental::CacheMissReason::FileDeleted => "deleted",
                            incremental::CacheMissReason::CacheCorrupted => "cache corrupted",
                            incremental::CacheMissReason::StdChanged => "std changed",
                        })
                        .collect();
                    let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    println!("    {} — {}", file_name, reason_strs.join(", "));
                }
            }
        }
        Err(e) => {
            if verbose {
                println!("{} Cache check failed: {}", "Warning".yellow(), e);
            }
        }
    }
    Ok(None)
}

/// Run type checking (parallel or sequential).
#[allow(clippy::too_many_arguments)]
fn run_type_check(
    final_ast: &vais_ast::Module,
    query_db: &QueryDatabase,
    input: &Path,
    main_source: &str,
    suggest_fixes: bool,
    force_rebuild: bool,
    use_parallel: bool,
    verbose: bool,
    cache: &mut Option<incremental::IncrementalCache>,
) -> Result<TypeChecker, String> {
    let mut tc_skipped = false;

    // Check if we can skip type checking based on cached signatures
    if !force_rebuild {
        if let Some(ref c) = cache {
            let tc_files: Vec<PathBuf> = final_ast
                .modules_map
                .as_ref()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_else(|| vec![input.to_path_buf()]);
            if incremental::can_skip_type_checking(c, &tc_files) {
                tc_skipped = true;
                if verbose {
                    println!(
                        "  {} Type check skipped (signatures unchanged)",
                        "⚡".cyan()
                    );
                }
            }
        }
    }

    let mut checker = TypeChecker::new();
    configure_type_checker(&mut checker);

    if !tc_skipped {
        // Calculate imported item count so ownership checker can skip imported items
        let input_canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
        // Phase 17.H1: stamp a non-zero file_id so single-module builds
        // also avoid the 0-vs-0 collision hazard.
        checker.set_current_file_id(phase17_fnv1a_file_id(&input_canonical.to_string_lossy()));
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

        checker.multi_error_mode = true;

        const MIN_MODULES_FOR_PARALLEL_TC: usize = 4;

        let tc_result = if use_parallel
            && final_ast
                .modules_map
                .as_ref()
                .is_some_and(|m| m.len() >= MIN_MODULES_FOR_PARALLEL_TC)
        {
            let (final_checker, all_errors) =
                super::parallel::run_parallel_type_check(final_ast, verbose);
            checker = final_checker;

            if !all_errors.is_empty() {
                // Phase 158/#4: VAIS_TC_NONFATAL was removed — TC errors are always
                // fatal. Parallel TC errors are surfaced as a single InferFailed.
                Err(vais_types::TypeError::InferFailed {
                    kind: "module".to_string(),
                    name: "parallel_tc".to_string(),
                    context: all_errors.join("\n---\n"),
                    span: None,
                    suggestion: None,
                })
            } else {
                Ok(())
            }
        } else {
            checker.check_module(final_ast)
        };

        // Handle type checking result (Phase 158/#4: TC errors are always fatal).
        if let Err(e) = tc_result {
            if suggest_fixes {
                print_suggested_fixes(&e, main_source);
            }
            let total_errors = 1 + checker.get_collected_errors().len();
            for collected_err in checker.get_collected_errors() {
                eprintln!(
                    "{}",
                    error_formatter::format_type_error(collected_err, main_source, input)
                );
            }
            if let Some(ref mut c) = cache {
                incremental::update_tc_cache(c, final_ast, false);
            }
            if total_errors > 1 {
                eprintln!("{}: {} errors found", "error".red().bold(), total_errors);
            }
            return Err(error_formatter::format_type_error(&e, main_source, input));
        }

        // Even if check_module succeeded, there may be collected errors.
        // Phase 158/#4: these are fatal — no VAIS_TC_NONFATAL escape hatch.
        if !checker.get_collected_errors().is_empty() {
            let total_errors = checker.get_collected_errors().len();
            for collected_err in checker.get_collected_errors() {
                eprintln!(
                    "{}",
                    error_formatter::format_type_error(collected_err, main_source, input)
                );
            }
            if let Some(ref mut c) = cache {
                incremental::update_tc_cache(c, final_ast, false);
            }
            return Err(format!("{} type error(s) found", total_errors));
        }

        if std::env::var("VAIS_CORE_CERTIFY").is_ok_and(|v| v == "1") {
            let input_canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
            let input_file_id = phase17_fnv1a_file_id(&input_canonical.to_string_lossy());
            checker
                .assert_fully_resolved_for_codegen_source(input_file_id, main_source.len())
                .map_err(|e| format!("Core codegen type invariant failed: {}", e))?;
        }

        // Update cache: TC passed
        if let Some(ref mut c) = cache {
            incremental::update_tc_cache(c, final_ast, true);
        }
    }

    Ok(checker)
}

/// Run MIR borrow checking if --strict-borrow is enabled.
fn run_mir_borrow_check(final_ast: &vais_ast::Module, verbose: bool) -> Result<(), String> {
    let borrow_start = std::time::Instant::now();
    if verbose {
        println!("  {} Running MIR borrow checker...", "🔍".cyan());
    }

    let mir_module = vais_mir::lower::lower_module(final_ast);
    let borrow_errors = vais_mir::borrow_check::check_module(&mir_module);

    if !borrow_errors.is_empty() {
        for error in &borrow_errors {
            eprintln!("{}: {}", "error".red().bold(), error);
        }
        return Err(format!(
            "Borrow checking failed: {} error(s) detected",
            borrow_errors.len()
        ));
    }

    if verbose {
        let borrow_time = borrow_start.elapsed();
        println!("  {}", "Borrow check passed".green());
        println!(
            "  {} Borrow check time: {:.3}s",
            "⏱".cyan(),
            borrow_time.as_secs_f64()
        );
    }
    Ok(())
}
