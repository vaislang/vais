//! Vais Parser
//!
//! Recursive descent parser for AI-optimized syntax.

mod error_display;
mod expr;
mod ffi;
mod item;
mod stmt;
mod types;

use thiserror::Error;
use vais_ast::*;
use vais_lexer::{SpannedToken, Token};

/// Error type for parsing failures.
///
/// Represents various kinds of syntax errors that can occur during parsing,
/// including unexpected tokens, premature EOF, and malformed expressions.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Unexpected token encountered during parsing
    #[error("Unexpected token {found:?} at {span:?}, expected {expected}")]
    UnexpectedToken {
        /// The token that was found
        found: Token,
        /// Source location of the unexpected token
        span: std::ops::Range<usize>,
        /// Description of what was expected
        expected: String,
    },
    /// Unexpected end of file while parsing
    #[error("Unexpected end of file")]
    UnexpectedEof {
        /// Location where EOF was encountered
        span: std::ops::Range<usize>,
    },
    /// Invalid or malformed expression
    #[error("Invalid expression")]
    InvalidExpression,
}

impl ParseError {
    /// Get the span associated with this error, if available
    pub fn span(&self) -> Option<std::ops::Range<usize>> {
        match self {
            ParseError::UnexpectedToken { span, .. } => Some(span.clone()),
            ParseError::UnexpectedEof { span } => Some(span.clone()),
            ParseError::InvalidExpression => None,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &str {
        match self {
            ParseError::UnexpectedToken { .. } => "P001",
            ParseError::UnexpectedEof { .. } => "P002",
            ParseError::InvalidExpression => "P003",
        }
    }

    /// Get the localized title for this error
    pub fn localized_title(&self) -> String {
        let key = format!("parse.{}.title", self.error_code());
        vais_i18n::get_simple(&key)
    }

    /// Get the localized message for this error
    pub fn localized_message(&self) -> String {
        let key = format!("parse.{}.message", self.error_code());
        match self {
            ParseError::UnexpectedToken {
                found, expected, ..
            } => vais_i18n::get(
                &key,
                &[("found", &format!("{:?}", found)), ("expected", expected)],
            ),
            ParseError::UnexpectedEof { .. } => vais_i18n::get_simple(&key),
            ParseError::InvalidExpression => vais_i18n::get_simple(&key),
        }
    }
}

type ParseResult<T> = Result<T, ParseError>;

/// Synchronization point tokens for error recovery.
/// These tokens mark natural boundaries in the source code where parsing can resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPoint {
    /// End of statement (`;` or `}`)
    Statement,
    /// End of expression (`,`, `)`, `]`, `}`)
    Expression,
    /// End of item (next top-level keyword)
    Item,
}

/// Recursive descent parser for Vais source code.
///
/// Converts a token stream into an Abstract Syntax Tree (AST).
/// Uses predictive parsing with single-token lookahead.
/// Supports error recovery to report multiple errors at once.
/// Maximum nesting depth for recursive parsing to prevent stack overflow
/// from deeply nested or malicious input.
const MAX_PARSE_DEPTH: usize = 256;

pub struct Parser {
    /// Token stream to parse
    tokens: Vec<SpannedToken>,
    /// Current position in the token stream
    pos: usize,
    /// Collected errors during parsing (for error recovery mode)
    errors: Vec<ParseError>,
    /// Whether error recovery mode is enabled
    recovery_mode: bool,
    /// Whether struct literals (Name{...}) are allowed in current context.
    /// Disabled when parsing loop/if conditions to avoid ambiguity with block start.
    allow_struct_literal: bool,
    /// Current recursion depth for nested expression parsing
    depth: usize,
    /// Source code for newline detection (used to prevent cross-line postfix parsing).
    /// Stored as String instead of &str to avoid lifetime parameters on Parser struct.
    /// The one-time allocation is acceptable as parsing happens once per file.
    source: String,
    /// Pre-computed newline byte positions for O(log n) newline detection.
    /// Built once from source; used by has_newline_between() to avoid repeated scanning.
    newline_positions: Vec<usize>,
    /// Compile-time cfg key-value pairs for conditional compilation.
    /// When set, items with `#[cfg(key = "value")]` are filtered out if they don't match.
    cfg_values: std::collections::HashMap<String, String>,
    /// Pending `>` count from splitting `>>` (Token::Shr) or `>>>` (Token::Shr + Token::Gt) in nested generics.
    /// When `Vec<Vec<Vec<i64>>>` is tokenized, multiple `>>` tokens appear.
    /// We split each into two `>` tokens: the first closes the inner generic,
    /// and this counter records how many additional `>` tokens are still pending.
    pending_gt_count: usize,
}

/// Build a sorted vec of byte positions where '\n' occurs in source.
/// Used for O(log n) newline-between queries via binary search.
fn build_newline_positions(source: &str) -> Vec<usize> {
    source
        .bytes()
        .enumerate()
        .filter_map(|(i, b)| if b == b'\n' { Some(i) } else { None })
        .collect()
}

impl Parser {
    /// Creates a new parser from a token stream.
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            recovery_mode: false,
            allow_struct_literal: true,
            depth: 0,
            source: String::new(),
            newline_positions: Vec::new(),
            cfg_values: std::collections::HashMap::new(),
            pending_gt_count: 0,
        }
    }

    /// Creates a new parser with source code for newline detection.
    ///
    /// Note: This clones the source string to avoid adding a lifetime parameter to Parser,
    /// which would require changes across ~94 usage sites. The allocation is acceptable
    /// since it happens once per file parse (not in a hot loop), and the source is only
    /// used for newline detection in postfix operator parsing.
    pub fn new_with_source(tokens: Vec<SpannedToken>, source: &str) -> Self {
        let newline_positions = build_newline_positions(source);
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            recovery_mode: false,
            allow_struct_literal: true,
            depth: 0,
            source: source.to_string(),
            newline_positions,
            cfg_values: std::collections::HashMap::new(),
            pending_gt_count: 0,
        }
    }

    /// Creates a new parser with error recovery enabled.
    ///
    /// In recovery mode, the parser will try to continue after errors,
    /// inserting Error nodes into the AST and collecting all errors.
    pub fn new_with_recovery(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            recovery_mode: true,
            allow_struct_literal: true,
            depth: 0,
            source: String::new(),
            newline_positions: Vec::new(),
            cfg_values: std::collections::HashMap::new(),
            pending_gt_count: 0,
        }
    }

    /// Set cfg key-value pairs for conditional compilation filtering.
    /// Items annotated with `#[cfg(key = "value")]` will be included only
    /// if the cfg values match.
    pub fn set_cfg_values(&mut self, values: std::collections::HashMap<String, String>) {
        self.cfg_values = values;
    }

    /// Check if there is a newline between two byte positions in the source.
    /// Uses pre-computed newline positions with binary search for O(log n) lookup.
    #[inline]
    fn has_newline_between(&self, start: usize, end: usize) -> bool {
        if self.newline_positions.is_empty() {
            return false;
        }
        // Binary search for the first newline position >= start
        let idx = self.newline_positions.partition_point(|&pos| pos < start);
        // Check if that newline is within [start, end)
        idx < self.newline_positions.len() && self.newline_positions[idx] < end
    }

    /// Increment the recursion depth, returning an error if MAX_PARSE_DEPTH is exceeded.
    fn enter_depth(&mut self) -> ParseResult<()> {
        self.depth += 1;
        if self.depth > MAX_PARSE_DEPTH {
            let span = self.current_span();
            Err(ParseError::UnexpectedToken {
                found: self
                    .peek()
                    .map(|t| t.token.clone())
                    .unwrap_or(Token::Ident("EOF".to_string())),
                span,
                expected: format!(
                    "expression (maximum nesting depth of {} exceeded)",
                    MAX_PARSE_DEPTH
                ),
            })
        } else {
            Ok(())
        }
    }

    /// Decrement the recursion depth.
    fn exit_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Returns all errors collected during parsing.
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// Takes all collected errors, leaving the error list empty.
    pub fn take_errors(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }

    /// Returns whether any errors were encountered during parsing.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Records an error for later reporting (used in recovery mode).
    pub(crate) fn record_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    /// Synchronize to the next statement boundary for error recovery.
    /// Skips tokens until a statement boundary is found.
    /// Returns the list of skipped tokens (as strings for debugging).
    fn synchronize_statement(&mut self) -> Vec<String> {
        let mut skipped = Vec::new();

        while !self.is_at_end() {
            // Check if we're at a statement boundary
            if let Some(tok) = self.peek() {
                match &tok.token {
                    // Statement-ending tokens
                    Token::Semi | Token::RBrace => {
                        // Consume the boundary token if it's a semicolon
                        if tok.token == Token::Semi {
                            skipped.push(";".to_string());
                            self.advance();
                        }
                        break;
                    }
                    // Statement-starting tokens (keywords)
                    Token::Return
                    | Token::Break
                    | Token::Continue
                    | Token::Defer
                    | Token::If
                    | Token::Loop
                    | Token::Match => {
                        break;
                    }
                    // Item-level keywords - if we hit these, we've gone too far
                    // (likely missing a closing brace in the function body)
                    // Note: Token::Trait (W) excluded — also used as while loop keyword
                    Token::Function
                    | Token::Struct
                    | Token::Enum
                    | Token::Union
                    | Token::Use
                    | Token::Impl
                    | Token::Macro
                    | Token::Pub
                    | Token::Async
                    | Token::Extern => {
                        break;
                    }
                    _ => {
                        // Check for let statement (ident followed by := or :)
                        if let Token::Ident(_) = &tok.token {
                            if let Some(next) = self.peek_next() {
                                if matches!(next.token, Token::ColonEq | Token::Colon) {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Skip this token
            if let Some(tok) = self.advance() {
                skipped.push(format!("{:?}", tok.token));
            }
        }

        skipped
    }

    /// Synchronize to the next item boundary for error recovery.
    /// Skips tokens until a top-level item keyword is found.
    /// Returns the list of skipped tokens (as strings for debugging).
    fn synchronize_item(&mut self) -> Vec<String> {
        let mut skipped = Vec::new();

        while !self.is_at_end() {
            if let Some(tok) = self.peek() {
                match &tok.token {
                    // Top-level item keywords
                    // Note: Token::Trait (W) excluded — also used as while loop keyword
                    Token::Function
                    | Token::Struct
                    | Token::Enum
                    | Token::Union
                    | Token::TypeKeyword
                    | Token::Use
                    | Token::Impl
                    | Token::Macro
                    | Token::Pub
                    | Token::Async => {
                        break;
                    }
                    // Attributes can also start items
                    Token::HashBracket => {
                        break;
                    }
                    _ => {}
                }
            }

            // Skip this token
            if let Some(tok) = self.advance() {
                skipped.push(format!("{:?}", tok.token));
            }
        }

        skipped
    }

    /// Parses a complete module (top-level items).
    ///
    /// This is the main entry point for parsing. It consumes all tokens
    /// and produces a Module containing all top-level definitions.
    ///
    /// In recovery mode, parsing errors are collected and Error nodes are
    /// inserted into the AST. Use `errors()` to retrieve all collected errors.
    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let mut items = Vec::new();
        let mut describe_test_names: Vec<String> = Vec::new();

        while !self.is_at_end() {
            // Check for describe("...", |t| { ... }) test blocks
            if self.check_ident("describe") {
                match self.parse_describe_block() {
                    Ok(test_fns) => {
                        for (name, func) in test_fns {
                            describe_test_names.push(name);
                            items.push(func);
                        }
                        continue;
                    }
                    Err(e) => {
                        if self.recovery_mode {
                            let start = self.current_span().start;
                            let message = e.to_string();
                            self.record_error(e);
                            let skipped_tokens = self.synchronize_item();
                            let end = self.prev_span().end;
                            items.push(Spanned::new(
                                Item::Error {
                                    message,
                                    skipped_tokens,
                                },
                                Span::new(start, end),
                            ));
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                }
            }

            match self.parse_item() {
                Ok(item) => {
                    // Apply cfg filtering: skip items whose #[cfg(...)] doesn't match
                    if self.should_include_item(&item.node) {
                        items.push(item);
                    }
                }
                Err(e) => {
                    if self.recovery_mode {
                        // Record the error and create an Error node
                        let start = self.current_span().start;
                        let message = e.to_string();
                        self.record_error(e);

                        // Synchronize to next item boundary
                        let skipped_tokens = self.synchronize_item();

                        let end = self.prev_span().end;
                        items.push(Spanned::new(
                            Item::Error {
                                message,
                                skipped_tokens,
                            },
                            Span::new(start, end),
                        ));
                    } else {
                        // Not in recovery mode, propagate the error
                        return Err(e);
                    }
                }
            }
        }

        // If we found describe blocks, generate a main() function that calls all test functions
        if !describe_test_names.is_empty() {
            // Check if a main already exists
            let has_main = items
                .iter()
                .any(|item| matches!(&item.node, Item::Function(f) if f.name.node == "main"));
            if !has_main {
                let span = Span::new(0, 0);
                let mut stmts = Vec::new();
                for name in &describe_test_names {
                    // Generate: test_name();
                    stmts.push(Spanned::new(
                        Stmt::Expr(Box::new(Spanned::new(
                            Expr::Call {
                                func: Box::new(Spanned::new(Expr::Ident(name.clone()), span)),
                                args: vec![],
                            },
                            span,
                        ))),
                        span,
                    ));
                }
                // return 0
                stmts.push(Spanned::new(
                    Stmt::Return(Some(Box::new(Spanned::new(Expr::Int(0), span)))),
                    span,
                ));
                let main_fn = Function {
                    name: Spanned::new("main".to_string(), span),
                    generics: vec![],
                    params: vec![],
                    ret_type: Some(Spanned::new(
                        Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        span,
                    )),
                    body: FunctionBody::Block(stmts),
                    is_pub: false,
                    is_async: false,
                    attributes: vec![],
                    where_clause: vec![],
                };
                items.push(Spanned::new(Item::Function(main_fn), span));
            }
        }

        Ok(Module {
            items,
            modules_map: None,
        })
    }

    /// Parses a complete module with error recovery enabled.
    ///
    /// This is a convenience method that enables recovery mode and returns
    /// both the AST and any collected errors. Unlike `parse_module()`, this
    /// method always succeeds (unless there's an unrecoverable internal error),
    /// returning an AST with Error nodes for problematic sections.
    ///
    /// # Returns
    ///
    /// A tuple of `(Module, Vec<ParseError>)` containing the parsed module
    /// and all errors encountered during parsing.
    pub fn parse_module_with_recovery(&mut self) -> (Module, Vec<ParseError>) {
        self.recovery_mode = true;
        let module = self.parse_module().unwrap_or_else(|e| {
            // This shouldn't happen in recovery mode, but just in case
            self.record_error(e);
            Module {
                items: Vec::new(),
                modules_map: None,
            }
        });
        let errors = self.take_errors();
        (module, errors)
    }

    // === Cfg filtering ===

    /// Check if an item should be included based on its cfg attributes.
    /// Returns true if the item has no cfg attributes, or if all cfg conditions match.
    fn should_include_item(&self, item: &Item) -> bool {
        if self.cfg_values.is_empty() {
            return true;
        }
        let attrs = match item {
            Item::Function(f) => &f.attributes,
            Item::Struct(s) => &s.attributes,
            Item::Const(c) => &c.attributes,
            _ => return true,
        };
        self.check_cfg_attrs(attrs)
    }

    /// Check if cfg attributes match the current cfg_values.
    /// Returns true if no cfg attribute is present or if all cfg conditions match.
    fn check_cfg_attrs(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.name == "cfg" && !self.eval_cfg_condition(&attr.args) {
                return false;
            }
        }
        true
    }

    /// Evaluate a cfg condition from attribute args.
    /// Supports: `#[cfg(target_os = "linux")]` → args = ["target_os", "=", "linux"]
    /// Also supports simple flags: `#[cfg(test)]` → args = ["test"]
    /// Also supports `not(...)`: `#[cfg(not(target_os = "windows"))]` → args = ["not", "target_os", "=", "windows"]
    fn eval_cfg_condition(&self, args: &[String]) -> bool {
        if args.is_empty() {
            return true;
        }

        // Handle not(...) — parsed as ["not", key, "=", value]
        if args[0] == "not" {
            return !self.eval_cfg_condition(&args[1..]);
        }

        // Handle key = "value" pattern
        if args.len() >= 3 && args[1] == "=" {
            let key = &args[0];
            let value = &args[2];

            // Feature keys are stored as "feature:<name>" to support multiple features
            if key == "feature" {
                let feature_key = format!("feature:{}", value);
                return self.cfg_values.contains_key(&feature_key);
            }

            return self.cfg_values.get(key) == Some(value);
        }

        // Handle simple flag: cfg(test)
        if args.len() == 1 {
            return self.cfg_values.contains_key(&args[0]);
        }

        // Unknown pattern — include by default
        true
    }

    // === Helper methods ===

    /// Parse an identifier, also accepting single-character keyword tokens
    /// in positions where an identifier is expected (e.g., type names, struct names).
    /// This allows `S C { }` where C is a struct name, not Continue.
    pub(crate) fn parse_ident_or_keyword(&mut self) -> ParseResult<Spanned<String>> {
        // parse_ident() already accepts all single-char keyword tokens as identifiers,
        // so this is a thin wrapper that makes the intent explicit at call sites.
        self.parse_ident()
    }

    /// Convert a single-character keyword token to its string representation
    /// for use as an identifier. Returns None for non-keyword tokens.
    #[allow(dead_code)]
    pub(crate) fn keyword_to_ident(token: &Token) -> Option<&'static str> {
        match token {
            Token::Break => Some("B"),
            Token::Continue => Some("C"),
            Token::Enum => Some("E"),
            Token::Function => Some("F"),
            Token::Global => Some("G"),
            Token::If => Some("I"),
            Token::Loop => Some("L"),
            Token::Match => Some("M"),
            Token::Extern => Some("N"),
            Token::Union => Some("O"),
            Token::Pub => Some("P"),
            Token::Return => Some("R"),
            Token::Struct => Some("S"),
            Token::TypeKeyword => Some("T"),
            Token::Use => Some("U"),
            Token::Trait => Some("W"),
            Token::Impl => Some("X"),
            Token::Await => Some("Y"),
            Token::Async => Some("A"),
            Token::Defer => Some("D"),
            _ => None,
        }
    }

    pub(crate) fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn peek_next(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos + 1)
    }

    /// Save the current parser position for backtracking
    pub(crate) fn save_position(&self) -> usize {
        self.pos
    }

    /// Restore the parser to a previously saved position
    pub(crate) fn restore_position(&mut self, pos: usize) {
        self.pos = pos;
        self.pending_gt_count = 0;
    }

    /// Check if the current "token" is `>`, accounting for a pending `>`
    /// left over from splitting a `>>` (Token::Shr) in nested generic contexts.
    /// Also returns true for `>>` (Token::Shr) because `>>` will be split into
    /// two `>` tokens when consumed via `consume_gt()`.
    pub(crate) fn check_gt(&self) -> bool {
        if self.pending_gt_count > 0 {
            return true;
        }
        matches!(
            self.peek().map(|t| &t.token),
            Some(Token::Gt) | Some(Token::Shr)
        )
    }

    /// Consume a single `>`, which may either be:
    /// 1. A pending second `>` from a previously split `>>`, or
    /// 2. A real `Token::Gt` in the stream, or
    /// 3. A `Token::Shr` (`>>`) which we split: consume it and increment `pending_gt_count`
    ///    so the next `consume_gt()` call returns the second `>`.
    ///
    /// Returns a synthetic `>` SpannedToken in the pending-gt and Shr cases.
    pub(crate) fn consume_gt(&mut self) -> ParseResult<SpannedToken> {
        if self.pending_gt_count > 0 {
            self.pending_gt_count -= 1;
            // Return a synthetic Gt token at the current span
            let span = self.current_span();
            return Ok(SpannedToken {
                token: Token::Gt,
                span,
            });
        }
        if self.check(&Token::Gt) {
            return self.advance().ok_or_else(|| ParseError::UnexpectedEof {
                span: self.current_span(),
            });
        }
        if self.check(&Token::Shr) {
            // Split `>>` into two `>` tokens: consume it and remember one pending `>`
            let tok = self.advance().ok_or_else(|| ParseError::UnexpectedEof {
                span: self.current_span(),
            })?;
            self.pending_gt_count += 1;
            // Return a synthetic Gt with the span of the Shr token
            return Ok(SpannedToken {
                token: Token::Gt,
                span: tok.span,
            });
        }
        Err(ParseError::UnexpectedToken {
            found: self
                .peek()
                .map(|t| t.token.clone())
                .unwrap_or(Token::Ident("EOF".into())),
            span: self.current_span(),
            expected: "'>'".to_string(),
        })
    }

    /// Check if the current token is an identifier with the given name
    pub(crate) fn check_ident(&self, name: &str) -> bool {
        self.peek()
            .map(|t| matches!(&t.token, Token::Ident(s) if s == name))
            .unwrap_or(false)
    }

    pub(crate) fn advance(&mut self) -> Option<SpannedToken> {
        if self.is_at_end() {
            None
        } else {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        }
    }

    /// Advance the parser position without cloning the token.
    /// Use this when the token's content is not needed (just consuming it).
    #[inline]
    pub(crate) fn advance_skip(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    pub(crate) fn check(&self, expected: &Token) -> bool {
        self.peek().map(|t| &t.token == expected).unwrap_or(false)
    }

    pub(crate) fn expect(&mut self, expected: &Token) -> ParseResult<SpannedToken> {
        if self.check(expected) {
            self.advance().ok_or_else(|| ParseError::UnexpectedToken {
                found: Token::Ident("EOF".into()),
                span: self.current_span(),
                expected: Self::token_to_friendly_name(expected),
            })
        } else {
            Err(ParseError::UnexpectedToken {
                found: self
                    .peek()
                    .map(|t| t.token.clone())
                    .unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: Self::token_to_friendly_name(expected),
            })
        }
    }

    /// Expect a token and advance without cloning it.
    /// Use this when the token value is not needed (just ensuring syntax correctness).
    #[inline]
    pub(crate) fn expect_skip(&mut self, expected: &Token) -> ParseResult<()> {
        if self.check(expected) {
            self.advance_skip();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                found: self
                    .peek()
                    .map(|t| t.token.clone())
                    .unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: Self::token_to_friendly_name(expected),
            })
        }
    }

    /// Expect a closing delimiter with enhanced error recovery.
    /// When the expected closer is not found, records an error with the opening
    /// delimiter's location and attempts to synchronize.
    fn expect_closing(
        &mut self,
        expected: &Token,
        open_span: std::ops::Range<usize>,
    ) -> ParseResult<SpannedToken> {
        if self.check(expected) {
            return self.advance().ok_or_else(|| ParseError::UnexpectedEof {
                span: self.current_span(),
            });
        }

        // Create a descriptive error message
        let closer_name = Self::token_to_friendly_name(expected);
        let opener_name = match expected {
            Token::RParen => "'('",
            Token::RBrace => "'{'",
            Token::RBracket => "'['",
            Token::Gt => "'<'",
            _ => "opening delimiter",
        };

        let found = self
            .peek()
            .map(|t| t.token.clone())
            .unwrap_or(Token::Ident("EOF".into()));
        let span = self.current_span();

        let err = ParseError::UnexpectedToken {
            found,
            span: span.clone(),
            expected: format!(
                "{} to match {} at position {}",
                closer_name, opener_name, open_span.start
            ),
        };

        if self.recovery_mode {
            self.record_error(err);
            // Try to skip to the closing delimiter or a sync point
            self.skip_to_closing(expected);
            // Return a synthetic token
            Ok(SpannedToken {
                token: expected.clone(),
                span,
            })
        } else {
            Err(err)
        }
    }

    /// Skip tokens until we find the expected closing delimiter or a sync point.
    fn skip_to_closing(&mut self, expected: &Token) {
        let mut depth = 0i32;
        let opener = match expected {
            Token::RParen => Some(Token::LParen),
            Token::RBrace => Some(Token::LBrace),
            Token::RBracket => Some(Token::LBracket),
            Token::Gt => Some(Token::Lt),
            _ => None,
        };

        while !self.is_at_end() {
            if let Some(tok) = self.peek() {
                // Found matching closer at depth 0
                if depth == 0 && &tok.token == expected {
                    self.advance(); // consume the closer
                    return;
                }

                // Track nesting
                if let Some(ref open) = opener {
                    if &tok.token == open {
                        depth += 1;
                    } else if &tok.token == expected {
                        depth -= 1;
                        if depth < 0 {
                            self.advance();
                            return;
                        }
                    }
                }

                // Stop at statement/item boundaries if we can't find the closer
                match &tok.token {
                    Token::Semi => {
                        return; // Don't consume - let the caller handle it
                    }
                    Token::Function | Token::Struct | Token::Enum | Token::Trait | Token::Impl
                        if depth == 0 =>
                    {
                        return; // At a top-level item - stop
                    }
                    _ => {}
                }
            }
            self.advance();
        }
    }

    /// Convert a token to a user-friendly name for error messages
    fn token_to_friendly_name(token: &Token) -> String {
        error_display::token_to_friendly_name(token)
    }

    /// Parse a describe("name", |t| { it("test", || { body }); ... }); block
    /// and desugar it into individual test functions.
    ///
    /// Returns a list of (function_name, Item::Function) pairs.
    fn parse_describe_block(&mut self) -> ParseResult<Vec<(String, Spanned<Item>)>> {
        let start = self.current_span().start;

        // consume "describe"
        self.advance_skip();

        // expect "("
        self.expect_skip(&Token::LParen)?;

        // expect string literal for describe name
        let describe_name = if let Some(tok) = self.peek() {
            if let Token::String(s) = &tok.token {
                let name = s.clone();
                self.advance_skip();
                name
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: tok.token.clone(),
                    span: tok.span.clone(),
                    expected: "string literal for describe name".into(),
                });
            }
        } else {
            return Err(ParseError::UnexpectedEof {
                span: self.current_span(),
            });
        };

        // Handle two syntax forms:
        // Form 1 (VaisDB): describe("name") { ... }
        // Form 2 (closure): describe("name", |t| { ... })
        if self.check(&Token::RParen) {
            // Form 1: consume ")" then expect "{"
            self.advance_skip();
            self.expect_skip(&Token::LBrace)?;
        } else if self.check(&Token::Comma) {
            // Form 2: consume "," then skip closure params until "{"
            self.advance_skip();
            let mut brace_depth = 0;
            while !self.is_at_end() {
                if self.check(&Token::LBrace)
                    && brace_depth == 0 {
                        break;
                    }
                if let Some(tok) = self.peek() {
                    if tok.token == Token::LBrace {
                        brace_depth += 1;
                    } else if tok.token == Token::RBrace {
                        brace_depth -= 1;
                    }
                }
                self.advance();
            }
            self.expect_skip(&Token::LBrace)?;
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.peek().map(|t| t.token.clone()).unwrap_or(Token::Semi),
                span: self.current_span(),
                expected: "')' or ',' after describe name".into(),
            });
        }

        let mut test_fns = Vec::new();

        // Parse it() blocks inside describe
        while !self.is_at_end() && !self.check(&Token::RBrace) {
            if self.check_ident("it") {
                match self.parse_it_block(&describe_name, start) {
                    Ok((name, func)) => {
                        test_fns.push((name, func));
                    }
                    Err(e) => {
                        if self.recovery_mode {
                            self.record_error(e);
                            self.synchronize_item();
                        } else {
                            return Err(e);
                        }
                    }
                }
            } else {
                // Skip non-it content (comments, etc.)
                self.advance();
            }
        }

        // consume "}" of describe block
        if self.check(&Token::RBrace) {
            self.advance_skip();
        }

        // consume ");" at the end of describe(...)
        if self.check(&Token::RParen) {
            self.advance_skip();
        }
        if self.check(&Token::Semi) {
            self.advance_skip();
        }

        Ok(test_fns)
    }

    /// Parse an it("test name", || { body }); block inside a describe block.
    /// Returns (function_name, Item::Function).
    fn parse_it_block(
        &mut self,
        describe_name: &str,
        _outer_start: usize,
    ) -> ParseResult<(String, Spanned<Item>)> {
        let start = self.current_span().start;

        // consume "it"
        self.advance_skip();

        // expect "("
        self.expect_skip(&Token::LParen)?;

        // expect string literal for test name
        let test_name = if let Some(tok) = self.peek() {
            if let Token::String(s) = &tok.token {
                let name = s.clone();
                self.advance_skip();
                name
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: tok.token.clone(),
                    span: tok.span.clone(),
                    expected: "string literal for it test name".into(),
                });
            }
        } else {
            return Err(ParseError::UnexpectedEof {
                span: self.current_span(),
            });
        };

        // Handle two syntax forms:
        // Form 1 (VaisDB): it("name") { ... }
        // Form 2 (closure): it("name", || { ... })
        let stmts = if self.check(&Token::RParen) {
            // Form 1: consume ")" then expect "{"
            self.advance_skip();
            self.expect_skip(&Token::LBrace)?;
            let stmts = self.parse_block_contents()?;
            self.expect_skip(&Token::RBrace)?;
            stmts
        } else if self.check(&Token::Comma) {
            // Form 2: consume "," then "||" then "{"
            self.advance_skip();
            if self.check(&Token::Pipe) {
                self.advance_skip();
                if self.check(&Token::Pipe) {
                    self.advance_skip();
                }
            }
            self.expect_skip(&Token::LBrace)?;
            let stmts = self.parse_block_contents()?;
            self.expect_skip(&Token::RBrace)?;
            self.expect_skip(&Token::RParen)?;
            stmts
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.peek().map(|t| t.token.clone()).unwrap_or(Token::Semi),
                span: self.current_span(),
                expected: "')' or ',' after it test name".into(),
            });
        };

        // optional ";"
        if self.check(&Token::Semi) {
            self.advance_skip();
        }

        // Build function name: test_{describe_snake}_{it_snake}
        let fn_name = format!(
            "test_{}_{}",
            to_snake_case(describe_name),
            to_snake_case(&test_name)
        );

        let span = Span::new(start, self.prev_span().end);

        // Add "return 0" at the end
        let mut body_stmts = stmts;
        body_stmts.push(Spanned::new(
            Stmt::Return(Some(Box::new(Spanned::new(Expr::Int(0), span)))),
            span,
        ));

        let func = Function {
            name: Spanned::new(fn_name.clone(), span),
            generics: vec![],
            params: vec![],
            ret_type: Some(Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span,
            )),
            body: FunctionBody::Block(body_stmts),
            is_pub: false,
            is_async: false,
            attributes: vec![],
            where_clause: vec![],
        };

        Ok((fn_name, Spanned::new(Item::Function(func), span)))
    }

    pub(crate) fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub(crate) fn current_span(&self) -> std::ops::Range<usize> {
        self.peek()
            .map(|t| t.span.clone())
            .unwrap_or(self.prev_span())
    }

    pub(crate) fn prev_span(&self) -> std::ops::Range<usize> {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span.clone()
        } else {
            0..0
        }
    }
}

/// Convert a string to snake_case for test function naming.
/// "name returns correct string" → "name_returns_correct_string"
/// "HybridCost" → "hybridcost"
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        } else if (c == ' ' || c == '-' || c == '_')
            && !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
        // Skip other chars (punctuation, etc.)
    }
    // Remove trailing underscore
    while result.ends_with('_') {
        result.pop();
    }
    result
}

/// Parses Vais source code into an Abstract Syntax Tree.
///
/// This is the main convenience function that performs both lexing and parsing
/// in a single step.
///
/// # Arguments
///
/// * `source` - The Vais source code to parse
///
/// # Returns
///
/// A Module containing all parsed items on success, or a ParseError on failure.
///
/// # Examples
///
/// ```
/// use vais_parser::parse;
///
/// let source = "F add(x:i64,y:i64)->i64=x+y";
/// let module = parse(source).unwrap();
/// assert_eq!(module.items.len(), 1);
/// ```
pub fn parse(source: &str) -> Result<Module, ParseError> {
    let tokens = vais_lexer::tokenize(source).map_err(|e| ParseError::UnexpectedToken {
        found: Token::Ident(format!("LexError: {}", e)),
        span: 0..0,
        expected: "valid token".into(),
    })?;

    let mut parser = Parser::new_with_source(tokens, source);
    parser.parse_module()
}

/// Parses Vais source code with compile-time cfg values for conditional compilation.
///
/// Items annotated with `#[cfg(key = "value")]` will be included only if
/// the provided cfg_values contain the matching key-value pair.
pub fn parse_with_cfg(
    source: &str,
    cfg_values: std::collections::HashMap<String, String>,
) -> Result<Module, ParseError> {
    let tokens = vais_lexer::tokenize(source).map_err(|e| ParseError::UnexpectedToken {
        found: Token::Ident(format!("LexError: {}", e)),
        span: 0..0,
        expected: "valid token".into(),
    })?;

    let mut parser = Parser::new_with_source(tokens, source);
    parser.set_cfg_values(cfg_values);
    parser.parse_module()
}

/// Parses Vais source code with error recovery enabled.
///
/// Unlike `parse()`, this function continues parsing after encountering errors,
/// inserting Error nodes into the AST. This is useful for IDE features and
/// error reporting when you want to see all errors at once.
///
/// # Arguments
///
/// * `source` - The Vais source code to parse
///
/// # Returns
///
/// A tuple of `(Module, Vec<ParseError>)` containing the parsed AST
/// (with Error nodes for problematic sections) and all collected errors.
///
/// # Examples
///
/// ```
/// use vais_parser::parse_with_recovery;
/// use vais_ast::Item;
///
/// // Source with a broken function followed by a valid struct
/// let source = "F broken(; S Valid{x:i64}";
/// let (module, errors) = parse_with_recovery(source);
/// assert!(!errors.is_empty());
/// // The module still contains the valid struct after recovery
/// assert!(module.items.iter().any(|item| {
///     matches!(&item.node, Item::Struct(s) if s.name.node == "Valid")
/// }));
/// ```
pub fn parse_with_recovery(source: &str) -> (Module, Vec<ParseError>) {
    let tokens = match vais_lexer::tokenize(source) {
        Ok(tokens) => tokens,
        Err(e) => {
            return (
                Module {
                    items: Vec::new(),
                    modules_map: None,
                },
                vec![ParseError::UnexpectedToken {
                    found: Token::Ident(format!("LexError: {}", e)),
                    span: 0..0,
                    expected: "valid token".into(),
                }],
            );
        }
    };

    let mut parser = Parser::new_with_recovery(tokens);
    parser.newline_positions = build_newline_positions(source);
    parser.source = source.to_string();
    parser.parse_module_with_recovery()
}

#[cfg(test)]
mod parser_tests;
