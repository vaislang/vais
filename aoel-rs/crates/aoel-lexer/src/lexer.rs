//! AOEL Lexer implementation
//!
//! Uses the `logos` crate for fast lexing.

use logos::Logos;
use crate::token::{Token, TokenKind, Span};
use crate::error::LexerError;

/// AOEL Lexer
///
/// Converts source code into a stream of tokens.
pub struct Lexer<'a> {
    source: &'a str,
    inner: logos::Lexer<'a, TokenKind>,
    peeked: Option<Option<Token>>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            inner: TokenKind::lexer(source),
            peeked: None,
        }
    }

    /// Get the original source code
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// Peek at the next token without consuming it
    pub fn peek(&mut self) -> Option<&Token> {
        if self.peeked.is_none() {
            self.peeked = Some(self.next_token());
        }
        self.peeked.as_ref().and_then(|t| t.as_ref())
    }

    /// Get the next token
    fn next_token(&mut self) -> Option<Token> {
        loop {
            let kind = self.inner.next()?;
            let span = self.inner.span();
            let text = self.inner.slice();

            match kind {
                Ok(TokenKind::Newline) => {
                    // Skip newlines but track them for line counting
                    continue;
                }
                Ok(kind) => {
                    return Some(Token::new(
                        kind,
                        Span::new(span.start, span.end),
                        text,
                    ));
                }
                Err(()) => {
                    return Some(Token::new(
                        TokenKind::Error,
                        Span::new(span.start, span.end),
                        text,
                    ));
                }
            }
        }
    }

    /// Tokenize the entire source and return all tokens
    pub fn tokenize(source: &str) -> Result<Vec<Token>, LexerError> {
        let lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        for token in lexer {
            if token.kind == TokenKind::Error {
                return Err(LexerError::InvalidToken {
                    span: token.span,
                    text: token.text,
                });
            }
            tokens.push(token);
        }

        // Add EOF token
        let eof_pos = source.len();
        tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(eof_pos, eof_pos),
            "",
        ));

        Ok(tokens)
    }

    /// Get line and column for a byte offset
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in self.source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(peeked) = self.peeked.take() {
            return peeked;
        }
        self.next_token()
    }
}

/// Helper struct for tracking source locations
#[derive(Debug, Clone)]
pub struct SourceMap {
    source: String,
    line_starts: Vec<usize>,
}

impl SourceMap {
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        let mut line_starts = vec![0];

        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }

        Self { source, line_starts }
    }

    /// Get line and column (1-indexed) for a byte offset
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = self
            .line_starts
            .iter()
            .position(|&start| start > offset)
            .unwrap_or(self.line_starts.len())
            .saturating_sub(1);

        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        let col = offset - line_start;

        (line + 1, col + 1)
    }

    /// Get the source text for a span
    pub fn span_text(&self, span: Span) -> &str {
        &self.source[span.start..span.end]
    }

    /// Get the line containing a byte offset
    pub fn line_text(&self, offset: usize) -> &str {
        let (line, _) = self.line_col(offset);
        let line_idx = line - 1;

        let start = self.line_starts.get(line_idx).copied().unwrap_or(0);
        let end = self
            .line_starts
            .get(line_idx + 1)
            .copied()
            .unwrap_or(self.source.len());

        self.source[start..end].trim_end_matches('\n')
    }
}
