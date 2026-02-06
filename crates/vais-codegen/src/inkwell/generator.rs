//! Inkwell-based LLVM code generator.
//!
//! Provides type-safe LLVM IR generation using the inkwell crate.

use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, GlobalValue, IntValue,
    PointerValue,
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};

use vais_ast::{
    self as ast, BinOp, Expr, IfElse, Literal, MatchArm, Module as VaisModule, Pattern, Spanned,
    Stmt, StringInterpPart, Type, UnaryOp,
};
use vais_types::ResolvedType;

use super::builtins;
use super::types::TypeMapper;
use crate::{CodegenError, CodegenResult, TargetTriple};

/// Loop context for break/continue handling.
struct LoopContext<'ctx> {
    /// Block to jump to on break
    break_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// Block to jump to on continue
    continue_block: inkwell::basic_block::BasicBlock<'ctx>,
}

/// Closure information for captured variables
#[derive(Clone)]
#[allow(dead_code)]
struct ClosureInfo<'ctx> {
    /// The generated LLVM function
    func: FunctionValue<'ctx>,
    /// Captured variable names and their values (for passing to the lambda)
    captures: Vec<(String, BasicValueEnum<'ctx>)>,
}

/// LLVM code generator using inkwell.
///
/// This generator provides direct LLVM API access for type-safe
/// code generation, as opposed to text-based IR generation.
pub struct InkwellCodeGenerator<'ctx> {
    /// LLVM context - owns all LLVM objects
    context: &'ctx Context,

    /// Current LLVM module being built
    module: Module<'ctx>,

    /// IR builder for instruction generation
    builder: Builder<'ctx>,

    /// Type mapper for Vais -> LLVM type conversion
    type_mapper: TypeMapper<'ctx>,

    /// Target architecture
    #[allow(dead_code)]
    target: TargetTriple,

    /// Registered functions by name
    functions: HashMap<String, FunctionValue<'ctx>>,

    /// Local variables (alloca pointers and their types)
    locals: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,

    /// String constants
    string_constants: HashMap<String, GlobalValue<'ctx>>,
    string_counter: usize,

    /// Loop stack for break/continue
    loop_stack: Vec<LoopContext<'ctx>>,

    /// Label counter for unique block names
    label_counter: usize,

    /// Current function being compiled
    current_function: Option<FunctionValue<'ctx>>,

    /// Generic type substitutions
    generic_substitutions: HashMap<String, ResolvedType>,

    /// Generated struct types (for deduplication)
    generated_structs: HashMap<String, StructType<'ctx>>,

    /// Struct field names (struct name -> field names in order)
    struct_fields: HashMap<String, Vec<String>>,

    /// Lambda function counter for unique naming
    lambda_counter: usize,

    /// Lambda functions generated during expression compilation
    lambda_functions: Vec<FunctionValue<'ctx>>,

    /// Enum variant tags: maps (enum_name, variant_name) -> tag
    enum_variants: HashMap<(String, String), i32>,

    /// Variable name -> struct type name tracking (for method call resolution)
    var_struct_types: HashMap<String, String>,

    /// Struct name -> generic parameter names (for method generic substitution)
    struct_generic_params: HashMap<String, Vec<String>>,

    /// Lambda binding info: variable name -> (lambda function name, captured values)
    lambda_bindings: HashMap<String, (String, Vec<(String, BasicValueEnum<'ctx>)>)>,

    /// Temporary storage for the last generated lambda (used by Stmt::Let to track bindings)
    _last_lambda_info: Option<(String, Vec<(String, BasicValueEnum<'ctx>)>)>,

    /// Constants: name -> value (evaluated at compile time)
    constants: HashMap<String, BasicValueEnum<'ctx>>,

    /// Function name -> return struct type name (for struct type inference)
    function_return_structs: HashMap<String, String>,

    /// Defer stack: expressions to execute in LIFO order before function return
    defer_stack: Vec<Expr>,

    /// TCO state: when generating a tail-recursive function as a loop,
    /// this holds the parameter allocas and the loop header block for jumping back.
    tco_state: Option<TcoState<'ctx>>,
}

/// Tail Call Optimization state for loop-based tail recursion elimination.
struct TcoState<'ctx> {
    /// Parameter allocas (name -> alloca pointer) for updating params before looping back
    param_allocas: Vec<(String, PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
    /// The loop header block to branch back to
    loop_header: inkwell::basic_block::BasicBlock<'ctx>,
}

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Creates a new inkwell code generator.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self::new_with_target(context, module_name, TargetTriple::Native)
    }

    /// Creates a new inkwell code generator with specified target.
    pub fn new_with_target(
        context: &'ctx Context,
        module_name: &str,
        target: TargetTriple,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let type_mapper = TypeMapper::new(context);

        // Set target triple if not native
        if target != TargetTriple::Native {
            module.set_triple(&inkwell::targets::TargetTriple::create(target.triple_str()));
        }

        let gen = Self {
            context,
            module,
            builder,
            type_mapper,
            target,
            functions: HashMap::new(),
            locals: HashMap::new(),
            string_constants: HashMap::new(),
            string_counter: 0,
            loop_stack: Vec::new(),
            label_counter: 0,
            current_function: None,
            generic_substitutions: HashMap::new(),
            generated_structs: HashMap::new(),
            struct_fields: HashMap::new(),
            lambda_counter: 0,
            lambda_functions: Vec::new(),
            enum_variants: HashMap::new(),
            var_struct_types: HashMap::new(),
            struct_generic_params: HashMap::new(),
            lambda_bindings: HashMap::new(),
            _last_lambda_info: None,
            constants: HashMap::new(),
            function_return_structs: HashMap::new(),
            defer_stack: Vec::new(),
            tco_state: None,
        };

        // Declare built-in functions
        builtins::declare_builtins(context, &gen.module);

        gen
    }

    /// Generates code for an entire Vais module.
    pub fn generate_module(&mut self, vais_module: &VaisModule) -> CodegenResult<()> {
        // First pass: collect all function signatures, struct definitions, enum definitions, and extern blocks
        for item in &vais_module.items {
            match &item.node {
                ast::Item::Function(func) => {
                    self.declare_function(func)?;
                }
                ast::Item::Struct(s) => {
                    self.define_struct(s)?;
                }
                ast::Item::Enum(e) => {
                    self.define_enum(e)?;
                }
                ast::Item::ExternBlock(extern_block) => {
                    self.declare_extern_block(extern_block)?;
                }
                ast::Item::Union(u) => {
                    self.define_union(u)?;
                }
                ast::Item::Const(const_def) => {
                    self.define_const(const_def)?;
                }
                _ => {}
            }
        }

        // Second pass: declare methods from Impl blocks and struct inline methods
        for item in &vais_module.items {
            match &item.node {
                ast::Item::Impl(impl_block) => {
                    if let Some(type_name) = Self::get_impl_type_name(&impl_block.target_type.node)
                    {
                        for method in &impl_block.methods {
                            self.declare_method(&type_name, &method.node)?;
                        }
                    }
                }
                ast::Item::Struct(s) => {
                    let type_name = s.name.node.clone();
                    for method in &s.methods {
                        self.declare_method(&type_name, &method.node)?;
                    }
                }
                _ => {}
            }
        }

        // Third pass: generate function bodies and method bodies
        for item in &vais_module.items {
            match &item.node {
                ast::Item::Function(func) => {
                    self.generate_function(func)?;
                }
                ast::Item::Impl(impl_block) => {
                    if let Some(type_name) = Self::get_impl_type_name(&impl_block.target_type.node)
                    {
                        for method in &impl_block.methods {
                            self.generate_method(&type_name, &method.node)?;
                        }
                    }
                }
                ast::Item::Struct(s) => {
                    let type_name = s.name.node.clone();
                    for method in &s.methods {
                        self.generate_method(&type_name, &method.node)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Returns the generated LLVM module.
    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Returns the LLVM IR as a string.
    pub fn get_ir_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Writes the LLVM IR to a file.
    pub fn write_to_file(&self, path: &std::path::Path) -> Result<(), String> {
        self.module.print_to_file(path).map_err(|e| e.to_string())
    }

    // ========== Declaration Phase ==========

    fn declare_function(&mut self, func: &ast::Function) -> CodegenResult<FunctionValue<'ctx>> {
        // For generic functions, set up default substitutions (T -> i64, etc.)
        let old_substitutions = self.generic_substitutions.clone();
        if !func.generics.is_empty() {
            for gp in &func.generics {
                self.generic_substitutions
                    .entry(gp.name.node.clone())
                    .or_insert(ResolvedType::I64);
            }
        }

        let ret_resolved = func
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);
        let ret_substituted = self.substitute_type(&ret_resolved);

        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| {
                let resolved = self.ast_type_to_resolved(&p.ty.node);
                // If param type is inferred (defaults to I64) but return type is F64,
                // infer param as F64 too (for simple arithmetic functions)
                let resolved = if matches!(p.ty.node, Type::Infer)
                    && matches!(ret_substituted, ResolvedType::F64 | ResolvedType::F32)
                {
                    ret_substituted.clone()
                } else {
                    resolved
                };
                let substituted = self.substitute_type(&resolved);
                self.type_mapper.map_type(&substituted).into()
            })
            .collect();

        let fn_type = if ret_substituted == ResolvedType::Unit {
            self.context.void_type().fn_type(&param_types, false)
        } else {
            let ret_type = self.type_mapper.map_type(&ret_substituted);
            ret_type.fn_type(&param_types, false)
        };

        // Restore substitutions
        self.generic_substitutions = old_substitutions;

        let fn_value = self.module.add_function(&func.name.node, fn_type, None);
        self.functions.insert(func.name.node.clone(), fn_value);

        // Track return struct type
        if let Some(ret_ty) = &func.ret_type {
            if let Some(sn) = self.extract_struct_type_name(&ret_ty.node) {
                self.function_return_structs
                    .insert(func.name.node.clone(), sn);
            }
        }

        Ok(fn_value)
    }

    fn define_struct(&mut self, s: &ast::Struct) -> CodegenResult<StructType<'ctx>> {
        // For generic structs, set up substitutions (default generic params to i64)
        let old_substitutions = self.generic_substitutions.clone();
        if !s.generics.is_empty() {
            let gp_names: Vec<String> = s.generics.iter().map(|gp| gp.name.node.clone()).collect();
            self.struct_generic_params
                .insert(s.name.node.clone(), gp_names);
            for gp in &s.generics {
                self.generic_substitutions
                    .entry(gp.name.node.clone())
                    .or_insert(ResolvedType::I64);
            }
        }

        let field_types: Vec<BasicTypeEnum> = s
            .fields
            .iter()
            .map(|f| {
                let resolved = self.ast_type_to_resolved(&f.ty.node);
                let substituted = self.substitute_type(&resolved);
                self.type_mapper.map_type(&substituted)
            })
            .collect();

        // Restore substitutions
        self.generic_substitutions = old_substitutions;

        // Store field names for later field access lookups
        let field_names: Vec<String> = s.fields.iter().map(|f| f.name.node.clone()).collect();
        let name = s.name.node.clone();
        self.struct_fields.insert(name.clone(), field_names);

        let struct_type = self.context.struct_type(&field_types, false);
        self.type_mapper.register_struct(&name, struct_type);
        self.generated_structs.insert(name, struct_type);

        Ok(struct_type)
    }

    fn define_enum(&mut self, e: &ast::Enum) -> CodegenResult<StructType<'ctx>> {
        // Enums are represented as tagged unions: { tag: i8, data: max_variant_size }
        // For simplicity, use { i8, i64 } to hold any primitive
        let tag_type = self.context.i8_type();
        let data_type = self.context.i64_type();
        let enum_type = self
            .context
            .struct_type(&[tag_type.into(), data_type.into()], false);

        // Register variant tags: each variant gets a sequential tag starting from 0
        let enum_name = e.name.node.clone();
        for (tag, variant) in e.variants.iter().enumerate() {
            self.enum_variants
                .insert((enum_name.clone(), variant.name.node.clone()), tag as i32);
        }

        self.type_mapper.register_struct(&e.name.node, enum_type);
        self.generated_structs.insert(enum_name, enum_type);

        Ok(enum_type)
    }

    fn declare_extern_block(&mut self, extern_block: &ast::ExternBlock) -> CodegenResult<()> {
        for func in &extern_block.functions {
            let fn_name = func.name.node.clone();

            // Skip if already declared (e.g., by builtins)
            if self.functions.contains_key(&fn_name) || self.module.get_function(&fn_name).is_some()
            {
                continue;
            }

            let param_types: Vec<BasicMetadataTypeEnum> = func
                .params
                .iter()
                .map(|p| {
                    let resolved = self.ast_type_to_resolved(&p.ty.node);
                    let substituted = self.substitute_type(&resolved);
                    self.type_mapper.map_type(&substituted).into()
                })
                .collect();

            let ret_resolved = func
                .ret_type
                .as_ref()
                .map(|t| self.ast_type_to_resolved(&t.node))
                .unwrap_or(ResolvedType::Unit);
            let ret_substituted = self.substitute_type(&ret_resolved);

            let fn_type = if ret_substituted == ResolvedType::Unit {
                self.context
                    .void_type()
                    .fn_type(&param_types, func.is_vararg)
            } else {
                let ret_type = self.type_mapper.map_type(&ret_substituted);
                ret_type.fn_type(&param_types, func.is_vararg)
            };

            let fn_value = self.module.add_function(&fn_name, fn_type, None);
            self.functions.insert(fn_name, fn_value);
        }
        Ok(())
    }

    fn define_union(&mut self, u: &ast::Union) -> CodegenResult<StructType<'ctx>> {
        // Union: all fields share memory - size = max field size
        // Represent as a struct with a single i64 field (simplification)
        let union_type = self
            .context
            .struct_type(&[self.context.i64_type().into()], false);

        let field_names: Vec<String> = u.fields.iter().map(|f| f.name.node.clone()).collect();
        let name = u.name.node.clone();
        self.struct_fields.insert(name.clone(), field_names);
        self.type_mapper.register_struct(&name, union_type);
        self.generated_structs.insert(name, union_type);

        Ok(union_type)
    }

    fn define_const(&mut self, const_def: &ast::ConstDef) -> CodegenResult<()> {
        // Evaluate constant value at compile time
        // For simple literals, we can directly create LLVM values
        let val = self.evaluate_const_expr(&const_def.value.node)?;
        self.constants.insert(const_def.name.node.clone(), val);
        Ok(())
    }

    /// Evaluates a constant expression at compile time.
    fn evaluate_const_expr(&self, expr: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Int(n) => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
            Expr::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            Expr::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Expr::String(s) => {
                // Create a global string constant
                let string_value = self.context.const_string(s.as_bytes(), true);
                let name = format!(".const_str.{}", self.string_counter);
                let global = self.module.add_global(
                    string_value.get_type(),
                    Some(AddressSpace::default()),
                    &name,
                );
                global.set_initializer(&string_value);
                global.set_constant(true);
                Ok(global.as_pointer_value().into())
            }
            Expr::Unary {
                op: UnaryOp::Neg,
                expr: inner,
            } => {
                let val = self.evaluate_const_expr(&inner.node)?;
                if val.is_int_value() {
                    Ok(self
                        .context
                        .i64_type()
                        .const_int(
                            (-(val
                                .into_int_value()
                                .get_sign_extended_constant()
                                .unwrap_or(0))) as u64,
                            true,
                        )
                        .into())
                } else if val.is_float_value() {
                    Ok(self
                        .context
                        .f64_type()
                        .const_float(
                            -val.into_float_value()
                                .get_constant()
                                .map(|(f, _)| f)
                                .unwrap_or(0.0),
                        )
                        .into())
                } else {
                    Ok(self.context.i64_type().const_int(0, false).into())
                }
            }
            _ => {
                // Default: return 0 for unsupported const expressions
                Ok(self.context.i64_type().const_int(0, false).into())
            }
        }
    }

    // ========== Code Generation Phase ==========

    /// Check if an expression contains a SelfCall (@) in tail position.
    /// Tail position means the result of the SelfCall is directly returned
    /// without any further computation.
    fn has_tail_self_call(expr: &Expr) -> bool {
        match expr {
            // Direct self-call: @(args) - this IS a tail call
            Expr::Call { func, .. } if matches!(&func.node, Expr::SelfCall) => true,
            // Ternary: cond ? then : else - check both branches
            Expr::Ternary { then, else_, .. } => {
                Self::has_tail_self_call(&then.node) || Self::has_tail_self_call(&else_.node)
            }
            // If expression: check then and else branches
            Expr::If { then, else_, .. } => {
                // Check last statement of then block
                let then_tail = then.last().is_some_and(|s| {
                    if let Stmt::Expr(e) = &s.node {
                        Self::has_tail_self_call(&e.node)
                    } else {
                        false
                    }
                });
                let else_tail = else_.as_ref().is_some_and(Self::if_else_has_tail);
                then_tail || else_tail
            }
            // Match expression: check arms
            Expr::Match { arms, .. } => arms
                .iter()
                .any(|arm| Self::has_tail_self_call(&arm.body.node)),
            // Block: check last expression
            Expr::Block(stmts) => stmts.last().is_some_and(|s| {
                if let Stmt::Expr(e) = &s.node {
                    Self::has_tail_self_call(&e.node)
                } else {
                    false
                }
            }),
            _ => false,
        }
    }

    fn if_else_has_tail(ie: &IfElse) -> bool {
        match ie {
            IfElse::Else(stmts) => stmts.last().is_some_and(|s| {
                if let Stmt::Expr(e) = &s.node {
                    Self::has_tail_self_call(&e.node)
                } else {
                    false
                }
            }),
            IfElse::ElseIf(_, then, else_) => {
                let then_tail = then.last().is_some_and(|s| {
                    if let Stmt::Expr(e) = &s.node {
                        Self::has_tail_self_call(&e.node)
                    } else {
                        false
                    }
                });
                let else_tail = else_.as_ref().is_some_and(|ie| Self::if_else_has_tail(ie));
                then_tail || else_tail
            }
        }
    }

    /// Generate a tail-recursive function body as a loop.
    /// Instead of recursive calls, we update the parameters and branch back to the loop header.
    fn generate_tco_function(&mut self, func: &ast::Function) -> CodegenResult<()> {
        let fn_value = *self
            .functions
            .get(&func.name.node)
            .ok_or_else(|| CodegenError::UndefinedFunction(func.name.node.clone()))?;

        self.current_function = Some(fn_value);
        self.locals.clear();
        self.var_struct_types.clear();
        self.defer_stack.clear();

        let old_substitutions = self.generic_substitutions.clone();
        if !func.generics.is_empty() {
            for gp in &func.generics {
                self.generic_substitutions
                    .entry(gp.name.node.clone())
                    .or_insert(ResolvedType::I64);
            }
        }

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate space for parameters (these will be updated on each loop iteration)
        let mut param_allocas = Vec::new();
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();
            let param_type = param_value.get_type();
            let alloca = self
                .builder
                .build_alloca(param_type, &param.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals
                .insert(param.name.node.clone(), (alloca, param_type));
            param_allocas.push((param.name.node.clone(), alloca, param_type));

            if let Some(struct_name) = self.extract_struct_type_name(&param.ty.node) {
                self.var_struct_types
                    .insert(param.name.node.clone(), struct_name);
            }
        }

        // Create loop header block (jump target for tail calls)
        let loop_header = self.context.append_basic_block(fn_value, "tco_loop");
        self.builder
            .build_unconditional_branch(loop_header)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder.position_at_end(loop_header);

        // Set TCO state so generate_call knows to emit loop-back instead of recursive call
        self.tco_state = Some(TcoState {
            param_allocas: param_allocas.clone(),
            loop_header,
        });

        // Generate function body
        let ret_resolved = func
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);
        let ret_substituted = self.substitute_type(&ret_resolved);

        match &func.body {
            ast::FunctionBody::Expr(body_expr) => {
                let body_value = self.generate_expr(&body_expr.node)?;
                // Only return if we haven't already (tail call branches back)
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        self.builder
                            .build_return(Some(&body_value))
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }
            }
            ast::FunctionBody::Block(stmts) => {
                let body_value = self.generate_block(stmts)?;
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let body_type_matches =
                            expected_ret_type.is_some_and(|ert| ert == body_value.get_type());
                        if body_type_matches {
                            self.builder
                                .build_return(Some(&body_value))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else if let Some(ert) = expected_ret_type {
                            let default_val = self.get_default_value(ert);
                            self.builder
                                .build_return(Some(&default_val))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else {
                            self.builder
                                .build_return(None)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }
                }
            }
        }

        // Clear TCO state
        self.tco_state = None;
        self.generic_substitutions = old_substitutions;
        self.current_function = None;
        Ok(())
    }

    fn generate_function(&mut self, func: &ast::Function) -> CodegenResult<()> {
        // Check if this function has tail-recursive self-calls
        let is_tail_recursive = match &func.body {
            ast::FunctionBody::Expr(body_expr) => Self::has_tail_self_call(&body_expr.node),
            ast::FunctionBody::Block(stmts) => stmts.last().is_some_and(|s| {
                if let Stmt::Expr(e) = &s.node {
                    Self::has_tail_self_call(&e.node)
                } else {
                    false
                }
            }),
        };

        if is_tail_recursive {
            return self.generate_tco_function(func);
        }

        let fn_value = *self
            .functions
            .get(&func.name.node)
            .ok_or_else(|| CodegenError::UndefinedFunction(func.name.node.clone()))?;

        self.current_function = Some(fn_value);
        self.locals.clear();
        self.var_struct_types.clear();
        self.defer_stack.clear();

        // For generic functions, set up default substitutions (T -> i64, etc.)
        let old_substitutions = self.generic_substitutions.clone();
        if !func.generics.is_empty() {
            for gp in &func.generics {
                self.generic_substitutions
                    .entry(gp.name.node.clone())
                    .or_insert(ResolvedType::I64);
            }
        }

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate space for parameters
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();
            // Use the actual LLVM parameter type from the declared function
            let param_type = param_value.get_type();
            let alloca = self
                .builder
                .build_alloca(param_type, &param.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals
                .insert(param.name.node.clone(), (alloca, param_type));

            // Track struct type for parameters
            if let Some(struct_name) = self.extract_struct_type_name(&param.ty.node) {
                self.var_struct_types
                    .insert(param.name.node.clone(), struct_name);
            }
        }

        // Generate contract checks (#[requires] attributes)
        for (idx, attr) in func.attributes.iter().enumerate() {
            if attr.name == "requires" {
                if let Some(expr) = &attr.expr {
                    let cond_val = self.generate_expr(&expr.node)?;
                    // Convert condition to i1 (bool)
                    let cond_i1 = if cond_val.is_int_value() {
                        self.builder
                            .build_int_compare(
                                IntPredicate::NE,
                                cond_val.into_int_value(),
                                cond_val.get_type().into_int_type().const_zero(),
                                "contract_cond",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    } else {
                        // Non-int condition: treat as truthy
                        self.context.bool_type().const_int(1, false)
                    };

                    let ok_block = self
                        .context
                        .append_basic_block(fn_value, &format!("contract_ok_{}", idx));
                    let fail_block = self
                        .context
                        .append_basic_block(fn_value, &format!("contract_fail_{}", idx));

                    self.builder
                        .build_conditional_branch(cond_i1, ok_block, fail_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Fail block: write message to stderr and exit
                    self.builder.position_at_end(fail_block);
                    let msg = format!("requires condition #{}", idx);
                    let msg_val = self.generate_string_literal(&msg)?;
                    // Write to stderr (fd=2) using write()
                    let write_fn = self.module.get_function("write").unwrap_or_else(|| {
                        self.module.add_function(
                            "write",
                            self.context.i64_type().fn_type(
                                &[
                                    self.context.i32_type().into(),
                                    self.context
                                        .i8_type()
                                        .ptr_type(AddressSpace::default())
                                        .into(),
                                    self.context.i64_type().into(),
                                ],
                                false,
                            ),
                            None,
                        )
                    });
                    let msg_len = self.context.i64_type().const_int(msg.len() as u64, false);
                    self.builder
                        .build_call(
                            write_fn,
                            &[
                                self.context.i32_type().const_int(2, false).into(),
                                msg_val.into(),
                                msg_len.into(),
                            ],
                            "contract_write",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    // Write newline
                    let newline = self.generate_string_literal("\n")?;
                    self.builder
                        .build_call(
                            write_fn,
                            &[
                                self.context.i32_type().const_int(2, false).into(),
                                newline.into(),
                                self.context.i64_type().const_int(1, false).into(),
                            ],
                            "contract_nl",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    let exit_fn = self.module.get_function("exit").unwrap_or_else(|| {
                        self.module.add_function(
                            "exit",
                            self.context
                                .void_type()
                                .fn_type(&[self.context.i32_type().into()], false),
                            None,
                        )
                    });
                    self.builder
                        .build_call(
                            exit_fn,
                            &[self.context.i32_type().const_int(1, false).into()],
                            "",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.builder
                        .build_unreachable()
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Continue in OK block
                    self.builder.position_at_end(ok_block);
                }
            }
        }

        // Generate function body
        let ret_resolved = func
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);
        let ret_substituted = self.substitute_type(&ret_resolved);

        match &func.body {
            ast::FunctionBody::Expr(body_expr) => {
                let body_value = self.generate_expr(&body_expr.node)?;
                self.emit_defer_cleanup()?;
                if ret_substituted == ResolvedType::Unit {
                    self.builder
                        .build_return(None)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    self.builder
                        .build_return(Some(&body_value))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
            }
            ast::FunctionBody::Block(stmts) => {
                let body_value = self.generate_block(stmts)?;
                // Only add return if the block doesn't already have a terminator
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        // Check if body value type matches expected return type
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let body_type_matches =
                            expected_ret_type.is_some_and(|ert| ert == body_value.get_type());
                        if body_type_matches {
                            self.builder
                                .build_return(Some(&body_value))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else if let Some(ert) = expected_ret_type {
                            // Type mismatch: return a default value of the expected type
                            let default_val = self.get_default_value(ert);
                            self.builder
                                .build_return(Some(&default_val))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else {
                            self.builder
                                .build_return(None)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }
                }
            }
        }

        // Restore generic substitutions
        self.generic_substitutions = old_substitutions;
        self.current_function = None;
        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        match expr {
            // Literals
            Expr::Int(n) => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
            Expr::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            Expr::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Expr::String(s) => self.generate_string_literal(s),
            Expr::Unit => Ok(self.context.struct_type(&[], false).const_zero().into()),

            // Variable
            Expr::Ident(name) => self.generate_var(name),

            // Binary/Unary operations
            Expr::Binary { op, left, right } => self.generate_binary(*op, &left.node, &right.node),
            Expr::Unary { op, expr: operand } => self.generate_unary(*op, &operand.node),

            // Function call
            Expr::Call { func, args } => self.generate_call(&func.node, args),

            // Block
            Expr::Block(stmts) => self.generate_block(stmts),

            // Control flow
            Expr::If { cond, then, else_ } => {
                self.generate_if_expr(&cond.node, then, else_.as_ref())
            }
            Expr::Loop {
                pattern,
                iter,
                body,
            } => self.generate_loop(pattern.as_ref(), iter.as_deref(), body),
            Expr::While { condition, body } => self.generate_while_loop(condition, body),
            Expr::Match {
                expr: match_expr,
                arms,
            } => self.generate_match(match_expr, arms),

            // Struct
            Expr::StructLit { name, fields } => self.generate_struct_literal(&name.node, fields),
            Expr::Field { expr: obj, field } => self.generate_field_access(&obj.node, &field.node),

            // Array/Tuple/Index
            Expr::Array(elements) => self.generate_array(elements),
            Expr::MapLit(_pairs) => {
                // Map literals not yet supported in inkwell backend
                Ok(self.context.i64_type().const_int(0, false).into())
            }
            Expr::Tuple(elements) => self.generate_tuple(elements),
            Expr::Index { expr: arr, index } => {
                // Check if this is a slice operation (index is a Range expression)
                if let Expr::Range {
                    start,
                    end,
                    inclusive,
                } = &index.node
                {
                    return self.generate_slice(
                        &arr.node,
                        start.as_deref(),
                        end.as_deref(),
                        *inclusive,
                    );
                }
                self.generate_index(&arr.node, &index.node)
            }

            // Method call
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.generate_method_call(&receiver.node, &method.node, args),

            // Lambda/Closure
            Expr::Lambda {
                params,
                body,
                captures,
            } => self.generate_lambda(params, &body.node, captures),

            // Try/Unwrap
            Expr::Try(inner) => self.generate_try(&inner.node),
            Expr::Unwrap(inner) => self.generate_unwrap(&inner.node),

            // Assignment
            Expr::Assign { target, value } => self.generate_assign(&target.node, &value.node),
            Expr::AssignOp { op, target, value } => {
                self.generate_assign_op(*op, &target.node, &value.node)
            }

            // Reference/Dereference
            Expr::Ref(inner) => {
                // Get address of inner expression (lvalue)
                match &inner.node {
                    Expr::Ident(name) => {
                        if let Some((ptr, _)) = self.locals.get(name) {
                            Ok((*ptr).into())
                        } else {
                            let val = self.generate_expr(&inner.node)?;
                            Ok(val)
                        }
                    }
                    _ => {
                        // For non-lvalue expressions, create a temporary alloca
                        let val = self.generate_expr(&inner.node)?;
                        let alloca = self
                            .builder
                            .build_alloca(val.get_type(), "ref_tmp")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.builder
                            .build_store(alloca, val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(alloca.into())
                    }
                }
            }
            Expr::Deref(inner) => {
                let ptr = self.generate_expr(&inner.node)?;
                let ptr_val = ptr.into_pointer_value();
                self.builder
                    .build_load(self.context.i64_type(), ptr_val, "deref")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
            }

            // Type cast
            Expr::Cast {
                expr: cast_expr,
                ty: cast_ty,
            } => {
                let val = self.generate_expr(&cast_expr.node)?;
                let target_resolved = self.ast_type_to_resolved(&cast_ty.node);
                let target_type = self.type_mapper.map_type(&target_resolved);

                // Perform actual type conversions
                if val.is_int_value() && target_type.is_float_type() {
                    // i64 -> f64
                    let result = self
                        .builder
                        .build_signed_int_to_float(
                            val.into_int_value(),
                            target_type.into_float_type(),
                            "cast_itof",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else if val.is_float_value() && target_type.is_int_type() {
                    // f64 -> i64
                    let result = self
                        .builder
                        .build_float_to_signed_int(
                            val.into_float_value(),
                            target_type.into_int_type(),
                            "cast_ftoi",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else if val.is_int_value() && target_type.is_int_type() {
                    let src_width = val.into_int_value().get_type().get_bit_width();
                    let dst_width = target_type.into_int_type().get_bit_width();
                    if src_width < dst_width {
                        let result = self
                            .builder
                            .build_int_s_extend(
                                val.into_int_value(),
                                target_type.into_int_type(),
                                "cast_sext",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(result.into())
                    } else if src_width > dst_width {
                        let result = self
                            .builder
                            .build_int_truncate(
                                val.into_int_value(),
                                target_type.into_int_type(),
                                "cast_trunc",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(result.into())
                    } else {
                        Ok(val)
                    }
                } else if val.is_int_value() && target_type.is_pointer_type() {
                    // i64 -> ptr
                    let result = self
                        .builder
                        .build_int_to_ptr(
                            val.into_int_value(),
                            target_type.into_pointer_type(),
                            "cast_itoptr",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else if val.is_pointer_value() && target_type.is_int_type() {
                    // ptr -> i64
                    let result = self
                        .builder
                        .build_ptr_to_int(
                            val.into_pointer_value(),
                            target_type.into_int_type(),
                            "cast_ptrtoi",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else {
                    // Same type or unsupported cast - return as-is
                    Ok(val)
                }
            }

            // Range
            Expr::Range {
                start,
                end,
                inclusive,
            } => self.generate_range(start.as_deref(), end.as_deref(), *inclusive),

            // Ternary
            Expr::Ternary { cond, then, else_ } => {
                self.generate_ternary(&cond.node, &then.node, &else_.node)
            }

            // Assert: evaluate condition, abort if false
            Expr::Assert { condition, message } => {
                self.generate_assert(&condition.node, message.as_deref())
            }

            // Comptime: evaluate at compile time (for now, just evaluate normally)
            Expr::Comptime { body } => self.generate_expr(&body.node),

            // Lazy: evaluate expression lazily (for now, just evaluate eagerly)
            Expr::Lazy(inner) => self.generate_expr(&inner.node),

            // Await: for now, just evaluate the inner expression
            Expr::Await(inner) => self.generate_expr(&inner.node),

            // Static method call: Type::method(args)
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                // Look up function as TypeName_method or just method
                let fn_name = format!("{}_{}", type_name.node, method.node);
                if self.functions.contains_key(&fn_name)
                    || self.module.get_function(&fn_name).is_some()
                {
                    let callee = Expr::Ident(fn_name);
                    self.generate_call(&callee, args)
                } else {
                    // Try just the method name
                    let callee = Expr::Ident(method.node.clone());
                    self.generate_call(&callee, args)
                }
            }

            // String interpolation
            Expr::StringInterp(parts) => self.generate_string_interp(parts),

            // Assume: compiler assumption for verification (no-op at runtime)
            Expr::Assume(inner) => {
                // Evaluate the inner expression but discard result
                let _ = self.generate_expr(&inner.node)?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }

            // Spread: just evaluate the inner expression
            Expr::Spread(inner) => self.generate_expr(&inner.node),

            // Force: evaluate a lazy value
            Expr::Force(inner) => self.generate_expr(&inner.node),

            // Spawn: for now just evaluate inner
            Expr::Spawn(inner) => self.generate_expr(&inner.node),

            // SelfCall (@): recursive call to current function
            Expr::SelfCall => {
                // Return a reference to the current function (for indirect calls)
                if let Some(func) = self.current_function {
                    Ok(func.as_global_value().as_pointer_value().into())
                } else {
                    Err(CodegenError::Unsupported(
                        "SelfCall outside of function".to_string(),
                    ))
                }
            }

            _ => Err(CodegenError::Unsupported(format!(
                "Expression kind not yet implemented: {:?}",
                expr
            ))),
        }
    }

    #[allow(dead_code)]
    fn generate_literal(&mut self, lit: &Literal) -> CodegenResult<BasicValueEnum<'ctx>> {
        match lit {
            Literal::Int(n) => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
            Literal::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            Literal::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Literal::String(s) => self.generate_string_literal(s),
        }
    }

    fn generate_string_literal(&mut self, s: &str) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Check if we already have this string
        if let Some(global) = self.string_constants.get(s) {
            let ptr = self
                .builder
                .build_pointer_cast(
                    global.as_pointer_value(),
                    self.context.i8_type().ptr_type(AddressSpace::default()),
                    "str_ptr",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(ptr.into());
        }

        // Create new global string constant
        let name = format!(".str.{}", self.string_counter);
        self.string_counter += 1;

        let string_value = self.context.const_string(s.as_bytes(), true);
        let global = self.module.add_global(
            string_value.get_type(),
            Some(AddressSpace::default()),
            &name,
        );
        global.set_initializer(&string_value);
        global.set_constant(true);

        self.string_constants.insert(s.to_string(), global);

        let ptr = self
            .builder
            .build_pointer_cast(
                global.as_pointer_value(),
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "str_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(ptr.into())
    }

    fn generate_var(&mut self, name: &str) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Handle None as a built-in value: { i8 tag=0, i64 data=0 }
        if name == "None" {
            let enum_type = self.context.struct_type(
                &[
                    self.context.i8_type().into(),
                    self.context.i64_type().into(),
                ],
                false,
            );
            let mut val = enum_type.get_undef();
            val = self
                .builder
                .build_insert_value(
                    val,
                    self.context.i8_type().const_int(0, false),
                    0,
                    "none_tag",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_struct_value();
            val = self
                .builder
                .build_insert_value(
                    val,
                    self.context.i64_type().const_int(0, false),
                    1,
                    "none_data",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_struct_value();
            return Ok(val.into());
        }

        let result = self.locals.get(name);

        if let Some((ptr, var_type)) = result {
            let value = self
                .builder
                .build_load(*var_type, *ptr, name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(value)
        } else if let Some(val) = self.constants.get(name) {
            // Check constants
            Ok(*val)
        } else {
            // Check if this is an enum variant name (e.g., Red, Green, Blue)
            for ((_, v_name), tag) in &self.enum_variants {
                if v_name == name {
                    let enum_type = self.context.struct_type(
                        &[
                            self.context.i8_type().into(),
                            self.context.i64_type().into(),
                        ],
                        false,
                    );
                    let mut val = enum_type.get_undef();
                    val = self
                        .builder
                        .build_insert_value(
                            val,
                            self.context.i8_type().const_int(*tag as u64, false),
                            0,
                            "variant_tag",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        .into_struct_value();
                    val = self
                        .builder
                        .build_insert_value(
                            val,
                            self.context.i64_type().const_int(0, false),
                            1,
                            "variant_data",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        .into_struct_value();
                    return Ok(val.into());
                }
            }

            // Check if this is a function name (function reference as variable)
            if let Some(func) = self
                .functions
                .get(name)
                .copied()
                .or_else(|| self.module.get_function(name))
            {
                // Return function pointer as i64
                let fn_ptr = func.as_global_value().as_pointer_value();
                let fn_int = self
                    .builder
                    .build_ptr_to_int(fn_ptr, self.context.i64_type(), "fn_ptr_as_int")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(fn_int.into());
            }

            // Collect all available symbols for suggestions
            let mut candidates: Vec<&str> = Vec::new();

            // Add local variables
            for var_name in self.locals.keys() {
                candidates.push(var_name.as_str());
            }

            // Add function names
            for func_name in self.functions.keys() {
                candidates.push(func_name.as_str());
            }

            // Get suggestions
            let suggestions = crate::suggest_similar(name, &candidates, 3);
            let suggestion_text = crate::format_did_you_mean(&suggestions);

            Err(CodegenError::UndefinedVar(format!(
                "{}{}",
                name, suggestion_text
            )))
        }
    }

    fn generate_binary(
        &mut self,
        op: BinOp,
        lhs: &Expr,
        rhs: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let lhs_val = self.generate_expr(lhs)?;
        let rhs_val = self.generate_expr(rhs)?;

        // Determine if we're dealing with integers or floats
        let is_float = lhs_val.is_float_value();

        if is_float {
            self.generate_float_binary(op, lhs_val.into_float_value(), rhs_val.into_float_value())
        } else {
            self.generate_int_binary(op, lhs_val.into_int_value(), rhs_val.into_int_value())
        }
    }

    fn generate_int_binary(
        &mut self,
        op: BinOp,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let result = match op {
            BinOp::Add => self.builder.build_int_add(lhs, rhs, "add"),
            BinOp::Sub => self.builder.build_int_sub(lhs, rhs, "sub"),
            BinOp::Mul => self.builder.build_int_mul(lhs, rhs, "mul"),
            BinOp::Div => self.builder.build_int_signed_div(lhs, rhs, "div"),
            BinOp::Mod => self.builder.build_int_signed_rem(lhs, rhs, "rem"),
            BinOp::Eq => self
                .builder
                .build_int_compare(IntPredicate::EQ, lhs, rhs, "eq"),
            BinOp::Neq => self
                .builder
                .build_int_compare(IntPredicate::NE, lhs, rhs, "ne"),
            BinOp::Lt => self
                .builder
                .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt"),
            BinOp::Lte => self
                .builder
                .build_int_compare(IntPredicate::SLE, lhs, rhs, "le"),
            BinOp::Gt => self
                .builder
                .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt"),
            BinOp::Gte => self
                .builder
                .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge"),
            BinOp::And => self.builder.build_and(lhs, rhs, "and"),
            BinOp::Or => self.builder.build_or(lhs, rhs, "or"),
            BinOp::BitAnd => self.builder.build_and(lhs, rhs, "bitand"),
            BinOp::BitOr => self.builder.build_or(lhs, rhs, "bitor"),
            BinOp::BitXor => self.builder.build_xor(lhs, rhs, "bitxor"),
            BinOp::Shl => self.builder.build_left_shift(lhs, rhs, "shl"),
            BinOp::Shr => self.builder.build_right_shift(lhs, rhs, true, "shr"),
            // _ => return Err(CodegenError::Unsupported(format!("Binary op: {:?}", op))),
        };
        result
            .map(|v| v.into())
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    fn generate_float_binary(
        &mut self,
        op: BinOp,
        lhs: inkwell::values::FloatValue<'ctx>,
        rhs: inkwell::values::FloatValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let result = match op {
            BinOp::Add => self
                .builder
                .build_float_add(lhs, rhs, "fadd")
                .map(|v| v.into()),
            BinOp::Sub => self
                .builder
                .build_float_sub(lhs, rhs, "fsub")
                .map(|v| v.into()),
            BinOp::Mul => self
                .builder
                .build_float_mul(lhs, rhs, "fmul")
                .map(|v| v.into()),
            BinOp::Div => self
                .builder
                .build_float_div(lhs, rhs, "fdiv")
                .map(|v| v.into()),
            BinOp::Mod => self
                .builder
                .build_float_rem(lhs, rhs, "frem")
                .map(|v| v.into()),
            BinOp::Eq => self
                .builder
                .build_float_compare(FloatPredicate::OEQ, lhs, rhs, "feq")
                .map(|v| v.into()),
            BinOp::Neq => self
                .builder
                .build_float_compare(FloatPredicate::ONE, lhs, rhs, "fne")
                .map(|v| v.into()),
            BinOp::Lt => self
                .builder
                .build_float_compare(FloatPredicate::OLT, lhs, rhs, "flt")
                .map(|v| v.into()),
            BinOp::Lte => self
                .builder
                .build_float_compare(FloatPredicate::OLE, lhs, rhs, "fle")
                .map(|v| v.into()),
            BinOp::Gt => self
                .builder
                .build_float_compare(FloatPredicate::OGT, lhs, rhs, "fgt")
                .map(|v| v.into()),
            BinOp::Gte => self
                .builder
                .build_float_compare(FloatPredicate::OGE, lhs, rhs, "fge")
                .map(|v| v.into()),
            _ => {
                return Err(CodegenError::Unsupported(format!(
                    "Float binary op: {:?}",
                    op
                )))
            }
        };
        result.map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    fn generate_unary(
        &mut self,
        op: UnaryOp,
        operand: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let val = self.generate_expr(operand)?;

        match op {
            UnaryOp::Neg => {
                if val.is_float_value() {
                    self.builder
                        .build_float_neg(val.into_float_value(), "fneg")
                        .map(|v| v.into())
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))
                } else {
                    self.builder
                        .build_int_neg(val.into_int_value(), "neg")
                        .map(|v| v.into())
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))
                }
            }
            UnaryOp::Not => self
                .builder
                .build_not(val.into_int_value(), "not")
                .map(|v| v.into())
                .map_err(|e| CodegenError::LlvmError(e.to_string())),
            UnaryOp::BitNot => self
                .builder
                .build_not(val.into_int_value(), "bitnot")
                .map(|v| v.into())
                .map_err(|e| CodegenError::LlvmError(e.to_string())),
        }
    }

    fn generate_call(
        &mut self,
        callee: &Expr,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Get function name
        let fn_name = match callee {
            Expr::Ident(name) => name.clone(),
            Expr::SelfCall => {
                // @ recursive call: check if we're in TCO mode
                if let Some(tco) = &self.tco_state {
                    // TCO: update parameters and branch back to loop header
                    let param_allocas = tco.param_allocas.clone();
                    let loop_header = tco.loop_header;

                    // Evaluate all arguments first (before updating any params)
                    let arg_values: Vec<BasicValueEnum<'ctx>> = args
                        .iter()
                        .map(|arg| self.generate_expr(&arg.node))
                        .collect::<CodegenResult<Vec<_>>>()?;

                    // Update parameter allocas with new values
                    for (i, (_, alloca, param_type)) in param_allocas.iter().enumerate() {
                        if i < arg_values.len() {
                            let mut new_val = arg_values[i];
                            // Cast if needed
                            if new_val.get_type() != *param_type
                                && new_val.is_int_value()
                                && param_type.is_int_type()
                            {
                                new_val = self
                                    .builder
                                    .build_int_cast(
                                        new_val.into_int_value(),
                                        param_type.into_int_type(),
                                        "tco_cast",
                                    )
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                                    .into();
                            }
                            self.builder
                                .build_store(*alloca, new_val)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }

                    // Branch back to loop header
                    self.builder
                        .build_unconditional_branch(loop_header)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Return a dummy value (this code path is dead after the branch)
                    return Ok(self.context.i64_type().const_int(0, false).into());
                }

                // Non-TCO: regular recursive call
                if let Some(func) = self.current_function {
                    let fn_name = func.get_name().to_str().unwrap_or("").to_string();
                    fn_name
                } else {
                    return Err(CodegenError::Unsupported(
                        "SelfCall outside function".to_string(),
                    ));
                }
            }
            _ => {
                return Err(CodegenError::Unsupported(
                    "Indirect calls not yet supported".to_string(),
                ))
            }
        };

        // Handle built-in pseudo-functions that need special codegen
        // Handle puts with string interpolation: printf the interp, then puts("") for newline
        if fn_name == "puts" && args.len() == 1 && matches!(&args[0].node, Expr::StringInterp(_)) {
            let _interp_val = self.generate_expr(&args[0].node)?;
            // String interp already printed via printf; now add newline
            let printf_fn = self
                .module
                .get_function("printf")
                .ok_or_else(|| CodegenError::UndefinedFunction("printf".to_string()))?;
            let newline = self.generate_string_literal("\n")?;
            self.builder
                .build_call(printf_fn, &[newline.into()], "puts_nl")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(self.context.struct_type(&[], false).const_zero().into());
        }
        match fn_name.as_str() {
            "println" => return self.generate_println_call(args),
            "print" => return self.generate_print_call(args),
            "format" => return self.generate_format_call(args),
            "store_i64" => return self.generate_store_i64(args),
            "load_i64" => return self.generate_load_i64(args),
            "store_byte" => return self.generate_store_byte(args),
            "load_byte" => return self.generate_load_byte(args),
            "store_f64" => return self.generate_store_f64(args),
            "load_f64" => return self.generate_load_f64(args),
            // Option constructors: Some(val) -> { i8 tag=1, i64 data=val }
            "Some" => {
                let enum_type = self.context.struct_type(
                    &[
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let data_val = if args.is_empty() {
                    self.context.i64_type().const_int(0, false)
                } else {
                    let v = self.generate_expr(&args[0].node)?;
                    if v.is_int_value() {
                        v.into_int_value()
                    } else {
                        self.context.i64_type().const_int(0, false)
                    }
                };
                let mut val = enum_type.get_undef();
                val = self
                    .builder
                    .build_insert_value(
                        val,
                        self.context.i8_type().const_int(1, false),
                        0,
                        "some_tag",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                val = self
                    .builder
                    .build_insert_value(val, data_val, 1, "some_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                return Ok(val.into());
            }
            // Result constructors: Ok(val) -> { i8 tag=0, i64 data=val }
            "Ok" => {
                let enum_type = self.context.struct_type(
                    &[
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let data_val = if args.is_empty() {
                    self.context.i64_type().const_int(0, false)
                } else {
                    let v = self.generate_expr(&args[0].node)?;
                    if v.is_int_value() {
                        v.into_int_value()
                    } else {
                        self.context.i64_type().const_int(0, false)
                    }
                };
                let mut val = enum_type.get_undef();
                val = self
                    .builder
                    .build_insert_value(
                        val,
                        self.context.i8_type().const_int(0, false),
                        0,
                        "ok_tag",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                val = self
                    .builder
                    .build_insert_value(val, data_val, 1, "ok_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                return Ok(val.into());
            }
            // Err(val) -> { i8 tag=1, i64 data=val }
            "Err" => {
                let enum_type = self.context.struct_type(
                    &[
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let data_val = if args.is_empty() {
                    self.context.i64_type().const_int(0, false)
                } else {
                    let v = self.generate_expr(&args[0].node)?;
                    if v.is_int_value() {
                        v.into_int_value()
                    } else {
                        self.context.i64_type().const_int(0, false)
                    }
                };
                let mut val = enum_type.get_undef();
                val = self
                    .builder
                    .build_insert_value(
                        val,
                        self.context.i8_type().const_int(1, false),
                        0,
                        "err_tag",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                val = self
                    .builder
                    .build_insert_value(val, data_val, 1, "err_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                return Ok(val.into());
            }
            "puts_ptr" => {
                // puts_ptr(i64) -> i32: convert i64 to ptr then call puts
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "puts_ptr requires 1 arg".to_string(),
                    ));
                }
                let arg = self.generate_expr(&args[0].node)?;
                let ptr = self
                    .builder
                    .build_int_to_ptr(
                        arg.into_int_value(),
                        self.context.i8_type().ptr_type(AddressSpace::default()),
                        "puts_ptr_arg",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                let puts_fn = self
                    .module
                    .get_function("puts")
                    .ok_or_else(|| CodegenError::UndefinedFunction("puts".to_string()))?;
                let call = self
                    .builder
                    .build_call(puts_fn, &[ptr.into()], "puts_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()));
            }
            "str_to_ptr" => {
                // str_to_ptr(ptr) -> i64: convert ptr to i64
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "str_to_ptr requires 1 arg".to_string(),
                    ));
                }
                let arg = self.generate_expr(&args[0].node)?;
                if arg.is_pointer_value() {
                    let result = self
                        .builder
                        .build_ptr_to_int(
                            arg.into_pointer_value(),
                            self.context.i64_type(),
                            "str_to_ptr_result",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    return Ok(result.into());
                } else {
                    // Already an integer
                    return Ok(arg);
                }
            }
            "strlen_ptr" => {
                // strlen_ptr(i64) -> i64: convert i64 to ptr then call strlen
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "strlen_ptr requires 1 arg".to_string(),
                    ));
                }
                let arg = self.generate_expr(&args[0].node)?;
                let ptr = self
                    .builder
                    .build_int_to_ptr(
                        arg.into_int_value(),
                        self.context.i8_type().ptr_type(AddressSpace::default()),
                        "strlen_ptr_arg",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                let strlen_fn = self
                    .module
                    .get_function("strlen")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strlen".to_string()))?;
                let call = self
                    .builder
                    .build_call(strlen_fn, &[ptr.into()], "strlen_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()));
            }
            "__strlen" => {
                // __strlen is an alias for strlen
                let strlen_fn = self
                    .module
                    .get_function("strlen")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strlen".to_string()))?;
                let arg_values: Vec<BasicMetadataValueEnum> = args
                    .iter()
                    .map(|arg| self.generate_expr(&arg.node).map(|v| v.into()))
                    .collect::<CodegenResult<Vec<_>>>()?;
                let call = self
                    .builder
                    .build_call(strlen_fn, &arg_values, "strlen_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()));
            }
            _ => {}
        }

        // Check if this is a lambda binding (closure call)
        if let Some((lambda_fn_name, captured_vals)) = self.lambda_bindings.get(&fn_name).cloned() {
            if let Some(lambda_fn) = self
                .functions
                .get(&lambda_fn_name)
                .copied()
                .or_else(|| self.module.get_function(&lambda_fn_name))
            {
                // Build args: captured values first, then actual args
                let mut arg_values: Vec<BasicMetadataValueEnum> =
                    captured_vals.iter().map(|(_, val)| (*val).into()).collect();
                for arg in args {
                    arg_values.push(self.generate_expr(&arg.node)?.into());
                }
                let call = self
                    .builder
                    .build_call(lambda_fn, &arg_values, "lambda_call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                return Ok(call
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.context.struct_type(&[], false).const_zero().into()));
            }
        }

        // Get function value
        let fn_value = self
            .functions
            .get(&fn_name)
            .copied()
            .or_else(|| self.module.get_function(&fn_name));

        let fn_value = if let Some(func) = fn_value {
            func
        } else {
            // Check if this is an enum variant constructor (tuple variant)
            let is_enum_variant = self
                .enum_variants
                .iter()
                .find(|((_, v_name), _)| v_name == &fn_name)
                .map(|((_, _), tag)| *tag);

            if let Some(tag) = is_enum_variant {
                // Build enum value: { i8 tag, i64 data }
                let enum_type = self.context.struct_type(
                    &[
                        self.context.i8_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let data_val = if args.is_empty() {
                    self.context.i64_type().const_int(0, false)
                } else {
                    let v = self.generate_expr(&args[0].node)?;
                    if v.is_int_value() {
                        v.into_int_value()
                    } else {
                        self.context.i64_type().const_int(0, false)
                    }
                };
                let mut val = enum_type.get_undef();
                val = self
                    .builder
                    .build_insert_value(
                        val,
                        self.context.i8_type().const_int(tag as u64, false),
                        0,
                        "variant_tag",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                val = self
                    .builder
                    .build_insert_value(val, data_val, 1, "variant_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
                return Ok(val.into());
            }

            // Collect all available function names for suggestions
            let mut candidate_strings: Vec<String> = Vec::new();

            // Add registered functions
            for func_name in self.functions.keys() {
                candidate_strings.push(func_name.clone());
            }

            // Add module functions
            let mut current_func = self.module.get_first_function();
            while let Some(func) = current_func {
                if let Ok(name) = func.get_name().to_str() {
                    candidate_strings.push(name.to_string());
                }
                current_func = func.get_next_function();
            }

            // Get suggestions
            let candidates: Vec<&str> = candidate_strings.iter().map(|s| s.as_str()).collect();
            let suggestions = crate::suggest_similar(&fn_name, &candidates, 3);
            let suggestion_text = crate::format_did_you_mean(&suggestions);

            return Err(CodegenError::UndefinedFunction(format!(
                "{}{}",
                fn_name, suggestion_text
            )));
        };

        // Generate arguments
        let arg_values: Vec<BasicMetadataValueEnum> = args
            .iter()
            .map(|arg| self.generate_expr(&arg.node).map(|v| v.into()))
            .collect::<CodegenResult<Vec<_>>>()?;

        // Build call
        let call = self
            .builder
            .build_call(fn_value, &arg_values, "call")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Return call result or unit
        Ok(call
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| self.context.struct_type(&[], false).const_zero().into()))
    }

    fn generate_block(&mut self, stmts: &[Spanned<Stmt>]) -> CodegenResult<BasicValueEnum<'ctx>> {
        let mut last_value: BasicValueEnum =
            self.context.struct_type(&[], false).const_zero().into();

        for stmt in stmts {
            // Stop generating after a terminator (return/break/continue)
            if let Some(block) = self.builder.get_insert_block() {
                if block.get_terminator().is_some() {
                    break;
                }
            }
            last_value = self.generate_stmt(&stmt.node)?;
        }

        Ok(last_value)
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> CodegenResult<BasicValueEnum<'ctx>> {
        match stmt {
            Stmt::Let {
                name, ty, value, ..
            } => {
                // Track struct type from the value expression before generating
                let struct_type_name = self.infer_value_struct_type(&value.node);
                let is_lambda = matches!(&value.node, Expr::Lambda { .. });

                // Clear last lambda info before generating
                self._last_lambda_info = None;
                let val = self.generate_expr(&value.node)?;

                // If this was a lambda binding, record the lambda info
                if is_lambda {
                    if let Some((lambda_fn_name, captures)) = self._last_lambda_info.take() {
                        self.lambda_bindings
                            .insert(name.node.clone(), (lambda_fn_name, captures));
                    }
                }
                let var_type = if let Some(t) = ty.as_ref() {
                    let resolved = self.ast_type_to_resolved(&t.node);
                    self.type_mapper.map_type(&resolved)
                } else if val.is_struct_value() {
                    // Use actual struct type for struct values
                    val.get_type()
                } else if val.is_float_value() {
                    // Keep float type
                    val.get_type()
                } else if val.is_pointer_value()
                    && matches!(&value.node, Expr::Array(_) | Expr::Index { .. })
                {
                    // Keep pointer type for array allocations and slice results
                    val.get_type()
                } else {
                    // Default to i64 for non-struct values (backward compatible)
                    self.type_mapper.map_type(&ResolvedType::I64)
                };
                let alloca = self
                    .builder
                    .build_alloca(var_type, &name.node)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(name.node.clone(), (alloca, var_type));

                // Record struct type for variable (from StructLit, function return type, or type annotation)
                if let Some(sn) = struct_type_name {
                    self.var_struct_types.insert(name.node.clone(), sn);
                } else if let Some(t) = ty.as_ref() {
                    if let Some(sn) = self.extract_struct_type_name(&t.node) {
                        self.var_struct_types.insert(name.node.clone(), sn);
                    }
                } else if val.is_struct_value() {
                    // Fallback: match the generated value's struct type against known structs
                    let struct_type = val.into_struct_value().get_type();
                    for (sn, st) in &self.generated_structs {
                        if *st == struct_type {
                            self.var_struct_types.insert(name.node.clone(), sn.clone());
                            break;
                        }
                    }
                }

                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Expr(expr) => self.generate_expr(&expr.node),
            Stmt::Return(Some(expr)) => {
                let val = self.generate_expr(&expr.node)?;
                self.emit_defer_cleanup()?;
                self.builder
                    .build_return(Some(&val))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Return(None) => {
                self.emit_defer_cleanup()?;
                self.builder
                    .build_return(None)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Break(value) => self.generate_break(value.as_ref().map(|v| &v.node)),
            Stmt::Continue => self.generate_continue(),
            Stmt::Defer(expr) => {
                // Add deferred expression to stack (will be executed in LIFO order before return)
                self.defer_stack.push(expr.node.clone());
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut: _,
            } => self.generate_let_destructure(&pattern.node, &value.node),
            _ => Err(CodegenError::Unsupported(format!("Statement: {:?}", stmt))),
        }
    }

    fn generate_if_expr(
        &mut self,
        cond: &Expr,
        then_stmts: &[Spanned<Stmt>],
        else_branch: Option<&IfElse>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for if expression".to_string())
        })?;

        // Generate condition
        let cond_val = self.generate_expr(cond)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            // Convert to i1 if needed
            if int_val.get_type().get_bit_width() > 1 {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "cond_bool",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        // Create blocks
        let then_block = self.context.append_basic_block(fn_value, "then");
        let else_block = self.context.append_basic_block(fn_value, "else");
        let merge_block = self.context.append_basic_block(fn_value, "merge");

        // Conditional branch
        self.builder
            .build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Then block
        self.builder.position_at_end(then_block);
        let then_val = self.generate_block(then_stmts)?;
        let then_end_block = self.builder.get_insert_block().unwrap();
        let then_terminated = then_end_block.get_terminator().is_some();
        if !then_terminated {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Else block
        self.builder.position_at_end(else_block);
        let else_val = if let Some(else_branch) = else_branch {
            self.generate_if_else(else_branch)?
        } else {
            self.context.struct_type(&[], false).const_zero().into()
        };
        let else_end_block = self.builder.get_insert_block().unwrap();
        let else_terminated = else_end_block.get_terminator().is_some();
        if !else_terminated {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Merge block with phi
        self.builder.position_at_end(merge_block);

        // If both branches are terminated (return/break), merge is unreachable
        if then_terminated && else_terminated {
            self.builder
                .build_unreachable()
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(self.context.struct_type(&[], false).const_zero().into());
        }

        // Build phi node - only include non-terminated branches
        let mut incoming: Vec<(
            &dyn BasicValue<'ctx>,
            inkwell::basic_block::BasicBlock<'ctx>,
        )> = Vec::new();
        if !then_terminated {
            incoming.push((&then_val, then_end_block));
        }
        if !else_terminated {
            incoming.push((&else_val, else_end_block));
        }

        if incoming.len() == 1 {
            // Only one branch reaches merge - no phi needed
            Ok(incoming[0].0.as_basic_value_enum())
        } else if !incoming.is_empty() && then_val.get_type() == else_val.get_type() {
            let phi = self
                .builder
                .build_phi(then_val.get_type(), "if_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            for (val, block) in &incoming {
                phi.add_incoming(&[(*val, *block)]);
            }
            Ok(phi.as_basic_value())
        } else {
            Ok(self.context.struct_type(&[], false).const_zero().into())
        }
    }

    fn generate_if_else(&mut self, if_else: &IfElse) -> CodegenResult<BasicValueEnum<'ctx>> {
        match if_else {
            IfElse::Else(stmts) => self.generate_block(stmts),
            IfElse::ElseIf(cond, then_stmts, else_branch) => self.generate_if_expr(
                &cond.node,
                then_stmts,
                else_branch.as_ref().map(|b| b.as_ref()),
            ),
        }
    }

    // ========== Loop Expression ==========

    fn generate_loop(
        &mut self,
        pattern: Option<&Spanned<Pattern>>,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self
            .current_function
            .ok_or_else(|| CodegenError::LlvmError("No current function for loop".to_string()))?;

        // Check if this is a range-based for loop
        let is_range_loop = iter
            .as_ref()
            .is_some_and(|it| matches!(&it.node, Expr::Range { .. }));

        if is_range_loop {
            if let (Some(pat), Some(it)) = (pattern, iter) {
                // Range-based for loop: L pattern : start..end { body }
                return self.generate_range_for_loop(fn_value, pat, it, body);
            }
        }
        // Condition-based or infinite loop
        self.generate_condition_loop(fn_value, pattern, iter, body)
    }

    fn generate_range_for_loop(
        &mut self,
        fn_value: inkwell::values::FunctionValue<'ctx>,
        pattern: &Spanned<Pattern>,
        iter: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Extract range start, end, inclusive from the iter expression
        let (start_expr, end_expr, inclusive) = match &iter.node {
            Expr::Range {
                start,
                end,
                inclusive,
            } => (start.as_deref(), end.as_deref(), *inclusive),
            _ => unreachable!("generate_range_for_loop called with non-range iter"),
        };

        // Generate start and end values
        let start_val = if let Some(s) = start_expr {
            self.generate_expr(&s.node)?.into_int_value()
        } else {
            self.context.i64_type().const_int(0, false)
        };

        let end_val = if let Some(e) = end_expr {
            self.generate_expr(&e.node)?.into_int_value()
        } else {
            self.context.i64_type().const_int(i64::MAX as u64, false)
        };

        // Create counter variable
        let counter_alloca = self
            .builder
            .build_alloca(self.context.i64_type(), "loop_counter")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(counter_alloca, start_val)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let loop_cond = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("for.cond"));
        let loop_body = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("for.body"));
        let loop_inc = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("for.inc"));
        let loop_end = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("for.end"));

        // Push loop context: continue goes to increment, break goes to end
        self.loop_stack.push(LoopContext {
            break_block: loop_end,
            continue_block: loop_inc,
        });

        // Branch to condition check
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Condition block: counter < end (or counter <= end for inclusive)
        self.builder.position_at_end(loop_cond);
        let current_val = self
            .builder
            .build_load(self.context.i64_type(), counter_alloca, "counter_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();

        let cmp_pred = if inclusive {
            IntPredicate::SLE
        } else {
            IntPredicate::SLT
        };
        let cond = self
            .builder
            .build_int_compare(cmp_pred, current_val, end_val, "for_cond")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder
            .build_conditional_branch(cond, loop_body, loop_end)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Body block: bind pattern to current counter value, execute body
        self.builder.position_at_end(loop_body);

        // Load current counter and bind to pattern
        let counter_for_bind = self
            .builder
            .build_load(self.context.i64_type(), counter_alloca, "bind_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.generate_pattern_bindings(pattern, &counter_for_bind)?;

        // Generate body
        let _body_val = self.generate_block(body)?;

        // Branch to increment (if not terminated by break/return)
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_inc)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Increment block: counter += 1
        self.builder.position_at_end(loop_inc);
        let inc_val = self
            .builder
            .build_load(self.context.i64_type(), counter_alloca, "inc_load")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        let next_val = self
            .builder
            .build_int_add(
                inc_val,
                self.context.i64_type().const_int(1, false),
                "inc_val",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(counter_alloca, next_val)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // End block
        self.builder.position_at_end(loop_end);
        self.loop_stack.pop();

        // For loops return unit
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_condition_loop(
        &mut self,
        fn_value: inkwell::values::FunctionValue<'ctx>,
        pattern: Option<&Spanned<Pattern>>,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let loop_start = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("loop.start"));
        let loop_body = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("loop.body"));
        let loop_end = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("loop.end"));

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            break_block: loop_end,
            continue_block: loop_start,
        });

        // Branch to loop start
        self.builder
            .build_unconditional_branch(loop_start)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop start - check condition if present
        self.builder.position_at_end(loop_start);

        if let Some(iter_expr) = iter {
            // Conditional loop (while-like)
            let cond_val = self.generate_expr(&iter_expr.node)?;
            let cond_bool = if cond_val.is_int_value() {
                let int_val = cond_val.into_int_value();
                if int_val.get_type().get_bit_width() > 1 {
                    self.builder
                        .build_int_compare(
                            IntPredicate::NE,
                            int_val,
                            int_val.get_type().const_int(0, false),
                            "loop_cond",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                } else {
                    int_val
                }
            } else {
                self.context.bool_type().const_int(1, false)
            };

            self.builder
                .build_conditional_branch(cond_bool, loop_body, loop_end)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        } else {
            // Infinite loop
            self.builder
                .build_unconditional_branch(loop_body)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Loop body
        self.builder.position_at_end(loop_body);

        // Bind pattern if present (for non-range patterns with condition value)
        if let (Some(pat), Some(iter_expr)) = (pattern, iter) {
            let iter_val = self.generate_expr(&iter_expr.node)?;
            self.generate_pattern_bindings(pat, &iter_val)?;
        }

        let _body_val = self.generate_block(body)?;

        // Branch back to loop start (if not terminated by break/return)
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_start)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Loop end
        self.builder.position_at_end(loop_end);
        self.loop_stack.pop();

        // Loops return unit by default
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_while_loop(
        &mut self,
        condition: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for while loop".to_string())
        })?;

        let loop_cond = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("while.cond"));
        let loop_body = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("while.body"));
        let loop_end = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("while.end"));

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            break_block: loop_end,
            continue_block: loop_cond,
        });

        // Branch to condition check
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Condition block
        self.builder.position_at_end(loop_cond);
        let cond_val = self.generate_expr(&condition.node)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() > 1 {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "while_cond",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        self.builder
            .build_conditional_branch(cond_bool, loop_body, loop_end)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop body
        self.builder.position_at_end(loop_body);
        let _body_val = self.generate_block(body)?;

        // Branch back to condition (if not terminated by break/return)
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_cond)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Loop end
        self.builder.position_at_end(loop_end);
        self.loop_stack.pop();

        // While loops return unit
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    /// Emit deferred expressions in LIFO order (before function return).
    fn emit_defer_cleanup(&mut self) -> CodegenResult<()> {
        let deferred: Vec<Expr> = self.defer_stack.iter().rev().cloned().collect();
        for expr in deferred {
            self.generate_expr(&expr)?;
        }
        Ok(())
    }

    fn generate_break(&mut self, value: Option<&Expr>) -> CodegenResult<BasicValueEnum<'ctx>> {
        let break_block = self
            .loop_stack
            .last()
            .ok_or_else(|| CodegenError::Unsupported("break outside of loop".to_string()))?
            .break_block;

        // Generate value if present (for loop with value)
        if let Some(val_expr) = value {
            let _val = self.generate_expr(val_expr)?;
            // In a full implementation, this would be used for loop-with-value
        }

        self.builder
            .build_unconditional_branch(break_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_continue(&mut self) -> CodegenResult<BasicValueEnum<'ctx>> {
        let loop_ctx = self
            .loop_stack
            .last()
            .ok_or_else(|| CodegenError::Unsupported("continue outside of loop".to_string()))?;

        let continue_block = loop_ctx.continue_block;
        self.builder
            .build_unconditional_branch(continue_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    // ========== Array/Tuple/Index ==========

    fn generate_array(
        &mut self,
        elements: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if elements.is_empty() {
            // Empty array - return null pointer
            return Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .const_null()
                .into());
        }

        // Generate all elements
        let mut values: Vec<BasicValueEnum> = Vec::new();
        for elem in elements {
            values.push(self.generate_expr(&elem.node)?);
        }

        // Determine element type from first element
        let elem_type = values[0].get_type();
        let array_type = elem_type.array_type(elements.len() as u32);

        // Allocate array on stack
        let array_ptr = self
            .builder
            .build_alloca(array_type, "array")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store each element
        for (i, val) in values.iter().enumerate() {
            let idx = self.context.i64_type().const_int(i as u64, false);
            let elem_ptr = unsafe {
                self.builder
                    .build_gep(
                        array_type,
                        array_ptr,
                        &[self.context.i64_type().const_int(0, false), idx],
                        &format!("array_elem_{}", i),
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            };
            self.builder
                .build_store(elem_ptr, *val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        Ok(array_ptr.into())
    }

    fn generate_tuple(
        &mut self,
        elements: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        if elements.is_empty() {
            return Ok(self.context.struct_type(&[], false).const_zero().into());
        }

        // Generate all elements
        let mut values: Vec<BasicValueEnum> = Vec::new();
        for elem in elements {
            values.push(self.generate_expr(&elem.node)?);
        }

        // Create anonymous struct type for tuple
        let field_types: Vec<BasicTypeEnum> = values.iter().map(|v| v.get_type()).collect();
        let tuple_type = self.context.struct_type(&field_types, false);

        // Build tuple value
        let mut tuple_val = tuple_type.get_undef();
        for (i, val) in values.iter().enumerate() {
            tuple_val = self
                .builder
                .build_insert_value(tuple_val, *val, i as u32, "tuple")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_struct_value();
        }

        Ok(tuple_val.into())
    }

    fn generate_index(&mut self, arr: &Expr, index: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        let arr_val = self.generate_expr(arr)?;
        let idx_val = self.generate_expr(index)?;

        let arr_ptr = arr_val.into_pointer_value();
        let idx_int = idx_val.into_int_value();

        // Get element pointer
        let elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i64_type(), // Assume i64 elements for now
                    arr_ptr,
                    &[idx_int],
                    "elem_ptr",
                )
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };

        // Load element
        self.builder
            .build_load(self.context.i64_type(), elem_ptr, "elem")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    fn generate_slice(
        &mut self,
        arr: &Expr,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        inclusive: bool,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self
            .current_function
            .ok_or_else(|| CodegenError::LlvmError("No current function for slice".to_string()))?;

        let arr_val = self.generate_expr(arr)?;
        let arr_ptr = arr_val.into_pointer_value();

        // Get start index (default 0)
        let start_val = if let Some(start_expr) = start {
            self.generate_expr(&start_expr.node)?.into_int_value()
        } else {
            self.context.i64_type().const_int(0, false)
        };

        // Get end index
        let end_val = if let Some(end_expr) = end {
            let val = self.generate_expr(&end_expr.node)?.into_int_value();
            if inclusive {
                self.builder
                    .build_int_add(val, self.context.i64_type().const_int(1, false), "incl_end")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                val
            }
        } else {
            return Err(CodegenError::Unsupported(
                "Slice without end index requires array length".to_string(),
            ));
        };

        // Calculate slice length: end - start
        let length = self
            .builder
            .build_int_sub(end_val, start_val, "slice_len")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Allocate new array: malloc(length * 8)
        let byte_size = self
            .builder
            .build_int_mul(
                length,
                self.context.i64_type().const_int(8, false),
                "byte_size",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
            let fn_type = self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .fn_type(&[self.context.i64_type().into()], false);
            self.module.add_function("malloc", fn_type, None)
        });

        let raw_ptr = self
            .builder
            .build_call(malloc_fn, &[byte_size.into()], "slice_raw")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .try_as_basic_value()
            .left()
            .unwrap();
        let slice_ptr = self
            .builder
            .build_pointer_cast(
                raw_ptr.into_pointer_value(),
                self.context.i64_type().ptr_type(AddressSpace::default()),
                "slice_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Copy elements using a loop
        let loop_var = self
            .builder
            .build_alloca(self.context.i64_type(), "slice_i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(loop_var, self.context.i64_type().const_int(0, false))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let loop_cond = self.context.append_basic_block(fn_value, "slice_cond");
        let loop_body = self.context.append_basic_block(fn_value, "slice_body");
        let loop_end = self.context.append_basic_block(fn_value, "slice_end");

        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop condition: i < length
        self.builder.position_at_end(loop_cond);
        let i = self
            .builder
            .build_load(self.context.i64_type(), loop_var, "i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_int_value();
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::SLT, i, length, "slice_cmp")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_conditional_branch(cmp, loop_body, loop_end)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop body: slice_ptr[i] = arr_ptr[start + i]
        self.builder.position_at_end(loop_body);
        let src_idx = self
            .builder
            .build_int_add(start_val, i, "src_idx")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let src_ptr = unsafe {
            self.builder
                .build_gep(self.context.i64_type(), arr_ptr, &[src_idx], "src_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        let elem = self
            .builder
            .build_load(self.context.i64_type(), src_ptr, "elem")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let dst_ptr = unsafe {
            self.builder
                .build_gep(self.context.i64_type(), slice_ptr, &[i], "dst_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };
        self.builder
            .build_store(dst_ptr, elem)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // i++
        let next_i = self
            .builder
            .build_int_add(i, self.context.i64_type().const_int(1, false), "next_i")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(loop_var, next_i)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // After loop
        self.builder.position_at_end(loop_end);
        Ok(slice_ptr.into())
    }

    // ========== Method Call ==========

    fn generate_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Transform method call to function call with receiver as first arg
        // e.g., obj.method(a, b) -> TypeName_method(obj, a, b)

        // Try to resolve the struct type name from the receiver
        let mut struct_name = self.infer_struct_name(receiver).ok();

        // For SelfCall (@), infer struct type from current function name (TypeName_method pattern)
        if struct_name.is_none() && matches!(receiver, Expr::SelfCall) {
            if let Some(func) = self.current_function {
                let fn_name = func.get_name().to_str().unwrap_or("").to_string();
                if let Some(idx) = fn_name.find('_') {
                    struct_name = Some(fn_name[..idx].to_string());
                }
            }
        }

        // Get receiver pointer for pass-by-reference self parameter
        let receiver_ptr: Option<PointerValue<'ctx>> = match receiver {
            Expr::Ident(name) => self.locals.get(name).map(|(ptr, _)| *ptr),
            Expr::SelfCall => {
                // @ in method context: self is already a pointer
                self.locals.get("self").map(|(ptr, _)| *ptr)
            }
            _ => None,
        };

        // Also generate the receiver value as fallback (for non-method calls or unknown receivers)
        let receiver_val = self.generate_expr(receiver)?;

        // Try qualified name: TypeName_method
        let qualified_name = struct_name.as_ref().map(|sn| format!("{}_{}", sn, method));

        let fn_value = qualified_name
            .as_ref()
            .and_then(|qn| {
                self.functions
                    .get(qn)
                    .copied()
                    .or_else(|| self.module.get_function(qn))
            })
            // Fallback: try bare method name
            .or_else(|| {
                self.functions
                    .get(method)
                    .copied()
                    .or_else(|| self.module.get_function(method))
            });

        // If not found, try broader search: look for any TypeName_method pattern
        let fn_value = if let Some(f) = fn_value {
            f
        } else {
            // Try all known struct types with this method name
            let mut found = None;
            for sn in self.generated_structs.keys() {
                let candidate = format!("{}_{}", sn, method);
                if let Some(f) = self
                    .functions
                    .get(&candidate)
                    .copied()
                    .or_else(|| self.module.get_function(&candidate))
                {
                    found = Some(f);
                    break;
                }
            }
            if let Some(f) = found {
                f
            } else {
                let tried = qualified_name.as_deref().unwrap_or(method);
                return Err(CodegenError::UndefinedFunction(format!(
                    "{} (method call on {:?})",
                    tried, receiver
                )));
            }
        };

        // Generate arguments (receiver first, pass as pointer for methods)
        let mut arg_values: Vec<BasicMetadataValueEnum> = if let Some(ptr) = receiver_ptr {
            // Pass receiver as pointer (self by reference)
            vec![ptr.into()]
        } else {
            // Fallback: for struct literal receivers or complex expressions,
            // create a temporary alloca and pass its pointer
            if struct_name.is_some() {
                let alloca = self
                    .builder
                    .build_alloca(receiver_val.get_type(), "tmp_self")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, receiver_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                vec![alloca.into()]
            } else {
                vec![receiver_val.into()]
            }
        };
        for arg in args {
            arg_values.push(self.generate_expr(&arg.node)?.into());
        }

        // Build call
        let call = self
            .builder
            .build_call(fn_value, &arg_values, "method_call")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(call
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| self.context.struct_type(&[], false).const_zero().into()))
    }

    // ========== Lambda/Closure ==========

    fn generate_lambda(
        &mut self,
        params: &[ast::Param],
        body: &Expr,
        captures: &[String],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Generate unique lambda function name
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Find captured variables from current scope
        // If captures list is empty (type checker didn't fill it), auto-detect from body
        let effective_captures: Vec<String> = if captures.is_empty() {
            let param_names: std::collections::HashSet<String> =
                params.iter().map(|p| p.name.node.clone()).collect();
            let used_idents = Self::collect_idents(body);
            used_idents
                .into_iter()
                .filter(|name| !param_names.contains(name) && self.locals.contains_key(name))
                .collect()
        } else {
            captures.to_vec()
        };

        let mut captured_vars: Vec<(String, BasicValueEnum<'ctx>, BasicTypeEnum<'ctx>)> =
            Vec::new();
        for cap_name in &effective_captures {
            if let Some((ptr, var_type)) = self.locals.get(cap_name) {
                // Load the captured value
                let val = self
                    .builder
                    .build_load(*var_type, *ptr, &format!("cap_{}", cap_name))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                captured_vars.push((cap_name.clone(), val, *var_type));
            }
        }

        // Build parameter types: captured vars first, then lambda params
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();

        // First add captured variables as parameters
        for (_, _, cap_type) in &captured_vars {
            param_types.push((*cap_type).into());
        }

        // Then add original lambda parameters
        for p in params {
            let resolved = self.ast_type_to_resolved(&p.ty.node);
            param_types.push(self.type_mapper.map_type(&resolved).into());
        }

        // Create function type (always returns i64 for now)
        let fn_type = self.context.i64_type().fn_type(&param_types, false);
        let lambda_fn = self.module.add_function(&lambda_name, fn_type, None);
        self.lambda_functions.push(lambda_fn);

        // Save current state
        let saved_function = self.current_function;
        let saved_locals = self.locals.clone();
        let saved_insert_block = self.builder.get_insert_block();

        // Set up lambda context
        self.current_function = Some(lambda_fn);
        self.locals.clear();

        // Create entry block for lambda
        let entry = self.context.append_basic_block(lambda_fn, "entry");
        self.builder.position_at_end(entry);

        // Register captured variables as parameters in lambda scope
        let mut param_idx = 0u32;
        for (cap_name, _, cap_type) in &captured_vars {
            let param_val = lambda_fn.get_nth_param(param_idx).unwrap();
            let alloca = self
                .builder
                .build_alloca(*cap_type, &format!("__cap_{}", cap_name))
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals.insert(cap_name.clone(), (alloca, *cap_type));
            param_idx += 1;
        }

        // Register original parameters
        for p in params {
            let param_val = lambda_fn.get_nth_param(param_idx).unwrap();
            let ty = self.ast_type_to_resolved(&p.ty.node);
            let param_type = self.type_mapper.map_type(&ty);
            let alloca = self
                .builder
                .build_alloca(param_type, &p.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals
                .insert(p.name.node.clone(), (alloca, param_type));
            param_idx += 1;
        }

        // Generate lambda body
        let body_val = self.generate_expr(body)?;

        // Add return
        self.builder
            .build_return(Some(&body_val))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Restore context
        self.current_function = saved_function;
        self.locals = saved_locals;
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        // Register lambda as a callable function
        self.functions.insert(lambda_name.clone(), lambda_fn);

        // Store captured values for later use at call sites
        let captured_for_binding: Vec<(String, BasicValueEnum<'ctx>)> = captured_vars
            .iter()
            .map(|(name, val, _)| (name.clone(), *val))
            .collect();

        // Store the last lambda info so Stmt::Let can track it
        self._last_lambda_info = Some((lambda_name.clone(), captured_for_binding));

        // Return function pointer as i64
        let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
        let fn_int = self
            .builder
            .build_ptr_to_int(fn_ptr, self.context.i64_type(), "lambda_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(fn_int.into())
    }

    /// Collect all Ident names used in an expression (for auto-capture detection)
    fn collect_idents(expr: &Expr) -> Vec<String> {
        let mut idents = Vec::new();
        Self::collect_idents_inner(expr, &mut idents);
        idents.sort();
        idents.dedup();
        idents
    }

    fn collect_idents_inner(expr: &Expr, idents: &mut Vec<String>) {
        match expr {
            Expr::Ident(name) => idents.push(name.clone()),
            Expr::Binary { left, right, .. } => {
                Self::collect_idents_inner(&left.node, idents);
                Self::collect_idents_inner(&right.node, idents);
            }
            Expr::Unary { expr, .. } => Self::collect_idents_inner(&expr.node, idents),
            Expr::Call { func, args } => {
                Self::collect_idents_inner(&func.node, idents);
                for arg in args {
                    Self::collect_idents_inner(&arg.node, idents);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    if let Stmt::Expr(e) = &stmt.node {
                        Self::collect_idents_inner(&e.node, idents);
                    }
                }
            }
            Expr::If { cond, then, else_ } => {
                Self::collect_idents_inner(&cond.node, idents);
                for stmt in then {
                    if let Stmt::Expr(e) = &stmt.node {
                        Self::collect_idents_inner(&e.node, idents);
                    }
                }
                if let Some(else_branch) = else_ {
                    match else_branch {
                        IfElse::ElseIf(cond_expr, then_stmts, _else_opt) => {
                            Self::collect_idents_inner(&cond_expr.node, idents);
                            for stmt in then_stmts {
                                if let Stmt::Expr(e) = &stmt.node {
                                    Self::collect_idents_inner(&e.node, idents);
                                }
                            }
                        }
                        IfElse::Else(stmts) => {
                            for stmt in stmts {
                                if let Stmt::Expr(e) = &stmt.node {
                                    Self::collect_idents_inner(&e.node, idents);
                                }
                            }
                        }
                    }
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                Self::collect_idents_inner(&receiver.node, idents);
                for arg in args {
                    Self::collect_idents_inner(&arg.node, idents);
                }
            }
            Expr::Field { expr, .. } => Self::collect_idents_inner(&expr.node, idents),
            Expr::Index { expr, index } => {
                Self::collect_idents_inner(&expr.node, idents);
                Self::collect_idents_inner(&index.node, idents);
            }
            Expr::Tuple(elems) | Expr::Array(elems) => {
                for e in elems {
                    Self::collect_idents_inner(&e.node, idents);
                }
            }
            _ => {}
        }
    }

    /// Convert AST Type to ResolvedType
    fn ast_type_to_resolved(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, generics } => match name.as_str() {
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
                    // Single uppercase letter is likely a generic type parameter
                    if name.len() == 1 && name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        ResolvedType::Generic(name.clone())
                    } else {
                        let generic_types: Vec<ResolvedType> = generics
                            .iter()
                            .map(|g| self.ast_type_to_resolved(&g.node))
                            .collect();
                        ResolvedType::Named {
                            name: name.clone(),
                            generics: generic_types,
                        }
                    }
                }
            },
            Type::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Array(inner) => {
                ResolvedType::Array(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Tuple(elems) => {
                let elem_types: Vec<ResolvedType> = elems
                    .iter()
                    .map(|e| self.ast_type_to_resolved(&e.node))
                    .collect();
                ResolvedType::Tuple(elem_types)
            }
            Type::FnPtr {
                params,
                ret,
                is_vararg,
            } => {
                let param_types: Vec<ResolvedType> = params
                    .iter()
                    .map(|p| self.ast_type_to_resolved(&p.node))
                    .collect();
                let ret_type = self.ast_type_to_resolved(&ret.node);
                ResolvedType::FnPtr {
                    params: param_types,
                    ret: Box::new(ret_type),
                    is_vararg: *is_vararg,
                    effects: None,
                }
            }
            Type::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Result(inner) => {
                ResolvedType::Result(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Map(key, value) => ResolvedType::Map(
                Box::new(self.ast_type_to_resolved(&key.node)),
                Box::new(self.ast_type_to_resolved(&value.node)),
            ),
            Type::Unit => ResolvedType::Unit,
            _ => ResolvedType::I64, // Fallback for Infer, ConstArray, etc.
        }
    }

    // ========== Generic Type Handling ==========

    /// Get current generic substitution for a type parameter
    pub fn get_generic_substitution(&self, param: &str) -> Option<ResolvedType> {
        self.generic_substitutions.get(param).cloned()
    }

    /// Set generic substitutions for the current context
    pub fn set_generic_substitutions(&mut self, subst: HashMap<String, ResolvedType>) {
        self.generic_substitutions = subst;
    }

    /// Clear generic substitutions
    pub fn clear_generic_substitutions(&mut self) {
        self.generic_substitutions.clear();
    }

    /// Substitute generic type parameters with concrete types
    pub fn substitute_type(&self, ty: &ResolvedType) -> ResolvedType {
        vais_types::substitute_type(ty, &self.generic_substitutions)
    }

    /// Generate mangled name for a generic struct
    pub fn mangle_struct_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Generate mangled name for a generic function
    pub fn mangle_function_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Map a type to LLVM, handling generics through substitution
    #[allow(dead_code)]
    fn map_type_with_generics(&self, ty: &ResolvedType) -> BasicTypeEnum<'ctx> {
        // First substitute any generic parameters
        let substituted = self.substitute_type(ty);
        self.type_mapper.map_type(&substituted)
    }

    /// Define a specialized (monomorphized) struct type
    pub fn define_specialized_struct(
        &mut self,
        base_name: &str,
        type_args: &[ResolvedType],
        fields: &[(String, ResolvedType)],
    ) -> CodegenResult<StructType<'ctx>> {
        let mangled_name = self.mangle_struct_name(base_name, type_args);

        // Check if already generated
        if let Some(st) = self.generated_structs.get(&mangled_name) {
            return Ok(*st);
        }

        // Build substitution map from generic params to type args
        let mut substitutions = HashMap::new();
        // Assume generic params are T, U, V... in order
        let generic_names = ["T", "U", "V", "W", "X", "Y", "Z"];
        for (i, type_arg) in type_args.iter().enumerate() {
            if let Some(name) = generic_names.get(i) {
                substitutions.insert(name.to_string(), type_arg.clone());
            }
        }

        // Substitute types in fields
        let field_types: Vec<BasicTypeEnum> = fields
            .iter()
            .map(|(_, ty)| {
                let substituted = vais_types::substitute_type(ty, &substitutions);
                self.type_mapper.map_type(&substituted)
            })
            .collect();

        // Store field names
        let field_names: Vec<String> = fields.iter().map(|(name, _)| name.clone()).collect();
        self.struct_fields.insert(mangled_name.clone(), field_names);

        // Create struct type
        let struct_type = self.context.struct_type(&field_types, false);
        self.type_mapper.register_struct(&mangled_name, struct_type);
        self.generated_structs.insert(mangled_name, struct_type);

        Ok(struct_type)
    }

    /// Declare a specialized (monomorphized) function
    pub fn declare_specialized_function(
        &mut self,
        base_name: &str,
        type_args: &[ResolvedType],
        param_types: &[ResolvedType],
        return_type: &ResolvedType,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        let mangled_name = self.mangle_function_name(base_name, type_args);

        // Check if already declared
        if let Some(fn_val) = self.functions.get(&mangled_name) {
            return Ok(*fn_val);
        }

        // Build substitution map
        let mut substitutions = HashMap::new();
        let generic_names = ["T", "U", "V", "W", "X", "Y", "Z"];
        for (i, type_arg) in type_args.iter().enumerate() {
            if let Some(name) = generic_names.get(i) {
                substitutions.insert(name.to_string(), type_arg.clone());
            }
        }

        // Substitute types in parameters
        let llvm_param_types: Vec<BasicMetadataTypeEnum> = param_types
            .iter()
            .map(|ty| {
                let substituted = vais_types::substitute_type(ty, &substitutions);
                self.type_mapper.map_type(&substituted).into()
            })
            .collect();

        // Substitute return type
        let substituted_ret = vais_types::substitute_type(return_type, &substitutions);
        let fn_type = if matches!(substituted_ret, ResolvedType::Unit) {
            self.context.void_type().fn_type(&llvm_param_types, false)
        } else {
            let ret_type = self.type_mapper.map_type(&substituted_ret);
            ret_type.fn_type(&llvm_param_types, false)
        };

        let fn_value = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name, fn_value);

        Ok(fn_value)
    }

    // ========== Try/Unwrap ==========

    fn generate_try(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Try (?) operator - propagate error if Result/Option is error/None
        // For now, just evaluate the inner expression
        self.generate_expr(inner)
    }

    fn generate_unwrap(&mut self, inner: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Unwrap (!) operator - panic if Result/Option is error/None
        // For now, just evaluate the inner expression
        self.generate_expr(inner)
    }

    // ========== Assignment ==========

    fn generate_assign(
        &mut self,
        target: &Expr,
        value: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let val = self.generate_expr(value)?;

        match target {
            Expr::Ident(name) => {
                if let Some((ptr, _var_type)) = self.locals.get(name).cloned() {
                    self.builder
                        .build_store(ptr, val)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    Ok(val)
                } else {
                    Err(CodegenError::UndefinedVar(name.clone()))
                }
            }
            Expr::Field { expr: obj, field } => {
                // Field assignment
                let struct_name = self.infer_struct_name(&obj.node)?;
                let field_idx = self.get_field_index(&struct_name, &field.node)?;

                // Get struct pointer
                if let Expr::Ident(var_name) = &obj.node {
                    if let Some((ptr, _)) = self.locals.get(var_name).cloned() {
                        let struct_type = self
                            .generated_structs
                            .get(&struct_name)
                            .ok_or_else(|| CodegenError::UndefinedVar(struct_name.clone()))?;

                        let field_ptr = self
                            .builder
                            .build_struct_gep(*struct_type, ptr, field_idx, "field_ptr")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        self.builder
                            .build_store(field_ptr, val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        return Ok(val);
                    }
                }
                Err(CodegenError::Unsupported(
                    "Complex field assignment".to_string(),
                ))
            }
            Expr::Index { expr: arr, index } => {
                // Array index assignment
                let arr_val = self.generate_expr(&arr.node)?;
                let idx_val = self.generate_expr(&index.node)?;

                let arr_ptr = arr_val.into_pointer_value();
                let idx_int = idx_val.into_int_value();

                let elem_ptr = unsafe {
                    self.builder
                        .build_gep(val.get_type(), arr_ptr, &[idx_int], "elem_ptr")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };

                self.builder
                    .build_store(elem_ptr, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(val)
            }
            Expr::Deref(inner) => {
                // Dereference assignment: *ptr = val
                let ptr_val = self.generate_expr(&inner.node)?;
                let ptr = if ptr_val.is_pointer_value() {
                    ptr_val.into_pointer_value()
                } else {
                    // Convert i64 to pointer
                    self.builder
                        .build_int_to_ptr(
                            ptr_val.into_int_value(),
                            val.get_type().ptr_type(AddressSpace::default()),
                            "deref_assign_ptr",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };
                self.builder
                    .build_store(ptr, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(val)
            }
            _ => Err(CodegenError::Unsupported("Assignment target".to_string())),
        }
    }

    fn generate_assign_op(
        &mut self,
        op: BinOp,
        target: &Expr,
        value: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Load current value
        let current = self.generate_expr(target)?;
        let rhs = self.generate_expr(value)?;

        // Perform operation
        let result = if current.is_int_value() {
            self.generate_int_binary(op, current.into_int_value(), rhs.into_int_value())?
        } else {
            self.generate_float_binary(op, current.into_float_value(), rhs.into_float_value())?
        };

        // Store back
        if let Expr::Ident(name) = target {
            if let Some((ptr, _)) = self.locals.get(name).cloned() {
                self.builder
                    .build_store(ptr, result)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }
        }

        Ok(result)
    }

    // ========== Range ==========

    fn generate_range(
        &mut self,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        _inclusive: bool,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Range is represented as a struct { start: i64, end: i64 }
        let range_type = self.context.struct_type(
            &[
                self.context.i64_type().into(),
                self.context.i64_type().into(),
            ],
            false,
        );

        let start_val = if let Some(s) = start {
            self.generate_expr(&s.node)?
        } else {
            self.context.i64_type().const_int(0, false).into()
        };

        let end_val = if let Some(e) = end {
            self.generate_expr(&e.node)?
        } else {
            self.context
                .i64_type()
                .const_int(i64::MAX as u64, false)
                .into()
        };

        let mut range_val = range_type.get_undef();
        range_val = self
            .builder
            .build_insert_value(range_val, start_val, 0, "range_start")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        range_val = self
            .builder
            .build_insert_value(range_val, end_val, 1, "range_end")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();

        Ok(range_val.into())
    }

    // ========== Ternary ==========

    fn generate_ternary(
        &mut self,
        cond: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for ternary".to_string())
        })?;

        let cond_val = self.generate_expr(cond)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() > 1 {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "ternary_cond",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        let then_block = self.context.append_basic_block(fn_value, "ternary_then");
        let else_block = self.context.append_basic_block(fn_value, "ternary_else");
        let merge_block = self.context.append_basic_block(fn_value, "ternary_merge");

        self.builder
            .build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder.position_at_end(then_block);
        let then_val = self.generate_expr(then_expr)?;
        let then_end = self.builder.get_insert_block().unwrap();
        let then_terminated = then_end.get_terminator().is_some();
        if !then_terminated {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        self.builder.position_at_end(else_block);
        let else_val = self.generate_expr(else_expr)?;
        let else_end = self.builder.get_insert_block().unwrap();
        let else_terminated = else_end.get_terminator().is_some();
        if !else_terminated {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        self.builder.position_at_end(merge_block);

        if then_terminated && else_terminated {
            self.builder
                .build_unreachable()
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(self.context.struct_type(&[], false).const_zero().into());
        }

        let mut incoming: Vec<(
            &dyn BasicValue<'ctx>,
            inkwell::basic_block::BasicBlock<'ctx>,
        )> = Vec::new();
        if !then_terminated {
            incoming.push((&then_val, then_end));
        }
        if !else_terminated {
            incoming.push((&else_val, else_end));
        }

        if incoming.len() == 1 {
            Ok(incoming[0].0.as_basic_value_enum())
        } else {
            let phi = self
                .builder
                .build_phi(then_val.get_type(), "ternary_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            for (val, block) in &incoming {
                phi.add_incoming(&[(*val, *block)]);
            }
            Ok(phi.as_basic_value())
        }
    }

    fn generate_struct_literal(
        &mut self,
        name: &str,
        fields: &[(Spanned<String>, Spanned<Expr>)],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Generate field values mapped by name
        let mut field_name_values: Vec<(String, BasicValueEnum)> = Vec::new();
        for (field_name, expr) in fields {
            let val = self.generate_expr(&expr.node)?;
            field_name_values.push((field_name.node.clone(), val));
        }

        let struct_type = *self
            .generated_structs
            .get(name)
            .ok_or_else(|| CodegenError::UndefinedVar(format!("Struct not found: {}", name)))?;

        // Check if this is a union (single LLVM field, multiple logical fields)
        let num_llvm_fields = struct_type.count_fields();
        let num_logical_fields = self.struct_fields.get(name).map(|f| f.len()).unwrap_or(0);
        let is_union = num_llvm_fields == 1 && num_logical_fields > 1;

        let mut struct_val = struct_type.get_undef();
        if is_union {
            // Union: all fields share index 0, need to bitcast if types differ
            if let Some((_, val)) = field_name_values.first() {
                let field_llvm_type = struct_type.get_field_type_at_index(0).unwrap();
                let coerced_val = if val.get_type() != field_llvm_type {
                    // Bitcast through memory for type punning (e.g., f64 -> i64)
                    if val.is_float_value() && field_llvm_type.is_int_type() {
                        let bits = self
                            .builder
                            .build_bitcast(*val, field_llvm_type, "union_bitcast")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        bits
                    } else if val.is_int_value() && field_llvm_type.is_float_type() {
                        let bits = self
                            .builder
                            .build_bitcast(*val, field_llvm_type, "union_bitcast")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        bits
                    } else {
                        *val
                    }
                } else {
                    *val
                };
                struct_val = self
                    .builder
                    .build_insert_value(struct_val, coerced_val, 0, "union_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
            }
        } else if let Some(struct_field_names) = self.struct_fields.get(name) {
            // Reorder fields to match struct definition order
            for (i, def_field_name) in struct_field_names.iter().enumerate() {
                let val = field_name_values
                    .iter()
                    .find(|(n, _)| n == def_field_name)
                    .map(|(_, v)| *v)
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into());
                struct_val = self
                    .builder
                    .build_insert_value(struct_val, val, i as u32, &format!("field_{}", i))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
            }
        } else {
            // Fallback: insert in source order
            for (i, (_, val)) in field_name_values.iter().enumerate() {
                struct_val = self
                    .builder
                    .build_insert_value(struct_val, *val, i as u32, &format!("field_{}", i))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_struct_value();
            }
        }
        Ok(struct_val.into())
    }

    fn generate_field_access(
        &mut self,
        obj: &Expr,
        field: &str,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let obj_val = self.generate_expr(obj)?;

        // Get struct type name from the expression
        let struct_name = self.infer_struct_name(obj)?;

        // Check if this is a union (single-field struct where all "fields" map to index 0)
        if let Some(struct_type) = self.generated_structs.get(&struct_name) {
            let num_fields = struct_type.count_fields();
            if let Some(field_names) = self.struct_fields.get(&struct_name) {
                if num_fields == 1 && field_names.len() > 1 {
                    // This is a union - all fields share index 0
                    let struct_val = obj_val.into_struct_value();
                    let raw_val = self
                        .builder
                        .build_extract_value(struct_val, 0, field)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    // If the field expects f64 but storage is i64, bitcast
                    // Determine expected field type from field name position
                    let field_idx = field_names.iter().position(|f| f == field).unwrap_or(0);
                    let _ = field_idx; // For now, return raw; the caller handles type interpretation
                    return Ok(raw_val);
                }
            }
        }

        // Lookup field index by name
        let field_idx = self.get_field_index(&struct_name, field)?;

        let struct_val = obj_val.into_struct_value();
        self.builder
            .build_extract_value(struct_val, field_idx, field)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Infers the struct name from an expression (for field access and method calls).
    fn infer_struct_name(&self, expr: &Expr) -> CodegenResult<String> {
        match expr {
            Expr::Ident(name) => {
                // Look up in var_struct_types
                if let Some(struct_name) = self.var_struct_types.get(name) {
                    return Ok(struct_name.clone());
                }
                Err(CodegenError::Unsupported(format!(
                    "Cannot infer struct type for variable: {}. Consider adding type annotations.",
                    name
                )))
            }
            Expr::StructLit { name, .. } => Ok(name.node.clone()),
            Expr::Field { expr: inner, field } => {
                // Recursively infer the inner expression's struct type,
                // then look up the field's type (if it's also a struct)
                let parent_struct = self.infer_struct_name(&inner.node)?;
                // Check if the field type is itself a known struct
                if let Some(fields) = self.struct_fields.get(&parent_struct) {
                    let field_idx = fields.iter().position(|f| f == &field.node);
                    if let Some(_idx) = field_idx {
                        // We'd need field type info to resolve nested struct types
                        // For now, just return an error for nested field access
                    }
                }
                Err(CodegenError::Unsupported(format!(
                    "Cannot infer struct type for nested field access: {}.{}",
                    parent_struct, field.node
                )))
            }
            Expr::Call { func, .. } => {
                // Try to infer return type as struct from function name
                if let Expr::Ident(fn_name) = &func.node {
                    // Check if the function name matches a struct constructor pattern
                    if self.generated_structs.contains_key(fn_name.as_str()) {
                        return Ok(fn_name.clone());
                    }
                }
                Err(CodegenError::Unsupported(
                    "Cannot infer struct type for call expression".to_string(),
                ))
            }
            Expr::SelfCall => {
                // @ in a method context refers to the current struct instance
                // Look up "self" in var_struct_types
                if let Some(struct_name) = self.var_struct_types.get("self") {
                    return Ok(struct_name.clone());
                }
                Err(CodegenError::Unsupported(
                    "SelfCall (@) used outside of method context".to_string(),
                ))
            }
            _ => Err(CodegenError::Unsupported(format!(
                "Cannot infer struct type for expression: {:?}",
                expr
            ))),
        }
    }

    /// Gets the field index by name for a struct.
    fn get_field_index(&self, struct_name: &str, field_name: &str) -> CodegenResult<u32> {
        if let Some(fields) = self.struct_fields.get(struct_name) {
            for (idx, name) in fields.iter().enumerate() {
                if name == field_name {
                    return Ok(idx as u32);
                }
            }
            Err(CodegenError::UndefinedVar(format!(
                "Field '{}' not found in struct '{}'",
                field_name, struct_name
            )))
        } else {
            Err(CodegenError::UndefinedVar(format!(
                "Struct '{}' not found",
                struct_name
            )))
        }
    }

    /// Generates a fresh label name.
    fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("{}{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Returns a default/zero value for a given LLVM type.
    fn get_default_value(&self, ty: BasicTypeEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match ty {
            BasicTypeEnum::IntType(it) => it.const_int(0, false).into(),
            BasicTypeEnum::FloatType(ft) => ft.const_float(0.0).into(),
            BasicTypeEnum::PointerType(pt) => pt.const_null().into(),
            BasicTypeEnum::StructType(st) => st.const_zero().into(),
            BasicTypeEnum::ArrayType(at) => at.const_zero().into(),
            BasicTypeEnum::VectorType(vt) => vt.const_zero().into(),
        }
    }

    // ========== Match Expression ==========

    fn generate_match(
        &mut self,
        match_expr: &Spanned<Expr>,
        arms: &[MatchArm],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for match expression".to_string())
        })?;

        // Generate the expression to match against
        let match_val = self.generate_expr(&match_expr.node)?;

        // Create merge block
        let merge_block = self
            .context
            .append_basic_block(fn_value, &self.fresh_label("match.merge"));

        // Track arm results for phi node: (value, block)
        let mut arm_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
            Vec::new();

        // Check if all arms are simple integer literals (can use switch)
        let all_int_literals = arms.iter().all(|arm| {
            matches!(
                &arm.pattern.node,
                Pattern::Literal(Literal::Int(_)) | Pattern::Wildcard
            )
        });

        if all_int_literals && !arms.is_empty() && match_val.is_int_value() {
            // Use LLVM switch instruction for integer pattern matching
            let default_block = self
                .context
                .append_basic_block(fn_value, &self.fresh_label("match.default"));

            // Collect cases and find default arm
            let mut cases: Vec<(IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
                Vec::new();
            let mut default_arm: Option<&MatchArm> = None;
            let mut case_arms: Vec<(&MatchArm, inkwell::basic_block::BasicBlock<'ctx>)> =
                Vec::new();

            for arm in arms {
                match &arm.pattern.node {
                    Pattern::Literal(Literal::Int(n)) => {
                        let arm_block = self
                            .context
                            .append_basic_block(fn_value, &self.fresh_label("match.arm"));
                        let case_val = self.context.i64_type().const_int(*n as u64, true);
                        cases.push((case_val, arm_block));
                        case_arms.push((arm, arm_block));
                    }
                    Pattern::Wildcard => {
                        default_arm = Some(arm);
                    }
                    _ => {}
                }
            }

            // Build switch instruction
            let switch_val = match_val.into_int_value();
            let switch = self
                .builder
                .build_switch(switch_val, default_block, &cases)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let _ = switch; // Suppress unused variable warning

            // Generate arm bodies for integer cases
            for (arm, arm_block) in case_arms {
                self.builder.position_at_end(arm_block);

                // Handle guard if present
                if let Some(guard) = &arm.guard {
                    let guard_pass = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.guard.pass"));
                    let guard_fail = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.guard.fail"));

                    let guard_val = self.generate_expr(&guard.node)?;
                    let guard_bool = if guard_val.is_int_value() {
                        guard_val.into_int_value()
                    } else {
                        self.context.bool_type().const_int(1, false) // Truthy fallback
                    };

                    self.builder
                        .build_conditional_branch(guard_bool, guard_pass, guard_fail)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Guard passed - execute body
                    self.builder.position_at_end(guard_pass);
                    let body_val = self.generate_expr(&arm.body.node)?;
                    let body_end = self.builder.get_insert_block().unwrap();
                    if body_end.get_terminator().is_none() {
                        self.builder
                            .build_unconditional_branch(merge_block)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        arm_results.push((body_val, body_end));
                    }

                    // Guard failed - go to default
                    self.builder.position_at_end(guard_fail);
                    self.builder
                        .build_unconditional_branch(default_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    let body_val = self.generate_expr(&arm.body.node)?;
                    let body_end = self.builder.get_insert_block().unwrap();
                    if body_end.get_terminator().is_none() {
                        self.builder
                            .build_unconditional_branch(merge_block)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        arm_results.push((body_val, body_end));
                    }
                }
            }

            // Generate default arm
            self.builder.position_at_end(default_block);
            if let Some(arm) = default_arm {
                let body_val = self.generate_expr(&arm.body.node)?;
                let default_end = self.builder.get_insert_block().unwrap();
                if default_end.get_terminator().is_none() {
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    arm_results.push((body_val, default_end));
                }
            } else {
                // No default arm - return default value (0)
                let default_val = self.context.i64_type().const_int(0, false);
                self.builder
                    .build_unconditional_branch(merge_block)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                arm_results.push((default_val.into(), default_block));
            }
        } else {
            // Fall back to chained conditional branches for complex patterns
            let mut current_block = self.builder.get_insert_block().unwrap();

            for (i, arm) in arms.iter().enumerate() {
                let is_last = i == arms.len() - 1;
                let next_block = if is_last {
                    merge_block
                } else {
                    self.context
                        .append_basic_block(fn_value, &self.fresh_label("match.check"))
                };
                let arm_body_block = self
                    .context
                    .append_basic_block(fn_value, &self.fresh_label("match.arm"));

                self.builder.position_at_end(current_block);

                // Generate pattern check
                let check_result = self.generate_pattern_check(&arm.pattern, &match_val)?;

                // Handle guard
                if let Some(guard) = &arm.guard {
                    let guard_bind_block = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.guard.bind"));
                    let guard_check_block = self
                        .context
                        .append_basic_block(fn_value, &self.fresh_label("match.guard.check"));

                    // First check pattern
                    self.builder
                        .build_conditional_branch(check_result, guard_bind_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Bind pattern variables for guard to use
                    self.builder.position_at_end(guard_bind_block);
                    self.generate_pattern_bindings(&arm.pattern, &match_val)?;
                    self.builder
                        .build_unconditional_branch(guard_check_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Then check guard
                    self.builder.position_at_end(guard_check_block);
                    let guard_val = self.generate_expr(&guard.node)?;
                    let guard_bool = if guard_val.is_int_value() {
                        let int_val = guard_val.into_int_value();
                        // Convert i64 to i1 if needed
                        if int_val.get_type().get_bit_width() > 1 {
                            self.builder
                                .build_int_compare(
                                    IntPredicate::NE,
                                    int_val,
                                    self.context.i64_type().const_int(0, false),
                                    "guard_bool",
                                )
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        } else {
                            int_val
                        }
                    } else {
                        self.context.bool_type().const_int(1, false)
                    };
                    self.builder
                        .build_conditional_branch(guard_bool, arm_body_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Generate arm body (bindings already done)
                    self.builder.position_at_end(arm_body_block);
                } else {
                    // No guard - branch based on pattern check
                    self.builder
                        .build_conditional_branch(check_result, arm_body_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Generate arm body
                    self.builder.position_at_end(arm_body_block);

                    // Bind pattern variables if needed
                    self.generate_pattern_bindings(&arm.pattern, &match_val)?;
                }

                let body_val = self.generate_expr(&arm.body.node)?;
                let body_end_block = self.builder.get_insert_block().unwrap();
                if body_end_block.get_terminator().is_none() {
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    arm_results.push((body_val, body_end_block));
                }

                current_block = next_block;
            }

            // Handle the case when no arm matched (last block leads to merge)
            // This should be unreachable for exhaustive matches
        }

        // Merge block with phi node
        self.builder.position_at_end(merge_block);

        if arm_results.is_empty() {
            // All arms terminated (return/break) - merge is unreachable
            self.builder
                .build_unreachable()
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(self.context.struct_type(&[], false).const_zero().into())
        } else if arm_results.len() == 1 {
            // Only one arm reaches merge - no phi needed
            Ok(arm_results[0].0)
        } else {
            // Build phi node
            let first_type = arm_results[0].0.get_type();
            let phi = self
                .builder
                .build_phi(first_type, "match_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            for (val, block) in &arm_results {
                phi.add_incoming(&[(val, *block)]);
            }

            Ok(phi.as_basic_value())
        }
    }

    /// Generates code to check if a pattern matches the given value.
    /// Returns an i1 (boolean) value.
    fn generate_pattern_check(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &BasicValueEnum<'ctx>,
    ) -> CodegenResult<IntValue<'ctx>> {
        match &pattern.node {
            Pattern::Wildcard => {
                // Wildcard always matches
                Ok(self.context.bool_type().const_int(1, false))
            }
            Pattern::Ident(name) => {
                // Check if this identifier is a known enum variant (simple variant without data)
                let is_enum_variant = self.enum_variants.iter().any(|((_, v), _)| v == name);
                if is_enum_variant && match_val.is_struct_value() {
                    // Compare the tag value
                    let struct_val = match_val.into_struct_value();
                    let tag_val = self
                        .builder
                        .build_extract_value(struct_val, 0, "enum_tag")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        .into_int_value();
                    let expected_tag = self.get_enum_variant_tag(name);
                    let result = self
                        .builder
                        .build_int_compare(
                            IntPredicate::EQ,
                            tag_val,
                            self.context.i8_type().const_int(expected_tag as u64, false),
                            "variant_check",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    Ok(result)
                } else {
                    // Regular binding - always matches
                    Ok(self.context.bool_type().const_int(1, false))
                }
            }
            Pattern::Literal(lit) => {
                match lit {
                    Literal::Int(n) => {
                        let lit_val = self.context.i64_type().const_int(*n as u64, true);
                        let cmp = self
                            .builder
                            .build_int_compare(
                                IntPredicate::EQ,
                                match_val.into_int_value(),
                                lit_val,
                                "pat_eq",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::Bool(b) => {
                        let lit_val = self.context.bool_type().const_int(*b as u64, false);
                        let match_int = match_val.into_int_value();
                        // Convert to same bit width if needed
                        let cmp = self
                            .builder
                            .build_int_compare(IntPredicate::EQ, match_int, lit_val, "pat_eq")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::Float(f) => {
                        let lit_val = self.context.f64_type().const_float(*f);
                        let cmp = self
                            .builder
                            .build_float_compare(
                                FloatPredicate::OEQ,
                                match_val.into_float_value(),
                                lit_val,
                                "pat_eq",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::String(s) => {
                        // String comparison using strcmp
                        // First, create the pattern string constant
                        let pattern_str = self.generate_string_literal(s)?;

                        // Get strcmp function
                        let strcmp_fn = self.module.get_function("strcmp").unwrap_or_else(|| {
                            let i32_type = self.context.i32_type();
                            let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
                            let fn_type =
                                i32_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
                            self.module.add_function("strcmp", fn_type, None)
                        });

                        // Call strcmp
                        let cmp_result = self
                            .builder
                            .build_call(
                                strcmp_fn,
                                &[(*match_val).into(), pattern_str.into()],
                                "strcmp_result",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        let cmp_int = cmp_result
                            .try_as_basic_value()
                            .left()
                            .ok_or_else(|| {
                                CodegenError::LlvmError("strcmp returned void".to_string())
                            })?
                            .into_int_value();

                        // Check if strcmp returned 0 (equal)
                        let result = self
                            .builder
                            .build_int_compare(
                                IntPredicate::EQ,
                                cmp_int,
                                self.context.i32_type().const_int(0, false),
                                "str_eq",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        Ok(result)
                    }
                }
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let mut lower_check = self.context.bool_type().const_int(1, false);
                let mut upper_check = self.context.bool_type().const_int(1, false);

                // Check lower bound
                if let Some(start_pat) = start {
                    if let Pattern::Literal(Literal::Int(n)) = &start_pat.node {
                        let start_val = self.context.i64_type().const_int(*n as u64, true);
                        lower_check = self
                            .builder
                            .build_int_compare(
                                IntPredicate::SGE,
                                match_val.into_int_value(),
                                start_val,
                                "range_lower",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }

                // Check upper bound
                if let Some(end_pat) = end {
                    if let Pattern::Literal(Literal::Int(n)) = &end_pat.node {
                        let end_val = self.context.i64_type().const_int(*n as u64, true);
                        let cmp = if *inclusive {
                            IntPredicate::SLE
                        } else {
                            IntPredicate::SLT
                        };
                        upper_check = self
                            .builder
                            .build_int_compare(
                                cmp,
                                match_val.into_int_value(),
                                end_val,
                                "range_upper",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }

                // Combine checks
                let result = self
                    .builder
                    .build_and(lower_check, upper_check, "range_check")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                Ok(result)
            }
            Pattern::Or(patterns) => {
                if patterns.is_empty() {
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                let mut result = self.generate_pattern_check(&patterns[0], match_val)?;
                for pat in patterns.iter().skip(1) {
                    let check = self.generate_pattern_check(pat, match_val)?;
                    result = self
                        .builder
                        .build_or(result, check, "or_pat")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }

                Ok(result)
            }
            Pattern::Tuple(patterns) => {
                if patterns.is_empty() {
                    return Ok(self.context.bool_type().const_int(1, false));
                }

                let struct_val = match_val.into_struct_value();
                let mut result = self.context.bool_type().const_int(1, false);

                for (i, pat) in patterns.iter().enumerate() {
                    let elem_val = self
                        .builder
                        .build_extract_value(struct_val, i as u32, &format!("tuple_{}", i))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    let check = self.generate_pattern_check(pat, &elem_val)?;
                    result = self
                        .builder
                        .build_and(result, check, "tuple_check")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }

                Ok(result)
            }
            Pattern::Variant { name, fields: _ } => {
                // Enum variant pattern: check the tag matches
                // Enum is represented as { i8 tag, i64 data }
                let struct_val = match_val.into_struct_value();
                let tag_val = self
                    .builder
                    .build_extract_value(struct_val, 0, "enum_tag")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_int_value();

                // Find expected tag value
                let expected_tag = self.get_enum_variant_tag(&name.node);

                let result = self
                    .builder
                    .build_int_compare(
                        IntPredicate::EQ,
                        tag_val,
                        self.context.i8_type().const_int(expected_tag as u64, false),
                        "variant_check",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                Ok(result)
            }
            Pattern::Struct { name, fields } => {
                // Struct pattern: check field patterns
                let struct_name = &name.node;
                let struct_val = match_val.into_struct_value();
                let mut result = self.context.bool_type().const_int(1, false);

                for (field_name, field_pat) in fields {
                    if let Some(pat) = field_pat {
                        // Get field index
                        let field_idx = self.get_field_index(struct_name, &field_name.node)?;
                        let field_val = self
                            .builder
                            .build_extract_value(struct_val, field_idx, &field_name.node)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        let check = self.generate_pattern_check(pat, &field_val)?;
                        result = self
                            .builder
                            .build_and(result, check, "struct_check")
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }

                Ok(result)
            }
        }
    }

    /// Generates code to bind pattern variables to their matched values.
    fn generate_pattern_bindings(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &BasicValueEnum<'ctx>,
    ) -> CodegenResult<()> {
        match &pattern.node {
            Pattern::Wildcard => {
                // Nothing to bind
                Ok(())
            }
            Pattern::Ident(name) => {
                // Bind identifier to the matched value
                let var_type = match_val.get_type();
                let alloca = self
                    .builder
                    .build_alloca(var_type, name)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, *match_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(name.clone(), (alloca, var_type));
                Ok(())
            }
            Pattern::Literal(_) => {
                // Literals don't bind anything
                Ok(())
            }
            Pattern::Range { .. } => {
                // Ranges don't bind anything
                Ok(())
            }
            Pattern::Or(patterns) => {
                // For or patterns, bind the first pattern (all alternatives should bind the same names)
                if let Some(first) = patterns.first() {
                    self.generate_pattern_bindings(first, match_val)?;
                }
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                let struct_val = match_val.into_struct_value();
                for (i, pat) in patterns.iter().enumerate() {
                    let elem_val = self
                        .builder
                        .build_extract_value(struct_val, i as u32, &format!("tuple_{}", i))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.generate_pattern_bindings(pat, &elem_val)?;
                }
                Ok(())
            }
            Pattern::Variant { name: _, fields } => {
                // Bind variant fields
                // Enum is { i8 tag, i64 data } - extract data and bind
                let struct_val = match_val.into_struct_value();
                let data_val = self
                    .builder
                    .build_extract_value(struct_val, 1, "variant_data")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                // For now, assume single field variant
                if let Some(first_field) = fields.first() {
                    self.generate_pattern_bindings(first_field, &data_val)?;
                }
                Ok(())
            }
            Pattern::Struct { name, fields } => {
                let struct_name = &name.node;
                let struct_val = match_val.into_struct_value();

                for (field_name, field_pat) in fields {
                    let field_idx = self.get_field_index(struct_name, &field_name.node)?;
                    let field_val = self
                        .builder
                        .build_extract_value(struct_val, field_idx, &field_name.node)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    if let Some(pat) = field_pat {
                        // Pattern specified - bind according to pattern
                        self.generate_pattern_bindings(pat, &field_val)?;
                    } else {
                        // Shorthand: `{x}` means `{x: x}`
                        let var_type = field_val.get_type();
                        let alloca = self
                            .builder
                            .build_alloca(var_type, &field_name.node)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.builder
                            .build_store(alloca, field_val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.locals
                            .insert(field_name.node.clone(), (alloca, var_type));
                    }
                }
                Ok(())
            }
        }
    }

    /// Gets the tag value for an enum variant by searching all registered enums.
    ///
    /// This method searches through all registered enum variants to find the tag
    /// for the given variant name. If multiple enums have the same variant name,
    /// the first match is returned.
    fn get_enum_variant_tag(&self, variant_name: &str) -> i32 {
        // Search through all registered enum variants
        for ((_, v_name), tag) in &self.enum_variants {
            if v_name == variant_name {
                return *tag;
            }
        }
        // Variant not found - this could happen for built-in types like Option/Result
        // In such cases, we use a simple heuristic: Some=1, None=0, Ok=0, Err=1
        match variant_name {
            "None" => 0,
            "Some" => 1,
            "Ok" => 0,
            "Err" => 1,
            _ => 0, // Default to 0 for unknown variants
        }
    }

    /// Gets the tag value for an enum variant with explicit enum name.
    ///
    /// This method provides more precise lookup when the enum name is known.
    #[allow(dead_code)]
    fn get_enum_variant_tag_with_enum(&self, enum_name: &str, variant_name: &str) -> i32 {
        self.enum_variants
            .get(&(enum_name.to_string(), variant_name.to_string()))
            .copied()
            .unwrap_or_else(|| self.get_enum_variant_tag(variant_name))
    }

    // ========== Assert ==========

    fn generate_assert(
        &mut self,
        condition: &Expr,
        _message: Option<&Spanned<Expr>>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self
            .current_function
            .ok_or_else(|| CodegenError::LlvmError("No current function for assert".to_string()))?;

        let cond_val = self.generate_expr(condition)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() > 1 {
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "assert_cond",
                    )
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        let pass_block = self.context.append_basic_block(fn_value, "assert_pass");
        let fail_block = self.context.append_basic_block(fn_value, "assert_fail");

        self.builder
            .build_conditional_branch(cond_bool, pass_block, fail_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Fail: call abort
        self.builder.position_at_end(fail_block);
        if let Some(abort_fn) = self.module.get_function("abort") {
            self.builder
                .build_call(abort_fn, &[], "")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }
        self.builder
            .build_unreachable()
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Pass: continue
        self.builder.position_at_end(pass_block);
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    // ========== String Interpolation ==========

    fn generate_string_interp(
        &mut self,
        parts: &[StringInterpPart],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Build a printf format string and collect args
        let mut format_str = String::new();
        let mut args: Vec<BasicMetadataValueEnum> = Vec::new();

        for part in parts {
            match part {
                StringInterpPart::Lit(s) => {
                    // Escape % for printf
                    format_str.push_str(&s.replace('%', "%%"));
                }
                StringInterpPart::Expr(expr) => {
                    let val = self.generate_expr(&expr.node)?;
                    if val.is_int_value() {
                        format_str.push_str("%lld");
                        args.push(val.into());
                    } else if val.is_float_value() {
                        format_str.push_str("%f");
                        args.push(val.into());
                    } else if val.is_pointer_value() {
                        format_str.push_str("%s");
                        args.push(val.into());
                    } else {
                        format_str.push_str("%lld");
                        args.push(val.into());
                    }
                }
            }
        }

        // Generate printf call
        let fmt_val = self.generate_string_literal(&format_str)?;
        let mut all_args: Vec<BasicMetadataValueEnum> = vec![fmt_val.into()];
        all_args.extend(args);

        if let Some(printf_fn) = self.module.get_function("printf") {
            let call = self
                .builder
                .build_call(printf_fn, &all_args, "printf_call")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(call
                .try_as_basic_value()
                .left()
                .unwrap_or_else(|| self.context.struct_type(&[], false).const_zero().into()))
        } else {
            Ok(self.context.struct_type(&[], false).const_zero().into())
        }
    }

    // ========== Let Destructure ==========

    fn generate_let_destructure(
        &mut self,
        pattern: &Pattern,
        value: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let val = self.generate_expr(value)?;

        match pattern {
            Pattern::Tuple(pats) => {
                // Value should be a struct (tuple), extract elements
                if val.is_struct_value() {
                    let struct_val = val.into_struct_value();
                    for (i, pat) in pats.iter().enumerate() {
                        if let Pattern::Ident(name) = &pat.node {
                            let elem = self
                                .builder
                                .build_extract_value(struct_val, i as u32, name)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                            let elem_type = elem.get_type();
                            let alloca = self
                                .builder
                                .build_alloca(elem_type, name)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                            self.builder
                                .build_store(alloca, elem)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                            self.locals.insert(name.clone(), (alloca, elem_type));
                        }
                    }
                } else {
                    // Fallback: if it's an i64 tuple packed value, handle differently
                    // For now, just bind the whole value to the first name
                    if let Some(pat) = pats.first() {
                        if let Pattern::Ident(name) = &pat.node {
                            let val_type = val.get_type();
                            let alloca = self
                                .builder
                                .build_alloca(val_type, name)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                            self.builder
                                .build_store(alloca, val)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                            self.locals.insert(name.clone(), (alloca, val_type));
                        }
                    }
                }
            }
            Pattern::Ident(name) => {
                let val_type = val.get_type();
                let alloca = self
                    .builder
                    .build_alloca(val_type, name)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(name.clone(), (alloca, val_type));
            }
            _ => {}
        }

        Ok(self.context.struct_type(&[], false).const_zero().into())
    }
    // ========== Impl/Method Support ==========

    /// Extracts a struct type name from an AST Type node (if it refers to a struct).
    fn extract_struct_type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Named { name, .. } => {
                // Check if it's a known struct or might be one (not a primitive type)
                if self.generated_structs.contains_key(name) {
                    Some(name.clone())
                } else if !matches!(
                    name.as_str(),
                    "i8" | "i16"
                        | "i32"
                        | "i64"
                        | "i128"
                        | "u8"
                        | "u16"
                        | "u32"
                        | "u64"
                        | "u128"
                        | "f32"
                        | "f64"
                        | "bool"
                        | "str"
                        | "ptr"
                ) && name.chars().next().is_some_and(|c| c.is_uppercase())
                {
                    // Capitalized name that's not a primitive - likely a struct
                    Some(name.clone())
                } else {
                    None
                }
            }
            Type::Ref(inner) | Type::RefMut(inner) => self.extract_struct_type_name(&inner.node),
            _ => None,
        }
    }

    /// Infers the struct type from a value expression (for Let bindings).
    fn infer_value_struct_type(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::StructLit { name, .. } => Some(name.node.clone()),
            Expr::Call { func, .. } => {
                if let Expr::Ident(fn_name) = &func.node {
                    // First check our explicit function->struct return type map
                    if let Some(sn) = self.function_return_structs.get(fn_name) {
                        return Some(sn.clone());
                    }
                    // Fallback: check LLVM return type
                    let fn_value = self
                        .functions
                        .get(fn_name)
                        .copied()
                        .or_else(|| self.module.get_function(fn_name));
                    if let Some(fn_value) = fn_value {
                        let ret_type = fn_value.get_type().get_return_type();
                        if let Some(ret) = ret_type {
                            if ret.is_struct_type() {
                                let struct_type = ret.into_struct_type();
                                for (name, st) in &self.generated_structs {
                                    if *st == struct_type {
                                        return Some(name.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                None
            }
            Expr::MethodCall {
                receiver, method, ..
            } => {
                // Try to infer from method return type
                let struct_name = self.infer_struct_name(&receiver.node).ok()?;
                let qualified = format!("{}_{}", struct_name, method.node);
                let fn_value = self
                    .functions
                    .get(&qualified)
                    .copied()
                    .or_else(|| self.module.get_function(&qualified));
                if let Some(fn_value) = fn_value {
                    let ret_type = fn_value.get_type().get_return_type();
                    if let Some(ret) = ret_type {
                        if ret.is_struct_type() {
                            let struct_type = ret.into_struct_type();
                            for (name, st) in &self.generated_structs {
                                if *st == struct_type {
                                    return Some(name.clone());
                                }
                            }
                        }
                    }
                }
                None
            }
            Expr::Block(stmts) => {
                // Return type of block is the last statement's value
                if let Some(last) = stmts.last() {
                    if let Stmt::Expr(e) = &last.node {
                        return self.infer_value_struct_type(&e.node);
                    }
                }
                None
            }
            Expr::StaticMethodCall {
                type_name, method, ..
            } => {
                // For static method calls like `FunctionSig.new(...)`, the return type
                // is typically the struct itself (constructor pattern)
                let qualified = format!("{}_{}", type_name.node, method.node);
                // First check function_return_structs for explicit mapping
                if let Some(sn) = self.function_return_structs.get(&qualified) {
                    return Some(sn.clone());
                }
                // For common constructor patterns (new, default, from_*, etc.),
                // assume the return type is the struct itself
                let method_name = method.node.as_str();
                if method_name == "new"
                    || method_name == "default"
                    || method_name.starts_with("from_")
                    || method_name.starts_with("with_")
                {
                    // Check if the type_name is a known struct
                    if self.generated_structs.contains_key(&type_name.node) {
                        return Some(type_name.node.clone());
                    }
                }
                // Fallback: check LLVM return type
                let fn_value = self
                    .functions
                    .get(&qualified)
                    .copied()
                    .or_else(|| self.module.get_function(&qualified));
                if let Some(fn_value) = fn_value {
                    let ret_type = fn_value.get_type().get_return_type();
                    if let Some(ret) = ret_type {
                        if ret.is_struct_type() {
                            let struct_type = ret.into_struct_type();
                            for (name, st) in &self.generated_structs {
                                if *st == struct_type {
                                    return Some(name.clone());
                                }
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Declares a method as `TypeName_methodName` function.
    fn declare_method(
        &mut self,
        type_name: &str,
        func: &ast::Function,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        let method_name = format!("{}_{}", type_name, func.name.node);

        // Set up generic substitutions from parent struct and method generics
        let old_substitutions = self.generic_substitutions.clone();
        if let Some(gp_names) = self.struct_generic_params.get(type_name).cloned() {
            for gp_name in &gp_names {
                self.generic_substitutions
                    .entry(gp_name.clone())
                    .or_insert(ResolvedType::I64);
            }
        }
        for gp in &func.generics {
            self.generic_substitutions
                .entry(gp.name.node.clone())
                .or_insert(ResolvedType::I64);
        }

        // Build parameter types: map self -> pointer type (pass by reference)
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();
        for p in &func.params {
            if p.name.node == "self" {
                // self parameter: pass as pointer for mutation visibility
                param_types.push(
                    self.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into(),
                );
            } else {
                let resolved = self.ast_type_to_resolved(&p.ty.node);
                let substituted = self.substitute_type(&resolved);
                param_types.push(self.type_mapper.map_type(&substituted).into());
            }
        }

        let ret_resolved = func
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);
        let ret_substituted = self.substitute_type(&ret_resolved);

        let fn_type = if ret_substituted == ResolvedType::Unit {
            self.context.void_type().fn_type(&param_types, false)
        } else {
            let ret_type = self.type_mapper.map_type(&ret_substituted);
            ret_type.fn_type(&param_types, false)
        };

        // Restore substitutions
        self.generic_substitutions = old_substitutions;

        let fn_value = self.module.add_function(&method_name, fn_type, None);
        self.functions.insert(method_name.clone(), fn_value);

        // Track return struct type for methods
        if let Some(ret_ty) = &func.ret_type {
            if let Some(sn) = self.extract_struct_type_name(&ret_ty.node) {
                self.function_return_structs.insert(method_name, sn);
            }
        }

        Ok(fn_value)
    }

    /// Generates the body of a method declared via `declare_method`.
    fn generate_method(&mut self, type_name: &str, func: &ast::Function) -> CodegenResult<()> {
        let method_name = format!("{}_{}", type_name, func.name.node);
        let fn_value = *self
            .functions
            .get(&method_name)
            .ok_or_else(|| CodegenError::UndefinedFunction(method_name.clone()))?;

        self.current_function = Some(fn_value);
        self.locals.clear();
        self.var_struct_types.clear();
        self.defer_stack.clear();

        // Set up generic substitutions from parent struct and method generics
        let old_substitutions = self.generic_substitutions.clone();
        if let Some(gp_names) = self.struct_generic_params.get(type_name).cloned() {
            for gp_name in &gp_names {
                self.generic_substitutions
                    .entry(gp_name.clone())
                    .or_insert(ResolvedType::I64);
            }
        }
        for gp in &func.generics {
            self.generic_substitutions
                .entry(gp.name.node.clone())
                .or_insert(ResolvedType::I64);
        }

        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate parameters
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();

            if param.name.node == "self" {
                // self is passed as a pointer to the caller's alloca
                // Use it directly as the local pointer (no separate alloca needed)
                let self_ptr = param_value.into_pointer_value();
                let struct_type = self
                    .generated_structs
                    .get(type_name)
                    .copied()
                    .unwrap_or_else(|| self.context.struct_type(&[], false));
                self.locals.insert(
                    "self".to_string(),
                    (self_ptr, BasicTypeEnum::StructType(struct_type)),
                );
                self.var_struct_types
                    .insert("self".to_string(), type_name.to_string());
            } else {
                let resolved = self.ast_type_to_resolved(&param.ty.node);
                let substituted = self.substitute_type(&resolved);
                let param_type = self.type_mapper.map_type(&substituted);

                let alloca = self
                    .builder
                    .build_alloca(param_type, &param.name.node)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder
                    .build_store(alloca, param_value)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals
                    .insert(param.name.node.clone(), (alloca, param_type));

                if let Some(sn) = self.extract_struct_type_name(&param.ty.node) {
                    self.var_struct_types.insert(param.name.node.clone(), sn);
                }
            }
        }

        // Generate body
        let ret_resolved = func
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);
        let ret_substituted = self.substitute_type(&ret_resolved);

        match &func.body {
            ast::FunctionBody::Expr(body_expr) => {
                let body_value = self.generate_expr(&body_expr.node)?;
                self.emit_defer_cleanup()?;
                if ret_substituted == ResolvedType::Unit {
                    self.builder
                        .build_return(None)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    self.builder
                        .build_return(Some(&body_value))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
            }
            ast::FunctionBody::Block(stmts) => {
                let body_value = self.generate_block(stmts)?;
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let body_type_matches =
                            expected_ret_type.is_some_and(|ert| ert == body_value.get_type());
                        if body_type_matches {
                            self.builder
                                .build_return(Some(&body_value))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else if let Some(ert) = expected_ret_type {
                            let default_val = self.get_default_value(ert);
                            self.builder
                                .build_return(Some(&default_val))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else {
                            self.builder
                                .build_return(None)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }
                }
            }
        }

        // Restore generic substitutions
        self.generic_substitutions = old_substitutions;
        self.current_function = None;
        Ok(())
    }

    /// Extracts the type name from an Impl target_type.
    fn get_impl_type_name(ty: &Type) -> Option<String> {
        match ty {
            Type::Named { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    // ========== Built-in pseudo-functions ==========

    fn generate_println_call(
        &mut self,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // println("fmt", args...) => printf("fmt\n", args...)
        let printf_fn = self
            .module
            .get_function("printf")
            .ok_or_else(|| CodegenError::UndefinedFunction("printf".to_string()))?;

        if args.is_empty() {
            // Just print newline
            let newline = self.generate_string_literal("\n")?;
            self.builder
                .build_call(printf_fn, &[newline.into()], "println_call")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        } else {
            // Check if first arg is a string interpolation - handle it specially
            let is_string_interp = matches!(&args[0].node, Expr::StringInterp(_));

            if is_string_interp {
                // String interpolation already calls printf internally
                // Just evaluate it (which prints), then print newline
                let _ = self.generate_expr(&args[0].node)?;
                let newline = self.generate_string_literal("\n")?;
                self.builder
                    .build_call(printf_fn, &[newline.into()], "println_nl")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            } else {
                // First arg is format string - append \n
                let mut arg_values: Vec<BasicMetadataValueEnum> = Vec::new();
                let first_val = self.generate_expr(&args[0].node)?;
                if first_val.is_pointer_value() {
                    arg_values.push(first_val.into());
                    for arg in &args[1..] {
                        let val = self.generate_expr(&arg.node)?;
                        arg_values.push(val.into());
                    }
                    self.builder
                        .build_call(printf_fn, &arg_values, "println_fmt")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    let newline = self.generate_string_literal("\n")?;
                    self.builder
                        .build_call(printf_fn, &[newline.into()], "println_nl")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    arg_values.push(first_val.into());
                    self.builder
                        .build_call(printf_fn, &arg_values, "println_call")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
            }
        }
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_print_call(
        &mut self,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // print("fmt", args...) => printf("fmt", args...)
        let printf_fn = self
            .module
            .get_function("printf")
            .ok_or_else(|| CodegenError::UndefinedFunction("printf".to_string()))?;

        let arg_values: Vec<BasicMetadataValueEnum> = args
            .iter()
            .map(|arg| self.generate_expr(&arg.node).map(|v| v.into()))
            .collect::<CodegenResult<Vec<_>>>()?;

        self.builder
            .build_call(printf_fn, &arg_values, "print_call")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_format_call(
        &mut self,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // format("fmt", args...) => snprintf to heap buffer, return ptr
        // Simplified: just return the format string for now
        if let Some(first) = args.first() {
            self.generate_expr(&first.node)
        } else {
            self.generate_string_literal("")
        }
    }

    fn generate_store_i64(
        &mut self,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // store_i64(ptr: i64, val: i64) -> void
        // Stores a 64-bit integer at the given pointer
        if args.len() < 2 {
            return Err(CodegenError::Unsupported(
                "store_i64 requires 2 args".to_string(),
            ));
        }
        let ptr_val = self.generate_expr(&args[0].node)?;
        let val = self.generate_expr(&args[1].node)?;

        let ptr = self
            .builder
            .build_int_to_ptr(
                ptr_val.into_int_value(),
                self.context.i64_type().ptr_type(AddressSpace::default()),
                "store_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder
            .build_store(ptr, val)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_load_i64(&mut self, args: &[Spanned<Expr>]) -> CodegenResult<BasicValueEnum<'ctx>> {
        // load_i64(ptr: i64) -> i64
        if args.is_empty() {
            return Err(CodegenError::Unsupported(
                "load_i64 requires 1 arg".to_string(),
            ));
        }
        let ptr_val = self.generate_expr(&args[0].node)?;

        let ptr = self
            .builder
            .build_int_to_ptr(
                ptr_val.into_int_value(),
                self.context.i64_type().ptr_type(AddressSpace::default()),
                "load_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder
            .build_load(self.context.i64_type(), ptr, "loaded_i64")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    fn generate_store_byte(
        &mut self,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // store_byte(ptr: i64, val: i64) -> void
        if args.len() < 2 {
            return Err(CodegenError::Unsupported(
                "store_byte requires 2 args".to_string(),
            ));
        }
        let ptr_val = self.generate_expr(&args[0].node)?;
        let val = self.generate_expr(&args[1].node)?;

        let ptr = self
            .builder
            .build_int_to_ptr(
                ptr_val.into_int_value(),
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "store_byte_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let byte_val = self
            .builder
            .build_int_truncate(val.into_int_value(), self.context.i8_type(), "trunc_byte")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder
            .build_store(ptr, byte_val)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_load_byte(
        &mut self,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // load_byte(ptr: i64) -> i64
        if args.is_empty() {
            return Err(CodegenError::Unsupported(
                "load_byte requires 1 arg".to_string(),
            ));
        }
        let ptr_val = self.generate_expr(&args[0].node)?;

        let ptr = self
            .builder
            .build_int_to_ptr(
                ptr_val.into_int_value(),
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "load_byte_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let byte = self
            .builder
            .build_load(self.context.i8_type(), ptr, "loaded_byte")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Zero-extend to i64
        let extended = self
            .builder
            .build_int_z_extend(byte.into_int_value(), self.context.i64_type(), "zext_byte")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(extended.into())
    }

    fn generate_store_f64(
        &mut self,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // store_f64(ptr: i64, val: f64) -> void
        if args.len() < 2 {
            return Err(CodegenError::Unsupported(
                "store_f64 requires 2 args".to_string(),
            ));
        }
        let ptr_val = self.generate_expr(&args[0].node)?;
        let val = self.generate_expr(&args[1].node)?;

        let ptr = self
            .builder
            .build_int_to_ptr(
                ptr_val.into_int_value(),
                self.context.f64_type().ptr_type(AddressSpace::default()),
                "store_f64_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder
            .build_store(ptr, val)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_load_f64(&mut self, args: &[Spanned<Expr>]) -> CodegenResult<BasicValueEnum<'ctx>> {
        // load_f64(ptr: i64) -> f64
        if args.is_empty() {
            return Err(CodegenError::Unsupported(
                "load_f64 requires 1 arg".to_string(),
            ));
        }
        let ptr_val = self.generate_expr(&args[0].node)?;

        let ptr = self
            .builder
            .build_int_to_ptr(
                ptr_val.into_int_value(),
                self.context.f64_type().ptr_type(AddressSpace::default()),
                "load_f64_ptr",
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder
            .build_load(self.context.f64_type(), ptr, "loaded_f64")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_generator() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test_module");
        assert!(!gen.get_ir_string().is_empty());
    }

    #[test]
    fn test_generate_literal_int() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        let lit = Literal::Int(42);
        let result = gen.generate_literal(&lit).unwrap();

        assert!(result.is_int_value());
        let int_val = result.into_int_value();
        assert_eq!(int_val.get_zero_extended_constant(), Some(42));
    }

    #[test]
    fn test_generate_literal_float() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        let lit = Literal::Float(3.14);
        let result = gen.generate_literal(&lit).unwrap();

        assert!(result.is_float_value());
    }

    #[test]
    fn test_generate_literal_bool() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        let lit = Literal::Bool(true);
        let result = gen.generate_literal(&lit).unwrap();

        assert!(result.is_int_value());
        let int_val = result.into_int_value();
        assert_eq!(int_val.get_zero_extended_constant(), Some(1));
    }

    #[test]
    fn test_generate_string_literal() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        // Need a function context for builder position
        let fn_type = context.void_type().fn_type(&[], false);
        let func = gen.module.add_function("__test_str", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        gen.builder.position_at_end(entry);

        let result = gen.generate_string_literal("hello").unwrap();
        assert!(result.is_pointer_value());
    }

    #[test]
    fn test_ast_type_to_resolved() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test");

        // Test basic types
        let i64_type = Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        };
        let resolved = gen.ast_type_to_resolved(&i64_type);
        assert!(matches!(resolved, ResolvedType::I64));

        let bool_type = Type::Named {
            name: "bool".to_string(),
            generics: vec![],
        };
        let resolved = gen.ast_type_to_resolved(&bool_type);
        assert!(matches!(resolved, ResolvedType::Bool));

        let unit_type = Type::Unit;
        let resolved = gen.ast_type_to_resolved(&unit_type);
        assert!(matches!(resolved, ResolvedType::Unit));
    }

    #[test]
    fn test_lambda_counter() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test");

        // Lambda counter should start at 0
        assert_eq!(gen.lambda_counter, 0);
    }

    #[test]
    fn test_generic_substitutions() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        // Initially empty
        assert!(gen.get_generic_substitution("T").is_none());

        // Set substitutions
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), ResolvedType::I64);
        subst.insert("U".to_string(), ResolvedType::Bool);
        gen.set_generic_substitutions(subst);

        // Check substitutions
        assert!(matches!(
            gen.get_generic_substitution("T"),
            Some(ResolvedType::I64)
        ));
        assert!(matches!(
            gen.get_generic_substitution("U"),
            Some(ResolvedType::Bool)
        ));
        assert!(gen.get_generic_substitution("V").is_none());

        // Clear substitutions
        gen.clear_generic_substitutions();
        assert!(gen.get_generic_substitution("T").is_none());
    }

    #[test]
    fn test_mangle_names() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test");

        // Empty type args
        let name = gen.mangle_struct_name("Vec", &[]);
        assert_eq!(name, "Vec");

        // With type args
        let name = gen.mangle_struct_name("Vec", &[ResolvedType::I64]);
        assert_eq!(name, "Vec$i64");

        // Multiple type args
        let name = gen.mangle_struct_name("HashMap", &[ResolvedType::Str, ResolvedType::I64]);
        assert_eq!(name, "HashMap$str_i64");
    }

    #[test]
    fn test_substitute_type() {
        let context = Context::create();
        let mut gen = InkwellCodeGenerator::new(&context, "test");

        // Set substitutions
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), ResolvedType::I64);
        gen.set_generic_substitutions(subst);

        // Substitute a generic type
        let generic_type = ResolvedType::Generic("T".to_string());
        let substituted = gen.substitute_type(&generic_type);
        assert!(matches!(substituted, ResolvedType::I64));

        // Non-generic type stays the same
        let concrete_type = ResolvedType::Bool;
        let substituted = gen.substitute_type(&concrete_type);
        assert!(matches!(substituted, ResolvedType::Bool));
    }
}
