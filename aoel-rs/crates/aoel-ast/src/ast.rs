//! AOEL AOEL Abstract Syntax Tree
//!
//! AOEL 문법에 최적화된 AST 노드 정의

use aoel_lexer::Span;
use serde::{Deserialize, Serialize};

// =============================================================================
// Program
// =============================================================================

/// 프로그램 (최상위 노드)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

/// 최상위 아이템
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    /// 함수 정의: name(params) = body
    Function(FunctionDef),
    /// 타입 정의: type Name = ...
    TypeDef(TypeDef),
    /// 모듈: mod name
    Module(ModuleDef),
    /// import: use path
    Use(UseDef),
    /// FFI 선언: ffi "lib" { ... }
    Ffi(FfiBlock),
    /// 표현식 (REPL용)
    Expr(Expr),
}

// =============================================================================
// Function Definition
// =============================================================================

/// 함수 정의
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    /// 함수 이름
    pub name: String,
    /// 매개변수
    pub params: Vec<Param>,
    /// 반환 타입 (옵션)
    pub return_type: Option<TypeExpr>,
    /// 함수 본문
    pub body: Expr,
    /// public 여부
    pub is_pub: bool,
    pub span: Span,
}

/// 매개변수
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    /// 매개변수 이름
    pub name: String,
    /// 타입 (옵션)
    pub ty: Option<TypeExpr>,
    /// 기본값 (옵션)
    pub default: Option<Expr>,
    pub span: Span,
}

// =============================================================================
// Type Definition
// =============================================================================

/// 타입 정의
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    pub ty: TypeExpr,
    pub is_pub: bool,
    pub span: Span,
}

/// 타입 표현식
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeExpr {
    /// 기본 타입: i, f, s, b
    Simple(String),
    /// 배열: [T]
    Array(Box<TypeExpr>),
    /// 맵: {K: V}
    Map(Box<TypeExpr>, Box<TypeExpr>),
    /// 튜플: (T1, T2, ...)
    Tuple(Vec<TypeExpr>),
    /// Optional: ?T
    Optional(Box<TypeExpr>),
    /// Result: !T
    Result(Box<TypeExpr>),
    /// 함수: (A, B) -> C
    Function(Vec<TypeExpr>, Box<TypeExpr>),
    /// 구조체 타입: { field: Type, ... }
    Struct(Vec<(String, TypeExpr)>),
}

// =============================================================================
// Module & Use
// =============================================================================

/// 모듈 정의
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDef {
    pub name: String,
    pub span: Span,
}

/// Use 문
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseDef {
    pub path: Vec<String>,
    /// 특정 아이템 import: use path.{a, b}
    pub items: Option<Vec<String>>,
    /// alias: use path as name
    pub alias: Option<String>,
    pub span: Span,
}

// =============================================================================
// FFI (Foreign Function Interface)
// =============================================================================

/// FFI 블록: ffi "libname" { fn_decls... }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiBlock {
    /// 라이브러리 이름 (예: "c", "math", "./mylib.so")
    pub lib_name: String,
    /// ABI (기본값: "C")
    pub abi: String,
    /// 함수 선언들
    pub functions: Vec<FfiFn>,
    pub span: Span,
}

/// FFI 함수 선언
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiFn {
    /// AOEL에서 사용할 이름
    pub name: String,
    /// 외부 라이브러리에서의 원래 이름 (없으면 name 사용)
    pub extern_name: Option<String>,
    /// 매개변수 (이름, 타입)
    pub params: Vec<(String, FfiType)>,
    /// 반환 타입
    pub return_type: FfiType,
    pub span: Span,
}

/// FFI 타입 (C 호환)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FfiType {
    /// void
    Void,
    /// i8, i16, i32, i64
    Int(u8),
    /// u8, u16, u32, u64
    Uint(u8),
    /// f32
    F32,
    /// f64
    F64,
    /// bool
    Bool,
    /// *const T (읽기 전용 포인터)
    Ptr(Box<FfiType>),
    /// *mut T (쓰기 가능 포인터)
    MutPtr(Box<FfiType>),
    /// C 문자열 (*const c_char)
    CStr,
    /// 불투명 포인터 (void*)
    Opaque,
}

impl FfiType {
    /// C 타입 문자열 반환
    pub fn to_c_type(&self) -> &'static str {
        match self {
            FfiType::Void => "void",
            FfiType::Int(8) => "int8_t",
            FfiType::Int(16) => "int16_t",
            FfiType::Int(32) => "int32_t",
            FfiType::Int(64) => "int64_t",
            FfiType::Int(_) => "int64_t",
            FfiType::Uint(8) => "uint8_t",
            FfiType::Uint(16) => "uint16_t",
            FfiType::Uint(32) => "uint32_t",
            FfiType::Uint(64) => "uint64_t",
            FfiType::Uint(_) => "uint64_t",
            FfiType::F32 => "float",
            FfiType::F64 => "double",
            FfiType::Bool => "bool",
            FfiType::Ptr(_) => "const void*",
            FfiType::MutPtr(_) => "void*",
            FfiType::CStr => "const char*",
            FfiType::Opaque => "void*",
        }
    }
}

// =============================================================================
// Expressions
// =============================================================================

/// 표현식
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    // === Literals ===
    /// 정수
    Integer(i64, Span),
    /// 실수
    Float(f64, Span),
    /// 문자열
    String(String, Span),
    /// 불리언
    Bool(bool, Span),
    /// nil
    Nil(Span),

    // === Identifiers ===
    /// 변수/함수 참조
    Ident(String, Span),
    /// 람다 매개변수 (_)
    LambdaParam(Span),

    // === Collections ===
    /// 배열: [a, b, c]
    Array(Vec<Expr>, Span),
    /// 맵/구조체: {key: value, ...}
    Map(Vec<(String, Expr)>, Span),
    /// 튜플: (a, b, c)
    Tuple(Vec<Expr>, Span),

    // === Operations ===
    /// 이항 연산: a + b, a == b, etc.
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    /// 단항 연산: -a, !a, #a
    Unary(UnaryOp, Box<Expr>, Span),

    // === AOEL Collection Operations ===
    /// Map: arr.@(expr)
    MapOp(Box<Expr>, Box<Expr>, Span),
    /// Filter: arr.?(expr)
    FilterOp(Box<Expr>, Box<Expr>, Span),
    /// Reduce: arr./op
    ReduceOp(Box<Expr>, ReduceKind, Span),

    // === Access ===
    /// 필드 접근: obj.field
    Field(Box<Expr>, String, Span),
    /// 인덱스: arr[i] or arr[start:end]
    Index(Box<Expr>, Box<IndexKind>, Span),
    /// 메서드 호출: obj.method(args)
    MethodCall(Box<Expr>, String, Vec<Expr>, Span),

    // === Control Flow ===
    /// 삼항: cond ? then : else
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>, Span),
    /// If 표현식: if cond { then } else { else }
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>, Span),
    /// Match 표현식
    Match(Box<Expr>, Vec<MatchArm>, Span),
    /// 블록: { expr1; expr2 }
    Block(Vec<Expr>, Span),

    // === Binding ===
    /// let 바인딩: let x = v : body 또는 let x = v; body
    Let(Vec<(String, Expr)>, Box<Expr>, Span),

    // === Function ===
    /// 함수 호출: f(args)
    Call(Box<Expr>, Vec<Expr>, Span),
    /// 재귀 호출: $(args)
    SelfCall(Vec<Expr>, Span),
    /// 람다: (params) => body 또는 암묵적 _
    Lambda(Vec<String>, Box<Expr>, Span),

    // === Special ===
    /// 범위: a..b
    Range(Box<Expr>, Box<Expr>, Span),
    /// 포함 여부: x @ arr
    Contains(Box<Expr>, Box<Expr>, Span),
    /// 에러: err 또는 err("msg")
    Error(Option<Box<Expr>>, Span),
    /// Optional unwrap: expr?
    Try(Box<Expr>, Span),
    /// Optional with default: expr ?? default
    Coalesce(Box<Expr>, Box<Expr>, Span),
}

impl Expr {
    /// 표현식의 Span 반환
    pub fn span(&self) -> Span {
        match self {
            Expr::Integer(_, s) => *s,
            Expr::Float(_, s) => *s,
            Expr::String(_, s) => *s,
            Expr::Bool(_, s) => *s,
            Expr::Nil(s) => *s,
            Expr::Ident(_, s) => *s,
            Expr::LambdaParam(s) => *s,
            Expr::Array(_, s) => *s,
            Expr::Map(_, s) => *s,
            Expr::Tuple(_, s) => *s,
            Expr::Binary(_, _, _, s) => *s,
            Expr::Unary(_, _, s) => *s,
            Expr::MapOp(_, _, s) => *s,
            Expr::FilterOp(_, _, s) => *s,
            Expr::ReduceOp(_, _, s) => *s,
            Expr::Field(_, _, s) => *s,
            Expr::Index(_, _, s) => *s,
            Expr::MethodCall(_, _, _, s) => *s,
            Expr::Ternary(_, _, _, s) => *s,
            Expr::If(_, _, _, s) => *s,
            Expr::Match(_, _, s) => *s,
            Expr::Block(_, s) => *s,
            Expr::Let(_, _, s) => *s,
            Expr::Call(_, _, s) => *s,
            Expr::SelfCall(_, s) => *s,
            Expr::Lambda(_, _, s) => *s,
            Expr::Range(_, _, s) => *s,
            Expr::Contains(_, _, s) => *s,
            Expr::Error(_, s) => *s,
            Expr::Try(_, s) => *s,
            Expr::Coalesce(_, _, s) => *s,
        }
    }
}

// =============================================================================
// Operators
// =============================================================================

/// 이항 연산자
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // 산술
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // 비교
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    // 논리
    And,
    Or,
    // 문자열
    Concat,
}

impl BinaryOp {
    /// 우선순위 반환 (높을수록 먼저 바인딩)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Eq | BinaryOp::NotEq => 3,
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => 4,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Concat => 5,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 6,
        }
    }

    /// 좌결합인지
    pub fn is_left_associative(&self) -> bool {
        true
    }
}

/// 단항 연산자
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// 부정: -x
    Neg,
    /// 논리 NOT: !x
    Not,
    /// 길이: #x
    Len,
}

/// Reduce 연산 종류
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReduceKind {
    /// ./+
    Sum,
    /// ./*
    Product,
    /// ./min
    Min,
    /// ./max
    Max,
    /// ./and
    And,
    /// ./or
    Or,
    /// 커스텀: ./(init, fn)
    Custom(Box<Expr>, Box<Expr>),
}

/// 인덱스 종류
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexKind {
    /// 단일 인덱스: arr[i]
    Single(Expr),
    /// 슬라이스: arr[start:end]
    Slice(Option<Expr>, Option<Expr>),
}

// =============================================================================
// Pattern Matching
// =============================================================================

/// Match 암
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

/// 패턴
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    /// 와일드카드: _
    Wildcard(Span),
    /// 리터럴: 1, "hello", true
    Literal(Expr),
    /// 바인딩: x
    Binding(String, Span),
    /// 구조체: { field, ... }
    Struct(Vec<(String, Option<Pattern>)>, Span),
    /// 튜플: (a, b, c)
    Tuple(Vec<Pattern>, Span),
    /// 배열: [a, b, ...]
    Array(Vec<Pattern>, Span),
    /// Enum variant: Some(x), None
    Variant(String, Option<Box<Pattern>>, Span),
}
