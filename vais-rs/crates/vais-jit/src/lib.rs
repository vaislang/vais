//! Vais JIT Compiler
//!
//! Cranelift 기반 JIT 컴파일러로 Vais IR을 네이티브 코드로 변환.
//! 핫 경로 감지 및 타입 특화를 통해 인터프리터 대비 5-20배 성능 향상.
//!
//! ## 최적화 기능
//!
//! - **SIMD 벡터화**: 배열 연산의 SIMD 가속 (AVX2/SSE4.2/NEON)
//! - **추측적 최적화**: 타입 프로파일링 기반 특화 코드 생성
//! - **인라인 캐싱**: 함수 호출 지점 캐싱으로 간접 호출 제거
//! - **다단계 JIT**: Interpreter → Baseline → Optimized 점진적 컴파일
//! - **OSR**: 실행 중 핫 루프 최적화 전환

// JIT 컴파일에서 ip 인덱스 기반 점프가 필요하므로 needless_range_loop 허용
#![allow(clippy::needless_range_loop)]
// JIT 함수는 컴파일 상태와 컨텍스트가 복잡하여 많은 인수 허용
#![allow(clippy::too_many_arguments)]

pub mod compiler;
mod error;
mod profiler;
mod runtime;

// 고급 최적화 모듈
pub mod inline_cache;
pub mod osr;
pub mod simd;
pub mod speculative;
pub mod tiered;

pub use compiler::{JitCompiler, JittedFnInt, FnSignature};
pub use error::{JitError, JitResult};
pub use profiler::{ExecutionProfiler, FunctionProfile, JIT_THRESHOLD};
pub use runtime::{JitRuntime, CompiledCode};

// 고급 최적화 re-exports
pub use inline_cache::{ICManager, ICState, InlineCache, DispatchTable};
pub use osr::{OsrManager, OsrPoint, OsrFrame, OsrDecision};
pub use simd::SimdCompiler;
pub use speculative::{SpeculativeContext, TypeGuard, TypeProfile, SpecType, DeoptFrame, DeoptReason};
pub use tiered::{TieredManager, CompilationTier, FunctionTierInfo};
