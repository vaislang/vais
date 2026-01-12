//! AOEL AOEL Type Checker
//!
//! Hindley-Milner 기반 타입 추론 및 검사

mod types;
mod infer;
mod checker;
mod error;

pub use types::Type;
pub use infer::TypeEnv;
pub use checker::check;
pub use error::{TypeError, TypeResult};
