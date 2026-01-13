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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    // === Program Tests ===

    #[test]
    fn test_program_creation() {
        let program = Program {
            items: vec![],
            span: dummy_span(),
        };
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_program_with_function() {
        let func = FunctionDef {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: Expr::Integer(42, dummy_span()),
            is_pub: false,
            span: dummy_span(),
        };
        let program = Program {
            items: vec![Item::Function(func)],
            span: dummy_span(),
        };
        assert_eq!(program.items.len(), 1);
    }

    // === Item Tests ===

    #[test]
    fn test_item_variants() {
        let func = FunctionDef {
            name: "f".to_string(),
            params: vec![],
            return_type: None,
            body: Expr::Nil(dummy_span()),
            is_pub: false,
            span: dummy_span(),
        };
        assert!(matches!(Item::Function(func), Item::Function(_)));

        let typedef = TypeDef {
            name: "MyType".to_string(),
            ty: TypeExpr::Simple("Int".to_string()),
            is_pub: false,
            span: dummy_span(),
        };
        assert!(matches!(Item::TypeDef(typedef), Item::TypeDef(_)));

        let module = ModuleDef {
            name: "mymod".to_string(),
            span: dummy_span(),
        };
        assert!(matches!(Item::Module(module), Item::Module(_)));

        let use_def = UseDef {
            path: vec!["std".to_string(), "io".to_string()],
            items: None,
            alias: None,
            span: dummy_span(),
        };
        assert!(matches!(Item::Use(use_def), Item::Use(_)));

        assert!(matches!(Item::Expr(Expr::Nil(dummy_span())), Item::Expr(_)));
    }

    // === FunctionDef Tests ===

    #[test]
    fn test_function_def_with_params() {
        let func = FunctionDef {
            name: "add".to_string(),
            params: vec![
                Param {
                    name: "a".to_string(),
                    ty: Some(TypeExpr::Simple("Int".to_string())),
                    default: None,
                    span: dummy_span(),
                },
                Param {
                    name: "b".to_string(),
                    ty: Some(TypeExpr::Simple("Int".to_string())),
                    default: Some(Expr::Integer(0, dummy_span())),
                    span: dummy_span(),
                },
            ],
            return_type: Some(TypeExpr::Simple("Int".to_string())),
            body: Expr::Binary(
                Box::new(Expr::Ident("a".to_string(), dummy_span())),
                BinaryOp::Add,
                Box::new(Expr::Ident("b".to_string(), dummy_span())),
                dummy_span(),
            ),
            is_pub: true,
            span: dummy_span(),
        };
        assert_eq!(func.name, "add");
        assert_eq!(func.params.len(), 2);
        assert!(func.is_pub);
    }

    // === TypeExpr Tests ===

    #[test]
    fn test_type_expr_variants() {
        let simple = TypeExpr::Simple("Int".to_string());
        assert!(matches!(simple, TypeExpr::Simple(_)));

        let array = TypeExpr::Array(Box::new(TypeExpr::Simple("Int".to_string())));
        assert!(matches!(array, TypeExpr::Array(_)));

        let map = TypeExpr::Map(
            Box::new(TypeExpr::Simple("String".to_string())),
            Box::new(TypeExpr::Simple("Int".to_string())),
        );
        assert!(matches!(map, TypeExpr::Map(_, _)));

        let tuple = TypeExpr::Tuple(vec![
            TypeExpr::Simple("Int".to_string()),
            TypeExpr::Simple("String".to_string()),
        ]);
        assert!(matches!(tuple, TypeExpr::Tuple(_)));

        let optional = TypeExpr::Optional(Box::new(TypeExpr::Simple("Int".to_string())));
        assert!(matches!(optional, TypeExpr::Optional(_)));

        let result = TypeExpr::Result(Box::new(TypeExpr::Simple("Int".to_string())));
        assert!(matches!(result, TypeExpr::Result(_)));

        let func = TypeExpr::Function(
            vec![TypeExpr::Simple("Int".to_string())],
            Box::new(TypeExpr::Simple("Int".to_string())),
        );
        assert!(matches!(func, TypeExpr::Function(_, _)));

        let struct_ty = TypeExpr::Struct(vec![
            ("x".to_string(), TypeExpr::Simple("Int".to_string())),
        ]);
        assert!(matches!(struct_ty, TypeExpr::Struct(_)));
    }

    // === FfiType Tests ===

    #[test]
    fn test_ffi_type_to_c_type() {
        assert_eq!(FfiType::Void.to_c_type(), "void");
        assert_eq!(FfiType::Int(8).to_c_type(), "int8_t");
        assert_eq!(FfiType::Int(16).to_c_type(), "int16_t");
        assert_eq!(FfiType::Int(32).to_c_type(), "int32_t");
        assert_eq!(FfiType::Int(64).to_c_type(), "int64_t");
        assert_eq!(FfiType::Uint(8).to_c_type(), "uint8_t");
        assert_eq!(FfiType::Uint(16).to_c_type(), "uint16_t");
        assert_eq!(FfiType::Uint(32).to_c_type(), "uint32_t");
        assert_eq!(FfiType::Uint(64).to_c_type(), "uint64_t");
        assert_eq!(FfiType::F32.to_c_type(), "float");
        assert_eq!(FfiType::F64.to_c_type(), "double");
        assert_eq!(FfiType::Bool.to_c_type(), "bool");
        assert_eq!(FfiType::CStr.to_c_type(), "const char*");
        assert_eq!(FfiType::Opaque.to_c_type(), "void*");
        assert_eq!(FfiType::Ptr(Box::new(FfiType::Int(32))).to_c_type(), "const void*");
        assert_eq!(FfiType::MutPtr(Box::new(FfiType::Int(32))).to_c_type(), "void*");
    }

    #[test]
    fn test_ffi_type_equality() {
        assert_eq!(FfiType::Void, FfiType::Void);
        assert_eq!(FfiType::Int(32), FfiType::Int(32));
        assert_ne!(FfiType::Int(32), FfiType::Int(64));
        assert_ne!(FfiType::Int(32), FfiType::Uint(32));
    }

    // === Expr Tests ===

    #[test]
    fn test_expr_span() {
        let span = Span::new(10, 20);

        assert_eq!(Expr::Integer(42, span).span(), span);
        assert_eq!(Expr::Float(3.14, span).span(), span);
        assert_eq!(Expr::String("test".to_string(), span).span(), span);
        assert_eq!(Expr::Bool(true, span).span(), span);
        assert_eq!(Expr::Nil(span).span(), span);
        assert_eq!(Expr::Ident("x".to_string(), span).span(), span);
        assert_eq!(Expr::LambdaParam(span).span(), span);
        assert_eq!(Expr::Array(vec![], span).span(), span);
        assert_eq!(Expr::Map(vec![], span).span(), span);
        assert_eq!(Expr::Tuple(vec![], span).span(), span);
        assert_eq!(Expr::Block(vec![], span).span(), span);
        assert_eq!(Expr::SelfCall(vec![], span).span(), span);
    }

    #[test]
    fn test_expr_literals() {
        assert!(matches!(Expr::Integer(42, dummy_span()), Expr::Integer(42, _)));
        assert!(matches!(Expr::Float(3.14, dummy_span()), Expr::Float(f, _) if (f - 3.14).abs() < f64::EPSILON));
        assert!(matches!(Expr::String("hello".to_string(), dummy_span()), Expr::String(s, _) if s == "hello"));
        assert!(matches!(Expr::Bool(true, dummy_span()), Expr::Bool(true, _)));
        assert!(matches!(Expr::Nil(dummy_span()), Expr::Nil(_)));
    }

    #[test]
    fn test_expr_binary() {
        let left = Box::new(Expr::Integer(1, dummy_span()));
        let right = Box::new(Expr::Integer(2, dummy_span()));
        let binary = Expr::Binary(left, BinaryOp::Add, right, dummy_span());
        assert!(matches!(binary, Expr::Binary(_, BinaryOp::Add, _, _)));
    }

    #[test]
    fn test_expr_unary() {
        let inner = Box::new(Expr::Integer(5, dummy_span()));
        let unary = Expr::Unary(UnaryOp::Neg, inner, dummy_span());
        assert!(matches!(unary, Expr::Unary(UnaryOp::Neg, _, _)));
    }

    #[test]
    fn test_expr_collection_ops() {
        let arr = Box::new(Expr::Ident("arr".to_string(), dummy_span()));
        let transform = Box::new(Expr::LambdaParam(dummy_span()));

        let map_op = Expr::MapOp(arr.clone(), transform.clone(), dummy_span());
        assert!(matches!(map_op, Expr::MapOp(_, _, _)));

        let filter_op = Expr::FilterOp(arr.clone(), transform.clone(), dummy_span());
        assert!(matches!(filter_op, Expr::FilterOp(_, _, _)));

        let reduce_op = Expr::ReduceOp(arr, ReduceKind::Sum, dummy_span());
        assert!(matches!(reduce_op, Expr::ReduceOp(_, ReduceKind::Sum, _)));
    }

    #[test]
    fn test_expr_control_flow() {
        let cond = Box::new(Expr::Bool(true, dummy_span()));
        let then_expr = Box::new(Expr::Integer(1, dummy_span()));
        let else_expr = Box::new(Expr::Integer(0, dummy_span()));

        let ternary = Expr::Ternary(cond.clone(), then_expr.clone(), else_expr.clone(), dummy_span());
        assert!(matches!(ternary, Expr::Ternary(_, _, _, _)));

        let if_expr = Expr::If(cond.clone(), then_expr.clone(), Some(else_expr.clone()), dummy_span());
        assert!(matches!(if_expr, Expr::If(_, _, Some(_), _)));

        let if_without_else = Expr::If(cond, then_expr, None, dummy_span());
        assert!(matches!(if_without_else, Expr::If(_, _, None, _)));
    }

    #[test]
    fn test_expr_function_call() {
        let func = Box::new(Expr::Ident("my_func".to_string(), dummy_span()));
        let args = vec![Expr::Integer(1, dummy_span()), Expr::Integer(2, dummy_span())];
        let call = Expr::Call(func, args, dummy_span());
        assert!(matches!(call, Expr::Call(_, args, _) if args.len() == 2));
    }

    #[test]
    fn test_expr_lambda() {
        let params = vec!["x".to_string(), "y".to_string()];
        let body = Box::new(Expr::Binary(
            Box::new(Expr::Ident("x".to_string(), dummy_span())),
            BinaryOp::Add,
            Box::new(Expr::Ident("y".to_string(), dummy_span())),
            dummy_span(),
        ));
        let lambda = Expr::Lambda(params.clone(), body, dummy_span());
        assert!(matches!(lambda, Expr::Lambda(p, _, _) if p.len() == 2));
    }

    // === BinaryOp Tests ===

    #[test]
    fn test_binary_op_precedence() {
        assert!(BinaryOp::Mul.precedence() > BinaryOp::Add.precedence());
        assert!(BinaryOp::Add.precedence() > BinaryOp::Lt.precedence());
        assert!(BinaryOp::Lt.precedence() > BinaryOp::And.precedence());
        assert!(BinaryOp::And.precedence() > BinaryOp::Or.precedence());
    }

    #[test]
    fn test_binary_op_same_precedence() {
        assert_eq!(BinaryOp::Add.precedence(), BinaryOp::Sub.precedence());
        assert_eq!(BinaryOp::Mul.precedence(), BinaryOp::Div.precedence());
        assert_eq!(BinaryOp::Eq.precedence(), BinaryOp::NotEq.precedence());
    }

    #[test]
    fn test_binary_op_left_associative() {
        assert!(BinaryOp::Add.is_left_associative());
        assert!(BinaryOp::Mul.is_left_associative());
        assert!(BinaryOp::And.is_left_associative());
    }

    // === UnaryOp Tests ===

    #[test]
    fn test_unary_op_variants() {
        assert!(matches!(UnaryOp::Neg, UnaryOp::Neg));
        assert!(matches!(UnaryOp::Not, UnaryOp::Not));
        assert!(matches!(UnaryOp::Len, UnaryOp::Len));
    }

    // === ReduceKind Tests ===

    #[test]
    fn test_reduce_kind_variants() {
        assert!(matches!(ReduceKind::Sum, ReduceKind::Sum));
        assert!(matches!(ReduceKind::Product, ReduceKind::Product));
        assert!(matches!(ReduceKind::Min, ReduceKind::Min));
        assert!(matches!(ReduceKind::Max, ReduceKind::Max));
        assert!(matches!(ReduceKind::And, ReduceKind::And));
        assert!(matches!(ReduceKind::Or, ReduceKind::Or));

        let init = Box::new(Expr::Integer(0, dummy_span()));
        let func = Box::new(Expr::LambdaParam(dummy_span()));
        assert!(matches!(ReduceKind::Custom(init, func), ReduceKind::Custom(_, _)));
    }

    // === IndexKind Tests ===

    #[test]
    fn test_index_kind_single() {
        let idx = IndexKind::Single(Expr::Integer(0, dummy_span()));
        assert!(matches!(idx, IndexKind::Single(_)));
    }

    #[test]
    fn test_index_kind_slice() {
        let full_slice = IndexKind::Slice(
            Some(Expr::Integer(0, dummy_span())),
            Some(Expr::Integer(10, dummy_span())),
        );
        assert!(matches!(full_slice, IndexKind::Slice(Some(_), Some(_))));

        let from_start = IndexKind::Slice(None, Some(Expr::Integer(5, dummy_span())));
        assert!(matches!(from_start, IndexKind::Slice(None, Some(_))));

        let to_end = IndexKind::Slice(Some(Expr::Integer(5, dummy_span())), None);
        assert!(matches!(to_end, IndexKind::Slice(Some(_), None)));

        let full = IndexKind::Slice(None, None);
        assert!(matches!(full, IndexKind::Slice(None, None)));
    }

    // === Pattern Tests ===

    #[test]
    fn test_pattern_variants() {
        assert!(matches!(Pattern::Wildcard(dummy_span()), Pattern::Wildcard(_)));
        assert!(matches!(Pattern::Literal(Expr::Integer(1, dummy_span())), Pattern::Literal(_)));
        assert!(matches!(Pattern::Binding("x".to_string(), dummy_span()), Pattern::Binding(_, _)));

        let struct_pat = Pattern::Struct(vec![
            ("x".to_string(), None),
            ("y".to_string(), Some(Pattern::Wildcard(dummy_span()))),
        ], dummy_span());
        assert!(matches!(struct_pat, Pattern::Struct(_, _)));

        let tuple_pat = Pattern::Tuple(vec![
            Pattern::Binding("a".to_string(), dummy_span()),
            Pattern::Wildcard(dummy_span()),
        ], dummy_span());
        assert!(matches!(tuple_pat, Pattern::Tuple(_, _)));

        let array_pat = Pattern::Array(vec![
            Pattern::Literal(Expr::Integer(1, dummy_span())),
        ], dummy_span());
        assert!(matches!(array_pat, Pattern::Array(_, _)));

        let variant_none = Pattern::Variant("None".to_string(), None, dummy_span());
        assert!(matches!(variant_none, Pattern::Variant(_, None, _)));

        let variant_some = Pattern::Variant(
            "Some".to_string(),
            Some(Box::new(Pattern::Binding("x".to_string(), dummy_span()))),
            dummy_span(),
        );
        assert!(matches!(variant_some, Pattern::Variant(_, Some(_), _)));
    }

    // === MatchArm Tests ===

    #[test]
    fn test_match_arm() {
        let arm = MatchArm {
            pattern: Pattern::Wildcard(dummy_span()),
            guard: None,
            body: Expr::Integer(42, dummy_span()),
            span: dummy_span(),
        };
        assert!(matches!(arm.pattern, Pattern::Wildcard(_)));
        assert!(arm.guard.is_none());
    }

    #[test]
    fn test_match_arm_with_guard() {
        let arm = MatchArm {
            pattern: Pattern::Binding("x".to_string(), dummy_span()),
            guard: Some(Expr::Binary(
                Box::new(Expr::Ident("x".to_string(), dummy_span())),
                BinaryOp::Gt,
                Box::new(Expr::Integer(0, dummy_span())),
                dummy_span(),
            )),
            body: Expr::String("positive".to_string(), dummy_span()),
            span: dummy_span(),
        };
        assert!(arm.guard.is_some());
    }

    // === Serialization Tests ===

    #[test]
    fn test_expr_serialize_deserialize() {
        let expr = Expr::Binary(
            Box::new(Expr::Integer(1, dummy_span())),
            BinaryOp::Add,
            Box::new(Expr::Integer(2, dummy_span())),
            dummy_span(),
        );

        let json = serde_json::to_string(&expr).unwrap();
        let deserialized: Expr = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, Expr::Binary(_, BinaryOp::Add, _, _)));
    }

    #[test]
    fn test_program_serialize_deserialize() {
        let program = Program {
            items: vec![Item::Expr(Expr::Integer(42, dummy_span()))],
            span: dummy_span(),
        };

        let json = serde_json::to_string(&program).unwrap();
        let deserialized: Program = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.items.len(), 1);
    }

    #[test]
    fn test_type_expr_serialize_deserialize() {
        let ty = TypeExpr::Function(
            vec![TypeExpr::Simple("Int".to_string())],
            Box::new(TypeExpr::Simple("Bool".to_string())),
        );

        let json = serde_json::to_string(&ty).unwrap();
        let deserialized: TypeExpr = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, TypeExpr::Function(_, _)));
    }

    #[test]
    fn test_ffi_type_serialize_deserialize() {
        let ffi_ty = FfiType::Ptr(Box::new(FfiType::Int(32)));

        let json = serde_json::to_string(&ffi_ty).unwrap();
        let deserialized: FfiType = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, FfiType::Ptr(_)));
    }

    // === Complex Structure Tests ===

    #[test]
    fn test_complex_function() {
        // fib(n: Int): Int = n < 2 ? n : $(n-1) + $(n-2)
        let func = FunctionDef {
            name: "fib".to_string(),
            params: vec![Param {
                name: "n".to_string(),
                ty: Some(TypeExpr::Simple("Int".to_string())),
                default: None,
                span: dummy_span(),
            }],
            return_type: Some(TypeExpr::Simple("Int".to_string())),
            body: Expr::Ternary(
                Box::new(Expr::Binary(
                    Box::new(Expr::Ident("n".to_string(), dummy_span())),
                    BinaryOp::Lt,
                    Box::new(Expr::Integer(2, dummy_span())),
                    dummy_span(),
                )),
                Box::new(Expr::Ident("n".to_string(), dummy_span())),
                Box::new(Expr::Binary(
                    Box::new(Expr::SelfCall(
                        vec![Expr::Binary(
                            Box::new(Expr::Ident("n".to_string(), dummy_span())),
                            BinaryOp::Sub,
                            Box::new(Expr::Integer(1, dummy_span())),
                            dummy_span(),
                        )],
                        dummy_span(),
                    )),
                    BinaryOp::Add,
                    Box::new(Expr::SelfCall(
                        vec![Expr::Binary(
                            Box::new(Expr::Ident("n".to_string(), dummy_span())),
                            BinaryOp::Sub,
                            Box::new(Expr::Integer(2, dummy_span())),
                            dummy_span(),
                        )],
                        dummy_span(),
                    )),
                    dummy_span(),
                )),
                dummy_span(),
            ),
            is_pub: false,
            span: dummy_span(),
        };

        assert_eq!(func.name, "fib");
        assert_eq!(func.params.len(), 1);
        assert!(func.return_type.is_some());
    }

    #[test]
    fn test_complex_match() {
        // match x {
        //   0 => "zero",
        //   n if n > 0 => "positive",
        //   _ => "negative"
        // }
        let match_expr = Expr::Match(
            Box::new(Expr::Ident("x".to_string(), dummy_span())),
            vec![
                MatchArm {
                    pattern: Pattern::Literal(Expr::Integer(0, dummy_span())),
                    guard: None,
                    body: Expr::String("zero".to_string(), dummy_span()),
                    span: dummy_span(),
                },
                MatchArm {
                    pattern: Pattern::Binding("n".to_string(), dummy_span()),
                    guard: Some(Expr::Binary(
                        Box::new(Expr::Ident("n".to_string(), dummy_span())),
                        BinaryOp::Gt,
                        Box::new(Expr::Integer(0, dummy_span())),
                        dummy_span(),
                    )),
                    body: Expr::String("positive".to_string(), dummy_span()),
                    span: dummy_span(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard(dummy_span()),
                    guard: None,
                    body: Expr::String("negative".to_string(), dummy_span()),
                    span: dummy_span(),
                },
            ],
            dummy_span(),
        );

        if let Expr::Match(_, arms, _) = match_expr {
            assert_eq!(arms.len(), 3);
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_ffi_block() {
        let ffi = FfiBlock {
            lib_name: "libc".to_string(),
            abi: "C".to_string(),
            functions: vec![
                FfiFn {
                    name: "abs".to_string(),
                    extern_name: None,
                    params: vec![("x".to_string(), FfiType::Int(32))],
                    return_type: FfiType::Int(32),
                    span: dummy_span(),
                },
                FfiFn {
                    name: "sqrt".to_string(),
                    extern_name: None,
                    params: vec![("x".to_string(), FfiType::F64)],
                    return_type: FfiType::F64,
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };

        assert_eq!(ffi.lib_name, "libc");
        assert_eq!(ffi.abi, "C");
        assert_eq!(ffi.functions.len(), 2);
    }
}
