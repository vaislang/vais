//! Symbol table for type checking
//!
//! Manages symbol definitions and lookups across scopes.

use aoel_ast::Type;
use aoel_lexer::Span;
use std::collections::HashMap;

/// Scope levels in AOEL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeLevel {
    /// Global scope (built-in functions)
    Global,
    /// INPUT block scope
    Input,
    /// OUTPUT block scope
    Output,
    /// FLOW block scope (nodes)
    Flow,
}

/// Symbol kinds
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// INPUT field
    InputField,
    /// OUTPUT field
    OutputField,
    /// FLOW node
    FlowNode {
        op_type: String,
    },
    /// Built-in function
    BuiltinFunction {
        param_count: usize,
        returns_bool: bool,
    },
}

/// A symbol entry in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub ty: Option<Type>,
    pub span: Span,
}

impl Symbol {
    pub fn new(name: String, kind: SymbolKind, ty: Option<Type>, span: Span) -> Self {
        Self { name, kind, ty, span }
    }

    pub fn input_field(name: String, ty: Type, span: Span) -> Self {
        Self::new(name, SymbolKind::InputField, Some(ty), span)
    }

    pub fn output_field(name: String, ty: Type, span: Span) -> Self {
        Self::new(name, SymbolKind::OutputField, Some(ty), span)
    }

    pub fn flow_node(name: String, op_type: String, span: Span) -> Self {
        Self::new(name, SymbolKind::FlowNode { op_type }, None, span)
    }

    pub fn builtin(name: &str, param_count: usize, returns_bool: bool) -> Self {
        Self::new(
            name.to_string(),
            SymbolKind::BuiltinFunction { param_count, returns_bool },
            None,
            Span::new(0, 0),
        )
    }
}

/// Symbol table with scoped symbol management
#[derive(Debug)]
pub struct SymbolTable {
    /// Symbols organized by scope
    scopes: HashMap<ScopeLevel, HashMap<String, Symbol>>,
    /// Built-in functions
    builtins: HashMap<String, Symbol>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            scopes: HashMap::new(),
            builtins: HashMap::new(),
        };
        table.register_builtins();
        table
    }

    /// Register built-in functions
    fn register_builtins(&mut self) {
        // LEN(collection) -> INT
        self.builtins.insert(
            "LEN".to_string(),
            Symbol::builtin("LEN", 1, false),
        );

        // CONTAINS(collection, element) -> BOOL
        self.builtins.insert(
            "CONTAINS".to_string(),
            Symbol::builtin("CONTAINS", 2, true),
        );

        // SUM(array) -> Numeric
        self.builtins.insert(
            "SUM".to_string(),
            Symbol::builtin("SUM", 1, false),
        );

        // COUNT(array) -> INT
        self.builtins.insert(
            "COUNT".to_string(),
            Symbol::builtin("COUNT", 1, false),
        );

        // AVG(array) -> FLOAT
        self.builtins.insert(
            "AVG".to_string(),
            Symbol::builtin("AVG", 1, false),
        );

        // MIN(array) -> element type
        self.builtins.insert(
            "MIN".to_string(),
            Symbol::builtin("MIN", 1, false),
        );

        // MAX(array) -> element type
        self.builtins.insert(
            "MAX".to_string(),
            Symbol::builtin("MAX", 1, false),
        );

        // RANGE(start, end) -> ARRAY<INT>
        self.builtins.insert(
            "RANGE".to_string(),
            Symbol::builtin("RANGE", 2, false),
        );

        // MATCH(string, pattern) -> BOOL
        self.builtins.insert(
            "MATCH".to_string(),
            Symbol::builtin("MATCH", 2, true),
        );

        // KEYS(map) -> ARRAY<key_type>
        self.builtins.insert(
            "KEYS".to_string(),
            Symbol::builtin("KEYS", 1, false),
        );

        // VALUES(map) -> ARRAY<value_type>
        self.builtins.insert(
            "VALUES".to_string(),
            Symbol::builtin("VALUES", 1, false),
        );
    }

    /// Define a symbol in the given scope
    /// Returns the previous symbol if one existed with the same name
    pub fn define(&mut self, scope: ScopeLevel, symbol: Symbol) -> Option<Symbol> {
        self.scopes
            .entry(scope)
            .or_default()
            .insert(symbol.name.clone(), symbol)
    }

    /// Lookup a symbol by name, checking built-ins first then scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Check built-ins first
        if let Some(sym) = self.builtins.get(name) {
            return Some(sym);
        }

        // Check scopes in order: Flow -> Output -> Input -> Global
        for scope in [
            ScopeLevel::Flow,
            ScopeLevel::Output,
            ScopeLevel::Input,
            ScopeLevel::Global,
        ] {
            if let Some(symbols) = self.scopes.get(&scope) {
                if let Some(sym) = symbols.get(name) {
                    return Some(sym);
                }
            }
        }
        None
    }

    /// Lookup a symbol in a specific scope only
    pub fn lookup_in_scope(&self, scope: ScopeLevel, name: &str) -> Option<&Symbol> {
        self.scopes.get(&scope)?.get(name)
    }

    /// Check if a symbol exists in any scope or builtins
    pub fn exists(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Check if a builtin function exists
    pub fn is_builtin(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }

    /// Get all symbols in a scope
    pub fn symbols_in_scope(&self, scope: ScopeLevel) -> Vec<&Symbol> {
        self.scopes
            .get(&scope)
            .map(|s| s.values().collect())
            .unwrap_or_default()
    }

    /// Get all symbol names for suggestions
    pub fn all_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .scopes
            .values()
            .flat_map(|s| s.keys().cloned())
            .collect();
        names.extend(self.builtins.keys().cloned());
        names.sort();
        names.dedup();
        names
    }

    /// Get all symbol names in a specific scope
    pub fn names_in_scope(&self, scope: ScopeLevel) -> Vec<String> {
        self.scopes
            .get(&scope)
            .map(|s| s.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// Calculate Levenshtein distance between two strings
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

/// Find similar names for suggestions
pub fn find_similar_names(name: &str, candidates: &[String], max_distance: usize) -> Vec<String> {
    let mut suggestions: Vec<(String, usize)> = candidates
        .iter()
        .map(|c| (c.clone(), levenshtein_distance(name, c)))
        .filter(|(_, dist)| *dist <= max_distance)
        .collect();

    suggestions.sort_by_key(|(_, dist)| *dist);
    suggestions.into_iter().take(3).map(|(s, _)| s).collect()
}
