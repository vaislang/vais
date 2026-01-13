//! High-Performance Vais VM using NaN-boxing and Trampoline Execution
//!
//! This VM uses:
//! - NaN-boxing: All values fit in 64 bits (Copy semantics)
//! - Type specialization: Fast path for Int operations
//! - Trampoline: Non-recursive execution with manual call stack
//! - Inline operations: No bounds checking in hot path

use std::collections::HashMap;
use std::sync::Arc;
use vais_ir::{Instruction, OpCode, NanBoxedValue};
use vais_lowering::CompiledFunction;
use crate::error::{RuntimeError, RuntimeResult};

const MAX_RECURSION_DEPTH: usize = 10000;
const INITIAL_STACK_CAPACITY: usize = 4096;
const INITIAL_LOCALS_CAPACITY: usize = 1024;
const INITIAL_CALL_STACK_CAPACITY: usize = 256;

/// Call frame for trampoline execution
#[derive(Clone)]
struct CallFrame {
    /// Function being executed
    func: Arc<CompiledFunction>,
    /// Instruction pointer
    ip: usize,
    /// Base index in locals array
    locals_base: usize,
    /// Base index in value stack (for cleanup)
    stack_base: usize,
}

/// High-performance VM with NaN-boxing and trampoline execution
pub struct FastVm {
    /// Value stack (NaN-boxed for O(1) operations)
    stack: Vec<NanBoxedValue>,
    /// Local variables (indexed by slot)
    locals: Vec<NanBoxedValue>,
    /// Call stack for trampoline execution
    call_stack: Vec<CallFrame>,
    /// Current locals base index
    locals_base: usize,
    /// Compiled functions
    functions: Arc<HashMap<String, Arc<CompiledFunction>>>,
    /// Current function reference (for SelfCall)
    current_func_ref: Option<Arc<CompiledFunction>>,
    /// Current function name (for SelfCall)
    current_function: Option<String>,
}

impl FastVm {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(INITIAL_STACK_CAPACITY),
            locals: Vec::with_capacity(INITIAL_LOCALS_CAPACITY),
            call_stack: Vec::with_capacity(INITIAL_CALL_STACK_CAPACITY),
            locals_base: 0,
            functions: Arc::new(HashMap::new()),
            current_func_ref: None,
            current_function: None,
        }
    }

    /// Inline stack push
    #[inline(always)]
    fn push(&mut self, value: NanBoxedValue) {
        self.stack.push(value);
    }

    /// Inline stack pop (unchecked)
    #[inline(always)]
    fn pop_unchecked(&mut self) -> NanBoxedValue {
        unsafe { self.stack.pop().unwrap_unchecked() }
    }

    /// Inline stack top (unchecked)
    #[inline(always)]
    fn top_unchecked(&self) -> NanBoxedValue {
        unsafe { *self.stack.last().unwrap_unchecked() }
    }

    pub fn load_functions(&mut self, funcs: Vec<CompiledFunction>) {
        let mut map = HashMap::new();
        for f in funcs {
            map.insert(f.name.clone(), Arc::new(f));
        }
        self.functions = Arc::new(map);
    }

    /// Public entry point - calls a function by name
    pub fn call_function(&mut self, name: &str, args: Vec<NanBoxedValue>) -> RuntimeResult<NanBoxedValue> {
        let func = self.functions.get(name)
            .ok_or_else(|| RuntimeError::UndefinedFunction(name.to_string()))?
            .clone();

        // Setup initial frame
        let local_count = func.local_count as usize;
        self.locals.resize(local_count, NanBoxedValue::void());

        // Bind arguments
        for (i, arg) in args.into_iter().enumerate() {
            if i < func.params.len() {
                self.locals[i] = arg;
            }
        }

        self.current_func_ref = Some(func.clone());
        self.current_function = Some(name.to_string());
        self.locals_base = 0;

        // Execute with trampoline
        self.run_trampoline(func)
    }

    /// Trampoline execution loop - no Rust recursion for function calls
    ///
    /// This eliminates the overhead of Rust stack frame creation/destruction
    /// for recursive calls, providing significant speedup for deep recursion.
    #[inline(never)]
    fn run_trampoline(&mut self, initial_func: Arc<CompiledFunction>) -> RuntimeResult<NanBoxedValue> {
        let mut func = initial_func;
        let mut ip: usize = 0;

        'outer: loop {
            let len = func.instructions.len();

            while ip < len {
                // Use a reference and match by pattern to avoid cloning
                match unsafe { &func.instructions.get_unchecked(ip).opcode } {
                    // === Hot path: Stack operations ===
                    OpCode::Const(value) => {
                        self.push(self.convert_value(value));
                    }

                    OpCode::LoadLocal(idx) => {
                        let idx = self.locals_base.wrapping_add(*idx as usize);
                        let value = unsafe { *self.locals.get_unchecked(idx) };
                        self.push(value);
                    }

                    OpCode::StoreLocal(idx) => {
                        let idx = self.locals_base.wrapping_add(*idx as usize);
                        let value = self.pop_unchecked();
                        unsafe { *self.locals.get_unchecked_mut(idx) = value };
                    }

                    // === Hot path: Integer arithmetic ===
                    OpCode::Add => {
                        let b = self.pop_unchecked();
                        let a = self.pop_unchecked();

                        if a.is_int() && b.is_int() {
                            self.push(NanBoxedValue::int(
                                a.as_int_unchecked().wrapping_add(b.as_int_unchecked())
                            ));
                        } else {
                            let fa = a.as_float().unwrap_or(0.0);
                            let fb = b.as_float().unwrap_or(0.0);
                            self.push(NanBoxedValue::float(fa + fb));
                        }
                    }

                    OpCode::Sub => {
                        let b = self.pop_unchecked();
                        let a = self.pop_unchecked();

                        if a.is_int() && b.is_int() {
                            self.push(NanBoxedValue::int(
                                a.as_int_unchecked().wrapping_sub(b.as_int_unchecked())
                            ));
                        } else {
                            let fa = a.as_float().unwrap_or(0.0);
                            let fb = b.as_float().unwrap_or(0.0);
                            self.push(NanBoxedValue::float(fa - fb));
                        }
                    }

                    OpCode::Mul => {
                        let b = self.pop_unchecked();
                        let a = self.pop_unchecked();

                        if a.is_int() && b.is_int() {
                            self.push(NanBoxedValue::int(
                                a.as_int_unchecked().wrapping_mul(b.as_int_unchecked())
                            ));
                        } else {
                            let fa = a.as_float().unwrap_or(0.0);
                            let fb = b.as_float().unwrap_or(0.0);
                            self.push(NanBoxedValue::float(fa * fb));
                        }
                    }

                    // === Hot path: Integer comparison ===
                    OpCode::Lt => {
                        let b = self.pop_unchecked();
                        let a = self.pop_unchecked();
                        let result = if a.is_int() && b.is_int() {
                            a.as_int_unchecked() < b.as_int_unchecked()
                        } else {
                            a.as_float().unwrap_or(0.0) < b.as_float().unwrap_or(0.0)
                        };
                        self.push(NanBoxedValue::bool(result));
                    }

                    OpCode::Lte => {
                        let b = self.pop_unchecked();
                        let a = self.pop_unchecked();
                        let result = if a.is_int() && b.is_int() {
                            a.as_int_unchecked() <= b.as_int_unchecked()
                        } else {
                            a.as_float().unwrap_or(0.0) <= b.as_float().unwrap_or(0.0)
                        };
                        self.push(NanBoxedValue::bool(result));
                    }

                    OpCode::Gt => {
                        let b = self.pop_unchecked();
                        let a = self.pop_unchecked();
                        let result = if a.is_int() && b.is_int() {
                            a.as_int_unchecked() > b.as_int_unchecked()
                        } else {
                            a.as_float().unwrap_or(0.0) > b.as_float().unwrap_or(0.0)
                        };
                        self.push(NanBoxedValue::bool(result));
                    }

                    OpCode::Eq => {
                        let b = self.pop_unchecked();
                        let a = self.pop_unchecked();
                        self.push(NanBoxedValue::bool(a == b));
                    }

                    // === Control flow (relative offset) ===
                    OpCode::Jump(offset) => {
                        ip = ((ip as i64) + (*offset as i64) + 1) as usize;
                        continue;
                    }

                    OpCode::JumpIfNot(offset) => {
                        let cond = self.pop_unchecked();
                        if !cond.is_truthy() {
                            ip = ((ip as i64) + (*offset as i64) + 1) as usize;
                            continue;
                        }
                    }

                    OpCode::JumpIf(offset) => {
                        let cond = self.pop_unchecked();
                        if cond.is_truthy() {
                            ip = ((ip as i64) + (*offset as i64) + 1) as usize;
                            continue;
                        }
                    }

                    // === Function calls (trampoline) ===
                    OpCode::Call(name, arg_count) => {
                        // Extract name to avoid borrow issue
                        let name = name.clone();
                        let arg_count = *arg_count;

                        let called_func = self.functions.get(&name)
                            .ok_or_else(|| RuntimeError::UndefinedFunction(name.to_string()))?
                            .clone();

                        // Save current frame
                        self.call_stack.push(CallFrame {
                            func: func.clone(),
                            ip: ip + 1, // Return to next instruction
                            locals_base: self.locals_base,
                            stack_base: self.stack.len() - arg_count,
                        });

                        if self.call_stack.len() > MAX_RECURSION_DEPTH {
                            return Err(RuntimeError::MaxRecursionDepth);
                        }

                        // Setup new frame
                        let new_locals_base = self.locals.len();
                        let local_count = called_func.local_count as usize;
                        self.locals.resize(new_locals_base + local_count, NanBoxedValue::void());

                        // Pop and bind arguments
                        let param_count = called_func.params.len();
                        for i in (0..arg_count).rev() {
                            let arg = self.pop_unchecked();
                            if i < param_count {
                                self.locals[new_locals_base + i] = arg;
                            }
                        }

                        self.locals_base = new_locals_base;
                        self.current_func_ref = Some(called_func.clone());
                        func = called_func;
                        ip = 0;
                        continue 'outer;
                    }

                    OpCode::SelfCall(arg_count) => {
                        let arg_count = *arg_count;

                        // Use current function (faster than hashmap lookup)
                        let called_func = self.current_func_ref.as_ref()
                            .ok_or_else(|| RuntimeError::Internal("SelfCall without function".to_string()))?
                            .clone();

                        // Save current frame
                        self.call_stack.push(CallFrame {
                            func: func.clone(),
                            ip: ip + 1,
                            locals_base: self.locals_base,
                            stack_base: self.stack.len() - arg_count,
                        });

                        if self.call_stack.len() > MAX_RECURSION_DEPTH {
                            return Err(RuntimeError::MaxRecursionDepth);
                        }

                        // Setup new frame
                        let new_locals_base = self.locals.len();
                        let local_count = called_func.local_count as usize;
                        self.locals.resize(new_locals_base + local_count, NanBoxedValue::void());

                        // Pop and bind arguments
                        let param_count = called_func.params.len();
                        for i in (0..arg_count).rev() {
                            let arg = self.pop_unchecked();
                            if i < param_count {
                                self.locals[new_locals_base + i] = arg;
                            }
                        }

                        self.locals_base = new_locals_base;
                        func = called_func;
                        ip = 0;
                        continue 'outer;
                    }

                    OpCode::Return => {
                        // Get return value (top of stack)
                        let return_value = if !self.stack.is_empty() {
                            self.pop_unchecked()
                        } else {
                            NanBoxedValue::void()
                        };

                        // Pop call frame
                        if let Some(frame) = self.call_stack.pop() {
                            // Restore state
                            self.locals.truncate(self.locals_base);
                            self.locals_base = frame.locals_base;
                            self.stack.truncate(frame.stack_base);

                            // Push return value
                            self.push(return_value);

                            // Resume caller
                            func = frame.func;
                            self.current_func_ref = Some(func.clone());
                            ip = frame.ip;
                            continue 'outer;
                        } else {
                            // Top-level return
                            return Ok(return_value);
                        }
                    }

                    // === Stack operations ===
                    OpCode::Pop => {
                        self.pop_unchecked();
                    }

                    OpCode::Dup => {
                        let val = self.top_unchecked();
                        self.push(val);
                    }

                    // === Other operations ===
                    opcode => {
                        self.execute_slow_path(opcode)?;
                    }
                }

                ip = ip.wrapping_add(1);
            }

            // Implicit return at end of function
            let return_value = if !self.stack.is_empty() {
                self.pop_unchecked()
            } else {
                NanBoxedValue::void()
            };

            if let Some(frame) = self.call_stack.pop() {
                self.locals.truncate(self.locals_base);
                self.locals_base = frame.locals_base;
                self.stack.truncate(frame.stack_base);
                self.push(return_value);
                func = frame.func;
                self.current_func_ref = Some(func.clone());
                ip = frame.ip;
            } else {
                return Ok(return_value);
            }
        }
    }

    /// Slow path for less common operations
    #[cold]
    fn execute_slow_path(&mut self, opcode: &OpCode) -> RuntimeResult<()> {
        match opcode {
            OpCode::Div => {
                let b = self.pop_unchecked();
                let a = self.pop_unchecked();

                if a.is_int() && b.is_int() {
                    let bv = b.as_int_unchecked();
                    if bv == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.push(NanBoxedValue::int(a.as_int_unchecked() / bv));
                } else {
                    let fb = b.as_float().unwrap_or(0.0);
                    if fb == 0.0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.push(NanBoxedValue::float(a.as_float().unwrap_or(0.0) / fb));
                }
            }

            OpCode::Mod => {
                let b = self.pop_unchecked();
                let a = self.pop_unchecked();

                if a.is_int() && b.is_int() {
                    let bv = b.as_int_unchecked();
                    if bv == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.push(NanBoxedValue::int(a.as_int_unchecked() % bv));
                } else {
                    return Err(RuntimeError::TypeError("Mod requires integers".to_string()));
                }
            }

            OpCode::Neg => {
                let a = self.pop_unchecked();
                if a.is_int() {
                    self.push(NanBoxedValue::int(-a.as_int_unchecked()));
                } else if a.is_float() {
                    self.push(NanBoxedValue::float(-a.as_float_unchecked()));
                } else {
                    return Err(RuntimeError::TypeError("Cannot negate".to_string()));
                }
            }

            OpCode::Not => {
                let a = self.pop_unchecked();
                self.push(NanBoxedValue::bool(!a.is_truthy()));
            }

            OpCode::Neq => {
                let b = self.pop_unchecked();
                let a = self.pop_unchecked();
                self.push(NanBoxedValue::bool(a != b));
            }

            OpCode::Gte => {
                let b = self.pop_unchecked();
                let a = self.pop_unchecked();

                let result = if a.is_int() && b.is_int() {
                    a.as_int_unchecked() >= b.as_int_unchecked()
                } else {
                    a.as_float().unwrap_or(0.0) >= b.as_float().unwrap_or(0.0)
                };
                self.push(NanBoxedValue::bool(result));
            }

            OpCode::And => {
                let b = self.pop_unchecked();
                let a = self.pop_unchecked();
                self.push(NanBoxedValue::bool(a.is_truthy() && b.is_truthy()));
            }

            OpCode::Or => {
                let b = self.pop_unchecked();
                let a = self.pop_unchecked();
                self.push(NanBoxedValue::bool(a.is_truthy() || b.is_truthy()));
            }

            OpCode::MakeArray(size) => {
                let mut items = Vec::with_capacity(*size);
                for _ in 0..*size {
                    items.push(self.pop_unchecked());
                }
                items.reverse();
                self.push(NanBoxedValue::array(items));
            }

            OpCode::Index => {
                let idx = self.pop_unchecked();
                let base = self.pop_unchecked();

                if let Some(arr) = base.as_array() {
                    let i = idx.as_int().unwrap_or(0) as usize;
                    let value = arr.get(i).copied().unwrap_or(NanBoxedValue::void());
                    self.push(value);
                } else {
                    self.push(NanBoxedValue::void());
                }
            }

            OpCode::Load(_name) => {
                // Named variable load (slow path) - push void for now
                self.push(NanBoxedValue::void());
            }

            OpCode::Store(_name) => {
                // Named variable store (slow path) - just pop
                self.pop_unchecked();
            }

            _ => {
                // Unsupported operation in fast VM - no-op
            }
        }

        Ok(())
    }

    /// Convert standard Value to NanBoxedValue
    fn convert_value(&self, value: &vais_ir::Value) -> NanBoxedValue {
        match value {
            vais_ir::Value::Void => NanBoxedValue::void(),
            vais_ir::Value::Bool(b) => NanBoxedValue::bool(*b),
            vais_ir::Value::Int(n) => NanBoxedValue::int(*n),
            vais_ir::Value::Float(f) => NanBoxedValue::float(*f),
            vais_ir::Value::String(s) => NanBoxedValue::string(s.clone()),
            vais_ir::Value::Array(arr) => {
                let converted: Vec<_> = arr.iter().map(|v| self.convert_value(v)).collect();
                NanBoxedValue::array(converted)
            }
            vais_ir::Value::Map(m) => {
                let converted: HashMap<_, _> = m.iter()
                    .map(|(k, v)| (k.clone(), self.convert_value(v)))
                    .collect();
                NanBoxedValue::map(converted)
            }
            _ => NanBoxedValue::void(),
        }
    }
}

impl Default for FastVm {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute function using FastVm
pub fn execute_fast(
    functions: Vec<CompiledFunction>,
    func_name: &str,
    args: Vec<vais_ir::Value>,
) -> RuntimeResult<NanBoxedValue> {
    let mut vm = FastVm::new();
    vm.load_functions(functions);

    // Convert args
    let converted_args: Vec<_> = args.iter().map(|v| match v {
        vais_ir::Value::Int(n) => NanBoxedValue::int(*n),
        vais_ir::Value::Float(f) => NanBoxedValue::float(*f),
        vais_ir::Value::Bool(b) => NanBoxedValue::bool(*b),
        vais_ir::Value::String(s) => NanBoxedValue::string(s.clone()),
        _ => NanBoxedValue::void(),
    }).collect();

    vm.call_function(func_name, converted_args)
}
