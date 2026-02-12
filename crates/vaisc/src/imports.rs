//! Module import resolution and loading.

use crate::error_formatter;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use vais_ast::{Item, Module, Spanned};
use vais_query::QueryDatabase;

pub(crate) fn filter_imported_items(
    items: Vec<Spanned<Item>>,
    selected: Option<&[Spanned<String>]>,
) -> Vec<Spanned<Item>> {
    let filtered = items
        .into_iter()
        .filter(|item| !matches!(&item.node, Item::Function(f) if f.name.node == "main"));

    match selected {
        None => filtered.collect(),
        Some(names) => {
            let name_set: std::collections::HashSet<&str> =
                names.iter().map(|s| s.node.as_str()).collect();
            filtered
                .filter(|item| {
                    let item_name = match &item.node {
                        Item::Function(f) => Some(f.name.node.as_str()),
                        Item::Struct(s) => Some(s.name.node.as_str()),
                        Item::Enum(e) => Some(e.name.node.as_str()),
                        Item::Union(u) => Some(u.name.node.as_str()),
                        Item::TypeAlias(t) => Some(t.name.node.as_str()),
                        Item::Trait(t) => Some(t.name.node.as_str()),
                        Item::Impl(_) => None, // Always include impls
                        Item::Const(c) => Some(c.name.node.as_str()),
                        Item::Global(g) => Some(g.name.node.as_str()),
                        Item::Macro(m) => Some(m.name.node.as_str()),
                        Item::ExternBlock(_) => None, // Always include extern blocks
                        Item::Use(_) => None,          // Always include nested imports
                        Item::Error { .. } => None,
                    };
                    match item_name {
                        Some(name) => name_set.contains(name),
                        None => true, // Include unnamed items (impls, extern blocks, etc.)
                    }
                })
                .collect()
        }
    }
}

/// Load a module and recursively resolve its imports
pub(crate) fn load_module_with_imports(
    path: &PathBuf,
    loaded: &mut HashSet<PathBuf>,
    loading_stack: &mut Vec<PathBuf>,
    verbose: bool,
    query_db: &QueryDatabase,
) -> Result<Module, String> {
    let source =
        fs::read_to_string(path).map_err(|e| format!("Cannot read '{}': {}", path.display(), e))?;
    load_module_with_imports_internal(path, loaded, loading_stack, verbose, &source, query_db)
}

/// Internal function to load a module with source already read
pub(crate) fn load_module_with_imports_internal(
    path: &Path,
    loaded: &mut HashSet<PathBuf>,
    loading_stack: &mut Vec<PathBuf>,
    verbose: bool,
    source: &str,
    query_db: &QueryDatabase,
) -> Result<Module, String> {
    // Canonicalize path to avoid duplicate loading
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;

    // Check for circular imports (module already in loading stack)
    if loading_stack.contains(&canonical) {
        // Build error message showing the import chain
        let mut chain: Vec<String> = loading_stack
            .iter()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        chain.push(
            canonical
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );
        return Err(format!("Circular import detected: {}", chain.join(" → ")));
    }

    // Skip if already loaded (completed loading)
    if loaded.contains(&canonical) {
        return Ok(Module {
            items: vec![],
            modules_map: None,
        });
    }
    loaded.insert(canonical.clone());

    // Push to loading stack
    loading_stack.push(canonical.clone());

    // Use QueryDatabase for memoized parsing
    let cached = query_db.has_current_source(&canonical, source);
    query_db.set_source_text(&canonical, source);

    if verbose {
        let cache_tag = if cached { " (cached)" } else { "" };
        println!(
            "{} {}{}",
            "Compiling".green().bold(),
            path.display(),
            cache_tag
        );
    }

    let ast = query_db.parse(&canonical).map_err(|e| match e {
        vais_query::QueryError::Parse(msg) => {
            // Try to provide formatted error using the original parser
            match vais_parser::parse(source) {
                Err(parse_err) => error_formatter::format_parse_error(&parse_err, source, path),
                Ok(_) => msg,
            }
        }
        other => format!("Error in '{}': {}", path.display(), other),
    })?;

    if verbose {
        println!("  {} items", ast.items.len());
    }

    // Collect items, processing imports, and build modules_map for per-module codegen
    let mut all_items = Vec::new();
    let mut modules_map: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    let base_dir = path.parent().unwrap_or(Path::new("."));

    for item in ast.items.iter() {
        match &item.node {
            Item::Use(use_stmt) => {
                // Resolve import path
                let module_path = resolve_import_path(base_dir, &use_stmt.path)?;

                if verbose {
                    println!("  {} {}", "Importing".cyan(), module_path.display());
                }

                // Recursively load the imported module
                let imported = load_module_with_imports(
                    &module_path,
                    loaded,
                    loading_stack,
                    verbose,
                    query_db,
                )?;

                // Propagate sub-module mappings with offset, or create new mapping
                let offset = all_items.len();
                let filtered = filter_imported_items(
                    imported.items,
                    use_stmt.items.as_deref(),
                );
                let filtered_len = filtered.len();

                if let Some(sub_map) = imported.modules_map {
                    // Items are already tracked in sub_map — just remap indices
                    for (sub_path, sub_indices) in sub_map {
                        let remapped: Vec<usize> = sub_indices.iter().map(|i| i + offset).collect();

                        // Validate remapped indices will be within bounds after extending all_items
                        let future_total_len = offset + filtered_len;
                        for (&original_idx, &remapped_idx) in
                            sub_indices.iter().zip(remapped.iter())
                        {
                            if remapped_idx >= future_total_len && verbose {
                                eprintln!(
                                    "{}: Remapped index {} (original {} + offset {}) >= total items {} for module '{}'",
                                    "Warning".yellow(),
                                    remapped_idx,
                                    original_idx,
                                    offset,
                                    future_total_len,
                                    sub_path.display()
                                );
                            }
                        }

                        modules_map.entry(sub_path).or_default().extend(remapped);
                    }
                } else {
                    // No sub_map — record all items under the imported module path
                    let module_canonical = module_path.canonicalize().unwrap_or(module_path);
                    for i in 0..filtered_len {
                        modules_map
                            .entry(module_canonical.clone())
                            .or_default()
                            .push(offset + i);
                    }
                }
                all_items.extend(filtered);
            }
            _ => {
                modules_map
                    .entry(canonical.clone())
                    .or_default()
                    .push(all_items.len());
                all_items.push(item.clone());
            }
        }
    }

    // Pop from loading stack
    loading_stack.pop();

    Ok(Module {
        items: all_items,
        modules_map: Some(modules_map),
    })
}

/// Load a module with parallel parsing of imports
///
/// First pass: parse the main module to discover import paths.
/// Second pass: parse all imported modules in parallel using rayon.
/// Third pass: merge all items in correct order.
pub(crate) fn load_module_with_imports_parallel(
    path: &Path,
    loaded: &mut HashSet<PathBuf>,
    verbose: bool,
    source: &str,
    query_db: &QueryDatabase,
) -> Result<Module, String> {
    use rayon::prelude::*;

    // Canonicalize path
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;

    if loaded.contains(&canonical) {
        return Ok(Module {
            items: vec![],
            modules_map: None,
        });
    }
    loaded.insert(canonical.clone());

    // Use QueryDatabase for memoized parsing
    let cached = query_db.has_current_source(&canonical, source);
    query_db.set_source_text(&canonical, source);

    if verbose {
        let cache_tag = if cached { " (cached)" } else { "" };
        println!(
            "{} {} (parallel){}",
            "Compiling".green().bold(),
            path.display(),
            cache_tag
        );
    }

    let ast = query_db
        .parse(&canonical)
        .map_err(|e| format!("Error in '{}': {}", path.display(), e))?;

    if verbose {
        println!("  {} items", ast.items.len());
    }

    let base_dir = path.parent().unwrap_or(Path::new("."));

    // Phase 1: Collect all import paths first
    let mut import_paths: Vec<PathBuf> = Vec::new();
    let mut import_indices: Vec<usize> = Vec::new();

    for (idx, item) in ast.items.iter().enumerate() {
        if let Item::Use(use_stmt) = &item.node {
            let module_path = resolve_import_path(base_dir, &use_stmt.path)?;
            let module_canonical = module_path
                .canonicalize()
                .map_err(|e| format!("Cannot resolve path '{}': {}", module_path.display(), e))?;
            if !loaded.contains(&module_canonical) {
                import_paths.push(module_path);
                import_indices.push(idx);
                loaded.insert(module_canonical);
            }
        }
    }

    // Phase 2: Parse all imports in parallel using QueryDatabase
    #[allow(clippy::type_complexity)]
    let parsed_results: Vec<(PathBuf, Result<Module, String>)> = if import_paths.len() > 1 {
        if verbose {
            println!(
                "  {} Parsing {} imports in parallel",
                "⚡".cyan(),
                import_paths.len()
            );
        }
        import_paths
            .par_iter()
            .map(|p| {
                let result = (|| -> Result<Module, String> {
                    let src = fs::read_to_string(p)
                        .map_err(|e| format!("Cannot read '{}': {}", p.display(), e))?;
                    let p_canonical = p
                        .canonicalize()
                        .map_err(|e| format!("Cannot resolve path '{}': {}", p.display(), e))?;
                    query_db.set_source_text(&p_canonical, &src);
                    let module = query_db
                        .parse(&p_canonical)
                        .map_err(|e| format!("Error in '{}': {}", p.display(), e))?;
                    Ok(Module {
                        items: module.items.to_vec(),
                        modules_map: None,
                    })
                })();
                (p.clone(), result)
            })
            .collect()
    } else {
        import_paths
            .iter()
            .map(|p| {
                let result = (|| -> Result<Module, String> {
                    let src = fs::read_to_string(p)
                        .map_err(|e| format!("Cannot read '{}': {}", p.display(), e))?;
                    let p_canonical = p
                        .canonicalize()
                        .map_err(|e| format!("Cannot resolve path '{}': {}", p.display(), e))?;
                    query_db.set_source_text(&p_canonical, &src);
                    let module = query_db
                        .parse(&p_canonical)
                        .map_err(|e| format!("Error in '{}': {}", p.display(), e))?;
                    Ok(Module {
                        items: module.items.to_vec(),
                        modules_map: None,
                    })
                })();
                (p.clone(), result)
            })
            .collect()
    };

    // Build a map from path -> parsed module
    let mut parsed_map: std::collections::HashMap<PathBuf, vais_ast::Module> =
        std::collections::HashMap::new();
    for (import_path, result) in parsed_results {
        let parsed_module = result?;
        // Recursively resolve imports within each parsed module
        let sub_base = import_path.parent().unwrap_or(Path::new("."));
        let import_canonical = import_path
            .canonicalize()
            .unwrap_or_else(|_| import_path.clone());
        let mut sub_items = Vec::new();
        let mut sub_modules_map: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        for item in parsed_module.items {
            match &item.node {
                Item::Use(use_stmt) => {
                    let sub_path = resolve_import_path(sub_base, &use_stmt.path)?;
                    let mut sub_loading_stack = Vec::new();
                    let sub_imported = load_module_with_imports(
                        &sub_path,
                        loaded,
                        &mut sub_loading_stack,
                        verbose,
                        query_db,
                    )?;
                    let sub_canonical = sub_path.canonicalize().unwrap_or(sub_path);

                    let offset = sub_items.len();
                    let filtered = filter_imported_items(
                        sub_imported.items,
                        use_stmt.items.as_deref(),
                    );

                    // Propagate sub-module mappings or create new
                    if let Some(sub_map) = sub_imported.modules_map {
                        for (sp, si) in sub_map {
                            let remapped: Vec<usize> = si.iter().map(|i| i + offset).collect();
                            sub_modules_map.entry(sp).or_default().extend(remapped);
                        }
                    } else {
                        for i in 0..filtered.len() {
                            sub_modules_map
                                .entry(sub_canonical.clone())
                                .or_default()
                                .push(offset + i);
                        }
                    }
                    sub_items.extend(filtered);
                }
                _ => {
                    sub_modules_map
                        .entry(import_canonical.clone())
                        .or_default()
                        .push(sub_items.len());
                    sub_items.push(item);
                }
            }
        }
        parsed_map.insert(
            import_path,
            Module {
                items: sub_items,
                modules_map: Some(sub_modules_map),
            },
        );
    }

    // Phase 3: Merge items in correct order, building modules_map
    let mut all_items = Vec::new();
    let mut modules_map: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    let mut import_idx = 0;
    for (idx, item) in ast.items.iter().enumerate() {
        match &item.node {
            Item::Use(use_stmt) => {
                if import_idx < import_indices.len() && import_indices[import_idx] == idx {
                    if let Some(imported_module) = parsed_map.remove(&import_paths[import_idx]) {
                        let import_canonical = import_paths[import_idx]
                            .canonicalize()
                            .unwrap_or_else(|_| import_paths[import_idx].clone());

                        let offset = all_items.len();
                        let filtered = filter_imported_items(
                            imported_module.items,
                            use_stmt.items.as_deref(),
                        );

                        // Propagate sub-module mappings or create new
                        if let Some(sub_map) = imported_module.modules_map {
                            for (sub_path, sub_indices) in sub_map {
                                let remapped: Vec<usize> =
                                    sub_indices.iter().map(|i| i + offset).collect();
                                modules_map.entry(sub_path).or_default().extend(remapped);
                            }
                        } else {
                            for i in 0..filtered.len() {
                                modules_map
                                    .entry(import_canonical.clone())
                                    .or_default()
                                    .push(offset + i);
                            }
                        }
                        all_items.extend(filtered);
                    }
                    import_idx += 1;
                }
            }
            _ => {
                modules_map
                    .entry(canonical.clone())
                    .or_default()
                    .push(all_items.len());
                all_items.push(item.clone());
            }
        }
    }

    Ok(Module {
        items: all_items,
        modules_map: Some(modules_map),
    })
}

/// Get the standard library path
pub(crate) fn get_std_path() -> Option<PathBuf> {
    // Try multiple locations for std library:
    // 1. Relative to current executable (for installed vaisc)
    // 2. Current working directory (for development)
    // 3. VAIS_STD_PATH environment variable

    if let Ok(std_path) = std::env::var("VAIS_STD_PATH") {
        let path = PathBuf::from(std_path);
        if path.exists() {
            return Some(path);
        }
    }

    // Try relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let std_path = exe_dir.join("std");
            if std_path.exists() {
                return Some(std_path);
            }
            // Also try ../std (for cargo run)
            let std_path = exe_dir.parent().map(|p| p.join("std"));
            if let Some(path) = std_path {
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    // Try current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let std_path = cwd.join("std");
        if std_path.exists() {
            return Some(std_path);
        }
    }

    None
}

/// Resolve import path to file path with security validation
pub(crate) fn resolve_import_path(
    base_dir: &Path,
    path: &[vais_ast::Spanned<String>],
) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err("Empty import path".to_string());
    }

    // Check if this is a std library import (starts with "std")
    let is_std_import = path.first().map(|s| s.node.as_str()) == Some("std");

    let search_base = if is_std_import {
        // For std imports, use the standard library path
        match get_std_path() {
            Some(std_path) => std_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
            None => return Err(
                "Cannot find Vais standard library. Set VAIS_STD_PATH or run from project root."
                    .to_string(),
            ),
        }
    } else {
        base_dir.to_path_buf()
    };

    // Canonicalize the search base to get the absolute path
    // This resolves symlinks and normalizes the path
    let canonical_base = search_base
        .canonicalize()
        .map_err(|_| format!("Cannot resolve base directory: {}", search_base.display()))?;

    // Convert module path to file path
    // e.g., `U utils` -> `utils.vais` or `utils/mod.vais`
    // e.g., `U std/option` -> `std/option.vais`
    let mut file_path = search_base;
    for (i, segment) in path.iter().enumerate() {
        if i == path.len() - 1 {
            // Last segment - try as file first, then as directory with mod.vais
            let as_file = file_path.join(format!("{}.vais", segment.node));
            let as_dir = file_path.join(&segment.node).join("mod.vais");

            // Try file path first
            if as_file.exists() {
                return validate_and_canonicalize_import(&as_file, &canonical_base);
            } else if as_dir.exists() {
                return validate_and_canonicalize_import(&as_dir, &canonical_base);
            }

            // Fall back to dependency search paths (set by pkg build)
            if let Ok(dep_paths) = std::env::var("VAIS_DEP_PATHS") {
                for dep_dir in dep_paths.split(':') {
                    let dep_base = Path::new(dep_dir);
                    if !dep_base.exists() {
                        continue;
                    }
                    // Rebuild file path from the full import path segments
                    let mut dep_file_path = dep_base.to_path_buf();
                    for (j, seg) in path.iter().enumerate() {
                        if j == path.len() - 1 {
                            let dep_as_file = dep_file_path.join(format!("{}.vais", seg.node));
                            let dep_as_dir = dep_file_path.join(&seg.node).join("mod.vais");
                            let dep_as_lib = dep_file_path.join(&seg.node).join("lib.vais");
                            if dep_as_file.exists() {
                                let dep_canonical = dep_base.canonicalize().map_err(|_| {
                                    format!("Cannot resolve dep directory: {}", dep_base.display())
                                })?;
                                return validate_and_canonicalize_import(
                                    &dep_as_file,
                                    &dep_canonical,
                                );
                            } else if dep_as_dir.exists() {
                                let dep_canonical = dep_base.canonicalize().map_err(|_| {
                                    format!("Cannot resolve dep directory: {}", dep_base.display())
                                })?;
                                return validate_and_canonicalize_import(
                                    &dep_as_dir,
                                    &dep_canonical,
                                );
                            } else if dep_as_lib.exists() {
                                let dep_canonical = dep_base.canonicalize().map_err(|_| {
                                    format!("Cannot resolve dep directory: {}", dep_base.display())
                                })?;
                                return validate_and_canonicalize_import(
                                    &dep_as_lib,
                                    &dep_canonical,
                                );
                            }
                        } else {
                            dep_file_path = dep_file_path.join(&seg.node);
                        }
                    }
                }
            }

            return Err(format!(
                "Cannot find module '{}': tried '{}' and '{}'",
                segment.node,
                as_file.display(),
                as_dir.display()
            ));
        } else {
            file_path = file_path.join(&segment.node);
        }
    }

    Err("Invalid import path".to_string())
}

/// Validate and canonicalize an import path for security
///
/// This function performs critical security checks:
/// 1. Resolves the real path (following symlinks)
/// 2. Ensures the resolved path is within allowed directories
/// 3. Prevents path traversal attacks (../)
/// 4. Prevents symlink attacks
pub(crate) fn validate_and_canonicalize_import(
    path: &Path,
    allowed_base: &Path,
) -> Result<PathBuf, String> {
    // Canonicalize the path to resolve symlinks and get absolute path
    let canonical_path = path
        .canonicalize()
        .map_err(|e| format!("Cannot access file '{}': {}", path.display(), e))?;

    // Get the project root for additional validation
    let project_root = std::env::current_dir().and_then(|p| p.canonicalize()).ok();

    // Get std library path for validation
    let std_root = get_std_path()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .and_then(|p| p.canonicalize().ok());

    // Check if the canonical path is within allowed directories
    let is_within_allowed = canonical_path.starts_with(allowed_base);

    // Also check if it's within project root or std library
    let is_within_project = project_root
        .as_ref()
        .map(|root| canonical_path.starts_with(root))
        .unwrap_or(false);

    let is_within_std = std_root
        .as_ref()
        .map(|root| canonical_path.starts_with(root))
        .unwrap_or(false);

    // Check if within dependency cache paths (set by pkg build)
    let is_within_dep_cache = std::env::var("VAIS_DEP_PATHS")
        .ok()
        .map(|dep_paths| {
            dep_paths.split(':').any(|dp| {
                Path::new(dp)
                    .canonicalize()
                    .ok()
                    .map(|cp| canonical_path.starts_with(&cp))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if !is_within_allowed && !is_within_project && !is_within_std && !is_within_dep_cache {
        // Security: Path traversal or symlink attack detected
        return Err(format!(
            "Import path '{}' is outside allowed directories",
            path.display()
        ));
    }

    // Verify the file has .vais extension for additional safety
    if canonical_path
        .extension()
        .map(|e| e != "vais")
        .unwrap_or(true)
    {
        return Err(format!(
            "Invalid import file type: '{}' (only .vais files allowed)",
            canonical_path.display()
        ));
    }

    Ok(canonical_path)
}
