//! JIT 지원 VM
//!
//! 프로파일링 기반 적응형 JIT 컴파일을 지원하는 VM.
//! 핫 함수를 자동으로 감지하고 네이티브 코드로 컴파일.

use std::collections::HashMap;

use aoel_ir::Value;
use aoel_jit::{JitCompiler, ExecutionProfiler, JIT_THRESHOLD, JittedFnInt};
use aoel_lowering::CompiledFunction;

use crate::error::{RuntimeError, RuntimeResult};
use crate::vm::Vm;

/// JIT 설정
#[derive(Debug, Clone)]
pub struct JitConfig {
    /// JIT 활성화 여부
    pub enabled: bool,
    /// 자동 JIT 컴파일 활성화 여부
    pub auto_jit: bool,
    /// 프로파일링 활성화 여부
    pub profiling: bool,
    /// JIT 컴파일 임계값 (호출 횟수)
    pub threshold: u64,
}

impl Default for JitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_jit: true,
            profiling: true,
            threshold: JIT_THRESHOLD,
        }
    }
}

/// JIT 지원 VM
///
/// 인터프리터와 JIT 컴파일러를 통합하여 최적의 성능을 제공.
pub struct JitVm {
    /// 인터프리터 VM
    interpreter: Vm,
    /// JIT 컴파일러
    jit: Option<JitCompiler>,
    /// 프로파일러
    profiler: ExecutionProfiler,
    /// 함수 정의 (원본)
    functions: HashMap<String, CompiledFunction>,
    /// JIT 컴파일된 함수 목록
    jitted_functions: HashMap<String, JittedFunction>,
    /// JIT 설정
    config: JitConfig,
}

/// JIT 컴파일된 함수 정보
struct JittedFunction {
    /// 함수 포인터
    ptr: *const u8,
    /// 파라미터 개수
    param_count: usize,
}

impl JitVm {
    /// 새 JIT VM 생성
    pub fn new() -> Self {
        Self::with_config(JitConfig::default())
    }

    /// 설정과 함께 JIT VM 생성
    pub fn with_config(config: JitConfig) -> Self {
        let jit = if config.enabled {
            JitCompiler::new().ok()
        } else {
            None
        };

        Self {
            interpreter: Vm::new(),
            jit,
            profiler: ExecutionProfiler::new(),
            functions: HashMap::new(),
            jitted_functions: HashMap::new(),
            config,
        }
    }

    /// 함수들 로드
    pub fn load_functions(&mut self, functions: Vec<CompiledFunction>) {
        for func in functions.clone() {
            self.functions.insert(func.name.clone(), func);
        }
        self.interpreter.load_functions(functions);
    }

    /// 함수 호출 (JIT 또는 인터프리터 자동 선택)
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> RuntimeResult<Value> {
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
        if self.config.profiling {
            self.profiler.begin_call(name, &type_sig);
        }

        // JIT 컴파일된 함수가 있고 모든 인자가 Int인지 확인
        if self.config.enabled {
            if let Some(jitted) = self.jitted_functions.get(name) {
                if Self::all_ints(&args) {
                    // Int 전용 JIT 경로
                    let int_args: Vec<i64> = args.iter()
                        .filter_map(|v| match v {
                            Value::Int(n) => Some(*n),
                            _ => None,
                        })
                        .collect();

                    if int_args.len() == jitted.param_count {
                        unsafe {
                            let func: JittedFnInt = std::mem::transmute(jitted.ptr);
                            let result = func(int_args.as_ptr(), int_args.len());

                            if self.config.profiling {
                                self.profiler.end_call(name);
                            }

                            return Ok(Value::Int(result));
                        }
                    }
                }
            }
        }

        // 자동 JIT: 핫 함수 감지 및 컴파일
        if self.config.auto_jit && self.config.enabled && self.jit.is_some() {
            if let Some(profile) = self.profiler.get_profile(name) {
                if profile.call_count >= self.config.threshold && !profile.is_jitted {
                    // Int 전용으로 특화 가능한지 확인
                    if let Some(pattern) = profile.dominant_type_pattern() {
                        let is_int_only = pattern.split(',').all(|t| t == "Int" || t.is_empty());
                        if is_int_only {
                            if let Some(func) = self.functions.get(name).cloned() {
                                if JitCompiler::can_jit(&func) {
                                    match self.compile_function_internal(&func) {
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
        }

        // 인터프리터로 실행
        let result = self.interpreter.call_function(name, args);

        if self.config.profiling {
            self.profiler.end_call(name);
        }

        result
    }

    /// 함수를 명시적으로 JIT 컴파일
    pub fn compile_function(&mut self, name: &str) -> RuntimeResult<()> {
        let func = self.functions.get(name)
            .ok_or_else(|| RuntimeError::UndefinedFunction(name.to_string()))?
            .clone();

        self.compile_function_internal(&func)
            .map_err(|e| RuntimeError::Internal(format!("JIT error: {:?}", e)))?;

        self.profiler.mark_jitted(name);
        Ok(())
    }

    /// 내부 JIT 컴파일
    fn compile_function_internal(&mut self, func: &CompiledFunction) -> Result<(), aoel_jit::JitError> {
        let jit = self.jit.as_mut()
            .ok_or_else(|| aoel_jit::JitError::Internal("JIT not available".to_string()))?;

        let ptr = jit.compile_function_int(func)?;

        self.jitted_functions.insert(func.name.clone(), JittedFunction {
            ptr,
            param_count: func.params.len(),
        });

        Ok(())
    }

    /// 모든 인자가 Int인지 확인
    fn all_ints(args: &[Value]) -> bool {
        args.iter().all(|v| matches!(v, Value::Int(_)))
    }

    /// 함수가 JIT 컴파일되었는지 확인
    pub fn is_jitted(&self, name: &str) -> bool {
        self.jitted_functions.contains_key(name)
    }

    /// 프로파일 통계 출력
    pub fn print_profile_stats(&self) {
        self.profiler.print_stats();
    }

    /// JIT 통계 출력
    pub fn print_jit_stats(&self) {
        println!("\n=== JIT Statistics ===");
        println!("JIT enabled: {}", self.config.enabled);
        println!("Auto JIT: {}", self.config.auto_jit);
        println!("JIT threshold: {} calls", self.config.threshold);
        println!("JIT compiled functions: {}", self.jitted_functions.len());

        if !self.jitted_functions.is_empty() {
            println!("\nJIT compiled:");
            for name in self.jitted_functions.keys() {
                println!("  - {}", name);
            }
        }
    }

    /// 인터프리터 VM에 대한 참조 반환
    pub fn interpreter(&self) -> &Vm {
        &self.interpreter
    }

    /// 인터프리터 VM에 대한 가변 참조 반환
    pub fn interpreter_mut(&mut self) -> &mut Vm {
        &mut self.interpreter
    }

    /// 프로파일러 참조 반환
    pub fn profiler(&self) -> &ExecutionProfiler {
        &self.profiler
    }

    /// JIT 설정 반환
    pub fn config(&self) -> &JitConfig {
        &self.config
    }

    /// JIT 설정 수정
    pub fn set_config(&mut self, config: JitConfig) {
        self.config = config;
    }
}

impl Default for JitVm {
    fn default() -> Self {
        Self::new()
    }
}

/// JIT VM으로 프로그램 실행
pub fn execute_with_jit(functions: Vec<CompiledFunction>) -> RuntimeResult<Value> {
    let mut vm = JitVm::new();
    vm.load_functions(functions);

    // main 함수 실행
    vm.call_function("main", vec![])
}

/// JIT VM으로 특정 함수 실행
pub fn execute_function_with_jit(
    functions: Vec<CompiledFunction>,
    func_name: &str,
    args: Vec<Value>,
) -> RuntimeResult<Value> {
    let mut vm = JitVm::new();
    vm.load_functions(functions);
    vm.call_function(func_name, args)
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
    fn test_jit_vm_basic() {
        let mut vm = JitVm::new();
        vm.load_functions(vec![make_add_function()]);

        // 인터프리터 실행
        let result = vm.call_function("add", vec![Value::Int(3), Value::Int(5)]).unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_jit_vm_explicit_compile() {
        let mut vm = JitVm::new();
        vm.load_functions(vec![make_add_function()]);

        // 명시적 JIT 컴파일
        vm.compile_function("add").unwrap();
        assert!(vm.is_jitted("add"));

        // JIT 실행
        let result = vm.call_function("add", vec![Value::Int(10), Value::Int(20)]).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_jit_vm_auto_compile() {
        let mut vm = JitVm::with_config(JitConfig {
            enabled: true,
            auto_jit: true,
            profiling: true,
            threshold: 50, // 낮은 임계값
        });
        vm.load_functions(vec![make_add_function()]);

        // 임계값 이하 호출
        for _ in 0..40 {
            let _ = vm.call_function("add", vec![Value::Int(1), Value::Int(2)]);
        }
        assert!(!vm.is_jitted("add"));

        // 임계값 초과 호출 - 자동 JIT
        for _ in 0..20 {
            let _ = vm.call_function("add", vec![Value::Int(1), Value::Int(2)]);
        }
        assert!(vm.is_jitted("add"));

        // JIT 실행 확인
        let result = vm.call_function("add", vec![Value::Int(100), Value::Int(200)]).unwrap();
        assert_eq!(result, Value::Int(300));
    }
}
