//! v6b VM Implementation
//!
//! 스택 기반 VM으로 v6b IR을 실행

use std::collections::HashMap;
use aoel_ir::{Instruction, OpCode, ReduceOp, Value};
use aoel_lowering::CompiledFunction;
use crate::error::{RuntimeError, RuntimeResult};

const MAX_RECURSION_DEPTH: usize = 1000;

/// Result type for TCO-aware execution
enum TcoResult {
    /// Normal return
    Return,
    /// Tail call with new arguments
    TailCall(Vec<Value>),
}

/// v6b Virtual Machine
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
        }
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
                    .ok_or_else(|| RuntimeError::IndexOutOfBounds { index: i, length: char_count })?;
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
                            let padding: String = std::iter::repeat(pad_char)
                                .take(target_len - s.len())
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
                            let padding: String = std::iter::repeat(pad_char)
                                .take(target_len - s.len())
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

            _ => None,
        };
        Ok(result)
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
}
