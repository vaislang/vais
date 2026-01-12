//! AOEL v6b Lexer
//!
//! AI 토큰 효율성에 최적화된 렉서.
//! Python 대비 44% 토큰 절감을 목표로 설계된 v6b 문법을 지원합니다.
//!
//! # 핵심 특징
//!
//! - `.@` - Map 연산자
//! - `.?` - Filter 연산자
//! - `./` - Reduce 연산자
//! - `$` - Self recursion
//! - `#` - Length operator
//! - `_` - Lambda parameter
//! - `@` - Contains operator
//! - `..` - Range operator
//!
//! # 예제
//!
//! ```
//! use aoel_v6b_lexer::Lexer;
//!
//! let source = "add(a,b)=a+b";
//! let mut lexer = Lexer::new(source);
//! let tokens = lexer.tokenize().unwrap();
//!
//! // tokens: [Identifier("add"), LParen, Identifier("a"), Comma, ...]
//! ```

pub mod error;
pub mod lexer;
pub mod token;

pub use error::LexError;
pub use lexer::Lexer;
pub use token::{Span, Token, TokenKind};

/// 소스 코드를 토큰화하는 편의 함수
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

/// 줄바꿈 제외하고 토큰화하는 편의 함수
pub fn tokenize_no_newlines(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize_no_newlines()
}
