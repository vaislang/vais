//! Procedural Macro Framework for Vais
//!
//! Provides the infrastructure for procedural macros that operate on token streams
//! and produce token streams. Three kinds of proc macros are supported:
//!
//! - **Function-like**: `my_macro!(input)` → output tokens
//! - **Derive**: `#[derive(MyDerive)]` → additional impl items
//! - **Attribute**: `#[my_attr] fn foo()` → transformed function

use std::collections::HashMap;
use std::fmt;
use vais_ast::Span;

/// Token stream for proc macro input/output
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStream {
    tokens: Vec<TokenTree>,
}

/// Individual token in a token stream
#[derive(Debug, Clone, PartialEq)]
pub enum TokenTree {
    /// Identifier or keyword
    Ident(String),
    /// Literal value (integer, float, string, char)
    Literal(LiteralToken),
    /// Punctuation character(s)
    Punct(char),
    /// Delimited group: `(...)`, `[...]`, `{...}`
    Group {
        delimiter: Delimiter,
        stream: TokenStream,
    },
}

/// Delimiter type for grouped tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Parenthesis, // ( )
    Bracket,     // [ ]
    Brace,       // { }
}

/// Literal token value
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralToken {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
}

impl TokenStream {
    /// Create an empty token stream
    pub fn new() -> Self {
        Self { tokens: vec![] }
    }

    /// Create a token stream from a vector of token trees
    pub fn from_tokens(tokens: Vec<TokenTree>) -> Self {
        Self { tokens }
    }

    /// Check if the stream is empty
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Get the number of tokens
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Get an iterator over the tokens
    pub fn iter(&self) -> impl Iterator<Item = &TokenTree> {
        self.tokens.iter()
    }

    /// Append a token tree
    pub fn push(&mut self, token: TokenTree) {
        self.tokens.push(token);
    }

    /// Extend with another token stream
    pub fn extend(&mut self, other: TokenStream) {
        self.tokens.extend(other.tokens);
    }

    /// Convert to a string representation
    pub fn to_source(&self) -> String {
        let mut parts = Vec::new();
        for token in &self.tokens {
            parts.push(token.to_source());
        }
        parts.join(" ")
    }
}

impl Default for TokenStream {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_source())
    }
}

impl TokenTree {
    /// Convert to source code string
    pub fn to_source(&self) -> String {
        match self {
            TokenTree::Ident(s) => s.clone(),
            TokenTree::Literal(lit) => lit.to_source(),
            TokenTree::Punct(c) => c.to_string(),
            TokenTree::Group { delimiter, stream } => {
                let (open, close) = match delimiter {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::Brace => ("{", "}"),
                };
                format!("{}{}{}", open, stream.to_source(), close)
            }
        }
    }
}

impl LiteralToken {
    pub fn to_source(&self) -> String {
        match self {
            LiteralToken::Integer(n) => n.to_string(),
            LiteralToken::Float(f) => f.to_string(),
            LiteralToken::String(s) => format!("\"{}\"", s),
            LiteralToken::Char(c) => format!("'{}'", c),
            LiteralToken::Bool(b) => b.to_string(),
        }
    }
}

/// Error from a procedural macro expansion
#[derive(Debug, Clone)]
pub struct ProcMacroError {
    pub message: String,
    pub span: Option<Span>,
}

impl ProcMacroError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }
}

impl fmt::Display for ProcMacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "proc macro error: {}", self.message)
    }
}

impl std::error::Error for ProcMacroError {}

pub type ProcMacroResult<T> = Result<T, ProcMacroError>;

/// Kind of procedural macro
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcMacroKind {
    /// Function-like: `my_macro!(input)`
    FunctionLike,
    /// Derive: `#[derive(MyDerive)]` on structs/enums
    Derive,
    /// Attribute: `#[my_attr]` on items
    Attribute,
}

/// Trait for implementing procedural macros
pub trait ProcMacro: Send + Sync {
    /// The name of this macro
    fn name(&self) -> &str;

    /// The kind of this macro
    fn kind(&self) -> ProcMacroKind;

    /// Expand a function-like macro: `name!(input)` → output
    fn expand(&self, input: TokenStream) -> ProcMacroResult<TokenStream> {
        let _ = input;
        Err(ProcMacroError::new(format!(
            "macro `{}` does not support function-like invocation",
            self.name()
        )))
    }

    /// Expand a derive macro: generates additional items from struct/enum definition
    fn derive(&self, input: TokenStream) -> ProcMacroResult<TokenStream> {
        let _ = input;
        Err(ProcMacroError::new(format!(
            "macro `{}` does not support derive invocation",
            self.name()
        )))
    }

    /// Expand an attribute macro: `#[name(args)]` applied to an item
    fn attribute(&self, args: TokenStream, input: TokenStream) -> ProcMacroResult<TokenStream> {
        let _ = (args, input);
        Err(ProcMacroError::new(format!(
            "macro `{}` does not support attribute invocation",
            self.name()
        )))
    }
}

/// Registry for procedural macros
pub struct ProcMacroRegistry {
    macros: HashMap<String, Box<dyn ProcMacro>>,
}

impl ProcMacroRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        let mut registry = Self {
            macros: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    /// Register a procedural macro
    pub fn register(&mut self, proc_macro: Box<dyn ProcMacro>) {
        self.macros
            .insert(proc_macro.name().to_string(), proc_macro);
    }

    /// Look up a proc macro by name
    pub fn get(&self, name: &str) -> Option<&dyn ProcMacro> {
        self.macros.get(name).map(|m| m.as_ref())
    }

    /// Check if a proc macro exists
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Get the number of registered proc macros
    pub fn len(&self) -> usize {
        self.macros.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.macros.is_empty()
    }

    /// List all registered proc macro names
    pub fn names(&self) -> Vec<&str> {
        self.macros.keys().map(|s| s.as_str()).collect()
    }

    /// Register built-in procedural macros
    fn register_builtins(&mut self) {
        self.register(Box::new(StringifyMacro));
        self.register(Box::new(ConcatMacro));
        self.register(Box::new(EnvMacro));
        self.register(Box::new(LineMacro));
        self.register(Box::new(FileMacro));
        self.register(Box::new(ColumnMacro));
    }
}

impl Default for ProcMacroRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Built-in Procedural Macros ====================

/// `stringify!(expr)` - Converts tokens to a string literal
struct StringifyMacro;

impl ProcMacro for StringifyMacro {
    fn name(&self) -> &str {
        "stringify"
    }
    fn kind(&self) -> ProcMacroKind {
        ProcMacroKind::FunctionLike
    }

    fn expand(&self, input: TokenStream) -> ProcMacroResult<TokenStream> {
        let source = input.to_source();
        Ok(TokenStream::from_tokens(vec![TokenTree::Literal(
            LiteralToken::String(source),
        )]))
    }
}

/// `concat!(a, b, c)` - Concatenates string literals
struct ConcatMacro;

impl ProcMacro for ConcatMacro {
    fn name(&self) -> &str {
        "concat"
    }
    fn kind(&self) -> ProcMacroKind {
        ProcMacroKind::FunctionLike
    }

    fn expand(&self, input: TokenStream) -> ProcMacroResult<TokenStream> {
        let mut result = String::new();
        for token in input.iter() {
            match token {
                TokenTree::Literal(LiteralToken::String(s)) => result.push_str(s),
                TokenTree::Literal(LiteralToken::Integer(n)) => result.push_str(&n.to_string()),
                TokenTree::Punct(',') => {} // Skip commas
                _ => result.push_str(&token.to_source()),
            }
        }
        Ok(TokenStream::from_tokens(vec![TokenTree::Literal(
            LiteralToken::String(result),
        )]))
    }
}

/// `env!("VAR")` - Reads environment variable at compile time
struct EnvMacro;

impl ProcMacro for EnvMacro {
    fn name(&self) -> &str {
        "env"
    }
    fn kind(&self) -> ProcMacroKind {
        ProcMacroKind::FunctionLike
    }

    fn expand(&self, input: TokenStream) -> ProcMacroResult<TokenStream> {
        let var_name = match input.iter().next() {
            Some(TokenTree::Literal(LiteralToken::String(s))) => s.clone(),
            _ => return Err(ProcMacroError::new("env! expects a string literal")),
        };
        let value = std::env::var(&var_name).unwrap_or_default();
        Ok(TokenStream::from_tokens(vec![TokenTree::Literal(
            LiteralToken::String(value),
        )]))
    }
}

/// `line!()` - Returns the current line number
struct LineMacro;

impl ProcMacro for LineMacro {
    fn name(&self) -> &str {
        "line"
    }
    fn kind(&self) -> ProcMacroKind {
        ProcMacroKind::FunctionLike
    }

    fn expand(&self, _input: TokenStream) -> ProcMacroResult<TokenStream> {
        // Line number would be provided by the compiler in a real impl
        Ok(TokenStream::from_tokens(vec![TokenTree::Literal(
            LiteralToken::Integer(0),
        )]))
    }
}

/// `file!()` - Returns the current file name
struct FileMacro;

impl ProcMacro for FileMacro {
    fn name(&self) -> &str {
        "file"
    }
    fn kind(&self) -> ProcMacroKind {
        ProcMacroKind::FunctionLike
    }

    fn expand(&self, _input: TokenStream) -> ProcMacroResult<TokenStream> {
        // File name would be provided by the compiler in a real impl
        Ok(TokenStream::from_tokens(vec![TokenTree::Literal(
            LiteralToken::String("unknown".to_string()),
        )]))
    }
}

/// `column!()` - Returns the current column number
struct ColumnMacro;

impl ProcMacro for ColumnMacro {
    fn name(&self) -> &str {
        "column"
    }
    fn kind(&self) -> ProcMacroKind {
        ProcMacroKind::FunctionLike
    }

    fn expand(&self, _input: TokenStream) -> ProcMacroResult<TokenStream> {
        Ok(TokenStream::from_tokens(vec![TokenTree::Literal(
            LiteralToken::Integer(0),
        )]))
    }
}

// ==================== Token Stream Construction Helpers ====================

/// Helper to create an ident token
pub fn ident(name: impl Into<String>) -> TokenTree {
    TokenTree::Ident(name.into())
}

/// Helper to create an integer literal token
pub fn int_lit(value: i64) -> TokenTree {
    TokenTree::Literal(LiteralToken::Integer(value))
}

/// Helper to create a string literal token
pub fn str_lit(value: impl Into<String>) -> TokenTree {
    TokenTree::Literal(LiteralToken::String(value.into()))
}

/// Helper to create a punctuation token
pub fn punct(c: char) -> TokenTree {
    TokenTree::Punct(c)
}

/// Helper to create a grouped token stream
pub fn group(delimiter: Delimiter, stream: TokenStream) -> TokenTree {
    TokenTree::Group { delimiter, stream }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_stream_creation() {
        let stream = TokenStream::new();
        assert!(stream.is_empty());
        assert_eq!(stream.len(), 0);
    }

    #[test]
    fn test_token_stream_from_tokens() {
        let stream =
            TokenStream::from_tokens(vec![ident("hello"), punct('('), int_lit(42), punct(')')]);
        assert_eq!(stream.len(), 4);
        assert_eq!(stream.to_source(), "hello ( 42 )");
    }

    #[test]
    fn test_stringify_macro() {
        let mac = StringifyMacro;
        let input = TokenStream::from_tokens(vec![ident("x"), punct('+'), int_lit(1)]);
        let output = mac.expand(input).unwrap();
        assert_eq!(output.len(), 1);
        match &output.tokens[0] {
            TokenTree::Literal(LiteralToken::String(s)) => {
                assert_eq!(s, "x + 1");
            }
            _ => panic!("expected string literal"),
        }
    }

    #[test]
    fn test_concat_macro() {
        let mac = ConcatMacro;
        let input = TokenStream::from_tokens(vec![
            TokenTree::Literal(LiteralToken::String("hello".to_string())),
            punct(','),
            TokenTree::Literal(LiteralToken::String(" world".to_string())),
        ]);
        let output = mac.expand(input).unwrap();
        match &output.tokens[0] {
            TokenTree::Literal(LiteralToken::String(s)) => {
                assert_eq!(s, "hello world");
            }
            _ => panic!("expected string literal"),
        }
    }

    #[test]
    fn test_proc_macro_registry() {
        let registry = ProcMacroRegistry::new();
        assert!(registry.contains("stringify"));
        assert!(registry.contains("concat"));
        assert!(registry.contains("env"));
        assert!(registry.contains("line"));
        assert!(registry.contains("file"));
        assert!(registry.contains("column"));
        assert_eq!(registry.len(), 6);
    }

    #[test]
    fn test_proc_macro_kind() {
        let registry = ProcMacroRegistry::new();
        let mac = registry.get("stringify").unwrap();
        assert_eq!(mac.kind(), ProcMacroKind::FunctionLike);
    }

    #[test]
    fn test_custom_proc_macro() {
        struct MyMacro;
        impl ProcMacro for MyMacro {
            fn name(&self) -> &str {
                "my_macro"
            }
            fn kind(&self) -> ProcMacroKind {
                ProcMacroKind::FunctionLike
            }
            fn expand(&self, _input: TokenStream) -> ProcMacroResult<TokenStream> {
                Ok(TokenStream::from_tokens(vec![int_lit(42)]))
            }
        }

        let mut registry = ProcMacroRegistry::new();
        registry.register(Box::new(MyMacro));
        assert!(registry.contains("my_macro"));

        let mac = registry.get("my_macro").unwrap();
        let output = mac.expand(TokenStream::new()).unwrap();
        assert_eq!(output.to_source(), "42");
    }

    #[test]
    fn test_group_token() {
        let inner = TokenStream::from_tokens(vec![ident("x"), punct(','), ident("y")]);
        let grouped = group(Delimiter::Parenthesis, inner);
        assert_eq!(grouped.to_source(), "(x , y)");
    }

    #[test]
    fn test_derive_proc_macro_error() {
        let registry = ProcMacroRegistry::new();
        let mac = registry.get("stringify").unwrap();
        let result = mac.derive(TokenStream::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_attribute_proc_macro_error() {
        let registry = ProcMacroRegistry::new();
        let mac = registry.get("stringify").unwrap();
        let result = mac.attribute(TokenStream::new(), TokenStream::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_line_macro() {
        let mac = LineMacro;
        let output = mac.expand(TokenStream::new()).unwrap();
        assert_eq!(output.len(), 1);
        match &output.tokens[0] {
            TokenTree::Literal(LiteralToken::Integer(_)) => {}
            _ => panic!("expected integer literal"),
        }
    }

    #[test]
    fn test_file_macro() {
        let mac = FileMacro;
        let output = mac.expand(TokenStream::new()).unwrap();
        assert_eq!(output.len(), 1);
        match &output.tokens[0] {
            TokenTree::Literal(LiteralToken::String(s)) => {
                assert_eq!(s, "unknown");
            }
            _ => panic!("expected string literal"),
        }
    }

    #[test]
    fn test_column_macro() {
        let mac = ColumnMacro;
        let output = mac.expand(TokenStream::new()).unwrap();
        assert_eq!(output.len(), 1);
        match &output.tokens[0] {
            TokenTree::Literal(LiteralToken::Integer(_)) => {}
            _ => panic!("expected integer literal"),
        }
    }

    #[test]
    fn test_env_macro_with_existing_var() {
        let mac = EnvMacro;
        std::env::set_var("TEST_VAR_FOR_VAIS", "test_value");
        let input = TokenStream::from_tokens(vec![TokenTree::Literal(LiteralToken::String(
            "TEST_VAR_FOR_VAIS".to_string(),
        ))]);
        let output = mac.expand(input).unwrap();
        match &output.tokens[0] {
            TokenTree::Literal(LiteralToken::String(s)) => {
                assert_eq!(s, "test_value");
            }
            _ => panic!("expected string literal"),
        }
        std::env::remove_var("TEST_VAR_FOR_VAIS");
    }

    #[test]
    fn test_env_macro_with_missing_var() {
        let mac = EnvMacro;
        let input = TokenStream::from_tokens(vec![TokenTree::Literal(LiteralToken::String(
            "NONEXISTENT_VAR_12345".to_string(),
        ))]);
        let output = mac.expand(input).unwrap();
        match &output.tokens[0] {
            TokenTree::Literal(LiteralToken::String(s)) => {
                assert_eq!(s, "");
            }
            _ => panic!("expected string literal"),
        }
    }

    #[test]
    fn test_concat_with_numbers() {
        let mac = ConcatMacro;
        let input = TokenStream::from_tokens(vec![
            TokenTree::Literal(LiteralToken::String("count: ".to_string())),
            punct(','),
            TokenTree::Literal(LiteralToken::Integer(42)),
        ]);
        let output = mac.expand(input).unwrap();
        match &output.tokens[0] {
            TokenTree::Literal(LiteralToken::String(s)) => {
                assert_eq!(s, "count: 42");
            }
            _ => panic!("expected string literal"),
        }
    }

    #[test]
    fn test_token_stream_extend() {
        let mut stream1 = TokenStream::from_tokens(vec![ident("a"), punct(',')]);
        let stream2 = TokenStream::from_tokens(vec![ident("b")]);

        stream1.extend(stream2);
        assert_eq!(stream1.len(), 3);
        assert_eq!(stream1.to_source(), "a , b");
    }

    #[test]
    fn test_token_stream_push() {
        let mut stream = TokenStream::new();
        stream.push(ident("hello"));
        stream.push(punct('!'));

        assert_eq!(stream.len(), 2);
        assert_eq!(stream.to_source(), "hello !");
    }

    #[test]
    fn test_nested_groups() {
        let inner = TokenStream::from_tokens(vec![ident("x")]);
        let middle = TokenStream::from_tokens(vec![group(Delimiter::Parenthesis, inner)]);
        let outer = group(Delimiter::Bracket, middle);

        assert_eq!(outer.to_source(), "[(x)]");
    }

    #[test]
    fn test_literal_token_bool() {
        let lit = LiteralToken::Bool(true);
        assert_eq!(lit.to_source(), "true");

        let lit = LiteralToken::Bool(false);
        assert_eq!(lit.to_source(), "false");
    }

    #[test]
    fn test_literal_token_char() {
        let lit = LiteralToken::Char('a');
        assert_eq!(lit.to_source(), "'a'");
    }

    #[test]
    fn test_proc_macro_error_display() {
        let err = ProcMacroError::new("test error");
        assert_eq!(err.to_string(), "proc macro error: test error");
    }

    #[test]
    fn test_proc_macro_error_with_span() {
        let span = Span::new(10, 20);
        let err = ProcMacroError::with_span("test error", span);
        assert!(err.span.is_some());
        assert_eq!(err.message, "test error");
    }
}
