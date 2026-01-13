//! AOEL VM Implementation
//!
//! 스택 기반 VM으로 AOEL IR을 실행

use std::collections::HashMap;
use aoel_ir::{Instruction, OpCode, ReduceOp, Value};
use aoel_lowering::CompiledFunction;
use crate::error::{RuntimeError, RuntimeResult};
use crate::ffi::FfiLoader;

const MAX_RECURSION_DEPTH: usize = 1000;

/// Result type for TCO-aware execution
enum TcoResult {
    /// Normal return
    Return,
    /// Tail call with new arguments
    TailCall(Vec<Value>),
}

/// AOEL Virtual Machine
pub struct Vm {
    /// 스택
    stack: Vec<Value>,
    /// 로컬 변수
    locals: HashMap<String, Value>,
    /// 컴파일된 함수들
    functions: HashMap<String, CompiledFunction>,
    /// 현재 실행 중인 함수 (재귀용)
    current_function: Option<String>,
    /// 재귀 깊이
    recursion_depth: usize,
    /// 클로저 저장소 (body_id -> instructions)
    closures: HashMap<usize, Vec<Instruction>>,
    /// 다음 클로저 ID
    next_closure_id: usize,
    /// FFI 동적 라이브러리 로더
    ffi_loader: FfiLoader,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            locals: HashMap::new(),
            functions: HashMap::new(),
            current_function: None,
            recursion_depth: 0,
            closures: HashMap::new(),
            next_closure_id: 0,
            ffi_loader: FfiLoader::new(),
        }
    }

    /// FFI 함수 등록 (동적 라이브러리용)
    pub fn register_ffi_function(
        &mut self,
        lib_name: &str,
        fn_name: &str,
        params: Vec<crate::ffi::FfiType>,
        return_type: crate::ffi::FfiType,
    ) {
        self.ffi_loader.register_function(lib_name, fn_name, params, return_type);
    }

    /// FFI 라이브러리 검색 경로 추가
    pub fn add_ffi_search_path(&mut self, path: &str) {
        self.ffi_loader.add_search_path(path);
    }

    /// FFI 로더에 대한 참조 반환
    pub fn ffi_loader(&self) -> &FfiLoader {
        &self.ffi_loader
    }

    /// FFI 로더에 대한 가변 참조 반환
    pub fn ffi_loader_mut(&mut self) -> &mut FfiLoader {
        &mut self.ffi_loader
    }

    /// 함수들을 로드
    pub fn load_functions(&mut self, functions: Vec<CompiledFunction>) {
        for func in functions {
            self.functions.insert(func.name.clone(), func);
        }
    }

    /// 함수 호출
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> RuntimeResult<Value> {
        let func = self.functions.get(name)
            .ok_or_else(|| RuntimeError::UndefinedFunction(name.to_string()))?
            .clone();

        // 재귀 깊이 체크
        self.recursion_depth += 1;
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            return Err(RuntimeError::MaxRecursionDepth);
        }

        // 이전 상태 저장 (스택 포함!)
        let prev_stack = std::mem::take(&mut self.stack);
        let prev_locals = std::mem::take(&mut self.locals);
        let prev_function = self.current_function.take();

        // 인자를 로컬 변수로 설정
        let mut current_args = args;
        for (i, param) in func.params.iter().enumerate() {
            if i < current_args.len() {
                self.locals.insert(param.clone(), current_args[i].clone());
            }
        }

        // 현재 함수 설정 (SelfCall용)
        self.current_function = Some(name.to_string());

        // TCO loop: execute instructions, and if TailSelfCall is encountered,
        // restart the function with new arguments instead of making a recursive call
        loop {
            // 명령어 실행
            let result = self.execute_instructions_with_tco(&func.instructions, &func.params);

            match result {
                Ok(TcoResult::Return) => {
                    // Normal return - get result from stack
                    let return_value = self.pop()?;

                    // 상태 복원
                    self.stack = prev_stack;
                    self.locals = prev_locals;
                    self.current_function = prev_function;
                    self.recursion_depth -= 1;

                    return Ok(return_value);
                }
                Ok(TcoResult::TailCall(new_args)) => {
                    // Tail call - restart with new arguments (no stack growth)
                    current_args = new_args;
                    self.stack.clear();
                    self.locals.clear();

                    // Rebind parameters
                    for (i, param) in func.params.iter().enumerate() {
                        if i < current_args.len() {
                            self.locals.insert(param.clone(), current_args[i].clone());
                        }
                    }
                    // Loop continues without increasing recursion depth
                }
                Err(e) => {
                    // 상태 복원
                    self.stack = prev_stack;
                    self.locals = prev_locals;
                    self.current_function = prev_function;
                    self.recursion_depth -= 1;
                    return Err(e);
                }
            }
        }
    }

    /// 명령어 시퀀스 실행
    fn execute_instructions(&mut self, instructions: &[Instruction]) -> RuntimeResult<()> {
        let mut ip = 0;
        let len = instructions.len();

        while ip < len {
            let instr = &instructions[ip];

            match &instr.opcode {
                // Jump 처리
                OpCode::Jump(offset) => {
                    let new_ip = (ip as i64) + (*offset as i64) + 1;
                    if new_ip < 0 || new_ip > len as i64 {
                        return Err(RuntimeError::Internal(format!(
                            "Jump out of bounds: ip={}, offset={}, target={}",
                            ip, offset, new_ip
                        )));
                    }
                    ip = new_ip as usize;
                    continue;
                }
                OpCode::JumpIf(offset) => {
                    let cond = self.pop()?;
                    if cond.is_truthy() {
                        let new_ip = (ip as i64) + (*offset as i64) + 1;
                        if new_ip < 0 || new_ip > len as i64 {
                            return Err(RuntimeError::Internal(format!(
                                "JumpIf out of bounds: ip={}, offset={}, target={}",
                                ip, offset, new_ip
                            )));
                        }
                        ip = new_ip as usize;
                        continue;
                    }
                }
                OpCode::JumpIfNot(offset) => {
                    let cond = self.pop()?;
                    if !cond.is_truthy() {
                        let new_ip = (ip as i64) + (*offset as i64) + 1;
                        if new_ip < 0 || new_ip > len as i64 {
                            return Err(RuntimeError::Internal(format!(
                                "JumpIfNot out of bounds: ip={}, offset={}, target={}",
                                ip, offset, new_ip
                            )));
                        }
                        ip = new_ip as usize;
                        continue;
                    }
                }
                OpCode::Return => {
                    return Ok(());
                }
                _ => {
                    self.execute_instruction(instr)?;
                }
            }
            ip += 1;
        }
        Ok(())
    }

    /// TCO-aware instruction execution
    /// Returns TcoResult::TailCall if a tail call is encountered
    fn execute_instructions_with_tco(
        &mut self,
        instructions: &[Instruction],
        _params: &[String],
    ) -> RuntimeResult<TcoResult> {
        let mut ip = 0;
        let len = instructions.len();

        while ip < len {
            let instr = &instructions[ip];

            match &instr.opcode {
                // Jump 처리
                OpCode::Jump(offset) => {
                    let new_ip = (ip as i64) + (*offset as i64) + 1;
                    if new_ip < 0 || new_ip > len as i64 {
                        return Err(RuntimeError::Internal(format!(
                            "Jump out of bounds: ip={}, offset={}, target={}",
                            ip, offset, new_ip
                        )));
                    }
                    ip = new_ip as usize;
                    continue;
                }
                OpCode::JumpIf(offset) => {
                    let cond = self.pop()?;
                    if cond.is_truthy() {
                        let new_ip = (ip as i64) + (*offset as i64) + 1;
                        if new_ip < 0 || new_ip > len as i64 {
                            return Err(RuntimeError::Internal(format!(
                                "JumpIf out of bounds: ip={}, offset={}, target={}",
                                ip, offset, new_ip
                            )));
                        }
                        ip = new_ip as usize;
                        continue;
                    }
                }
                OpCode::JumpIfNot(offset) => {
                    let cond = self.pop()?;
                    if !cond.is_truthy() {
                        let new_ip = (ip as i64) + (*offset as i64) + 1;
                        if new_ip < 0 || new_ip > len as i64 {
                            return Err(RuntimeError::Internal(format!(
                                "JumpIfNot out of bounds: ip={}, offset={}, target={}",
                                ip, offset, new_ip
                            )));
                        }
                        ip = new_ip as usize;
                        continue;
                    }
                }
                OpCode::Return => {
                    return Ok(TcoResult::Return);
                }
                OpCode::TailSelfCall(arg_count) => {
                    // Collect arguments from stack
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    // Signal tail call to caller
                    return Ok(TcoResult::TailCall(args));
                }
                _ => {
                    self.execute_instruction(instr)?;
                }
            }
            ip += 1;
        }
        Ok(TcoResult::Return)
    }

    /// 단일 명령어 실행
    fn execute_instruction(&mut self, instr: &Instruction) -> RuntimeResult<()> {
        match &instr.opcode {
            // === Stack Operations ===
            OpCode::Const(value) => {
                self.stack.push(value.clone());
            }
            OpCode::Pop => {
                self.pop()?;
            }
            OpCode::Dup => {
                let val = self.peek()?.clone();
                self.stack.push(val);
            }

            // === Variable Operations ===
            OpCode::Load(name) => {
                let value = self.locals.get(name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()))?
                    .clone();
                self.stack.push(value);
            }
            OpCode::Store(name) => {
                let value = self.pop()?;
                self.locals.insert(name.clone(), value);
            }

            // === Arithmetic Operations ===
            OpCode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.binary_add(a, b)?;
                self.stack.push(result);
            }
            OpCode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.binary_sub(a, b)?;
                self.stack.push(result);
            }
            OpCode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.binary_mul(a, b)?;
                self.stack.push(result);
            }
            OpCode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.binary_div(a, b)?;
                self.stack.push(result);
            }
            OpCode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.binary_mod(a, b)?;
                self.stack.push(result);
            }
            OpCode::Neg => {
                let a = self.pop()?;
                let result = self.unary_neg(a)?;
                self.stack.push(result);
            }

            // === Comparison Operations ===
            OpCode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(a == b));
            }
            OpCode::Neq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(a != b));
            }
            OpCode::Lt => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(self.compare_lt(&a, &b)?));
            }
            OpCode::Gt => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(self.compare_lt(&b, &a)?));
            }
            OpCode::Lte => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(!self.compare_lt(&b, &a)?));
            }
            OpCode::Gte => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(!self.compare_lt(&a, &b)?));
            }

            // === Logical Operations ===
            OpCode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(a.is_truthy() && b.is_truthy()));
            }
            OpCode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(a.is_truthy() || b.is_truthy()));
            }
            OpCode::Not => {
                let a = self.pop()?;
                self.stack.push(Value::Bool(!a.is_truthy()));
            }

            // === Collection Operations ===
            OpCode::Len => {
                let a = self.pop()?;
                let len = match &a {
                    Value::Array(arr) => arr.len(),
                    Value::String(s) => s.chars().count(), // Unicode char count
                    Value::Map(m) => m.len(),
                    _ => 0,
                };
                self.stack.push(Value::Int(len as i64));
            }
            OpCode::Index => {
                let index = self.pop()?;
                let base = self.pop()?;
                let result = self.index_access(base, index)?;
                self.stack.push(result);
            }
            OpCode::GetField(name) => {
                let base = self.pop()?;
                let result = self.field_access(base, name)?;
                self.stack.push(result);
            }
            OpCode::MakeArray(count) => {
                let mut items = Vec::new();
                for _ in 0..*count {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.stack.push(Value::Array(items));
            }
            OpCode::MakeStruct(fields) => {
                let mut struct_val = HashMap::new();
                for field in fields.iter().rev() {
                    let value = self.pop()?;
                    struct_val.insert(field.clone(), value);
                }
                self.stack.push(Value::Struct(struct_val));
            }
            OpCode::Slice => {
                let end = self.pop()?;
                let start = self.pop()?;
                let base = self.pop()?;
                let result = self.slice_access(base, start, end)?;
                self.stack.push(result);
            }
            OpCode::Range => {
                let end = self.pop()?;
                let start = self.pop()?;
                let result = self.create_range(start, end)?;
                self.stack.push(result);
            }
            OpCode::Contains => {
                let arr = self.pop()?;
                let elem = self.pop()?;
                let result = self.contains_check(&elem, &arr)?;
                self.stack.push(Value::Bool(result));
            }
            OpCode::Concat => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.concat_values(a, b)?;
                self.stack.push(result);
            }

            // === Collection Operations (Map/Filter/Reduce) ===
            OpCode::Map(transform_instrs) => {
                let arr = self.pop()?;
                if let Value::Array(items) = arr {
                    let mut results = Vec::new();
                    for item in items {
                        // _ 변수에 현재 아이템 저장
                        self.locals.insert("_".to_string(), item);
                        self.execute_instructions(transform_instrs)?;
                        results.push(self.pop()?);
                    }
                    self.stack.push(Value::Array(results));
                } else {
                    self.stack.push(Value::Array(vec![]));
                }
            }
            OpCode::Filter(pred_instrs) => {
                let arr = self.pop()?;
                if let Value::Array(items) = arr {
                    let mut results = Vec::new();
                    for item in items {
                        self.locals.insert("_".to_string(), item.clone());
                        self.execute_instructions(pred_instrs)?;
                        let keep = self.pop()?;
                        if keep.is_truthy() {
                            results.push(item);
                        }
                    }
                    self.stack.push(Value::Array(results));
                } else {
                    self.stack.push(Value::Array(vec![]));
                }
            }
            OpCode::Reduce(reduce_op, _init) => {
                let arr = self.pop()?;
                if let Value::Array(items) = arr {
                    let result = match reduce_op {
                        ReduceOp::Sum => {
                            let mut sum = 0i64;
                            for item in items {
                                if let Some(n) = item.as_int() {
                                    sum += n;
                                } else if let Some(f) = item.as_float() {
                                    sum += f as i64;
                                }
                            }
                            Value::Int(sum)
                        }
                        ReduceOp::Product => {
                            let mut product = 1i64;
                            for item in items {
                                if let Some(n) = item.as_int() {
                                    product *= n;
                                }
                            }
                            Value::Int(product)
                        }
                        ReduceOp::Min => {
                            items.into_iter().min_by(|a, b| {
                                a.as_int().unwrap_or(0).cmp(&b.as_int().unwrap_or(0))
                            }).unwrap_or(Value::Void)
                        }
                        ReduceOp::Max => {
                            items.into_iter().max_by(|a, b| {
                                a.as_int().unwrap_or(0).cmp(&b.as_int().unwrap_or(0))
                            }).unwrap_or(Value::Void)
                        }
                        ReduceOp::All => {
                            Value::Bool(items.iter().all(|i| i.is_truthy()))
                        }
                        ReduceOp::Any => {
                            Value::Bool(items.iter().any(|i| i.is_truthy()))
                        }
                        ReduceOp::Count => Value::Int(items.len() as i64),
                        ReduceOp::First => items.first().cloned().unwrap_or(Value::Void),
                        ReduceOp::Last => items.last().cloned().unwrap_or(Value::Void),
                        ReduceOp::Avg => {
                            if items.is_empty() {
                                Value::Float(0.0)
                            } else {
                                let sum: f64 = items.iter()
                                    .filter_map(|i| i.as_float())
                                    .sum();
                                Value::Float(sum / items.len() as f64)
                            }
                        }
                        ReduceOp::Custom(reducer_instrs) => {
                            // Custom reduce: fold with provided function
                            // reducer_instrs expects _ (current element) and __acc__ (accumulator)
                            if items.is_empty() {
                                Value::Void
                            } else {
                                let mut acc = items[0].clone();
                                for item in items.into_iter().skip(1) {
                                    // Set up variables for reducer
                                    self.locals.insert("_".to_string(), item);
                                    self.locals.insert("__acc__".to_string(), acc.clone());
                                    self.execute_instructions(reducer_instrs)?;
                                    acc = self.pop()?;
                                }
                                acc
                            }
                        }
                    };
                    self.stack.push(result);
                } else {
                    self.stack.push(Value::Void);
                }
            }

            // === Function Calls ===
            OpCode::Call(name, arg_count) => {
                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(self.pop()?);
                }
                args.reverse();

                // 빌트인 함수 체크
                if let Some(result) = self.call_builtin(name, &args)? {
                    self.stack.push(result);
                } else {
                    // 사용자 정의 함수 호출
                    let result = self.call_function(name, args)?;
                    self.stack.push(result);
                }
            }
            OpCode::SelfCall(arg_count) => {
                let func_name = self.current_function.clone()
                    .ok_or_else(|| RuntimeError::Internal("SelfCall outside function".to_string()))?;

                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(self.pop()?);
                }
                args.reverse();

                let result = self.call_function(&func_name, args)?;
                self.stack.push(result);
            }

            // Tail-recursive self-call (optimized)
            // This is handled specially in execute_instructions to implement TCO
            OpCode::TailSelfCall(_) => {
                return Err(RuntimeError::Internal(
                    "TailSelfCall should be handled in execute_instructions".to_string()
                ));
            }

            // === Closure Operations ===
            OpCode::MakeClosure(params, body) => {
                // 클로저 본문을 저장하고 ID 할당
                let body_id = self.next_closure_id;
                self.next_closure_id += 1;
                self.closures.insert(body_id, body.as_ref().clone());

                // 현재 환경에서 변수 캡처 (단순 구현: 모든 로컬 변수 캡처)
                let captured = self.locals.clone();

                self.stack.push(Value::Closure {
                    params: params.clone(),
                    captured,
                    body_id,
                });
            }

            OpCode::CallClosure(arg_count) => {
                // 인자들을 먼저 팝
                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(self.pop()?);
                }
                args.reverse();

                // 클로저를 팝
                let closure = self.pop()?;

                if let Value::Closure { params, captured, body_id } = closure {
                    // 클로저 본문 가져오기
                    let body = self.closures.get(&body_id)
                        .ok_or_else(|| RuntimeError::Internal("Closure body not found".to_string()))?
                        .clone();

                    // 상태 저장
                    let prev_locals = std::mem::take(&mut self.locals);

                    // 캡처된 환경 복원
                    self.locals = captured;

                    // 인자 바인딩
                    for (i, param) in params.iter().enumerate() {
                        if i < args.len() {
                            self.locals.insert(param.clone(), args[i].clone());
                        }
                    }

                    // 본문 실행
                    self.execute_instructions(&body)?;
                    let result = self.pop()?;

                    // 상태 복원
                    self.locals = prev_locals;

                    self.stack.push(result);
                } else {
                    return Err(RuntimeError::TypeError("Cannot call non-closure".to_string()));
                }
            }

            // === Optional/Error Handling ===
            OpCode::Try => {
                let value = self.pop()?;
                match value {
                    Value::Optional(Some(v)) => self.stack.push(*v),
                    Value::Optional(None) => {
                        return Err(RuntimeError::Internal("Unwrap on None".to_string()));
                    }
                    other => self.stack.push(other),
                }
            }
            OpCode::Coalesce => {
                let default = self.pop()?;
                let value = self.pop()?;
                match value {
                    Value::Optional(Some(v)) => self.stack.push(*v),
                    Value::Optional(None) | Value::Void => self.stack.push(default),
                    other => self.stack.push(other),
                }
            }

            // === Builtin Function Call ===
            OpCode::CallBuiltin(name, arg_count) => {
                let mut args = Vec::with_capacity(*arg_count);
                for _ in 0..*arg_count {
                    args.push(self.pop()?);
                }
                args.reverse(); // 스택에서 역순으로 pop했으므로 원래 순서로 복원

                if let Some(result) = self.call_builtin(name, &args)? {
                    self.stack.push(result);
                } else {
                    // builtin이 None을 반환하면 Void 푸시
                    self.stack.push(Value::Void);
                }
            }

            // === FFI Function Call ===
            OpCode::CallFfi(lib_name, fn_name, arg_count) => {
                let mut args = Vec::with_capacity(*arg_count);
                for _ in 0..*arg_count {
                    args.push(self.pop()?);
                }
                args.reverse();

                let result = self.call_ffi(lib_name, fn_name, &args)?;
                self.stack.push(result);
            }

            // === Special ===
            OpCode::Nop => {}
            OpCode::Halt => {
                return Err(RuntimeError::Internal("Halt".to_string()));
            }
            OpCode::Error(msg) => {
                return Err(RuntimeError::Internal(msg.clone()));
            }

            // Unhandled opcodes - should not happen
            other => {
                return Err(RuntimeError::Internal(format!("Unhandled opcode: {:?}", other)));
            }
        }
        Ok(())
    }

    // === Helper Functions ===

    fn pop(&mut self) -> RuntimeResult<Value> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    fn peek(&self) -> RuntimeResult<&Value> {
        self.stack.last().ok_or(RuntimeError::StackUnderflow)
    }

    fn binary_add(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 + y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + y as f64)),
            (Value::String(x), Value::String(y)) => Ok(Value::String(format!("{}{}", x, y))),
            (Value::Array(mut x), Value::Array(y)) => {
                x.extend(y);
                Ok(Value::Array(x))
            }
            _ => Err(RuntimeError::TypeError("Cannot add these types".to_string())),
        }
    }

    fn binary_sub(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 - y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x - y as f64)),
            _ => Err(RuntimeError::TypeError("Cannot subtract these types".to_string())),
        }
    }

    fn binary_mul(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 * y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x * y as f64)),
            _ => Err(RuntimeError::TypeError("Cannot multiply these types".to_string())),
        }
    }

    fn binary_div(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 { return Err(RuntimeError::DivisionByZero); }
                Ok(Value::Int(x / y))
            }
            (Value::Float(x), Value::Float(y)) => {
                if y == 0.0 { return Err(RuntimeError::DivisionByZero); }
                Ok(Value::Float(x / y))
            }
            (Value::Int(x), Value::Float(y)) => {
                if y == 0.0 { return Err(RuntimeError::DivisionByZero); }
                Ok(Value::Float(x as f64 / y))
            }
            (Value::Float(x), Value::Int(y)) => {
                if y == 0 { return Err(RuntimeError::DivisionByZero); }
                Ok(Value::Float(x / y as f64))
            }
            _ => Err(RuntimeError::TypeError("Cannot divide these types".to_string())),
        }
    }

    fn binary_mod(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 { return Err(RuntimeError::DivisionByZero); }
                Ok(Value::Int(x % y))
            }
            _ => Err(RuntimeError::TypeError("Cannot modulo these types".to_string())),
        }
    }

    fn unary_neg(&self, a: Value) -> RuntimeResult<Value> {
        match a {
            Value::Int(x) => Ok(Value::Int(-x)),
            Value::Float(x) => Ok(Value::Float(-x)),
            _ => Err(RuntimeError::TypeError("Cannot negate this type".to_string())),
        }
    }

    fn compare_lt(&self, a: &Value, b: &Value) -> RuntimeResult<bool> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(x < y),
            (Value::Float(x), Value::Float(y)) => Ok(x < y),
            (Value::Int(x), Value::Float(y)) => Ok((*x as f64) < *y),
            (Value::Float(x), Value::Int(y)) => Ok(*x < (*y as f64)),
            (Value::String(x), Value::String(y)) => Ok(x < y),
            _ => Ok(false),
        }
    }

    fn index_access(&self, base: Value, index: Value) -> RuntimeResult<Value> {
        match (base, index) {
            (Value::Array(arr), Value::Int(i)) => {
                let idx = if i < 0 { arr.len() as i64 + i } else { i };
                if idx < 0 || idx as usize >= arr.len() {
                    return Err(RuntimeError::IndexOutOfBounds { index: i, length: arr.len() });
                }
                Ok(arr[idx as usize].clone())
            }
            (Value::String(s), Value::Int(i)) => {
                // Use char count for proper Unicode handling
                let char_count = s.chars().count();
                let idx = if i < 0 { char_count as i64 + i } else { i };
                if idx < 0 || idx as usize >= char_count {
                    return Err(RuntimeError::IndexOutOfBounds { index: i, length: char_count });
                }
                // Safe: we've validated the index is in bounds
                let ch = s.chars().nth(idx as usize)
                    .ok_or(RuntimeError::IndexOutOfBounds { index: i, length: char_count })?;
                Ok(Value::String(ch.to_string()))
            }
            (Value::Map(m), Value::String(key)) => {
                Ok(m.get(&key).cloned().unwrap_or(Value::Void))
            }
            _ => Ok(Value::Void),
        }
    }

    fn field_access(&self, base: Value, field: &str) -> RuntimeResult<Value> {
        match base {
            Value::Struct(s) => Ok(s.get(field).cloned().unwrap_or(Value::Void)),
            Value::Map(m) => Ok(m.get(field).cloned().unwrap_or(Value::Void)),
            _ => Err(RuntimeError::InvalidFieldAccess { field: field.to_string() }),
        }
    }

    fn slice_access(&self, base: Value, start: Value, end: Value) -> RuntimeResult<Value> {
        let start_idx = start.as_int().unwrap_or(0) as usize;
        let end_idx = end.as_int().unwrap_or(-1);

        match base {
            Value::Array(arr) => {
                let len = arr.len();
                let actual_end = if end_idx < 0 { len } else { (end_idx as usize).min(len) };
                let actual_start = start_idx.min(len);
                Ok(Value::Array(arr[actual_start..actual_end].to_vec()))
            }
            Value::String(s) => {
                // Use char count for proper Unicode handling
                let char_count = s.chars().count();
                let actual_end = if end_idx < 0 { char_count } else { (end_idx as usize).min(char_count) };
                let actual_start = start_idx.min(char_count);
                Ok(Value::String(s.chars().skip(actual_start).take(actual_end.saturating_sub(actual_start)).collect()))
            }
            _ => Ok(Value::Void),
        }
    }

    fn create_range(&self, start: Value, end: Value) -> RuntimeResult<Value> {
        match (start, end) {
            (Value::Int(s), Value::Int(e)) => {
                let range: Vec<Value> = (s..e).map(Value::Int).collect();
                Ok(Value::Array(range))
            }
            _ => Ok(Value::Array(vec![])),
        }
    }

    fn contains_check(&self, elem: &Value, container: &Value) -> RuntimeResult<bool> {
        match container {
            Value::Array(arr) => Ok(arr.contains(elem)),
            Value::String(s) => {
                if let Value::String(e) = elem {
                    Ok(s.contains(e.as_str()))
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    fn concat_values(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Array(mut x), Value::Array(y)) => {
                x.extend(y);
                Ok(Value::Array(x))
            }
            (Value::String(x), Value::String(y)) => {
                Ok(Value::String(format!("{}{}", x, y)))
            }
            _ => Err(RuntimeError::TypeError("Cannot concat these types".to_string())),
        }
    }

    /// 빌트인 함수 호출
    fn call_builtin(&self, name: &str, args: &[Value]) -> RuntimeResult<Option<Value>> {
        let result = match name.to_uppercase().as_str() {
            // === Collection functions ===
            "LEN" => {
                if let Some(v) = args.first() {
                    Some(Value::Int(v.len().unwrap_or(0) as i64))
                } else {
                    Some(Value::Int(0))
                }
            }
            "FIRST" => {
                if let Some(Value::Array(arr)) = args.first() {
                    arr.first().cloned()
                } else {
                    None
                }
            }
            "LAST" => {
                if let Some(Value::Array(arr)) = args.first() {
                    arr.last().cloned()
                } else {
                    None
                }
            }
            "REVERSE" => {
                if let Some(Value::Array(arr)) = args.first() {
                    let mut rev = arr.clone();
                    rev.reverse();
                    Some(Value::Array(rev))
                } else if let Some(Value::String(s)) = args.first() {
                    Some(Value::String(s.chars().rev().collect()))
                } else {
                    None
                }
            }
            "CONCAT" => {
                if args.len() >= 2 {
                    match (&args[0], &args[1]) {
                        (Value::Array(a), Value::Array(b)) => {
                            let mut result = a.clone();
                            result.extend(b.clone());
                            Some(Value::Array(result))
                        }
                        (Value::String(a), Value::String(b)) => {
                            Some(Value::String(format!("{}{}", a, b)))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "RANGE" => {
                match args.len() {
                    1 => {
                        if let Value::Int(end) = args[0] {
                            Some(Value::Array((0..end).map(Value::Int).collect()))
                        } else {
                            None
                        }
                    }
                    2 => {
                        if let (Value::Int(start), Value::Int(end)) = (&args[0], &args[1]) {
                            Some(Value::Array((*start..*end).map(Value::Int).collect()))
                        } else {
                            None
                        }
                    }
                    3 => {
                        if let (Value::Int(start), Value::Int(end), Value::Int(step)) = (&args[0], &args[1], &args[2]) {
                            let mut result = Vec::new();
                            let mut i = *start;
                            while (step > &0 && i < *end) || (step < &0 && i > *end) {
                                result.push(Value::Int(i));
                                i += step;
                            }
                            Some(Value::Array(result))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }

            // === Math functions ===
            "ABS" => {
                if let Some(Value::Int(n)) = args.first() {
                    Some(Value::Int(n.abs()))
                } else if let Some(Value::Float(f)) = args.first() {
                    Some(Value::Float(f.abs()))
                } else {
                    None
                }
            }
            "SQRT" => {
                if let Some(Value::Int(n)) = args.first() {
                    Some(Value::Float((*n as f64).sqrt()))
                } else if let Some(Value::Float(f)) = args.first() {
                    Some(Value::Float(f.sqrt()))
                } else {
                    None
                }
            }
            "POW" => {
                if args.len() >= 2 {
                    let base = match &args[0] {
                        Value::Int(n) => *n as f64,
                        Value::Float(f) => *f,
                        _ => return Ok(None),
                    };
                    let exp = match &args[1] {
                        Value::Int(n) => *n as f64,
                        Value::Float(f) => *f,
                        _ => return Ok(None),
                    };
                    Some(Value::Float(base.powf(exp)))
                } else {
                    None
                }
            }
            "SIN" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).sin())),
                    Some(Value::Float(f)) => Some(Value::Float(f.sin())),
                    _ => None,
                }
            }
            "COS" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).cos())),
                    Some(Value::Float(f)) => Some(Value::Float(f.cos())),
                    _ => None,
                }
            }
            "TAN" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).tan())),
                    Some(Value::Float(f)) => Some(Value::Float(f.tan())),
                    _ => None,
                }
            }
            "LOG" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).ln())),
                    Some(Value::Float(f)) => Some(Value::Float(f.ln())),
                    _ => None,
                }
            }
            "LOG10" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).log10())),
                    Some(Value::Float(f)) => Some(Value::Float(f.log10())),
                    _ => None,
                }
            }
            "FLOOR" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Int(*n)),
                    Some(Value::Float(f)) => Some(Value::Int(f.floor() as i64)),
                    _ => None,
                }
            }
            "CEIL" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Int(*n)),
                    Some(Value::Float(f)) => Some(Value::Int(f.ceil() as i64)),
                    _ => None,
                }
            }
            "ROUND" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Int(*n)),
                    Some(Value::Float(f)) => Some(Value::Int(f.round() as i64)),
                    _ => None,
                }
            }
            "MIN" => {
                if args.len() >= 2 {
                    match (&args[0], &args[1]) {
                        (Value::Int(a), Value::Int(b)) => Some(Value::Int(*a.min(b))),
                        (Value::Float(a), Value::Float(b)) => Some(Value::Float(a.min(*b))),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "MAX" => {
                if args.len() >= 2 {
                    match (&args[0], &args[1]) {
                        (Value::Int(a), Value::Int(b)) => Some(Value::Int(*a.max(b))),
                        (Value::Float(a), Value::Float(b)) => Some(Value::Float(a.max(*b))),
                        _ => None,
                    }
                } else {
                    None
                }
            }

            // === String functions ===
            "UPPER" => {
                if let Some(Value::String(s)) = args.first() {
                    Some(Value::String(s.to_uppercase()))
                } else {
                    None
                }
            }
            "LOWER" => {
                if let Some(Value::String(s)) = args.first() {
                    Some(Value::String(s.to_lowercase()))
                } else {
                    None
                }
            }
            "TRIM" => {
                if let Some(Value::String(s)) = args.first() {
                    Some(Value::String(s.trim().to_string()))
                } else {
                    None
                }
            }
            "SPLIT" => {
                if args.len() >= 2 {
                    if let (Value::String(s), Value::String(delim)) = (&args[0], &args[1]) {
                        let parts: Vec<Value> = s.split(delim.as_str())
                            .map(|p| Value::String(p.to_string()))
                            .collect();
                        Some(Value::Array(parts))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "JOIN" => {
                if args.len() >= 2 {
                    if let (Value::Array(arr), Value::String(delim)) = (&args[0], &args[1]) {
                        let strings: Vec<String> = arr.iter()
                            .map(|v| v.to_string())
                            .collect();
                        Some(Value::String(strings.join(delim)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "SUBSTR" => {
                if args.len() >= 3 {
                    if let (Value::String(s), Value::Int(start), Value::Int(len)) = (&args[0], &args[1], &args[2]) {
                        let chars: Vec<char> = s.chars().collect();
                        let char_len = chars.len();

                        // Handle negative start index (from end)
                        let start_idx = if *start < 0 {
                            char_len.saturating_sub((-*start) as usize)
                        } else {
                            (*start as usize).min(char_len)
                        };

                        let len_val = (*len).max(0) as usize;
                        let end_idx = (start_idx + len_val).min(char_len);

                        Some(Value::String(chars[start_idx..end_idx].iter().collect()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "REPLACE" => {
                if args.len() >= 3 {
                    if let (Value::String(s), Value::String(from), Value::String(to)) = (&args[0], &args[1], &args[2]) {
                        Some(Value::String(s.replace(from.as_str(), to.as_str())))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "CONTAINS" => {
                if args.len() >= 2 {
                    if let (Value::String(s), Value::String(sub)) = (&args[0], &args[1]) {
                        Some(Value::Bool(s.contains(sub.as_str())))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "STARTS_WITH" => {
                if args.len() >= 2 {
                    if let (Value::String(s), Value::String(prefix)) = (&args[0], &args[1]) {
                        Some(Value::Bool(s.starts_with(prefix.as_str())))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "ENDS_WITH" => {
                if args.len() >= 2 {
                    if let (Value::String(s), Value::String(suffix)) = (&args[0], &args[1]) {
                        Some(Value::Bool(s.ends_with(suffix.as_str())))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            // === Type conversion ===
            "INT" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Int(*n)),
                    Some(Value::Float(f)) => Some(Value::Int(*f as i64)),
                    Some(Value::String(s)) => s.parse::<i64>().ok().map(Value::Int),
                    Some(Value::Bool(b)) => Some(Value::Int(if *b { 1 } else { 0 })),
                    _ => None,
                }
            }
            "FLOAT" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float(*n as f64)),
                    Some(Value::Float(f)) => Some(Value::Float(*f)),
                    Some(Value::String(s)) => s.parse::<f64>().ok().map(Value::Float),
                    _ => None,
                }
            }
            "STR" | "STRING" => {
                args.first().map(|v| Value::String(v.to_string()))
            }
            "BOOL" => {
                match args.first() {
                    Some(Value::Bool(b)) => Some(Value::Bool(*b)),
                    Some(Value::Int(n)) => Some(Value::Bool(*n != 0)),
                    Some(Value::String(s)) => Some(Value::Bool(!s.is_empty())),
                    Some(Value::Array(arr)) => Some(Value::Bool(!arr.is_empty())),
                    _ => None,
                }
            }

            // === I/O functions ===
            "PRINT" => {
                for arg in args {
                    print!("{}", arg);
                }
                println!();
                Some(Value::Void)
            }
            "PRINTLN" => {
                for arg in args {
                    print!("{}", arg);
                }
                println!();
                Some(Value::Void)
            }

            // === Array functions (extended) ===
            "PUSH" => {
                // Note: In a pure functional style, this creates a new array
                if args.len() >= 2 {
                    if let Value::Array(arr) = &args[0] {
                        let mut new_arr = arr.clone();
                        new_arr.push(args[1].clone());
                        Some(Value::Array(new_arr))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "POP" => {
                // Returns (new_array, popped_element) as a tuple
                if let Some(Value::Array(arr)) = args.first() {
                    if arr.is_empty() {
                        None
                    } else {
                        let mut new_arr = arr.clone();
                        let popped = new_arr.pop().unwrap();
                        Some(Value::Array(vec![Value::Array(new_arr), popped]))
                    }
                } else {
                    None
                }
            }
            "TAKE" => {
                // Take first n elements
                if args.len() >= 2 {
                    match (&args[0], &args[1]) {
                        (Value::Array(arr), Value::Int(n)) => {
                            let n = (*n).max(0) as usize;
                            Some(Value::Array(arr.iter().take(n).cloned().collect()))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "DROP" => {
                // Drop first n elements
                if args.len() >= 2 {
                    match (&args[0], &args[1]) {
                        (Value::Array(arr), Value::Int(n)) => {
                            let n = (*n).max(0) as usize;
                            Some(Value::Array(arr.iter().skip(n).cloned().collect()))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "ZIP" => {
                // Zip two arrays into array of tuples
                if args.len() >= 2 {
                    match (&args[0], &args[1]) {
                        (Value::Array(a), Value::Array(b)) => {
                            let zipped: Vec<Value> = a.iter().zip(b.iter())
                                .map(|(x, y)| Value::Array(vec![x.clone(), y.clone()]))
                                .collect();
                            Some(Value::Array(zipped))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "FLATTEN" => {
                // Flatten one level of nesting
                if let Some(Value::Array(arr)) = args.first() {
                    let mut result = Vec::new();
                    for item in arr {
                        if let Value::Array(inner) = item {
                            result.extend(inner.clone());
                        } else {
                            result.push(item.clone());
                        }
                    }
                    Some(Value::Array(result))
                } else {
                    None
                }
            }
            "SORT" => {
                if let Some(Value::Array(arr)) = args.first() {
                    let mut sorted = arr.clone();
                    sorted.sort_by(|a, b| {
                        match (a, b) {
                            (Value::Int(x), Value::Int(y)) => x.cmp(y),
                            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                            (Value::String(x), Value::String(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        }
                    });
                    Some(Value::Array(sorted))
                } else {
                    None
                }
            }
            "UNIQUE" => {
                // Remove duplicates while preserving order
                if let Some(Value::Array(arr)) = args.first() {
                    let mut seen = std::collections::HashSet::new();
                    let mut result = Vec::new();
                    for item in arr {
                        let key = format!("{:?}", item);
                        if seen.insert(key) {
                            result.push(item.clone());
                        }
                    }
                    Some(Value::Array(result))
                } else {
                    None
                }
            }
            "INDEX_OF" => {
                // Find index of element in array
                if args.len() >= 2 {
                    if let Value::Array(arr) = &args[0] {
                        let needle = &args[1];
                        for (i, item) in arr.iter().enumerate() {
                            if format!("{:?}", item) == format!("{:?}", needle) {
                                return Ok(Some(Value::Int(i as i64)));
                            }
                        }
                        Some(Value::Int(-1))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            // === More math functions ===
            "EXP" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).exp())),
                    Some(Value::Float(f)) => Some(Value::Float(f.exp())),
                    _ => None,
                }
            }
            "LOG2" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).log2())),
                    Some(Value::Float(f)) => Some(Value::Float(f.log2())),
                    _ => None,
                }
            }
            "ASIN" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).asin())),
                    Some(Value::Float(f)) => Some(Value::Float(f.asin())),
                    _ => None,
                }
            }
            "ACOS" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).acos())),
                    Some(Value::Float(f)) => Some(Value::Float(f.acos())),
                    _ => None,
                }
            }
            "ATAN" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).atan())),
                    Some(Value::Float(f)) => Some(Value::Float(f.atan())),
                    _ => None,
                }
            }
            "ATAN2" => {
                if args.len() >= 2 {
                    let y = match &args[0] {
                        Value::Int(n) => *n as f64,
                        Value::Float(f) => *f,
                        _ => return Ok(None),
                    };
                    let x = match &args[1] {
                        Value::Int(n) => *n as f64,
                        Value::Float(f) => *f,
                        _ => return Ok(None),
                    };
                    Some(Value::Float(y.atan2(x)))
                } else {
                    None
                }
            }
            "CLAMP" => {
                // Clamp value between min and max
                if args.len() >= 3 {
                    match (&args[0], &args[1], &args[2]) {
                        (Value::Int(v), Value::Int(min), Value::Int(max)) => {
                            Some(Value::Int((*v).clamp(*min, *max)))
                        }
                        (Value::Float(v), Value::Float(min), Value::Float(max)) => {
                            Some(Value::Float(v.clamp(*min, *max)))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }

            // === More string functions ===
            "CHARS" => {
                // Split string into array of characters
                if let Some(Value::String(s)) = args.first() {
                    Some(Value::Array(
                        s.chars().map(|c| Value::String(c.to_string())).collect()
                    ))
                } else {
                    None
                }
            }
            "PAD_LEFT" => {
                // Pad string on the left to reach target length
                if args.len() >= 3 {
                    if let (Value::String(s), Value::Int(len), Value::String(pad)) = (&args[0], &args[1], &args[2]) {
                        let target_len = (*len).max(0) as usize;
                        let pad_char = pad.chars().next().unwrap_or(' ');
                        if s.len() >= target_len {
                            Some(Value::String(s.clone()))
                        } else {
                            let padding: String = std::iter::repeat_n(pad_char, target_len - s.len())
                                .collect();
                            Some(Value::String(format!("{}{}", padding, s)))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "PAD_RIGHT" => {
                // Pad string on the right to reach target length
                if args.len() >= 3 {
                    if let (Value::String(s), Value::Int(len), Value::String(pad)) = (&args[0], &args[1], &args[2]) {
                        let target_len = (*len).max(0) as usize;
                        let pad_char = pad.chars().next().unwrap_or(' ');
                        if s.len() >= target_len {
                            Some(Value::String(s.clone()))
                        } else {
                            let padding: String = std::iter::repeat_n(pad_char, target_len - s.len())
                                .collect();
                            Some(Value::String(format!("{}{}", s, padding)))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "REPEAT" => {
                // Repeat string n times
                if args.len() >= 2 {
                    if let (Value::String(s), Value::Int(n)) = (&args[0], &args[1]) {
                        let n = (*n).max(0) as usize;
                        Some(Value::String(s.repeat(n)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            // === Type checking ===
            "TYPE" => {
                args.first().map(|v| Value::String(match v {
                    Value::Int(_) => "int",
                    Value::Float(_) => "float",
                    Value::String(_) => "string",
                    Value::Bool(_) => "bool",
                    Value::Array(_) => "array",
                    Value::Map(_) => "map",
                    Value::Closure { .. } => "function",
                    Value::Void => "void",
                    Value::Bytes(_) => "bytes",
                    Value::Optional(_) => "optional",
                    Value::Error(_) => "error",
                    Value::Struct(_) => "struct",
                }.to_string()))
            }
            "IS_INT" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::Int(_))))
            }
            "IS_FLOAT" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::Float(_))))
            }
            "IS_STRING" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::String(_))))
            }
            "IS_BOOL" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::Bool(_))))
            }
            "IS_ARRAY" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::Array(_))))
            }
            "IS_MAP" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::Map(_))))
            }

            // === File I/O functions (std.io) ===
            "READ_FILE" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::read_to_string(path) {
                        Ok(content) => Some(Value::String(content)),
                        Err(e) => Some(Value::Error(format!("read_file: {}", e))),
                    }
                } else {
                    Some(Value::Error("read_file: expected string path".to_string()))
                }
            }
            "READ_FILE_BYTES" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::read(path) {
                        Ok(bytes) => Some(Value::Bytes(bytes)),
                        Err(e) => Some(Value::Error(format!("read_file_bytes: {}", e))),
                    }
                } else {
                    Some(Value::Error("read_file_bytes: expected string path".to_string()))
                }
            }
            "WRITE_FILE" => {
                if args.len() >= 2 {
                    if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                        match std::fs::write(path, content) {
                            Ok(_) => Some(Value::Bool(true)),
                            Err(e) => Some(Value::Error(format!("write_file: {}", e))),
                        }
                    } else if let (Value::String(path), Value::Bytes(bytes)) = (&args[0], &args[1]) {
                        match std::fs::write(path, bytes) {
                            Ok(_) => Some(Value::Bool(true)),
                            Err(e) => Some(Value::Error(format!("write_file: {}", e))),
                        }
                    } else {
                        Some(Value::Error("write_file: expected (path, content)".to_string()))
                    }
                } else {
                    Some(Value::Error("write_file: expected 2 arguments".to_string()))
                }
            }
            "APPEND_FILE" => {
                if args.len() >= 2 {
                    if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                        use std::io::Write;
                        match std::fs::OpenOptions::new().create(true).append(true).open(path) {
                            Ok(mut file) => {
                                match file.write_all(content.as_bytes()) {
                                    Ok(_) => Some(Value::Bool(true)),
                                    Err(e) => Some(Value::Error(format!("append_file: {}", e))),
                                }
                            }
                            Err(e) => Some(Value::Error(format!("append_file: {}", e))),
                        }
                    } else {
                        Some(Value::Error("append_file: expected (path, content)".to_string()))
                    }
                } else {
                    Some(Value::Error("append_file: expected 2 arguments".to_string()))
                }
            }
            "READ_LINES" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::read_to_string(path) {
                        Ok(content) => {
                            let lines: Vec<Value> = content
                                .lines()
                                .map(|s| Value::String(s.to_string()))
                                .collect();
                            Some(Value::Array(lines))
                        }
                        Err(e) => Some(Value::Error(format!("read_lines: {}", e))),
                    }
                } else {
                    Some(Value::Error("read_lines: expected string path".to_string()))
                }
            }

            // === Path functions ===
            "PATH_EXISTS" => {
                if let Some(Value::String(path)) = args.first() {
                    Some(Value::Bool(std::path::Path::new(path).exists()))
                } else {
                    Some(Value::Bool(false))
                }
            }
            "PATH_IS_FILE" => {
                if let Some(Value::String(path)) = args.first() {
                    Some(Value::Bool(std::path::Path::new(path).is_file()))
                } else {
                    Some(Value::Bool(false))
                }
            }
            "PATH_IS_DIR" => {
                if let Some(Value::String(path)) = args.first() {
                    Some(Value::Bool(std::path::Path::new(path).is_dir()))
                } else {
                    Some(Value::Bool(false))
                }
            }
            "PATH_JOIN" => {
                if args.len() >= 2 {
                    let mut path = std::path::PathBuf::new();
                    for arg in args {
                        if let Value::String(s) = arg {
                            path.push(s);
                        }
                    }
                    Some(Value::String(path.to_string_lossy().to_string()))
                } else if let Some(Value::String(s)) = args.first() {
                    Some(Value::String(s.clone()))
                } else {
                    Some(Value::String(String::new()))
                }
            }
            "PATH_PARENT" => {
                if let Some(Value::String(path)) = args.first() {
                    let p = std::path::Path::new(path);
                    match p.parent() {
                        Some(parent) => Some(Value::String(parent.to_string_lossy().to_string())),
                        None => Some(Value::String(String::new())),
                    }
                } else {
                    Some(Value::String(String::new()))
                }
            }
            "PATH_FILENAME" => {
                if let Some(Value::String(path)) = args.first() {
                    let p = std::path::Path::new(path);
                    match p.file_name() {
                        Some(name) => Some(Value::String(name.to_string_lossy().to_string())),
                        None => Some(Value::String(String::new())),
                    }
                } else {
                    Some(Value::String(String::new()))
                }
            }
            "PATH_EXTENSION" => {
                if let Some(Value::String(path)) = args.first() {
                    let p = std::path::Path::new(path);
                    match p.extension() {
                        Some(ext) => Some(Value::String(ext.to_string_lossy().to_string())),
                        None => Some(Value::String(String::new())),
                    }
                } else {
                    Some(Value::String(String::new()))
                }
            }
            "PATH_STEM" => {
                if let Some(Value::String(path)) = args.first() {
                    let p = std::path::Path::new(path);
                    match p.file_stem() {
                        Some(stem) => Some(Value::String(stem.to_string_lossy().to_string())),
                        None => Some(Value::String(String::new())),
                    }
                } else {
                    Some(Value::String(String::new()))
                }
            }
            "PATH_ABSOLUTE" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::canonicalize(path) {
                        Ok(abs) => Some(Value::String(abs.to_string_lossy().to_string())),
                        Err(e) => Some(Value::Error(format!("path_absolute: {}", e))),
                    }
                } else {
                    Some(Value::Error("path_absolute: expected string path".to_string()))
                }
            }

            // === Directory functions ===
            "LIST_DIR" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::read_dir(path) {
                        Ok(entries) => {
                            let mut items = Vec::new();
                            for entry in entries.flatten() {
                                items.push(Value::String(
                                    entry.file_name().to_string_lossy().to_string()
                                ));
                            }
                            Some(Value::Array(items))
                        }
                        Err(e) => Some(Value::Error(format!("list_dir: {}", e))),
                    }
                } else {
                    Some(Value::Error("list_dir: expected string path".to_string()))
                }
            }
            "CREATE_DIR" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::create_dir(path) {
                        Ok(_) => Some(Value::Bool(true)),
                        Err(e) => Some(Value::Error(format!("create_dir: {}", e))),
                    }
                } else {
                    Some(Value::Error("create_dir: expected string path".to_string()))
                }
            }
            "CREATE_DIR_ALL" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::create_dir_all(path) {
                        Ok(_) => Some(Value::Bool(true)),
                        Err(e) => Some(Value::Error(format!("create_dir_all: {}", e))),
                    }
                } else {
                    Some(Value::Error("create_dir_all: expected string path".to_string()))
                }
            }
            "REMOVE_FILE" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::remove_file(path) {
                        Ok(_) => Some(Value::Bool(true)),
                        Err(e) => Some(Value::Error(format!("remove_file: {}", e))),
                    }
                } else {
                    Some(Value::Error("remove_file: expected string path".to_string()))
                }
            }
            "REMOVE_DIR" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::remove_dir(path) {
                        Ok(_) => Some(Value::Bool(true)),
                        Err(e) => Some(Value::Error(format!("remove_dir: {}", e))),
                    }
                } else {
                    Some(Value::Error("remove_dir: expected string path".to_string()))
                }
            }
            "REMOVE_DIR_ALL" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::remove_dir_all(path) {
                        Ok(_) => Some(Value::Bool(true)),
                        Err(e) => Some(Value::Error(format!("remove_dir_all: {}", e))),
                    }
                } else {
                    Some(Value::Error("remove_dir_all: expected string path".to_string()))
                }
            }
            "COPY_FILE" => {
                if args.len() >= 2 {
                    if let (Value::String(src), Value::String(dst)) = (&args[0], &args[1]) {
                        match std::fs::copy(src, dst) {
                            Ok(bytes) => Some(Value::Int(bytes as i64)),
                            Err(e) => Some(Value::Error(format!("copy_file: {}", e))),
                        }
                    } else {
                        Some(Value::Error("copy_file: expected (src, dst)".to_string()))
                    }
                } else {
                    Some(Value::Error("copy_file: expected 2 arguments".to_string()))
                }
            }
            "RENAME" => {
                if args.len() >= 2 {
                    if let (Value::String(src), Value::String(dst)) = (&args[0], &args[1]) {
                        match std::fs::rename(src, dst) {
                            Ok(_) => Some(Value::Bool(true)),
                            Err(e) => Some(Value::Error(format!("rename: {}", e))),
                        }
                    } else {
                        Some(Value::Error("rename: expected (src, dst)".to_string()))
                    }
                } else {
                    Some(Value::Error("rename: expected 2 arguments".to_string()))
                }
            }

            // === File metadata ===
            "FILE_SIZE" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::metadata(path) {
                        Ok(meta) => Some(Value::Int(meta.len() as i64)),
                        Err(e) => Some(Value::Error(format!("file_size: {}", e))),
                    }
                } else {
                    Some(Value::Error("file_size: expected string path".to_string()))
                }
            }

            // === Standard input ===
            "READLINE" => {
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let mut line = String::new();
                match stdin.lock().read_line(&mut line) {
                    Ok(_) => {
                        // Remove trailing newline
                        if line.ends_with('\n') {
                            line.pop();
                            if line.ends_with('\r') {
                                line.pop();
                            }
                        }
                        Some(Value::String(line))
                    }
                    Err(e) => Some(Value::Error(format!("readline: {}", e))),
                }
            }

            // === Environment ===
            "ENV_GET" => {
                if let Some(Value::String(key)) = args.first() {
                    match std::env::var(key) {
                        Ok(val) => Some(Value::String(val)),
                        Err(_) => Some(Value::Optional(None)),
                    }
                } else {
                    Some(Value::Error("env_get: expected string key".to_string()))
                }
            }
            "ENV_SET" => {
                if args.len() >= 2 {
                    if let (Value::String(key), Value::String(val)) = (&args[0], &args[1]) {
                        std::env::set_var(key, val);
                        Some(Value::Bool(true))
                    } else {
                        Some(Value::Error("env_set: expected (key, value)".to_string()))
                    }
                } else {
                    Some(Value::Error("env_set: expected 2 arguments".to_string()))
                }
            }
            "CWD" => {
                match std::env::current_dir() {
                    Ok(path) => Some(Value::String(path.to_string_lossy().to_string())),
                    Err(e) => Some(Value::Error(format!("cwd: {}", e))),
                }
            }
            "CHDIR" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::env::set_current_dir(path) {
                        Ok(_) => Some(Value::Bool(true)),
                        Err(e) => Some(Value::Error(format!("chdir: {}", e))),
                    }
                } else {
                    Some(Value::Error("chdir: expected string path".to_string()))
                }
            }

            // === JSON functions (std.json) ===
            "JSON_PARSE" => {
                if let Some(Value::String(s)) = args.first() {
                    match serde_json::from_str::<serde_json::Value>(s) {
                        Ok(json) => Some(Self::json_to_value(&json)),
                        Err(e) => Some(Value::Error(format!("json_parse: {}", e))),
                    }
                } else {
                    Some(Value::Error("json_parse: expected string".to_string()))
                }
            }
            "JSON_STRINGIFY" => {
                if let Some(val) = args.first() {
                    let json = Self::value_to_json(val);
                    match serde_json::to_string(&json) {
                        Ok(s) => Some(Value::String(s)),
                        Err(e) => Some(Value::Error(format!("json_stringify: {}", e))),
                    }
                } else {
                    Some(Value::String("null".to_string()))
                }
            }
            "JSON_STRINGIFY_PRETTY" => {
                if let Some(val) = args.first() {
                    let json = Self::value_to_json(val);
                    match serde_json::to_string_pretty(&json) {
                        Ok(s) => Some(Value::String(s)),
                        Err(e) => Some(Value::Error(format!("json_stringify_pretty: {}", e))),
                    }
                } else {
                    Some(Value::String("null".to_string()))
                }
            }
            "JSON_GET" => {
                // json_get(obj, key) or json_get(obj, "path.to.key")
                if args.len() >= 2 {
                    if let (val, Value::String(key)) = (&args[0], &args[1]) {
                        Some(Self::json_path_get(val, key))
                    } else if let (val, Value::Int(idx)) = (&args[0], &args[1]) {
                        // Array index access
                        if let Value::Array(arr) = val {
                            let i = if *idx < 0 { arr.len() as i64 + *idx } else { *idx };
                            if i >= 0 && (i as usize) < arr.len() {
                                Some(arr[i as usize].clone())
                            } else {
                                Some(Value::Void)
                            }
                        } else {
                            Some(Value::Void)
                        }
                    } else {
                        Some(Value::Error("json_get: expected (value, key)".to_string()))
                    }
                } else {
                    Some(Value::Error("json_get: expected 2 arguments".to_string()))
                }
            }
            "JSON_SET" => {
                // json_set(obj, key, value)
                if args.len() >= 3 {
                    if let Value::String(key) = &args[1] {
                        Some(Self::json_path_set(&args[0], key, &args[2]))
                    } else {
                        Some(Value::Error("json_set: key must be string".to_string()))
                    }
                } else {
                    Some(Value::Error("json_set: expected 3 arguments".to_string()))
                }
            }
            "JSON_KEYS" => {
                if let Some(Value::Map(m)) = args.first() {
                    let keys: Vec<Value> = m.keys()
                        .map(|k| Value::String(k.clone()))
                        .collect();
                    Some(Value::Array(keys))
                } else if let Some(Value::Struct(s)) = args.first() {
                    let keys: Vec<Value> = s.keys()
                        .map(|k| Value::String(k.clone()))
                        .collect();
                    Some(Value::Array(keys))
                } else {
                    Some(Value::Array(vec![]))
                }
            }
            "JSON_VALUES" => {
                if let Some(Value::Map(m)) = args.first() {
                    let values: Vec<Value> = m.values().cloned().collect();
                    Some(Value::Array(values))
                } else if let Some(Value::Struct(s)) = args.first() {
                    let values: Vec<Value> = s.values().cloned().collect();
                    Some(Value::Array(values))
                } else {
                    Some(Value::Array(vec![]))
                }
            }
            "JSON_HAS" => {
                // Check if key exists
                if args.len() >= 2 {
                    if let (val, Value::String(key)) = (&args[0], &args[1]) {
                        let result = Self::json_path_get(val, key);
                        Some(Value::Bool(!matches!(result, Value::Void)))
                    } else {
                        Some(Value::Bool(false))
                    }
                } else {
                    Some(Value::Bool(false))
                }
            }
            "JSON_REMOVE" => {
                // Remove a key from object
                if args.len() >= 2 {
                    if let (Value::Map(m), Value::String(key)) = (&args[0], &args[1]) {
                        let mut new_map = m.clone();
                        new_map.remove(key);
                        Some(Value::Map(new_map))
                    } else if let (Value::Struct(s), Value::String(key)) = (&args[0], &args[1]) {
                        let mut new_struct = s.clone();
                        new_struct.remove(key);
                        Some(Value::Struct(new_struct))
                    } else {
                        Some(args[0].clone())
                    }
                } else {
                    Some(Value::Error("json_remove: expected 2 arguments".to_string()))
                }
            }
            "JSON_MERGE" => {
                // Merge two objects
                if args.len() >= 2 {
                    match (&args[0], &args[1]) {
                        (Value::Map(a), Value::Map(b)) => {
                            let mut merged = a.clone();
                            merged.extend(b.clone());
                            Some(Value::Map(merged))
                        }
                        (Value::Struct(a), Value::Struct(b)) => {
                            let mut merged = a.clone();
                            merged.extend(b.clone());
                            Some(Value::Struct(merged))
                        }
                        _ => Some(Value::Error("json_merge: expected two objects".to_string()))
                    }
                } else {
                    Some(Value::Error("json_merge: expected 2 arguments".to_string()))
                }
            }
            "JSON_TYPE" => {
                // Get JSON type as string
                args.first().map(|v| Value::String(match v {
                    Value::Void => "null",
                    Value::Bool(_) => "boolean",
                    Value::Int(_) | Value::Float(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Map(_) | Value::Struct(_) => "object",
                    _ => "unknown",
                }.to_string()))
            }
            "JSON_IS_NULL" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::Void)))
            }
            "JSON_IS_OBJECT" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::Map(_) | Value::Struct(_))))
            }
            "JSON_IS_ARRAY" => {
                args.first().map(|v| Value::Bool(matches!(v, Value::Array(_))))
            }

            // === HTTP functions (std.net) ===
            "HTTP_GET" => {
                if let Some(Value::String(url)) = args.first() {
                    match ureq::get(url).call() {
                        Ok(response) => {
                            let status = response.status();
                            let body = response.into_string().unwrap_or_default();
                            let mut result = HashMap::new();
                            result.insert("status".to_string(), Value::Int(status as i64));
                            result.insert("body".to_string(), Value::String(body));
                            result.insert("ok".to_string(), Value::Bool((200..300).contains(&status)));
                            Some(Value::Map(result))
                        }
                        Err(e) => Some(Value::Error(format!("http_get: {}", e))),
                    }
                } else {
                    Some(Value::Error("http_get: expected url string".to_string()))
                }
            }
            "HTTP_GET_JSON" => {
                if let Some(Value::String(url)) = args.first() {
                    match ureq::get(url).call() {
                        Ok(response) => {
                            let body = response.into_string().unwrap_or_default();
                            match serde_json::from_str::<serde_json::Value>(&body) {
                                Ok(json) => Some(Self::json_to_value(&json)),
                                Err(e) => Some(Value::Error(format!("http_get_json: parse error: {}", e))),
                            }
                        }
                        Err(e) => Some(Value::Error(format!("http_get_json: {}", e))),
                    }
                } else {
                    Some(Value::Error("http_get_json: expected url string".to_string()))
                }
            }
            "HTTP_POST" => {
                if args.len() >= 2 {
                    if let (Value::String(url), body) = (&args[0], &args[1]) {
                        let body_str = match body {
                            Value::String(s) => s.clone(),
                            _ => {
                                let json = Self::value_to_json(body);
                                serde_json::to_string(&json).unwrap_or_default()
                            }
                        };

                        match ureq::post(url)
                            .set("Content-Type", "application/json")
                            .send_string(&body_str)
                        {
                            Ok(response) => {
                                let status = response.status();
                                let resp_body = response.into_string().unwrap_or_default();
                                let mut result = HashMap::new();
                                result.insert("status".to_string(), Value::Int(status as i64));
                                result.insert("body".to_string(), Value::String(resp_body));
                                result.insert("ok".to_string(), Value::Bool((200..300).contains(&status)));
                                Some(Value::Map(result))
                            }
                            Err(e) => Some(Value::Error(format!("http_post: {}", e))),
                        }
                    } else {
                        Some(Value::Error("http_post: expected (url, body)".to_string()))
                    }
                } else {
                    Some(Value::Error("http_post: expected 2 arguments".to_string()))
                }
            }
            "HTTP_POST_JSON" => {
                if args.len() >= 2 {
                    if let (Value::String(url), body) = (&args[0], &args[1]) {
                        let json_body = Self::value_to_json(body);

                        match ureq::post(url)
                            .set("Content-Type", "application/json")
                            .send_json(&json_body)
                        {
                            Ok(response) => {
                                let body = response.into_string().unwrap_or_default();
                                match serde_json::from_str::<serde_json::Value>(&body) {
                                    Ok(json) => Some(Self::json_to_value(&json)),
                                    Err(_) => Some(Value::String(body)),
                                }
                            }
                            Err(e) => Some(Value::Error(format!("http_post_json: {}", e))),
                        }
                    } else {
                        Some(Value::Error("http_post_json: expected (url, body)".to_string()))
                    }
                } else {
                    Some(Value::Error("http_post_json: expected 2 arguments".to_string()))
                }
            }
            "HTTP_PUT" => {
                if args.len() >= 2 {
                    if let (Value::String(url), body) = (&args[0], &args[1]) {
                        let body_str = match body {
                            Value::String(s) => s.clone(),
                            _ => {
                                let json = Self::value_to_json(body);
                                serde_json::to_string(&json).unwrap_or_default()
                            }
                        };

                        match ureq::put(url)
                            .set("Content-Type", "application/json")
                            .send_string(&body_str)
                        {
                            Ok(response) => {
                                let status = response.status();
                                let resp_body = response.into_string().unwrap_or_default();
                                let mut result = HashMap::new();
                                result.insert("status".to_string(), Value::Int(status as i64));
                                result.insert("body".to_string(), Value::String(resp_body));
                                result.insert("ok".to_string(), Value::Bool((200..300).contains(&status)));
                                Some(Value::Map(result))
                            }
                            Err(e) => Some(Value::Error(format!("http_put: {}", e))),
                        }
                    } else {
                        Some(Value::Error("http_put: expected (url, body)".to_string()))
                    }
                } else {
                    Some(Value::Error("http_put: expected 2 arguments".to_string()))
                }
            }
            "HTTP_DELETE" => {
                if let Some(Value::String(url)) = args.first() {
                    match ureq::delete(url).call() {
                        Ok(response) => {
                            let status = response.status();
                            let body = response.into_string().unwrap_or_default();
                            let mut result = HashMap::new();
                            result.insert("status".to_string(), Value::Int(status as i64));
                            result.insert("body".to_string(), Value::String(body));
                            result.insert("ok".to_string(), Value::Bool((200..300).contains(&status)));
                            Some(Value::Map(result))
                        }
                        Err(e) => Some(Value::Error(format!("http_delete: {}", e))),
                    }
                } else {
                    Some(Value::Error("http_delete: expected url string".to_string()))
                }
            }
            "HTTP_HEAD" => {
                if let Some(Value::String(url)) = args.first() {
                    match ureq::head(url).call() {
                        Ok(response) => {
                            let status = response.status();
                            let mut headers = HashMap::new();
                            for name in response.headers_names() {
                                if let Some(value) = response.header(&name) {
                                    headers.insert(name, Value::String(value.to_string()));
                                }
                            }
                            let mut result = HashMap::new();
                            result.insert("status".to_string(), Value::Int(status as i64));
                            result.insert("headers".to_string(), Value::Map(headers));
                            result.insert("ok".to_string(), Value::Bool((200..300).contains(&status)));
                            Some(Value::Map(result))
                        }
                        Err(e) => Some(Value::Error(format!("http_head: {}", e))),
                    }
                } else {
                    Some(Value::Error("http_head: expected url string".to_string()))
                }
            }
            "HTTP_REQUEST" => {
                // Generic HTTP request: http_request(method, url, headers, body)
                if args.len() >= 2 {
                    let method = match &args[0] {
                        Value::String(s) => s.to_uppercase(),
                        _ => return Ok(Some(Value::Error("http_request: method must be string".to_string()))),
                    };
                    let url = match &args[1] {
                        Value::String(s) => s.clone(),
                        _ => return Ok(Some(Value::Error("http_request: url must be string".to_string()))),
                    };

                    let mut request = match method.as_str() {
                        "GET" => ureq::get(&url),
                        "POST" => ureq::post(&url),
                        "PUT" => ureq::put(&url),
                        "DELETE" => ureq::delete(&url),
                        "HEAD" => ureq::head(&url),
                        "PATCH" => ureq::patch(&url),
                        _ => return Ok(Some(Value::Error(format!("http_request: unsupported method: {}", method)))),
                    };

                    // Add headers if provided
                    if args.len() >= 3 {
                        if let Value::Map(headers) = &args[2] {
                            for (key, value) in headers {
                                if let Value::String(v) = value {
                                    request = request.set(key, v);
                                }
                            }
                        }
                    }

                    // Send with body if provided
                    let response_result = if args.len() >= 4 {
                        let body_str = match &args[3] {
                            Value::String(s) => s.clone(),
                            Value::Void => String::new(),
                            other => {
                                let json = Self::value_to_json(other);
                                serde_json::to_string(&json).unwrap_or_default()
                            }
                        };
                        if body_str.is_empty() {
                            request.call()
                        } else {
                            request.send_string(&body_str)
                        }
                    } else {
                        request.call()
                    };

                    match response_result {
                        Ok(response) => {
                            let status = response.status();
                            let mut headers = HashMap::new();
                            for name in response.headers_names() {
                                if let Some(value) = response.header(&name) {
                                    headers.insert(name, Value::String(value.to_string()));
                                }
                            }
                            let body = response.into_string().unwrap_or_default();

                            let mut result = HashMap::new();
                            result.insert("status".to_string(), Value::Int(status as i64));
                            result.insert("headers".to_string(), Value::Map(headers));
                            result.insert("body".to_string(), Value::String(body));
                            result.insert("ok".to_string(), Value::Bool((200..300).contains(&status)));
                            Some(Value::Map(result))
                        }
                        Err(e) => Some(Value::Error(format!("http_request: {}", e))),
                    }
                } else {
                    Some(Value::Error("http_request: expected at least (method, url)".to_string()))
                }
            }
            "URL_ENCODE" => {
                if let Some(Value::String(s)) = args.first() {
                    // Simple percent encoding for URL parameters
                    let encoded: String = s.chars().map(|c| {
                        match c {
                            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                            ' ' => "+".to_string(),
                            _ => format!("%{:02X}", c as u8),
                        }
                    }).collect();
                    Some(Value::String(encoded))
                } else {
                    Some(Value::String(String::new()))
                }
            }
            "URL_DECODE" => {
                if let Some(Value::String(s)) = args.first() {
                    let mut result = String::new();
                    let mut chars = s.chars().peekable();
                    while let Some(c) = chars.next() {
                        match c {
                            '+' => result.push(' '),
                            '%' => {
                                let hex: String = chars.by_ref().take(2).collect();
                                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                    result.push(byte as char);
                                } else {
                                    result.push('%');
                                    result.push_str(&hex);
                                }
                            }
                            _ => result.push(c),
                        }
                    }
                    Some(Value::String(result))
                } else {
                    Some(Value::String(String::new()))
                }
            }

            // === Time functions ===
            "TIME_NOW" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                Some(Value::Int(now.as_secs() as i64))
            }
            "TIME_NOW_MS" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                Some(Value::Int(now.as_millis() as i64))
            }
            "TIME_FORMAT" => {
                // time_format(timestamp, format) - basic ISO 8601 format
                if let Some(Value::Int(ts)) = args.first() {
                    // Simple formatting without chrono dependency
                    let secs = *ts;
                    let days = secs / 86400;
                    let remaining = secs % 86400;
                    let hours = remaining / 3600;
                    let mins = (remaining % 3600) / 60;
                    let sec = remaining % 60;

                    // Calculate year/month/day from days since epoch (1970-01-01)
                    let (year, month, day) = Self::days_to_ymd(days as i32 + 719468);

                    Some(Value::String(format!(
                        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                        year, month, day, hours, mins, sec
                    )))
                } else {
                    Some(Value::Error("time_format: expected timestamp".to_string()))
                }
            }
            "TIME_PARSE" => {
                // Parse ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
                if let Some(Value::String(s)) = args.first() {
                    let parts: Vec<&str> = s.split(|c| c == '-' || c == 'T' || c == ':' || c == 'Z')
                        .filter(|s| !s.is_empty())
                        .collect();
                    if parts.len() >= 6 {
                        if let (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(min), Ok(sec)) = (
                            parts[0].parse::<i32>(),
                            parts[1].parse::<u32>(),
                            parts[2].parse::<u32>(),
                            parts[3].parse::<i64>(),
                            parts[4].parse::<i64>(),
                            parts[5].parse::<i64>(),
                        ) {
                            let days = Self::ymd_to_days(year, month, day) - 719468;
                            let timestamp = days as i64 * 86400 + hour * 3600 + min * 60 + sec;
                            Some(Value::Int(timestamp))
                        } else {
                            Some(Value::Error("time_parse: invalid date format".to_string()))
                        }
                    } else {
                        Some(Value::Error("time_parse: expected YYYY-MM-DDTHH:MM:SSZ".to_string()))
                    }
                } else {
                    Some(Value::Error("time_parse: expected string".to_string()))
                }
            }
            "SLEEP" => {
                if let Some(Value::Int(ms)) = args.first() {
                    if *ms > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                    }
                    Some(Value::Void)
                } else {
                    Some(Value::Error("sleep: expected milliseconds".to_string()))
                }
            }

            // === Random functions ===
            "RANDOM" => {
                // Simple random number between 0.0 and 1.0
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                // Simple LCG random
                let random = ((seed.wrapping_mul(6364136223846793005).wrapping_add(1)) % (1 << 32)) as f64 / (1u64 << 32) as f64;
                Some(Value::Float(random))
            }
            "RANDOM_INT" => {
                // random_int(min, max) - random integer in range [min, max]
                if args.len() >= 2 {
                    if let (Value::Int(min), Value::Int(max)) = (&args[0], &args[1]) {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64;
                        let range = (max - min + 1) as u64;
                        let random = (seed.wrapping_mul(6364136223846793005).wrapping_add(1)) % range;
                        Some(Value::Int(min + random as i64))
                    } else {
                        Some(Value::Error("random_int: expected (min, max)".to_string()))
                    }
                } else {
                    Some(Value::Error("random_int: expected 2 arguments".to_string()))
                }
            }
            "SHUFFLE" => {
                if let Some(Value::Array(arr)) = args.first() {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let mut result = arr.clone();
                    let mut seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;

                    // Fisher-Yates shuffle
                    for i in (1..result.len()).rev() {
                        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let j = (seed as usize) % (i + 1);
                        result.swap(i, j);
                    }
                    Some(Value::Array(result))
                } else {
                    Some(Value::Error("shuffle: expected array".to_string()))
                }
            }
            "SAMPLE" => {
                // sample(array, n) - pick n random elements
                if args.len() >= 2 {
                    if let (Value::Array(arr), Value::Int(n)) = (&args[0], &args[1]) {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let n = (*n as usize).min(arr.len());
                        let mut indices: Vec<usize> = (0..arr.len()).collect();
                        let mut seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64;

                        // Partial Fisher-Yates
                        for i in 0..n {
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            let j = i + (seed as usize) % (indices.len() - i);
                            indices.swap(i, j);
                        }

                        let result: Vec<Value> = indices[..n].iter().map(|&i| arr[i].clone()).collect();
                        Some(Value::Array(result))
                    } else {
                        Some(Value::Error("sample: expected (array, n)".to_string()))
                    }
                } else {
                    Some(Value::Error("sample: expected 2 arguments".to_string()))
                }
            }

            // === Additional Math functions ===
            "SIGN" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Int(n.signum())),
                    Some(Value::Float(n)) => Some(Value::Float(if *n > 0.0 { 1.0 } else if *n < 0.0 { -1.0 } else { 0.0 })),
                    _ => Some(Value::Error("sign: expected number".to_string())),
                }
            }
            "HYPOT" => {
                // hypot(a, b) = sqrt(a^2 + b^2)
                if args.len() >= 2 {
                    let a = match &args[0] {
                        Value::Int(n) => *n as f64,
                        Value::Float(n) => *n,
                        _ => return Ok(Some(Value::Error("hypot: expected numbers".to_string()))),
                    };
                    let b = match &args[1] {
                        Value::Int(n) => *n as f64,
                        Value::Float(n) => *n,
                        _ => return Ok(Some(Value::Error("hypot: expected numbers".to_string()))),
                    };
                    Some(Value::Float(a.hypot(b)))
                } else {
                    Some(Value::Error("hypot: expected 2 arguments".to_string()))
                }
            }
            "SINH" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).sinh())),
                    Some(Value::Float(n)) => Some(Value::Float(n.sinh())),
                    _ => Some(Value::Error("sinh: expected number".to_string())),
                }
            }
            "COSH" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).cosh())),
                    Some(Value::Float(n)) => Some(Value::Float(n.cosh())),
                    _ => Some(Value::Error("cosh: expected number".to_string())),
                }
            }
            "TANH" => {
                match args.first() {
                    Some(Value::Int(n)) => Some(Value::Float((*n as f64).tanh())),
                    Some(Value::Float(n)) => Some(Value::Float(n.tanh())),
                    _ => Some(Value::Error("tanh: expected number".to_string())),
                }
            }

            // === String functions ===
            "CHAR_CODE" => {
                if let Some(Value::String(s)) = args.first() {
                    Some(Value::Int(s.chars().next().map(|c| c as i64).unwrap_or(0)))
                } else if let Some(Value::Int(n)) = args.first() {
                    // int -> char
                    Some(Value::String(char::from_u32(*n as u32).map(|c| c.to_string()).unwrap_or_default()))
                } else {
                    Some(Value::Error("char_code: expected string or int".to_string()))
                }
            }
            "HEX" => {
                if let Some(Value::Int(n)) = args.first() {
                    Some(Value::String(format!("{:x}", n)))
                } else if let Some(Value::String(s)) = args.first() {
                    // hex string -> int
                    let s = s.trim_start_matches("0x").trim_start_matches("0X");
                    i64::from_str_radix(s, 16)
                        .map(Value::Int)
                        .ok()
                        .or(Some(Value::Error("hex: invalid hex string".to_string())))
                } else {
                    Some(Value::Error("hex: expected int or string".to_string()))
                }
            }
            "BIN" => {
                if let Some(Value::Int(n)) = args.first() {
                    Some(Value::String(format!("{:b}", n)))
                } else if let Some(Value::String(s)) = args.first() {
                    // binary string -> int
                    let s = s.trim_start_matches("0b").trim_start_matches("0B");
                    i64::from_str_radix(s, 2)
                        .map(Value::Int)
                        .ok()
                        .or(Some(Value::Error("bin: invalid binary string".to_string())))
                } else {
                    Some(Value::Error("bin: expected int or string".to_string()))
                }
            }
            "FORMAT" => {
                // format(template, ...args) - simple string formatting with {}
                if !args.is_empty() {
                    if let Value::String(template) = &args[0] {
                        let mut result = template.clone();
                        for (i, arg) in args.iter().skip(1).enumerate() {
                            let placeholder = format!("{{{}}}", i);
                            let value_str = match arg {
                                Value::String(s) => s.clone(),
                                Value::Int(n) => n.to_string(),
                                Value::Float(n) => n.to_string(),
                                Value::Bool(b) => b.to_string(),
                                _ => format!("{:?}", arg),
                            };
                            result = result.replace(&placeholder, &value_str);
                            // Also replace {} in order
                            result = result.replacen("{}", &value_str, 1);
                        }
                        Some(Value::String(result))
                    } else {
                        Some(Value::Error("format: first arg must be string".to_string()))
                    }
                } else {
                    Some(Value::Error("format: expected template string".to_string()))
                }
            }

            // === Array functions ===
            "SUM" => {
                if let Some(Value::Array(arr)) = args.first() {
                    let mut int_sum: i64 = 0;
                    let mut float_sum: f64 = 0.0;
                    let mut has_float = false;

                    for v in arr {
                        match v {
                            Value::Int(n) => int_sum += n,
                            Value::Float(n) => { float_sum += n; has_float = true; }
                            _ => {}
                        }
                    }

                    if has_float {
                        Some(Value::Float(int_sum as f64 + float_sum))
                    } else {
                        Some(Value::Int(int_sum))
                    }
                } else {
                    Some(Value::Error("sum: expected array".to_string()))
                }
            }
            "PRODUCT" => {
                if let Some(Value::Array(arr)) = args.first() {
                    let mut int_prod: i64 = 1;
                    let mut float_prod: f64 = 1.0;
                    let mut has_float = false;

                    for v in arr {
                        match v {
                            Value::Int(n) => int_prod *= n,
                            Value::Float(n) => { float_prod *= n; has_float = true; }
                            _ => {}
                        }
                    }

                    if has_float {
                        Some(Value::Float(int_prod as f64 * float_prod))
                    } else {
                        Some(Value::Int(int_prod))
                    }
                } else {
                    Some(Value::Error("product: expected array".to_string()))
                }
            }
            "AVERAGE" => {
                if let Some(Value::Array(arr)) = args.first() {
                    if arr.is_empty() {
                        return Ok(Some(Value::Float(0.0)));
                    }
                    let mut sum: f64 = 0.0;
                    let mut count = 0;

                    for v in arr {
                        match v {
                            Value::Int(n) => { sum += *n as f64; count += 1; }
                            Value::Float(n) => { sum += n; count += 1; }
                            _ => {}
                        }
                    }

                    Some(Value::Float(if count > 0 { sum / count as f64 } else { 0.0 }))
                } else {
                    Some(Value::Error("average: expected array".to_string()))
                }
            }
            "FIND" => {
                // find(array, value) - returns index or -1
                if args.len() >= 2 {
                    if let Value::Array(arr) = &args[0] {
                        let target = &args[1];
                        let idx = arr.iter().position(|v| v == target);
                        Some(Value::Int(idx.map(|i| i as i64).unwrap_or(-1)))
                    } else {
                        Some(Value::Error("find: expected array".to_string()))
                    }
                } else {
                    Some(Value::Error("find: expected 2 arguments".to_string()))
                }
            }
            "COUNT" => {
                // count(array, value) - count occurrences
                if args.len() >= 2 {
                    if let Value::Array(arr) = &args[0] {
                        let target = &args[1];
                        let count = arr.iter().filter(|v| *v == target).count();
                        Some(Value::Int(count as i64))
                    } else {
                        Some(Value::Error("count: expected array".to_string()))
                    }
                } else {
                    Some(Value::Error("count: expected 2 arguments".to_string()))
                }
            }
            "GROUP_BY" => {
                // group_by(array, key_fn) - for now, just group by value
                if let Some(Value::Array(arr)) = args.first() {
                    let mut groups: HashMap<String, Vec<Value>> = HashMap::new();
                    for v in arr {
                        let key = match v {
                            Value::String(s) => s.clone(),
                            Value::Int(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => format!("{:?}", v),
                        };
                        groups.entry(key).or_default().push(v.clone());
                    }
                    let result: HashMap<String, Value> = groups.into_iter()
                        .map(|(k, v)| (k, Value::Array(v)))
                        .collect();
                    Some(Value::Map(result))
                } else {
                    Some(Value::Error("group_by: expected array".to_string()))
                }
            }
            "PARTITION" => {
                // partition(array, index) - split array at index
                if args.len() >= 2 {
                    if let (Value::Array(arr), Value::Int(idx)) = (&args[0], &args[1]) {
                        let idx = (*idx as usize).min(arr.len());
                        let left: Vec<Value> = arr[..idx].to_vec();
                        let right: Vec<Value> = arr[idx..].to_vec();
                        Some(Value::Array(vec![Value::Array(left), Value::Array(right)]))
                    } else {
                        Some(Value::Error("partition: expected (array, index)".to_string()))
                    }
                } else {
                    Some(Value::Error("partition: expected 2 arguments".to_string()))
                }
            }
            "CHUNK" => {
                // chunk(array, size) - split array into chunks
                if args.len() >= 2 {
                    if let (Value::Array(arr), Value::Int(size)) = (&args[0], &args[1]) {
                        let size = (*size as usize).max(1);
                        let chunks: Vec<Value> = arr.chunks(size)
                            .map(|c| Value::Array(c.to_vec()))
                            .collect();
                        Some(Value::Array(chunks))
                    } else {
                        Some(Value::Error("chunk: expected (array, size)".to_string()))
                    }
                } else {
                    Some(Value::Error("chunk: expected 2 arguments".to_string()))
                }
            }

            _ => None,
        };
        Ok(result)
    }

    /// Convert serde_json::Value to AOEL Value
    fn json_to_value(json: &serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Void,
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Void
                }
            }
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.iter().map(Self::json_to_value).collect())
            }
            serde_json::Value::Object(obj) => {
                let map: HashMap<String, Value> = obj.iter()
                    .map(|(k, v)| (k.clone(), Self::json_to_value(v)))
                    .collect();
                Value::Map(map)
            }
        }
    }

    /// Convert AOEL Value to serde_json::Value
    fn value_to_json(val: &Value) -> serde_json::Value {
        match val {
            Value::Void => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(n) => serde_json::Value::Number((*n).into()),
            Value::Float(f) => {
                serde_json::Number::from_f64(*f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(Self::value_to_json).collect())
            }
            Value::Map(m) => {
                let obj: serde_json::Map<String, serde_json::Value> = m.iter()
                    .map(|(k, v)| (k.clone(), Self::value_to_json(v)))
                    .collect();
                serde_json::Value::Object(obj)
            }
            Value::Struct(s) => {
                let obj: serde_json::Map<String, serde_json::Value> = s.iter()
                    .map(|(k, v)| (k.clone(), Self::value_to_json(v)))
                    .collect();
                serde_json::Value::Object(obj)
            }
            _ => serde_json::Value::Null,
        }
    }

    /// Get value at JSON path (e.g., "user.name" or "items.0.id")
    fn json_path_get(val: &Value, path: &str) -> Value {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = val.clone();

        for part in parts {
            current = match &current {
                Value::Map(m) => m.get(part).cloned().unwrap_or(Value::Void),
                Value::Struct(s) => s.get(part).cloned().unwrap_or(Value::Void),
                Value::Array(arr) => {
                    if let Ok(idx) = part.parse::<usize>() {
                        arr.get(idx).cloned().unwrap_or(Value::Void)
                    } else {
                        Value::Void
                    }
                }
                _ => Value::Void,
            };

            if matches!(current, Value::Void) {
                break;
            }
        }

        current
    }

    /// Set value at JSON path
    fn json_path_set(val: &Value, path: &str, new_val: &Value) -> Value {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return val.clone();
        }

        if parts.len() == 1 {
            // Direct set
            match val {
                Value::Map(m) => {
                    let mut new_map = m.clone();
                    new_map.insert(parts[0].to_string(), new_val.clone());
                    Value::Map(new_map)
                }
                Value::Struct(s) => {
                    let mut new_struct = s.clone();
                    new_struct.insert(parts[0].to_string(), new_val.clone());
                    Value::Struct(new_struct)
                }
                _ => val.clone()
            }
        } else {
            // Nested set
            let key = parts[0];
            let rest = parts[1..].join(".");

            match val {
                Value::Map(m) => {
                    let mut new_map = m.clone();
                    let nested = m.get(key).cloned().unwrap_or(Value::Map(HashMap::new()));
                    new_map.insert(key.to_string(), Self::json_path_set(&nested, &rest, new_val));
                    Value::Map(new_map)
                }
                Value::Struct(s) => {
                    let mut new_struct = s.clone();
                    let nested = s.get(key).cloned().unwrap_or(Value::Map(HashMap::new()));
                    new_struct.insert(key.to_string(), Self::json_path_set(&nested, &rest, new_val));
                    Value::Struct(new_struct)
                }
                _ => val.clone()
            }
        }
    }

    /// Convert days since March 1, year 0 to year/month/day
    /// Using the algorithm from https://howardhinnant.github.io/date_algorithms.html
    fn days_to_ymd(days: i32) -> (i32, u32, u32) {
        let era = if days >= 0 { days } else { days - 146096 } / 146097;
        let doe = (days - era * 146097) as u32;
        let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
        let y = yoe as i32 + era * 400;
        let doy = doe - (365*yoe + yoe/4 - yoe/100);
        let mp = (5*doy + 2) / 153;
        let d = doy - (153*mp + 2)/5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = if m <= 2 { y + 1 } else { y };
        (year, m, d)
    }

    /// Convert year/month/day to days since March 1, year 0
    fn ymd_to_days(year: i32, month: u32, day: u32) -> i32 {
        let y = if month <= 2 { year - 1 } else { year };
        let m = if month <= 2 { month + 12 } else { month };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = (y - era * 400) as u32;
        let doy = (153 * (m - 3) + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe/4 - yoe/100 + doy;
        era * 146097 + doe as i32
    }

    /// FFI 함수 호출
    fn call_ffi(&mut self, lib_name: &str, fn_name: &str, args: &[Value]) -> RuntimeResult<Value> {
        // 1. 먼저 내장된 FFI 함수 체크 (빠른 경로)
        if let Some(result) = self.call_builtin_ffi(lib_name, fn_name, args) {
            return result;
        }

        // 2. 동적 라이브러리에서 함수 호출 시도
        self.ffi_loader.call_function(lib_name, fn_name, args)
    }

    /// 내장 FFI 함수 호출 (Rust 네이티브 구현)
    fn call_builtin_ffi(&self, lib_name: &str, fn_name: &str, args: &[Value]) -> Option<RuntimeResult<Value>> {
        // 내장 FFI 함수들 - 동적 로딩 없이 빠르게 실행
        let result = match (lib_name, fn_name) {
            // C stdlib 함수들
            ("c", "abs") | ("libc", "abs") => {
                if let Some(Value::Int(n)) = args.first() {
                    Ok(Value::Int(n.abs()))
                } else {
                    Err(RuntimeError::TypeError("abs: expected int".to_string()))
                }
            }
            ("c", "floor") | ("libc", "floor") | ("libm", "floor") => {
                if let Some(Value::Float(f)) = args.first() {
                    Ok(Value::Float(f.floor()))
                } else if let Some(Value::Int(n)) = args.first() {
                    Ok(Value::Int(*n))
                } else {
                    Err(RuntimeError::TypeError("floor: expected number".to_string()))
                }
            }
            ("c", "ceil") | ("libc", "ceil") | ("libm", "ceil") => {
                if let Some(Value::Float(f)) = args.first() {
                    Ok(Value::Float(f.ceil()))
                } else if let Some(Value::Int(n)) = args.first() {
                    Ok(Value::Int(*n))
                } else {
                    Err(RuntimeError::TypeError("ceil: expected number".to_string()))
                }
            }
            ("c", "sqrt") | ("libm", "sqrt") => {
                match args.first() {
                    Some(Value::Float(f)) => Ok(Value::Float(f.sqrt())),
                    Some(Value::Int(n)) => Ok(Value::Float((*n as f64).sqrt())),
                    _ => Err(RuntimeError::TypeError("sqrt: expected number".to_string())),
                }
            }
            ("c", "pow") | ("libm", "pow") => {
                if args.len() >= 2 {
                    let base = match &args[0] {
                        Value::Float(f) => *f,
                        Value::Int(n) => *n as f64,
                        _ => return Some(Err(RuntimeError::TypeError("pow: expected numbers".to_string()))),
                    };
                    let exp = match &args[1] {
                        Value::Float(f) => *f,
                        Value::Int(n) => *n as f64,
                        _ => return Some(Err(RuntimeError::TypeError("pow: expected numbers".to_string()))),
                    };
                    Ok(Value::Float(base.powf(exp)))
                } else {
                    Err(RuntimeError::TypeError("pow: expected 2 arguments".to_string()))
                }
            }
            ("c", "sin") | ("libm", "sin") => {
                match args.first() {
                    Some(Value::Float(f)) => Ok(Value::Float(f.sin())),
                    Some(Value::Int(n)) => Ok(Value::Float((*n as f64).sin())),
                    _ => Err(RuntimeError::TypeError("sin: expected number".to_string())),
                }
            }
            ("c", "cos") | ("libm", "cos") => {
                match args.first() {
                    Some(Value::Float(f)) => Ok(Value::Float(f.cos())),
                    Some(Value::Int(n)) => Ok(Value::Float((*n as f64).cos())),
                    _ => Err(RuntimeError::TypeError("cos: expected number".to_string())),
                }
            }
            ("c", "tan") | ("libm", "tan") => {
                match args.first() {
                    Some(Value::Float(f)) => Ok(Value::Float(f.tan())),
                    Some(Value::Int(n)) => Ok(Value::Float((*n as f64).tan())),
                    _ => Err(RuntimeError::TypeError("tan: expected number".to_string())),
                }
            }
            ("c", "log") | ("libm", "log") => {
                match args.first() {
                    Some(Value::Float(f)) => Ok(Value::Float(f.ln())),
                    Some(Value::Int(n)) => Ok(Value::Float((*n as f64).ln())),
                    _ => Err(RuntimeError::TypeError("log: expected number".to_string())),
                }
            }
            ("c", "exp") | ("libm", "exp") => {
                match args.first() {
                    Some(Value::Float(f)) => Ok(Value::Float(f.exp())),
                    Some(Value::Int(n)) => Ok(Value::Float((*n as f64).exp())),
                    _ => Err(RuntimeError::TypeError("exp: expected number".to_string())),
                }
            }
            ("c", "strlen") | ("libc", "strlen") => {
                if let Some(Value::String(s)) = args.first() {
                    Ok(Value::Int(s.len() as i64))
                } else {
                    Err(RuntimeError::TypeError("strlen: expected string".to_string()))
                }
            }
            ("c", "atoi") | ("libc", "atoi") => {
                if let Some(Value::String(s)) = args.first() {
                    match s.trim().parse::<i64>() {
                        Ok(n) => Ok(Value::Int(n)),
                        Err(_) => Ok(Value::Int(0)),
                    }
                } else {
                    Err(RuntimeError::TypeError("atoi: expected string".to_string()))
                }
            }
            ("c", "atof") | ("libc", "atof") => {
                if let Some(Value::String(s)) = args.first() {
                    match s.trim().parse::<f64>() {
                        Ok(f) => Ok(Value::Float(f)),
                        Err(_) => Ok(Value::Float(0.0)),
                    }
                } else {
                    Err(RuntimeError::TypeError("atof: expected string".to_string()))
                }
            }
            ("c", "getenv") | ("libc", "getenv") => {
                if let Some(Value::String(key)) = args.first() {
                    match std::env::var(key) {
                        Ok(val) => Ok(Value::String(val)),
                        Err(_) => Ok(Value::Void),
                    }
                } else {
                    Err(RuntimeError::TypeError("getenv: expected string".to_string()))
                }
            }
            ("c", "time") | ("libc", "time") => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                Ok(Value::Int(now))
            }
            ("c", "rand") | ("libc", "rand") => {
                // 간단한 난수 생성 (실제로는 rand 크레이트 사용 권장)
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0) as i64;
                Ok(Value::Int((seed % 32767).abs()))
            }
            _ => {
                // 내장 FFI 함수에 없음 - None 반환하여 동적 로딩 시도
                return None;
            }
        };
        Some(result)
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

/// 프로그램 실행 편의 함수
pub fn execute(functions: Vec<CompiledFunction>) -> RuntimeResult<Value> {
    let mut vm = Vm::new();
    vm.load_functions(functions);

    // __main__ 함수가 있으면 실행
    if vm.functions.contains_key("__main__") {
        vm.call_function("__main__", vec![])
    } else {
        Ok(Value::Void)
    }
}

/// 특정 함수 실행
pub fn execute_function(functions: Vec<CompiledFunction>, name: &str, args: Vec<Value>) -> RuntimeResult<Value> {
    let mut vm = Vm::new();
    vm.load_functions(functions);
    vm.call_function(name, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_const(v: Value) -> Instruction {
        Instruction::new(OpCode::Const(v))
    }

    #[test]
    fn test_simple_arithmetic() {
        let func = CompiledFunction {
            name: "__main__".to_string(),
            params: vec![],
            instructions: vec![
                make_const(Value::Int(2)),
                make_const(Value::Int(3)),
                Instruction::new(OpCode::Mul),
                make_const(Value::Int(1)),
                Instruction::new(OpCode::Add),
            ],
        };

        let result = execute(vec![func]).unwrap();
        assert_eq!(result, Value::Int(7)); // 2*3+1 = 7
    }

    #[test]
    fn test_function_call() {
        // add(a, b) = a + b
        // 인자는 VM이 locals에 직접 넣어주므로 Store 불필요
        let add = CompiledFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                Instruction::new(OpCode::Load("a".to_string())),
                Instruction::new(OpCode::Load("b".to_string())),
                Instruction::new(OpCode::Add),
                Instruction::new(OpCode::Return),
            ],
        };

        let result = execute_function(vec![add], "add", vec![Value::Int(3), Value::Int(4)]).unwrap();
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_recursion() {
        // fact(n) = n < 2 ? 1 : n * $(n-1)
        // 인자는 VM이 locals에 직접 넣어주므로 Store 불필요
        //
        // 명령어 인덱스:
        // 0: Load("n")
        // 1: Const(2)
        // 2: Lt
        // 3: JumpIfNot(2)  -- if n >= 2, jump to index 6 (else branch)
        // 4: Const(1)      -- then: push 1
        // 5: Return        -- return 1
        // 6: Load("n")     -- else: push n
        // 7: Load("n")     -- push n again
        // 8: Const(1)
        // 9: Sub           -- n-1
        // 10: SelfCall(1)  -- fact(n-1)
        // 11: Mul          -- n * fact(n-1)
        // 12: Return
        let fact = CompiledFunction {
            name: "fact".to_string(),
            params: vec!["n".to_string()],
            instructions: vec![
                // 0: n < 2
                Instruction::new(OpCode::Load("n".to_string())),
                // 1
                make_const(Value::Int(2)),
                // 2
                Instruction::new(OpCode::Lt),
                // 3: if not (n < 2), jump 2 → index 6
                Instruction::new(OpCode::JumpIfNot(2)),
                // 4: then: return 1
                make_const(Value::Int(1)),
                // 5: return
                Instruction::new(OpCode::Return),
                // 6: else: n * $(n-1)
                Instruction::new(OpCode::Load("n".to_string())),
                // 7
                Instruction::new(OpCode::Load("n".to_string())),
                // 8
                make_const(Value::Int(1)),
                // 9
                Instruction::new(OpCode::Sub),
                // 10
                Instruction::new(OpCode::SelfCall(1)),
                // 11
                Instruction::new(OpCode::Mul),
                // 12
                Instruction::new(OpCode::Return),
            ],
        };

        let result = execute_function(vec![fact], "fact", vec![Value::Int(5)]).unwrap();
        assert_eq!(result, Value::Int(120)); // 5! = 120
    }

    #[test]
    fn test_map_operation() {
        // [1,2,3].@(_*2) = [2,4,6]
        let main = CompiledFunction {
            name: "__main__".to_string(),
            params: vec![],
            instructions: vec![
                make_const(Value::Int(1)),
                make_const(Value::Int(2)),
                make_const(Value::Int(3)),
                Instruction::new(OpCode::MakeArray(3)),
                Instruction::new(OpCode::Map(Box::new(vec![
                    Instruction::new(OpCode::Load("_".to_string())),
                    make_const(Value::Int(2)),
                    Instruction::new(OpCode::Mul),
                ]))),
            ],
        };

        let result = execute(vec![main]).unwrap();
        assert_eq!(result, Value::Array(vec![Value::Int(2), Value::Int(4), Value::Int(6)]));
    }

    #[test]
    fn test_filter_operation() {
        // [1,2,3,4,5].?(_>2) = [3,4,5]
        let main = CompiledFunction {
            name: "__main__".to_string(),
            params: vec![],
            instructions: vec![
                make_const(Value::Int(1)),
                make_const(Value::Int(2)),
                make_const(Value::Int(3)),
                make_const(Value::Int(4)),
                make_const(Value::Int(5)),
                Instruction::new(OpCode::MakeArray(5)),
                Instruction::new(OpCode::Filter(Box::new(vec![
                    Instruction::new(OpCode::Load("_".to_string())),
                    make_const(Value::Int(2)),
                    Instruction::new(OpCode::Gt),
                ]))),
            ],
        };

        let result = execute(vec![main]).unwrap();
        assert_eq!(result, Value::Array(vec![Value::Int(3), Value::Int(4), Value::Int(5)]));
    }

    #[test]
    fn test_reduce_sum() {
        // [1,2,3,4,5]./+ = 15
        let main = CompiledFunction {
            name: "__main__".to_string(),
            params: vec![],
            instructions: vec![
                make_const(Value::Int(1)),
                make_const(Value::Int(2)),
                make_const(Value::Int(3)),
                make_const(Value::Int(4)),
                make_const(Value::Int(5)),
                Instruction::new(OpCode::MakeArray(5)),
                Instruction::new(OpCode::Reduce(ReduceOp::Sum, Value::Int(0))),
            ],
        };

        let result = execute(vec![main]).unwrap();
        assert_eq!(result, Value::Int(15));
    }

    // === std.io tests ===

    #[test]
    fn test_file_io_write_read() {
        use std::fs;
        let test_path = "/tmp/aoel_test_io.txt";
        let test_content = "Hello, AOEL!";

        // Clean up if exists
        let _ = fs::remove_file(test_path);

        let vm = Vm::new();

        // Test write_file
        let result = vm.call_builtin("WRITE_FILE", &[
            Value::String(test_path.to_string()),
            Value::String(test_content.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        // Test read_file
        let result = vm.call_builtin("READ_FILE", &[
            Value::String(test_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String(test_content.to_string())));

        // Test path_exists
        let result = vm.call_builtin("PATH_EXISTS", &[
            Value::String(test_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        // Test path_is_file
        let result = vm.call_builtin("PATH_IS_FILE", &[
            Value::String(test_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        // Test file_size
        let result = vm.call_builtin("FILE_SIZE", &[
            Value::String(test_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Int(test_content.len() as i64)));

        // Cleanup
        fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_file_io_append() {
        use std::fs;
        let test_path = "/tmp/aoel_test_append.txt";

        // Clean up if exists
        let _ = fs::remove_file(test_path);

        let vm = Vm::new();

        // Write initial content
        vm.call_builtin("WRITE_FILE", &[
            Value::String(test_path.to_string()),
            Value::String("Line1\n".to_string()),
        ]).unwrap();

        // Append more content
        vm.call_builtin("APPEND_FILE", &[
            Value::String(test_path.to_string()),
            Value::String("Line2\n".to_string()),
        ]).unwrap();

        // Read and verify
        let result = vm.call_builtin("READ_FILE", &[
            Value::String(test_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("Line1\nLine2\n".to_string())));

        // Test read_lines
        let result = vm.call_builtin("READ_LINES", &[
            Value::String(test_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Array(vec![
            Value::String("Line1".to_string()),
            Value::String("Line2".to_string()),
        ])));

        // Cleanup
        fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_path_functions() {
        let vm = Vm::new();

        // Test path_join
        let result = vm.call_builtin("PATH_JOIN", &[
            Value::String("/home".to_string()),
            Value::String("user".to_string()),
            Value::String("docs".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("/home/user/docs".to_string())));

        // Test path_parent
        let result = vm.call_builtin("PATH_PARENT", &[
            Value::String("/home/user/file.txt".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("/home/user".to_string())));

        // Test path_filename
        let result = vm.call_builtin("PATH_FILENAME", &[
            Value::String("/home/user/file.txt".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("file.txt".to_string())));

        // Test path_extension
        let result = vm.call_builtin("PATH_EXTENSION", &[
            Value::String("/home/user/file.txt".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("txt".to_string())));

        // Test path_stem
        let result = vm.call_builtin("PATH_STEM", &[
            Value::String("/home/user/file.txt".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("file".to_string())));
    }

    #[test]
    fn test_directory_operations() {
        use std::fs;
        let test_dir = "/tmp/aoel_test_dir";
        let test_subdir = "/tmp/aoel_test_dir/subdir";

        // Clean up if exists
        let _ = fs::remove_dir_all(test_dir);

        let vm = Vm::new();

        // Test create_dir_all
        let result = vm.call_builtin("CREATE_DIR_ALL", &[
            Value::String(test_subdir.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        // Test path_is_dir
        let result = vm.call_builtin("PATH_IS_DIR", &[
            Value::String(test_dir.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        // Create a file in the directory
        vm.call_builtin("WRITE_FILE", &[
            Value::String(format!("{}/test.txt", test_dir)),
            Value::String("test".to_string()),
        ]).unwrap();

        // Test list_dir
        let result = vm.call_builtin("LIST_DIR", &[
            Value::String(test_dir.to_string()),
        ]).unwrap();
        if let Some(Value::Array(items)) = result {
            assert!(items.contains(&Value::String("subdir".to_string())));
            assert!(items.contains(&Value::String("test.txt".to_string())));
        } else {
            panic!("Expected array from list_dir");
        }

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_copy_and_rename() {
        use std::fs;
        let src_path = "/tmp/aoel_test_src.txt";
        let dst_path = "/tmp/aoel_test_dst.txt";
        let renamed_path = "/tmp/aoel_test_renamed.txt";

        // Clean up
        let _ = fs::remove_file(src_path);
        let _ = fs::remove_file(dst_path);
        let _ = fs::remove_file(renamed_path);

        let vm = Vm::new();

        // Create source file
        vm.call_builtin("WRITE_FILE", &[
            Value::String(src_path.to_string()),
            Value::String("source content".to_string()),
        ]).unwrap();

        // Test copy_file
        let result = vm.call_builtin("COPY_FILE", &[
            Value::String(src_path.to_string()),
            Value::String(dst_path.to_string()),
        ]).unwrap();
        assert!(matches!(result, Some(Value::Int(_))));

        // Verify copy
        let result = vm.call_builtin("READ_FILE", &[
            Value::String(dst_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("source content".to_string())));

        // Test rename
        let result = vm.call_builtin("RENAME", &[
            Value::String(dst_path.to_string()),
            Value::String(renamed_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        // Verify rename
        let result = vm.call_builtin("PATH_EXISTS", &[
            Value::String(dst_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(false)));

        let result = vm.call_builtin("PATH_EXISTS", &[
            Value::String(renamed_path.to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        // Cleanup
        fs::remove_file(src_path).unwrap();
        fs::remove_file(renamed_path).unwrap();
    }

    #[test]
    fn test_environment_functions() {
        let vm = Vm::new();

        // Test env_set and env_get
        vm.call_builtin("ENV_SET", &[
            Value::String("AOEL_TEST_VAR".to_string()),
            Value::String("test_value".to_string()),
        ]).unwrap();

        let result = vm.call_builtin("ENV_GET", &[
            Value::String("AOEL_TEST_VAR".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("test_value".to_string())));

        // Test cwd
        let result = vm.call_builtin("CWD", &[]).unwrap();
        assert!(matches!(result, Some(Value::String(_))));
    }

    #[test]
    fn test_error_handling() {
        let vm = Vm::new();

        // Test read_file on non-existent file
        let result = vm.call_builtin("READ_FILE", &[
            Value::String("/nonexistent/path/file.txt".to_string()),
        ]).unwrap();
        assert!(matches!(result, Some(Value::Error(_))));

        // Test path_absolute on non-existent path
        let result = vm.call_builtin("PATH_ABSOLUTE", &[
            Value::String("/nonexistent/path".to_string()),
        ]).unwrap();
        assert!(matches!(result, Some(Value::Error(_))));
    }

    // === std.json tests ===

    #[test]
    fn test_json_parse_and_stringify() {
        let vm = Vm::new();

        // Test json_parse with object
        let json_str = r#"{"name": "Alice", "age": 30, "active": true}"#;
        let result = vm.call_builtin("JSON_PARSE", &[
            Value::String(json_str.to_string()),
        ]).unwrap();

        if let Some(Value::Map(m)) = result {
            assert_eq!(m.get("name"), Some(&Value::String("Alice".to_string())));
            assert_eq!(m.get("age"), Some(&Value::Int(30)));
            assert_eq!(m.get("active"), Some(&Value::Bool(true)));
        } else {
            panic!("Expected Map from json_parse");
        }

        // Test json_stringify
        let mut map = HashMap::new();
        map.insert("x".to_string(), Value::Int(10));
        map.insert("y".to_string(), Value::Int(20));
        let result = vm.call_builtin("JSON_STRINGIFY", &[
            Value::Map(map),
        ]).unwrap();

        if let Some(Value::String(s)) = result {
            assert!(s.contains("\"x\":10") || s.contains("\"x\": 10"));
            assert!(s.contains("\"y\":20") || s.contains("\"y\": 20"));
        } else {
            panic!("Expected String from json_stringify");
        }
    }

    #[test]
    fn test_json_parse_array() {
        let vm = Vm::new();

        let json_str = r#"[1, 2, 3, "hello", null]"#;
        let result = vm.call_builtin("JSON_PARSE", &[
            Value::String(json_str.to_string()),
        ]).unwrap();

        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Int(1));
            assert_eq!(arr[1], Value::Int(2));
            assert_eq!(arr[2], Value::Int(3));
            assert_eq!(arr[3], Value::String("hello".to_string()));
            assert_eq!(arr[4], Value::Void);
        } else {
            panic!("Expected Array from json_parse");
        }
    }

    #[test]
    fn test_json_get_nested() {
        let vm = Vm::new();

        let json_str = r#"{"user": {"name": "Bob", "address": {"city": "Seoul"}}}"#;
        let parsed = vm.call_builtin("JSON_PARSE", &[
            Value::String(json_str.to_string()),
        ]).unwrap().unwrap();

        // Test nested path access
        let result = vm.call_builtin("JSON_GET", &[
            parsed.clone(),
            Value::String("user.name".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("Bob".to_string())));

        let result = vm.call_builtin("JSON_GET", &[
            parsed.clone(),
            Value::String("user.address.city".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("Seoul".to_string())));

        // Test non-existent path
        let result = vm.call_builtin("JSON_GET", &[
            parsed,
            Value::String("user.email".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Void));
    }

    #[test]
    fn test_json_set() {
        let vm = Vm::new();

        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("Alice".to_string()));
        let obj = Value::Map(map);

        // Set a new field
        let result = vm.call_builtin("JSON_SET", &[
            obj.clone(),
            Value::String("age".to_string()),
            Value::Int(25),
        ]).unwrap();

        if let Some(Value::Map(m)) = result {
            assert_eq!(m.get("name"), Some(&Value::String("Alice".to_string())));
            assert_eq!(m.get("age"), Some(&Value::Int(25)));
        } else {
            panic!("Expected Map from json_set");
        }

        // Test nested set
        let result = vm.call_builtin("JSON_SET", &[
            obj,
            Value::String("address.city".to_string()),
            Value::String("Seoul".to_string()),
        ]).unwrap();

        if let Some(Value::Map(m)) = &result {
            if let Some(Value::Map(addr)) = m.get("address") {
                assert_eq!(addr.get("city"), Some(&Value::String("Seoul".to_string())));
            } else {
                panic!("Expected nested address object");
            }
        } else {
            panic!("Expected Map from json_set");
        }
    }

    #[test]
    fn test_json_keys_values() {
        let vm = Vm::new();

        let mut map = HashMap::new();
        map.insert("a".to_string(), Value::Int(1));
        map.insert("b".to_string(), Value::Int(2));
        let obj = Value::Map(map);

        // Test json_keys
        let result = vm.call_builtin("JSON_KEYS", std::slice::from_ref(&obj)).unwrap();
        if let Some(Value::Array(keys)) = result {
            assert_eq!(keys.len(), 2);
            assert!(keys.contains(&Value::String("a".to_string())));
            assert!(keys.contains(&Value::String("b".to_string())));
        } else {
            panic!("Expected Array from json_keys");
        }

        // Test json_values
        let result = vm.call_builtin("JSON_VALUES", &[obj]).unwrap();
        if let Some(Value::Array(values)) = result {
            assert_eq!(values.len(), 2);
            assert!(values.contains(&Value::Int(1)));
            assert!(values.contains(&Value::Int(2)));
        } else {
            panic!("Expected Array from json_values");
        }
    }

    #[test]
    fn test_json_has_and_remove() {
        let vm = Vm::new();

        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("Test".to_string()));
        map.insert("value".to_string(), Value::Int(42));
        let obj = Value::Map(map);

        // Test json_has
        let result = vm.call_builtin("JSON_HAS", &[
            obj.clone(),
            Value::String("name".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = vm.call_builtin("JSON_HAS", &[
            obj.clone(),
            Value::String("missing".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::Bool(false)));

        // Test json_remove
        let result = vm.call_builtin("JSON_REMOVE", &[
            obj,
            Value::String("name".to_string()),
        ]).unwrap();

        if let Some(Value::Map(m)) = result {
            assert!(!m.contains_key("name"));
            assert!(m.contains_key("value"));
        } else {
            panic!("Expected Map from json_remove");
        }
    }

    #[test]
    fn test_json_merge() {
        let vm = Vm::new();

        let mut map1 = HashMap::new();
        map1.insert("a".to_string(), Value::Int(1));
        map1.insert("b".to_string(), Value::Int(2));

        let mut map2 = HashMap::new();
        map2.insert("b".to_string(), Value::Int(3)); // Override b
        map2.insert("c".to_string(), Value::Int(4));

        let result = vm.call_builtin("JSON_MERGE", &[
            Value::Map(map1),
            Value::Map(map2),
        ]).unwrap();

        if let Some(Value::Map(m)) = result {
            assert_eq!(m.get("a"), Some(&Value::Int(1)));
            assert_eq!(m.get("b"), Some(&Value::Int(3))); // Overwritten
            assert_eq!(m.get("c"), Some(&Value::Int(4)));
        } else {
            panic!("Expected Map from json_merge");
        }
    }

    #[test]
    fn test_json_type_checks() {
        let vm = Vm::new();

        // Test json_type
        let result = vm.call_builtin("JSON_TYPE", &[Value::Int(42)]).unwrap();
        assert_eq!(result, Some(Value::String("number".to_string())));

        let result = vm.call_builtin("JSON_TYPE", &[Value::String("hello".to_string())]).unwrap();
        assert_eq!(result, Some(Value::String("string".to_string())));

        let result = vm.call_builtin("JSON_TYPE", &[Value::Array(vec![])]).unwrap();
        assert_eq!(result, Some(Value::String("array".to_string())));

        let result = vm.call_builtin("JSON_TYPE", &[Value::Map(HashMap::new())]).unwrap();
        assert_eq!(result, Some(Value::String("object".to_string())));

        let result = vm.call_builtin("JSON_TYPE", &[Value::Void]).unwrap();
        assert_eq!(result, Some(Value::String("null".to_string())));

        // Test json_is_* functions
        let result = vm.call_builtin("JSON_IS_NULL", &[Value::Void]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = vm.call_builtin("JSON_IS_OBJECT", &[Value::Map(HashMap::new())]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = vm.call_builtin("JSON_IS_ARRAY", &[Value::Array(vec![])]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_json_parse_error() {
        let vm = Vm::new();

        // Test invalid JSON
        let result = vm.call_builtin("JSON_PARSE", &[
            Value::String("{ invalid json }".to_string()),
        ]).unwrap();
        assert!(matches!(result, Some(Value::Error(_))));
    }

    // === std.net tests ===

    #[test]
    fn test_url_encode_decode() {
        let vm = Vm::new();

        // Test url_encode
        let result = vm.call_builtin("URL_ENCODE", &[
            Value::String("hello world".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("hello+world".to_string())));

        let result = vm.call_builtin("URL_ENCODE", &[
            Value::String("a=1&b=2".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("a%3D1%26b%3D2".to_string())));

        // Test url_decode
        let result = vm.call_builtin("URL_DECODE", &[
            Value::String("hello+world".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("hello world".to_string())));

        let result = vm.call_builtin("URL_DECODE", &[
            Value::String("a%3D1%26b%3D2".to_string()),
        ]).unwrap();
        assert_eq!(result, Some(Value::String("a=1&b=2".to_string())));
    }

    #[test]
    fn test_http_error_handling() {
        let vm = Vm::new();

        // Test with invalid URL (should return error)
        let result = vm.call_builtin("HTTP_GET", &[
            Value::String("http://localhost:99999/nonexistent".to_string()),
        ]).unwrap();
        assert!(matches!(result, Some(Value::Error(_))));

        // Test with missing arguments
        let result = vm.call_builtin("HTTP_POST", &[
            Value::String("http://example.com".to_string()),
        ]).unwrap();
        assert!(matches!(result, Some(Value::Error(_))));
    }

    // Note: Real HTTP tests would require a test server or mock
    // The following test is marked as ignored for CI but can be run manually
    #[test]
    #[ignore]
    fn test_http_get_real() {
        let vm = Vm::new();

        // Test against a real public API
        let result = vm.call_builtin("HTTP_GET_JSON", &[
            Value::String("https://httpbin.org/get".to_string()),
        ]).unwrap();

        if let Some(Value::Map(m)) = result {
            assert!(m.contains_key("url"));
        } else {
            panic!("Expected Map from http_get_json");
        }
    }

    // === New stdlib tests ===

    #[test]
    fn test_time_functions() {
        let vm = Vm::new();

        // Test time_now returns a timestamp
        let result = vm.call_builtin("TIME_NOW", &[]).unwrap();
        assert!(matches!(result, Some(Value::Int(_))));

        // Test time_now_ms returns milliseconds
        let result = vm.call_builtin("TIME_NOW_MS", &[]).unwrap();
        assert!(matches!(result, Some(Value::Int(_))));

        // Test time_format
        let result = vm.call_builtin("TIME_FORMAT", &[Value::Int(0)]).unwrap();
        assert!(matches!(result, Some(Value::String(_))));
        if let Some(Value::String(s)) = result {
            assert!(s.starts_with("1970-01-01T"));
        }

        // Test time_parse
        let result = vm.call_builtin("TIME_PARSE", &[
            Value::String("1970-01-01T00:00:00Z".to_string())
        ]).unwrap();
        assert_eq!(result, Some(Value::Int(0)));
    }

    #[test]
    fn test_random_functions() {
        let vm = Vm::new();

        // Test random returns float between 0 and 1
        let result = vm.call_builtin("RANDOM", &[]).unwrap();
        if let Some(Value::Float(f)) = result {
            assert!(f >= 0.0 && f < 1.0);
        } else {
            panic!("Expected float from random");
        }

        // Test random_int
        let result = vm.call_builtin("RANDOM_INT", &[
            Value::Int(1),
            Value::Int(10)
        ]).unwrap();
        if let Some(Value::Int(n)) = result {
            assert!(n >= 1 && n <= 10);
        } else {
            panic!("Expected int from random_int");
        }

        // Test shuffle
        let result = vm.call_builtin("SHUFFLE", &[
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        ]).unwrap();
        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array from shuffle");
        }

        // Test sample
        let result = vm.call_builtin("SAMPLE", &[
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)]),
            Value::Int(2)
        ]).unwrap();
        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected array from sample");
        }
    }

    #[test]
    fn test_new_math_functions() {
        let vm = Vm::new();

        // Test sign
        assert_eq!(vm.call_builtin("SIGN", &[Value::Int(-5)]).unwrap(), Some(Value::Int(-1)));
        assert_eq!(vm.call_builtin("SIGN", &[Value::Int(0)]).unwrap(), Some(Value::Int(0)));
        assert_eq!(vm.call_builtin("SIGN", &[Value::Int(5)]).unwrap(), Some(Value::Int(1)));

        // Test hypot
        let result = vm.call_builtin("HYPOT", &[Value::Int(3), Value::Int(4)]).unwrap();
        if let Some(Value::Float(f)) = result {
            assert!((f - 5.0).abs() < 1e-10);
        } else {
            panic!("Expected float from hypot");
        }

        // Test sinh/cosh/tanh
        let result = vm.call_builtin("SINH", &[Value::Int(0)]).unwrap();
        if let Some(Value::Float(f)) = result {
            assert!(f.abs() < 1e-10);
        }

        let result = vm.call_builtin("COSH", &[Value::Int(0)]).unwrap();
        if let Some(Value::Float(f)) = result {
            assert!((f - 1.0).abs() < 1e-10);
        }

        let result = vm.call_builtin("TANH", &[Value::Int(0)]).unwrap();
        if let Some(Value::Float(f)) = result {
            assert!(f.abs() < 1e-10);
        }
    }

    #[test]
    fn test_string_functions() {
        let vm = Vm::new();

        // Test char_code (string to int)
        assert_eq!(vm.call_builtin("CHAR_CODE", &[Value::String("A".to_string())]).unwrap(), Some(Value::Int(65)));

        // Test char_code (int to string)
        assert_eq!(vm.call_builtin("CHAR_CODE", &[Value::Int(65)]).unwrap(), Some(Value::String("A".to_string())));

        // Test hex (int to hex)
        assert_eq!(vm.call_builtin("HEX", &[Value::Int(255)]).unwrap(), Some(Value::String("ff".to_string())));

        // Test hex (hex to int)
        assert_eq!(vm.call_builtin("HEX", &[Value::String("ff".to_string())]).unwrap(), Some(Value::Int(255)));

        // Test bin (int to binary)
        assert_eq!(vm.call_builtin("BIN", &[Value::Int(5)]).unwrap(), Some(Value::String("101".to_string())));

        // Test bin (binary to int)
        assert_eq!(vm.call_builtin("BIN", &[Value::String("101".to_string())]).unwrap(), Some(Value::Int(5)));

        // Test format
        let result = vm.call_builtin("FORMAT", &[
            Value::String("Hello, {}!".to_string()),
            Value::String("World".to_string())
        ]).unwrap();
        assert_eq!(result, Some(Value::String("Hello, World!".to_string())));
    }

    #[test]
    fn test_array_functions() {
        let vm = Vm::new();

        // Test sum
        let result = vm.call_builtin("SUM", &[
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        ]).unwrap();
        assert_eq!(result, Some(Value::Int(6)));

        // Test product
        let result = vm.call_builtin("PRODUCT", &[
            Value::Array(vec![Value::Int(2), Value::Int(3), Value::Int(4)])
        ]).unwrap();
        assert_eq!(result, Some(Value::Int(24)));

        // Test average
        let result = vm.call_builtin("AVERAGE", &[
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        ]).unwrap();
        if let Some(Value::Float(f)) = result {
            assert!((f - 2.0).abs() < 1e-10);
        }

        // Test find
        let result = vm.call_builtin("FIND", &[
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
            Value::Int(2)
        ]).unwrap();
        assert_eq!(result, Some(Value::Int(1)));

        // Test count
        let result = vm.call_builtin("COUNT", &[
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(1), Value::Int(1)]),
            Value::Int(1)
        ]).unwrap();
        assert_eq!(result, Some(Value::Int(3)));

        // Test group_by
        let result = vm.call_builtin("GROUP_BY", &[
            Value::Array(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("a".to_string())
            ])
        ]).unwrap();
        if let Some(Value::Map(m)) = result {
            assert!(m.contains_key("a"));
            assert!(m.contains_key("b"));
        }

        // Test partition
        let result = vm.call_builtin("PARTITION", &[
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]),
            Value::Int(2)
        ]).unwrap();
        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 2);
            if let (Value::Array(left), Value::Array(right)) = (&arr[0], &arr[1]) {
                assert_eq!(left.len(), 2);
                assert_eq!(right.len(), 2);
            }
        }

        // Test chunk
        let result = vm.call_builtin("CHUNK", &[
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)]),
            Value::Int(2)
        ]).unwrap();
        if let Some(Value::Array(chunks)) = result {
            assert_eq!(chunks.len(), 3);
        }
    }
}
