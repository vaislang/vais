//! Vais Static Security Analysis
//!
//! This crate provides static security analysis for Vais programs by analyzing
//! the Abstract Syntax Tree (AST) to detect potential security vulnerabilities.
//!
//! # Features
//!
//! - **Buffer Overflow Detection**: Identifies unsafe memory operations without bounds checking
//! - **Pointer Safety**: Detects unsafe pointer arithmetic and use-after-free issues
//! - **Injection Prevention**: Finds SQL/command injection vulnerabilities
//! - **Secret Detection**: Locates hardcoded secrets, API keys, and passwords
//! - **Integer Overflow**: Identifies arithmetic operations on unchecked inputs
//!
//! # Example
//!
//! ```rust,ignore
//! use vais_security::{SecurityAnalyzer, Severity};
//! use vais_parser::Parser;
//! use vais_lexer::Lexer;
//!
//! let source = r#"
//!     F main() -> i64 {
//!         ptr := malloc(100)
//!         store_i64(ptr, 42)
//!         free(ptr)
//!         0
//!     }
//! "#;
//!
//! let lexer = Lexer::new(source);
//! let tokens = lexer.collect::<Result<Vec<_>, _>>().unwrap();
//! let mut parser = Parser::new(&tokens);
//! let module = parser.parse_module().unwrap();
//!
//! let mut analyzer = SecurityAnalyzer::new();
//! let findings = analyzer.analyze(&module);
//!
//! for finding in findings {
//!     if finding.severity >= Severity::High {
//!         println!("{}", finding);
//!     }
//! }
//! ```
//!
//! # Security Checks
//!
//! The analyzer performs the following checks:
//!
//! ## Buffer Overflow Risks
//! - Direct memory operations (`malloc`, `free`, `load_byte`, `store_byte`)
//! - Array indexing without bounds checking
//! - Unsafe C functions (`strcpy`, `strcat`, `gets`, `sprintf`)
//!
//! ## Unsafe Pointer Arithmetic
//! - Raw pointer addition/subtraction
//! - Dereferencing potentially invalid pointers
//!
//! ## Injection Vulnerabilities
//! - Command execution with string concatenation (`system`, `exec`)
//! - SQL queries built with string concatenation
//!
//! ## Hardcoded Secrets
//! - API keys, tokens, passwords in string literals
//! - High-entropy strings that may be secrets
//!
//! ## Integer Overflow
//! - Arithmetic on unchecked user input
//! - Operations that may overflow without checking

pub mod analyzer;
pub mod findings;

#[cfg(test)]
mod tests;

pub use analyzer::SecurityAnalyzer;
pub use findings::{SecurityFinding, Severity, FindingCategory};
