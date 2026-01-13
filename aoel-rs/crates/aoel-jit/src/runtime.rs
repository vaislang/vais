//! JIT 런타임
//!
//! JIT 컴파일된 함수와 인터프리터 사이의 통합 레이어.

use std::collections::HashMap;

use aoel_ir::Value;
use aoel_lowering::CompiledFunction;

use crate::compiler::{JitCompiler, JittedFnInt};
use crate::error::{JitError, JitResult};
use crate::profiler::{ExecutionProfiler, JIT_THRESHOLD};

/// 컴파일된 코드 정보
#[derive(Debug)]
pub struct CompiledCode {
    /// 함수 포인터
    pub ptr: *const u8,
    /// 파라미터 개수
    pub param_count: usize,
    /// Int 전용 최적화 여부
    pub is_int_specialized: bool,
}

/// JIT 런타임
///
/// 프로파일링 기반 적응형 컴파일을 수행.
/// 핫 함수를 자동으로 감지하고 JIT 컴파일.
pub struct JitRuntime {
    /// JIT 컴파일러
    compiler: JitCompiler,
    /// 프로파일러
    profiler: ExecutionProfiler,
    /// 함수 정의 (원본)
    functions: HashMap<String, CompiledFunction>,
    /// JIT 컴파일된 함수 캐시
    compiled_cache: HashMap<String, CompiledCode>,
    /// JIT 활성화 여부
    jit_enabled: bool,
    /// 자동 JIT 컴파일 활성화 여부
    auto_jit: bool,
}

impl JitRuntime {
    /// 새 JIT 런타임 생성
    pub fn new() -> JitResult<Self> {
        Ok(Self {
            compiler: JitCompiler::new()?,
            profiler: ExecutionProfiler::new(),
            functions: HashMap::new(),
            compiled_cache: HashMap::new(),
            jit_enabled: true,
            auto_jit: true,
        })
    }

    /// JIT 활성화/비활성화
    pub fn set_jit_enabled(&mut self, enabled: bool) {
        self.jit_enabled = enabled;
    }

    /// 자동 JIT 활성화/비활성화
    pub fn set_auto_jit(&mut self, enabled: bool) {
        self.auto_jit = enabled;
    }

    /// 프로파일링 활성화/비활성화
    pub fn set_profiling_enabled(&mut self, enabled: bool) {
        self.profiler.set_enabled(enabled);
    }

    /// 함수들 로드
    pub fn load_functions(&mut self, functions: Vec<CompiledFunction>) {
        for func in functions {
            self.functions.insert(func.name.clone(), func);
        }
    }

    /// 함수 실행 (JIT 또는 인터프리터 자동 선택)
    pub fn execute(&mut self, name: &str, args: Vec<Value>) -> JitResult<ExecuteResult> {
        // 타입 시그니처 생성
        let type_sig = args.iter()
            .map(|v| match v {
                Value::Int(_) => "Int",
                Value::Float(_) => "Float",
                Value::Bool(_) => "Bool",
                Value::String(_) => "String",
                Value::Array(_) => "Array",
                _ => "Other",
            })
            .collect::<Vec<_>>()
            .join(",");

        // 프로파일 기록 시작
        self.profiler.begin_call(name, &type_sig);

        // JIT 컴파일된 함수가 있는지 확인
        if self.jit_enabled {
            if let Some(compiled) = self.compiled_cache.get(name) {
                if compiled.is_int_specialized && Self::all_ints(&args) {
                    // Int 전용 JIT 경로
                    let int_args: Vec<i64> = args.iter()
                        .filter_map(|v| match v {
                            Value::Int(n) => Some(*n),
                            _ => None,
                        })
                        .collect();

                    unsafe {
                        let func: JittedFnInt = std::mem::transmute(compiled.ptr);
                        let result = func(int_args.as_ptr(), int_args.len());
                        self.profiler.end_call(name);
                        return Ok(ExecuteResult::Jit(Value::Int(result)));
                    }
                }
            }
        }

        // 자동 JIT: 핫 함수 감지 및 컴파일
        if self.auto_jit && self.jit_enabled {
            if let Some(profile) = self.profiler.get_profile(name) {
                if profile.call_count >= JIT_THRESHOLD && !profile.is_jitted {
                    // Int 전용으로 특화 가능한지 확인
                    if profile.dominant_type_pattern() == Some("Int") ||
                       profile.dominant_type_pattern().map(|p| p.chars().all(|c| c == ',' || c == 'I' || c == 'n' || c == 't')).unwrap_or(false) {
                        if let Some(func) = self.functions.get(name) {
                            if JitCompiler::can_jit(func) {
                                match self.compile_function(name) {
                                    Ok(_) => {
                                        self.profiler.mark_jitted(name);
                                    }
                                    Err(e) => {
                                        // JIT 실패 시 조용히 인터프리터로 폴백
                                        eprintln!("JIT compilation failed for {}: {:?}", name, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.profiler.end_call(name);

        // 인터프리터로 폴백
        Ok(ExecuteResult::Interpret)
    }

    /// 함수를 명시적으로 JIT 컴파일
    pub fn compile_function(&mut self, name: &str) -> JitResult<()> {
        let func = self.functions.get(name)
            .ok_or_else(|| JitError::FunctionNotFound(name.to_string()))?
            .clone();

        let ptr = self.compiler.compile_function_int(&func)?;

        self.compiled_cache.insert(name.to_string(), CompiledCode {
            ptr,
            param_count: func.params.len(),
            is_int_specialized: true,
        });

        self.profiler.mark_jitted(name);
        Ok(())
    }

    /// 모든 인자가 Int인지 확인
    fn all_ints(args: &[Value]) -> bool {
        args.iter().all(|v| matches!(v, Value::Int(_)))
    }

    /// JIT 컴파일된 함수 직접 호출 (Int 전용)
    ///
    /// # Safety
    ///
    /// - 함수가 JIT 컴파일되어 있어야 함 (`is_jitted(name)`이 true)
    /// - `args` 배열의 길이가 함수의 파라미터 개수와 일치해야 함
    /// - 함수가 정수 전용으로 컴파일되어 있어야 함
    pub unsafe fn call_jit_int(&self, name: &str, args: &[i64]) -> JitResult<i64> {
        let compiled = self.compiled_cache.get(name)
            .ok_or_else(|| JitError::FunctionNotFound(name.to_string()))?;

        let func: JittedFnInt = std::mem::transmute(compiled.ptr);
        Ok(func(args.as_ptr(), args.len()))
    }

    /// 함수가 JIT 컴파일되었는지 확인
    pub fn is_jitted(&self, name: &str) -> bool {
        self.compiled_cache.contains_key(name)
    }

    /// 프로파일 통계 출력
    pub fn print_profile_stats(&self) {
        self.profiler.print_stats();
    }

    /// 프로파일러 참조 반환
    pub fn profiler(&self) -> &ExecutionProfiler {
        &self.profiler
    }

    /// 프로파일러 가변 참조 반환
    pub fn profiler_mut(&mut self) -> &mut ExecutionProfiler {
        &mut self.profiler
    }

    /// 컴파일된 함수 목록 반환
    pub fn get_jitted_functions(&self) -> Vec<&str> {
        self.compiled_cache.keys().map(|s| s.as_str()).collect()
    }
}

/// 실행 결과
#[derive(Debug)]
pub enum ExecuteResult {
    /// JIT 실행 결과
    Jit(Value),
    /// 인터프리터로 폴백 필요
    Interpret,
}

impl Default for JitRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create JIT runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoel_ir::{Instruction, OpCode};

    fn make_add_function() -> CompiledFunction {
        CompiledFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                Instruction::new(OpCode::Load("a".to_string())),
                Instruction::new(OpCode::Load("b".to_string())),
                Instruction::new(OpCode::Add),
                Instruction::new(OpCode::Return),
            ],
        }
    }

    #[test]
    fn test_runtime_basic() {
        let mut runtime = JitRuntime::new().unwrap();
        runtime.load_functions(vec![make_add_function()]);

        // 명시적 JIT 컴파일
        runtime.compile_function("add").unwrap();
        assert!(runtime.is_jitted("add"));

        // JIT 실행
        unsafe {
            let result = runtime.call_jit_int("add", &[10, 20]).unwrap();
            assert_eq!(result, 30);
        }
    }

    #[test]
    fn test_auto_jit() {
        let mut runtime = JitRuntime::new().unwrap();
        runtime.load_functions(vec![make_add_function()]);

        // 임계값 이하 호출 - JIT 안됨
        for _ in 0..50 {
            let _ = runtime.execute("add", vec![Value::Int(1), Value::Int(2)]);
        }
        assert!(!runtime.is_jitted("add"));

        // 임계값 초과 호출 - 자동 JIT
        for _ in 0..60 {
            let _ = runtime.execute("add", vec![Value::Int(1), Value::Int(2)]);
        }
        assert!(runtime.is_jitted("add"));
    }
}
