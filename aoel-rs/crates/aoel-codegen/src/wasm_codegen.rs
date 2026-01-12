//! WebAssembly Code Generator
//!
//! Generates WAT (WebAssembly Text Format) from compiled AOEL functions.
//! The output can be compiled to WASM using tools like wat2wasm.

use std::fmt::Write;

use aoel_ir::{OpCode, ReduceOp, Value};
use aoel_lowering::CompiledFunction;

use crate::error::CodegenResult;

/// WASM 코드 생성기
pub struct WasmCodeGenerator {
    output: String,
    indent: usize,
    label_counter: usize,
}

impl WasmCodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            label_counter: 0,
        }
    }

    fn indent_str(&self) -> String {
        "  ".repeat(self.indent)
    }

    fn writeln(&mut self, s: &str) {
        let _ = writeln!(self.output, "{}{}", self.indent_str(), s);
    }

    fn next_label(&mut self) -> String {
        let label = format!("$L{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    /// 전체 WASM 모듈 생성
    pub fn generate(&mut self, functions: &[CompiledFunction]) -> CodegenResult<String> {
        self.writeln("(module");
        self.indent += 1;

        // Memory 선언
        self.writeln(";; Memory for heap allocation");
        self.writeln("(memory (export \"memory\") 1)");
        self.writeln("");

        // Global variables for memory management
        self.writeln(";; Heap pointer");
        self.writeln("(global $heap_ptr (mut i32) (i32.const 1024))");
        self.writeln("");

        // Import WASI for I/O (optional)
        self.generate_imports();

        // Helper functions
        self.generate_helpers();

        // User functions
        for func in functions {
            self.generate_function(func)?;
        }

        // Export main function if exists
        if functions.iter().any(|f| f.name == "__main__") {
            self.writeln("");
            self.writeln("(export \"main\" (func $__main__))");
        }

        // Start function
        self.writeln("");
        self.writeln("(start $__main__)");

        self.indent -= 1;
        self.writeln(")");

        Ok(self.output.clone())
    }

    fn generate_imports(&mut self) {
        self.writeln(";; WASI imports for I/O");
        self.writeln("(import \"wasi_snapshot_preview1\" \"fd_write\"");
        self.indent += 1;
        self.writeln("(func $fd_write (param i32 i32 i32 i32) (result i32))");
        self.indent -= 1;
        self.writeln(")");
        self.writeln("");
    }

    fn generate_helpers(&mut self) {
        // Memory allocation helper
        self.writeln(";; Simple bump allocator");
        self.writeln("(func $alloc (param $size i32) (result i32)");
        self.indent += 1;
        self.writeln("(local $ptr i32)");
        self.writeln("(local.set $ptr (global.get $heap_ptr))");
        self.writeln("(global.set $heap_ptr (i32.add (global.get $heap_ptr) (local.get $size)))");
        self.writeln("(local.get $ptr)");
        self.indent -= 1;
        self.writeln(")");
        self.writeln("");

        // Print integer helper
        self.writeln(";; Print integer to stdout");
        self.writeln("(func $print_i64 (param $val i64)");
        self.indent += 1;
        self.writeln(";; Simple implementation - write to memory and call fd_write");
        self.writeln("(local $ptr i32)");
        self.writeln("(local $len i32)");
        self.writeln("(local $tmp i64)");
        self.writeln("(local $neg i32)");
        self.writeln("");
        self.writeln(";; For now, just drop the value (full implementation would convert to string)");
        self.writeln("(drop (local.get $val))");
        self.indent -= 1;
        self.writeln(")");
        self.writeln("");
    }

    fn generate_function(&mut self, func: &CompiledFunction) -> CodegenResult<()> {
        self.writeln("");
        self.writeln(&format!(";; Function: {}", func.name));

        // Function signature
        let params = func.params.iter()
            .map(|p| format!("(param ${} i64)", self.sanitize_name(p)))
            .collect::<Vec<_>>()
            .join(" ");

        if params.is_empty() {
            self.writeln(&format!("(func ${} (result i64)", self.sanitize_name(&func.name)));
        } else {
            self.writeln(&format!("(func ${} {} (result i64)", self.sanitize_name(&func.name), params));
        }
        self.indent += 1;

        // Local variables for stack simulation
        self.writeln(";; Stack simulation locals");
        self.writeln("(local $tmp1 i64)");
        self.writeln("(local $tmp2 i64)");
        self.writeln("(local $cond i32)");
        self.writeln("");

        // Generate instructions
        self.generate_instructions(&func.instructions, &func.name, &func.params)?;

        self.indent -= 1;
        self.writeln(")");

        Ok(())
    }

    fn generate_instructions(
        &mut self,
        instructions: &[aoel_ir::Instruction],
        func_name: &str,
        params: &[String],
    ) -> CodegenResult<()> {
        // WASM uses structured control flow, so we need to analyze jumps
        // For simplicity, we'll generate basic blocks with br_table

        let mut i = 0;
        while i < instructions.len() {
            let instr = &instructions[i];

            match &instr.opcode {
                OpCode::Const(value) => {
                    match value {
                        Value::Int(n) => self.writeln(&format!("(i64.const {})", n)),
                        Value::Float(f) => {
                            // Convert float to i64 bits for simplicity
                            let bits = f.to_bits();
                            self.writeln(&format!(";; float {} as i64 bits", f));
                            self.writeln(&format!("(i64.const {})", bits as i64));
                        }
                        Value::Bool(b) => {
                            self.writeln(&format!("(i64.const {})", if *b { 1 } else { 0 }));
                        }
                        _ => {
                            self.writeln("(i64.const 0) ;; unsupported value type");
                        }
                    }
                }
                OpCode::Load(name) => {
                    if params.contains(name) {
                        self.writeln(&format!("(local.get ${})", self.sanitize_name(name)));
                    } else {
                        // For non-parameter variables, we'd need local declarations
                        self.writeln(&format!(";; load {}", name));
                        self.writeln("(i64.const 0) ;; TODO: implement variable storage");
                    }
                }
                OpCode::Store(name) => {
                    if params.contains(name) {
                        self.writeln(&format!("(local.set ${})", self.sanitize_name(name)));
                    } else {
                        self.writeln(&format!(";; store {} (dropped for now)", name));
                        self.writeln("(drop)");
                    }
                }
                OpCode::Add => {
                    self.writeln("(i64.add)");
                }
                OpCode::Sub => {
                    self.writeln("(i64.sub)");
                }
                OpCode::Mul => {
                    self.writeln("(i64.mul)");
                }
                OpCode::Div => {
                    self.writeln("(i64.div_s)");
                }
                OpCode::Mod => {
                    self.writeln("(i64.rem_s)");
                }
                OpCode::Neg => {
                    self.writeln("(i64.const -1)");
                    self.writeln("(i64.mul)");
                }
                OpCode::Lt => {
                    self.writeln("(i64.lt_s)");
                    self.writeln("(i64.extend_i32_s)");
                }
                OpCode::Gt => {
                    self.writeln("(i64.gt_s)");
                    self.writeln("(i64.extend_i32_s)");
                }
                OpCode::Lte => {
                    self.writeln("(i64.le_s)");
                    self.writeln("(i64.extend_i32_s)");
                }
                OpCode::Gte => {
                    self.writeln("(i64.ge_s)");
                    self.writeln("(i64.extend_i32_s)");
                }
                OpCode::Eq => {
                    self.writeln("(i64.eq)");
                    self.writeln("(i64.extend_i32_s)");
                }
                OpCode::Neq => {
                    self.writeln("(i64.ne)");
                    self.writeln("(i64.extend_i32_s)");
                }
                OpCode::And => {
                    self.writeln("(i64.and)");
                }
                OpCode::Or => {
                    self.writeln("(i64.or)");
                }
                OpCode::Not => {
                    self.writeln("(i64.eqz)");
                    self.writeln("(i64.extend_i32_s)");
                }
                OpCode::Jump(offset) => {
                    let target = (i as i32 + offset + 1) as usize;
                    self.writeln(&format!(";; jump to instruction {}", target));
                    // WASM doesn't support arbitrary jumps - need structured control flow
                    // This is a simplified implementation
                }
                OpCode::JumpIf(offset) => {
                    let target = (i as i32 + offset + 1) as usize;
                    self.writeln(&format!(";; jump if true to instruction {}", target));
                    self.writeln("(i32.wrap_i64)");
                    self.writeln("(if");
                    self.indent += 1;
                    self.writeln("(then");
                    self.indent += 1;
                    self.writeln(&format!(";; branch to {}", target));
                    self.indent -= 1;
                    self.writeln(")");
                    self.indent -= 1;
                    self.writeln(")");
                }
                OpCode::JumpIfNot(offset) => {
                    let target = (i as i32 + offset + 1) as usize;
                    self.writeln(&format!(";; jump if false to instruction {}", target));
                    self.writeln("(i32.wrap_i64)");
                    self.writeln("(i32.eqz)");
                    self.writeln("(if");
                    self.indent += 1;
                    self.writeln("(then");
                    self.indent += 1;
                    self.writeln(&format!(";; branch to {}", target));
                    self.indent -= 1;
                    self.writeln(")");
                    self.indent -= 1;
                    self.writeln(")");
                }
                OpCode::Call(name, arg_count) => {
                    self.writeln(&format!(";; call {} with {} args", name, arg_count));
                    self.writeln(&format!("(call ${})", self.sanitize_name(name)));
                }
                OpCode::SelfCall(arg_count) => {
                    self.writeln(&format!(";; self-recursive call with {} args", arg_count));
                    self.writeln(&format!("(call ${})", self.sanitize_name(func_name)));
                }
                OpCode::TailSelfCall(arg_count) => {
                    // WASM tail calls are a proposal, use regular call for now
                    self.writeln(&format!(";; tail call with {} args (using regular call)", arg_count));
                    self.writeln(&format!("(call ${})", self.sanitize_name(func_name)));
                }
                OpCode::Return => {
                    self.writeln("(return)");
                }
                OpCode::Pop => {
                    self.writeln("(drop)");
                }
                OpCode::Dup => {
                    self.writeln("(local.set $tmp1)");
                    self.writeln("(local.get $tmp1)");
                    self.writeln("(local.get $tmp1)");
                }
                OpCode::Nop => {
                    self.writeln("(nop)");
                }
                OpCode::MakeArray(count) => {
                    self.writeln(&format!(";; make array with {} elements", count));
                    // Simplified: just return count for now
                    for _ in 0..*count {
                        self.writeln("(drop)");
                    }
                    self.writeln(&format!("(i64.const {}) ;; array placeholder", count));
                }
                OpCode::Len => {
                    self.writeln(";; array length (simplified)");
                    // Already have the length as the array placeholder
                }
                OpCode::Index => {
                    self.writeln(";; array index (simplified)");
                    self.writeln("(drop) ;; drop index");
                    self.writeln("(drop) ;; drop array");
                    self.writeln("(i64.const 0) ;; placeholder result");
                }
                OpCode::Reduce(op, init) => {
                    let init_val = match init {
                        Value::Int(n) => *n,
                        _ => 0,
                    };
                    self.writeln(&format!(";; reduce {:?} with init {}", op, init_val));
                    match op {
                        ReduceOp::Sum => {
                            self.writeln(&format!("(i64.const {})", init_val));
                            self.writeln("(i64.add)");
                        }
                        ReduceOp::Product => {
                            self.writeln(&format!("(i64.const {})", init_val));
                            self.writeln("(i64.mul)");
                        }
                        _ => {
                            self.writeln("(drop)");
                            self.writeln(&format!("(i64.const {})", init_val));
                        }
                    }
                }
                OpCode::CallBuiltin(name, _arg_count) => {
                    self.writeln(&format!(";; builtin: {}", name));
                    match name.to_uppercase().as_str() {
                        "PRINT" | "PRINTLN" => {
                            self.writeln("(call $print_i64)");
                            self.writeln("(i64.const 0) ;; void");
                        }
                        "ABS" => {
                            // abs(x) = x < 0 ? -x : x
                            self.writeln("(local.set $tmp1)");
                            self.writeln("(local.get $tmp1)");
                            self.writeln("(i64.const 0)");
                            self.writeln("(i64.lt_s)");
                            self.writeln("(if (result i64)");
                            self.indent += 1;
                            self.writeln("(then");
                            self.indent += 1;
                            self.writeln("(i64.const 0)");
                            self.writeln("(local.get $tmp1)");
                            self.writeln("(i64.sub)");
                            self.indent -= 1;
                            self.writeln(")");
                            self.writeln("(else");
                            self.indent += 1;
                            self.writeln("(local.get $tmp1)");
                            self.indent -= 1;
                            self.writeln(")");
                            self.indent -= 1;
                            self.writeln(")");
                        }
                        "MIN" => {
                            self.writeln("(local.set $tmp2)");
                            self.writeln("(local.set $tmp1)");
                            self.writeln("(local.get $tmp1)");
                            self.writeln("(local.get $tmp2)");
                            self.writeln("(local.get $tmp1)");
                            self.writeln("(local.get $tmp2)");
                            self.writeln("(i64.lt_s)");
                            self.writeln("(select)");
                        }
                        "MAX" => {
                            self.writeln("(local.set $tmp2)");
                            self.writeln("(local.set $tmp1)");
                            self.writeln("(local.get $tmp1)");
                            self.writeln("(local.get $tmp2)");
                            self.writeln("(local.get $tmp1)");
                            self.writeln("(local.get $tmp2)");
                            self.writeln("(i64.gt_s)");
                            self.writeln("(select)");
                        }
                        _ => {
                            self.writeln(&format!(";; unsupported builtin: {}", name));
                            self.writeln("(i64.const 0)");
                        }
                    }
                }
                _ => {
                    self.writeln(&format!(";; unsupported: {:?}", instr.opcode));
                }
            }

            i += 1;
        }

        // Default return
        self.writeln(";; default return");
        self.writeln("(i64.const 0)");

        Ok(())
    }

    fn sanitize_name(&self, name: &str) -> String {
        name.replace('-', "_").replace('.', "_")
    }
}

impl Default for WasmCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 편의 함수: 컴파일된 함수들을 WAT 코드로 변환
pub fn generate_wat(functions: &[CompiledFunction]) -> CodegenResult<String> {
    let mut gen = WasmCodeGenerator::new();
    gen.generate(functions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoel_ir::{Instruction, OpCode, Value};

    #[test]
    fn test_simple_add() {
        let func = CompiledFunction {
            name: "__main__".to_string(),
            params: vec![],
            instructions: vec![
                Instruction::new(OpCode::Const(Value::Int(2))),
                Instruction::new(OpCode::Const(Value::Int(3))),
                Instruction::new(OpCode::Add),
                Instruction::new(OpCode::Return),
            ],
        };

        let wat = generate_wat(&[func]).unwrap();
        assert!(wat.contains("(module"));
        assert!(wat.contains("(func $__main__"));
        assert!(wat.contains("i64.const 2"));
        assert!(wat.contains("i64.const 3"));
        assert!(wat.contains("i64.add"));
    }

    #[test]
    fn test_function_with_params() {
        let func = CompiledFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                Instruction::new(OpCode::Load("a".to_string())),
                Instruction::new(OpCode::Load("b".to_string())),
                Instruction::new(OpCode::Add),
                Instruction::new(OpCode::Return),
            ],
        };

        let wat = generate_wat(&[func]).unwrap();
        assert!(wat.contains("(param $a i64)"));
        assert!(wat.contains("(param $b i64)"));
        assert!(wat.contains("local.get $a"));
        assert!(wat.contains("local.get $b"));
    }

    #[test]
    fn test_comparison() {
        let func = CompiledFunction {
            name: "__main__".to_string(),
            params: vec![],
            instructions: vec![
                Instruction::new(OpCode::Const(Value::Int(5))),
                Instruction::new(OpCode::Const(Value::Int(3))),
                Instruction::new(OpCode::Lt),
                Instruction::new(OpCode::Return),
            ],
        };

        let wat = generate_wat(&[func]).unwrap();
        assert!(wat.contains("i64.lt_s"));
    }

    #[test]
    fn test_recursive_call() {
        let func = CompiledFunction {
            name: "fact".to_string(),
            params: vec!["n".to_string()],
            instructions: vec![
                Instruction::new(OpCode::Load("n".to_string())),
                Instruction::new(OpCode::Const(Value::Int(1))),
                Instruction::new(OpCode::Sub),
                Instruction::new(OpCode::SelfCall(1)),
                Instruction::new(OpCode::Return),
            ],
        };

        let wat = generate_wat(&[func]).unwrap();
        assert!(wat.contains("(call $fact)"));
    }
}
