//! Cranelift Code Generator
//!
//! Generates native code using Cranelift JIT compiler.
//! This provides faster compilation than LLVM while still producing efficient code.

#[cfg(feature = "cranelift")]
mod implementation {
    use std::collections::HashMap;

    use cranelift_codegen::entity::EntityRef;
    use cranelift_codegen::ir::{
        types, AbiParam, InstBuilder, Signature, UserFuncName,
    };
    use cranelift_codegen::settings::{self, Configurable};
    use cranelift_codegen::Context;
    use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::{DataDescription, FuncId, Linkage, Module};
    use cranelift_native;

    use vais_ir::{OpCode, Value as VaisValue};
    use vais_lowering::CompiledFunction;

    use crate::error::{CodegenError, CodegenResult};

    /// Cranelift JIT 코드 생성기
    pub struct CraneliftCodeGenerator {
        module: JITModule,
        ctx: Context,
        data_description: DataDescription,
        function_ids: HashMap<String, FuncId>,
    }

    impl CraneliftCodeGenerator {
        pub fn new() -> CodegenResult<Self> {
            let mut flag_builder = settings::builder();
            flag_builder.set("use_colocated_libcalls", "false").unwrap();
            flag_builder.set("is_pic", "false").unwrap();
            let isa_builder = cranelift_native::builder().map_err(|e| {
                CodegenError::Internal(format!("Failed to create native ISA builder: {}", e))
            })?;
            let isa = isa_builder
                .finish(settings::Flags::new(flag_builder))
                .map_err(|e| CodegenError::Internal(format!("Failed to create ISA: {}", e)))?;

            let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
            let module = JITModule::new(builder);
            let ctx = module.make_context();
            let data_description = DataDescription::new();

            Ok(Self {
                module,
                ctx,
                data_description,
                function_ids: HashMap::new(),
            })
        }

        /// Compile all functions and return a callable for __main__
        pub fn compile(&mut self, functions: &[CompiledFunction]) -> CodegenResult<*const u8> {
            // First pass: declare all functions
            for func in functions {
                let sig = self.create_signature(func.params.len());
                let func_id = self
                    .module
                    .declare_function(&func.name, Linkage::Local, &sig)
                    .map_err(|e| CodegenError::Internal(format!("Failed to declare function: {}", e)))?;
                self.function_ids.insert(func.name.clone(), func_id);
            }

            // Second pass: define all functions
            for func in functions {
                self.compile_function(func)?;
            }

            // Finalize
            self.module.finalize_definitions().map_err(|e| {
                CodegenError::Internal(format!("Failed to finalize definitions: {}", e))
            })?;

            // Get main function pointer
            let main_id = self
                .function_ids
                .get("__main__")
                .ok_or_else(|| CodegenError::Internal("No __main__ function found".to_string()))?;

            let code_ptr = self.module.get_finalized_function(*main_id);
            Ok(code_ptr)
        }

        fn create_signature(&self, param_count: usize) -> Signature {
            let mut sig = self.module.make_signature();
            // All values are i64 (tagged)
            for _ in 0..param_count {
                sig.params.push(AbiParam::new(types::I64));
            }
            sig.returns.push(AbiParam::new(types::I64));
            sig
        }

        fn compile_function(&mut self, func: &CompiledFunction) -> CodegenResult<()> {
            let func_id = *self.function_ids.get(&func.name).unwrap();

            self.ctx.func.signature = self.create_signature(func.params.len());
            self.ctx.func.name = UserFuncName::user(0, func_id.as_u32());

            let mut fn_builder_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut fn_builder_ctx);

            // Create entry block
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            // Create variables for parameters
            let mut variables: HashMap<String, Variable> = HashMap::new();
            let mut var_index = 0;

            for (i, param_name) in func.params.iter().enumerate() {
                let var = Variable::new(var_index);
                var_index += 1;
                builder.declare_var(var, types::I64);
                let param_val = builder.block_params(entry_block)[i];
                builder.def_var(var, param_val);
                variables.insert(param_name.clone(), var);
            }

            // Stack for operations (virtual - we track values symbolically)
            let mut stack: Vec<cranelift_codegen::ir::Value> = Vec::new();

            // Track whether current block has a terminator
            let mut has_terminator = false;

            // For control flow, we use a simplified approach:
            // Skip Jump/JumpIfNot instructions and rely on non-branching code paths
            // This works for simple expressions without complex control flow

            // Generate code for each instruction
            for (i, instr) in func.instructions.iter().enumerate() {
                match &instr.opcode {
                    OpCode::Const(value) => {
                        let val = match value {
                            VaisValue::Int(n) => {
                                // Tag: 2 (int), value in upper bits
                                let tagged = ((*n as u64) << 8) | 2;
                                builder.ins().iconst(types::I64, tagged as i64)
                            }
                            VaisValue::Float(f) => {
                                // Tag: 3 (float), bits in upper 56 bits
                                let bits = f.to_bits();
                                let tagged = (bits << 8) | 3;
                                builder.ins().iconst(types::I64, tagged as i64)
                            }
                            VaisValue::Bool(b) => {
                                // Tag: 1 (bool)
                                let tagged = (if *b { 1u64 } else { 0u64 } << 8) | 1;
                                builder.ins().iconst(types::I64, tagged as i64)
                            }
                            _ => {
                                // Tag: 0 (void)
                                builder.ins().iconst(types::I64, 0)
                            }
                        };
                        stack.push(val);
                    }
                    OpCode::Load(name) => {
                        if let Some(var) = variables.get(name) {
                            let val = builder.use_var(*var);
                            stack.push(val);
                        } else {
                            // Variable not found, push void
                            let void_val = builder.ins().iconst(types::I64, 0);
                            stack.push(void_val);
                        }
                    }
                    OpCode::Store(name) => {
                        if let Some(val) = stack.pop() {
                            let var = if let Some(v) = variables.get(name) {
                                *v
                            } else {
                                let v = Variable::new(var_index);
                                var_index += 1;
                                builder.declare_var(v, types::I64);
                                variables.insert(name.clone(), v);
                                v
                            };
                            builder.def_var(var, val);
                        }
                    }
                    OpCode::Add => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            // Extract values (shift right 8)
                            let a_val = builder.ins().sshr_imm(a, 8);
                            let b_val = builder.ins().sshr_imm(b, 8);
                            let result = builder.ins().iadd(a_val, b_val);
                            // Re-tag as int
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 2);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Sub => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            let a_val = builder.ins().sshr_imm(a, 8);
                            let b_val = builder.ins().sshr_imm(b, 8);
                            let result = builder.ins().isub(a_val, b_val);
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 2);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Mul => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            let a_val = builder.ins().sshr_imm(a, 8);
                            let b_val = builder.ins().sshr_imm(b, 8);
                            let result = builder.ins().imul(a_val, b_val);
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 2);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Div => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            let a_val = builder.ins().sshr_imm(a, 8);
                            let b_val = builder.ins().sshr_imm(b, 8);
                            let result = builder.ins().sdiv(a_val, b_val);
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 2);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Lt => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            let a_val = builder.ins().sshr_imm(a, 8);
                            let b_val = builder.ins().sshr_imm(b, 8);
                            let cmp = builder.ins().icmp(
                                cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                                a_val,
                                b_val,
                            );
                            let result = builder.ins().uextend(types::I64, cmp);
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 1);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Gt => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            let a_val = builder.ins().sshr_imm(a, 8);
                            let b_val = builder.ins().sshr_imm(b, 8);
                            let cmp = builder.ins().icmp(
                                cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan,
                                a_val,
                                b_val,
                            );
                            let result = builder.ins().uextend(types::I64, cmp);
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 1);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Lte => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            let a_val = builder.ins().sshr_imm(a, 8);
                            let b_val = builder.ins().sshr_imm(b, 8);
                            let cmp = builder.ins().icmp(
                                cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                                a_val,
                                b_val,
                            );
                            let result = builder.ins().uextend(types::I64, cmp);
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 1);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Gte => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            let a_val = builder.ins().sshr_imm(a, 8);
                            let b_val = builder.ins().sshr_imm(b, 8);
                            let cmp = builder.ins().icmp(
                                cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                                a_val,
                                b_val,
                            );
                            let result = builder.ins().uextend(types::I64, cmp);
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 1);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Eq => {
                        if stack.len() >= 2 {
                            let b = stack.pop().unwrap();
                            let a = stack.pop().unwrap();
                            let cmp = builder.ins().icmp(
                                cranelift_codegen::ir::condcodes::IntCC::Equal,
                                a,
                                b,
                            );
                            let result = builder.ins().uextend(types::I64, cmp);
                            let shifted = builder.ins().ishl_imm(result, 8);
                            let tag = builder.ins().iconst(types::I64, 1);
                            let tagged = builder.ins().bor(shifted, tag);
                            stack.push(tagged);
                        }
                    }
                    OpCode::Return => {
                        if let Some(val) = stack.pop() {
                            builder.ins().return_(&[val]);
                        } else {
                            let void_val = builder.ins().iconst(types::I64, 0);
                            builder.ins().return_(&[void_val]);
                        }
                        has_terminator = true;
                    }
                    OpCode::Pop => {
                        stack.pop();
                    }
                    OpCode::Dup => {
                        if let Some(val) = stack.last().cloned() {
                            stack.push(val);
                        }
                    }
                    OpCode::Jump(_offset) => {
                        // Skip jump in simplified mode - control flow not fully supported
                        // Complex control flow should use VM interpreter instead
                    }
                    OpCode::JumpIfNot(_offset) => {
                        // Pop the condition but don't branch - simplified mode
                        // This means JIT only works for non-branching code paths
                        stack.pop();
                    }
                    OpCode::Call(name, arg_count) => {
                        // Function call
                        if let Some(&callee_id) = self.function_ids.get(name) {
                            let callee_ref = self.module.declare_func_in_func(callee_id, builder.func);

                            // Pop arguments from stack
                            let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
                            for _ in 0..*arg_count {
                                if let Some(arg) = stack.pop() {
                                    args.push(arg);
                                }
                            }
                            args.reverse(); // Arguments are in reverse order

                            let call = builder.ins().call(callee_ref, &args);
                            let result = builder.inst_results(call)[0];
                            stack.push(result);
                        } else {
                            // Function not found, push void
                            let void_val = builder.ins().iconst(types::I64, 0);
                            stack.push(void_val);
                        }
                    }
                    OpCode::SelfCall(arg_count) => {
                        // Self/recursive call
                        let callee_ref = self.module.declare_func_in_func(func_id, builder.func);

                        // Pop arguments from stack
                        let mut args: Vec<cranelift_codegen::ir::Value> = Vec::new();
                        for _ in 0..*arg_count {
                            if let Some(arg) = stack.pop() {
                                args.push(arg);
                            }
                        }
                        args.reverse(); // Arguments are in reverse order

                        let call = builder.ins().call(callee_ref, &args);
                        let result = builder.inst_results(call)[0];
                        stack.push(result);
                    }
                    _ => {
                        // Unsupported instruction, push void
                        let void_val = builder.ins().iconst(types::I64, 0);
                        stack.push(void_val);
                    }
                }
            }

            // Default return if no explicit return
            if !has_terminator {
                if let Some(val) = stack.pop() {
                    builder.ins().return_(&[val]);
                } else {
                    let void_val = builder.ins().iconst(types::I64, 0);
                    builder.ins().return_(&[void_val]);
                }
            }

            builder.finalize();

            self.module
                .define_function(func_id, &mut self.ctx)
                .map_err(|e| CodegenError::Internal(format!("Failed to define function: {}", e)))?;

            self.module.clear_context(&mut self.ctx);

            Ok(())
        }

        /// Execute the compiled __main__ function and return the result
        pub fn execute(&self, code_ptr: *const u8) -> i64 {
            let func: fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
            func()
        }

        /// Decode a tagged value to a displayable string
        pub fn decode_value(tagged: i64) -> String {
            let tag = (tagged & 0xFF) as u8;
            let value = tagged >> 8;

            match tag {
                0 => "void".to_string(),
                1 => {
                    if value != 0 {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                2 => value.to_string(),
                3 => {
                    let bits = (tagged as u64) >> 8;
                    let f = f64::from_bits(bits << 8); // Approximate
                    format!("{}", f)
                }
                _ => format!("<unknown:{}>", tagged),
            }
        }
    }

    impl Default for CraneliftCodeGenerator {
        fn default() -> Self {
            Self::new().expect("Failed to create CraneliftCodeGenerator")
        }
    }

    /// JIT compile and execute Vais functions
    pub fn jit_execute(functions: &[CompiledFunction]) -> CodegenResult<i64> {
        let mut gen = CraneliftCodeGenerator::new()?;
        let code_ptr = gen.compile(functions)?;
        Ok(gen.execute(code_ptr))
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
            };

            let result = jit_execute(&[func]).unwrap();
            let decoded = (result >> 8) as i64;
            assert_eq!(decoded, 5);
        }

        #[test]
        fn test_arithmetic() {
            let func = CompiledFunction {
                name: "__main__".to_string(),
                params: vec![],
                instructions: vec![
                    Instruction::new(OpCode::Const(Value::Int(10))),
                    Instruction::new(OpCode::Const(Value::Int(3))),
                    Instruction::new(OpCode::Mul),
                    Instruction::new(OpCode::Const(Value::Int(5))),
                    Instruction::new(OpCode::Sub),
                    Instruction::new(OpCode::Return),
                ],
            };

            let result = jit_execute(&[func]).unwrap();
            let decoded = (result >> 8) as i64;
            assert_eq!(decoded, 25); // 10 * 3 - 5 = 25
        }

        #[test]
        fn test_comparison() {
            let func = CompiledFunction {
                name: "__main__".to_string(),
                params: vec![],
                instructions: vec![
                    Instruction::new(OpCode::Const(Value::Int(5))),
                    Instruction::new(OpCode::Const(Value::Int(3))),
                    Instruction::new(OpCode::Gt),
                    Instruction::new(OpCode::Return),
                ],
            };

            let result = jit_execute(&[func]).unwrap();
            let tag = (result & 0xFF) as u8;
            let value = result >> 8;
            assert_eq!(tag, 1); // bool tag
            assert_eq!(value, 1); // true
        }
    }
}

#[cfg(feature = "cranelift")]
pub use implementation::*;

#[cfg(not(feature = "cranelift"))]
pub fn jit_execute(
    _functions: &[vais_lowering::CompiledFunction],
) -> crate::error::CodegenResult<i64> {
    Err(crate::error::CodegenError::Internal(
        "Cranelift feature not enabled. Build with --features cranelift".to_string(),
    ))
}
