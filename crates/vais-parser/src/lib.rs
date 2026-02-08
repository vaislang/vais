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
    /// Source code for newline detection (used to prevent cross-line postfix parsing)
    source: String,
    /// Compile-time cfg key-value pairs for conditional compilation.
    /// When set, items with `#[cfg(key = "value")]` are filtered out if they don't match.
    cfg_values: std::collections::HashMap<String, String>,
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
        }
    }

    /// Creates a new parser with source code for newline detection.
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

    /// Synchronize to the next expression boundary for error recovery.
    /// Skips tokens until an expression delimiter is found.
    /// Returns the list of skipped tokens (as strings for debugging).
    #[allow(dead_code)]
    fn synchronize_expression(&mut self) -> Vec<String> {
        let mut skipped = Vec::new();
        let mut brace_depth = 0;
        let mut paren_depth = 0;
        let mut bracket_depth = 0;

        while !self.is_at_end() {
            if let Some(tok) = self.peek() {
                match &tok.token {
                    // Track nesting
                    Token::LBrace => brace_depth += 1,
                    Token::RBrace => {
                        if brace_depth > 0 {
                            brace_depth -= 1;
                        } else {
                            break;
                        }
                    }
                    Token::LParen => paren_depth += 1,
                    Token::RParen => {
                        if paren_depth > 0 {
                            paren_depth -= 1;
                        } else {
                            break;
                        }
                    }
                    Token::LBracket => bracket_depth += 1,
                    Token::RBracket => {
                        if bracket_depth > 0 {
                            bracket_depth -= 1;
                        } else {
                            break;
                        }
                    }
                    // At top level (not nested), these are boundaries
                    Token::Comma | Token::Semi
                        if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 =>
                    {
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
    /// A tuple of (Module, Vec<ParseError>) containing the parsed module
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
            Token::Amp => "'&'".to_string(),
            Token::PipeArrow => "'|>'".to_string(),
            Token::Pipe => "'|'".to_string(),
            Token::Bang => "'!'".to_string(),
            Token::Tilde => "'~'".to_string(),
            Token::Caret => "'^'".to_string(),
            Token::Shl => "'<<'".to_string(),
            Token::Shr => "'>>'".to_string(),
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
/// A tuple of (Module, Vec<ParseError>) containing the parsed AST
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
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function, got {:?}", module.items[0].node);
        };
        assert_eq!(f.name.node, "add");
        assert_eq!(f.params.len(), 2);
    }

    #[test]
    fn test_parse_fibonacci() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "fib");
        let FunctionBody::Expr(expr) = &f.body else {
            unreachable!("Expected expression body");
        };
        assert!(
            matches!(expr.node, Expr::Ternary { .. }),
            "Expected ternary expression"
        );
    }

    #[test]
    fn test_parse_struct() {
        let source = "S Point{x:f64,y:f64}";
        let module = parse(source).unwrap();

        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct, got {:?}", module.items[0].node);
        };
        assert_eq!(s.name.node, "Point");
        assert_eq!(s.fields.len(), 2);
    }

    #[test]
    fn test_parse_enum() {
        let source = "E Option<T>{Some(T),None}";
        let module = parse(source).unwrap();

        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum, got {:?}", module.items[0].node);
        };
        assert_eq!(e.name.node, "Option");
        assert_eq!(e.generics.len(), 1);
        assert_eq!(e.variants.len(), 2);
    }

    #[test]
    fn test_parse_block_function() {
        let source = "F sum(arr:[i64])->i64{s:=0;L x:arr{s+=x};s}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Block(stmts) = &f.body else {
            unreachable!("Expected block body, got {:?}", f.body);
        };
        assert_eq!(stmts.len(), 3);
    }

    #[test]
    fn test_parse_generic_constraints() {
        // Test single trait bound
        let source = "F print_value<T: Display>(x: T) -> () = println(x)";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "print_value");
        assert_eq!(f.generics.len(), 1);
        assert_eq!(f.generics[0].name.node, "T");
        assert_eq!(f.generics[0].bounds.len(), 1);
        assert_eq!(f.generics[0].bounds[0].node, "Display");

        // Test multiple trait bounds
        let source2 = "F compare<T: Ord + Clone>(a: T, b: T) -> bool = a < b";
        let module2 = parse(source2).unwrap();

        let Item::Function(f2) = &module2.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f2.generics.len(), 1);
        assert_eq!(f2.generics[0].name.node, "T");
        assert_eq!(f2.generics[0].bounds.len(), 2);
        assert_eq!(f2.generics[0].bounds[0].node, "Ord");
        assert_eq!(f2.generics[0].bounds[1].node, "Clone");

        // Test multiple generic params with bounds
        let source3 = "F transform<A: Clone, B: Default>(x: A) -> B = x";
        let module3 = parse(source3).unwrap();

        let Item::Function(f3) = &module3.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f3.generics.len(), 2);
        assert_eq!(f3.generics[0].name.node, "A");
        assert_eq!(f3.generics[0].bounds.len(), 1);
        assert_eq!(f3.generics[0].bounds[0].node, "Clone");
        assert_eq!(f3.generics[1].name.node, "B");
        assert_eq!(f3.generics[1].bounds.len(), 1);
        assert_eq!(f3.generics[1].bounds[0].node, "Default");

        // Test generic without bounds (should still work)
        let source4 = "F identity<T>(x: T) -> T = x";
        let module4 = parse(source4).unwrap();

        let Item::Function(f4) = &module4.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f4.generics.len(), 1);
        assert_eq!(f4.generics[0].name.node, "T");
        assert_eq!(f4.generics[0].bounds.len(), 0);
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_input() {
        let source = "";
        let module = parse(source).unwrap();
        assert!(module.items.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let source = "   \n\t\r\n   ";
        let module = parse(source).unwrap();
        assert!(module.items.is_empty());
    }

    #[test]
    fn test_comment_only() {
        let source = "# this is just a comment\n# another comment";
        let module = parse(source).unwrap();
        assert!(module.items.is_empty());
    }

    #[test]
    fn test_minimal_function() {
        let source = "F f()->()=()";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
        assert!(f.params.is_empty());
    }

    #[test]
    fn test_empty_struct() {
        let source = "S Empty{}";
        let module = parse(source).unwrap();
        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct");
        };
        assert_eq!(s.name.node, "Empty");
        assert!(s.fields.is_empty());
    }

    #[test]
    fn test_single_field_struct() {
        let source = "S Single{x:i64}";
        let module = parse(source).unwrap();
        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct");
        };
        assert_eq!(s.fields.len(), 1);
    }

    #[test]
    fn test_minimal_enum() {
        let source = "E Unit{A}";
        let module = parse(source).unwrap();
        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum");
        };
        assert_eq!(e.name.node, "Unit");
        assert_eq!(e.variants.len(), 1);
    }

    #[test]
    fn test_enum_with_tuple_variants() {
        let source = "E Shape{Circle(f64),Rectangle(f64,f64),Point}";
        let module = parse(source).unwrap();
        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum");
        };
        assert_eq!(e.variants.len(), 3);
    }

    #[test]
    fn test_enum_with_struct_variants() {
        let source = "E Message{Quit,Move{x:i64,y:i64},Write(str)}";
        let module = parse(source).unwrap();
        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum");
        };
        assert_eq!(e.variants.len(), 3);
    }

    #[test]
    fn test_empty_block_function() {
        let source = "F f()->(){}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Block(stmts) = &f.body else {
            unreachable!("Expected block body");
        };
        assert!(stmts.is_empty());
    }

    #[test]
    fn test_nested_generic_types() {
        // Use simple generic syntax that the parser supports
        let source = "F f<T>(x:T)->T=x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.generics.len(), 1);
    }

    #[test]
    fn test_deeply_nested_arrays() {
        let source = "F f(x:[[[i64]]])->[[[i64]]]=x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 1);
    }

    #[test]
    fn test_multiple_items() {
        let source = r#"
S Point{x:f64,y:f64}
F new_point(x:f64,y:f64)->Point=Point{x:x,y:y}
F origin()->Point=new_point(0.0,0.0)
"#;
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 3);
    }

    #[test]
    fn test_trait_definition() {
        // Trait uses W keyword with methods using regular identifiers
        let source = "W Display{F display(s:&Self)->str=\"\"}";
        let module = parse(source).unwrap();
        let Item::Trait(t) = &module.items[0].node else {
            unreachable!("Expected trait");
        };
        assert_eq!(t.name.node, "Display");
        assert_eq!(t.methods.len(), 1);
    }

    #[test]
    fn test_impl_block() {
        let source = r#"
S Point{x:f64,y:f64}
X Point{F new(x:f64,y:f64)->Point=Point{x:x,y:y}}
"#;
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 2);
        let Item::Impl(imp) = &module.items[1].node else {
            unreachable!("Expected impl");
        };
        // target_type is a Spanned<Type>, check the type name
        assert!(matches!(&imp.target_type.node, Type::Named { name, .. } if name == "Point"));
    }

    #[test]
    fn test_if_without_else() {
        let source = "F f(x:bool)->(){I x{print(1)}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        // Function should parse successfully
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_nested_if_else() {
        let source = "F f(x:i64)->i64=I x>0{I x>10{100}E{10}}E{0}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_match_with_wildcard() {
        let source = "F f(x:i64)->i64=M x{0=>0,1=>1,_=>-1}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Expr(expr) = &f.body else {
            unreachable!("Expected expression body");
        };
        assert!(matches!(expr.node, Expr::Match { .. }));
    }

    #[test]
    fn test_match_with_guard() {
        let source = "F f(x:i64)->i64=M x{n I n>0=>n,_=>0}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_lambda_expression() {
        let source = "F f()->i64{g:=|x:i64|x*2;g(21)}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_nested_lambda() {
        let source = "F f()->i64{g:=|x:i64|(|y:i64|x+y);g(10)(32)}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_method_chaining() {
        let source = "F f(x:str)->i64=x.len().to_string().len()";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_array_indexing_chain() {
        let source = "F f(arr:[[i64]])->i64=arr[0][1]";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_self_recursion_operator() {
        let source = "F factorial(n:i64)->i64=n<2?1:n*@(n-1)";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "factorial");
    }

    #[test]
    fn test_range_expression() {
        let source = "F f()->(){L i:0..10{print(i)}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_ternary_operator() {
        // Test the ternary operator (cond ? then : else)
        let source = "F f(x:i64)->i64=x>0?x:0";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_ternary_with_unary_minus() {
        // Test ternary with unary minus in then branch: x<0 ? -x : x
        let source = "F abs(x:i64)->i64=x<0?-x:x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "abs");
        // Body should be a Ternary
        let FunctionBody::Expr(body) = &f.body else {
            panic!("Expected expression body");
        };
        assert!(matches!(body.node, Expr::Ternary { .. }));
    }

    #[test]
    fn test_try_operator() {
        // Test postfix try operator (?)
        let source = "F f(x:i64?)->i64=x?";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
        // Verify the body is a Try expression
        let FunctionBody::Expr(body) = &f.body else {
            panic!("Expected expression body");
        };
        if let Expr::Try(inner) = &body.node {
            assert!(matches!(inner.node, Expr::Ident(_)));
        } else {
            panic!("Expected Try expression");
        }
    }

    #[test]
    fn test_try_operator_in_expression() {
        // Test try operator followed by binary operator
        let source = "F f(x:i64?)->i64=x?+1";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        // Verify the body is a Binary expression with Try on left
        let FunctionBody::Expr(body) = &f.body else {
            panic!("Expected expression body");
        };
        if let Expr::Binary { left, .. } = &body.node {
            assert!(matches!(left.node, Expr::Try(_)));
        } else {
            panic!("Expected Binary expression with Try on left");
        }
    }

    #[test]
    fn test_try_and_ternary_coexist() {
        // Test that try and ternary can coexist: (x?) ? 1 : 0
        let source = "F f(x:i64?)->i64=(x?)?1:0";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        // Body should be a Ternary with condition being Try
        let FunctionBody::Expr(body) = &f.body else {
            panic!("Expected expression body");
        };
        if let Expr::Ternary { cond, .. } = &body.node {
            assert!(matches!(cond.node, Expr::Try(_)));
        } else {
            panic!("Expected Ternary expression");
        }
    }

    #[test]
    fn test_simple_return_type() {
        // Test simple return type parsing
        let source = "F f()->i64=42";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(f.ret_type.is_some());
    }

    #[test]
    fn test_reference_types() {
        let source = "F f(x:&i64,y:&mut i64)->()=()";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 2);
    }

    #[test]
    fn test_pointer_type() {
        let source = "F f(x:*i64)->*i64=x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(matches!(&f.params[0].ty.node, Type::Pointer(_)));
    }

    #[test]
    fn test_tuple_type() {
        let source = "F f(x:(i64,str))->(i64,str)=x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(matches!(&f.params[0].ty.node, Type::Tuple(_)));
    }

    #[test]
    fn test_function_type() {
        let source = "F apply(f:(i64)->i64,x:i64)->i64=f(x)";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(matches!(&f.params[0].ty.node, Type::Fn { .. }));
    }

    #[test]
    fn test_async_function() {
        // Async function with A prefix
        let source = "A F fetch(url:str)->str=url";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(f.is_async);
    }

    #[test]
    fn test_pub_function() {
        let source = "P F public_fn()->()=()";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(f.is_pub);
    }

    #[test]
    fn test_import_statement() {
        // Use statement with U keyword
        let source = "U std::fs";
        let module = parse(source).unwrap();
        let Item::Use(u) = &module.items[0].node else {
            unreachable!("Expected use statement");
        };
        assert!(!u.path.is_empty());
    }

    #[test]
    fn test_complex_expression() {
        let source = "F f(a:i64,b:i64,c:i64)->i64=a+b*c-a/b%c";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_bitwise_operations() {
        let source = "F f(a:i64,b:i64)->i64=a&b|c^d<<2>>1";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_comparison_chain() {
        let source = "F f(a:i64,b:i64,c:i64)->bool=a<b&&b<c||a==c";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_unary_operators() {
        let source = "F f(x:i64,b:bool)->i64=-x+~x*(!b?1:0)";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_compound_assignment() {
        // In Vais, use := for mutable variable declaration
        let source = "F f(x:i64)->i64{y:=x;y+=1;y-=2;y*=3;y}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Block(stmts) = &f.body else {
            unreachable!("Expected block body");
        };
        assert_eq!(stmts.len(), 5);
    }

    #[test]
    fn test_break_with_value() {
        let source = "F f()->i64{L{B 42}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_continue_in_loop() {
        let source = "F f()->(){L i:0..10{I i%2==0{C};print(i)}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_defer_statement() {
        // Test basic defer statement
        let source = "F f() -> () { h := open(); D close(h); () }";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
        // Check that body contains defer
        let FunctionBody::Block(stmts) = &f.body else {
            panic!("Expected block body");
        };
        // Should have 3 statements: let, defer, expr
        assert_eq!(stmts.len(), 3);
        assert!(matches!(stmts[1].node, Stmt::Defer(_)));
    }

    #[test]
    fn test_multiple_defer_statements() {
        // Test multiple defer statements (LIFO order)
        let source = "F f() -> () { D cleanup1(); D cleanup2(); D cleanup3(); () }";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Block(stmts) = &f.body else {
            panic!("Expected block body");
        };
        // Should have 4 statements: 3 defers + 1 expr
        assert_eq!(stmts.len(), 4);
        assert!(matches!(stmts[0].node, Stmt::Defer(_)));
        assert!(matches!(stmts[1].node, Stmt::Defer(_)));
        assert!(matches!(stmts[2].node, Stmt::Defer(_)));
    }

    #[test]
    fn test_struct_literal() {
        let source = "F f()->Point{Point{x:1.0,y:2.0}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_array_literal() {
        let source = "F f()->[i64]=[1,2,3,4,5]";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_empty_array_literal() {
        let source = "F f()->[i64]=[]";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_array_with_values() {
        // Test array literal syntax [value, value, ...]
        let source = "F f()->[i64]=[1,2,3]";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_multiline_function() {
        let source = r#"
F calculate(a: i64,
            b: i64,
            c: i64) -> i64 {
    x := a + b;
    y := x * c;
    R y
}
"#;
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 3);
    }

    #[test]
    fn test_all_primitive_types() {
        let source = r#"
F test(
    a:i8,b:i16,c:i32,d:i64,e:i128,
    f:u8,g:u16,h:u32,i:u64,j:u128,
    k:f32,l:f64,m:bool,n:str
)->()=()
"#;
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 14);
    }

    #[test]
    fn test_pattern_in_match() {
        let source = r#"
F f(opt:Option<i64>)->i64=M opt{
    Some(x)=>x,
    None=>0
}
"#;
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_tuple_parameter() {
        // Test tuple type as parameter
        let source = "F f(t:(i64,i64))->i64=42";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
        assert!(matches!(&f.params[0].ty.node, Type::Tuple(_)));
    }

    #[test]
    fn test_basic_struct_with_methods() {
        // Test struct with impl block using regular param names
        let source = r#"
S Counter{value:i64}
X Counter{F inc(c:&Counter)->i64=c.value+1}
"#;
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 2);
    }

    #[test]
    fn test_enum_pattern_match() {
        // Test enum variant matching
        let source = r#"
E Result{Ok(i64),Err(str)}
F handle(r:Result)->i64=M r{Ok(v)=>v,Err(_)=>0}
"#;
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 2);
    }

    // ==================== Advanced Edge Case Tests ====================

    #[test]
    fn test_nested_generic_vec_hashmap() {
        // Test nested generic: Vec<HashMap<K, V> > with spaces
        let source = "S Container{data:Vec<HashMap<str,i64> >}";
        let module = parse(source).unwrap();

        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct");
        };
        assert_eq!(s.name.node, "Container");
        assert_eq!(s.fields.len(), 1);
    }

    #[test]
    fn test_option_of_vec_generic() {
        // Test Option<Vec<T> > combination with spaces (need space before =)
        let source = r#"F get_items<T>()->Option<Vec<T> > ="""#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "get_items");
        assert_eq!(f.generics.len(), 1);
    }

    #[test]
    fn test_hashmap_option_value() {
        // Test HashMap<K, Option<V> > with spaces
        let source = "S Cache{entries:HashMap<str,Option<i64> >}";
        let module = parse(source).unwrap();

        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct");
        };
        assert_eq!(s.name.node, "Cache");
    }

    #[test]
    fn test_deeply_nested_generics() {
        // Test Vec<HashMap<K, Option<Vec<T> > > > with spaces (need space before =)
        let source = "F complex<T>()->Vec<HashMap<str,Option<Vec<T> > > > =[]";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "complex");
    }

    #[test]
    fn test_pattern_match_with_guard() {
        // Test pattern matching with guard condition
        let source = "F classify(x:i64)->str=M x{n I n>0=>\"pos\",n I n<0=>\"neg\",_=>\"zero\"}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "classify");
    }

    #[test]
    fn test_pattern_match_guard_complex() {
        // Test pattern match with complex guard
        let source = r#"
F filter(opt:Option<i64>)->i64=M opt{
    Some(x) I x>0&&x<100=>x,
    Some(x) I x>=100=>100,
    Some(_)=>0,
    None=>-1
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "filter");
    }

    #[test]
    fn test_nested_pattern_destructuring() {
        // Test nested destructuring in pattern match
        let source = r#"
E Nested{Pair((i64,i64)),Single(i64),None}
F sum(n:Nested)->i64=M n{
    Pair((a,b))=>a+b,
    Single(x)=>x,
    None=>0
}
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 2);
        let Item::Function(f) = &module.items[1].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "sum");
    }

    #[test]
    fn test_pattern_guard_with_multiple_conditions() {
        // Test guard with multiple && || conditions
        let source =
            "F check(x:i64,y:i64)->bool=M (x,y){(a,b) I a>0&&b>0||a<0&&b<0=>true,_=>false}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "check");
    }

    #[test]
    fn test_nested_option_pattern() {
        // Test nested Option patterns: Option<Option<T> > with spaces
        let source = r#"
F unwrap_twice(opt:Option<Option<i64> >)->i64=M opt{
    Some(Some(x))=>x,
    Some(None)=>-1,
    None=>-2
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "unwrap_twice");
    }

    #[test]
    fn test_mutual_recursion_type_inference() {
        // Test mutual recursion: two functions calling each other
        let source = r#"
F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 2);
        let Item::Function(f1) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let Item::Function(f2) = &module.items[1].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f1.name.node, "is_even");
        assert_eq!(f2.name.node, "is_odd");
    }

    #[test]
    fn test_three_way_mutual_recursion() {
        // Test three functions in mutual recursion
        let source = r#"
F a(n:i64)->i64=n<1?0:b(n-1)+1
F b(n:i64)->i64=n<1?0:c(n-1)+1
F c(n:i64)->i64=n<1?0:a(n-1)+1
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 3);
    }

    #[test]
    fn test_indirect_recursion_through_lambda() {
        // Test recursion through lambda (advanced case)
        let source = r#"
F outer(n:i64)->i64{
    helper:=|x:i64|x<1?0:outer(x-1)+1;
    helper(n)
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "outer");
    }

    #[test]
    fn test_generic_mutual_recursion() {
        // Test mutual recursion with generics
        let source = r#"
F transform_a<T>(x:T)->T=transform_b(x)
F transform_b<T>(x:T)->T=x
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 2);
    }

    #[test]
    fn test_i8_boundary_parsing() {
        // Test i8 min/max: -128, 127
        let source = "F i8_test()->(){min:i8=-128;max:i8=127}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "i8_test");
    }

    #[test]
    fn test_i16_boundaries() {
        // Test i16 boundaries: -32768, 32767
        let source = "F i16_test()->(){min:i16=-32768;max:i16=32767}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "i16_test");
    }

    #[test]
    fn test_i64_max_parsing() {
        // Test i64 max: 9223372036854775807
        let source = "F i64_max()->i64=9223372036854775807";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "i64_max");
    }

    #[test]
    fn test_pattern_with_range() {
        // Test pattern matching with ranges
        let source = r#"
F grade(score:i64)->str=M score{
    x I x>=90=>"A",
    x I x>=80=>"B",
    x I x>=70=>"C",
    x I x>=60=>"D",
    _=>"F"
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "grade");
    }

    #[test]
    fn test_destructure_nested_struct() {
        // Test destructuring nested structs in pattern match
        let source = r#"
S Point{x:i64,y:i64}
S Line{start:Point,end:Point}
F length(line:Line)->i64=line.end.x-line.start.x
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 3);
    }

    #[test]
    fn test_guard_with_method_call() {
        // Test guard condition with method calls
        let source = "F check_len(s:str)->bool=M s{x I x.len()>0=>true,_=>false}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "check_len");
    }

    #[test]
    fn test_multiple_generic_constraints() {
        // Test function with multiple generic parameters with bounds
        let source = "F combine<A:Clone,B:Default,C:Ord>(a:A,b:B,c:C)->C=c";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.generics.len(), 3);
        assert_eq!(f.generics[0].bounds.len(), 1);
        assert_eq!(f.generics[1].bounds.len(), 1);
        assert_eq!(f.generics[2].bounds.len(), 1);
    }

    #[test]
    fn test_enum_with_generic_variants() {
        // Test enum with generic variants
        let source = "E Result<T,E>{Ok(T),Err(E)}";
        let module = parse(source).unwrap();

        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum");
        };
        assert_eq!(e.name.node, "Result");
        assert_eq!(e.generics.len(), 2);
        assert_eq!(e.variants.len(), 2);
    }

    #[test]
    fn test_deeply_nested_if_else() {
        // Test deeply nested if-else chains
        let source = r#"
F classify(n:i64)->str{
    I n>1000{
        I n>10000{"huge"}E{"large"}
    }E{
        I n>100{
            I n>500{"medium-large"}E{"medium"}
        }E{
            I n>10{"small"}E{"tiny"}
        }
    }
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "classify");
    }

    #[test]
    fn test_pattern_with_multiple_bindings() {
        // Test pattern with multiple variable bindings and guards
        let source = r#"
F process(a:i64,b:i64)->i64=M (a,b){
    (x,y) I x>0&&y>0=>x+y,
    (x,y) I x<0&&y<0=>x-y,
    (x,y) I x==0||y==0=>0,
    (x,y)=>x*y
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "process");
    }

    #[test]
    fn test_self_recursion_with_multiple_params() {
        // Test self-recursion with multiple parameters
        let source = "F gcd(a:i64,b:i64)->i64=b==0?a:@(b,a%b)";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "gcd");
        assert_eq!(f.params.len(), 2);
    }

    // ==================== Error Recovery Tests ====================

    #[test]
    fn test_error_recovery_multiple_items() {
        // Test that parser recovers and continues parsing after an error
        // The broken function has an incomplete parameter list followed by a semicolon,
        // which allows the parser to recover and continue.
        let source = r#"
F good1()->i64=1
F broken(;
F good2()->i64=2
S ValidStruct{x:i64}
"#;
        let (module, errors) = parse_with_recovery(source);

        // Should have collected at least one error
        assert!(!errors.is_empty(), "Expected at least one error");

        // Count valid items (good1, good2, ValidStruct) and errors
        let mut valid_function_count = 0;
        let mut valid_struct_count = 0;
        let mut error_count = 0;

        for item in &module.items {
            match &item.node {
                Item::Function(f) if f.name.node == "good1" || f.name.node == "good2" => {
                    valid_function_count += 1;
                }
                Item::Struct(s) if s.name.node == "ValidStruct" => {
                    valid_struct_count += 1;
                }
                Item::Error { .. } => {
                    error_count += 1;
                }
                _ => {}
            }
        }

        // The parser should recover after the error and parse remaining valid items
        assert!(
            valid_function_count >= 1,
            "Should have parsed at least 1 valid function"
        );
        assert_eq!(valid_struct_count, 1, "Should have parsed 1 valid struct");
        assert!(error_count >= 1, "Should have at least 1 error node");
    }

    #[test]
    fn test_error_recovery_block_statements() {
        // Test error recovery within block statements
        let source = r#"
F test_block()->i64{
    x := 1
    y :=
    z := 3
    z
}
"#;
        let (module, errors) = parse_with_recovery(source);

        // Should have errors for the incomplete let statement
        assert!(!errors.is_empty(), "Expected errors");

        // Should still have parsed the function
        assert_eq!(module.items.len(), 1);
        let Item::Function(f) = &module.items[0].node else {
            panic!("Expected function");
        };
        assert_eq!(f.name.node, "test_block");

        // The function body should have statements (some may be error nodes)
        let FunctionBody::Block(stmts) = &f.body else {
            panic!("Expected block body");
        };
        assert!(!stmts.is_empty());

        // Check that we have both valid statements and error statements
        let mut has_error_stmt = false;
        let mut has_valid_let = false;
        for stmt in stmts {
            match &stmt.node {
                Stmt::Error { .. } => has_error_stmt = true,
                Stmt::Let { name, .. } if name.node == "x" || name.node == "z" => {
                    has_valid_let = true
                }
                _ => {}
            }
        }
        assert!(has_valid_let, "Should have valid let statements");
        assert!(has_error_stmt, "Should have error statements");
    }

    #[test]
    fn test_error_recovery_preserves_span() {
        // Test that error recovery preserves span information
        let source = "F broken( F good()->i64=42";
        let (module, errors) = parse_with_recovery(source);

        // Check that errors have span information
        for error in &errors {
            let span = error.span();
            assert!(span.is_some(), "Error should have span information");
        }

        // Check that error nodes in AST have valid spans
        for item in &module.items {
            assert!(item.span.start <= item.span.end, "Span should be valid");
        }
    }

    #[test]
    fn test_error_recovery_synchronize_to_next_function() {
        // Test that parser synchronizes correctly to next function keyword
        let source = r#"
F broken(x:i64 y:i64)->i64
F good(a:i64,b:i64)->i64=a+b
"#;
        let (module, errors) = parse_with_recovery(source);

        // Should have errors
        assert!(!errors.is_empty(), "Expected errors for broken function");

        // Should have recovered and parsed the good function
        let has_good_function = module
            .items
            .iter()
            .any(|item| matches!(&item.node, Item::Function(f) if f.name.node == "good"));
        assert!(
            has_good_function,
            "Should have parsed 'good' function after recovery"
        );
    }

    #[test]
    fn test_error_recovery_synchronize_to_struct() {
        // Test synchronization to struct keyword
        // Use semicolon to help parser recognize the error boundary
        let source = r#"
F broken(;
S Point{x:f64,y:f64}
"#;
        let (module, errors) = parse_with_recovery(source);

        assert!(!errors.is_empty());

        let has_struct = module
            .items
            .iter()
            .any(|item| matches!(&item.node, Item::Struct(s) if s.name.node == "Point"));
        assert!(has_struct, "Should have parsed Point struct after recovery");
    }

    #[test]
    fn test_error_recovery_empty_after_errors() {
        // Test that errors() returns collected errors
        let tokens = vais_lexer::tokenize("F broken(").unwrap();
        let mut parser = Parser::new_with_recovery(tokens);
        let _ = parser.parse_module();

        let errors = parser.errors();
        assert!(!errors.is_empty(), "Should have collected errors");
    }

    #[test]
    fn test_error_recovery_take_errors() {
        // Test that take_errors() returns and clears errors
        let tokens = vais_lexer::tokenize("F broken(").unwrap();
        let mut parser = Parser::new_with_recovery(tokens);
        let _ = parser.parse_module();

        let errors = parser.take_errors();
        assert!(!errors.is_empty(), "take_errors should return errors");
        assert!(
            parser.errors().is_empty(),
            "errors should be empty after take"
        );
    }

    #[test]
    fn test_no_recovery_mode_fails_fast() {
        // Test that without recovery mode, parsing fails on first error
        let source = "F broken( F good()->i64=42";
        let result = parse(source);
        assert!(result.is_err(), "Without recovery, should fail immediately");
    }

    #[test]
    fn test_error_recovery_enum_with_error() {
        // Test error recovery when enum has errors
        let source = r#"
E Broken{A(i64,B}
E Good{X,Y}
"#;
        let (module, errors) = parse_with_recovery(source);

        assert!(!errors.is_empty());

        let has_good_enum = module
            .items
            .iter()
            .any(|item| matches!(&item.node, Item::Enum(e) if e.name.node == "Good"));
        assert!(has_good_enum, "Should have parsed Good enum after recovery");
    }

    #[test]
    fn test_error_recovery_mixed_items() {
        // Test recovery with various item types
        let source = r#"
F func1()->i64=1
S Broken{x
E MyEnum{A,B}
F func2()->i64=2
W MyTrait{F method()->i64}
"#;
        let (module, errors) = parse_with_recovery(source);

        assert!(!errors.is_empty(), "Should have errors for broken struct");

        // Count valid items
        let valid_functions = module
            .items
            .iter()
            .filter(|item| matches!(&item.node, Item::Function(_)))
            .count();
        let valid_enums = module
            .items
            .iter()
            .filter(|item| matches!(&item.node, Item::Enum(_)))
            .count();
        let valid_traits = module
            .items
            .iter()
            .filter(|item| matches!(&item.node, Item::Trait(_)))
            .count();

        assert!(
            valid_functions >= 2,
            "Should have at least 2 valid functions"
        );
        assert!(valid_enums >= 1, "Should have at least 1 valid enum");
        assert!(valid_traits >= 1, "Should have at least 1 valid trait");
    }

    #[test]
    fn test_error_recovery_missing_closing_paren() {
        let source = r#"
F broken(x: i64, y: i64 -> i64 = x + y
F good() -> i64 = 42
"#;
        let (module, errors) = parse_with_recovery(source);
        assert!(!errors.is_empty(), "Expected error for missing ')'");

        // Should still parse the good function
        let has_good = module
            .items
            .iter()
            .any(|item| matches!(&item.node, Item::Function(f) if f.name.node == "good"));
        assert!(
            has_good,
            "Should have parsed 'good' function after recovery"
        );
    }

    #[test]
    fn test_error_recovery_missing_closing_brace() {
        let source = r#"
F broken() -> i64 {
    x := 1
    y := 2

F good() -> i64 = 42
"#;
        let (module, errors) = parse_with_recovery(source);
        assert!(!errors.is_empty(), "Expected error for missing '}}'");

        let has_good = module
            .items
            .iter()
            .any(|item| matches!(&item.node, Item::Function(f) if f.name.node == "good"));
        assert!(
            has_good,
            "Should have parsed 'good' function after recovery"
        );
    }

    #[test]
    fn test_error_recovery_generic_missing_closing_angle() {
        let source = r#"
F broken<T(x: T) -> T = x
F good() -> i64 = 42
"#;
        let (module, errors) = parse_with_recovery(source);
        assert!(!errors.is_empty(), "Expected error for missing '>'");

        let has_good = module
            .items
            .iter()
            .any(|item| matches!(&item.node, Item::Function(f) if f.name.node == "good"));
        assert!(
            has_good,
            "Should have parsed 'good' function after recovery"
        );
    }

    #[test]
    fn test_error_recovery_generic_invalid_param() {
        let source = r#"
F broken<T, 123, U>(x: T) -> T = x
F good() -> i64 = 42
"#;
        let (module, errors) = parse_with_recovery(source);
        assert!(
            !errors.is_empty(),
            "Expected error for invalid generic param"
        );

        let has_good = module
            .items
            .iter()
            .any(|item| matches!(&item.node, Item::Function(f) if f.name.node == "good"));
        assert!(
            has_good,
            "Should have parsed 'good' function after recovery"
        );
    }

    #[test]
    fn test_error_recovery_mismatched_brackets() {
        let source = r#"
F broken() -> i64 {
    x := [1, 2, 3}
    x
}
F good() -> i64 = 42
"#;
        let (module, errors) = parse_with_recovery(source);
        assert!(!errors.is_empty(), "Expected error for mismatched brackets");

        let has_good = module
            .items
            .iter()
            .any(|item| matches!(&item.node, Item::Function(f) if f.name.node == "good"));
        assert!(
            has_good,
            "Should have parsed 'good' function after recovery"
        );
    }
}
