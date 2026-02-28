//! Vais Parser
//!
//! Recursive descent parser for AI-optimized syntax.

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
    /// Compile-time cfg key-value pairs for conditional compilation.
    /// When set, items with `#[cfg(key = "value")]` are filtered out if they don't match.
    cfg_values: std::collections::HashMap<String, String>,
    /// Pending `>` token from a split `>>` (Token::Shr) in nested generics.
    /// When `Vec<Vec<i64>>` is tokenized, the `>>` becomes Token::Shr.
    /// We split it into two `>` tokens: the first closes the inner generic,
    /// and this flag records that a second `>` is still pending for the outer generic.
    pending_gt: bool,
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
            cfg_values: std::collections::HashMap::new(),
            pending_gt: false,
        }
    }

    /// Creates a new parser with source code for newline detection.
    ///
    /// Note: This clones the source string to avoid adding a lifetime parameter to Parser,
    /// which would require changes across ~94 usage sites. The allocation is acceptable
    /// since it happens once per file parse (not in a hot loop), and the source is only
    /// used for newline detection in postfix operator parsing.
    pub fn new_with_source(tokens: Vec<SpannedToken>, source: &str) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            recovery_mode: false,
            allow_struct_literal: true,
            depth: 0,
            source: source.to_string(),
            cfg_values: std::collections::HashMap::new(),
            pending_gt: false,
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
            cfg_values: std::collections::HashMap::new(),
            pending_gt: false,
        }
    }

    /// Set cfg key-value pairs for conditional compilation filtering.
    /// Items annotated with `#[cfg(key = "value")]` will be included only
    /// if the cfg values match.
    pub fn set_cfg_values(&mut self, values: std::collections::HashMap<String, String>) {
        self.cfg_values = values;
    }

    /// Check if there is a newline between two byte positions in the source.
    fn has_newline_between(&self, start: usize, end: usize) -> bool {
        if self.source.is_empty() {
            return false;
        }
        let s = start.min(self.source.len());
        let e = end.min(self.source.len());
        self.source[s..e].contains('\n')
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
                    Token::Function
                    | Token::Struct
                    | Token::Enum
                    | Token::Union
                    | Token::Use
                    | Token::Trait
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
                    Token::Function
                    | Token::Struct
                    | Token::Enum
                    | Token::Union
                    | Token::TypeKeyword
                    | Token::Use
                    | Token::Trait
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

        while !self.is_at_end() {
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
        self.pending_gt = false;
    }

    /// Check if the current "token" is `>`, accounting for a pending `>`
    /// left over from splitting a `>>` (Token::Shr) in nested generic contexts.
    /// Also returns true for `>>` (Token::Shr) because `>>` will be split into
    /// two `>` tokens when consumed via `consume_gt()`.
    pub(crate) fn check_gt(&self) -> bool {
        if self.pending_gt {
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
    /// 3. A `Token::Shr` (`>>`) which we split: consume it and set `pending_gt = true`
    ///    so the next `consume_gt()` call returns the second `>`.
    ///
    /// Returns a synthetic `>` SpannedToken in the pending-gt and Shr cases.
    pub(crate) fn consume_gt(&mut self) -> ParseResult<SpannedToken> {
        if self.pending_gt {
            self.pending_gt = false;
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
            self.pending_gt = true;
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
        match token {
            // Delimiters
            Token::LParen => "'('".to_string(),
            Token::RParen => "')'".to_string(),
            Token::LBrace => "'{'".to_string(),
            Token::RBrace => "'}'".to_string(),
            Token::LBracket => "'['".to_string(),
            Token::RBracket => "']'".to_string(),
            Token::Comma => "','".to_string(),
            Token::Colon => "':'".to_string(),
            Token::ColonColon => "'::'".to_string(),
            Token::Semi => "';'".to_string(),
            Token::Dot => "'.'".to_string(),
            Token::DotDot => "'..'".to_string(),
            Token::DotDotEq => "'..='".to_string(),
            Token::Arrow => "'->'".to_string(),
            Token::FatArrow => "'=>'".to_string(),
            // Operators
            Token::Eq => "'='".to_string(),
            Token::ColonEq => "':=' (let binding)".to_string(),
            Token::EqEq => "'=='".to_string(),
            Token::Neq => "'!='".to_string(),
            Token::Lt => "'<'".to_string(),
            Token::Lte => "'<='".to_string(),
            Token::Gt => "'>'".to_string(),
            Token::Gte => "'>='".to_string(),
            Token::Plus => "'+'".to_string(),
            Token::PlusEq => "'+='".to_string(),
            Token::Minus => "'-'".to_string(),
            Token::MinusEq => "'-='".to_string(),
            Token::Star => "'*'".to_string(),
            Token::StarEq => "'*='".to_string(),
            Token::Slash => "'/'".to_string(),
            Token::SlashEq => "'/='".to_string(),
            Token::Percent => "'%'".to_string(),
            Token::PercentEq => "'%='".to_string(),
            Token::Amp => "'&'".to_string(),
            Token::AmpEq => "'&='".to_string(),
            Token::PipeArrow => "'|>'".to_string(),
            Token::Pipe => "'|'".to_string(),
            Token::PipeEq => "'|='".to_string(),
            Token::Bang => "'!'".to_string(),
            Token::Tilde => "'~'".to_string(),
            Token::Caret => "'^'".to_string(),
            Token::CaretEq => "'^='".to_string(),
            Token::Shl => "'<<'".to_string(),
            Token::ShlEq => "'<<='".to_string(),
            Token::Shr => "'>>'".to_string(),
            Token::ShrEq => "'>>='".to_string(),
            Token::Question => "'?'".to_string(),
            Token::At => "'@' (self-recursion)".to_string(),
            Token::HashBracket => "'#[' (attribute)".to_string(),
            // Keywords
            Token::Function => "function keyword 'F'".to_string(),
            Token::Struct => "struct keyword 'S'".to_string(),
            Token::Enum => "enum keyword 'E'".to_string(),
            Token::Trait => "trait keyword 'W'".to_string(),
            Token::Impl => "impl keyword 'X'".to_string(),
            Token::If => "if keyword 'I'".to_string(),
            Token::Loop => "loop keyword 'L'".to_string(),
            Token::Match => "match keyword 'M'".to_string(),
            Token::Return => "return keyword 'R'".to_string(),
            Token::Break => "break keyword 'B'".to_string(),
            Token::Continue => "continue keyword 'C'".to_string(),
            Token::Use => "use keyword 'U'".to_string(),
            Token::Pub => "pub keyword 'P'".to_string(),
            Token::Async => "async keyword 'A'".to_string(),
            Token::Await => "await keyword 'Y'".to_string(),
            Token::Spawn => "'spawn' keyword".to_string(),
            Token::Yield => "'yield' keyword".to_string(),
            Token::True => "'true'".to_string(),
            Token::False => "'false'".to_string(),
            Token::Defer => "defer keyword 'D'".to_string(),
            Token::Union => "union keyword 'O'".to_string(),
            Token::Comptime => "'comptime' keyword".to_string(),
            Token::Const => "'const' keyword".to_string(),
            Token::Mut => "'mut' keyword".to_string(),
            Token::SelfLower => "'self'".to_string(),
            Token::SelfUpper => "'Self'".to_string(),
            Token::TypeKeyword => "type keyword 'T'".to_string(),
            // Literals
            Token::Ident(name) => format!("identifier '{}'", name),
            Token::Int(_) => "integer literal".to_string(),
            Token::Float(_) => "float literal".to_string(),
            Token::String(_) => "string literal".to_string(),
            // Default for any other token
            _ => format!("{:?}", token),
        }
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
    parser.source = source.to_string();
    parser.parse_module_with_recovery()
}

#[cfg(test)]
mod parser_tests;
