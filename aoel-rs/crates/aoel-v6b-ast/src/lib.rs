//! AOEL v6b Abstract Syntax Tree
//!
//! v6b 문법에 최적화된 AST 노드 정의.
//!
//! # 주요 노드
//!
//! - `Program` - 최상위 노드
//! - `FunctionDef` - 함수 정의
//! - `Expr` - 표현식 (모든 것이 표현식)
//!
//! # 예제
//!
//! ```ignore
//! // add(a,b)=a+b 파싱 결과
//! FunctionDef {
//!     name: "add",
//!     params: [Param { name: "a" }, Param { name: "b" }],
//!     body: Binary(Ident("a"), Add, Ident("b")),
//! }
//! ```

pub mod ast;

pub use ast::*;
