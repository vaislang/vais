//! WASM용 경량 VM
//!
//! 웹 브라우저에서 실행되므로 파일 I/O, 네트워크 등은 제외

use std::collections::HashMap;
use aoel_ir::{Instruction, OpCode, ReduceOp, Value};
use aoel_lowering::CompiledFunction;

const MAX_RECURSION_DEPTH: usize = 500;

/// WASM VM 에러
#[derive(Debug)]
pub enum WasmVmError {
    StackUnderflow,
    DivisionByZero,
    TypeError(String),
    UndefinedVariable(String),
    UndefinedFunction(String),
    IndexOutOfBounds(i64, usize),
    MaxRecursionDepth,
    Internal(String),
}

impl std::fmt::Display for WasmVmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmVmError::StackUnderflow => write!(f, "Stack underflow"),
            WasmVmError::DivisionByZero => write!(f, "Division by zero"),
            WasmVmError::TypeError(s) => write!(f, "Type error: {}", s),
            WasmVmError::UndefinedVariable(s) => write!(f, "Undefined variable: {}", s),
            WasmVmError::UndefinedFunction(s) => write!(f, "Undefined function: {}", s),
            WasmVmError::IndexOutOfBounds(i, len) => write!(f, "Index {} out of bounds (length {})", i, len),
            WasmVmError::MaxRecursionDepth => write!(f, "Maximum recursion depth exceeded"),
            WasmVmError::Internal(s) => write!(f, "Internal error: {}", s),
        }
    }
}

pub type WasmVmResult<T> = Result<T, WasmVmError>;

/// WASM용 경량 VM
pub struct WasmVm {
    stack: Vec<Value>,
    locals: HashMap<String, Value>,
    functions: HashMap<String, CompiledFunction>,
    current_function: Option<String>,
    recursion_depth: usize,
    closures: HashMap<usize, Vec<Instruction>>,
    next_closure_id: usize,
    output: String,
}

impl WasmVm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            locals: HashMap::new(),
            functions: HashMap::new(),
            current_function: None,
            recursion_depth: 0,
            closures: HashMap::new(),
            next_closure_id: 0,
            output: String::new(),
        }
    }

    pub fn load_functions(&mut self, functions: Vec<CompiledFunction>) {
        for func in functions {
            self.functions.insert(func.name.clone(), func);
        }
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn first_function_name(&self) -> Option<&str> {
        self.functions.keys().next().map(|s| s.as_str())
    }

    pub fn get_output(&self) -> String {
        self.output.clone()
    }

    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> WasmVmResult<Value> {
        let func = self.functions.get(name)
            .ok_or_else(|| WasmVmError::UndefinedFunction(name.to_string()))?
            .clone();

        self.recursion_depth += 1;
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            return Err(WasmVmError::MaxRecursionDepth);
        }

        let prev_stack = std::mem::take(&mut self.stack);
        let prev_locals = std::mem::take(&mut self.locals);
        let prev_function = self.current_function.take();

        for (i, param) in func.params.iter().enumerate() {
            if i < args.len() {
                self.locals.insert(param.clone(), args[i].clone());
            }
        }

        self.current_function = Some(name.to_string());

        self.execute_instructions(&func.instructions)?;

        let return_value = self.pop().unwrap_or(Value::Void);

        self.stack = prev_stack;
        self.locals = prev_locals;
        self.current_function = prev_function;
        self.recursion_depth -= 1;

        Ok(return_value)
    }

    fn execute_instructions(&mut self, instructions: &[Instruction]) -> WasmVmResult<()> {
        let mut ip = 0;

        while ip < instructions.len() {
            let instr = &instructions[ip];
            ip += 1;

            match &instr.opcode {
                OpCode::Const(v) => self.stack.push(v.clone()),
                OpCode::Load(name) => {
                    let val = self.locals.get(name)
                        .ok_or_else(|| WasmVmError::UndefinedVariable(name.clone()))?
                        .clone();
                    self.stack.push(val);
                }
                OpCode::Store(name) => {
                    let val = self.pop()?;
                    self.locals.insert(name.clone(), val);
                }
                OpCode::Pop => { self.pop()?; }
                OpCode::Dup => {
                    let val = self.peek()?.clone();
                    self.stack.push(val);
                }

                // 산술
                OpCode::Add => self.binary_op(|a, b| a + b, |a, b| a + b)?,
                OpCode::Sub => self.binary_op(|a, b| a - b, |a, b| a - b)?,
                OpCode::Mul => self.binary_op(|a, b| a * b, |a, b| a * b)?,
                OpCode::Div => self.div_op()?,
                OpCode::Mod => self.mod_op()?,
                OpCode::Neg => self.unary_op(|a| -a, |a| -a)?,

                // 비교
                OpCode::Eq => self.compare_op(|a, b| a == b)?,
                OpCode::Neq => self.compare_op(|a, b| a != b)?,
                OpCode::Lt => self.compare_op(|a, b| a < b)?,
                OpCode::Gt => self.compare_op(|a, b| a > b)?,
                OpCode::Lte => self.compare_op(|a, b| a <= b)?,
                OpCode::Gte => self.compare_op(|a, b| a >= b)?,

                // 논리
                OpCode::And => {
                    let b = self.pop_bool()?;
                    let a = self.pop_bool()?;
                    self.stack.push(Value::Bool(a && b));
                }
                OpCode::Or => {
                    let b = self.pop_bool()?;
                    let a = self.pop_bool()?;
                    self.stack.push(Value::Bool(a || b));
                }
                OpCode::Not => {
                    let a = self.pop_bool()?;
                    self.stack.push(Value::Bool(!a));
                }

                // 점프
                OpCode::Jump(offset) => {
                    ip = (ip as i64 + *offset as i64 - 1) as usize;
                }
                OpCode::JumpIfNot(offset) => {
                    let cond = self.pop_bool()?;
                    if !cond {
                        ip = (ip as i64 + *offset as i64 - 1) as usize;
                    }
                }
                OpCode::JumpIf(offset) => {
                    let cond = self.pop_bool()?;
                    if cond {
                        ip = (ip as i64 + *offset as i64 - 1) as usize;
                    }
                }

                // 함수 호출
                OpCode::Call(name, arg_count) => {
                    let mut args = Vec::with_capacity(*arg_count);
                    for _ in 0..*arg_count {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    let result = if self.is_builtin(name) {
                        self.call_builtin(name, args)?
                    } else {
                        self.call_function(name, args)?
                    };
                    self.stack.push(result);
                }

                OpCode::SelfCall(arg_count) => {
                    let func_name = self.current_function.as_ref()
                        .ok_or_else(|| WasmVmError::Internal("No current function".to_string()))?
                        .clone();

                    let mut args = Vec::with_capacity(*arg_count);
                    for _ in 0..*arg_count {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    let result = self.call_function(&func_name, args)?;
                    self.stack.push(result);
                }

                // 배열/맵
                OpCode::MakeArray(size) => {
                    let mut elems = Vec::with_capacity(*size);
                    for _ in 0..*size {
                        elems.push(self.pop()?);
                    }
                    elems.reverse();
                    self.stack.push(Value::Array(elems));
                }
                OpCode::MakeStruct(fields) => {
                    let mut map = HashMap::new();
                    for field in fields.iter().rev() {
                        let value = self.pop()?;
                        map.insert(field.clone(), value);
                    }
                    self.stack.push(Value::Struct(map));
                }
                OpCode::Index => {
                    let index = self.pop()?;
                    let arr = self.pop()?;
                    let result = self.index_op(arr, index)?;
                    self.stack.push(result);
                }
                OpCode::GetField(field) => {
                    let obj = self.pop()?;
                    let result = self.get_field(obj, field)?;
                    self.stack.push(result);
                }
                OpCode::Len => {
                    let val = self.pop()?;
                    let len = match val {
                        Value::Array(arr) => arr.len() as i64,
                        Value::String(s) => s.len() as i64,
                        Value::Map(m) => m.len() as i64,
                        _ => return Err(WasmVmError::TypeError("Cannot get length".into())),
                    };
                    self.stack.push(Value::Int(len));
                }
                OpCode::Concat => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.concat_op(a, b)?;
                    self.stack.push(result);
                }

                // 컬렉션 연산
                OpCode::Map(body) => self.map_op(body)?,
                OpCode::Filter(body) => self.filter_op(body)?,
                OpCode::Reduce(op, _init) => self.reduce_op(op)?,
                OpCode::Range => {
                    let end = self.pop_int()?;
                    let start = self.pop_int()?;
                    let range: Vec<Value> = (start..end).map(Value::Int).collect();
                    self.stack.push(Value::Array(range));
                }

                // 클로저
                OpCode::MakeClosure(params, body) => {
                    let id = self.next_closure_id;
                    self.next_closure_id += 1;
                    self.closures.insert(id, (**body).clone());
                    self.stack.push(Value::Closure {
                        params: params.clone(),
                        captured: HashMap::new(),
                        body_id: id,
                    });
                }
                OpCode::CallClosure(arg_count) => {
                    let mut args = Vec::with_capacity(*arg_count);
                    for _ in 0..*arg_count {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    let closure = self.pop()?;
                    let result = self.call_closure(closure, args)?;
                    self.stack.push(result);
                }

                OpCode::Return => break,
                OpCode::Nop => {}

                _ => {
                    // 지원하지 않는 opcode는 무시
                }
            }
        }

        Ok(())
    }

    fn pop(&mut self) -> WasmVmResult<Value> {
        self.stack.pop().ok_or(WasmVmError::StackUnderflow)
    }

    fn peek(&self) -> WasmVmResult<&Value> {
        self.stack.last().ok_or(WasmVmError::StackUnderflow)
    }

    fn pop_int(&mut self) -> WasmVmResult<i64> {
        match self.pop()? {
            Value::Int(n) => Ok(n),
            Value::Float(f) => Ok(f as i64),
            v => Err(WasmVmError::TypeError(format!("Expected int, got {:?}", v))),
        }
    }

    fn pop_bool(&mut self) -> WasmVmResult<bool> {
        match self.pop()? {
            Value::Bool(b) => Ok(b),
            Value::Int(n) => Ok(n != 0),
            v => Err(WasmVmError::TypeError(format!("Expected bool, got {:?}", v))),
        }
    }

    fn binary_op<F, G>(&mut self, int_op: F, float_op: G) -> WasmVmResult<()>
    where
        F: Fn(i64, i64) -> i64,
        G: Fn(f64, f64) -> f64,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (a, b) {
            (Value::Int(x), Value::Int(y)) => Value::Int(int_op(x, y)),
            (Value::Float(x), Value::Float(y)) => Value::Float(float_op(x, y)),
            (Value::Int(x), Value::Float(y)) => Value::Float(float_op(x as f64, y)),
            (Value::Float(x), Value::Int(y)) => Value::Float(float_op(x, y as f64)),
            _ => return Err(WasmVmError::TypeError("Binary op type error".into())),
        };
        self.stack.push(result);
        Ok(())
    }

    fn unary_op<F, G>(&mut self, int_op: F, float_op: G) -> WasmVmResult<()>
    where
        F: Fn(i64) -> i64,
        G: Fn(f64) -> f64,
    {
        let a = self.pop()?;
        let result = match a {
            Value::Int(x) => Value::Int(int_op(x)),
            Value::Float(x) => Value::Float(float_op(x)),
            _ => return Err(WasmVmError::TypeError("Unary op type error".into())),
        };
        self.stack.push(result);
        Ok(())
    }

    fn div_op(&mut self) -> WasmVmResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 { return Err(WasmVmError::DivisionByZero); }
                Value::Int(x / y)
            }
            (Value::Float(x), Value::Float(y)) => Value::Float(x / y),
            (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 / y),
            (Value::Float(x), Value::Int(y)) => Value::Float(x / y as f64),
            _ => return Err(WasmVmError::TypeError("Division type error".into())),
        };
        self.stack.push(result);
        Ok(())
    }

    fn mod_op(&mut self) -> WasmVmResult<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 { return Err(WasmVmError::DivisionByZero); }
                Value::Int(x % y)
            }
            (Value::Float(x), Value::Float(y)) => Value::Float(x % y),
            (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 % y),
            (Value::Float(x), Value::Int(y)) => Value::Float(x % y as f64),
            _ => return Err(WasmVmError::TypeError("Modulo type error".into())),
        };
        self.stack.push(result);
        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> WasmVmResult<()>
    where
        F: Fn(f64, f64) -> bool,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (a, b) {
            (Value::Int(x), Value::Int(y)) => op(x as f64, y as f64),
            (Value::Float(x), Value::Float(y)) => op(x, y),
            (Value::Int(x), Value::Float(y)) => op(x as f64, y),
            (Value::Float(x), Value::Int(y)) => op(x, y as f64),
            (Value::String(x), Value::String(y)) => x.cmp(&y) == std::cmp::Ordering::Equal && op(0.0, 0.0)
                || x.cmp(&y) == std::cmp::Ordering::Less && op(-1.0, 0.0)
                || x.cmp(&y) == std::cmp::Ordering::Greater && op(1.0, 0.0),
            (Value::Bool(x), Value::Bool(y)) => op(x as i32 as f64, y as i32 as f64),
            _ => return Err(WasmVmError::TypeError("Compare type error".into())),
        };
        self.stack.push(Value::Bool(result));
        Ok(())
    }

    fn index_op(&self, arr: Value, index: Value) -> WasmVmResult<Value> {
        match (arr, index) {
            (Value::Array(a), Value::Int(i)) => {
                let idx = if i < 0 { a.len() as i64 + i } else { i } as usize;
                a.get(idx).cloned().ok_or(WasmVmError::IndexOutOfBounds(i, a.len()))
            }
            (Value::String(s), Value::Int(i)) => {
                let idx = if i < 0 { s.len() as i64 + i } else { i } as usize;
                s.chars().nth(idx)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or(WasmVmError::IndexOutOfBounds(i, s.len()))
            }
            (Value::Map(m), Value::String(k)) => {
                m.get(&k).cloned().ok_or(WasmVmError::UndefinedVariable(k))
            }
            _ => Err(WasmVmError::TypeError("Invalid index operation".into())),
        }
    }

    fn get_field(&self, obj: Value, field: &str) -> WasmVmResult<Value> {
        match obj {
            Value::Map(m) => m.get(field).cloned()
                .ok_or(WasmVmError::UndefinedVariable(field.to_string())),
            Value::Array(a) => match field {
                "length" | "len" => Ok(Value::Int(a.len() as i64)),
                "first" => a.first().cloned().ok_or(WasmVmError::IndexOutOfBounds(0, 0)),
                "last" => a.last().cloned().ok_or(WasmVmError::IndexOutOfBounds(-1, 0)),
                _ => Err(WasmVmError::UndefinedVariable(field.to_string())),
            },
            Value::String(s) => match field {
                "length" | "len" => Ok(Value::Int(s.len() as i64)),
                _ => Err(WasmVmError::UndefinedVariable(field.to_string())),
            },
            _ => Err(WasmVmError::TypeError("Field access on non-object".into())),
        }
    }

    fn concat_op(&self, a: Value, b: Value) -> WasmVmResult<Value> {
        match (a, b) {
            (Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)),
            (Value::Array(mut x), Value::Array(y)) => {
                x.extend(y);
                Ok(Value::Array(x))
            }
            (Value::String(x), y) => Ok(Value::String(x + &format!("{}", y))),
            (x, Value::String(y)) => Ok(Value::String(format!("{}", x) + &y)),
            _ => Err(WasmVmError::TypeError("Cannot concat".into())),
        }
    }

    fn map_op(&mut self, body: &[Instruction]) -> WasmVmResult<()> {
        let arr = self.pop()?;

        if let Value::Array(items) = arr {
            let mut result = Vec::with_capacity(items.len());
            for item in items {
                let prev_locals = std::mem::take(&mut self.locals);
                self.locals.insert("_".to_string(), item);

                self.execute_instructions(body)?;

                let mapped = self.pop().unwrap_or(Value::Void);
                self.locals = prev_locals;
                result.push(mapped);
            }
            self.stack.push(Value::Array(result));
            Ok(())
        } else {
            Err(WasmVmError::TypeError("Map requires array".into()))
        }
    }

    fn filter_op(&mut self, body: &[Instruction]) -> WasmVmResult<()> {
        let arr = self.pop()?;

        if let Value::Array(items) = arr {
            let mut result = Vec::new();
            for item in items {
                let prev_locals = std::mem::take(&mut self.locals);
                self.locals.insert("_".to_string(), item.clone());

                self.execute_instructions(body)?;

                let keep = self.pop().unwrap_or(Value::Bool(false));
                self.locals = prev_locals;
                if matches!(keep, Value::Bool(true)) {
                    result.push(item);
                }
            }
            self.stack.push(Value::Array(result));
            Ok(())
        } else {
            Err(WasmVmError::TypeError("Filter requires array".into()))
        }
    }

    fn reduce_op(&mut self, op: &ReduceOp) -> WasmVmResult<()> {
        let arr = self.pop()?;

        if let Value::Array(items) = arr {
            let result = match op {
                ReduceOp::Sum => {
                    let mut sum = Value::Int(0);
                    for item in items {
                        sum = self.add_values(sum, item)?;
                    }
                    sum
                }
                ReduceOp::Product => {
                    let mut prod = Value::Int(1);
                    for item in items {
                        prod = self.mul_values(prod, item)?;
                    }
                    prod
                }
                ReduceOp::Min => {
                    items.into_iter().reduce(|a, b| self.min_value(a, b))
                        .unwrap_or(Value::Void)
                }
                ReduceOp::Max => {
                    items.into_iter().reduce(|a, b| self.max_value(a, b))
                        .unwrap_or(Value::Void)
                }
                ReduceOp::All => {
                    let all = items.iter().all(|v| matches!(v, Value::Bool(true)));
                    Value::Bool(all)
                }
                ReduceOp::Any => {
                    let any = items.iter().any(|v| matches!(v, Value::Bool(true)));
                    Value::Bool(any)
                }
                ReduceOp::Count => {
                    Value::Int(items.len() as i64)
                }
                ReduceOp::Avg => {
                    if items.is_empty() {
                        Value::Float(0.0)
                    } else {
                        let mut sum = 0.0;
                        for item in &items {
                            match item {
                                Value::Int(n) => sum += *n as f64,
                                Value::Float(f) => sum += *f,
                                _ => {}
                            }
                        }
                        Value::Float(sum / items.len() as f64)
                    }
                }
                ReduceOp::First => {
                    items.into_iter().next().unwrap_or(Value::Void)
                }
                ReduceOp::Last => {
                    items.into_iter().last().unwrap_or(Value::Void)
                }
                ReduceOp::Custom(_body) => {
                    // Custom reduce는 미지원
                    Value::Void
                }
            };
            self.stack.push(result);
            Ok(())
        } else {
            Err(WasmVmError::TypeError("Reduce requires array".into()))
        }
    }

    fn call_closure(&mut self, closure: Value, args: Vec<Value>) -> WasmVmResult<Value> {
        match closure {
            Value::Closure { params, body_id, .. } => {
                let body = self.closures.get(&body_id)
                    .ok_or_else(|| WasmVmError::Internal("Closure not found".into()))?
                    .clone();

                let prev_locals = std::mem::take(&mut self.locals);

                // 파라미터 바인딩
                for (i, param) in params.iter().enumerate() {
                    if i < args.len() {
                        self.locals.insert(param.clone(), args[i].clone());
                    }
                }
                // _ 파라미터도 설정
                if let Some(arg) = args.first() {
                    self.locals.insert("_".to_string(), arg.clone());
                }

                self.execute_instructions(&body)?;

                let result = self.pop().unwrap_or(Value::Void);
                self.locals = prev_locals;
                Ok(result)
            }
            _ => Err(WasmVmError::TypeError("Not a closure".into())),
        }
    }

    fn add_values(&self, a: Value, b: Value) -> WasmVmResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 + y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + y as f64)),
            _ => Err(WasmVmError::TypeError("Cannot add".into())),
        }
    }

    fn mul_values(&self, a: Value, b: Value) -> WasmVmResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 * y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x * y as f64)),
            _ => Err(WasmVmError::TypeError("Cannot multiply".into())),
        }
    }

    fn min_value(&self, a: Value, b: Value) -> Value {
        match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => if x < y { a } else { b },
            (Value::Float(x), Value::Float(y)) => if x < y { a } else { b },
            _ => a,
        }
    }

    fn max_value(&self, a: Value, b: Value) -> Value {
        match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => if x > y { a } else { b },
            (Value::Float(x), Value::Float(y)) => if x > y { a } else { b },
            _ => a,
        }
    }

    fn is_builtin(&self, name: &str) -> bool {
        matches!(name,
            "print" | "println" | "len" | "type" | "str" | "int" | "float" |
            "abs" | "sqrt" | "pow" | "sin" | "cos" | "tan" | "log" | "exp" |
            "floor" | "ceil" | "round" | "min" | "max" |
            "head" | "tail" | "init" | "last" | "reverse" | "sort" | "unique" |
            "concat" | "flatten" | "zip" | "enumerate" | "sum" | "product" |
            "split" | "join" | "trim" | "upper" | "lower" | "contains" | "replace" |
            "keys" | "values" | "entries" | "has_key" |
            "range" | "repeat" | "take" | "drop" | "slice"
        )
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> WasmVmResult<Value> {
        match name {
            "print" | "println" => {
                let output: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
                let line = output.join(" ");
                if !self.output.is_empty() {
                    self.output.push('\n');
                }
                self.output.push_str(&line);
                Ok(Value::Void)
            }
            "len" => {
                let val = args.first().ok_or(WasmVmError::TypeError("len requires 1 argument".into()))?;
                match val {
                    Value::Array(a) => Ok(Value::Int(a.len() as i64)),
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    Value::Map(m) => Ok(Value::Int(m.len() as i64)),
                    _ => Err(WasmVmError::TypeError("len: invalid type".into())),
                }
            }
            "type" => {
                let val = args.first().ok_or(WasmVmError::TypeError("type requires 1 argument".into()))?;
                let t = match val {
                    Value::Int(_) => "int",
                    Value::Float(_) => "float",
                    Value::Bool(_) => "bool",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Map(_) => "map",
                    Value::Struct(_) => "struct",
                    Value::Void => "void",
                    Value::Closure { .. } => "closure",
                    Value::Bytes(_) => "bytes",
                    Value::Optional(_) => "optional",
                    Value::Error(_) => "error",
                };
                Ok(Value::String(t.to_string()))
            }
            "str" => {
                let val = args.first().ok_or(WasmVmError::TypeError("str requires 1 argument".into()))?;
                Ok(Value::String(format!("{}", val)))
            }
            "int" => {
                let val = args.first().ok_or(WasmVmError::TypeError("int requires 1 argument".into()))?;
                match val {
                    Value::Int(n) => Ok(Value::Int(*n)),
                    Value::Float(f) => Ok(Value::Int(*f as i64)),
                    Value::String(s) => Ok(Value::Int(s.parse().unwrap_or(0))),
                    Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                    _ => Err(WasmVmError::TypeError("Cannot convert to int".into())),
                }
            }
            "float" => {
                let val = args.first().ok_or(WasmVmError::TypeError("float requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(*f)),
                    Value::Int(n) => Ok(Value::Float(*n as f64)),
                    Value::String(s) => Ok(Value::Float(s.parse().unwrap_or(0.0))),
                    _ => Err(WasmVmError::TypeError("Cannot convert to float".into())),
                }
            }
            "abs" => {
                let val = args.first().ok_or(WasmVmError::TypeError("abs requires 1 argument".into()))?;
                match val {
                    Value::Int(n) => Ok(Value::Int(n.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    _ => Err(WasmVmError::TypeError("abs: expected number".into())),
                }
            }
            "sqrt" => {
                let val = args.first().ok_or(WasmVmError::TypeError("sqrt requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.sqrt())),
                    Value::Int(n) => Ok(Value::Float((*n as f64).sqrt())),
                    _ => Err(WasmVmError::TypeError("sqrt: expected number".into())),
                }
            }
            "pow" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("pow requires 2 arguments".into()));
                }
                let base = match &args[0] {
                    Value::Float(f) => *f,
                    Value::Int(n) => *n as f64,
                    _ => return Err(WasmVmError::TypeError("pow: expected number".into())),
                };
                let exp = match &args[1] {
                    Value::Float(f) => *f,
                    Value::Int(n) => *n as f64,
                    _ => return Err(WasmVmError::TypeError("pow: expected number".into())),
                };
                Ok(Value::Float(base.powf(exp)))
            }
            "sin" => {
                let val = args.first().ok_or(WasmVmError::TypeError("sin requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.sin())),
                    Value::Int(n) => Ok(Value::Float((*n as f64).sin())),
                    _ => Err(WasmVmError::TypeError("sin: expected number".into())),
                }
            }
            "cos" => {
                let val = args.first().ok_or(WasmVmError::TypeError("cos requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.cos())),
                    Value::Int(n) => Ok(Value::Float((*n as f64).cos())),
                    _ => Err(WasmVmError::TypeError("cos: expected number".into())),
                }
            }
            "tan" => {
                let val = args.first().ok_or(WasmVmError::TypeError("tan requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.tan())),
                    Value::Int(n) => Ok(Value::Float((*n as f64).tan())),
                    _ => Err(WasmVmError::TypeError("tan: expected number".into())),
                }
            }
            "log" => {
                let val = args.first().ok_or(WasmVmError::TypeError("log requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.ln())),
                    Value::Int(n) => Ok(Value::Float((*n as f64).ln())),
                    _ => Err(WasmVmError::TypeError("log: expected number".into())),
                }
            }
            "exp" => {
                let val = args.first().ok_or(WasmVmError::TypeError("exp requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.exp())),
                    Value::Int(n) => Ok(Value::Float((*n as f64).exp())),
                    _ => Err(WasmVmError::TypeError("exp: expected number".into())),
                }
            }
            "floor" => {
                let val = args.first().ok_or(WasmVmError::TypeError("floor requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.floor())),
                    Value::Int(n) => Ok(Value::Int(*n)),
                    _ => Err(WasmVmError::TypeError("floor: expected number".into())),
                }
            }
            "ceil" => {
                let val = args.first().ok_or(WasmVmError::TypeError("ceil requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.ceil())),
                    Value::Int(n) => Ok(Value::Int(*n)),
                    _ => Err(WasmVmError::TypeError("ceil: expected number".into())),
                }
            }
            "round" => {
                let val = args.first().ok_or(WasmVmError::TypeError("round requires 1 argument".into()))?;
                match val {
                    Value::Float(f) => Ok(Value::Float(f.round())),
                    Value::Int(n) => Ok(Value::Int(*n)),
                    _ => Err(WasmVmError::TypeError("round: expected number".into())),
                }
            }
            "min" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("min requires 2 arguments".into()));
                }
                Ok(self.min_value(args[0].clone(), args[1].clone()))
            }
            "max" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("max requires 2 arguments".into()));
                }
                Ok(self.max_value(args[0].clone(), args[1].clone()))
            }
            "head" => {
                let arr = args.first().ok_or(WasmVmError::TypeError("head requires 1 argument".into()))?;
                match arr {
                    Value::Array(a) => a.first().cloned().ok_or(WasmVmError::IndexOutOfBounds(0, 0)),
                    _ => Err(WasmVmError::TypeError("head: expected array".into())),
                }
            }
            "tail" => {
                let arr = args.first().ok_or(WasmVmError::TypeError("tail requires 1 argument".into()))?;
                match arr {
                    Value::Array(a) => {
                        if a.is_empty() {
                            Ok(Value::Array(vec![]))
                        } else {
                            Ok(Value::Array(a[1..].to_vec()))
                        }
                    }
                    _ => Err(WasmVmError::TypeError("tail: expected array".into())),
                }
            }
            "init" => {
                let arr = args.first().ok_or(WasmVmError::TypeError("init requires 1 argument".into()))?;
                match arr {
                    Value::Array(a) => {
                        if a.is_empty() {
                            Ok(Value::Array(vec![]))
                        } else {
                            Ok(Value::Array(a[..a.len()-1].to_vec()))
                        }
                    }
                    _ => Err(WasmVmError::TypeError("init: expected array".into())),
                }
            }
            "last" => {
                let arr = args.first().ok_or(WasmVmError::TypeError("last requires 1 argument".into()))?;
                match arr {
                    Value::Array(a) => a.last().cloned().ok_or(WasmVmError::IndexOutOfBounds(-1, 0)),
                    _ => Err(WasmVmError::TypeError("last: expected array".into())),
                }
            }
            "reverse" => {
                let arr = args.first().ok_or(WasmVmError::TypeError("reverse requires 1 argument".into()))?;
                match arr {
                    Value::Array(a) => {
                        let mut reversed = a.clone();
                        reversed.reverse();
                        Ok(Value::Array(reversed))
                    }
                    Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
                    _ => Err(WasmVmError::TypeError("reverse: expected array or string".into())),
                }
            }
            "sort" => {
                let arr = args.first().ok_or(WasmVmError::TypeError("sort requires 1 argument".into()))?;
                match arr {
                    Value::Array(a) => {
                        let mut sorted = a.clone();
                        sorted.sort_by(|a, b| {
                            match (a, b) {
                                (Value::Int(x), Value::Int(y)) => x.cmp(y),
                                (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                (Value::String(x), Value::String(y)) => x.cmp(y),
                                _ => std::cmp::Ordering::Equal,
                            }
                        });
                        Ok(Value::Array(sorted))
                    }
                    _ => Err(WasmVmError::TypeError("sort: expected array".into())),
                }
            }
            "sum" => {
                let arr = args.first().ok_or(WasmVmError::TypeError("sum requires 1 argument".into()))?;
                match arr {
                    Value::Array(a) => {
                        let mut total = Value::Int(0);
                        for item in a {
                            total = self.add_values(total, item.clone())?;
                        }
                        Ok(total)
                    }
                    _ => Err(WasmVmError::TypeError("sum: expected array".into())),
                }
            }
            "product" => {
                let arr = args.first().ok_or(WasmVmError::TypeError("product requires 1 argument".into()))?;
                match arr {
                    Value::Array(a) => {
                        let mut total = Value::Int(1);
                        for item in a {
                            total = self.mul_values(total, item.clone())?;
                        }
                        Ok(total)
                    }
                    _ => Err(WasmVmError::TypeError("product: expected array".into())),
                }
            }
            "split" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("split requires 2 arguments".into()));
                }
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(sep)) => {
                        let parts: Vec<Value> = s.split(sep.as_str())
                            .map(|p| Value::String(p.to_string()))
                            .collect();
                        Ok(Value::Array(parts))
                    }
                    _ => Err(WasmVmError::TypeError("split: expected strings".into())),
                }
            }
            "join" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("join requires 2 arguments".into()));
                }
                match (&args[0], &args[1]) {
                    (Value::Array(a), Value::String(sep)) => {
                        let parts: Vec<String> = a.iter()
                            .map(|v| format!("{}", v))
                            .collect();
                        Ok(Value::String(parts.join(sep)))
                    }
                    _ => Err(WasmVmError::TypeError("join: expected array and string".into())),
                }
            }
            "trim" => {
                let val = args.first().ok_or(WasmVmError::TypeError("trim requires 1 argument".into()))?;
                match val {
                    Value::String(s) => Ok(Value::String(s.trim().to_string())),
                    _ => Err(WasmVmError::TypeError("trim: expected string".into())),
                }
            }
            "upper" => {
                let val = args.first().ok_or(WasmVmError::TypeError("upper requires 1 argument".into()))?;
                match val {
                    Value::String(s) => Ok(Value::String(s.to_uppercase())),
                    _ => Err(WasmVmError::TypeError("upper: expected string".into())),
                }
            }
            "lower" => {
                let val = args.first().ok_or(WasmVmError::TypeError("lower requires 1 argument".into()))?;
                match val {
                    Value::String(s) => Ok(Value::String(s.to_lowercase())),
                    _ => Err(WasmVmError::TypeError("lower: expected string".into())),
                }
            }
            "contains" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("contains requires 2 arguments".into()));
                }
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(sub)) => Ok(Value::Bool(s.contains(sub.as_str()))),
                    (Value::Array(a), item) => Ok(Value::Bool(a.contains(item))),
                    _ => Err(WasmVmError::TypeError("contains: invalid types".into())),
                }
            }
            "replace" => {
                if args.len() < 3 {
                    return Err(WasmVmError::TypeError("replace requires 3 arguments".into()));
                }
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::String(from), Value::String(to)) => {
                        Ok(Value::String(s.replace(from.as_str(), to.as_str())))
                    }
                    _ => Err(WasmVmError::TypeError("replace: expected strings".into())),
                }
            }
            "keys" => {
                let val = args.first().ok_or(WasmVmError::TypeError("keys requires 1 argument".into()))?;
                match val {
                    Value::Map(m) => {
                        let keys: Vec<Value> = m.keys().map(|k| Value::String(k.clone())).collect();
                        Ok(Value::Array(keys))
                    }
                    _ => Err(WasmVmError::TypeError("keys: expected map".into())),
                }
            }
            "values" => {
                let val = args.first().ok_or(WasmVmError::TypeError("values requires 1 argument".into()))?;
                match val {
                    Value::Map(m) => {
                        let values: Vec<Value> = m.values().cloned().collect();
                        Ok(Value::Array(values))
                    }
                    _ => Err(WasmVmError::TypeError("values: expected map".into())),
                }
            }
            "has_key" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("has_key requires 2 arguments".into()));
                }
                match (&args[0], &args[1]) {
                    (Value::Map(m), Value::String(k)) => Ok(Value::Bool(m.contains_key(k))),
                    _ => Err(WasmVmError::TypeError("has_key: expected map and string".into())),
                }
            }
            "range" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("range requires 2 arguments".into()));
                }
                match (&args[0], &args[1]) {
                    (Value::Int(start), Value::Int(end)) => {
                        let range: Vec<Value> = (*start..*end).map(Value::Int).collect();
                        Ok(Value::Array(range))
                    }
                    _ => Err(WasmVmError::TypeError("range: expected integers".into())),
                }
            }
            "take" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("take requires 2 arguments".into()));
                }
                match (&args[0], &args[1]) {
                    (Value::Array(a), Value::Int(n)) => {
                        let n = *n as usize;
                        Ok(Value::Array(a.iter().take(n).cloned().collect()))
                    }
                    _ => Err(WasmVmError::TypeError("take: expected array and int".into())),
                }
            }
            "drop" => {
                if args.len() < 2 {
                    return Err(WasmVmError::TypeError("drop requires 2 arguments".into()));
                }
                match (&args[0], &args[1]) {
                    (Value::Array(a), Value::Int(n)) => {
                        let n = *n as usize;
                        Ok(Value::Array(a.iter().skip(n).cloned().collect()))
                    }
                    _ => Err(WasmVmError::TypeError("drop: expected array and int".into())),
                }
            }
            _ => Err(WasmVmError::UndefinedFunction(name.to_string())),
        }
    }
}
