//! LLVM IR Code Generator
//!
//! Generates LLVM IR text format from compiled Vais functions.
//! The output can be compiled using llc or clang.

use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use vais_ir::{OpCode, Value};
use vais_lowering::CompiledFunction;

use crate::error::CodegenResult;

/// LLVM IR 코드 생성기
pub struct LlvmCodeGenerator {
    output: String,
    temp_counter: usize,
    block_counter: usize,
}

impl LlvmCodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            temp_counter: 0,
            block_counter: 0,
        }
    }

    fn writeln(&mut self, s: &str) {
        let _ = writeln!(self.output, "{}", s);
    }

    fn next_temp(&mut self) -> String {
        let temp = format!("%t{}", self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    fn next_block(&mut self) -> String {
        let block = format!("block{}", self.block_counter);
        self.block_counter += 1;
        block
    }

    /// 전체 LLVM IR 모듈 생성
    pub fn generate(&mut self, functions: &[CompiledFunction]) -> CodegenResult<String> {
        self.generate_header();
        self.generate_runtime_decls();

        for func in functions {
            self.generate_function(func)?;
        }

        self.generate_main_wrapper(functions);

        Ok(self.output.clone())
    }

    fn generate_header(&mut self) {
        self.writeln("; Vais Generated LLVM IR");
        self.writeln("; Compile with: clang -O2 output.ll -o output");
        self.writeln("");
        self.writeln("; Target triple for common platforms");
        self.writeln("target datalayout = \"e-m:o-i64:64-i128:128-n32:64-S128\"");
        self.writeln("");

        // Value type definition (tagged union)
        self.writeln("; Vais Value type (tagged union)");
        self.writeln("; Tag: 0=void, 1=bool, 2=int, 3=float, 4=string, 5=array");
        self.writeln("%Value = type { i8, i64 }");
        self.writeln("");

        // String constants
        self.writeln("; Format strings");
        self.writeln("@.str.int = private unnamed_addr constant [5 x i8] c\"%lld\\00\"");
        self.writeln("@.str.float = private unnamed_addr constant [3 x i8] c\"%g\\00\"");
        self.writeln("@.str.true = private unnamed_addr constant [5 x i8] c\"true\\00\"");
        self.writeln("@.str.false = private unnamed_addr constant [6 x i8] c\"false\\00\"");
        self.writeln("@.str.newline = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"");
        self.writeln("");
    }

    fn generate_runtime_decls(&mut self) {
        self.writeln("; External declarations");
        self.writeln("declare i32 @printf(i8*, ...)");
        self.writeln("declare i8* @malloc(i64)");
        self.writeln("declare void @free(i8*)");
        self.writeln("declare double @sqrt(double)");
        self.writeln("declare double @sin(double)");
        self.writeln("declare double @cos(double)");
        self.writeln("declare double @tan(double)");
        self.writeln("declare double @log(double)");
        self.writeln("declare double @pow(double, double)");
        self.writeln("declare double @fabs(double)");
        self.writeln("declare double @floor(double)");
        self.writeln("declare double @ceil(double)");
        self.writeln("");

        // Helper functions
        self.writeln("; Helper: create int value");
        self.writeln("define %Value @val_int(i64 %v) {");
        self.writeln("  %1 = insertvalue %Value { i8 2, i64 undef }, i64 %v, 1");
        self.writeln("  ret %Value %1");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Helper: create float value (as i64 bits)");
        self.writeln("define %Value @val_float(double %v) {");
        self.writeln("  %1 = bitcast double %v to i64");
        self.writeln("  %2 = insertvalue %Value { i8 3, i64 undef }, i64 %1, 1");
        self.writeln("  ret %Value %2");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Helper: create bool value");
        self.writeln("define %Value @val_bool(i1 %v) {");
        self.writeln("  %1 = zext i1 %v to i64");
        self.writeln("  %2 = insertvalue %Value { i8 1, i64 undef }, i64 %1, 1");
        self.writeln("  ret %Value %2");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Helper: create void value");
        self.writeln("define %Value @val_void() {");
        self.writeln("  ret %Value { i8 0, i64 0 }");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Helper: extract int from value");
        self.writeln("define i64 @get_int(%Value %v) {");
        self.writeln("  %1 = extractvalue %Value %v, 1");
        self.writeln("  ret i64 %1");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Helper: extract float from value");
        self.writeln("define double @get_float(%Value %v) {");
        self.writeln("  %1 = extractvalue %Value %v, 1");
        self.writeln("  %2 = bitcast i64 %1 to double");
        self.writeln("  ret double %2");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Helper: extract bool from value");
        self.writeln("define i1 @get_bool(%Value %v) {");
        self.writeln("  %1 = extractvalue %Value %v, 1");
        self.writeln("  %2 = trunc i64 %1 to i1");
        self.writeln("  ret i1 %2");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Helper: get value type tag");
        self.writeln("define i8 @get_type(%Value %v) {");
        self.writeln("  %1 = extractvalue %Value %v, 0");
        self.writeln("  ret i8 %1");
        self.writeln("}");
        self.writeln("");

        // Arithmetic operations
        self.writeln("; Arithmetic: add");
        self.writeln("define %Value @val_add(%Value %a, %Value %b) {");
        self.writeln("  %ta = call i8 @get_type(%Value %a)");
        self.writeln("  %tb = call i8 @get_type(%Value %b)");
        self.writeln("  %is_int_a = icmp eq i8 %ta, 2");
        self.writeln("  %is_int_b = icmp eq i8 %tb, 2");
        self.writeln("  %both_int = and i1 %is_int_a, %is_int_b");
        self.writeln("  br i1 %both_int, label %int_add, label %float_add");
        self.writeln("int_add:");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %ir = add i64 %ia, %ib");
        self.writeln("  %rv1 = call %Value @val_int(i64 %ir)");
        self.writeln("  ret %Value %rv1");
        self.writeln("float_add:");
        self.writeln("  %fa = call double @get_float(%Value %a)");
        self.writeln("  %fb = call double @get_float(%Value %b)");
        self.writeln("  %fr = fadd double %fa, %fb");
        self.writeln("  %rv2 = call %Value @val_float(double %fr)");
        self.writeln("  ret %Value %rv2");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Arithmetic: sub");
        self.writeln("define %Value @val_sub(%Value %a, %Value %b) {");
        self.writeln("  %ta = call i8 @get_type(%Value %a)");
        self.writeln("  %tb = call i8 @get_type(%Value %b)");
        self.writeln("  %is_int_a = icmp eq i8 %ta, 2");
        self.writeln("  %is_int_b = icmp eq i8 %tb, 2");
        self.writeln("  %both_int = and i1 %is_int_a, %is_int_b");
        self.writeln("  br i1 %both_int, label %int_sub, label %float_sub");
        self.writeln("int_sub:");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %ir = sub i64 %ia, %ib");
        self.writeln("  %rv1 = call %Value @val_int(i64 %ir)");
        self.writeln("  ret %Value %rv1");
        self.writeln("float_sub:");
        self.writeln("  %fa = call double @get_float(%Value %a)");
        self.writeln("  %fb = call double @get_float(%Value %b)");
        self.writeln("  %fr = fsub double %fa, %fb");
        self.writeln("  %rv2 = call %Value @val_float(double %fr)");
        self.writeln("  ret %Value %rv2");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Arithmetic: mul");
        self.writeln("define %Value @val_mul(%Value %a, %Value %b) {");
        self.writeln("  %ta = call i8 @get_type(%Value %a)");
        self.writeln("  %tb = call i8 @get_type(%Value %b)");
        self.writeln("  %is_int_a = icmp eq i8 %ta, 2");
        self.writeln("  %is_int_b = icmp eq i8 %tb, 2");
        self.writeln("  %both_int = and i1 %is_int_a, %is_int_b");
        self.writeln("  br i1 %both_int, label %int_mul, label %float_mul");
        self.writeln("int_mul:");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %ir = mul i64 %ia, %ib");
        self.writeln("  %rv1 = call %Value @val_int(i64 %ir)");
        self.writeln("  ret %Value %rv1");
        self.writeln("float_mul:");
        self.writeln("  %fa = call double @get_float(%Value %a)");
        self.writeln("  %fb = call double @get_float(%Value %b)");
        self.writeln("  %fr = fmul double %fa, %fb");
        self.writeln("  %rv2 = call %Value @val_float(double %fr)");
        self.writeln("  ret %Value %rv2");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Arithmetic: div");
        self.writeln("define %Value @val_div(%Value %a, %Value %b) {");
        self.writeln("  %ta = call i8 @get_type(%Value %a)");
        self.writeln("  %tb = call i8 @get_type(%Value %b)");
        self.writeln("  %is_int_a = icmp eq i8 %ta, 2");
        self.writeln("  %is_int_b = icmp eq i8 %tb, 2");
        self.writeln("  %both_int = and i1 %is_int_a, %is_int_b");
        self.writeln("  br i1 %both_int, label %int_div, label %float_div");
        self.writeln("int_div:");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %ir = sdiv i64 %ia, %ib");
        self.writeln("  %rv1 = call %Value @val_int(i64 %ir)");
        self.writeln("  ret %Value %rv1");
        self.writeln("float_div:");
        self.writeln("  %fa = call double @get_float(%Value %a)");
        self.writeln("  %fb = call double @get_float(%Value %b)");
        self.writeln("  %fr = fdiv double %fa, %fb");
        self.writeln("  %rv2 = call %Value @val_float(double %fr)");
        self.writeln("  ret %Value %rv2");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Arithmetic: mod");
        self.writeln("define %Value @val_mod(%Value %a, %Value %b) {");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %ir = srem i64 %ia, %ib");
        self.writeln("  %rv = call %Value @val_int(i64 %ir)");
        self.writeln("  ret %Value %rv");
        self.writeln("}");
        self.writeln("");

        // Comparison operations
        self.writeln("; Comparison: lt");
        self.writeln("define %Value @val_lt(%Value %a, %Value %b) {");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %cmp = icmp slt i64 %ia, %ib");
        self.writeln("  %rv = call %Value @val_bool(i1 %cmp)");
        self.writeln("  ret %Value %rv");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Comparison: gt");
        self.writeln("define %Value @val_gt(%Value %a, %Value %b) {");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %cmp = icmp sgt i64 %ia, %ib");
        self.writeln("  %rv = call %Value @val_bool(i1 %cmp)");
        self.writeln("  ret %Value %rv");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Comparison: le");
        self.writeln("define %Value @val_le(%Value %a, %Value %b) {");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %cmp = icmp sle i64 %ia, %ib");
        self.writeln("  %rv = call %Value @val_bool(i1 %cmp)");
        self.writeln("  ret %Value %rv");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Comparison: ge");
        self.writeln("define %Value @val_ge(%Value %a, %Value %b) {");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %cmp = icmp sge i64 %ia, %ib");
        self.writeln("  %rv = call %Value @val_bool(i1 %cmp)");
        self.writeln("  ret %Value %rv");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Comparison: eq");
        self.writeln("define %Value @val_eq(%Value %a, %Value %b) {");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %cmp = icmp eq i64 %ia, %ib");
        self.writeln("  %rv = call %Value @val_bool(i1 %cmp)");
        self.writeln("  ret %Value %rv");
        self.writeln("}");
        self.writeln("");

        self.writeln("; Comparison: ne");
        self.writeln("define %Value @val_ne(%Value %a, %Value %b) {");
        self.writeln("  %ia = call i64 @get_int(%Value %a)");
        self.writeln("  %ib = call i64 @get_int(%Value %b)");
        self.writeln("  %cmp = icmp ne i64 %ia, %ib");
        self.writeln("  %rv = call %Value @val_bool(i1 %cmp)");
        self.writeln("  ret %Value %rv");
        self.writeln("}");
        self.writeln("");

        // Print function
        self.writeln("; Print value");
        self.writeln("define void @val_print(%Value %v) {");
        self.writeln("  %tag = call i8 @get_type(%Value %v)");
        self.writeln("  switch i8 %tag, label %done [");
        self.writeln("    i8 1, label %print_bool");
        self.writeln("    i8 2, label %print_int");
        self.writeln("    i8 3, label %print_float");
        self.writeln("  ]");
        self.writeln("print_bool:");
        self.writeln("  %b = call i1 @get_bool(%Value %v)");
        self.writeln("  br i1 %b, label %print_true, label %print_false");
        self.writeln("print_true:");
        self.writeln("  %str_true = getelementptr [5 x i8], [5 x i8]* @.str.true, i64 0, i64 0");
        self.writeln("  call i32 (i8*, ...) @printf(i8* %str_true)");
        self.writeln("  br label %done");
        self.writeln("print_false:");
        self.writeln("  %str_false = getelementptr [6 x i8], [6 x i8]* @.str.false, i64 0, i64 0");
        self.writeln("  call i32 (i8*, ...) @printf(i8* %str_false)");
        self.writeln("  br label %done");
        self.writeln("print_int:");
        self.writeln("  %i = call i64 @get_int(%Value %v)");
        self.writeln("  %fmt_int = getelementptr [5 x i8], [5 x i8]* @.str.int, i64 0, i64 0");
        self.writeln("  call i32 (i8*, ...) @printf(i8* %fmt_int, i64 %i)");
        self.writeln("  br label %done");
        self.writeln("print_float:");
        self.writeln("  %f = call double @get_float(%Value %v)");
        self.writeln("  %fmt_float = getelementptr [3 x i8], [3 x i8]* @.str.float, i64 0, i64 0");
        self.writeln("  call i32 (i8*, ...) @printf(i8* %fmt_float, double %f)");
        self.writeln("  br label %done");
        self.writeln("done:");
        self.writeln("  ret void");
        self.writeln("}");
        self.writeln("");
    }

    fn generate_function(&mut self, func: &CompiledFunction) -> CodegenResult<()> {
        self.temp_counter = 0;
        self.block_counter = 0;

        let func_name = self.sanitize_name(&func.name);

        // Function signature
        let params = func
            .params
            .iter()
            .map(|p| format!("%Value %{}", self.sanitize_name(p)))
            .collect::<Vec<_>>()
            .join(", ");

        self.writeln(&format!("; Function: {}", func.name));
        self.writeln(&format!("define %Value @{}({}) {{", func_name, params));

        // Generate using basic blocks
        self.generate_with_basic_blocks(func)?;

        self.writeln("}");
        self.writeln("");

        Ok(())
    }

    /// Generate code using basic blocks for proper control flow
    fn generate_with_basic_blocks(&mut self, func: &CompiledFunction) -> CodegenResult<()> {
        let instructions = &func.instructions;

        // Pass 1: Find all block entry points (targets of jumps)
        let mut block_entries: HashSet<usize> = HashSet::new();
        block_entries.insert(0); // Entry block

        for (i, instr) in instructions.iter().enumerate() {
            match &instr.opcode {
                OpCode::Jump(offset) => {
                    // Jump is relative: target = ip + offset + 1
                    let target = (i as i64 + *offset as i64 + 1) as usize;
                    if target <= instructions.len() {
                        block_entries.insert(target);
                    }
                    // Next instruction after jump is also a potential entry
                    if i + 1 < instructions.len() {
                        block_entries.insert(i + 1);
                    }
                }
                OpCode::JumpIfNot(offset) => {
                    // JumpIfNot is relative: target = ip + offset + 1
                    let target = (i as i64 + *offset as i64 + 1) as usize;
                    if target <= instructions.len() {
                        block_entries.insert(target);
                    }
                    // Fall-through is also an entry
                    if i + 1 < instructions.len() {
                        block_entries.insert(i + 1);
                    }
                }
                OpCode::Return => {
                    // Next instruction after return starts a new block
                    if i + 1 < instructions.len() {
                        block_entries.insert(i + 1);
                    }
                }
                _ => {}
            }
        }

        // Create ordered list of block starts
        let mut block_starts: Vec<usize> = block_entries.into_iter().collect();
        block_starts.sort();

        // Create mapping from instruction index to block name
        let mut block_names: HashMap<usize, String> = HashMap::new();
        for &start in &block_starts {
            let name = if start == 0 {
                "entry".to_string()
            } else {
                self.next_block()
            };
            block_names.insert(start, name);
        }

        // Add default end block
        let default_end_block = self.next_block();
        let return_void_block = self.next_block();

        // Find the end of each block
        let mut block_ends: HashMap<usize, usize> = HashMap::new();
        for (i, &start) in block_starts.iter().enumerate() {
            let end = if i + 1 < block_starts.len() {
                block_starts[i + 1]
            } else {
                instructions.len()
            };
            block_ends.insert(start, end);
        }

        // Generate entry block with allocations
        self.writeln("entry:");

        // Stack allocation
        self.writeln("  ; Stack allocation");
        self.writeln("  %stack = alloca [64 x %Value]");
        self.writeln("  %sp = alloca i32");
        self.writeln("  store i32 0, i32* %sp");
        self.writeln("");

        // Allocate local variables for parameters
        for param in &func.params {
            let param_name = self.sanitize_name(param);
            self.writeln(&format!("  %local_{} = alloca %Value", param_name));
            self.writeln(&format!(
                "  store %Value %{}, %Value* %local_{}",
                param_name, param_name
            ));
        }

        // Collect all variables that will be used (for Store operations)
        let mut local_vars: HashSet<String> = HashSet::new();
        for instr in instructions {
            if let OpCode::Store(name) = &instr.opcode {
                let local_name = self.sanitize_name(name);
                if !func.params.contains(name) && !local_vars.contains(&local_name) {
                    local_vars.insert(local_name.clone());
                    self.writeln(&format!("  %local_{} = alloca %Value", local_name));
                }
            }
        }
        self.writeln("");

        // If entry block has instructions, start generating
        // Check if 0 is the only entry, if so continue in same block
        if block_starts.len() == 1 && block_starts[0] == 0 {
            // Simple case: no jumps, generate all instructions in entry block
            self.generate_block_instructions(
                instructions,
                0,
                instructions.len(),
                &func.name,
                &block_names,
                &default_end_block,
            )?;

            // Add default return
            self.writeln(&format!("  br label %{}", default_end_block));

            self.writeln(&format!("{}:", default_end_block));
            self.generate_default_return(&return_void_block);
        } else {
            // Complex case: has jumps, need multiple blocks
            // Branch to actual first block if entry has instructions, otherwise jump to next block
            let first_block = block_names.get(&0)
                .ok_or_else(|| crate::error::CodegenError::Internal("Missing block 0 in block_names".to_string()))?;
            if first_block == "entry" {
                // Generate entry block content
                let end = *block_ends.get(&0)
                    .ok_or_else(|| crate::error::CodegenError::Internal("Missing block 0 in block_ends".to_string()))?;
                self.generate_block_instructions(
                    instructions,
                    0,
                    end,
                    &func.name,
                    &block_names,
                    &default_end_block,
                )?;

                // Check if block already has a terminator
                if !self.block_has_terminator(instructions, 0, end) {
                    let next_block = if end < instructions.len() {
                        block_names
                            .get(&end)
                            .cloned()
                            .unwrap_or_else(|| default_end_block.clone())
                    } else {
                        default_end_block.clone()
                    };
                    self.writeln(&format!("  br label %{}", next_block));
                }
            } else {
                self.writeln(&format!("  br label %{}", first_block));
            }

            // Generate other blocks
            for &start in &block_starts {
                if start == 0 {
                    continue; // Already handled entry
                }

                let block_name = block_names.get(&start)
                    .ok_or_else(|| crate::error::CodegenError::Internal(format!("Missing block {} in block_names", start)))?;
                let end = *block_ends.get(&start)
                    .ok_or_else(|| crate::error::CodegenError::Internal(format!("Missing block {} in block_ends", start)))?;

                self.writeln(&format!("{}:", block_name));
                self.generate_block_instructions(
                    instructions,
                    start,
                    end,
                    &func.name,
                    &block_names,
                    &default_end_block,
                )?;

                // Add terminator if not present
                if !self.block_has_terminator(instructions, start, end) {
                    let next_block = if end < instructions.len() {
                        block_names
                            .get(&end)
                            .cloned()
                            .unwrap_or_else(|| default_end_block.clone())
                    } else {
                        default_end_block.clone()
                    };
                    self.writeln(&format!("  br label %{}", next_block));
                }
            }

            // Generate default return block
            self.writeln(&format!("{}:", default_end_block));
            self.generate_default_return(&return_void_block);
        }

        Ok(())
    }

    fn block_has_terminator(&self, instructions: &[vais_ir::Instruction], start: usize, end: usize) -> bool {
        if start >= end || end > instructions.len() {
            return false;
        }
        let last_idx = end - 1;
        if last_idx >= instructions.len() {
            return false;
        }
        matches!(
            instructions[last_idx].opcode,
            OpCode::Return | OpCode::Jump(_) | OpCode::JumpIfNot(_)
        )
    }

    fn generate_default_return(&mut self, return_void_block: &str) {
        // Check if stack has value
        let sp_check = self.next_temp();
        self.writeln(&format!("  {} = load i32, i32* %sp", sp_check));
        let has_value = self.next_temp();
        self.writeln(&format!("  {} = icmp sgt i32 {}, 0", has_value, sp_check));
        self.writeln(&format!(
            "  br i1 {}, label %return_stack, label %{}",
            has_value, return_void_block
        ));

        self.writeln("return_stack:");
        let stack_val = self.pop_stack();
        self.writeln(&format!("  ret %Value {}", stack_val));

        self.writeln(&format!("{}:", return_void_block));
        let default_ret = self.next_temp();
        self.writeln(&format!("  {} = call %Value @val_void()", default_ret));
        self.writeln(&format!("  ret %Value {}", default_ret));
    }

    fn generate_block_instructions(
        &mut self,
        instructions: &[vais_ir::Instruction],
        start: usize,
        end: usize,
        func_name: &str,
        block_names: &HashMap<usize, String>,
        default_end_block: &str,
    ) -> CodegenResult<()> {
        for i in start..end {
            if i >= instructions.len() {
                break;
            }
            let instr = &instructions[i];
            self.writeln(&format!("  ; Instruction {}: {:?}", i, instr.opcode));

            match &instr.opcode {
                OpCode::Const(value) => {
                    let val_str = match value {
                        Value::Int(n) => format!("call %Value @val_int(i64 {})", n),
                        Value::Float(f) => format!("call %Value @val_float(double {:e})", f),
                        Value::Bool(b) => {
                            format!("call %Value @val_bool(i1 {})", if *b { 1 } else { 0 })
                        }
                        _ => "call %Value @val_void()".to_string(),
                    };
                    let temp = self.next_temp();
                    self.writeln(&format!("  {} = {}", temp, val_str));
                    self.push_stack(&temp);
                }
                OpCode::Load(name) => {
                    let local_name = self.sanitize_name(name);
                    let temp = self.next_temp();
                    self.writeln(&format!(
                        "  {} = load %Value, %Value* %local_{}",
                        temp, local_name
                    ));
                    self.push_stack(&temp);
                }
                OpCode::Store(name) => {
                    let local_name = self.sanitize_name(name);
                    let temp = self.pop_stack();
                    self.writeln(&format!(
                        "  store %Value {}, %Value* %local_{}",
                        temp, local_name
                    ));
                }
                OpCode::Add => self.generate_binary_op("val_add"),
                OpCode::Sub => self.generate_binary_op("val_sub"),
                OpCode::Mul => self.generate_binary_op("val_mul"),
                OpCode::Div => self.generate_binary_op("val_div"),
                OpCode::Mod => self.generate_binary_op("val_mod"),
                OpCode::Lt => self.generate_binary_op("val_lt"),
                OpCode::Gt => self.generate_binary_op("val_gt"),
                OpCode::Lte => self.generate_binary_op("val_le"),
                OpCode::Gte => self.generate_binary_op("val_ge"),
                OpCode::Eq => self.generate_binary_op("val_eq"),
                OpCode::Neq => self.generate_binary_op("val_ne"),
                OpCode::And => {
                    let b = self.pop_stack();
                    let a = self.pop_stack();
                    let ta = self.next_temp();
                    let tb = self.next_temp();
                    let tr = self.next_temp();
                    let result = self.next_temp();
                    self.writeln(&format!("  {} = call i1 @get_bool(%Value {})", ta, a));
                    self.writeln(&format!("  {} = call i1 @get_bool(%Value {})", tb, b));
                    self.writeln(&format!("  {} = and i1 {}, {}", tr, ta, tb));
                    self.writeln(&format!(
                        "  {} = call %Value @val_bool(i1 {})",
                        result, tr
                    ));
                    self.push_stack(&result);
                }
                OpCode::Or => {
                    let b = self.pop_stack();
                    let a = self.pop_stack();
                    let ta = self.next_temp();
                    let tb = self.next_temp();
                    let tr = self.next_temp();
                    let result = self.next_temp();
                    self.writeln(&format!("  {} = call i1 @get_bool(%Value {})", ta, a));
                    self.writeln(&format!("  {} = call i1 @get_bool(%Value {})", tb, b));
                    self.writeln(&format!("  {} = or i1 {}, {}", tr, ta, tb));
                    self.writeln(&format!(
                        "  {} = call %Value @val_bool(i1 {})",
                        result, tr
                    ));
                    self.push_stack(&result);
                }
                OpCode::Not => {
                    let a = self.pop_stack();
                    let ta = self.next_temp();
                    let tr = self.next_temp();
                    let result = self.next_temp();
                    self.writeln(&format!("  {} = call i1 @get_bool(%Value {})", ta, a));
                    self.writeln(&format!("  {} = xor i1 {}, true", tr, ta));
                    self.writeln(&format!(
                        "  {} = call %Value @val_bool(i1 {})",
                        result, tr
                    ));
                    self.push_stack(&result);
                }
                OpCode::Neg => {
                    let a = self.pop_stack();
                    let ta = self.next_temp();
                    let tr = self.next_temp();
                    let result = self.next_temp();
                    self.writeln(&format!("  {} = call i64 @get_int(%Value {})", ta, a));
                    self.writeln(&format!("  {} = sub i64 0, {}", tr, ta));
                    self.writeln(&format!(
                        "  {} = call %Value @val_int(i64 {})",
                        result, tr
                    ));
                    self.push_stack(&result);
                }
                OpCode::Jump(offset) => {
                    // Jump is relative: target = ip + offset + 1
                    let target = (i as i64 + *offset as i64 + 1) as usize;
                    let target_block = block_names
                        .get(&target)
                        .cloned()
                        .unwrap_or_else(|| default_end_block.to_string());
                    self.writeln(&format!("  br label %{}", target_block));
                }
                OpCode::JumpIfNot(offset) => {
                    // Pop condition from stack
                    let cond_val = self.pop_stack();
                    let cond_bool = self.next_temp();
                    self.writeln(&format!(
                        "  {} = call i1 @get_bool(%Value {})",
                        cond_bool, cond_val
                    ));

                    // JumpIfNot is relative: target = ip + offset + 1
                    let target = (i as i64 + *offset as i64 + 1) as usize;
                    let target_block = block_names
                        .get(&target)
                        .cloned()
                        .unwrap_or_else(|| default_end_block.to_string());

                    // Fall-through block (next instruction)
                    let fallthrough_block = if i + 1 < instructions.len() {
                        block_names
                            .get(&(i + 1))
                            .cloned()
                            .unwrap_or_else(|| default_end_block.to_string())
                    } else {
                        default_end_block.to_string()
                    };

                    // JumpIfNot: jump to target if condition is FALSE
                    // So: if true -> fallthrough, if false -> target
                    self.writeln(&format!(
                        "  br i1 {}, label %{}, label %{}",
                        cond_bool, fallthrough_block, target_block
                    ));
                }
                OpCode::Call(name, arg_count) => {
                    let callee = self.sanitize_name(name);
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.pop_stack());
                    }
                    args.reverse();

                    let args_str = args
                        .iter()
                        .map(|a| format!("%Value {}", a))
                        .collect::<Vec<_>>()
                        .join(", ");

                    let result = self.next_temp();
                    self.writeln(&format!(
                        "  {} = call %Value @{}({})",
                        result, callee, args_str
                    ));
                    self.push_stack(&result);
                }
                OpCode::SelfCall(arg_count) | OpCode::TailSelfCall(arg_count) => {
                    let callee = self.sanitize_name(func_name);
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.pop_stack());
                    }
                    args.reverse();

                    let args_str = args
                        .iter()
                        .map(|a| format!("%Value {}", a))
                        .collect::<Vec<_>>()
                        .join(", ");

                    let result = self.next_temp();
                    self.writeln(&format!(
                        "  {} = call %Value @{}({})",
                        result, callee, args_str
                    ));
                    self.push_stack(&result);
                }
                OpCode::Return => {
                    let result = self.pop_stack();
                    self.writeln(&format!("  ret %Value {}", result));
                }
                OpCode::Pop => {
                    self.pop_stack();
                }
                OpCode::Dup => {
                    // For Dup, we need to peek without popping, then push again
                    let val = self.peek_stack();
                    self.push_stack(&val);
                }
                OpCode::Nop => {
                    self.writeln("  ; nop");
                }
                OpCode::CallBuiltin(name, arg_count) => {
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.pop_stack());
                    }
                    args.reverse();

                    match name.to_uppercase().as_str() {
                        "PRINT" | "PRINTLN" => {
                            for arg in &args {
                                self.writeln(&format!("  call void @val_print(%Value {})", arg));
                            }
                            let nl = self.next_temp();
                            self.writeln(&format!(
                                "  {} = getelementptr [2 x i8], [2 x i8]* @.str.newline, i64 0, i64 0",
                                nl
                            ));
                            self.writeln(&format!("  call i32 (i8*, ...) @printf(i8* {})", nl));
                            let result = self.next_temp();
                            self.writeln(&format!("  {} = call %Value @val_void()", result));
                            self.push_stack(&result);
                        }
                        "SQRT" => {
                            let arg = &args[0];
                            let fval = self.next_temp();
                            let result_f = self.next_temp();
                            let result = self.next_temp();
                            self.writeln(&format!(
                                "  {} = call double @get_float(%Value {})",
                                fval, arg
                            ));
                            self.writeln(&format!(
                                "  {} = call double @sqrt(double {})",
                                result_f, fval
                            ));
                            self.writeln(&format!(
                                "  {} = call %Value @val_float(double {})",
                                result, result_f
                            ));
                            self.push_stack(&result);
                        }
                        "ABS" => {
                            let arg = &args[0];
                            let ival = self.next_temp();
                            let is_neg = self.next_temp();
                            let neg_val = self.next_temp();
                            let result_i = self.next_temp();
                            let result = self.next_temp();
                            self.writeln(&format!(
                                "  {} = call i64 @get_int(%Value {})",
                                ival, arg
                            ));
                            self.writeln(&format!("  {} = icmp slt i64 {}, 0", is_neg, ival));
                            self.writeln(&format!("  {} = sub i64 0, {}", neg_val, ival));
                            self.writeln(&format!(
                                "  {} = select i1 {}, i64 {}, i64 {}",
                                result_i, is_neg, neg_val, ival
                            ));
                            self.writeln(&format!(
                                "  {} = call %Value @val_int(i64 {})",
                                result, result_i
                            ));
                            self.push_stack(&result);
                        }
                        _ => {
                            self.writeln(&format!("  ; unsupported builtin: {}", name));
                            let result = self.next_temp();
                            self.writeln(&format!("  {} = call %Value @val_void()", result));
                            self.push_stack(&result);
                        }
                    }
                }
                _ => {
                    self.writeln(&format!("  ; TODO: {:?}", instr.opcode));
                    let result = self.next_temp();
                    self.writeln(&format!("  {} = call %Value @val_void()", result));
                    self.push_stack(&result);
                }
            }
        }

        Ok(())
    }

    fn generate_binary_op(&mut self, op_func: &str) {
        let b = self.pop_stack();
        let a = self.pop_stack();
        let result = self.next_temp();
        self.writeln(&format!(
            "  {} = call %Value @{}(%Value {}, %Value {})",
            result, op_func, a, b
        ));
        self.push_stack(&result);
    }

    fn push_stack(&mut self, val: &str) {
        let sp_val = self.next_temp();
        let ptr = self.next_temp();
        let new_sp = self.next_temp();

        self.writeln(&format!("  {} = load i32, i32* %sp", sp_val));
        self.writeln(&format!(
            "  {} = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 {}",
            ptr, sp_val
        ));
        self.writeln(&format!("  store %Value {}, %Value* {}", val, ptr));
        self.writeln(&format!("  {} = add i32 {}, 1", new_sp, sp_val));
        self.writeln(&format!("  store i32 {}, i32* %sp", new_sp));
    }

    fn pop_stack(&mut self) -> String {
        let sp_val = self.next_temp();
        let new_sp = self.next_temp();
        let ptr = self.next_temp();
        let val = self.next_temp();

        self.writeln(&format!("  {} = load i32, i32* %sp", sp_val));
        self.writeln(&format!("  {} = sub i32 {}, 1", new_sp, sp_val));
        self.writeln(&format!("  store i32 {}, i32* %sp", new_sp));
        self.writeln(&format!(
            "  {} = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 {}",
            ptr, new_sp
        ));
        self.writeln(&format!("  {} = load %Value, %Value* {}", val, ptr));

        val
    }

    fn peek_stack(&mut self) -> String {
        let sp_val = self.next_temp();
        let top_idx = self.next_temp();
        let ptr = self.next_temp();
        let val = self.next_temp();

        self.writeln(&format!("  {} = load i32, i32* %sp", sp_val));
        self.writeln(&format!("  {} = sub i32 {}, 1", top_idx, sp_val));
        self.writeln(&format!(
            "  {} = getelementptr [64 x %Value], [64 x %Value]* %stack, i64 0, i32 {}",
            ptr, top_idx
        ));
        self.writeln(&format!("  {} = load %Value, %Value* {}", val, ptr));

        val
    }

    fn generate_main_wrapper(&mut self, functions: &[CompiledFunction]) {
        self.writeln("; Main entry point");
        self.writeln("define i32 @main() {");

        if functions.iter().any(|f| f.name == "__main__") {
            self.writeln("  %result = call %Value @__main__()");
            self.writeln("  call void @val_print(%Value %result)");
            self.writeln(
                "  %nl = getelementptr [2 x i8], [2 x i8]* @.str.newline, i64 0, i64 0",
            );
            self.writeln("  call i32 (i8*, ...) @printf(i8* %nl)");
        }

        self.writeln("  ret i32 0");
        self.writeln("}");
    }

    fn sanitize_name(&self, name: &str) -> String {
        name.replace(['-', '.'], "_")
    }
}

impl Default for LlvmCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 편의 함수: 컴파일된 함수들을 LLVM IR로 변환
pub fn generate_llvm_ir(functions: &[CompiledFunction]) -> CodegenResult<String> {
    let mut gen = LlvmCodeGenerator::new();
    gen.generate(functions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_ir::{Instruction, OpCode, Value};

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
            local_count: 0,
        };

        let ir = generate_llvm_ir(&[func]).unwrap();
        assert!(ir.contains("define %Value @__main__"));
        assert!(ir.contains("val_int(i64 2)"));
        assert!(ir.contains("val_int(i64 3)"));
        assert!(ir.contains("@val_add"));
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
            local_count: 2,
        };

        let ir = generate_llvm_ir(&[func]).unwrap();
        assert!(ir.contains("define %Value @add(%Value %a, %Value %b)"));
        assert!(ir.contains("load %Value, %Value* %local_a"));
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
            local_count: 0,
        };

        let ir = generate_llvm_ir(&[func]).unwrap();
        assert!(ir.contains("@val_lt"));
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
            local_count: 1,
        };

        let ir = generate_llvm_ir(&[func]).unwrap();
        assert!(ir.contains("call %Value @fact("));
    }

    #[test]
    fn test_jump_instructions() {
        // Simulate: if n <= 1 then return 1 else return n * fact(n-1)
        // JumpIfNot offset is RELATIVE: target = ip + offset + 1
        // So at ip=3, JumpIfNot(2) -> target = 3 + 2 + 1 = 6
        let func = CompiledFunction {
            name: "fact".to_string(),
            params: vec!["n".to_string()],
            instructions: vec![
                // 0: Load n
                Instruction::new(OpCode::Load("n".to_string())),
                // 1: Const 1
                Instruction::new(OpCode::Const(Value::Int(1))),
                // 2: Lte (n <= 1)
                Instruction::new(OpCode::Lte),
                // 3: JumpIfNot(2) -> if false, jump to 3+2+1=6 (else branch)
                Instruction::new(OpCode::JumpIfNot(2i32)),
                // 4: Const 1 (return 1)
                Instruction::new(OpCode::Const(Value::Int(1))),
                // 5: Return
                Instruction::new(OpCode::Return),
                // 6: Load n
                Instruction::new(OpCode::Load("n".to_string())),
                // 7: Load n
                Instruction::new(OpCode::Load("n".to_string())),
                // 8: Const 1
                Instruction::new(OpCode::Const(Value::Int(1))),
                // 9: Sub (n - 1)
                Instruction::new(OpCode::Sub),
                // 10: SelfCall (fact(n-1))
                Instruction::new(OpCode::SelfCall(1)),
                // 11: Mul (n * result)
                Instruction::new(OpCode::Mul),
                // 12: Return
                Instruction::new(OpCode::Return),
            ],
            local_count: 1,
        };

        let ir = generate_llvm_ir(&[func]).unwrap();
        // Should have multiple blocks
        assert!(ir.contains("br i1"));
        assert!(ir.contains("br label"));
        // Should have the comparison
        assert!(ir.contains("@val_le"));
        // Should have recursive call
        assert!(ir.contains("call %Value @fact("));
    }
}
