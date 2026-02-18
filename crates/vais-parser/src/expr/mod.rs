//! Expression parsing for Vais language.
//!
//! Implements recursive descent parsing for all expression types including:
//! - Binary operations (arithmetic, logical, bitwise, comparison)
//! - Unary operations (negation, not, bitwise not, reference, dereference)
//! - Control flow expressions (if, loop, match)
//! - Literals, identifiers, function calls, method calls
//! - Lambda expressions, pattern matching
//! - Array and struct literals

use vais_ast::*;

use crate::{ParseResult, Parser};

mod precedence;
mod unary;
mod postfix;
mod primary;

/// Check if a string contains interpolation syntax: `{<non-empty>}`.
/// Empty `{}` is NOT interpolation (backward compat with format strings).
/// `{{` is an escaped brace, not interpolation.
fn has_interpolation(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                // Escaped {{ - skip
                chars.next();
            } else if chars.peek() != Some(&'}') {
                // Non-empty content inside braces - this is interpolation
                // Verify there's a closing brace
                let mut depth = 1;
                for c in chars.by_ref() {
                    if c == '{' {
                        depth += 1;
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            return true;
                        }
                    }
                }
            } else {
                // Empty {} - skip
                chars.next();
            }
        } else if ch == '}' && chars.peek() == Some(&'}') {
            // Escaped }}
            chars.next();
        }
    }
    false
}

impl Parser {
    /// Parse expression
    pub fn parse_expr(&mut self) -> ParseResult<Spanned<Expr>> {
        self.enter_depth()?;
        let result = self.parse_assignment();
        self.exit_depth();
        result
    }
}
