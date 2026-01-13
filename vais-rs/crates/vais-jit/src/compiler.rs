//! Cranelift JIT 컴파일러
//!
//! Vais IR을 Cranelift IR로 변환하고 네이티브 코드를 생성.

use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_codegen::ir::FuncRef;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

use vais_ir::{Instruction, OpCode, Value};
use vais_lowering::CompiledFunction;

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
    /// 선언된 함수 ID 맵 (배치 컴파일용)
    declared_functions: HashMap<String, FuncId>,
    /// 함수 시그니처 캐시
    func_signatures: HashMap<String, (cranelift_codegen::ir::Signature, usize)>, // (sig, param_count)
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
            declared_functions: HashMap::new(),
            func_signatures: HashMap::new(),
        })
    }

    /// 함수를 JIT 컴파일 (Int 전용 최적화)
    pub fn compile_function_int(&mut self, func: &CompiledFunction) -> JitResult<*const u8> {
        let name = &func.name;

        // 이미 컴파일된 경우 캐시에서 반환
        if let Some(compiled) = self.compiled_functions.get(name) {
            return Ok(compiled.ptr);
        }

        // 재귀 호출이 있는지 확인
        let has_self_call = func.instructions.iter().any(|i| {
            matches!(i.opcode, OpCode::SelfCall(_) | OpCode::TailSelfCall(_))
        });

        if has_self_call {
            self.compile_recursive_function_int(func)
        } else {
            self.compile_simple_function_int(func)
        }
    }

    /// 단순 함수 컴파일 (재귀 없음)
    fn compile_simple_function_int(&mut self, func: &CompiledFunction) -> JitResult<*const u8> {
        let name = &func.name;

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
            // seal은 compile_instructions_int에서 처리

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

            // 제어 흐름이 있는지 확인
            let has_control_flow = func.instructions.iter().any(|i| {
                matches!(i.opcode, OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_))
            });

            if has_control_flow {
                // 제어 흐름이 있으면 블록 기반 컴파일
                Self::compile_instructions_with_control_flow(&mut builder, &func.instructions, &mut locals, &mut stack)?;
            } else {
                // 선형 코드 - 단순 컴파일
                builder.seal_block(entry_block);
                Self::compile_linear_instructions(&mut builder, &func.instructions, &mut locals, &mut stack)?;

                // 결과 반환
                let result = stack.pop().ok_or_else(|| JitError::CodeGen("Empty stack at return".to_string()))?;
                builder.ins().return_(&[result]);
            }

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

    /// 재귀 함수 컴파일 (SelfCall/TailSelfCall 지원)
    fn compile_recursive_function_int(&mut self, func: &CompiledFunction) -> JitResult<*const u8> {
        let name = &func.name;
        let param_count = func.params.len();

        // 함수 시그니처 정의: (i64*, usize) -> i64
        let ptr_type = self.module.target_config().pointer_type();
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(ptr_type)); // args pointer
        sig.params.push(AbiParam::new(types::I64)); // arg count
        sig.returns.push(AbiParam::new(types::I64)); // return value

        // 함수 선언
        let func_id = self.module
            .declare_function(name, Linkage::Local, &sig)?;

        // 함수 참조 (자기 자신 호출용) - 아래에서 다시 선언하므로 여기서는 사용하지 않음
        let _func_ref = self.module.declare_func_in_func(func_id, &mut self.module.make_context().func);

        // 컨텍스트 생성
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig.clone();
        ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32());

        // 함수 참조를 컨텍스트에 다시 선언
        let self_func_ref = self.module.declare_func_in_func(func_id, &mut ctx.func);

        // 함수 빌더 컨텍스트
        let mut func_ctx = FunctionBuilderContext::new();

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

            // === TCO를 위한 구조 ===
            // entry_block: 초기 파라미터 로드
            // loop_header: 루프 시작점 (TailSelfCall이 점프하는 곳)
            // body_block: 실제 함수 본문
            // return_block: 반환

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);

            // 루프 헤더 블록 - 파라미터 개수만큼 블록 파라미터 추가
            let loop_header = builder.create_block();
            for _ in 0..param_count {
                builder.append_block_param(loop_header, types::I64);
            }

            let return_block = builder.create_block();

            // 결과 변수
            let result_var = Variable::new(0);
            builder.declare_var(result_var, types::I64);

            // 파라미터 변수들 (SSA가 아닌 Cranelift 변수)
            let mut param_vars: Vec<Variable> = Vec::new();
            for i in 0..param_count {
                let var = Variable::new(i + 1);
                builder.declare_var(var, types::I64);
                param_vars.push(var);
            }

            // === Entry Block ===
            builder.switch_to_block(entry_block);
            let args_ptr = builder.block_params(entry_block)[0];
            let _arg_count = builder.block_params(entry_block)[1];

            // 초기 파라미터 로드
            let mut initial_params: Vec<cranelift::prelude::Value> = Vec::new();
            for i in 0..param_count {
                let offset = (i * 8) as i32;
                let val = builder.ins().load(types::I64, MemFlags::trusted(), args_ptr, offset);
                initial_params.push(val);
            }

            // 기본 result 초기화
            let zero = builder.ins().iconst(types::I64, 0);
            builder.def_var(result_var, zero);

            // 루프 헤더로 점프 (초기 파라미터 전달)
            builder.ins().jump(loop_header, &initial_params);
            builder.seal_block(entry_block);

            // === Loop Header Block ===
            builder.switch_to_block(loop_header);

            // 블록 파라미터에서 현재 파라미터 값 가져오기
            let loop_params = builder.block_params(loop_header).to_vec();
            for (i, &val) in loop_params.iter().enumerate() {
                builder.def_var(param_vars[i], val);
            }

            // 함수 본문 컴파일
            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();
            let mut locals: HashMap<String, cranelift::prelude::Value> = HashMap::new();

            // 파라미터를 로컬에 등록
            for (i, param_name) in func.params.iter().enumerate() {
                locals.insert(param_name.clone(), loop_params[i]);
            }

            // 제어 흐름 분석
            let has_control_flow = func.instructions.iter().any(|i| {
                matches!(i.opcode, OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_))
            });

            if has_control_flow {
                // 제어 흐름이 있는 재귀 함수 컴파일
                // loop_header와 return_block의 seal은 이 함수 내에서 처리됨
                Self::compile_recursive_with_control_flow(
                    &mut builder,
                    &func.instructions,
                    &mut locals,
                    &mut stack,
                    &param_vars,
                    &func.params,
                    loop_header,
                    return_block,
                    result_var,
                    self_func_ref,
                    ptr_type,
                )?;
            } else {
                // 선형 재귀 함수 컴파일
                Self::compile_recursive_linear(
                    &mut builder,
                    &func.instructions,
                    &mut locals,
                    &mut stack,
                    &param_vars,
                    &func.params,
                    loop_header,
                    return_block,
                    result_var,
                    self_func_ref,
                    ptr_type,
                )?;
                // 선형 코드의 경우 여기서 seal
                builder.seal_block(loop_header);
            }

            builder.seal_block(return_block);

            // === Return Block ===
            builder.switch_to_block(return_block);
            let result = builder.use_var(result_var);
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
            signature: FnSignature::IntOnly { param_count },
        });

        Ok(code_ptr)
    }

    /// 선형 재귀 함수 컴파일 (제어 흐름 없음)
    #[allow(clippy::too_many_arguments)]
    fn compile_recursive_linear(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
        _param_vars: &[Variable],
        _param_names: &[String],
        loop_header: Block,
        return_block: Block,
        result_var: Variable,
        self_func_ref: FuncRef,
        ptr_type: Type,
    ) -> JitResult<()> {
        for instr in instructions {
            match &instr.opcode {
                OpCode::Return => {
                    let result = stack.pop()
                        .ok_or_else(|| JitError::CodeGen("Stack underflow at Return".to_string()))?;
                    builder.def_var(result_var, result);
                    builder.ins().jump(return_block, &[]);
                    return Ok(());
                }
                OpCode::TailSelfCall(arg_count) => {
                    // TCO: 새 파라미터로 loop_header로 점프
                    let mut new_args: Vec<cranelift::prelude::Value> = Vec::new();
                    for _ in 0..*arg_count {
                        new_args.push(stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at TailSelfCall".to_string()))?);
                    }
                    new_args.reverse();
                    builder.ins().jump(loop_header, &new_args);
                    return Ok(());
                }
                OpCode::SelfCall(arg_count) => {
                    // 일반 재귀: 스택에 인자 배열 만들고 자기 자신 호출
                    Self::compile_self_call(builder, stack, *arg_count, self_func_ref, ptr_type)?;
                }
                _ => {
                    Self::compile_single_instruction(builder, &instr.opcode, locals, stack)?;
                }
            }
        }
        Ok(())
    }

    /// 제어 흐름이 있는 재귀 함수 컴파일
    #[allow(clippy::too_many_arguments)]
    fn compile_recursive_with_control_flow(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        _stack: &mut Vec<cranelift::prelude::Value>,
        _param_vars: &[Variable],
        _param_names: &[String],
        loop_header: Block,
        return_block: Block,
        result_var: Variable,
        self_func_ref: FuncRef,
        ptr_type: Type,
    ) -> JitResult<()> {
        // 1단계: 제어 흐름 분석 - 블록 시작점과 점프 타겟 수집
        let mut block_starts: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        block_starts.insert(0);

        for (ip, instr) in instructions.iter().enumerate() {
            match &instr.opcode {
                OpCode::Jump(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if target < instructions.len() {
                        block_starts.insert(target);
                    }
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                OpCode::JumpIf(offset) | OpCode::JumpIfNot(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if target < instructions.len() {
                        block_starts.insert(target);
                    }
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                OpCode::Return | OpCode::TailSelfCall(_) => {
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                _ => {}
            }
        }

        // 2단계: 블록 ID 매핑 생성
        let block_starts_vec: Vec<usize> = block_starts.iter().cloned().collect();
        let mut ip_to_block: HashMap<usize, usize> = HashMap::new();
        for (block_idx, &start) in block_starts_vec.iter().enumerate() {
            let end = block_starts_vec.get(block_idx + 1).cloned().unwrap_or(instructions.len());
            for ip in start..end {
                ip_to_block.insert(ip, block_idx);
            }
        }

        // 3단계: Cranelift 블록 생성 (첫 블록은 loop_header를 사용)
        let current_block = builder.current_block()
            .ok_or_else(|| JitError::Internal("No current block in builder".to_string()))?;
        let mut blocks: Vec<Block> = Vec::new();
        blocks.push(current_block); // 블록 0은 현재 블록 (loop_header 내부)

        for _ in 1..block_starts_vec.len() {
            let block = builder.create_block();
            blocks.push(block);
        }

        // 4단계: 각 블록 컴파일
        for (block_idx, &start) in block_starts_vec.iter().enumerate() {
            let end = block_starts_vec.get(block_idx + 1).cloned().unwrap_or(instructions.len());
            let block = blocks[block_idx];

            if block_idx > 0 {
                builder.switch_to_block(block);
            }

            // 블록별 로컬 스택
            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();

            let mut terminated = false;
            for ip in start..end {
                let instr = &instructions[ip];

                match &instr.opcode {
                    OpCode::Jump(offset) => {
                        if let Some(val) = stack.last() {
                            builder.def_var(result_var, *val);
                        }
                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];
                        builder.ins().jump(target_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIf(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIf".to_string()))?;
                        let zero = builder.ins().iconst(types::I64, 0);
                        let cmp = builder.ins().icmp(IntCC::NotEqual, cond, zero);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIfNot(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIfNot".to_string()))?;
                        let zero = builder.ins().iconst(types::I64, 0);
                        let cmp = builder.ins().icmp(IntCC::Equal, cond, zero);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::Return => {
                        if let Some(result) = stack.pop() {
                            builder.def_var(result_var, result);
                        }
                        builder.ins().jump(return_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::TailSelfCall(arg_count) => {
                        // TCO: 새 파라미터로 loop_header로 점프
                        let mut new_args: Vec<cranelift::prelude::Value> = Vec::new();
                        for _ in 0..*arg_count {
                            new_args.push(stack.pop()
                                .ok_or_else(|| JitError::CodeGen("Stack underflow at TailSelfCall".to_string()))?);
                        }
                        new_args.reverse();
                        builder.ins().jump(loop_header, &new_args);
                        terminated = true;
                        break;
                    }
                    OpCode::SelfCall(arg_count) => {
                        Self::compile_self_call(builder, &mut stack, *arg_count, self_func_ref, ptr_type)?;
                    }
                    _ => {
                        Self::compile_single_instruction(builder, &instr.opcode, locals, &mut stack)?;
                    }
                }
            }

            // 블록 끝에 값이 있으면 result_var에 저장하고 다음 블록으로
            if !terminated {
                if let Some(val) = stack.last() {
                    builder.def_var(result_var, *val);
                }
                if block_idx + 1 < blocks.len() {
                    builder.ins().jump(blocks[block_idx + 1], &[]);
                } else {
                    builder.ins().jump(return_block, &[]);
                }
            }
        }

        // 5단계: 모든 블록 seal
        // blocks[0]는 loop_header이므로 따로 처리 (TailSelfCall이 점프해올 수 있음)
        // loop_header는 호출자가 seal함
        for (i, block) in blocks.iter().enumerate() {
            if i > 0 {
                builder.seal_block(*block);
            }
        }
        // loop_header도 여기서 seal (모든 TailSelfCall 점프가 추가된 후)
        builder.seal_block(loop_header);

        Ok(())
    }

    /// SelfCall 컴파일: 자기 자신을 재귀 호출
    fn compile_self_call(
        builder: &mut FunctionBuilder,
        stack: &mut Vec<cranelift::prelude::Value>,
        arg_count: usize,
        self_func_ref: FuncRef,
        ptr_type: Type,
    ) -> JitResult<()> {
        // 스택에서 인자 pop
        let mut args: Vec<cranelift::prelude::Value> = Vec::new();
        for _ in 0..arg_count {
            args.push(stack.pop()
                .ok_or_else(|| JitError::CodeGen("Stack underflow at SelfCall".to_string()))?);
        }
        args.reverse();

        // 스택에 인자 배열 할당 (stack_slot 사용)
        let slot_size = (arg_count * 8) as u32;
        let slot = builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, slot_size, 0));
        let slot_addr = builder.ins().stack_addr(ptr_type, slot, 0);

        // 인자를 스택 슬롯에 저장
        for (i, arg) in args.iter().enumerate() {
            let offset = (i * 8) as i32;
            builder.ins().store(MemFlags::trusted(), *arg, slot_addr, offset);
        }

        // 자기 자신 호출
        let arg_count_val = builder.ins().iconst(types::I64, arg_count as i64);
        let call = builder.ins().call(self_func_ref, &[slot_addr, arg_count_val]);
        let result = builder.inst_results(call)[0];
        stack.push(result);

        Ok(())
    }

    /// 선형 명령어 컴파일 (제어 흐름 없음)
    fn compile_linear_instructions(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
    ) -> JitResult<()> {
        for instr in instructions {
            if matches!(instr.opcode, OpCode::Return) {
                break;
            }
            Self::compile_single_instruction(builder, &instr.opcode, locals, stack)?;
        }
        Ok(())
    }

    /// 제어 흐름이 있는 명령어 컴파일 (블록 기반 + SSA 변수)
    fn compile_instructions_with_control_flow(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        _stack: &mut Vec<cranelift::prelude::Value>,
    ) -> JitResult<()> {
        // 1단계: 제어 흐름 분석 - 블록 시작점과 점프 타겟 수집
        let mut block_starts: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        block_starts.insert(0);

        for (ip, instr) in instructions.iter().enumerate() {
            match &instr.opcode {
                OpCode::Jump(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if target < instructions.len() {
                        block_starts.insert(target);
                    }
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                OpCode::JumpIf(offset) | OpCode::JumpIfNot(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if target < instructions.len() {
                        block_starts.insert(target);
                    }
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                OpCode::Return => {
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                _ => {}
            }
        }

        // 2단계: 블록 ID 매핑 생성
        let block_starts_vec: Vec<usize> = block_starts.iter().cloned().collect();
        let mut ip_to_block: HashMap<usize, usize> = HashMap::new();
        for (block_idx, &start) in block_starts_vec.iter().enumerate() {
            let end = block_starts_vec.get(block_idx + 1).cloned().unwrap_or(instructions.len());
            for ip in start..end {
                ip_to_block.insert(ip, block_idx);
            }
        }

        // 3단계: Cranelift 블록 생성
        let entry_block = builder.current_block()
            .ok_or_else(|| JitError::Internal("No current block in builder".to_string()))?;
        let mut blocks: Vec<Block> = Vec::new();
        blocks.push(entry_block);

        for _ in 1..block_starts_vec.len() {
            let block = builder.create_block();
            blocks.push(block);
        }

        // 결과를 위한 Cranelift 변수 생성
        let result_var = Variable::new(0);
        builder.declare_var(result_var, types::I64);
        // 기본값 0으로 초기화
        let zero = builder.ins().iconst(types::I64, 0);
        builder.def_var(result_var, zero);

        // Return 블록
        let return_block = builder.create_block();

        // 4단계: 각 블록 컴파일
        for (block_idx, &start) in block_starts_vec.iter().enumerate() {
            let end = block_starts_vec.get(block_idx + 1).cloned().unwrap_or(instructions.len());
            let block = blocks[block_idx];

            if block_idx > 0 {
                builder.switch_to_block(block);
            }

            // 블록별 로컬 스택
            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();

            let mut terminated = false;
            for ip in start..end {
                let instr = &instructions[ip];

                match &instr.opcode {
                    OpCode::Jump(offset) => {
                        // 스택에 값이 있으면 result_var에 저장
                        if let Some(val) = stack.last() {
                            builder.def_var(result_var, *val);
                        }
                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {} (from ip {})", target_ip, ip)))?;
                        let target_block = blocks[*target_block_idx];
                        builder.ins().jump(target_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIf(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIf".to_string()))?;
                        let zero = builder.ins().iconst(types::I64, 0);
                        let cmp = builder.ins().icmp(IntCC::NotEqual, cond, zero);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIfNot(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIfNot".to_string()))?;
                        let zero = builder.ins().iconst(types::I64, 0);
                        let cmp = builder.ins().icmp(IntCC::Equal, cond, zero);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::Return => {
                        // Return이 단독 블록에 있을 때 스택이 비어있을 수 있음
                        // 이 경우 이전 블록에서 result_var에 저장된 값을 사용
                        if let Some(result) = stack.pop() {
                            builder.def_var(result_var, result);
                        }
                        // result_var의 현재 값으로 반환
                        builder.ins().jump(return_block, &[]);
                        terminated = true;
                        break;
                    }
                    _ => {
                        Self::compile_single_instruction(builder, &instr.opcode, locals, &mut stack)?;
                    }
                }
            }

            // 블록 끝에 값이 있으면 result_var에 저장하고 다음 블록으로
            if !terminated {
                if let Some(val) = stack.last() {
                    builder.def_var(result_var, *val);
                }
                if block_idx + 1 < blocks.len() {
                    builder.ins().jump(blocks[block_idx + 1], &[]);
                } else {
                    builder.ins().jump(return_block, &[]);
                }
            }
        }

        // 5단계: 모든 블록 seal
        for block in &blocks {
            builder.seal_block(*block);
        }
        builder.seal_block(return_block);

        // 6단계: Return 블록
        builder.switch_to_block(return_block);
        let result = builder.use_var(result_var);
        builder.ins().return_(&[result]);

        Ok(())
    }

    /// 단일 명령어 컴파일
    fn compile_single_instruction(
        builder: &mut FunctionBuilder,
        opcode: &OpCode,
        _locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
    ) -> JitResult<()> {
        match opcode {
            // === 상수 ===
            OpCode::Const(Value::Int(n)) => {
                let val = builder.ins().iconst(types::I64, *n);
                stack.push(val);
            }
            OpCode::Const(Value::Bool(b)) => {
                let val = builder.ins().iconst(types::I64, if *b { 1 } else { 0 });
                stack.push(val);
            }

            // === 변수 (선형 모드용) ===
            OpCode::Load(name) => {
                let val = _locals.get(name)
                    .ok_or_else(|| JitError::CodeGen(format!("Undefined variable: {}", name)))?;
                stack.push(*val);
            }
            OpCode::Store(name) => {
                let val = stack.pop()
                    .ok_or_else(|| JitError::CodeGen("Stack underflow at Store".to_string()))?;
                _locals.insert(name.clone(), val);
            }

            // === 로컬 변수 (인덱스 기반, 최적화) ===
            OpCode::LoadLocal(idx) => {
                // LoadLocal은 파라미터 인덱스로 접근
                // args 배열에서 직접 로드
                let param_name = format!("__param_{}", idx);
                let val = _locals.get(&param_name)
                    .ok_or_else(|| JitError::CodeGen(format!("Undefined local: {}", idx)))?;
                stack.push(*val);
            }
            OpCode::StoreLocal(idx) => {
                let val = stack.pop()
                    .ok_or_else(|| JitError::CodeGen("Stack underflow at StoreLocal".to_string()))?;
                let param_name = format!("__param_{}", idx);
                _locals.insert(param_name, val);
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

            // === 논리 연산 ===
            OpCode::And => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let result = builder.ins().band(a, b);
                stack.push(result);
            }
            OpCode::Or => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let result = builder.ins().bor(a, b);
                stack.push(result);
            }
            OpCode::Not => {
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let zero = builder.ins().iconst(types::I64, 0);
                let cmp = builder.ins().icmp(IntCC::Equal, a, zero);
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
            OpCode::Nop => {}

            // 제어 흐름은 별도 처리
            OpCode::Return | OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_) => {}

            // 지원하지 않는 opcode는 에러
            _ => {
                return Err(JitError::UnsupportedOpcode(format!("{:?}", opcode)));
            }
        }
        Ok(())
    }

    /// 컴파일된 함수 호출 (Int 전용)
    ///
    /// # Safety
    /// - 함수가 `compile_function_int`로 컴파일되어 있어야 함
    /// - `args` 배열의 길이가 함수의 파라미터 개수와 일치해야 함
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
                // 지원되는 명령어 - 상수
                OpCode::Const(Value::Int(_)) |
                OpCode::Const(Value::Bool(_)) |
                // 변수
                OpCode::Load(_) |
                OpCode::Store(_) |
                OpCode::LoadLocal(_) |
                OpCode::StoreLocal(_) |
                // 산술
                OpCode::Add |
                OpCode::Sub |
                OpCode::Mul |
                OpCode::Div |
                OpCode::Mod |
                OpCode::Neg |
                // 비교
                OpCode::Lt |
                OpCode::Gt |
                OpCode::Lte |
                OpCode::Gte |
                OpCode::Eq |
                OpCode::Neq |
                // 논리
                OpCode::And |
                OpCode::Or |
                OpCode::Not |
                // 스택
                OpCode::Dup |
                OpCode::Pop |
                OpCode::Nop |
                // 제어 흐름
                OpCode::Jump(_) |
                OpCode::JumpIf(_) |
                OpCode::JumpIfNot(_) |
                OpCode::Return |
                // 재귀 호출
                OpCode::SelfCall(_) |
                OpCode::TailSelfCall(_) |
                // 함수 호출
                OpCode::Call(_, _) => {}

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

    /// 여러 함수를 배치로 컴파일 (함수 간 호출 지원)
    ///
    /// 이 메서드는 모든 함수를 먼저 선언한 후 정의하므로,
    /// 함수 간 상호 호출이 가능합니다.
    pub fn compile_functions_batch(&mut self, functions: &[CompiledFunction]) -> JitResult<()> {
        // 1단계: 모든 함수 선언 (시그니처만)
        let ptr_type = self.module.target_config().pointer_type();

        for func in functions {
            if self.compiled_functions.contains_key(&func.name) {
                continue; // 이미 컴파일됨
            }

            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(ptr_type)); // args pointer
            sig.params.push(AbiParam::new(types::I64)); // arg count
            sig.returns.push(AbiParam::new(types::I64)); // return value

            let func_id = self.module
                .declare_function(&func.name, Linkage::Local, &sig)?;

            self.declared_functions.insert(func.name.clone(), func_id);
            self.func_signatures.insert(func.name.clone(), (sig, func.params.len()));
        }

        // 2단계: 모든 함수 정의
        for func in functions {
            if self.compiled_functions.contains_key(&func.name) {
                continue;
            }

            self.compile_function_with_calls(func)?;
        }

        // 3단계: 모든 정의 finalize
        self.module.finalize_definitions()
            .map_err(|e| JitError::Module(e.to_string()))?;

        // 4단계: 함수 포인터 수집
        for func in functions {
            if self.compiled_functions.contains_key(&func.name) {
                continue;
            }

            let func_id = self.declared_functions.get(&func.name)
                .ok_or_else(|| JitError::FunctionNotFound(func.name.clone()))?;

            let code_ptr = self.module.get_finalized_function(*func_id);

            self.compiled_functions.insert(func.name.clone(), CompiledFn {
                name: func.name.clone(),
                ptr: code_ptr,
                signature: FnSignature::IntOnly { param_count: func.params.len() },
            });
        }

        Ok(())
    }

    /// Call opcode를 지원하는 함수 컴파일 (내부용)
    fn compile_function_with_calls(&mut self, func: &CompiledFunction) -> JitResult<()> {
        let name = &func.name;
        let _param_count = func.params.len();

        let func_id = *self.declared_functions.get(name)
            .ok_or_else(|| JitError::FunctionNotFound(name.clone()))?;

        let (sig, _) = self.func_signatures.get(name)
            .ok_or_else(|| JitError::FunctionNotFound(name.clone()))?
            .clone();

        let ptr_type = self.module.target_config().pointer_type();

        // 컨텍스트 생성
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32());

        // 호출할 수 있는 함수들의 FuncRef 맵 생성
        let mut func_refs: HashMap<String, FuncRef> = HashMap::new();
        for (fn_name, &fn_id) in &self.declared_functions {
            let func_ref = self.module.declare_func_in_func(fn_id, &mut ctx.func);
            func_refs.insert(fn_name.clone(), func_ref);
        }

        // 자기 자신에 대한 참조
        let self_func_ref = *func_refs.get(name)
            .ok_or_else(|| JitError::Internal("Self reference not found".to_string()))?;

        // 함수 빌더 컨텍스트
        let mut func_ctx = FunctionBuilderContext::new();

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

            // 재귀 호출 또는 다른 함수 호출이 있는지 확인
            let has_self_call = func.instructions.iter().any(|i| {
                matches!(i.opcode, OpCode::SelfCall(_) | OpCode::TailSelfCall(_))
            });

            let has_other_call = func.instructions.iter().any(|i| {
                matches!(i.opcode, OpCode::Call(_, _))
            });

            if has_self_call {
                // 재귀 함수 컴파일
                self.compile_recursive_with_calls_internal(
                    &mut builder,
                    func,
                    self_func_ref,
                    &func_refs,
                    ptr_type,
                )?;
            } else if has_other_call {
                // 다른 함수 호출이 있는 함수 컴파일
                self.compile_with_calls_internal(
                    &mut builder,
                    func,
                    &func_refs,
                    ptr_type,
                )?;
            } else {
                // 단순 함수 컴파일
                self.compile_simple_internal(&mut builder, func, ptr_type)?;
            }

            builder.finalize();
        }

        // 함수 정의
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| JitError::CodeGen(e.to_string()))?;

        self.module.clear_context(&mut ctx);

        Ok(())
    }

    /// 단순 함수 내부 컴파일 (재귀/호출 없음)
    fn compile_simple_internal(
        &self,
        builder: &mut FunctionBuilder,
        func: &CompiledFunction,
        _ptr_type: Type,
    ) -> JitResult<()> {
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        let args_ptr = builder.block_params(entry_block)[0];
        let _arg_count = builder.block_params(entry_block)[1];

        let mut locals: HashMap<String, cranelift::prelude::Value> = HashMap::new();

        // 파라미터 로드
        for (i, param) in func.params.iter().enumerate() {
            let offset = (i * 8) as i32;
            let val = builder.ins().load(types::I64, MemFlags::trusted(), args_ptr, offset);
            locals.insert(param.clone(), val);
        }

        let mut stack: Vec<cranelift::prelude::Value> = Vec::new();

        // 제어 흐름 확인
        let has_control_flow = func.instructions.iter().any(|i| {
            matches!(i.opcode, OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_))
        });

        if has_control_flow {
            Self::compile_instructions_with_control_flow(builder, &func.instructions, &mut locals, &mut stack)?;
        } else {
            builder.seal_block(entry_block);
            Self::compile_linear_instructions(builder, &func.instructions, &mut locals, &mut stack)?;
            let result = stack.pop().ok_or_else(|| JitError::CodeGen("Empty stack at return".to_string()))?;
            builder.ins().return_(&[result]);
        }

        Ok(())
    }

    /// 다른 함수 호출이 있는 함수 컴파일 (재귀 없음)
    fn compile_with_calls_internal(
        &self,
        builder: &mut FunctionBuilder,
        func: &CompiledFunction,
        func_refs: &HashMap<String, FuncRef>,
        ptr_type: Type,
    ) -> JitResult<()> {
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        let args_ptr = builder.block_params(entry_block)[0];

        let mut locals: HashMap<String, cranelift::prelude::Value> = HashMap::new();

        // 파라미터 로드
        for (i, param) in func.params.iter().enumerate() {
            let offset = (i * 8) as i32;
            let val = builder.ins().load(types::I64, MemFlags::trusted(), args_ptr, offset);
            locals.insert(param.clone(), val);
        }

        let mut stack: Vec<cranelift::prelude::Value> = Vec::new();

        // 제어 흐름 확인
        let has_control_flow = func.instructions.iter().any(|i| {
            matches!(i.opcode, OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_))
        });

        if has_control_flow {
            Self::compile_with_calls_control_flow(
                builder,
                &func.instructions,
                &mut locals,
                &mut stack,
                func_refs,
                ptr_type,
            )?;
        } else {
            builder.seal_block(entry_block);
            Self::compile_with_calls_linear(
                builder,
                &func.instructions,
                &mut locals,
                &mut stack,
                func_refs,
                ptr_type,
            )?;
            let result = stack.pop().ok_or_else(|| JitError::CodeGen("Empty stack at return".to_string()))?;
            builder.ins().return_(&[result]);
        }

        Ok(())
    }

    /// 재귀 + 다른 함수 호출 지원 컴파일
    fn compile_recursive_with_calls_internal(
        &self,
        builder: &mut FunctionBuilder,
        func: &CompiledFunction,
        self_func_ref: FuncRef,
        func_refs: &HashMap<String, FuncRef>,
        ptr_type: Type,
    ) -> JitResult<()> {
        let param_count = func.params.len();

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);

        // 루프 헤더 블록
        let loop_header = builder.create_block();
        for _ in 0..param_count {
            builder.append_block_param(loop_header, types::I64);
        }

        let return_block = builder.create_block();

        // 결과 변수
        let result_var = Variable::new(0);
        builder.declare_var(result_var, types::I64);

        // 파라미터 변수들
        let mut param_vars: Vec<Variable> = Vec::new();
        for i in 0..param_count {
            let var = Variable::new(i + 1);
            builder.declare_var(var, types::I64);
            param_vars.push(var);
        }

        // === Entry Block ===
        builder.switch_to_block(entry_block);
        let args_ptr = builder.block_params(entry_block)[0];

        // 초기 파라미터 로드
        let mut initial_params: Vec<cranelift::prelude::Value> = Vec::new();
        for i in 0..param_count {
            let offset = (i * 8) as i32;
            let val = builder.ins().load(types::I64, MemFlags::trusted(), args_ptr, offset);
            initial_params.push(val);
        }

        let zero = builder.ins().iconst(types::I64, 0);
        builder.def_var(result_var, zero);

        builder.ins().jump(loop_header, &initial_params);
        builder.seal_block(entry_block);

        // === Loop Header Block ===
        builder.switch_to_block(loop_header);

        let loop_params = builder.block_params(loop_header).to_vec();
        for (i, &val) in loop_params.iter().enumerate() {
            builder.def_var(param_vars[i], val);
        }

        let mut stack: Vec<cranelift::prelude::Value> = Vec::new();
        let mut locals: HashMap<String, cranelift::prelude::Value> = HashMap::new();

        for (i, param_name) in func.params.iter().enumerate() {
            locals.insert(param_name.clone(), loop_params[i]);
        }

        // 제어 흐름 분석
        let has_control_flow = func.instructions.iter().any(|i| {
            matches!(i.opcode, OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_))
        });

        if has_control_flow {
            Self::compile_recursive_with_calls_cf(
                builder,
                &func.instructions,
                &mut locals,
                &mut stack,
                loop_header,
                return_block,
                result_var,
                self_func_ref,
                func_refs,
                ptr_type,
            )?;
        } else {
            Self::compile_recursive_with_calls_linear(
                builder,
                &func.instructions,
                &mut locals,
                &mut stack,
                loop_header,
                return_block,
                result_var,
                self_func_ref,
                func_refs,
                ptr_type,
            )?;
            builder.seal_block(loop_header);
        }

        builder.seal_block(return_block);

        // === Return Block ===
        builder.switch_to_block(return_block);
        let result = builder.use_var(result_var);
        builder.ins().return_(&[result]);

        Ok(())
    }

    /// 선형 코드 + 함수 호출 컴파일
    fn compile_with_calls_linear(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
        func_refs: &HashMap<String, FuncRef>,
        ptr_type: Type,
    ) -> JitResult<()> {
        for instr in instructions {
            match &instr.opcode {
                OpCode::Return => break,
                OpCode::Call(name, arg_count) => {
                    Self::compile_call(builder, stack, name, *arg_count, func_refs, ptr_type)?;
                }
                _ => {
                    Self::compile_single_instruction(builder, &instr.opcode, locals, stack)?;
                }
            }
        }
        Ok(())
    }

    /// 재귀 + 함수 호출 선형 컴파일
    #[allow(clippy::too_many_arguments)]
    fn compile_recursive_with_calls_linear(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
        loop_header: Block,
        return_block: Block,
        result_var: Variable,
        self_func_ref: FuncRef,
        func_refs: &HashMap<String, FuncRef>,
        ptr_type: Type,
    ) -> JitResult<()> {
        for instr in instructions {
            match &instr.opcode {
                OpCode::Return => {
                    let result = stack.pop()
                        .ok_or_else(|| JitError::CodeGen("Stack underflow at Return".to_string()))?;
                    builder.def_var(result_var, result);
                    builder.ins().jump(return_block, &[]);
                    return Ok(());
                }
                OpCode::TailSelfCall(arg_count) => {
                    let mut new_args: Vec<cranelift::prelude::Value> = Vec::new();
                    for _ in 0..*arg_count {
                        new_args.push(stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at TailSelfCall".to_string()))?);
                    }
                    new_args.reverse();
                    builder.ins().jump(loop_header, &new_args);
                    return Ok(());
                }
                OpCode::SelfCall(arg_count) => {
                    Self::compile_self_call(builder, stack, *arg_count, self_func_ref, ptr_type)?;
                }
                OpCode::Call(name, arg_count) => {
                    Self::compile_call(builder, stack, name, *arg_count, func_refs, ptr_type)?;
                }
                _ => {
                    Self::compile_single_instruction(builder, &instr.opcode, locals, stack)?;
                }
            }
        }
        Ok(())
    }

    /// 제어 흐름 + 함수 호출 컴파일
    #[allow(clippy::too_many_arguments)]
    fn compile_with_calls_control_flow(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        _stack: &mut Vec<cranelift::prelude::Value>,
        func_refs: &HashMap<String, FuncRef>,
        ptr_type: Type,
    ) -> JitResult<()> {
        // 블록 분석
        let mut block_starts: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        block_starts.insert(0);

        for (ip, instr) in instructions.iter().enumerate() {
            match &instr.opcode {
                OpCode::Jump(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if target < instructions.len() {
                        block_starts.insert(target);
                    }
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                OpCode::JumpIf(offset) | OpCode::JumpIfNot(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if target < instructions.len() {
                        block_starts.insert(target);
                    }
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                OpCode::Return => {
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                _ => {}
            }
        }

        let block_starts_vec: Vec<usize> = block_starts.iter().cloned().collect();
        let mut ip_to_block: HashMap<usize, usize> = HashMap::new();
        for (block_idx, &start) in block_starts_vec.iter().enumerate() {
            let end = block_starts_vec.get(block_idx + 1).cloned().unwrap_or(instructions.len());
            for ip in start..end {
                ip_to_block.insert(ip, block_idx);
            }
        }

        let entry_block = builder.current_block()
            .ok_or_else(|| JitError::Internal("No current block in builder".to_string()))?;
        let mut blocks: Vec<Block> = Vec::new();
        blocks.push(entry_block);

        for _ in 1..block_starts_vec.len() {
            let block = builder.create_block();
            blocks.push(block);
        }

        let result_var = Variable::new(0);
        builder.declare_var(result_var, types::I64);
        let zero = builder.ins().iconst(types::I64, 0);
        builder.def_var(result_var, zero);

        let return_block = builder.create_block();

        // 각 블록 컴파일
        for (block_idx, &start) in block_starts_vec.iter().enumerate() {
            let end = block_starts_vec.get(block_idx + 1).cloned().unwrap_or(instructions.len());
            let block = blocks[block_idx];

            if block_idx > 0 {
                builder.switch_to_block(block);
            }

            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();
            let mut terminated = false;

            for ip in start..end {
                let instr = &instructions[ip];

                match &instr.opcode {
                    OpCode::Jump(offset) => {
                        if let Some(val) = stack.last() {
                            builder.def_var(result_var, *val);
                        }
                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];
                        builder.ins().jump(target_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIf(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIf".to_string()))?;
                        let zero = builder.ins().iconst(types::I64, 0);
                        let cmp = builder.ins().icmp(IntCC::NotEqual, cond, zero);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIfNot(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIfNot".to_string()))?;
                        let zero = builder.ins().iconst(types::I64, 0);
                        let cmp = builder.ins().icmp(IntCC::Equal, cond, zero);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::Return => {
                        if let Some(result) = stack.pop() {
                            builder.def_var(result_var, result);
                        }
                        builder.ins().jump(return_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::Call(name, arg_count) => {
                        Self::compile_call(builder, &mut stack, name, *arg_count, func_refs, ptr_type)?;
                    }
                    _ => {
                        Self::compile_single_instruction(builder, &instr.opcode, locals, &mut stack)?;
                    }
                }
            }

            if !terminated {
                if let Some(val) = stack.last() {
                    builder.def_var(result_var, *val);
                }
                if block_idx + 1 < blocks.len() {
                    builder.ins().jump(blocks[block_idx + 1], &[]);
                } else {
                    builder.ins().jump(return_block, &[]);
                }
            }
        }

        // 블록 seal
        for block in &blocks {
            builder.seal_block(*block);
        }
        builder.seal_block(return_block);

        // Return 블록
        builder.switch_to_block(return_block);
        let result = builder.use_var(result_var);
        builder.ins().return_(&[result]);

        Ok(())
    }

    /// 재귀 + 제어 흐름 + 함수 호출 컴파일
    #[allow(clippy::too_many_arguments)]
    fn compile_recursive_with_calls_cf(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        _stack: &mut Vec<cranelift::prelude::Value>,
        loop_header: Block,
        return_block: Block,
        result_var: Variable,
        self_func_ref: FuncRef,
        func_refs: &HashMap<String, FuncRef>,
        ptr_type: Type,
    ) -> JitResult<()> {
        // 블록 분석
        let mut block_starts: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        block_starts.insert(0);

        for (ip, instr) in instructions.iter().enumerate() {
            match &instr.opcode {
                OpCode::Jump(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if target < instructions.len() {
                        block_starts.insert(target);
                    }
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                OpCode::JumpIf(offset) | OpCode::JumpIfNot(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if target < instructions.len() {
                        block_starts.insert(target);
                    }
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                OpCode::Return | OpCode::TailSelfCall(_) => {
                    if ip + 1 < instructions.len() {
                        block_starts.insert(ip + 1);
                    }
                }
                _ => {}
            }
        }

        let block_starts_vec: Vec<usize> = block_starts.iter().cloned().collect();
        let mut ip_to_block: HashMap<usize, usize> = HashMap::new();
        for (block_idx, &start) in block_starts_vec.iter().enumerate() {
            let end = block_starts_vec.get(block_idx + 1).cloned().unwrap_or(instructions.len());
            for ip in start..end {
                ip_to_block.insert(ip, block_idx);
            }
        }

        let current_block = builder.current_block()
            .ok_or_else(|| JitError::Internal("No current block in builder".to_string()))?;
        let mut blocks: Vec<Block> = Vec::new();
        blocks.push(current_block);

        for _ in 1..block_starts_vec.len() {
            let block = builder.create_block();
            blocks.push(block);
        }

        // 각 블록 컴파일
        for (block_idx, &start) in block_starts_vec.iter().enumerate() {
            let end = block_starts_vec.get(block_idx + 1).cloned().unwrap_or(instructions.len());
            let block = blocks[block_idx];

            if block_idx > 0 {
                builder.switch_to_block(block);
            }

            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();
            let mut terminated = false;

            for ip in start..end {
                let instr = &instructions[ip];

                match &instr.opcode {
                    OpCode::Jump(offset) => {
                        if let Some(val) = stack.last() {
                            builder.def_var(result_var, *val);
                        }
                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];
                        builder.ins().jump(target_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIf(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIf".to_string()))?;
                        let zero = builder.ins().iconst(types::I64, 0);
                        let cmp = builder.ins().icmp(IntCC::NotEqual, cond, zero);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIfNot(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIfNot".to_string()))?;
                        let zero = builder.ins().iconst(types::I64, 0);
                        let cmp = builder.ins().icmp(IntCC::Equal, cond, zero);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::Return => {
                        if let Some(result) = stack.pop() {
                            builder.def_var(result_var, result);
                        }
                        builder.ins().jump(return_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::TailSelfCall(arg_count) => {
                        let mut new_args: Vec<cranelift::prelude::Value> = Vec::new();
                        for _ in 0..*arg_count {
                            new_args.push(stack.pop()
                                .ok_or_else(|| JitError::CodeGen("Stack underflow at TailSelfCall".to_string()))?);
                        }
                        new_args.reverse();
                        builder.ins().jump(loop_header, &new_args);
                        terminated = true;
                        break;
                    }
                    OpCode::SelfCall(arg_count) => {
                        Self::compile_self_call(builder, &mut stack, *arg_count, self_func_ref, ptr_type)?;
                    }
                    OpCode::Call(name, arg_count) => {
                        Self::compile_call(builder, &mut stack, name, *arg_count, func_refs, ptr_type)?;
                    }
                    _ => {
                        Self::compile_single_instruction(builder, &instr.opcode, locals, &mut stack)?;
                    }
                }
            }

            if !terminated {
                if let Some(val) = stack.last() {
                    builder.def_var(result_var, *val);
                }
                if block_idx + 1 < blocks.len() {
                    builder.ins().jump(blocks[block_idx + 1], &[]);
                } else {
                    builder.ins().jump(return_block, &[]);
                }
            }
        }

        // 블록 seal
        for (i, block) in blocks.iter().enumerate() {
            if i > 0 {
                builder.seal_block(*block);
            }
        }
        builder.seal_block(loop_header);

        Ok(())
    }

    /// Call opcode 컴파일: 다른 함수 호출
    fn compile_call(
        builder: &mut FunctionBuilder,
        stack: &mut Vec<cranelift::prelude::Value>,
        name: &str,
        arg_count: usize,
        func_refs: &HashMap<String, FuncRef>,
        ptr_type: Type,
    ) -> JitResult<()> {
        let func_ref = func_refs.get(name)
            .ok_or_else(|| JitError::FunctionNotFound(name.to_string()))?;

        // 스택에서 인자 pop
        let mut args: Vec<cranelift::prelude::Value> = Vec::new();
        for _ in 0..arg_count {
            args.push(stack.pop()
                .ok_or_else(|| JitError::CodeGen("Stack underflow at Call".to_string()))?);
        }
        args.reverse();

        // 스택에 인자 배열 할당
        let slot_size = (arg_count * 8) as u32;
        let slot = builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, slot_size.max(8), 0));
        let slot_addr = builder.ins().stack_addr(ptr_type, slot, 0);

        // 인자를 스택 슬롯에 저장
        for (i, arg) in args.iter().enumerate() {
            let offset = (i * 8) as i32;
            builder.ins().store(MemFlags::trusted(), *arg, slot_addr, offset);
        }

        // 함수 호출
        let arg_count_val = builder.ins().iconst(types::I64, arg_count as i64);
        let call = builder.ins().call(*func_ref, &[slot_addr, arg_count_val]);
        let result = builder.inst_results(call)[0];
        stack.push(result);

        Ok(())
    }

    // ========================================================================
    // Float Support
    // ========================================================================

    /// 함수를 JIT 컴파일 (Float 전용 최적화)
    pub fn compile_function_float(&mut self, func: &CompiledFunction) -> JitResult<*const u8> {
        let name = &func.name;

        // 이미 컴파일된 경우 캐시에서 반환
        if let Some(compiled) = self.compiled_functions.get(name) {
            return Ok(compiled.ptr);
        }

        // 재귀 호출이 있는지 확인
        let has_self_call = func.instructions.iter().any(|i| {
            matches!(i.opcode, OpCode::SelfCall(_) | OpCode::TailSelfCall(_))
        });

        if has_self_call {
            self.compile_recursive_function_float(func)
        } else {
            self.compile_simple_function_float(func)
        }
    }

    /// 단순 함수 컴파일 (Float, 재귀 없음)
    fn compile_simple_function_float(&mut self, func: &CompiledFunction) -> JitResult<*const u8> {
        let name = &func.name;

        // 함수 시그니처 정의: (f64*, usize) -> f64
        let ptr_type = self.module.target_config().pointer_type();
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(ptr_type)); // args pointer
        sig.params.push(AbiParam::new(types::I64)); // arg count
        sig.returns.push(AbiParam::new(types::F64)); // return value

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

            // 인자 추출
            let args_ptr = builder.block_params(entry_block)[0];

            // 로컬 변수 맵 (파라미터 -> SSA value)
            let mut locals: HashMap<String, cranelift::prelude::Value> = HashMap::new();

            // 파라미터를 로컬 변수로 로드 (f64로)
            for (i, param) in func.params.iter().enumerate() {
                let offset = (i * 8) as i32;
                let val = builder.ins().load(types::F64, MemFlags::trusted(), args_ptr, offset);
                locals.insert(param.clone(), val);
            }

            // 스택 시뮬레이션
            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();

            // 제어 흐름이 있는지 확인
            let has_control_flow = func.instructions.iter().any(|i| {
                matches!(i.opcode, OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_))
            });

            if has_control_flow {
                Self::compile_float_instructions_with_control_flow(&mut builder, &func.instructions, &mut locals, &mut stack)?;
            } else {
                builder.seal_block(entry_block);
                Self::compile_float_linear_instructions(&mut builder, &func.instructions, &mut locals, &mut stack)?;

                let result = stack.pop().ok_or_else(|| JitError::CodeGen("Empty stack at return".to_string()))?;
                builder.ins().return_(&[result]);
            }

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
            signature: FnSignature::FloatOnly { param_count: func.params.len() },
        });

        Ok(code_ptr)
    }

    /// 재귀 함수 컴파일 (Float, SelfCall/TailSelfCall 지원)
    fn compile_recursive_function_float(&mut self, func: &CompiledFunction) -> JitResult<*const u8> {
        let name = &func.name;
        let param_count = func.params.len();

        // 함수 시그니처 정의: (f64*, usize) -> f64
        let ptr_type = self.module.target_config().pointer_type();
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(ptr_type));
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::F64));

        // 함수 선언
        let func_id = self.module
            .declare_function(name, Linkage::Local, &sig)?;

        // 컨텍스트 생성
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig.clone();
        ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32());

        // 함수 참조
        let self_func_ref = self.module.declare_func_in_func(func_id, &mut ctx.func);

        // 함수 빌더 컨텍스트
        let mut func_ctx = FunctionBuilderContext::new();

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);

            // 루프 헤더 블록 - Float 파라미터
            let loop_header = builder.create_block();
            for _ in 0..param_count {
                builder.append_block_param(loop_header, types::F64);
            }

            let return_block = builder.create_block();

            // 결과 변수
            let result_var = Variable::new(0);
            builder.declare_var(result_var, types::F64);

            // 파라미터 변수들
            let mut param_vars: Vec<Variable> = Vec::new();
            for i in 0..param_count {
                let var = Variable::new(i + 1);
                builder.declare_var(var, types::F64);
                param_vars.push(var);
            }

            // === Entry Block ===
            builder.switch_to_block(entry_block);
            let args_ptr = builder.block_params(entry_block)[0];

            // 초기 파라미터 로드 (f64)
            let mut initial_params: Vec<cranelift::prelude::Value> = Vec::new();
            for i in 0..param_count {
                let offset = (i * 8) as i32;
                let val = builder.ins().load(types::F64, MemFlags::trusted(), args_ptr, offset);
                initial_params.push(val);
            }

            // 기본 result 초기화
            let zero = builder.ins().f64const(0.0);
            builder.def_var(result_var, zero);

            // 루프 헤더로 점프
            builder.ins().jump(loop_header, &initial_params);
            builder.seal_block(entry_block);

            // === Loop Header Block ===
            builder.switch_to_block(loop_header);

            let loop_params = builder.block_params(loop_header).to_vec();
            for (i, &val) in loop_params.iter().enumerate() {
                builder.def_var(param_vars[i], val);
            }

            // 스택 및 로컬
            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();
            let mut locals: HashMap<String, cranelift::prelude::Value> = HashMap::new();

            for (i, param_name) in func.params.iter().enumerate() {
                locals.insert(param_name.clone(), loop_params[i]);
            }

            // 제어 흐름 분석
            let has_control_flow = func.instructions.iter().any(|i| {
                matches!(i.opcode, OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_))
            });

            if has_control_flow {
                Self::compile_float_recursive_with_cf(
                    &mut builder,
                    func,
                    &mut locals,
                    &mut stack,
                    result_var,
                    &param_vars,
                    loop_header,
                    return_block,
                    self_func_ref,
                    ptr_type,
                )?;
            } else {
                // 선형 코드
                Self::compile_float_recursive_linear(
                    &mut builder,
                    func,
                    &mut locals,
                    &mut stack,
                    result_var,
                    loop_header,
                    return_block,
                    self_func_ref,
                    ptr_type,
                )?;
            }

            // === Return Block ===
            builder.switch_to_block(return_block);
            builder.seal_block(return_block);
            let final_result = builder.use_var(result_var);
            builder.ins().return_(&[final_result]);

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
            signature: FnSignature::FloatOnly { param_count: func.params.len() },
        });

        Ok(code_ptr)
    }

    /// Float 선형 명령어 컴파일
    fn compile_float_linear_instructions(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
    ) -> JitResult<()> {
        for instr in instructions {
            Self::compile_float_single_instruction(builder, &instr.opcode, locals, stack)?;
        }
        Ok(())
    }

    /// Float 단일 명령어 컴파일
    fn compile_float_single_instruction(
        builder: &mut FunctionBuilder,
        opcode: &OpCode,
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
    ) -> JitResult<()> {
        match opcode {
            // === 상수 ===
            OpCode::Const(Value::Float(f)) => {
                let val = builder.ins().f64const(*f);
                stack.push(val);
            }
            OpCode::Const(Value::Int(n)) => {
                // Int를 Float로 변환
                let int_val = builder.ins().iconst(types::I64, *n);
                let val = builder.ins().fcvt_from_sint(types::F64, int_val);
                stack.push(val);
            }
            OpCode::Const(Value::Bool(b)) => {
                let val = builder.ins().f64const(if *b { 1.0 } else { 0.0 });
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

            // === 산술 연산 (Float) ===
            OpCode::Add => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let result = builder.ins().fadd(a, b);
                stack.push(result);
            }
            OpCode::Sub => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let result = builder.ins().fsub(a, b);
                stack.push(result);
            }
            OpCode::Mul => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let result = builder.ins().fmul(a, b);
                stack.push(result);
            }
            OpCode::Div => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let result = builder.ins().fdiv(a, b);
                stack.push(result);
            }
            OpCode::Neg => {
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let result = builder.ins().fneg(a);
                stack.push(result);
            }

            // === 비교 연산 (Float -> Float로 결과 반환: 0.0 또는 1.0) ===
            OpCode::Lt => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let cmp = builder.ins().fcmp(FloatCC::LessThan, a, b);
                let int_result = builder.ins().uextend(types::I64, cmp);
                let result = builder.ins().fcvt_from_sint(types::F64, int_result);
                stack.push(result);
            }
            OpCode::Gt => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let cmp = builder.ins().fcmp(FloatCC::GreaterThan, a, b);
                let int_result = builder.ins().uextend(types::I64, cmp);
                let result = builder.ins().fcvt_from_sint(types::F64, int_result);
                stack.push(result);
            }
            OpCode::Lte => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let cmp = builder.ins().fcmp(FloatCC::LessThanOrEqual, a, b);
                let int_result = builder.ins().uextend(types::I64, cmp);
                let result = builder.ins().fcvt_from_sint(types::F64, int_result);
                stack.push(result);
            }
            OpCode::Gte => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let cmp = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, a, b);
                let int_result = builder.ins().uextend(types::I64, cmp);
                let result = builder.ins().fcvt_from_sint(types::F64, int_result);
                stack.push(result);
            }
            OpCode::Eq => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let cmp = builder.ins().fcmp(FloatCC::Equal, a, b);
                let int_result = builder.ins().uextend(types::I64, cmp);
                let result = builder.ins().fcvt_from_sint(types::F64, int_result);
                stack.push(result);
            }
            OpCode::Neq => {
                let b = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let a = stack.pop().ok_or_else(|| JitError::CodeGen("Stack underflow".to_string()))?;
                let cmp = builder.ins().fcmp(FloatCC::NotEqual, a, b);
                let int_result = builder.ins().uextend(types::I64, cmp);
                let result = builder.ins().fcvt_from_sint(types::F64, int_result);
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
            OpCode::Nop => {}

            // Return, Jump 등은 caller에서 처리
            OpCode::Return | OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_) => {}

            // Mod는 Float에서 지원하지 않음 (필요시 fmod 호출)
            OpCode::Mod => {
                return Err(JitError::UnsupportedOpcode("Mod not supported for float".to_string()));
            }

            // And, Or, Not은 Float에서 비트연산 불가
            OpCode::And | OpCode::Or | OpCode::Not => {
                return Err(JitError::UnsupportedOpcode(format!("{:?} not supported for float", opcode)));
            }

            _ => {
                return Err(JitError::UnsupportedOpcode(format!("{:?}", opcode)));
            }
        }
        Ok(())
    }

    /// Float 제어 흐름 명령어 컴파일
    fn compile_float_instructions_with_control_flow(
        builder: &mut FunctionBuilder,
        instructions: &[Instruction],
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
    ) -> JitResult<()> {
        // 블록 경계 분석
        let mut block_starts: Vec<usize> = vec![0];
        for (ip, instr) in instructions.iter().enumerate() {
            match &instr.opcode {
                OpCode::Jump(offset) | OpCode::JumpIf(offset) | OpCode::JumpIfNot(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if !block_starts.contains(&target) {
                        block_starts.push(target);
                    }
                    if !block_starts.contains(&(ip + 1)) {
                        block_starts.push(ip + 1);
                    }
                }
                _ => {}
            }
        }
        block_starts.sort();
        block_starts.dedup();

        // 블록 생성
        let entry_block = builder.current_block()
            .ok_or_else(|| JitError::Internal("No current block in builder".to_string()))?;
        let mut blocks: Vec<Block> = vec![entry_block];
        let mut ip_to_block: HashMap<usize, usize> = HashMap::new();
        ip_to_block.insert(0, 0);

        for &start in block_starts.iter().skip(1) {
            let block = builder.create_block();
            ip_to_block.insert(start, blocks.len());
            blocks.push(block);
        }

        // 결과 변수
        let result_var = Variable::new(0);
        builder.declare_var(result_var, types::F64);
        let zero = builder.ins().f64const(0.0);
        builder.def_var(result_var, zero);

        // 블록별 컴파일
        for (block_idx, block) in blocks.iter().enumerate() {
            if block_idx > 0 {
                builder.switch_to_block(*block);
            }

            let start_ip = block_starts[block_idx];
            let end_ip = if block_idx + 1 < block_starts.len() {
                block_starts[block_idx + 1]
            } else {
                instructions.len()
            };

            let mut terminated = false;

            for ip in start_ip..end_ip {
                let instr = &instructions[ip];

                match &instr.opcode {
                    OpCode::Jump(offset) => {
                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];
                        builder.ins().jump(target_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIf(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIf".to_string()))?;
                        let zero_f = builder.ins().f64const(0.0);
                        let cmp = builder.ins().fcmp(FloatCC::NotEqual, cond, zero_f);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIfNot(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIfNot".to_string()))?;
                        let zero_f = builder.ins().f64const(0.0);
                        let cmp = builder.ins().fcmp(FloatCC::Equal, cond, zero_f);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::Return => {
                        let result = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at Return".to_string()))?;
                        builder.ins().return_(&[result]);
                        terminated = true;
                        break;
                    }
                    _ => {
                        Self::compile_float_single_instruction(builder, &instr.opcode, locals, stack)?;
                    }
                }
            }

            if !terminated {
                if block_idx + 1 < blocks.len() {
                    builder.ins().jump(blocks[block_idx + 1], &[]);
                } else if let Some(result) = stack.pop() {
                    builder.ins().return_(&[result]);
                }
            }
        }

        // 블록 seal
        for block in &blocks {
            builder.seal_block(*block);
        }

        Ok(())
    }

    /// Float 재귀 선형 컴파일
    fn compile_float_recursive_linear(
        builder: &mut FunctionBuilder,
        func: &CompiledFunction,
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
        result_var: Variable,
        loop_header: Block,
        return_block: Block,
        self_func_ref: FuncRef,
        ptr_type: Type,
    ) -> JitResult<()> {
        for instr in &func.instructions {
            match &instr.opcode {
                OpCode::Return => {
                    let result = stack.pop()
                        .ok_or_else(|| JitError::CodeGen("Stack underflow at Return".to_string()))?;
                    builder.def_var(result_var, result);
                    builder.ins().jump(return_block, &[]);
                    builder.seal_block(loop_header);
                    return Ok(());
                }
                OpCode::TailSelfCall(arg_count) => {
                    let mut new_args: Vec<cranelift::prelude::Value> = Vec::new();
                    for _ in 0..*arg_count {
                        new_args.push(stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at TailSelfCall".to_string()))?);
                    }
                    new_args.reverse();
                    builder.ins().jump(loop_header, &new_args);
                    builder.seal_block(loop_header);
                    return Ok(());
                }
                OpCode::SelfCall(arg_count) => {
                    Self::compile_float_self_call(builder, stack, *arg_count, self_func_ref, ptr_type)?;
                }
                _ => {
                    Self::compile_float_single_instruction(builder, &instr.opcode, locals, stack)?;
                }
            }
        }

        builder.seal_block(loop_header);
        Ok(())
    }

    /// Float 재귀 제어 흐름 컴파일
    #[allow(clippy::too_many_arguments)]
    fn compile_float_recursive_with_cf(
        builder: &mut FunctionBuilder,
        func: &CompiledFunction,
        locals: &mut HashMap<String, cranelift::prelude::Value>,
        stack: &mut Vec<cranelift::prelude::Value>,
        result_var: Variable,
        param_vars: &[Variable],
        loop_header: Block,
        return_block: Block,
        self_func_ref: FuncRef,
        ptr_type: Type,
    ) -> JitResult<()> {
        let instructions = &func.instructions;

        // 블록 경계 분석
        let mut block_starts: Vec<usize> = vec![0];
        for (ip, instr) in instructions.iter().enumerate() {
            match &instr.opcode {
                OpCode::Jump(offset) | OpCode::JumpIf(offset) | OpCode::JumpIfNot(offset) => {
                    let target = ((ip as i32) + *offset) as usize;
                    if !block_starts.contains(&target) {
                        block_starts.push(target);
                    }
                    if !block_starts.contains(&(ip + 1)) {
                        block_starts.push(ip + 1);
                    }
                }
                _ => {}
            }
        }
        block_starts.sort();
        block_starts.dedup();

        // 블록 생성
        let mut blocks: Vec<Block> = Vec::new();
        let mut ip_to_block: HashMap<usize, usize> = HashMap::new();

        for &start in &block_starts {
            if start == 0 {
                blocks.push(loop_header);
            } else {
                blocks.push(builder.create_block());
            }
            ip_to_block.insert(start, blocks.len() - 1);
        }

        // 블록별 컴파일
        for (block_idx, block) in blocks.iter().enumerate() {
            if block_idx > 0 {
                builder.switch_to_block(*block);
            }

            let start_ip = block_starts[block_idx];
            let end_ip = if block_idx + 1 < block_starts.len() {
                block_starts[block_idx + 1]
            } else {
                instructions.len()
            };

            let mut terminated = false;

            for ip in start_ip..end_ip {
                let instr = &instructions[ip];

                match &instr.opcode {
                    OpCode::Jump(offset) => {
                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];
                        builder.ins().jump(target_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIf(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIf".to_string()))?;
                        let zero_f = builder.ins().f64const(0.0);
                        let cmp = builder.ins().fcmp(FloatCC::NotEqual, cond, zero_f);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::JumpIfNot(offset) => {
                        let cond = stack.pop()
                            .ok_or_else(|| JitError::CodeGen("Stack underflow at JumpIfNot".to_string()))?;
                        let zero_f = builder.ins().f64const(0.0);
                        let cmp = builder.ins().fcmp(FloatCC::Equal, cond, zero_f);

                        let target_ip = ((ip as i32) + *offset) as usize;
                        let target_block_idx = ip_to_block.get(&target_ip)
                            .ok_or_else(|| JitError::CodeGen(format!("Invalid jump target: {}", target_ip)))?;
                        let target_block = blocks[*target_block_idx];

                        let fallthrough_block_idx = ip_to_block.get(&(ip + 1))
                            .ok_or_else(|| JitError::CodeGen("Invalid fallthrough".to_string()))?;
                        let fallthrough_block = blocks[*fallthrough_block_idx];

                        builder.ins().brif(cmp, target_block, &[], fallthrough_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::Return => {
                        if let Some(result) = stack.pop() {
                            builder.def_var(result_var, result);
                        }
                        builder.ins().jump(return_block, &[]);
                        terminated = true;
                        break;
                    }
                    OpCode::TailSelfCall(arg_count) => {
                        let mut new_args: Vec<cranelift::prelude::Value> = Vec::new();
                        for _ in 0..*arg_count {
                            new_args.push(stack.pop()
                                .ok_or_else(|| JitError::CodeGen("Stack underflow at TailSelfCall".to_string()))?);
                        }
                        new_args.reverse();
                        builder.ins().jump(loop_header, &new_args);
                        terminated = true;
                        break;
                    }
                    OpCode::SelfCall(arg_count) => {
                        Self::compile_float_self_call(builder, stack, *arg_count, self_func_ref, ptr_type)?;
                    }
                    _ => {
                        Self::compile_float_single_instruction(builder, &instr.opcode, locals, stack)?;
                    }
                }
            }

            if !terminated {
                if let Some(val) = stack.last() {
                    builder.def_var(result_var, *val);
                }
                if block_idx + 1 < blocks.len() {
                    builder.ins().jump(blocks[block_idx + 1], &[]);
                } else {
                    builder.ins().jump(return_block, &[]);
                }
            }
        }

        // 블록 seal
        for (i, block) in blocks.iter().enumerate() {
            if i > 0 {
                builder.seal_block(*block);
            }
        }
        builder.seal_block(loop_header);

        // param_vars 사용 (미사용 경고 방지)
        let _ = param_vars;

        Ok(())
    }

    /// Float SelfCall 컴파일
    fn compile_float_self_call(
        builder: &mut FunctionBuilder,
        stack: &mut Vec<cranelift::prelude::Value>,
        arg_count: usize,
        self_func_ref: FuncRef,
        ptr_type: Type,
    ) -> JitResult<()> {
        // 인자 pop
        let mut args: Vec<cranelift::prelude::Value> = Vec::new();
        for _ in 0..arg_count {
            args.push(stack.pop()
                .ok_or_else(|| JitError::CodeGen("Stack underflow at SelfCall".to_string()))?);
        }
        args.reverse();

        // 스택 슬롯에 인자 저장
        let slot_size = (arg_count * 8) as u32;
        let slot = builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, slot_size.max(8), 0));
        let slot_addr = builder.ins().stack_addr(ptr_type, slot, 0);

        for (i, arg) in args.iter().enumerate() {
            let offset = (i * 8) as i32;
            builder.ins().store(MemFlags::trusted(), *arg, slot_addr, offset);
        }

        // 함수 호출
        let arg_count_val = builder.ins().iconst(types::I64, arg_count as i64);
        let call = builder.ins().call(self_func_ref, &[slot_addr, arg_count_val]);
        let result = builder.inst_results(call)[0];
        stack.push(result);

        Ok(())
    }

    /// 컴파일된 함수 호출 (Float 전용)
    ///
    /// # Safety
    /// - 함수가 `compile_function_float`로 컴파일되어 있어야 함
    /// - `args` 배열의 길이가 함수의 파라미터 개수와 일치해야 함
    pub unsafe fn call_float(&self, name: &str, args: &[f64]) -> JitResult<f64> {
        let compiled = self.compiled_functions.get(name)
            .ok_or_else(|| JitError::FunctionNotFound(name.to_string()))?;

        let func: JittedFnFloat = std::mem::transmute(compiled.ptr);
        Ok(func(args.as_ptr(), args.len()))
    }

    /// 함수 분석 - Float 타입 감지
    pub fn analyze_function_type(func: &CompiledFunction) -> FnSignature {
        let mut has_float = false;
        let mut has_int_only = true;

        for instr in &func.instructions {
            match &instr.opcode {
                OpCode::Const(Value::Float(_)) => {
                    has_float = true;
                    has_int_only = false;
                }
                OpCode::Const(Value::Int(_)) | OpCode::Const(Value::Bool(_)) => {}
                _ => {}
            }
        }

        if has_float && !has_int_only {
            FnSignature::FloatOnly { param_count: func.params.len() }
        } else {
            FnSignature::IntOnly { param_count: func.params.len() }
        }
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
    use vais_ir::Instruction;

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
            local_count: 2,
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
            local_count: 2,
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            // (5 + 3) * (5 - 3) = 8 * 2 = 16
            let result = jit.call_int("calc", &[5, 3]).unwrap();
            assert_eq!(result, 16);
        }
    }

    #[test]
    fn test_logical_operations() {
        let mut jit = JitCompiler::new().unwrap();

        // and_test(a, b) = a && b
        let func = CompiledFunction {
            name: "and_test".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::And),
                make_instruction(OpCode::Return),
            ],
            local_count: 2,
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            assert_eq!(jit.call_int("and_test", &[1, 1]).unwrap(), 1);
            assert_eq!(jit.call_int("and_test", &[1, 0]).unwrap(), 0);
            assert_eq!(jit.call_int("and_test", &[0, 0]).unwrap(), 0);
        }
    }

    #[test]
    fn test_control_flow_simple_if() {
        let mut jit = JitCompiler::new().unwrap();

        // max(a, b) = a > b ? a : b
        // IR:
        //   0: Load a
        //   1: Load b
        //   2: Gt          ; stack: [a > b]
        //   3: JumpIfNot 3 ; if false, jump to 6
        //   4: Load a      ; then branch
        //   5: Jump 2      ; jump to 7 (return)
        //   6: Load b      ; else branch
        //   7: Return
        let func = CompiledFunction {
            name: "max".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),   // 0
                make_instruction(OpCode::Load("b".to_string())),   // 1
                make_instruction(OpCode::Gt),                       // 2
                make_instruction(OpCode::JumpIfNot(3)),             // 3: if !(a > b), jump to 6
                make_instruction(OpCode::Load("a".to_string())),   // 4: then
                make_instruction(OpCode::Jump(2)),                  // 5: jump to 7
                make_instruction(OpCode::Load("b".to_string())),   // 6: else
                make_instruction(OpCode::Return),                   // 7
            ],
            local_count: 2,
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            assert_eq!(jit.call_int("max", &[10, 5]).unwrap(), 10);
            assert_eq!(jit.call_int("max", &[3, 8]).unwrap(), 8);
            assert_eq!(jit.call_int("max", &[5, 5]).unwrap(), 5);
        }
    }

    #[test]
    fn test_control_flow_abs() {
        let mut jit = JitCompiler::new().unwrap();

        // abs(n) = n < 0 ? -n : n
        // IR:
        //   0: Load n
        //   1: Const 0
        //   2: Lt          ; n < 0
        //   3: JumpIfNot 3 ; if false, jump to 6
        //   4: Load n
        //   5: Neg         ; -n
        //   6: Jump 1      ; jump to 7
        //   7: Load n      ; else: n
        //   8: Return
        let func = CompiledFunction {
            name: "abs".to_string(),
            params: vec!["n".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("n".to_string())),   // 0
                make_instruction(OpCode::Const(Value::Int(0))),    // 1
                make_instruction(OpCode::Lt),                       // 2
                make_instruction(OpCode::JumpIfNot(4)),             // 3: if !(n < 0), jump to 7
                make_instruction(OpCode::Load("n".to_string())),   // 4: then
                make_instruction(OpCode::Neg),                      // 5
                make_instruction(OpCode::Jump(2)),                  // 6: jump to 8
                make_instruction(OpCode::Load("n".to_string())),   // 7: else
                make_instruction(OpCode::Return),                   // 8
            ],
            local_count: 1,
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            assert_eq!(jit.call_int("abs", &[10]).unwrap(), 10);
            assert_eq!(jit.call_int("abs", &[-5]).unwrap(), 5);
            assert_eq!(jit.call_int("abs", &[0]).unwrap(), 0);
        }
    }

    #[test]
    fn test_recursive_factorial_selfcall() {
        let mut jit = JitCompiler::new().unwrap();

        // factorial(n) = n <= 1 ? 1 : n * factorial(n - 1)
        // IR:
        //   0: Load n
        //   1: Const 1
        //   2: Lte          ; n <= 1
        //   3: JumpIfNot 3  ; if false, jump to 6
        //   4: Const 1      ; then: return 1
        //   5: Return
        //   6: Load n       ; else: n * factorial(n-1)
        //   7: Load n
        //   8: Const 1
        //   9: Sub          ; n - 1
        //  10: SelfCall(1)  ; factorial(n-1)
        //  11: Mul          ; n * factorial(n-1)
        //  12: Return
        let func = CompiledFunction {
            name: "factorial".to_string(),
            params: vec!["n".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("n".to_string())),    // 0
                make_instruction(OpCode::Const(Value::Int(1))),     // 1
                make_instruction(OpCode::Lte),                       // 2
                make_instruction(OpCode::JumpIfNot(3)),              // 3: if !(n <= 1), jump to 6
                make_instruction(OpCode::Const(Value::Int(1))),     // 4: return 1
                make_instruction(OpCode::Return),                    // 5
                make_instruction(OpCode::Load("n".to_string())),    // 6: else
                make_instruction(OpCode::Load("n".to_string())),    // 7
                make_instruction(OpCode::Const(Value::Int(1))),     // 8
                make_instruction(OpCode::Sub),                       // 9: n - 1
                make_instruction(OpCode::SelfCall(1)),               // 10: factorial(n-1)
                make_instruction(OpCode::Mul),                       // 11: n * result
                make_instruction(OpCode::Return),                    // 12
            ],
            local_count: 1,
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            assert_eq!(jit.call_int("factorial", &[0]).unwrap(), 1);
            assert_eq!(jit.call_int("factorial", &[1]).unwrap(), 1);
            assert_eq!(jit.call_int("factorial", &[5]).unwrap(), 120);
            assert_eq!(jit.call_int("factorial", &[10]).unwrap(), 3628800);
        }
    }

    #[test]
    fn test_tail_recursive_sum() {
        let mut jit = JitCompiler::new().unwrap();

        // sum_tail(n, acc) = n <= 0 ? acc : sum_tail(n - 1, acc + n)
        // IR:
        //   0: Load n
        //   1: Const 0
        //   2: Lte          ; n <= 0
        //   3: JumpIfNot 3  ; if false, jump to 6
        //   4: Load acc     ; return acc
        //   5: Return
        //   6: Load n       ; else: tail call
        //   7: Const 1
        //   8: Sub          ; n - 1
        //   9: Load acc
        //  10: Load n
        //  11: Add          ; acc + n
        //  12: TailSelfCall(2)
        let func = CompiledFunction {
            name: "sum_tail".to_string(),
            params: vec!["n".to_string(), "acc".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("n".to_string())),    // 0
                make_instruction(OpCode::Const(Value::Int(0))),     // 1
                make_instruction(OpCode::Lte),                       // 2
                make_instruction(OpCode::JumpIfNot(3)),              // 3: if !(n <= 0), jump to 6
                make_instruction(OpCode::Load("acc".to_string())),  // 4: return acc
                make_instruction(OpCode::Return),                    // 5
                make_instruction(OpCode::Load("n".to_string())),    // 6: n - 1
                make_instruction(OpCode::Const(Value::Int(1))),     // 7
                make_instruction(OpCode::Sub),                       // 8
                make_instruction(OpCode::Load("acc".to_string())),  // 9: acc + n
                make_instruction(OpCode::Load("n".to_string())),    // 10
                make_instruction(OpCode::Add),                       // 11
                make_instruction(OpCode::TailSelfCall(2)),           // 12: tail call
            ],
            local_count: 2,
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            // sum(10) = 10 + 9 + 8 + ... + 1 = 55
            assert_eq!(jit.call_int("sum_tail", &[10, 0]).unwrap(), 55);
            // sum(100) = 5050
            assert_eq!(jit.call_int("sum_tail", &[100, 0]).unwrap(), 5050);
            // sum(0) = 0
            assert_eq!(jit.call_int("sum_tail", &[0, 0]).unwrap(), 0);
        }
    }

    #[test]
    fn test_tail_recursive_factorial() {
        let mut jit = JitCompiler::new().unwrap();

        // fact_tail(n, acc) = n <= 1 ? acc : fact_tail(n - 1, acc * n)
        // IR:
        //   0: Load n
        //   1: Const 1
        //   2: Lte          ; n <= 1
        //   3: JumpIfNot 3  ; if false, jump to 6
        //   4: Load acc     ; return acc
        //   5: Return
        //   6: Load n       ; else: tail call
        //   7: Const 1
        //   8: Sub          ; n - 1
        //   9: Load acc
        //  10: Load n
        //  11: Mul          ; acc * n
        //  12: TailSelfCall(2)
        let func = CompiledFunction {
            name: "fact_tail".to_string(),
            params: vec!["n".to_string(), "acc".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("n".to_string())),    // 0
                make_instruction(OpCode::Const(Value::Int(1))),     // 1
                make_instruction(OpCode::Lte),                       // 2
                make_instruction(OpCode::JumpIfNot(3)),              // 3
                make_instruction(OpCode::Load("acc".to_string())),  // 4
                make_instruction(OpCode::Return),                    // 5
                make_instruction(OpCode::Load("n".to_string())),    // 6
                make_instruction(OpCode::Const(Value::Int(1))),     // 7
                make_instruction(OpCode::Sub),                       // 8
                make_instruction(OpCode::Load("acc".to_string())),  // 9
                make_instruction(OpCode::Load("n".to_string())),    // 10
                make_instruction(OpCode::Mul),                       // 11
                make_instruction(OpCode::TailSelfCall(2)),           // 12
            ],
            local_count: 2,
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            assert_eq!(jit.call_int("fact_tail", &[1, 1]).unwrap(), 1);
            assert_eq!(jit.call_int("fact_tail", &[5, 1]).unwrap(), 120);
            assert_eq!(jit.call_int("fact_tail", &[10, 1]).unwrap(), 3628800);
            // TCO를 사용하면 스택 오버플로우 없이 큰 값도 계산 가능
            assert_eq!(jit.call_int("fact_tail", &[20, 1]).unwrap(), 2432902008176640000);
        }
    }

    #[test]
    fn test_recursive_fibonacci() {
        let mut jit = JitCompiler::new().unwrap();

        // fib(n) = n <= 1 ? n : fib(n-1) + fib(n-2)
        // IR:
        //   0: Load n
        //   1: Const 1
        //   2: Lte          ; n <= 1
        //   3: JumpIfNot 3  ; if false, jump to 6
        //   4: Load n       ; return n
        //   5: Return
        //   6: Load n       ; fib(n-1)
        //   7: Const 1
        //   8: Sub
        //   9: SelfCall(1)
        //  10: Load n       ; fib(n-2)
        //  11: Const 2
        //  12: Sub
        //  13: SelfCall(1)
        //  14: Add          ; fib(n-1) + fib(n-2)
        //  15: Return
        let func = CompiledFunction {
            name: "fib".to_string(),
            params: vec!["n".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("n".to_string())),    // 0
                make_instruction(OpCode::Const(Value::Int(1))),     // 1
                make_instruction(OpCode::Lte),                       // 2
                make_instruction(OpCode::JumpIfNot(3)),              // 3
                make_instruction(OpCode::Load("n".to_string())),    // 4
                make_instruction(OpCode::Return),                    // 5
                make_instruction(OpCode::Load("n".to_string())),    // 6
                make_instruction(OpCode::Const(Value::Int(1))),     // 7
                make_instruction(OpCode::Sub),                       // 8
                make_instruction(OpCode::SelfCall(1)),               // 9
                make_instruction(OpCode::Load("n".to_string())),    // 10
                make_instruction(OpCode::Const(Value::Int(2))),     // 11
                make_instruction(OpCode::Sub),                       // 12
                make_instruction(OpCode::SelfCall(1)),               // 13
                make_instruction(OpCode::Add),                       // 14
                make_instruction(OpCode::Return),                    // 15
            ],
            local_count: 1,
        };

        jit.compile_function_int(&func).unwrap();

        unsafe {
            assert_eq!(jit.call_int("fib", &[0]).unwrap(), 0);
            assert_eq!(jit.call_int("fib", &[1]).unwrap(), 1);
            assert_eq!(jit.call_int("fib", &[2]).unwrap(), 1);
            assert_eq!(jit.call_int("fib", &[10]).unwrap(), 55);
            assert_eq!(jit.call_int("fib", &[20]).unwrap(), 6765);
        }
    }

    // === Inter-function Call Tests ===

    #[test]
    fn test_function_calls_simple() {
        let mut jit = JitCompiler::new().unwrap();

        // double(x) = x * 2
        let double_func = CompiledFunction {
            name: "double".to_string(),
            params: vec!["x".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("x".to_string())),
                make_instruction(OpCode::Const(Value::Int(2))),
                make_instruction(OpCode::Mul),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        // quadruple(x) = double(double(x))
        // 즉 x * 4
        let quadruple_func = CompiledFunction {
            name: "quadruple".to_string(),
            params: vec!["x".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("x".to_string())),
                make_instruction(OpCode::Call("double".to_string(), 1)),
                make_instruction(OpCode::Call("double".to_string(), 1)),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        // 배치 컴파일
        jit.compile_functions_batch(&[double_func, quadruple_func]).unwrap();

        unsafe {
            // double(5) = 10
            assert_eq!(jit.call_int("double", &[5]).unwrap(), 10);
            // quadruple(5) = double(double(5)) = double(10) = 20
            assert_eq!(jit.call_int("quadruple", &[5]).unwrap(), 20);
            // quadruple(10) = 40
            assert_eq!(jit.call_int("quadruple", &[10]).unwrap(), 40);
        }
    }

    #[test]
    fn test_function_calls_chain() {
        let mut jit = JitCompiler::new().unwrap();

        // add1(x) = x + 1
        let add1_func = CompiledFunction {
            name: "add1".to_string(),
            params: vec!["x".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("x".to_string())),
                make_instruction(OpCode::Const(Value::Int(1))),
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        // add2(x) = add1(add1(x)) = x + 2
        let add2_func = CompiledFunction {
            name: "add2".to_string(),
            params: vec!["x".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("x".to_string())),
                make_instruction(OpCode::Call("add1".to_string(), 1)),
                make_instruction(OpCode::Call("add1".to_string(), 1)),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        // add4(x) = add2(add2(x)) = x + 4
        let add4_func = CompiledFunction {
            name: "add4".to_string(),
            params: vec!["x".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("x".to_string())),
                make_instruction(OpCode::Call("add2".to_string(), 1)),
                make_instruction(OpCode::Call("add2".to_string(), 1)),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        jit.compile_functions_batch(&[add1_func, add2_func, add4_func]).unwrap();

        unsafe {
            assert_eq!(jit.call_int("add1", &[0]).unwrap(), 1);
            assert_eq!(jit.call_int("add2", &[0]).unwrap(), 2);
            assert_eq!(jit.call_int("add4", &[0]).unwrap(), 4);
            assert_eq!(jit.call_int("add4", &[100]).unwrap(), 104);
        }
    }

    #[test]
    fn test_function_calls_multiple_args() {
        let mut jit = JitCompiler::new().unwrap();

        // add(a, b) = a + b
        let add_func = CompiledFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Return),
            ],
            local_count: 2,
        };

        // mul(a, b) = a * b
        let mul_func = CompiledFunction {
            name: "mul".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Mul),
                make_instruction(OpCode::Return),
            ],
            local_count: 2,
        };

        // calc(a, b) = add(a, b) * mul(a, b) = (a+b) * (a*b)
        let calc_func = CompiledFunction {
            name: "calc".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Call("add".to_string(), 2)),
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Call("mul".to_string(), 2)),
                make_instruction(OpCode::Mul),
                make_instruction(OpCode::Return),
            ],
            local_count: 2,
        };

        jit.compile_functions_batch(&[add_func, mul_func, calc_func]).unwrap();

        unsafe {
            // add(3, 4) = 7
            assert_eq!(jit.call_int("add", &[3, 4]).unwrap(), 7);
            // mul(3, 4) = 12
            assert_eq!(jit.call_int("mul", &[3, 4]).unwrap(), 12);
            // calc(3, 4) = (3+4) * (3*4) = 7 * 12 = 84
            assert_eq!(jit.call_int("calc", &[3, 4]).unwrap(), 84);
            // calc(2, 5) = (2+5) * (2*5) = 7 * 10 = 70
            assert_eq!(jit.call_int("calc", &[2, 5]).unwrap(), 70);
        }
    }

    #[test]
    fn test_function_calls_with_control_flow() {
        let mut jit = JitCompiler::new().unwrap();

        // double(x) = x * 2
        let double_func = CompiledFunction {
            name: "double".to_string(),
            params: vec!["x".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("x".to_string())),
                make_instruction(OpCode::Const(Value::Int(2))),
                make_instruction(OpCode::Mul),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        // max_double(a, b) = a > b ? double(a) : double(b)
        // IR:
        //   0: Load a
        //   1: Load b
        //   2: Gt           ; a > b
        //   3: JumpIfNot 4  ; if false, jump to 7
        //   4: Load a       ; then: double(a)
        //   5: Call double, 1
        //   6: Jump 3       ; jump to 9
        //   7: Load b       ; else: double(b)
        //   8: Call double, 1
        //   9: Return
        let max_double_func = CompiledFunction {
            name: "max_double".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),    // 0
                make_instruction(OpCode::Load("b".to_string())),    // 1
                make_instruction(OpCode::Gt),                        // 2
                make_instruction(OpCode::JumpIfNot(4)),              // 3
                make_instruction(OpCode::Load("a".to_string())),    // 4
                make_instruction(OpCode::Call("double".to_string(), 1)), // 5
                make_instruction(OpCode::Jump(3)),                   // 6
                make_instruction(OpCode::Load("b".to_string())),    // 7
                make_instruction(OpCode::Call("double".to_string(), 1)), // 8
                make_instruction(OpCode::Return),                    // 9
            ],
            local_count: 2,
        };

        jit.compile_functions_batch(&[double_func, max_double_func]).unwrap();

        unsafe {
            // max_double(10, 5) = double(10) = 20
            assert_eq!(jit.call_int("max_double", &[10, 5]).unwrap(), 20);
            // max_double(3, 8) = double(8) = 16
            assert_eq!(jit.call_int("max_double", &[3, 8]).unwrap(), 16);
            // max_double(5, 5) = double(5) = 10 (a > b is false when equal)
            assert_eq!(jit.call_int("max_double", &[5, 5]).unwrap(), 10);
        }
    }

    #[test]
    fn test_function_calls_recursive_with_helper() {
        let mut jit = JitCompiler::new().unwrap();

        // add(a, b) = a + b
        let add_func = CompiledFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Return),
            ],
            local_count: 2,
        };

        // sum_recursive(n, acc) = n <= 0 ? acc : sum_recursive(n-1, add(acc, n))
        // 즉, sum(n) = 1 + 2 + ... + n
        // IR:
        //   0: Load n
        //   1: Const 0
        //   2: Lte          ; n <= 0
        //   3: JumpIfNot 3  ; if false, jump to 6
        //   4: Load acc
        //   5: Return
        //   6: Load n       ; n - 1
        //   7: Const 1
        //   8: Sub
        //   9: Load acc     ; add(acc, n)
        //  10: Load n
        //  11: Call add, 2
        //  12: TailSelfCall(2)
        let sum_func = CompiledFunction {
            name: "sum_recursive".to_string(),
            params: vec!["n".to_string(), "acc".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("n".to_string())),    // 0
                make_instruction(OpCode::Const(Value::Int(0))),     // 1
                make_instruction(OpCode::Lte),                       // 2
                make_instruction(OpCode::JumpIfNot(3)),              // 3
                make_instruction(OpCode::Load("acc".to_string())),  // 4
                make_instruction(OpCode::Return),                    // 5
                make_instruction(OpCode::Load("n".to_string())),    // 6
                make_instruction(OpCode::Const(Value::Int(1))),     // 7
                make_instruction(OpCode::Sub),                       // 8
                make_instruction(OpCode::Load("acc".to_string())),  // 9
                make_instruction(OpCode::Load("n".to_string())),    // 10
                make_instruction(OpCode::Call("add".to_string(), 2)), // 11
                make_instruction(OpCode::TailSelfCall(2)),           // 12
            ],
            local_count: 2,
        };

        jit.compile_functions_batch(&[add_func, sum_func]).unwrap();

        unsafe {
            // sum_recursive(10, 0) = 55
            assert_eq!(jit.call_int("sum_recursive", &[10, 0]).unwrap(), 55);
            // sum_recursive(100, 0) = 5050
            assert_eq!(jit.call_int("sum_recursive", &[100, 0]).unwrap(), 5050);
        }
    }

    // ========================================================================
    // Float JIT Tests
    // ========================================================================

    #[test]
    fn test_float_simple_add() {
        let mut jit = JitCompiler::new().unwrap();

        // add(a, b) = a + b
        let func = CompiledFunction {
            name: "add_float".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Return),
            ],
            local_count: 2,
        };

        jit.compile_function_float(&func).unwrap();

        unsafe {
            let result = jit.call_float("add_float", &[1.5, 2.5]).unwrap();
            assert!((result - 4.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_arithmetic() {
        let mut jit = JitCompiler::new().unwrap();

        // calc(a, b) = (a + b) * (a - b) / 2.0
        // = (a² - b²) / 2
        let func = CompiledFunction {
            name: "calc_float".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Add),                    // a + b
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Sub),                    // a - b
                make_instruction(OpCode::Mul),                    // (a+b) * (a-b)
                make_instruction(OpCode::Const(Value::Float(2.0))),
                make_instruction(OpCode::Div),                    // / 2.0
                make_instruction(OpCode::Return),
            ],
            local_count: 2,
        };

        jit.compile_function_float(&func).unwrap();

        unsafe {
            // (5+3)*(5-3)/2 = 8*2/2 = 8
            let result = jit.call_float("calc_float", &[5.0, 3.0]).unwrap();
            assert!((result - 8.0).abs() < 1e-10);

            // (10+2)*(10-2)/2 = 12*8/2 = 48
            let result = jit.call_float("calc_float", &[10.0, 2.0]).unwrap();
            assert!((result - 48.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_negation() {
        let mut jit = JitCompiler::new().unwrap();

        // neg(x) = -x
        let func = CompiledFunction {
            name: "neg_float".to_string(),
            params: vec!["x".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("x".to_string())),
                make_instruction(OpCode::Neg),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        jit.compile_function_float(&func).unwrap();

        unsafe {
            let result = jit.call_float("neg_float", &[3.15]).unwrap();
            assert!((result + 3.15).abs() < 1e-10);

            let result = jit.call_float("neg_float", &[-2.5]).unwrap();
            assert!((result - 2.5).abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_comparison() {
        let mut jit = JitCompiler::new().unwrap();

        // is_greater(a, b) = a > b ? 1.0 : 0.0
        let func = CompiledFunction {
            name: "is_greater".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Load("b".to_string())),
                make_instruction(OpCode::Gt),
                make_instruction(OpCode::Return),
            ],
            local_count: 2,
        };

        jit.compile_function_float(&func).unwrap();

        unsafe {
            let result = jit.call_float("is_greater", &[5.0, 3.0]).unwrap();
            assert!((result - 1.0).abs() < 1e-10);

            let result = jit.call_float("is_greater", &[2.0, 4.0]).unwrap();
            assert!(result.abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_conditional() {
        let mut jit = JitCompiler::new().unwrap();

        // max(a, b) = a > b ? a : b
        // IR:
        //   0: Load a
        //   1: Load b
        //   2: Gt           ; a > b
        //   3: JumpIfNot 3  ; if false, jump to 6
        //   4: Load a
        //   5: Return
        //   6: Load b
        //   7: Return
        let func = CompiledFunction {
            name: "max_float".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),  // 0
                make_instruction(OpCode::Load("b".to_string())),  // 1
                make_instruction(OpCode::Gt),                      // 2
                make_instruction(OpCode::JumpIfNot(3)),            // 3: jump to 6
                make_instruction(OpCode::Load("a".to_string())),  // 4
                make_instruction(OpCode::Return),                  // 5
                make_instruction(OpCode::Load("b".to_string())),  // 6
                make_instruction(OpCode::Return),                  // 7
            ],
            local_count: 2,
        };

        jit.compile_function_float(&func).unwrap();

        unsafe {
            let result = jit.call_float("max_float", &[5.0, 3.0]).unwrap();
            assert!((result - 5.0).abs() < 1e-10);

            let result = jit.call_float("max_float", &[2.0, 7.0]).unwrap();
            assert!((result - 7.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_recursive_sum() {
        let mut jit = JitCompiler::new().unwrap();

        // sum(n, acc) = n <= 0 ? acc : sum(n - 1, acc + n)
        // IR:
        //   0: Load n
        //   1: Const 0.0
        //   2: Lte          ; n <= 0
        //   3: JumpIfNot 3  ; if false, jump to 6
        //   4: Load acc
        //   5: Return
        //   6: Load n       ; n - 1
        //   7: Const 1.0
        //   8: Sub
        //   9: Load acc     ; acc + n
        //  10: Load n
        //  11: Add
        //  12: TailSelfCall(2)
        let func = CompiledFunction {
            name: "sum_float".to_string(),
            params: vec!["n".to_string(), "acc".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("n".to_string())),       // 0
                make_instruction(OpCode::Const(Value::Float(0.0))),    // 1
                make_instruction(OpCode::Lte),                          // 2
                make_instruction(OpCode::JumpIfNot(3)),                 // 3
                make_instruction(OpCode::Load("acc".to_string())),     // 4
                make_instruction(OpCode::Return),                       // 5
                make_instruction(OpCode::Load("n".to_string())),       // 6
                make_instruction(OpCode::Const(Value::Float(1.0))),    // 7
                make_instruction(OpCode::Sub),                          // 8
                make_instruction(OpCode::Load("acc".to_string())),     // 9
                make_instruction(OpCode::Load("n".to_string())),       // 10
                make_instruction(OpCode::Add),                          // 11
                make_instruction(OpCode::TailSelfCall(2)),              // 12
            ],
            local_count: 2,
        };

        jit.compile_function_float(&func).unwrap();

        unsafe {
            // sum(10, 0) = 55
            let result = jit.call_float("sum_float", &[10.0, 0.0]).unwrap();
            assert!((result - 55.0).abs() < 1e-10);

            // sum(100, 0) = 5050
            let result = jit.call_float("sum_float", &[100.0, 0.0]).unwrap();
            assert!((result - 5050.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_int_conversion() {
        let mut jit = JitCompiler::new().unwrap();

        // add_int_to_float(a) = a + 10 (10 is int constant)
        let func = CompiledFunction {
            name: "add_int_to_float".to_string(),
            params: vec!["a".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Const(Value::Int(10))),  // Int constant
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        jit.compile_function_float(&func).unwrap();

        unsafe {
            let result = jit.call_float("add_int_to_float", &[2.5]).unwrap();
            assert!((result - 12.5).abs() < 1e-10);
        }
    }

    #[test]
    fn test_analyze_function_type() {
        // Int only function
        let int_func = CompiledFunction {
            name: "int_func".to_string(),
            params: vec!["a".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Const(Value::Int(1))),
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        match JitCompiler::analyze_function_type(&int_func) {
            FnSignature::IntOnly { param_count } => assert_eq!(param_count, 1),
            _ => panic!("Expected IntOnly"),
        }

        // Float function
        let float_func = CompiledFunction {
            name: "float_func".to_string(),
            params: vec!["a".to_string()],
            instructions: vec![
                make_instruction(OpCode::Load("a".to_string())),
                make_instruction(OpCode::Const(Value::Float(1.5))),
                make_instruction(OpCode::Add),
                make_instruction(OpCode::Return),
            ],
            local_count: 1,
        };

        match JitCompiler::analyze_function_type(&float_func) {
            FnSignature::FloatOnly { param_count } => assert_eq!(param_count, 1),
            _ => panic!("Expected FloatOnly"),
        }
    }
}
