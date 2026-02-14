use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::types::{FunctionMetadata, ModuleSignatureHash, TypeMetadata};
use super::cache::IncrementalCache;

/// Compute SHA256 hash of a file
pub fn compute_file_hash(path: &Path) -> Result<String, String> {
    let content =
        fs::read(path).map_err(|e| format!("Cannot read file '{}': {}", path.display(), e))?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

/// Compute SHA256 hash of a string (for function bodies, type definitions)
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Compute a signature hash for a set of AST items.
/// This captures the "public interface" — function signatures, struct fields,
/// enum variants, trait definitions — but NOT function bodies.
/// Used to determine if type checking results can be reused.
pub fn compute_signature_hash(items: &[vais_ast::Spanned<vais_ast::Item>]) -> String {
    use std::fmt::Write;
    let mut sig = String::new();

    for item in items {
        match &item.node {
            vais_ast::Item::Function(f) => {
                let _ = write!(sig, "fn:{}", f.name.node);
                for p in &f.params {
                    let _ = write!(sig, ",p:{}:{:?}", p.name.node, p.ty.node);
                }
                if let Some(ret) = &f.ret_type {
                    let _ = write!(sig, "->:{:?}", ret.node);
                }
                sig.push(';');
            }
            vais_ast::Item::Struct(s) => {
                let _ = write!(sig, "struct:{}", s.name.node);
                for f in &s.fields {
                    let _ = write!(sig, ",f:{}:{:?}", f.name.node, f.ty.node);
                }
                for m in &s.methods {
                    let _ = write!(sig, ",m:{}", m.node.name.node);
                    for p in &m.node.params {
                        let _ = write!(sig, ",mp:{}:{:?}", p.name.node, p.ty.node);
                    }
                    if let Some(ret) = &m.node.ret_type {
                        let _ = write!(sig, "->:{:?}", ret.node);
                    }
                }
                sig.push(';');
            }
            vais_ast::Item::Enum(e) => {
                let _ = write!(sig, "enum:{}", e.name.node);
                for v in &e.variants {
                    let _ = write!(sig, ",v:{}:{:?}", v.name.node, v.fields);
                }
                sig.push(';');
            }
            vais_ast::Item::Trait(t) => {
                let _ = write!(sig, "trait:{}", t.name.node);
                for m in &t.methods {
                    let _ = write!(sig, ",tm:{}", m.name.node);
                    for p in &m.params {
                        let _ = write!(sig, ",tp:{}:{:?}", p.name.node, p.ty.node);
                    }
                    if let Some(ret) = &m.ret_type {
                        let _ = write!(sig, "->:{:?}", ret.node);
                    }
                }
                sig.push(';');
            }
            vais_ast::Item::Impl(imp) => {
                let _ = write!(sig, "impl:{:?}", imp.target_type.node);
                if let Some(tn) = &imp.trait_name {
                    let _ = write!(sig, ":trait:{}", tn.node);
                }
                for m in &imp.methods {
                    let _ = write!(sig, ",im:{}", m.node.name.node);
                    for p in &m.node.params {
                        let _ = write!(sig, ",ip:{}:{:?}", p.name.node, p.ty.node);
                    }
                    if let Some(ret) = &m.node.ret_type {
                        let _ = write!(sig, "->:{:?}", ret.node);
                    }
                }
                sig.push(';');
            }
            vais_ast::Item::TypeAlias(ta) => {
                let _ = write!(sig, "type:{}={:?};", ta.name.node, ta.ty.node);
            }
            vais_ast::Item::TraitAlias(ta) => {
                let bounds: Vec<&str> = ta.bounds.iter().map(|b| b.node.as_str()).collect();
                let _ = write!(sig, "traitalias:{}={};", ta.name.node, bounds.join("+"));
            }
            vais_ast::Item::Const(c) => {
                let _ = write!(sig, "const:{}:{:?};", c.name.node, c.ty.node);
            }
            vais_ast::Item::Global(g) => {
                let _ = write!(sig, "global:{}:{:?};", g.name.node, g.ty.node);
            }
            vais_ast::Item::Union(u) => {
                let _ = write!(sig, "union:{}", u.name.node);
                for f in &u.fields {
                    let _ = write!(sig, ",f:{}:{:?}", f.name.node, f.ty.node);
                }
                sig.push(';');
            }
            vais_ast::Item::ExternBlock(eb) => {
                for f in &eb.functions {
                    let _ = write!(sig, "extern:{}:{}", eb.abi, f.name.node);
                    for p in &f.params {
                        let _ = write!(sig, ",ep:{}:{:?}", p.name.node, p.ty.node);
                    }
                    if let Some(ret) = &f.ret_type {
                        let _ = write!(sig, "->:{:?}", ret.node);
                    }
                    sig.push(';');
                }
            }
            _ => {} // Use, Macro, Error — don't affect signature
        }
    }

    compute_content_hash(&sig)
}

/// Check if type checking can be skipped based on cached signatures.
/// Returns true if ALL files have unchanged content hashes AND signature hashes,
/// meaning type checking results are still valid.
pub fn can_skip_type_checking(cache: &IncrementalCache, files: &[PathBuf]) -> bool {
    for file in files {
        let canonical = match file.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        match cache.state.dep_graph.file_metadata.get(&canonical) {
            Some(meta) => {
                // Check file content hash
                let current_hash = match compute_file_hash(&canonical) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                if current_hash != meta.hash {
                    return false;
                }
                // Check that TC previously passed for this file
                match &meta.signature_hash {
                    Some(sig) if sig.tc_passed => {}
                    _ => return false,
                }
            }
            None => return false, // File not in cache
        }
    }
    true
}

/// Update the signature hash and TC result for files in the cache.
/// Call this after successful type checking.
pub fn update_tc_cache(cache: &mut IncrementalCache, module: &vais_ast::Module, tc_passed: bool) {
    // If no modules_map, compute a single hash for the whole module
    if let Some(modules_map) = &module.modules_map {
        for (file_path, indices) in modules_map {
            let canonical = match file_path.canonicalize() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let file_items: Vec<vais_ast::Spanned<vais_ast::Item>> = indices
                .iter()
                .filter_map(|&i| module.items.get(i).cloned())
                .collect();

            let sig_hash = compute_signature_hash(&file_items);

            if let Some(meta) = cache.state.dep_graph.file_metadata.get_mut(&canonical) {
                meta.signature_hash = Some(ModuleSignatureHash {
                    hash: sig_hash,
                    tc_passed,
                });
            }
        }
    }
}

/// Get the path for a cached object file based on IR content hash.
/// This enables skipping `clang -c` when the generated IR hasn't changed.
pub fn get_ir_cached_object_path(cache_dir: &Path, ir_hash: &str, opt_level: u8) -> PathBuf {
    cache_dir.join(format!("ir_O{}_{}.o", opt_level, &ir_hash[..16]))
}

/// Check if a cached object file exists for the given IR hash.
pub fn has_ir_cached_object(cache_dir: &Path, ir_hash: &str, opt_level: u8) -> bool {
    get_ir_cached_object_path(cache_dir, ir_hash, opt_level).exists()
}

/// Function/type extractor for incremental compilation
pub struct DefinitionExtractor {
    /// Extracted function metadata
    pub functions: HashMap<String, FunctionMetadata>,
    /// Extracted type metadata
    pub types: HashMap<String, TypeMetadata>,
}

impl DefinitionExtractor {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// Extract definitions from source content
    /// This is a simplified parser that looks for function and type patterns
    pub fn extract_from_source(&mut self, content: &str) -> Result<(), String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut current_line = 0;

        while current_line < lines.len() {
            let line = lines[current_line].trim();

            // Function definition: F name(...) or F name<...>(...)
            if let Some(func_info) = self.try_parse_function(line, &lines, current_line) {
                let (name, start, end, body) = func_info;
                let hash = compute_content_hash(&body);
                let deps = self.extract_dependencies(&body);

                self.functions.insert(
                    name.clone(),
                    FunctionMetadata {
                        hash,
                        line_range: (start as u32, end as u32),
                        dependencies: deps,
                        is_dirty: false,
                    },
                );

                current_line = end + 1;
                continue;
            }

            // Struct definition: S name { ... }
            if let Some(type_info) = self.try_parse_struct(line, &lines, current_line) {
                let (name, start, end, body) = type_info;
                let hash = compute_content_hash(&body);
                let deps = self.extract_type_dependencies(&body);

                self.types.insert(
                    name.clone(),
                    TypeMetadata {
                        hash,
                        line_range: (start as u32, end as u32),
                        dependencies: deps,
                    },
                );

                current_line = end + 1;
                continue;
            }

            // Enum definition: E name { ... }
            if let Some(type_info) = self.try_parse_enum(line, &lines, current_line) {
                let (name, start, end, body) = type_info;
                let hash = compute_content_hash(&body);
                let deps = self.extract_type_dependencies(&body);

                self.types.insert(
                    name.clone(),
                    TypeMetadata {
                        hash,
                        line_range: (start as u32, end as u32),
                        dependencies: deps,
                    },
                );

                current_line = end + 1;
                continue;
            }

            current_line += 1;
        }

        Ok(())
    }

    /// Try to parse a function definition, returns (name, start_line, end_line, body)
    fn try_parse_function(
        &self,
        line: &str,
        lines: &[&str],
        start: usize,
    ) -> Option<(String, usize, usize, String)> {
        // Match patterns: "F name(", "F name<", "pub F name("
        let line_trimmed = line.trim_start_matches("pub ").trim();
        if !line_trimmed.starts_with("F ") {
            return None;
        }

        // Extract function name
        let after_f = line_trimmed[2..].trim();
        let name_end = after_f.find(['(', '<']).unwrap_or(after_f.len());
        let name = after_f[..name_end].trim().to_string();

        if name.is_empty() {
            return None;
        }

        // Find matching braces
        let (end_line, body) = self.find_block_end(lines, start)?;

        Some((name, start, end_line, body))
    }

    /// Try to parse a struct definition
    fn try_parse_struct(
        &self,
        line: &str,
        lines: &[&str],
        start: usize,
    ) -> Option<(String, usize, usize, String)> {
        let line_trimmed = line.trim_start_matches("pub ").trim();
        if !line_trimmed.starts_with("S ") {
            return None;
        }

        let after_s = line_trimmed[2..].trim();
        let name_end = after_s.find(['{', '<', ' ']).unwrap_or(after_s.len());
        let name = after_s[..name_end].trim().to_string();

        if name.is_empty() {
            return None;
        }

        let (end_line, body) = self.find_block_end(lines, start)?;
        Some((name, start, end_line, body))
    }

    /// Try to parse an enum definition
    fn try_parse_enum(
        &self,
        line: &str,
        lines: &[&str],
        start: usize,
    ) -> Option<(String, usize, usize, String)> {
        let line_trimmed = line.trim_start_matches("pub ").trim();
        if !line_trimmed.starts_with("E ") {
            return None;
        }

        let after_e = line_trimmed[2..].trim();
        let name_end = after_e.find(['{', '<', ' ']).unwrap_or(after_e.len());
        let name = after_e[..name_end].trim().to_string();

        if name.is_empty() {
            return None;
        }

        let (end_line, body) = self.find_block_end(lines, start)?;
        Some((name, start, end_line, body))
    }

    /// Find the end of a block (matching braces)
    fn find_block_end(&self, lines: &[&str], start: usize) -> Option<(usize, String)> {
        let mut brace_count = 0;
        let mut found_open = false;
        let mut body = String::new();

        for (i, line) in lines.iter().enumerate().skip(start) {
            body.push_str(line);
            body.push('\n');

            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_open = true;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }

            if found_open && brace_count == 0 {
                return Some((i, body));
            }
        }

        None
    }

    /// Extract function dependencies from body (called functions, used types)
    fn extract_dependencies(&self, body: &str) -> Vec<String> {
        let mut deps = Vec::new();

        // Simple pattern matching for function calls: name(
        // This is a simplified approach - a real implementation would use the AST
        let words: Vec<&str> = body
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .collect();

        for window in words.windows(1) {
            let word = window[0];
            // Skip Vais keywords
            if !is_vais_keyword(word)
                && word
                    .chars()
                    .next()
                    .map(|c| c.is_alphabetic())
                    .unwrap_or(false)
            {
                // Check if followed by ( in original body
                if (body.contains(&format!("{}(", word)) || body.contains(&format!("{}<", word)))
                    && !deps.contains(&word.to_string())
                {
                    deps.push(word.to_string());
                }
            }
        }

        deps
    }

    /// Extract type dependencies from type definition
    fn extract_type_dependencies(&self, body: &str) -> Vec<String> {
        let mut deps = Vec::new();

        // Look for type references: field: Type, Vec<Type>, etc.
        let words: Vec<&str> = body
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .collect();

        for word in words {
            // Type names start with uppercase (convention)
            if word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
                && !is_vais_keyword(word)
                && !is_builtin_type(word)
                && !deps.contains(&word.to_string())
            {
                deps.push(word.to_string());
            }
        }

        deps
    }
}

impl Default for DefinitionExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a word is a Vais keyword
fn is_vais_keyword(word: &str) -> bool {
    matches!(
        word,
        "F" | "S"
            | "E"
            | "T"
            | "I"
            | "M"
            | "N"
            | "C"
            | "V"
            | "L"
            | "W"
            | "R"
            | "B"
            | "P"
            | "if"
            | "else"
            | "for"
            | "while"
            | "return"
            | "break"
            | "continue"
            | "true"
            | "false"
            | "self"
            | "Self"
            | "pub"
            | "mut"
            | "async"
            | "await"
            | "import"
            | "from"
            | "as"
            | "match"
            | "spawn"
            | "defer"
    )
}

/// Check if a type is a builtin type
fn is_builtin_type(word: &str) -> bool {
    matches!(
        word,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "f32"
            | "f64"
            | "bool"
            | "str"
            | "String"
            | "Vec"
            | "HashMap"
            | "HashSet"
            | "Option"
            | "Result"
            | "Box"
            | "Rc"
            | "Arc"
            | "RefCell"
            | "Mutex"
    )
}

/// Compare function metadata and detect changes
pub fn detect_function_changes(
    old_meta: &HashMap<String, FunctionMetadata>,
    new_meta: &HashMap<String, FunctionMetadata>,
) -> FunctionChangeSet {
    let mut change_set = FunctionChangeSet::default();

    // Find added and modified functions
    for (name, new_fn) in new_meta {
        if let Some(old_fn) = old_meta.get(name) {
            if old_fn.hash != new_fn.hash {
                change_set.modified.insert(name.clone());
            }
        } else {
            change_set.added.insert(name.clone());
        }
    }

    // Find removed functions
    for name in old_meta.keys() {
        if !new_meta.contains_key(name) {
            change_set.removed.insert(name.clone());
        }
    }

    // Find affected functions (functions that depend on changed functions)
    let all_changed: HashSet<_> = change_set
        .modified
        .iter()
        .chain(change_set.added.iter())
        .chain(change_set.removed.iter())
        .cloned()
        .collect();

    for (name, func) in new_meta {
        if all_changed.contains(name) {
            continue;
        }
        for dep in &func.dependencies {
            if all_changed.contains(dep) {
                change_set.affected.insert(name.clone());
                break;
            }
        }
    }

    change_set
}

/// Set of function changes
#[derive(Debug, Default)]
pub struct FunctionChangeSet {
    /// Newly added functions
    pub added: HashSet<String>,
    /// Modified functions (hash changed)
    pub modified: HashSet<String>,
    /// Removed functions
    pub removed: HashSet<String>,
    /// Functions affected by changes (through dependencies)
    pub affected: HashSet<String>,
}

impl FunctionChangeSet {
    /// Check if there are any changes
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.affected.is_empty()
    }

    /// Get all functions that need recompilation
    pub fn all_dirty(&self) -> HashSet<String> {
        let mut all = self.added.clone();
        all.extend(self.modified.clone());
        all.extend(self.affected.clone());
        all
    }

    /// Total count of changes
    pub fn count(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len() + self.affected.len()
    }
}

/// Import tracker for collecting dependencies during parsing
#[derive(Default)]
pub struct ImportTracker {
    /// Current file being parsed
    pub current_file: Option<PathBuf>,
    /// Collected imports: from_file -> to_files
    pub imports: HashMap<PathBuf, Vec<PathBuf>>,
}

impl ImportTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking imports for a file
    pub fn start_file(&mut self, path: PathBuf) {
        self.current_file = Some(path.clone());
        self.imports.entry(path).or_default();
    }

    /// Record an import
    pub fn add_import(&mut self, imported_path: PathBuf) {
        if let Some(current) = &self.current_file {
            self.imports
                .entry(current.clone())
                .or_default()
                .push(imported_path);
        }
    }

    /// Finish tracking and get all imports
    pub fn finish(self) -> HashMap<PathBuf, Vec<PathBuf>> {
        self.imports
    }
}

/// Determine cache directory for a given source file
pub fn get_cache_dir(source_file: &Path) -> PathBuf {
    // Use parent directory of source file, or current directory
    let base = source_file.parent().unwrap_or(Path::new("."));
    base.join(".vais-cache")
}
