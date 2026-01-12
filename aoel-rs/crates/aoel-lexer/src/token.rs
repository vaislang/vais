//! Token definitions for AOEL
//!
//! Defines all token types and their representations.

use logos::Logos;
use serde::{Serialize, Deserialize};

/// Span represents a range in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A token with its kind and location
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: text.into(),
        }
    }
}

/// All token types in AOEL
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]  // Skip whitespace (not newlines)
#[logos(skip r"#[^\n]*")]   // Skip comments
pub enum TokenKind {
    // =========================================================================
    // Block Keywords
    // =========================================================================
    #[token("UNIT")]
    Unit,
    #[token("META")]
    Meta,
    #[token("ENDMETA")]
    EndMeta,
    #[token("INPUT")]
    Input,
    #[token("ENDINPUT")]
    EndInput,
    #[token("OUTPUT")]
    Output,
    #[token("ENDOUTPUT")]
    EndOutput,
    #[token("INTENT")]
    Intent,
    #[token("ENDINTENT")]
    EndIntent,
    #[token("CONSTRAINT")]
    Constraint,
    #[token("ENDCONSTRAINT")]
    EndConstraint,
    #[token("FLOW")]
    Flow,
    #[token("ENDFLOW")]
    EndFlow,
    #[token("EXECUTION")]
    Execution,
    #[token("ENDEXECUTION")]
    EndExecution,
    #[token("VERIFY")]
    Verify,
    #[token("ENDVERIFY")]
    EndVerify,
    #[token("END")]
    End,

    // =========================================================================
    // Unit Types
    // =========================================================================
    #[token("FUNCTION")]
    Function,
    #[token("SERVICE")]
    Service,
    #[token("PIPELINE")]
    Pipeline,
    #[token("MODULE")]
    Module,

    // =========================================================================
    // Meta Keywords
    // =========================================================================
    #[token("DOMAIN")]
    Domain,
    #[token("DETERMINISM")]
    Determinism,
    #[token("IDEMPOTENT")]
    Idempotent,
    #[token("PURE")]
    Pure,
    #[token("TIMEOUT")]
    Timeout,
    #[token("RETRY")]
    Retry,

    // =========================================================================
    // Intent Keywords
    // =========================================================================
    #[token("GOAL")]
    Goal,
    #[token("PRIORITY")]
    Priority,
    #[token("ON_FAILURE")]
    OnFailure,

    // Goal Types
    #[token("TRANSFORM")]
    Transform,
    #[token("VALIDATE")]
    Validate,
    #[token("AGGREGATE")]
    Aggregate,
    #[token("FILTER")]
    Filter,
    #[token("ROUTE")]
    Route,
    #[token("COMPOSE")]
    Compose,
    #[token("FETCH")]
    Fetch,

    // Priority Values
    #[token("CORRECTNESS")]
    Correctness,
    #[token("PERFORMANCE")]
    Performance,
    #[token("LATENCY")]
    Latency,
    #[token("THROUGHPUT")]
    Throughput,

    // Failure Strategies
    #[token("ABORT")]
    Abort,
    #[token("FALLBACK")]
    Fallback,
    #[token("DEFAULT")]
    Default,

    // =========================================================================
    // Constraint Keywords
    // =========================================================================
    #[token("REQUIRE")]
    Require,
    #[token("FORBID")]
    Forbid,
    #[token("PREFER")]
    Prefer,
    #[token("INVARIANT")]
    Invariant,
    #[token("WITHIN")]
    Within,

    // =========================================================================
    // Flow Keywords
    // =========================================================================
    #[token("NODE")]
    Node,
    #[token("EDGE")]
    Edge,
    #[token("WHEN")]
    When,

    // Operations
    #[token("MAP")]
    Map,
    #[token("REDUCE")]
    Reduce,
    #[token("SPLIT")]
    Split,
    #[token("MERGE")]
    Merge,
    #[token("BRANCH")]
    Branch,
    #[token("JOIN")]
    Join,
    #[token("RACE")]
    Race,
    #[token("STORE")]
    Store,
    #[token("CALL")]
    Call,
    #[token("EMIT")]
    Emit,
    #[token("SUBSCRIBE")]
    Subscribe,
    #[token("SANITIZE")]
    Sanitize,
    #[token("AUTHORIZE")]
    Authorize,

    // =========================================================================
    // Execution Keywords
    // =========================================================================
    #[token("PARALLEL")]
    Parallel,
    #[token("TARGET")]
    Target,
    #[token("ISOLATION")]
    Isolation,
    #[token("CACHE")]
    Cache,

    // Target Values
    #[token("ANY")]
    Any,
    #[token("CPU")]
    Cpu,
    #[token("GPU")]
    Gpu,
    #[token("WASM")]
    Wasm,
    #[token("NATIVE")]
    Native,

    // Memory Values
    #[token("MEMORY")]
    Memory,
    #[token("BOUNDED")]
    Bounded,
    #[token("UNBOUNDED")]
    Unbounded,
    #[token("STACK_ONLY")]
    StackOnly,

    // Isolation Values
    #[token("NONE")]
    None_,
    #[token("THREAD")]
    Thread,
    #[token("PROCESS")]
    Process,
    #[token("CONTAINER")]
    Container,

    // Cache Values
    #[token("LRU")]
    Lru,
    #[token("TTL")]
    Ttl,

    // =========================================================================
    // Verify Keywords
    // =========================================================================
    #[token("ASSERT")]
    Assert,
    #[token("PROPERTY")]
    Property,
    #[token("POSTCONDITION")]
    Postcondition,
    #[token("TEST")]
    Test,
    #[token("FORALL")]
    Forall,
    #[token("EXISTS")]
    Exists,
    #[token("EVENTUALLY")]
    Eventually,
    #[token("ALWAYS")]
    Always,

    // =========================================================================
    // Type Keywords
    // =========================================================================
    #[token("INT")]
    Int,
    #[token("INT8")]
    Int8,
    #[token("INT16")]
    Int16,
    #[token("INT32")]
    Int32,
    #[token("INT64")]
    Int64,
    #[token("UINT")]
    Uint,
    #[token("UINT8")]
    Uint8,
    #[token("UINT16")]
    Uint16,
    #[token("UINT32")]
    Uint32,
    #[token("UINT64")]
    Uint64,
    #[token("FLOAT32")]
    Float32,
    #[token("FLOAT64")]
    Float64,
    #[token("BOOL")]
    Bool,
    #[token("STRING")]
    String_,
    #[token("BYTES")]
    Bytes,
    #[token("VOID")]
    Void,
    #[token("ARRAY")]
    Array,
    #[token("STRUCT")]
    Struct,
    #[token("OPTIONAL")]
    Optional,
    #[token("UNION")]
    Union,

    // =========================================================================
    // Logical Operators
    // =========================================================================
    #[token("AND")]
    And,
    #[token("OR")]
    Or,
    #[token("XOR")]
    Xor,
    #[token("NOT")]
    Not,
    #[token("IMPLIES")]
    Implies,
    #[token("IN")]
    In,
    #[token("MATCH")]
    Match,

    // =========================================================================
    // Built-in Functions
    // =========================================================================
    #[token("LEN")]
    Len,
    #[token("CONTAINS")]
    Contains,
    #[token("RANGE")]
    Range,
    #[token("NOW")]
    Now,
    #[token("SUM")]
    Sum,
    #[token("COUNT")]
    Count,

    // =========================================================================
    // Literals
    // =========================================================================
    #[token("true")]
    True,
    #[token("false")]
    False,

    #[regex(r"-?[0-9]+", priority = 2)]
    Integer,

    #[regex(r"-?[0-9]+\.[0-9]+")]
    Float,

    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,

    #[regex(r"/([^/\\]|\\.)+/")]
    Regex,

    // Version (V1.0.0)
    #[regex(r"V[0-9]+\.[0-9]+\.[0-9]+")]
    Version,

    // Duration (10s, 5m, 100ms)
    #[regex(r"[0-9]+(ms|s|m|h)")]
    Duration,

    // Size (256MB, 1GB)
    #[regex(r"[0-9]+(KB|MB|GB)")]
    Size,

    // =========================================================================
    // Identifiers and References
    // =========================================================================
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"@[a-zA-Z_][a-zA-Z0-9_.]*")]
    ExternalRef,

    // =========================================================================
    // Punctuation
    // =========================================================================
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("->")]
    Arrow,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Lte,
    #[token(">=")]
    Gte,
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("=")]
    Assign,
    #[token("|")]
    Pipe,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,

    // =========================================================================
    // Special
    // =========================================================================
    #[token("\n")]
    Newline,

    Eof,

    Error,
}

impl TokenKind {
    /// Returns true if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Unit
                | TokenKind::Meta
                | TokenKind::EndMeta
                | TokenKind::Input
                | TokenKind::EndInput
                | TokenKind::Output
                | TokenKind::EndOutput
                | TokenKind::Intent
                | TokenKind::EndIntent
                | TokenKind::Constraint
                | TokenKind::EndConstraint
                | TokenKind::Flow
                | TokenKind::EndFlow
                | TokenKind::Execution
                | TokenKind::EndExecution
                | TokenKind::Verify
                | TokenKind::EndVerify
                | TokenKind::End
        )
    }

    /// Returns true if this token is a type keyword
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            TokenKind::Int
                | TokenKind::Int8
                | TokenKind::Int16
                | TokenKind::Int32
                | TokenKind::Int64
                | TokenKind::Uint
                | TokenKind::Uint8
                | TokenKind::Uint16
                | TokenKind::Uint32
                | TokenKind::Uint64
                | TokenKind::Float32
                | TokenKind::Float64
                | TokenKind::Bool
                | TokenKind::String_
                | TokenKind::Bytes
                | TokenKind::Void
                | TokenKind::Array
                | TokenKind::Struct
                | TokenKind::Optional
                | TokenKind::Union
        )
    }

    /// Returns true if this token is an operator
    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            TokenKind::And
                | TokenKind::Or
                | TokenKind::Xor
                | TokenKind::Not
                | TokenKind::Implies
                | TokenKind::In
                | TokenKind::Match
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Lte
                | TokenKind::Gte
                | TokenKind::Eq
                | TokenKind::Neq
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
        )
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Unit => write!(f, "UNIT"),
            TokenKind::Meta => write!(f, "META"),
            TokenKind::EndMeta => write!(f, "ENDMETA"),
            TokenKind::Input => write!(f, "INPUT"),
            TokenKind::EndInput => write!(f, "ENDINPUT"),
            TokenKind::Output => write!(f, "OUTPUT"),
            TokenKind::EndOutput => write!(f, "ENDOUTPUT"),
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::Integer => write!(f, "integer"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::StringLiteral => write!(f, "string"),
            TokenKind::Eof => write!(f, "end of file"),
            TokenKind::Error => write!(f, "error"),
            other => write!(f, "{:?}", other),
        }
    }
}
