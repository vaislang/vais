//! AOEL Lexer
//!
//! Tokenizes AOEL source code into a stream of tokens.
//!
//! # Example
//!
//! ```
//! use aoel_lexer::{Lexer, TokenKind};
//!
//! let source = "UNIT FUNCTION test V1.0.0";
//! let lexer = Lexer::new(source);
//! let tokens: Vec<_> = lexer.collect();
//! ```

mod token;
mod lexer;
mod error;

pub use token::{Token, TokenKind, Span};
pub use lexer::Lexer;
pub use error::LexerError;

#[cfg(test)]
mod tests;
