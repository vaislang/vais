//! Vais Vais Lexer Implementation
//!
//! logos 기반의 빠른 렉서 구현

use logos::Logos;

use crate::error::LexError;
use crate::token::{Span, Token, TokenKind};

/// Vais Vais Lexer
#[derive(Clone)]
pub struct Lexer<'src> {
    source: &'src str,
    inner: logos::Lexer<'src, TokenKind>,
    peeked: Option<Token>,
    at_eof: bool,
}

impl<'src> Lexer<'src> {
    /// 새 렉서 생성
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            inner: TokenKind::lexer(source),
            peeked: None,
            at_eof: false,
        }
    }

    /// 소스 코드 반환
    pub fn source(&self) -> &'src str {
        self.source
    }

    /// 다음 토큰을 미리 보기 (소비하지 않음)
    pub fn peek(&mut self) -> Option<&Token> {
        if self.peeked.is_none() {
            self.peeked = self.next_token_internal();
        }
        self.peeked.as_ref()
    }

    /// 다음 토큰 반환
    pub fn next_token(&mut self) -> Option<Token> {
        if let Some(token) = self.peeked.take() {
            return Some(token);
        }
        self.next_token_internal()
    }

    /// 내부 토큰 추출
    fn next_token_internal(&mut self) -> Option<Token> {
        if self.at_eof {
            return None;
        }

        loop {
            match self.inner.next() {
                Some(result) => {
                    let span = self.inner.span();
                    let text = self.inner.slice();

                    match result {
                        Ok(kind) => {
                            // 주석과 줄바꿈 스킵 (필요시)
                            if kind == TokenKind::Comment {
                                continue;
                            }

                            return Some(Token::new(
                                kind,
                                Span::new(span.start, span.end),
                                text,
                            ));
                        }
                        Err(_) => {
                            // 알 수 없는 문자
                            return Some(Token::new(
                                TokenKind::Error,
                                Span::new(span.start, span.end),
                                text,
                            ));
                        }
                    }
                }
                None => {
                    self.at_eof = true;
                    // EOF 토큰 반환
                    let pos = self.source.len();
                    return Some(Token::new(
                        TokenKind::Eof,
                        Span::new(pos, pos),
                        "",
                    ));
                }
            }
        }
    }

    /// 모든 토큰 수집 (주석 제외)
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while let Some(token) = self.next_token() {
            if token.kind == TokenKind::Eof {
                tokens.push(token);
                break;
            }

            if token.kind == TokenKind::Error {
                errors.push(LexError::UnexpectedCharacter {
                    char: token.text.chars().next().unwrap_or('?'),
                    span: token.span,
                });
            } else {
                tokens.push(token);
            }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors.into_iter().next().unwrap())
        }
    }

    /// 줄바꿈 제외하고 모든 토큰 수집
    pub fn tokenize_no_newlines(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while let Some(token) = self.next_token() {
            if token.kind == TokenKind::Eof {
                tokens.push(token);
                break;
            }

            if token.kind == TokenKind::Newline {
                continue;
            }

            if token.kind == TokenKind::Error {
                errors.push(LexError::UnexpectedCharacter {
                    char: token.text.chars().next().unwrap_or('?'),
                    span: token.span,
                });
            } else {
                tokens.push(token);
            }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors.into_iter().next().unwrap())
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token()?;
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let source = "add(a,b)=a+b";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                &TokenKind::Identifier, // add
                &TokenKind::LParen,     // (
                &TokenKind::Identifier, // a
                &TokenKind::Comma,      // ,
                &TokenKind::Identifier, // b
                &TokenKind::RParen,     // )
                &TokenKind::Eq,         // =
                &TokenKind::Identifier, // a
                &TokenKind::Plus,       // +
                &TokenKind::Identifier, // b
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_collection_operators() {
        let source = "arr.@(_*2).?(_>0)./+";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                &TokenKind::Identifier,   // arr
                &TokenKind::DotAt,        // .@
                &TokenKind::LParen,       // (
                &TokenKind::Underscore,   // _
                &TokenKind::Star,         // *
                &TokenKind::Integer,      // 2
                &TokenKind::RParen,       // )
                &TokenKind::DotQuestion,  // .?
                &TokenKind::LParen,       // (
                &TokenKind::Underscore,   // _
                &TokenKind::Gt,           // >
                &TokenKind::Integer,      // 0
                &TokenKind::RParen,       // )
                &TokenKind::DotSlash,     // ./
                &TokenKind::Plus,         // +
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_recursion() {
        let source = "fib(n)=n<2?n:$(n-1)+$(n-2)";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        // $ 토큰이 있는지 확인
        let has_dollar = tokens.iter().any(|t| t.kind == TokenKind::Dollar);
        assert!(has_dollar, "Should have $ token for self-recursion");
    }

    #[test]
    fn test_ternary() {
        let source = "a>b?a:b";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                &TokenKind::Identifier, // a
                &TokenKind::Gt,         // >
                &TokenKind::Identifier, // b
                &TokenKind::Question,   // ?
                &TokenKind::Identifier, // a
                &TokenKind::Colon,      // :
                &TokenKind::Identifier, // b
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_length_operator() {
        let source = "#arr";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Hash);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_contains_operator() {
        let source = "x@arr";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                &TokenKind::Identifier, // x
                &TokenKind::At,         // @
                &TokenKind::Identifier, // arr
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_range_operator() {
        let source = "1..10";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                &TokenKind::Integer, // 1
                &TokenKind::DotDot,  // ..
                &TokenKind::Integer, // 10
                &TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let source = r#"hello()="Hello, World!""#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        let string_token = tokens.iter().find(|t| t.kind == TokenKind::String);
        assert!(string_token.is_some());
        assert_eq!(string_token.unwrap().text, "\"Hello, World!\"");
    }

    #[test]
    fn test_float_literal() {
        let source = "pi=3.14159";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        let float_token = tokens.iter().find(|t| t.kind == TokenKind::Float);
        assert!(float_token.is_some());
        assert_eq!(float_token.unwrap().text, "3.14159");
    }

    #[test]
    fn test_keywords() {
        let source = "let if else match for in fn pub mod use type";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        for token in &tokens[..tokens.len() - 1] {
            assert!(token.kind.is_keyword(), "{:?} should be keyword", token.kind);
        }
    }

    #[test]
    fn test_complex_example() {
        // 실제 Vais 예제
        let source = "qs(a)=#a<2?a:let p=a[0],r=a[1:]:$(r.?(_<p))+[p]+$(r.?(_>=p))";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize_no_newlines().unwrap();

        // 토큰화 성공 확인
        assert!(!tokens.is_empty());
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);

        // 에러 토큰 없음 확인
        let errors: Vec<_> = tokens.iter().filter(|t| t.kind == TokenKind::Error).collect();
        assert!(errors.is_empty(), "Should have no error tokens: {:?}", errors);
    }
}
