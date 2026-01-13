//! Cranelift JIT 컴파일러
//!
//! AOEL IR을 Cranelift IR로 변환하고 네이티브 코드를 생성.

use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use aoel_ir::{Instruction, OpCode, Value};
use aoel_lowering::CompiledFunction;

use crate::error::{JitError, JitResult};

/// Value 타입을 나타내는 태그
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueTag {
    Nil = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    String = 4,
    Array = 5,
    Struct = 6,
    Closure = 7,
}

/// JIT 컴파일된 함수 시그니처 (Int 전용 최적화 경로)
/// 인자: i64 배열 포인터, 인자 개수
/// 반환: i64
pub type JittedFnInt = unsafe extern "C" fn(*const i64, usize) -> i64;

/// JIT 컴파일된 함수 시그니처 (Float 전용)
pub type JittedFnFloat = unsafe extern "C" fn(*const f64, usize) -> f64;

/// JIT 컴파일된 함수 시그니처 (일반, tagged union)
/// 인자: (tag, value) 쌍의 배열 포인터, 인자 개수
/// 반환: (tag, value)
pub type JittedFnGeneric = unsafe extern "C" fn(*const u8, usize) -> u64;

/// 컴파일된 함수 정보
#[derive(Debug)]
pub struct CompiledFn {
    pub name: String,
    pub ptr: *const u8,
    pub signature: FnSignature,
}

/// 함수 시그니처 타입
#[derive(Debug, Clone)]
pub enum FnSignature {
    /// Int -> Int (최적화 경로)
    IntOnly { param_count: usize },
    /// Float -> Float (최적화 경로)
    FloatOnly { param_count: usize },
    /// 일반 (런타임 타입 체크)
    Generic { param_count: usize },
}

/// JIT 컴파일러
pub struct JitCompiler {
    /// Cranelift JIT 모듈
    module: JITModule,
    /// 컴파일된 함수 캐시
    compiled_functions: HashMap<String, CompiledFn>,
}

impl JitCompiler {
    /// 새 JIT 컴파일러 생성
    pub fn new() -> JitResult<Self> {
        let mut flag_builder = settings::builder();
        flag_builder.set("opt_level", "speed").map_err(|e| JitError::Internal(e.to_string()))?;
        flag_builder.set("is_pic", "false").map_err(|e| JitError::Internal(e.to_string()))?;

        let isa_builder = cranelift_native::builder()
            .map_err(|e| JitError::Internal(e.to_string()))?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| JitError::Internal(e.to_string()))?;

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Ok(Self {
            module,
            compiled_functions: HashMap::new(),
        })
    }

    /// 함수를 JIT 컴파일 (Int 전용 최적화)
    pub fn compile_function_int(&mut self, func: &CompiledFunction) -> JitResult<*const u8> {
        let name = &func.name;

        // 이미 컴파일된 경우 캐시에서 반환
        if let Some(compiled) = self.compiled_functions.get(name) {
            return Ok(compiled.ptr);
        }

        // 함수 시그니처 정의: (i64*, usize) -> i64
        let ptr_type = self.module.target_config().pointer_type();
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(ptr_type)); // args pointer
        sig.params.push(AbiParam::new(types::I64)); // arg count
        sig.returns.push(AbiParam::new(types::I64)); // return value

        // 함수 선언
        let func_id = self.module
            .declare_function(name, Linkage::Local, &sig)?;

        // 컨텍스트 생성
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32());

        // 함수 빌더 컨텍스트
        let mut func_ctx = FunctionBuilderContext::new();

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

            // 엔트리 블록 생성
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            // 인자 추출
            let args_ptr = builder.block_params(entry_block)[0];
            let _arg_count = builder.block_params(entry_block)[1];

            // 로컬 변수 맵 (파라미터 -> SSA value)
            let mut locals: HashMap<String, cranelift::prelude::Value> = HashMap::new();

            // 파라미터를 로컬 변수로 로드
            for (i, param) in func.params.iter().enumerate() {
                let offset = (i * 8) as i32;
                let val = builder.ins().load(types::I64, MemFlags::trusted(), args_ptr, offset);
                locals.insert(param.clone(), val);
            }

            // 스택 시뮬레이션 (Cranelift SSA values)
            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();

            // 명령어 컴파일
            Self::compile_instructions_int(&mut builder, &func.instructions, &mut locals, &mut stack)?;

            // 결과 반환
            let result = stack.pop().ok_or_else(|| JitError::CodeGen("Empty stack at return".to_string()))?;
            builder.ins().return_(&[result]);

            builder.finalize();
        }

        // 컴파일 및 함수 포인터 획득
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| JitError::CodeGen(e.to_string()))?;

        self.module.clear_context(&mut ctx);
        self.module.finalize_definitions()
            .map_err(|e| JitError::Module(e.to_string()))?;

        let code_ptr = self.module.get_finalized_function(func_id);

        // 캐시에 저장
        self.compiled_functions.insert(name.clone(), CompiledFn {
            name: name.clone(),
            ptr: code_ptr,
            signature: FnSignature::IntOnly { param_count: func.params.len() },
        });

        Ok(code_ptr)
    }

    /// Int 전용 명령어 컴파일
    fn compile_instructions_int(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
    ) -> JitResult<()> {
        let mut ip = 0;

        while ip < instructions.len() {
            let instr = &instructions[ip];

            match &instr.opcode {
                // === 상수 ===
                OpCode::Const(Value::Int(n)) => {
                    let val = builder.ins().iconst(types::I64, *n);
                    stack.push(val);
                }
                OpCode::Const(Value::Bool(b)) => {
                    let val = builder.ins().iconst(types::I64, if *b { 1 } else { 0 });
                    stack.push(val);
                }

                // === 변수 ===
                OpCode::Load(name) => {
                    let val = locals.get(name)
                        .ok_or_else(|| JitError::CodeGen(format!("Undefined variable: {}", name)))?;
                    stack.push(*val);
                }
                OpCode::Store(name) => {
                    let val = stack.pop()
                        .ok_or_else(|| JitError::CodeGen("Stack underflow at Store".to_string()))?;
                    locals.insert(name.clone(), val);
                }

                // === 산술 연산 (Int) ===
                OpCode::Add => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let result = builder.ins().iadd(a, b);
                    stack.push(result);
                }
                OpCode::Sub => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let result = builder.ins().isub(a, b);
                    stack.push(result);
                }
                OpCode::Mul => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let result = builder.ins().imul(a, b);
                    stack.push(result);
                }
                OpCode::Div => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let result = builder.ins().sdiv(a, b);
                    stack.push(result);
                }
                OpCode::Mod => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let result = builder.ins().srem(a, b);
                    stack.push(result);
                }
                OpCode::Neg => {
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let result = builder.ins().ineg(a);
                    stack.push(result);
                }

                // === 비교 연산 ===
                OpCode::Lt => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let cmp = builder.ins().icmp(IntCC::SignedLessThan, a, b);
                    let result = builder.ins().uextend(types::I64, cmp);
                    stack.push(result);
                }
                OpCode::Gt => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, a, b);
                    let result = builder.ins().uextend(types::I64, cmp);
                    stack.push(result);
                }
                OpCode::Lte => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, a, b);
                    let result = builder.ins().uextend(types::I64, cmp);
                    stack.push(result);
                }
                OpCode::Gte => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, a, b);
                    let result = builder.ins().uextend(types::I64, cmp);
                    stack.push(result);
                }
                OpCode::Eq => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let cmp = builder.ins().icmp(IntCC::Equal, a, b);
                    let result = builder.ins().uextend(types::I64, cmp);
                    stack.push(result);
                }
                OpCode::Neq => {
                    let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                    let cmp = builder.ins().icmp(IntCC::NotEqual, a, b);
                    let result = builder.ins().uextend(types::I64, cmp);
                    stack.push(result);
                }

                // === 스택 연산 ===
                OpCode::Dup => {
                    let val = stack.last()
                        .ok_or_else(|| JitError::CodeGen("Stack underflow at Dup".to_string()))?;
                    stack.push(*val);
                }
                OpCode::Pop => {
                    stack.pop();
                }

                // === 제어 흐름 ===
                // Note: 복잡한 제어 흐름은 별도 블록 생성 필요
                // 간단한 경우만 처리 (ternary)
                OpCode::Return => {
                    // 반환은 외부에서 처리
                    break;
                }

                // 지원하지 않는 opcode는 에러
                _ => {
                    return Err(JitError::UnsupportedOpcode(format!("{:?}", instr.opcode)));
                }
            }

            ip += 1;
        }

        Ok(())
    }

    /// 컴파일된 함수 호출 (Int 전용)
    pub unsafe fn call_int(&self, name: &str, args: &[i64]) -> JitResult<i64> {
        let compiled = self.compiled_functions.get(name)
            .ok_or_else(|| JitError::FunctionNotFound(name.to_string()))?;

        let func: JittedFnInt = std::mem::transmute(compiled.ptr);
        Ok(func(args.as_ptr(), args.len()))
    }

    /// 함수가 JIT 컴파일 가능한지 확인
    pub fn can_jit(func: &CompiledFunction) -> bool {
        Self::analyze_function(func).is_ok()
    }

    /// 함수 분석 (JIT 가능 여부 및 최적화 타입 결정)
    fn analyze_function(func: &CompiledFunction) -> JitResult<FnSignature> {
        // 모든 명령어가 Int 전용 경로로 처리 가능한지 확인
        for instr in &func.instructions {
            match &instr.opcode {
                // 지원되는 명령어
                OpCode::Const(Value::Int(_)) |
                OpCode::Const(Value::Bool(_)) |
                OpCode::Load(_) |
                OpCode::Store(_) |
                OpCode::Add |
                OpCode::Sub |
                OpCode::Mul |
                OpCode::Div |
                OpCode::Mod |
                OpCode::Neg |
                OpCode::Lt |
                OpCode::Gt |
                OpCode::Lte |
                OpCode::Gte |
                OpCode::Eq |
                OpCode::Neq |
                OpCode::Dup |
                OpCode::Pop |
                OpCode::Return => {}

                // 지원되지 않는 명령어
                _ => {
                    return Err(JitError::UnsupportedOpcode(format!("{:?}", instr.opcode)));
                }
            }
        }

        Ok(FnSignature::IntOnly { param_count: func.params.len() })
    }

    /// 컴파일된 함수 목록 반환
    pub fn get_compiled_functions(&self) -> Vec<&str> {
        self.compiled_functions.keys().map(|s| s.as_str()).collect()
    }

    /// 특정 함수의 컴파일된 포인터 반환
    pub fn get_compiled_ptr(&self, name: &str) -> Option<*const u8> {
        self.compiled_functions.get(name).map(|f| f.ptr)
    }
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new().expect("Failed to create JIT compiler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoel_ir::Instruction;

    fn make_instruction(opcode: OpCode) -> Instruction {
        Instruction::new(opcode)
    }

    #[test]
    fn test_simple_add() {
        let mut jit = JitCompiler::new().unwrap();

        // add(a, b) = a + b
        let func = CompiledFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Return),
            ],
        };

        let _ptr = jit.compile_function_int(&func).unwrap();

        unsafe {
            let result = jit.call_int("add", &[3, 5]).unwrap();
            assert_eq!(result, 8);

            let result = jit.call_int("add", &[100, 200]).unwrap();
            assert_eq!(result, 300);
        }
    }

    #[test]
    fn test_arithmetic() {
        let mut jit = JitCompiler::new().unwrap();

        // calc(a, b) = (a + b) * (a - b)
        let func = CompiledFunction {
            name: "calc".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Sub),
                make_instruction(OpCode::Mul),
                make_instruction(OpCode::Return),
            ],
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            // (5 + 3) * (5 - 3) = 8 * 2 = 16
            let result = jit.call_int("calc", &[5, 3]).unwrap();
            assert_eq!(result, 16);
        }
    }
}
