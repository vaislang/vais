//! JIT compiler implementation using Cranelift.

use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};

use vais_ast::{BinOp, Expr, Function, FunctionBody, Item, Module as AstModule, Spanned, Stmt, Type, UnaryOp};
use vais_types::ResolvedType;

use crate::runtime::JitRuntime;
use crate::types::TypeMapper;
use crate::JitError;

/// JIT compiler for Vais code.
pub struct JitCompiler {
    /// Cranelift JIT module.
    module: JITModule,
    /// Function builder context (reused across compilations).
    builder_context: FunctionBuilderContext,
    /// Cranelift codegen context.
    ctx: codegen::Context,
    /// Data section description.
    #[allow(dead_code)]
    data_description: DataDescription,
    /// Type mapper for Vais -> Cranelift type conversion.
    type_mapper: TypeMapper,
    /// JIT runtime for external function resolution.
    #[allow(dead_code)]
    runtime: JitRuntime,
    /// Map of compiled function names to their IDs.
    compiled_functions: HashMap<String, FuncId>,
    /// Map of external function names to their IDs.
    external_functions: HashMap<String, FuncId>,
}

impl JitCompiler {
    /// Creates a new JIT compiler.
    pub fn new() -> Result<Self, JitError> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| JitError::Cranelift(e.to_string()))?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| JitError::Cranelift(e.to_string()))?;
        flag_builder
            .set("opt_level", "speed")
            .map_err(|e| JitError::Cranelift(e.to_string()))?;

        let isa_builder = cranelift_native::builder()
            .map_err(|e| JitError::Cranelift(format!("Native ISA error: {}", e)))?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| JitError::Cranelift(e.to_string()))?;

        let pointer_type = isa.pointer_type();

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        // Register runtime symbols
        let runtime = JitRuntime::new();
        for name in runtime.registered_functions() {
            if let Some(ptr) = runtime.lookup(name) {
                builder.symbol(name, ptr);
            }
        }

        let module = JITModule::new(builder);

        Ok(Self {
            module,
            builder_context: FunctionBuilderContext::new(),
            ctx: codegen::Context::new(),
            data_description: DataDescription::new(),
            type_mapper: TypeMapper::new(pointer_type),
            runtime,
            compiled_functions: HashMap::new(),
            external_functions: HashMap::new(),
        })
    }

    /// Compiles and runs the main function from a module, returning its i64 result.
    pub fn compile_and_run_main(&mut self, ast: &AstModule) -> Result<i64, JitError> {
        // Compile all items
        self.compile_module(ast)?;

        // Look up main or __repl_main
        let main_name = if self.compiled_functions.contains_key("main") {
            "main"
        } else if self.compiled_functions.contains_key("__repl_main") {
            "__repl_main"
        } else {
            return Err(JitError::FunctionNotFound("main".to_string()));
        };

        let func_id = self.compiled_functions[main_name];

        // Finalize the module and get the function pointer
        self.module.finalize_definitions()?;

        let code_ptr = self.module.get_finalized_function(func_id);

        // Call the function
        let func: fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
        Ok(func())
    }

    /// Compiles a module (all items).
    pub fn compile_module(&mut self, ast: &AstModule) -> Result<(), JitError> {
        // First pass: declare all functions
        for item in &ast.items {
            if let Item::Function(func) = &item.node {
                self.declare_function(func)?;
            }
        }

        // Second pass: compile all function bodies
        for item in &ast.items {
            if let Item::Function(func) = &item.node {
                self.compile_function(func)?;
            }
        }

        Ok(())
    }

    /// Declares a function signature.
    fn declare_function(&mut self, func: &Function) -> Result<FuncId, JitError> {
        let name = &func.name.node;

        // Build function signature
        let mut sig = self.module.make_signature();

        // Add parameters
        for param in &func.params {
            let ty = self.resolve_type(&param.ty.node);
            let cl_ty = self.type_mapper.map_type(&ty);
            sig.params.push(AbiParam::new(cl_ty));
        }

        // Add return type
        let ret_ty = func
            .ret_type
            .as_ref()
            .map(|t| self.resolve_type(&t.node))
            .unwrap_or(ResolvedType::Unit);
        if !matches!(ret_ty, ResolvedType::Unit) {
            let cl_ret = self.type_mapper.map_type(&ret_ty);
            sig.returns.push(AbiParam::new(cl_ret));
        }

        // Declare the function
        let func_id = self
            .module
            .declare_function(name, Linkage::Local, &sig)?;

        self.compiled_functions.insert(name.clone(), func_id);

        Ok(func_id)
    }

    /// Compiles a function body.
    fn compile_function(&mut self, func: &Function) -> Result<(), JitError> {
        let func_id = self.compiled_functions[&func.name.node];

        // Clear the context for reuse
        self.ctx.clear();

        // Pre-resolve parameter types (before borrowing builder_context)
        let param_types: Vec<_> = func.params.iter()
            .map(|param| {
                let ty = self.resolve_type(&param.ty.node);
                self.type_mapper.map_type(&ty)
            })
            .collect();

        let ret_ty = func
            .ret_type
            .as_ref()
            .map(|t| self.resolve_type(&t.node))
            .unwrap_or(ResolvedType::Unit);

        // Build function signature
        let mut sig = self.module.make_signature();
        for cl_ty in &param_types {
            sig.params.push(AbiParam::new(*cl_ty));
        }

        if !matches!(ret_ty, ResolvedType::Unit) {
            let cl_ret = self.type_mapper.map_type(&ret_ty);
            sig.returns.push(AbiParam::new(cl_ret));
        }

        self.ctx.func.signature = sig;

        // Create function builder
        let mut builder =
            FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Build variable map for parameters
        let mut variables: HashMap<String, Variable> = HashMap::new();
        let mut var_index = 0;

        for (i, (param, cl_ty)) in func.params.iter().zip(param_types.iter()).enumerate() {
            let var = Variable::new(var_index);
            var_index += 1;

            builder.declare_var(var, *cl_ty);

            let val = builder.block_params(entry_block)[i];
            builder.def_var(var, val);

            variables.insert(param.name.node.clone(), var);
        }

        // Compile the function body
        let mut compiler = FunctionCompiler {
            builder: &mut builder,
            module: &mut self.module,
            type_mapper: &self.type_mapper,
            variables,
            var_index,
            external_functions: &mut self.external_functions,
            compiled_functions: &self.compiled_functions,
        };

        let result = match &func.body {
            FunctionBody::Expr(expr) => compiler.compile_expr(&expr.node)?,
            FunctionBody::Block(stmts) => {
                let mut last_val = None;
                for (i, stmt) in stmts.iter().enumerate() {
                    // For the last statement, if it's an expression, capture its value
                    if i == stmts.len() - 1 {
                        if let Stmt::Expr(expr) = &stmt.node {
                            last_val = Some(compiler.compile_expr(&expr.node)?);
                        } else {
                            compiler.compile_stmt(&stmt.node)?;
                        }
                    } else {
                        compiler.compile_stmt(&stmt.node)?;
                    }
                }
                last_val.unwrap_or_else(|| builder.ins().iconst(types::I64, 0))
            }
        };

        // Return the result
        if !matches!(ret_ty, ResolvedType::Unit) {
            builder.ins().return_(&[result]);
        } else {
            builder.ins().return_(&[]);
        }

        // Finalize the function
        builder.finalize();

        // Define the function
        self.module.define_function(func_id, &mut self.ctx)?;

        Ok(())
    }

    /// Resolves a vais_ast::Type to a ResolvedType.
    fn resolve_type(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, generics } => {
                match name.as_str() {
                    "i8" => ResolvedType::I8,
                    "i16" => ResolvedType::I16,
                    "i32" => ResolvedType::I32,
                    "i64" => ResolvedType::I64,
                    "i128" => ResolvedType::I128,
                    "u8" => ResolvedType::U8,
                    "u16" => ResolvedType::U16,
                    "u32" => ResolvedType::U32,
                    "u64" => ResolvedType::U64,
                    "u128" => ResolvedType::U128,
                    "f32" => ResolvedType::F32,
                    "f64" => ResolvedType::F64,
                    "bool" => ResolvedType::Bool,
                    "str" => ResolvedType::Str,
                    _ => {
                        let resolved_generics: Vec<_> = generics
                            .iter()
                            .map(|g| self.resolve_type(&g.node))
                            .collect();
                        ResolvedType::Named {
                            name: name.clone(),
                            generics: resolved_generics,
                        }
                    }
                }
            }
            Type::Unit => ResolvedType::Unit,
            Type::Pointer(inner) => ResolvedType::Pointer(Box::new(self.resolve_type(&inner.node))),
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.resolve_type(&inner.node))),
            Type::RefMut(inner) => ResolvedType::RefMut(Box::new(self.resolve_type(&inner.node))),
            Type::Array(inner) => ResolvedType::Array(Box::new(self.resolve_type(&inner.node))),
            Type::Map(key, val) => ResolvedType::Map(
                Box::new(self.resolve_type(&key.node)),
                Box::new(self.resolve_type(&val.node)),
            ),
            Type::Tuple(elems) => {
                ResolvedType::Tuple(elems.iter().map(|e| self.resolve_type(&e.node)).collect())
            }
            Type::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.resolve_type(&inner.node)))
            }
            Type::Result(inner) => ResolvedType::Result(Box::new(self.resolve_type(&inner.node))),
            Type::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.resolve_type(&p.node)).collect(),
                ret: Box::new(self.resolve_type(&ret.node)),
            },
            Type::ConstArray { element, size } => {
                let resolved_element = self.resolve_type(&element.node);
                let resolved_size = self.resolve_const_expr(size);
                ResolvedType::ConstArray {
                    element: Box::new(resolved_element),
                    size: resolved_size,
                }
            }
            Type::Infer => ResolvedType::Unknown,
            Type::DynTrait { trait_name, generics } => {
                let resolved_generics: Vec<_> = generics
                    .iter()
                    .map(|g| self.resolve_type(&g.node))
                    .collect();
                ResolvedType::DynTrait {
                    trait_name: trait_name.clone(),
                    generics: resolved_generics,
                }
            }
        }
    }

    /// Resolves a const expression to a ResolvedConst.
    fn resolve_const_expr(&self, expr: &vais_ast::ConstExpr) -> vais_types::ResolvedConst {
        match expr {
            vais_ast::ConstExpr::Literal(n) => vais_types::ResolvedConst::Value(*n),
            vais_ast::ConstExpr::Param(name) => vais_types::ResolvedConst::Param(name.clone()),
            vais_ast::ConstExpr::BinOp { op, left, right } => {
                let resolved_left = self.resolve_const_expr(left);
                let resolved_right = self.resolve_const_expr(right);
                let resolved_op = match op {
                    vais_ast::ConstBinOp::Add => vais_types::ConstBinOp::Add,
                    vais_ast::ConstBinOp::Sub => vais_types::ConstBinOp::Sub,
                    vais_ast::ConstBinOp::Mul => vais_types::ConstBinOp::Mul,
                    vais_ast::ConstBinOp::Div => vais_types::ConstBinOp::Div,
                };
                vais_types::ResolvedConst::BinOp {
                    op: resolved_op,
                    left: Box::new(resolved_left),
                    right: Box::new(resolved_right),
                }
            }
        }
    }

    /// Clears compiled state for a new REPL session.
    pub fn clear(&mut self) -> Result<(), JitError> {
        // Create a new JIT module
        *self = Self::new()?;
        Ok(())
    }
}

/// Function-level compiler state.
struct FunctionCompiler<'a, 'b> {
    builder: &'a mut FunctionBuilder<'b>,
    module: &'a mut JITModule,
    type_mapper: &'a TypeMapper,
    variables: HashMap<String, Variable>,
    var_index: usize,
    external_functions: &'a mut HashMap<String, FuncId>,
    compiled_functions: &'a HashMap<String, FuncId>,
}

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    /// Compiles an expression and returns its Cranelift value.
    fn compile_expr(&mut self, expr: &Expr) -> Result<Value, JitError> {
        match expr {
            Expr::Int(n) => Ok(self.builder.ins().iconst(types::I64, *n)),

            Expr::Float(f) => Ok(self.builder.ins().f64const(*f)),

            Expr::Bool(b) => {
                let val = if *b { 1i64 } else { 0i64 };
                Ok(self.builder.ins().iconst(types::I8, val))
            }

            Expr::String(s) => {
                // Create a data section for the string
                self.compile_string_literal(s)
            }

            Expr::Unit => Ok(self.builder.ins().iconst(types::I8, 0)),

            Expr::Ident(name) => {
                if let Some(&var) = self.variables.get(name) {
                    Ok(self.builder.use_var(var))
                } else {
                    Err(JitError::FunctionNotFound(format!(
                        "Variable not found: {}",
                        name
                    )))
                }
            }

            Expr::Binary { op, left, right } => {
                let lhs = self.compile_expr(&left.node)?;
                let rhs = self.compile_expr(&right.node)?;
                self.compile_binary_op(*op, lhs, rhs)
            }

            Expr::Unary { op, expr } => {
                let val = self.compile_expr(&expr.node)?;
                self.compile_unary_op(*op, val)
            }

            Expr::Call { func, args } => {
                if let Expr::Ident(name) = &func.node {
                    self.compile_call(name, args)
                } else {
                    Err(JitError::Unsupported(
                        "Indirect function calls".to_string(),
                    ))
                }
            }

            Expr::If { cond, then, else_ } => {
                self.compile_if(&cond.node, then, else_.as_ref())
            }

            Expr::Block(stmts) => {
                let mut last_val = None;
                for (i, stmt) in stmts.iter().enumerate() {
                    if i == stmts.len() - 1 {
                        if let Stmt::Expr(expr) = &stmt.node {
                            last_val = Some(self.compile_expr(&expr.node)?);
                        } else if let Stmt::Return(Some(expr)) = &stmt.node {
                            return self.compile_expr(&expr.node);
                        } else {
                            self.compile_stmt(&stmt.node)?;
                        }
                    } else {
                        self.compile_stmt(&stmt.node)?;
                    }
                }
                Ok(last_val.unwrap_or_else(|| self.builder.ins().iconst(types::I64, 0)))
            }

            Expr::Ternary { cond, then, else_ } => {
                self.compile_ternary(&cond.node, &then.node, &else_.node)
            }

            Expr::Loop { body, .. } => self.compile_loop(body),

            _ => Err(JitError::Unsupported(format!(
                "Expression type: {:?}",
                std::mem::discriminant(expr)
            ))),
        }
    }

    /// Compiles a statement.
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), JitError> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let var = Variable::new(self.var_index);
                self.var_index += 1;

                // Declare variable with i64 type (default)
                self.builder.declare_var(var, types::I64);

                let val = self.compile_expr(&value.node)?;
                self.builder.def_var(var, val);

                self.variables.insert(name.node.clone(), var);
                Ok(())
            }

            Stmt::Expr(expr) => {
                self.compile_expr(&expr.node)?;
                Ok(())
            }

            Stmt::Return(Some(expr)) => {
                let val = self.compile_expr(&expr.node)?;
                self.builder.ins().return_(&[val]);
                Ok(())
            }

            Stmt::Return(None) => {
                self.builder.ins().return_(&[]);
                Ok(())
            }

            Stmt::Break(_) => {
                Err(JitError::Unsupported("break outside of loop context".to_string()))
            }

            Stmt::Continue => {
                Err(JitError::Unsupported("continue outside of loop context".to_string()))
            }

            Stmt::Defer(_) => {
                // Defer is not yet supported in JIT mode
                Err(JitError::Unsupported("defer not yet supported in JIT mode".to_string()))
            }
        }
    }

    /// Compiles a binary operation.
    fn compile_binary_op(
        &mut self,
        op: BinOp,
        lhs: Value,
        rhs: Value,
    ) -> Result<Value, JitError> {
        let result = match op {
            BinOp::Add => self.builder.ins().iadd(lhs, rhs),
            BinOp::Sub => self.builder.ins().isub(lhs, rhs),
            BinOp::Mul => self.builder.ins().imul(lhs, rhs),
            BinOp::Div => self.builder.ins().sdiv(lhs, rhs),
            BinOp::Mod => self.builder.ins().srem(lhs, rhs),
            BinOp::BitAnd => self.builder.ins().band(lhs, rhs),
            BinOp::BitOr => self.builder.ins().bor(lhs, rhs),
            BinOp::BitXor => self.builder.ins().bxor(lhs, rhs),
            BinOp::Shl => self.builder.ins().ishl(lhs, rhs),
            BinOp::Shr => self.builder.ins().sshr(lhs, rhs),
            BinOp::Eq => {
                let cmp = self.builder.ins().icmp(IntCC::Equal, lhs, rhs);
                self.builder.ins().uextend(types::I64, cmp)
            }
            BinOp::Neq => {
                let cmp = self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
                self.builder.ins().uextend(types::I64, cmp)
            }
            BinOp::Lt => {
                let cmp = self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
                self.builder.ins().uextend(types::I64, cmp)
            }
            BinOp::Lte => {
                let cmp = self
                    .builder
                    .ins()
                    .icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                self.builder.ins().uextend(types::I64, cmp)
            }
            BinOp::Gt => {
                let cmp = self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                self.builder.ins().uextend(types::I64, cmp)
            }
            BinOp::Gte => {
                let cmp = self
                    .builder
                    .ins()
                    .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                self.builder.ins().uextend(types::I64, cmp)
            }
            BinOp::And => {
                // Logical AND: convert to bool, AND, extend back
                let lhs_bool = self
                    .builder
                    .ins()
                    .icmp_imm(IntCC::NotEqual, lhs, 0);
                let rhs_bool = self
                    .builder
                    .ins()
                    .icmp_imm(IntCC::NotEqual, rhs, 0);
                let result = self.builder.ins().band(lhs_bool, rhs_bool);
                self.builder.ins().uextend(types::I64, result)
            }
            BinOp::Or => {
                // Logical OR: convert to bool, OR, extend back
                let lhs_bool = self
                    .builder
                    .ins()
                    .icmp_imm(IntCC::NotEqual, lhs, 0);
                let rhs_bool = self
                    .builder
                    .ins()
                    .icmp_imm(IntCC::NotEqual, rhs, 0);
                let result = self.builder.ins().bor(lhs_bool, rhs_bool);
                self.builder.ins().uextend(types::I64, result)
            }
        };
        Ok(result)
    }

    /// Compiles a unary operation.
    fn compile_unary_op(&mut self, op: UnaryOp, val: Value) -> Result<Value, JitError> {
        let result = match op {
            UnaryOp::Neg => self.builder.ins().ineg(val),
            UnaryOp::Not => {
                // Logical NOT: compare with 0, invert
                let is_zero = self.builder.ins().icmp_imm(IntCC::Equal, val, 0);
                self.builder.ins().uextend(types::I64, is_zero)
            }
            UnaryOp::BitNot => self.builder.ins().bnot(val),
        };
        Ok(result)
    }

    /// Compiles a function call.
    fn compile_call(&mut self, name: &str, args: &[Spanned<Expr>]) -> Result<Value, JitError> {
        // Compile arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.compile_expr(&arg.node)?);
        }

        // Check if it's a compiled function
        if let Some(&func_id) = self.compiled_functions.get(name) {
            let func_ref = self.module.declare_func_in_func(func_id, self.builder.func);
            let call = self.builder.ins().call(func_ref, &arg_values);
            let results = self.builder.inst_results(call);
            if results.is_empty() {
                Ok(self.builder.ins().iconst(types::I64, 0))
            } else {
                Ok(results[0])
            }
        }
        // Check if it's an external function
        else if let Some(&func_id) = self.external_functions.get(name) {
            let func_ref = self.module.declare_func_in_func(func_id, self.builder.func);
            let call = self.builder.ins().call(func_ref, &arg_values);
            let results = self.builder.inst_results(call);
            if results.is_empty() {
                Ok(self.builder.ins().iconst(types::I64, 0))
            } else {
                Ok(results[0])
            }
        }
        // Try to declare as external
        else {
            // Build signature based on common patterns
            let mut sig = self.module.make_signature();
            sig.returns.push(AbiParam::new(types::I64));
            for _ in args {
                sig.params.push(AbiParam::new(types::I64));
            }

            let func_id = self
                .module
                .declare_function(name, Linkage::Import, &sig)?;

            self.external_functions.insert(name.to_string(), func_id);

            let func_ref = self.module.declare_func_in_func(func_id, self.builder.func);
            let call = self.builder.ins().call(func_ref, &arg_values);
            let results = self.builder.inst_results(call);
            if results.is_empty() {
                Ok(self.builder.ins().iconst(types::I64, 0))
            } else {
                Ok(results[0])
            }
        }
    }

    /// Compiles an if expression.
    fn compile_if(
        &mut self,
        cond: &Expr,
        then_stmts: &[Spanned<Stmt>],
        else_expr: Option<&vais_ast::IfElse>,
    ) -> Result<Value, JitError> {
        let cond_val = self.compile_expr(cond)?;

        // Create blocks
        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let merge_block = self.builder.create_block();

        // Add block parameter for the result
        self.builder
            .append_block_param(merge_block, types::I64);

        // Branch based on condition
        let cond_bool = self
            .builder
            .ins()
            .icmp_imm(IntCC::NotEqual, cond_val, 0);
        self.builder
            .ins()
            .brif(cond_bool, then_block, &[], else_block, &[]);

        // Then block
        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);
        let then_val = self.compile_stmts_as_expr(then_stmts)?;
        self.builder.ins().jump(merge_block, &[then_val]);

        // Else block
        self.builder.switch_to_block(else_block);
        self.builder.seal_block(else_block);
        let else_val = if let Some(else_branch) = else_expr {
            self.compile_if_else(else_branch)?
        } else {
            self.builder.ins().iconst(types::I64, 0)
        };
        self.builder.ins().jump(merge_block, &[else_val]);

        // Merge block
        self.builder.switch_to_block(merge_block);
        self.builder.seal_block(merge_block);

        Ok(self.builder.block_params(merge_block)[0])
    }

    /// Compiles an if-else branch.
    fn compile_if_else(&mut self, if_else: &vais_ast::IfElse) -> Result<Value, JitError> {
        match if_else {
            vais_ast::IfElse::ElseIf(cond, stmts, next) => {
                self.compile_if(&cond.node, stmts, next.as_deref())
            }
            vais_ast::IfElse::Else(stmts) => self.compile_stmts_as_expr(stmts),
        }
    }

    /// Compiles a list of statements and returns the last expression value.
    fn compile_stmts_as_expr(&mut self, stmts: &[Spanned<Stmt>]) -> Result<Value, JitError> {
        let mut last_val = None;
        for (i, stmt) in stmts.iter().enumerate() {
            if i == stmts.len() - 1 {
                if let Stmt::Expr(expr) = &stmt.node {
                    last_val = Some(self.compile_expr(&expr.node)?);
                } else {
                    self.compile_stmt(&stmt.node)?;
                }
            } else {
                self.compile_stmt(&stmt.node)?;
            }
        }
        Ok(last_val.unwrap_or_else(|| self.builder.ins().iconst(types::I64, 0)))
    }

    /// Compiles a ternary expression.
    fn compile_ternary(
        &mut self,
        cond: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> Result<Value, JitError> {
        let cond_val = self.compile_expr(cond)?;

        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let merge_block = self.builder.create_block();

        self.builder.append_block_param(merge_block, types::I64);

        let cond_bool = self.builder.ins().icmp_imm(IntCC::NotEqual, cond_val, 0);
        self.builder.ins().brif(cond_bool, then_block, &[], else_block, &[]);

        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);
        let then_val = self.compile_expr(then_expr)?;
        self.builder.ins().jump(merge_block, &[then_val]);

        self.builder.switch_to_block(else_block);
        self.builder.seal_block(else_block);
        let else_val = self.compile_expr(else_expr)?;
        self.builder.ins().jump(merge_block, &[else_val]);

        self.builder.switch_to_block(merge_block);
        self.builder.seal_block(merge_block);

        Ok(self.builder.block_params(merge_block)[0])
    }

    /// Compiles a loop expression.
    /// Note: Infinite loops without break will not terminate.
    /// For now, we return a dummy value for loop expressions.
    fn compile_loop(&mut self, body: &[Spanned<Stmt>]) -> Result<Value, JitError> {
        let loop_block = self.builder.create_block();
        let exit_block = self.builder.create_block();
        self.builder.append_block_param(exit_block, types::I64);

        // Jump to loop
        self.builder.ins().jump(loop_block, &[]);

        // Loop body
        self.builder.switch_to_block(loop_block);

        // Compile body
        for stmt in body {
            self.compile_stmt(&stmt.node)?;
        }

        // Unconditional loop back
        self.builder.ins().jump(loop_block, &[]);

        self.builder.seal_block(loop_block);

        // Exit block (unreachable without break)
        self.builder.switch_to_block(exit_block);
        self.builder.seal_block(exit_block);

        Ok(self.builder.block_params(exit_block)[0])
    }

    /// Compiles a string literal to a data section reference.
    fn compile_string_literal(&mut self, s: &str) -> Result<Value, JitError> {
        use cranelift_module::DataDescription;

        // Create unique name for this string
        let name = format!("str_{}", s.len());

        // Create null-terminated string data
        let mut data = s.as_bytes().to_vec();
        data.push(0);

        // Create data description
        let mut desc = DataDescription::new();
        desc.define(data.into_boxed_slice());

        // Declare and define the data
        let data_id = self.module.declare_data(&name, Linkage::Local, false, false)?;
        self.module.define_data(data_id, &desc)?;

        // Get reference to data
        let local_data_id = self.module.declare_data_in_func(data_id, self.builder.func);
        let ptr = self.builder.ins().symbol_value(self.type_mapper.pointer_type(), local_data_id);

        Ok(ptr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_parser::parse;
    use vais_types::TypeChecker;

    fn compile_and_run(source: &str) -> Result<i64, JitError> {
        let ast = parse(source).map_err(|e| JitError::Cranelift(format!("Parse error: {}", e)))?;

        let mut checker = TypeChecker::new();
        checker
            .check_module(&ast)
            .map_err(|e| JitError::Type(e.to_string()))?;

        let mut jit = JitCompiler::new()?;
        jit.compile_and_run_main(&ast)
    }

    #[test]
    fn test_simple_return() {
        let result = compile_and_run("F main()->i64{42}").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_addition() {
        let result = compile_and_run("F main()->i64{1+2+3}").unwrap();
        assert_eq!(result, 6);
    }

    #[test]
    fn test_arithmetic() {
        let result = compile_and_run("F main()->i64{2*3+4}").unwrap();
        assert_eq!(result, 10);
    }

    #[test]
    fn test_subtraction() {
        let result = compile_and_run("F main()->i64{10-3}").unwrap();
        assert_eq!(result, 7);
    }

    #[test]
    fn test_multiplication() {
        let result = compile_and_run("F main()->i64{6*7}").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_division() {
        let result = compile_and_run("F main()->i64{100/10}").unwrap();
        assert_eq!(result, 10);
    }

    #[test]
    fn test_modulo() {
        let result = compile_and_run("F main()->i64{17%5}").unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_comparison_eq() {
        let result = compile_and_run("F main()->i64{I 5==5{1}E{0}}").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_comparison_lt() {
        let result = compile_and_run("F main()->i64{I 3<5{1}E{0}}").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_if_else() {
        let result = compile_and_run("F main()->i64{I true{42}E{0}}").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_if_false() {
        let result = compile_and_run("F main()->i64{I false{0}E{99}}").unwrap();
        assert_eq!(result, 99);
    }

    #[test]
    fn test_local_variable() {
        let result = compile_and_run("F main()->i64{x:=10;x+5}").unwrap();
        assert_eq!(result, 15);
    }

    #[test]
    fn test_multiple_variables() {
        let result = compile_and_run("F main()->i64{a:=3;b:=4;a*b}").unwrap();
        assert_eq!(result, 12);
    }

    #[test]
    fn test_function_call() {
        let result = compile_and_run(
            "F add(a:i64,b:i64)->i64{a+b} F main()->i64{add(3,4)}",
        )
        .unwrap();
        assert_eq!(result, 7);
    }

    #[test]
    fn test_nested_calls() {
        let result = compile_and_run(
            "F double(x:i64)->i64{x*2} F main()->i64{double(double(5))}",
        )
        .unwrap();
        assert_eq!(result, 20);
    }

    #[test]
    fn test_negation() {
        let result = compile_and_run("F main()->i64{0-42}").unwrap();
        assert_eq!(result, -42);
    }

    #[test]
    fn test_logical_and() {
        let result = compile_and_run("F main()->i64{I true&&true{1}E{0}}").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_logical_or() {
        let result = compile_and_run("F main()->i64{I false||true{1}E{0}}").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_bitwise_and() {
        let result = compile_and_run("F main()->i64{12&10}").unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_bitwise_or() {
        let result = compile_and_run("F main()->i64{12|3}").unwrap();
        assert_eq!(result, 15);
    }
}
