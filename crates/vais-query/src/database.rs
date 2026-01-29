//! Query-based compilation database with automatic memoization.
//!
//! Implements a Salsa-style incremental computation framework where:
//! - Source files are "inputs" that can be set/changed
//! - Derived queries (tokenize, parse, type_check, codegen) are memoized
//! - Changing an input invalidates all dependent cached results

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use sha2::{Digest, Sha256};

use vais_ast::Module;
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_lexer::SpannedToken;

use crate::revision::{Revision, RevisionCounter};

/// Result of a query, wrapping various compiler errors.
/// Query errors store string representations because upstream error types
/// (ParseError, TypeError, CodegenError) do not implement Clone.
#[derive(Debug, Clone)]
pub enum QueryError {
    Lex(String),
    Parse(String),
    Type(String),
    Codegen(String),
    FileNotFound(PathBuf),
    FileReadError(PathBuf, String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::Lex(e) => write!(f, "Lex error: {}", e),
            QueryError::Parse(e) => write!(f, "Parse error: {}", e),
            QueryError::Type(e) => write!(f, "Type error: {}", e),
            QueryError::Codegen(e) => write!(f, "Codegen error: {}", e),
            QueryError::FileNotFound(p) => write!(f, "File not found: {}", p.display()),
            QueryError::FileReadError(p, e) => {
                write!(f, "File read error: {}: {}", p.display(), e)
            }
        }
    }
}

impl std::error::Error for QueryError {}

pub type QueryResult<T> = Result<T, QueryError>;

/// Cached entry for a derived query, tagged with the revision when it was computed.
#[derive(Clone)]
struct CachedEntry<T> {
    value: T,
    /// The revision of the input when this entry was computed.
    input_revision: Revision,
}

/// Per-file cached compilation results.
struct FileCaches {
    tokens: Option<CachedEntry<QueryResult<Arc<Vec<SpannedToken>>>>>,
    ast: Option<CachedEntry<QueryResult<Arc<Module>>>>,
    type_checked: Option<CachedEntry<QueryResult<()>>>,
    ir: Option<CachedEntry<QueryResult<Arc<String>>>>,
    ir_target: Option<TargetTriple>,
}

impl FileCaches {
    fn new() -> Self {
        Self {
            tokens: None,
            ast: None,
            type_checked: None,
            ir: None,
            ir_target: None,
        }
    }

    fn invalidate(&mut self) {
        self.tokens = None;
        self.ast = None;
        self.type_checked = None;
        self.ir = None;
        self.ir_target = None;
    }
}

/// Input source file entry.
struct SourceInput {
    text: String,
    hash: [u8; 32],
    revision: Revision,
}

/// The main query database for incremental compilation.
///
/// Thread-safe: all access goes through `RwLock`-protected internal state.
///
/// # Usage
///
/// ```no_run
/// use vais_query::QueryDatabase;
/// use vais_codegen::TargetTriple;
///
/// let db = QueryDatabase::new();
///
/// // Set input sources
/// db.set_source_text("main.vais", "F main() -> i64 { 42 }");
///
/// // Run queries (automatically memoized)
/// let tokens = db.tokenize("main.vais").unwrap();
/// let ast = db.parse("main.vais").unwrap();
/// db.type_check("main.vais").unwrap();
/// let ir = db.generate_ir("main.vais", TargetTriple::Native).unwrap();
///
/// // Change source → only re-runs affected queries
/// db.set_source_text("main.vais", "F main() -> i64 { 100 }");
/// let new_ir = db.generate_ir("main.vais", TargetTriple::Native).unwrap();
/// ```
pub struct QueryDatabase {
    revision: RevisionCounter,
    sources: RwLock<HashMap<PathBuf, SourceInput>>,
    caches: RwLock<HashMap<PathBuf, FileCaches>>,
}

impl QueryDatabase {
    /// Create a new empty query database.
    pub fn new() -> Self {
        Self {
            revision: RevisionCounter::new(),
            sources: RwLock::new(HashMap::new()),
            caches: RwLock::new(HashMap::new()),
        }
    }

    /// Current global revision.
    pub fn current_revision(&self) -> Revision {
        self.revision.current()
    }

    // ─── Input Queries ───────────────────────────────────────────────

    /// Set the source text for a file. Increments the revision if the content changed.
    pub fn set_source_text(&self, path: impl AsRef<Path>, text: impl Into<String>) {
        let path = path.as_ref().to_path_buf();
        let text = text.into();
        let hash = Self::hash_source(&text);

        let mut sources = self.sources.write();

        // Check if content actually changed
        if let Some(existing) = sources.get(&path) {
            if existing.hash == hash {
                return; // No change, skip invalidation
            }
        }

        let rev = self.revision.increment();
        sources.insert(
            path.clone(),
            SourceInput {
                text,
                hash,
                revision: rev,
            },
        );

        // Invalidate all caches for this file
        let mut caches = self.caches.write();
        if let Some(cache) = caches.get_mut(&path) {
            cache.invalidate();
        }
    }

    /// Load a source file from disk and set it as input.
    pub fn load_source_file(&self, path: impl AsRef<Path>) -> QueryResult<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(QueryError::FileNotFound(path.to_path_buf()));
        }
        let text = std::fs::read_to_string(path).map_err(|e| {
            QueryError::FileReadError(path.to_path_buf(), e.to_string())
        })?;
        self.set_source_text(path, text);
        Ok(())
    }

    /// Get the source text for a file (returns None if not set).
    pub fn source_text(&self, path: impl AsRef<Path>) -> Option<String> {
        let sources = self.sources.read();
        sources.get(path.as_ref()).map(|s| s.text.clone())
    }

    /// Get the list of all registered source file paths.
    pub fn source_files(&self) -> Vec<PathBuf> {
        self.sources.read().keys().cloned().collect()
    }

    /// Remove a source file from the database.
    pub fn remove_source(&self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.sources.write().remove(path);
        self.caches.write().remove(path);
        self.revision.increment();
    }

    // ─── Derived Queries ─────────────────────────────────────────────

    /// Tokenize a source file. Results are memoized.
    pub fn tokenize(&self, path: impl AsRef<Path>) -> QueryResult<Arc<Vec<SpannedToken>>> {
        let path = path.as_ref().to_path_buf();
        let input_rev = self.input_revision(&path)?;

        // Check cache
        {
            let caches = self.caches.read();
            if let Some(file_cache) = caches.get(&path) {
                if let Some(ref entry) = file_cache.tokens {
                    if entry.input_revision == input_rev {
                        return entry.value.clone();
                    }
                }
            }
        }

        // Compute
        let source = self.source_text(&path).ok_or_else(|| {
            QueryError::FileNotFound(path.clone())
        })?;

        let result = vais_lexer::tokenize(&source)
            .map(Arc::new)
            .map_err(|e| QueryError::Lex(format!("{:?}", e)));

        // Store in cache
        self.store_cache(&path, input_rev, |cache| {
            cache.tokens = Some(CachedEntry {
                value: result.clone(),
                input_revision: input_rev,
            });
        });

        result
    }

    /// Parse a source file into an AST. Results are memoized.
    pub fn parse(&self, path: impl AsRef<Path>) -> QueryResult<Arc<Module>> {
        let path = path.as_ref().to_path_buf();
        let input_rev = self.input_revision(&path)?;

        // Check cache
        {
            let caches = self.caches.read();
            if let Some(file_cache) = caches.get(&path) {
                if let Some(ref entry) = file_cache.ast {
                    if entry.input_revision == input_rev {
                        return entry.value.clone();
                    }
                }
            }
        }

        // Compute (parse doesn't depend on tokenize output directly,
        // the parser re-tokenizes internally)
        let source = self.source_text(&path).ok_or_else(|| {
            QueryError::FileNotFound(path.clone())
        })?;

        let result = vais_parser::parse(&source)
            .map(Arc::new)
            .map_err(|e| QueryError::Parse(format!("{:?}", e)));

        // Store in cache
        self.store_cache(&path, input_rev, |cache| {
            cache.ast = Some(CachedEntry {
                value: result.clone(),
                input_revision: input_rev,
            });
        });

        result
    }

    /// Type-check a source file. Results are memoized.
    /// Returns Ok(()) if type checking succeeds.
    pub fn type_check(&self, path: impl AsRef<Path>) -> QueryResult<()> {
        let path = path.as_ref().to_path_buf();
        let input_rev = self.input_revision(&path)?;

        // Check cache
        {
            let caches = self.caches.read();
            if let Some(file_cache) = caches.get(&path) {
                if let Some(ref entry) = file_cache.type_checked {
                    if entry.input_revision == input_rev {
                        return entry.value.clone();
                    }
                }
            }
        }

        // First, parse (memoized)
        let module = self.parse(&path)?;

        // Type check
        let mut checker = vais_types::TypeChecker::new();
        let result = checker
            .check_module(&module)
            .map_err(|e| QueryError::Type(format!("{:?}", e)));

        // Store in cache
        self.store_cache(&path, input_rev, |cache| {
            cache.type_checked = Some(CachedEntry {
                value: result.clone(),
                input_revision: input_rev,
            });
        });

        result
    }

    /// Generate LLVM IR for a source file. Results are memoized.
    /// Invalidated if source changes or target changes.
    pub fn generate_ir(
        &self,
        path: impl AsRef<Path>,
        target: TargetTriple,
    ) -> QueryResult<Arc<String>> {
        let path = path.as_ref().to_path_buf();
        let input_rev = self.input_revision(&path)?;

        // Check cache (must also match target)
        {
            let caches = self.caches.read();
            if let Some(file_cache) = caches.get(&path) {
                if let Some(ref entry) = file_cache.ir {
                    let target_matches = file_cache
                        .ir_target
                        .as_ref()
                        .is_some_and(|t| *t == target);
                    if entry.input_revision == input_rev && target_matches {
                        return entry.value.clone();
                    }
                }
            }
        }

        // First, type check (memoized)
        self.type_check(&path)?;

        // Then parse again to get module (memoized)
        let module = self.parse(&path)?;

        // Generate IR
        let mut gen = CodeGenerator::new_with_target("main", target.clone());
        let result = gen
            .generate_module(&module)
            .map(Arc::new)
            .map_err(|e| QueryError::Codegen(format!("{:?}", e)));

        // Store in cache
        self.store_cache(&path, input_rev, |cache| {
            cache.ir = Some(CachedEntry {
                value: result.clone(),
                input_revision: input_rev,
            });
            cache.ir_target = Some(target);
        });

        result
    }

    // ─── Cache Statistics ────────────────────────────────────────────

    /// Get the number of cached files.
    pub fn cached_file_count(&self) -> usize {
        self.caches.read().len()
    }

    /// Get the number of source files.
    pub fn source_file_count(&self) -> usize {
        self.sources.read().len()
    }

    /// Clear all caches (inputs remain).
    pub fn clear_caches(&self) {
        self.caches.write().clear();
    }

    /// Clear everything (inputs and caches).
    pub fn clear_all(&self) {
        self.sources.write().clear();
        self.caches.write().clear();
    }

    /// Check if a query result is cached and valid for a file.
    pub fn is_cached(&self, path: impl AsRef<Path>, query: &str) -> bool {
        let path = path.as_ref();
        let input_rev = match self.input_revision(path) {
            Ok(rev) => rev,
            Err(_) => return false,
        };

        let caches = self.caches.read();
        let Some(file_cache) = caches.get(path) else {
            return false;
        };

        match query {
            "tokenize" => file_cache
                .tokens
                .as_ref()
                .is_some_and(|e| e.input_revision == input_rev),
            "parse" => file_cache
                .ast
                .as_ref()
                .is_some_and(|e| e.input_revision == input_rev),
            "type_check" => file_cache
                .type_checked
                .as_ref()
                .is_some_and(|e| e.input_revision == input_rev),
            "generate_ir" => file_cache
                .ir
                .as_ref()
                .is_some_and(|e| e.input_revision == input_rev),
            _ => false,
        }
    }

    // ─── Internal Helpers ────────────────────────────────────────────

    fn input_revision(&self, path: &Path) -> QueryResult<Revision> {
        let sources = self.sources.read();
        sources
            .get(path)
            .map(|s| s.revision)
            .ok_or_else(|| QueryError::FileNotFound(path.to_path_buf()))
    }

    fn store_cache(
        &self,
        path: &Path,
        _input_rev: Revision,
        f: impl FnOnce(&mut FileCaches),
    ) {
        let mut caches = self.caches.write();
        let file_cache = caches.entry(path.to_path_buf()).or_insert_with(FileCaches::new);
        f(file_cache);
    }

    fn hash_source(text: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hasher.finalize().into()
    }
}

impl Default for QueryDatabase {
    fn default() -> Self {
        Self::new()
    }
}
