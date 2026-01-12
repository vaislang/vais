//! AOEL Parser
//!
//! Parses AOEL source code into an AST.
//!
//! # Example
//!
//! ```ignore
//! use aoel_parser::Parser;
//!
//! let source = r#"
//! UNIT FUNCTION test V1.0.0
//! META
//!   DOMAIN test
//!   DETERMINISM true
//! ENDMETA
//! ...
//! END
//! "#;
//!
//! let unit = Parser::parse(source)?;
//! ```

mod parser;
mod error;

pub use parser::Parser;
pub use error::{ParseError, ParseResult};

/// Parse AOEL source code into a Unit AST
pub fn parse(source: &str) -> ParseResult<aoel_ast::Unit> {
    Parser::parse(source)
}
