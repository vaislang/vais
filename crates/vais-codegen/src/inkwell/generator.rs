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
use inkwell::{AddressSpace, IntPredicate, FloatPredicate};

use vais_ast::{self as ast, BinaryOp, Expr, IfElse, Literal, MatchArm, Module as VaisModule, Pattern, Spanned, Stmt, Type, UnaryOp};
use vais_types::ResolvedType;

use crate::{CodegenError, CodegenResult, TargetTriple};
use super::builtins;
use super::types::TypeMapper;

/// Loop context for break/continue handling.
struct LoopContext<'ctx> {
    /// Block to jump to on break
    break_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// Block to jump to on continue
    continue_block: inkwell::basic_block::BasicBlock<'ctx>,
}

/// Closure information for captured variables
#[derive(Clone)]
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
}

impl<'ctx> InkwellCodeGenerator<'ctx> {
    /// Creates a new inkwell code generator.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self::new_with_target(context, module_name, TargetTriple::Native)
    }

    /// Creates a new inkwell code generator with specified target.
    pub fn new_with_target(context: &'ctx Context, module_name: &str, target: TargetTriple) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let type_mapper = TypeMapper::new(context);

        // Set target triple if not native
        if target != TargetTriple::Native {
            module.set_triple(&inkwell::targets::TargetTriple::create(target.triple_str()));
        }

        let mut gen = Self {
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
        };

        // Declare built-in functions
        builtins::declare_builtins(context, &gen.module);

        gen
    }

    /// Generates code for an entire Vais module.
    pub fn generate_module(&mut self, vais_module: &VaisModule) -> CodegenResult<()> {
        // First pass: collect all function signatures and struct definitions
        for item in &vais_module.items {
            match item {
                ast::Item::Function(func) => {
                    self.declare_function(func)?;
                }
                ast::Item::Struct(s) => {
                    self.define_struct(s)?;
                }
                ast::Item::Enum(e) => {
                    self.define_enum(e)?;
                }
                _ => {}
            }
        }

        // Second pass: generate function bodies
        for item in &vais_module.items {
            if let ast::Item::Function(func) = item {
                self.generate_function(func)?;
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
        self.module
            .print_to_file(path)
            .map_err(|e| e.to_string())
    }

    // ========== Declaration Phase ==========

    fn declare_function(&mut self, func: &ast::Function) -> CodegenResult<FunctionValue<'ctx>> {
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| self.type_mapper.map_type(&p.ty).into())
            .collect();

        let fn_type = if func.return_type == ResolvedType::Unit {
            self.context.void_type().fn_type(&param_types, false)
        } else {
            let ret_type = self.type_mapper.map_type(&func.return_type);
            ret_type.fn_type(&param_types, false)
        };

        let fn_value = self.module.add_function(&func.name, fn_type, None);
        self.functions.insert(func.name.clone(), fn_value);

        Ok(fn_value)
    }

    fn define_struct(&mut self, s: &ast::Struct) -> CodegenResult<StructType<'ctx>> {
        let field_types: Vec<BasicTypeEnum> = s
            .fields
            .iter()
            .map(|f| self.type_mapper.map_type(&f.ty))
            .collect();

        // Store field names for later field access lookups
        let field_names: Vec<String> = s.fields.iter().map(|f| f.name.clone()).collect();
        self.struct_fields.insert(s.name.clone(), field_names);

        let struct_type = self.context.struct_type(&field_types, false);
        self.type_mapper.register_struct(&s.name, struct_type);
        self.generated_structs.insert(s.name.clone(), struct_type);

        Ok(struct_type)
    }

    fn define_enum(&mut self, e: &ast::Enum) -> CodegenResult<StructType<'ctx>> {
        // Enums are represented as tagged unions: { tag: i8, data: max_variant_size }
        // For simplicity, use { i8, i64 } to hold any primitive
        let tag_type = self.context.i8_type();
        let data_type = self.context.i64_type();
        let enum_type = self.context.struct_type(&[tag_type.into(), data_type.into()], false);

        // Register variant tags: each variant gets a sequential tag starting from 0
        let enum_name = e.name.node.clone();
        for (tag, variant) in e.variants.iter().enumerate() {
            self.enum_variants.insert(
                (enum_name.clone(), variant.name.node.clone()),
                tag as i32,
            );
        }

        self.type_mapper.register_struct(&e.name, enum_type);
        self.generated_structs.insert(enum_name, enum_type);

        Ok(enum_type)
    }

    // ========== Code Generation Phase ==========

    fn generate_function(&mut self, func: &ast::Function) -> CodegenResult<()> {
        let fn_value = *self
            .functions
            .get(&func.name)
            .ok_or_else(|| CodegenError::UndefinedFunction(func.name.clone()))?;

        self.current_function = Some(fn_value);
        self.locals.clear();

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate space for parameters
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();
            let param_type = self.type_mapper.map_type(&param.ty);
            let alloca = self.builder.build_alloca(param_type, &param.name)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder.build_store(alloca, param_value)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals.insert(param.name.clone(), (alloca, param_type));
        }

        // Generate function body
        let body_value = self.generate_expr(&func.body)?;

        // Add return if needed
        if func.return_type == ResolvedType::Unit {
            self.builder.build_return(None)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        } else {
            self.builder.build_return(Some(&body_value))
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

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
            Expr::If { cond, then, else_ } => self.generate_if_expr(&cond.node, then, else_.as_ref()),
            Expr::Loop { pattern, iter, body } => self.generate_loop(pattern.as_ref(), iter.as_deref(), body),
            Expr::While { condition, body } => self.generate_while_loop(condition, body),
            Expr::Match { expr: match_expr, arms } => self.generate_match(match_expr, arms),

            // Struct
            Expr::StructLit { name, fields } => self.generate_struct_literal(&name.node, fields),
            Expr::Field { expr: obj, field } => self.generate_field_access(&obj.node, &field.node),

            // Array/Tuple/Index
            Expr::Array(elements) => self.generate_array(elements),
            Expr::Tuple(elements) => self.generate_tuple(elements),
            Expr::Index { expr: arr, index } => self.generate_index(&arr.node, &index.node),

            // Method call
            Expr::MethodCall { receiver, method, args } => {
                self.generate_method_call(&receiver.node, &method.node, args)
            }

            // Lambda/Closure
            Expr::Lambda { params, body, captures } => {
                self.generate_lambda(params, &body.node, captures)
            }

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
                // Get address of inner expression
                let val = self.generate_expr(&inner.node)?;
                Ok(val) // Simplified - needs proper lvalue handling
            }
            Expr::Deref(inner) => {
                let ptr = self.generate_expr(&inner.node)?;
                let ptr_val = ptr.into_pointer_value();
                self.builder
                    .build_load(self.context.i64_type(), ptr_val, "deref")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
            }

            // Type cast
            Expr::Cast { expr, ty: _ } => {
                // For now, just evaluate the expression and return as-is
                // Most casts between same-size types don't need actual conversion
                self.generate_expr(&expr.node)
            }

            // Range
            Expr::Range { start, end, inclusive } => {
                self.generate_range(start.as_deref(), end.as_deref(), *inclusive)
            }

            // Ternary
            Expr::Ternary { cond, then, else_ } => {
                self.generate_ternary(&cond.node, &then.node, &else_.node)
            }

            _ => Err(CodegenError::Unsupported(format!(
                "Expression kind not yet implemented: {:?}",
                expr
            ))),
        }
    }

    fn generate_literal(&mut self, lit: &Literal) -> CodegenResult<BasicValueEnum<'ctx>> {
        match lit {
            Literal::Int(n) => {
                Ok(self.context.i64_type().const_int(*n as u64, true).into())
            }
            Literal::Float(f) => {
                Ok(self.context.f64_type().const_float(*f).into())
            }
            Literal::Bool(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }
            Literal::Char(c) => {
                Ok(self.context.i32_type().const_int(*c as u64, false).into())
            }
            Literal::String(s) => self.generate_string_literal(s),
            Literal::Unit => {
                // Return empty struct for unit
                let unit_type = self.context.struct_type(&[], false);
                Ok(unit_type.const_zero().into())
            }
        }
    }

    fn generate_string_literal(&mut self, s: &str) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Check if we already have this string
        if let Some(global) = self.string_constants.get(s) {
            let ptr = self.builder.build_pointer_cast(
                global.as_pointer_value(),
                self.context.ptr_type(AddressSpace::default()),
                "str_ptr",
            ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
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

        let ptr = self.builder.build_pointer_cast(
            global.as_pointer_value(),
            self.context.ptr_type(AddressSpace::default()),
            "str_ptr",
        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(ptr.into())
    }

    fn generate_var(&mut self, name: &str) -> CodegenResult<BasicValueEnum<'ctx>> {
        let (ptr, var_type) = self
            .locals
            .get(name)
            .ok_or_else(|| CodegenError::UndefinedVar(name.to_string()))?;

        let value = self.builder.build_load(
            *var_type,
            *ptr,
            name,
        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(value)
    }

    fn generate_binary(
        &mut self,
        op: BinaryOp,
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
        op: BinaryOp,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let result = match op {
            BinaryOp::Add => self.builder.build_int_add(lhs, rhs, "add"),
            BinaryOp::Sub => self.builder.build_int_sub(lhs, rhs, "sub"),
            BinaryOp::Mul => self.builder.build_int_mul(lhs, rhs, "mul"),
            BinaryOp::Div => self.builder.build_int_signed_div(lhs, rhs, "div"),
            BinaryOp::Mod => self.builder.build_int_signed_rem(lhs, rhs, "rem"),
            BinaryOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, lhs, rhs, "eq"),
            BinaryOp::Ne => self.builder.build_int_compare(IntPredicate::NE, lhs, rhs, "ne"),
            BinaryOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, lhs, rhs, "lt"),
            BinaryOp::Le => self.builder.build_int_compare(IntPredicate::SLE, lhs, rhs, "le"),
            BinaryOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, lhs, rhs, "gt"),
            BinaryOp::Ge => self.builder.build_int_compare(IntPredicate::SGE, lhs, rhs, "ge"),
            BinaryOp::And => self.builder.build_and(lhs, rhs, "and"),
            BinaryOp::Or => self.builder.build_or(lhs, rhs, "or"),
            BinaryOp::BitAnd => self.builder.build_and(lhs, rhs, "bitand"),
            BinaryOp::BitOr => self.builder.build_or(lhs, rhs, "bitor"),
            BinaryOp::BitXor => self.builder.build_xor(lhs, rhs, "bitxor"),
            BinaryOp::Shl => self.builder.build_left_shift(lhs, rhs, "shl"),
            BinaryOp::Shr => self.builder.build_right_shift(lhs, rhs, true, "shr"),
            _ => return Err(CodegenError::Unsupported(format!("Binary op: {:?}", op))),
        };
        result.map(|v| v.into()).map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    fn generate_float_binary(
        &mut self,
        op: BinaryOp,
        lhs: inkwell::values::FloatValue<'ctx>,
        rhs: inkwell::values::FloatValue<'ctx>,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let result = match op {
            BinaryOp::Add => self.builder.build_float_add(lhs, rhs, "fadd").map(|v| v.into()),
            BinaryOp::Sub => self.builder.build_float_sub(lhs, rhs, "fsub").map(|v| v.into()),
            BinaryOp::Mul => self.builder.build_float_mul(lhs, rhs, "fmul").map(|v| v.into()),
            BinaryOp::Div => self.builder.build_float_div(lhs, rhs, "fdiv").map(|v| v.into()),
            BinaryOp::Mod => self.builder.build_float_rem(lhs, rhs, "frem").map(|v| v.into()),
            BinaryOp::Eq => self.builder.build_float_compare(FloatPredicate::OEQ, lhs, rhs, "feq").map(|v| v.into()),
            BinaryOp::Ne => self.builder.build_float_compare(FloatPredicate::ONE, lhs, rhs, "fne").map(|v| v.into()),
            BinaryOp::Lt => self.builder.build_float_compare(FloatPredicate::OLT, lhs, rhs, "flt").map(|v| v.into()),
            BinaryOp::Le => self.builder.build_float_compare(FloatPredicate::OLE, lhs, rhs, "fle").map(|v| v.into()),
            BinaryOp::Gt => self.builder.build_float_compare(FloatPredicate::OGT, lhs, rhs, "fgt").map(|v| v.into()),
            BinaryOp::Ge => self.builder.build_float_compare(FloatPredicate::OGE, lhs, rhs, "fge").map(|v| v.into()),
            _ => return Err(CodegenError::Unsupported(format!("Float binary op: {:?}", op))),
        };
        result.map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    fn generate_unary(&mut self, op: UnaryOp, operand: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
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
            UnaryOp::Not => {
                self.builder
                    .build_not(val.into_int_value(), "not")
                    .map(|v| v.into())
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
            }
            UnaryOp::BitNot => {
                self.builder
                    .build_not(val.into_int_value(), "bitnot")
                    .map(|v| v.into())
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
            }
            UnaryOp::Deref => {
                // Load from pointer
                let ptr = val.into_pointer_value();
                self.builder
                    .build_load(self.context.i64_type(), ptr, "deref")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
            }
            UnaryOp::Ref => {
                // Get address - operand must be an lvalue
                // For now, return the value as-is (simplified)
                Ok(val)
            }
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
            _ => return Err(CodegenError::Unsupported("Indirect calls not yet supported".to_string())),
        };

        // Get function value
        let fn_value = self
            .functions
            .get(&fn_name)
            .or_else(|| self.module.get_function(&fn_name))
            .ok_or_else(|| CodegenError::UndefinedFunction(fn_name.clone()))?;

        // Generate arguments
        let arg_values: Vec<BasicMetadataValueEnum> = args
            .iter()
            .map(|arg| self.generate_expr(&arg.node).map(|v| v.into()))
            .collect::<CodegenResult<Vec<_>>>()?;

        // Build call
        let call = self
            .builder
            .build_call(*fn_value, &arg_values, "call")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Return call result or unit
        call.try_as_basic_value()
            .left()
            .ok_or_else(|| {
                // Void function - return unit
                Ok(self.context.struct_type(&[], false).const_zero().into())
            })
            .unwrap_or_else(|v| v)
    }

    fn generate_block(&mut self, stmts: &[Spanned<Stmt>]) -> CodegenResult<BasicValueEnum<'ctx>> {
        let mut last_value: BasicValueEnum = self.context.struct_type(&[], false).const_zero().into();

        for stmt in stmts {
            last_value = self.generate_stmt(&stmt.node)?;
        }

        Ok(last_value)
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> CodegenResult<BasicValueEnum<'ctx>> {
        match stmt {
            Stmt::Let { name, ty, value, .. } => {
                let val = self.generate_expr(&value.node)?;
                let var_type = self.type_mapper.map_type(ty);
                let alloca = self.builder.build_alloca(var_type, &name.node)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder.build_store(alloca, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(name.node.clone(), (alloca, var_type));
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Expr(expr) => self.generate_expr(&expr.node),
            Stmt::Return(Some(expr)) => {
                let val = self.generate_expr(&expr.node)?;
                self.builder.build_return(Some(&val))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Return(None) => {
                self.builder.build_return(None)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Break(value) => self.generate_break(value.as_ref().map(|v| &v.node)),
            Stmt::Continue => self.generate_continue(),
            Stmt::Defer(expr) => {
                // Add deferred expression to stack (will be executed on function exit)
                // For now, just return unit - full defer requires more infrastructure
                let _ = expr;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
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
                self.builder.build_int_compare(
                    IntPredicate::NE,
                    int_val,
                    int_val.get_type().const_int(0, false),
                    "cond_bool"
                ).map_err(|e| CodegenError::LlvmError(e.to_string()))?
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
        self.builder.build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Then block
        self.builder.position_at_end(then_block);
        let then_val = self.generate_block(then_stmts)?;
        self.builder.build_unconditional_branch(merge_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let then_end_block = self.builder.get_insert_block().unwrap();

        // Else block
        self.builder.position_at_end(else_block);
        let else_val = if let Some(else_branch) = else_branch {
            self.generate_if_else(else_branch)?
        } else {
            self.context.struct_type(&[], false).const_zero().into()
        };
        self.builder.build_unconditional_branch(merge_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let else_end_block = self.builder.get_insert_block().unwrap();

        // Merge block with phi
        self.builder.position_at_end(merge_block);

        // Build phi node if both branches return values
        if then_val.get_type() == else_val.get_type() {
            let phi = self.builder.build_phi(then_val.get_type(), "if_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            phi.add_incoming(&[(&then_val, then_end_block), (&else_val, else_end_block)]);
            Ok(phi.as_basic_value())
        } else {
            Ok(then_val)
        }
    }

    fn generate_if_else(&mut self, if_else: &IfElse) -> CodegenResult<BasicValueEnum<'ctx>> {
        match if_else {
            IfElse::Else(stmts) => self.generate_block(stmts),
            IfElse::ElseIf(cond, then_stmts, else_branch) => {
                self.generate_if_expr(&cond.node, then_stmts, else_branch.as_ref().map(|b| b.as_ref()))
            }
        }
    }

    // ========== Loop Expression ==========

    fn generate_loop(
        &mut self,
        pattern: Option<&Spanned<Pattern>>,
        iter: Option<&Spanned<Expr>>,
        body: &[Spanned<Stmt>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for loop".to_string())
        })?;

        let loop_start = self.context.append_basic_block(fn_value, &self.fresh_label("loop.start"));
        let loop_body = self.context.append_basic_block(fn_value, &self.fresh_label("loop.body"));
        let loop_end = self.context.append_basic_block(fn_value, &self.fresh_label("loop.end"));

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            break_block: loop_end,
            continue_block: loop_start,
        });

        // Branch to loop start
        self.builder.build_unconditional_branch(loop_start)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop start - check condition if present
        self.builder.position_at_end(loop_start);

        if let Some(iter_expr) = iter {
            // Conditional loop (while-like)
            let cond_val = self.generate_expr(&iter_expr.node)?;
            let cond_bool = if cond_val.is_int_value() {
                let int_val = cond_val.into_int_value();
                if int_val.get_type().get_bit_width() > 1 {
                    self.builder.build_int_compare(
                        IntPredicate::NE,
                        int_val,
                        int_val.get_type().const_int(0, false),
                        "loop_cond"
                    ).map_err(|e| CodegenError::LlvmError(e.to_string()))?
                } else {
                    int_val
                }
            } else {
                self.context.bool_type().const_int(1, false)
            };

            self.builder.build_conditional_branch(cond_bool, loop_body, loop_end)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        } else {
            // Infinite loop
            self.builder.build_unconditional_branch(loop_body)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Loop body
        self.builder.position_at_end(loop_body);

        // Bind pattern if present
        if let Some(_pat) = pattern {
            // For now, pattern binding in for loops is not fully implemented
            // This would require iterator support
        }

        let _body_val = self.generate_block(body)?;

        // Branch back to loop start (if not terminated by break/return)
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(loop_start)
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

        let loop_cond = self.context.append_basic_block(fn_value, &self.fresh_label("while.cond"));
        let loop_body = self.context.append_basic_block(fn_value, &self.fresh_label("while.body"));
        let loop_end = self.context.append_basic_block(fn_value, &self.fresh_label("while.end"));

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            break_block: loop_end,
            continue_block: loop_cond,
        });

        // Branch to condition check
        self.builder.build_unconditional_branch(loop_cond)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Condition block
        self.builder.position_at_end(loop_cond);
        let cond_val = self.generate_expr(&condition.node)?;
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() > 1 {
                self.builder.build_int_compare(
                    IntPredicate::NE,
                    int_val,
                    int_val.get_type().const_int(0, false),
                    "while_cond"
                ).map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        self.builder.build_conditional_branch(cond_bool, loop_body, loop_end)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Loop body
        self.builder.position_at_end(loop_body);
        let _body_val = self.generate_block(body)?;

        // Branch back to condition (if not terminated by break/return)
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(loop_cond)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        // Loop end
        self.builder.position_at_end(loop_end);
        self.loop_stack.pop();

        // While loops return unit
        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_break(&mut self, value: Option<&Expr>) -> CodegenResult<BasicValueEnum<'ctx>> {
        let loop_ctx = self.loop_stack.last()
            .ok_or_else(|| CodegenError::Unsupported("break outside of loop".to_string()))?;

        // Generate value if present (for loop with value)
        if let Some(val_expr) = value {
            let _val = self.generate_expr(val_expr)?;
            // In a full implementation, this would be used for loop-with-value
        }

        let break_block = loop_ctx.break_block;
        self.builder.build_unconditional_branch(break_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    fn generate_continue(&mut self) -> CodegenResult<BasicValueEnum<'ctx>> {
        let loop_ctx = self.loop_stack.last()
            .ok_or_else(|| CodegenError::Unsupported("continue outside of loop".to_string()))?;

        let continue_block = loop_ctx.continue_block;
        self.builder.build_unconditional_branch(continue_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        Ok(self.context.struct_type(&[], false).const_zero().into())
    }

    // ========== Array/Tuple/Index ==========

    fn generate_array(&mut self, elements: &[Spanned<Expr>]) -> CodegenResult<BasicValueEnum<'ctx>> {
        if elements.is_empty() {
            // Empty array - return null pointer
            return Ok(self.context.ptr_type(AddressSpace::default()).const_null().into());
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
        let array_ptr = self.builder.build_alloca(array_type, "array")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store each element
        for (i, val) in values.iter().enumerate() {
            let idx = self.context.i64_type().const_int(i as u64, false);
            let elem_ptr = unsafe {
                self.builder.build_gep(
                    array_type,
                    array_ptr,
                    &[self.context.i64_type().const_int(0, false), idx],
                    &format!("array_elem_{}", i)
                ).map_err(|e| CodegenError::LlvmError(e.to_string()))?
            };
            self.builder.build_store(elem_ptr, *val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        }

        Ok(array_ptr.into())
    }

    fn generate_tuple(&mut self, elements: &[Spanned<Expr>]) -> CodegenResult<BasicValueEnum<'ctx>> {
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
            tuple_val = self.builder.build_insert_value(tuple_val, *val, i as u32, "tuple")
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
            self.builder.build_gep(
                self.context.i64_type(), // Assume i64 elements for now
                arr_ptr,
                &[idx_int],
                "elem_ptr"
            ).map_err(|e| CodegenError::LlvmError(e.to_string()))?
        };

        // Load element
        self.builder.build_load(self.context.i64_type(), elem_ptr, "elem")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    // ========== Method Call ==========

    fn generate_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // For now, transform method call to function call with receiver as first arg
        // e.g., obj.method(a, b) -> Type_method(obj, a, b)

        let receiver_val = self.generate_expr(receiver)?;

        // Get method function name (would need type info for proper resolution)
        let fn_name = method.to_string();

        // Get function value
        let fn_value = self.module.get_function(&fn_name)
            .ok_or_else(|| CodegenError::UndefinedFunction(fn_name.clone()))?;

        // Generate arguments (receiver first)
        let mut arg_values: Vec<BasicMetadataValueEnum> = vec![receiver_val.into()];
        for arg in args {
            arg_values.push(self.generate_expr(&arg.node)?.into());
        }

        // Build call
        let call = self.builder
            .build_call(fn_value, &arg_values, "method_call")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        call.try_as_basic_value()
            .left()
            .ok_or_else(|| Ok(self.context.struct_type(&[], false).const_zero().into()))
            .unwrap_or_else(|v| v)
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
        let mut captured_vars: Vec<(String, BasicValueEnum<'ctx>, BasicTypeEnum<'ctx>)> = Vec::new();
        for cap_name in captures {
            if let Some((ptr, var_type)) = self.locals.get(cap_name) {
                // Load the captured value
                let val = self.builder.build_load(*var_type, *ptr, &format!("cap_{}", cap_name))
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
            let ty = self.ast_type_to_resolved(&p.ty.node);
            param_types.push(self.type_mapper.map_type(&ty).into());
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
            let alloca = self.builder.build_alloca(*cap_type, &format!("__cap_{}", cap_name))
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder.build_store(alloca, param_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals.insert(cap_name.clone(), (alloca, *cap_type));
            param_idx += 1;
        }

        // Register original parameters
        for p in params {
            let param_val = lambda_fn.get_nth_param(param_idx).unwrap();
            let ty = self.ast_type_to_resolved(&p.ty.node);
            let param_type = self.type_mapper.map_type(&ty);
            let alloca = self.builder.build_alloca(param_type, &p.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder.build_store(alloca, param_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals.insert(p.name.node.clone(), (alloca, param_type));
            param_idx += 1;
        }

        // Generate lambda body
        let body_val = self.generate_expr(body)?;

        // Add return
        self.builder.build_return(Some(&body_val))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Restore context
        self.current_function = saved_function;
        self.locals = saved_locals;
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        // Return function pointer as i64
        // If there are captures, we need to create a closure struct
        if captured_vars.is_empty() {
            // No captures - just return function pointer as i64
            let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
            let fn_int = self.builder.build_ptr_to_int(
                fn_ptr,
                self.context.i64_type(),
                "lambda_ptr"
            ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            Ok(fn_int.into())
        } else {
            // With captures - create closure struct { fn_ptr, captures... }
            // For simplicity, return function pointer as i64
            // Full closure support would require creating a struct and trampoline
            let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
            let fn_int = self.builder.build_ptr_to_int(
                fn_ptr,
                self.context.i64_type(),
                "lambda_ptr"
            ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Store captured values for later use (in a real implementation,
            // these would be packed into a closure struct)
            // For now, we just return the function pointer
            Ok(fn_int.into())
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
                "u8" => ResolvedType::U8,
                "u16" => ResolvedType::U16,
                "u32" => ResolvedType::U32,
                "u64" => ResolvedType::U64,
                "f32" => ResolvedType::F32,
                "f64" => ResolvedType::F64,
                "bool" => ResolvedType::Bool,
                "str" => ResolvedType::Str,
                _ => {
                    // Single uppercase letter is likely a generic type parameter
                    if name.len() == 1 && name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        ResolvedType::Generic(name.clone())
                    } else if !generics.is_empty() {
                        // Named type with generics
                        let generic_types: Vec<ResolvedType> = generics
                            .iter()
                            .map(|g| self.ast_type_to_resolved(&g.node))
                            .collect();
                        ResolvedType::Struct(name.clone(), generic_types)
                    } else {
                        // Simple named type
                        ResolvedType::Struct(name.clone(), vec![])
                    }
                }
            },
            Type::Unit => ResolvedType::Unit,
            Type::Pointer(inner) => ResolvedType::Ptr(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::Array(inner) => ResolvedType::Array(Box::new(self.ast_type_to_resolved(&inner.node)), 0),
            Type::Tuple(elems) => {
                let elem_types: Vec<ResolvedType> = elems
                    .iter()
                    .map(|e| self.ast_type_to_resolved(&e.node))
                    .collect();
                ResolvedType::Tuple(elem_types)
            }
            Type::Fn { params, ret } => {
                let param_types: Vec<ResolvedType> = params
                    .iter()
                    .map(|p| self.ast_type_to_resolved(&p.node))
                    .collect();
                let ret_type = self.ast_type_to_resolved(&ret.node);
                ResolvedType::Function(param_types, Box::new(ret_type))
            }
            Type::Optional(inner) => ResolvedType::Option(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::Result(inner) => ResolvedType::Result(
                Box::new(self.ast_type_to_resolved(&inner.node)),
                Box::new(ResolvedType::Str) // Default error type
            ),
            Type::Ref(inner) => ResolvedType::Ptr(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::RefMut(inner) => ResolvedType::Ptr(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::Infer => ResolvedType::I64, // Default to i64 for unresolved types
            _ => ResolvedType::I64, // Fallback
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

    fn generate_assign(&mut self, target: &Expr, value: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        let val = self.generate_expr(value)?;

        match target {
            Expr::Ident(name) => {
                if let Some((ptr, var_type)) = self.locals.get(name).cloned() {
                    self.builder.build_store(ptr, val)
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
                        let struct_type = self.generated_structs.get(&struct_name)
                            .ok_or_else(|| CodegenError::UndefinedVar(struct_name.clone()))?;

                        let field_ptr = self.builder.build_struct_gep(
                            *struct_type,
                            ptr,
                            field_idx,
                            "field_ptr"
                        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        self.builder.build_store(field_ptr, val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        return Ok(val);
                    }
                }
                Err(CodegenError::Unsupported("Complex field assignment".to_string()))
            }
            Expr::Index { expr: arr, index } => {
                // Array index assignment
                let arr_val = self.generate_expr(&arr.node)?;
                let idx_val = self.generate_expr(&index.node)?;

                let arr_ptr = arr_val.into_pointer_value();
                let idx_int = idx_val.into_int_value();

                let elem_ptr = unsafe {
                    self.builder.build_gep(
                        val.get_type(),
                        arr_ptr,
                        &[idx_int],
                        "elem_ptr"
                    ).map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };

                self.builder.build_store(elem_ptr, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(val)
            }
            _ => Err(CodegenError::Unsupported("Assignment target".to_string())),
        }
    }

    fn generate_assign_op(
        &mut self,
        op: BinaryOp,
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
        match target {
            Expr::Ident(name) => {
                if let Some((ptr, _)) = self.locals.get(name).cloned() {
                    self.builder.build_store(ptr, result)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
            }
            _ => {}
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
        let range_type = self.context.struct_type(&[
            self.context.i64_type().into(),
            self.context.i64_type().into(),
        ], false);

        let start_val = if let Some(s) = start {
            self.generate_expr(&s.node)?
        } else {
            self.context.i64_type().const_int(0, false).into()
        };

        let end_val = if let Some(e) = end {
            self.generate_expr(&e.node)?
        } else {
            self.context.i64_type().const_int(i64::MAX as u64, false).into()
        };

        let mut range_val = range_type.get_undef();
        range_val = self.builder.build_insert_value(range_val, start_val, 0, "range_start")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
            .into_struct_value();
        range_val = self.builder.build_insert_value(range_val, end_val, 1, "range_end")
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
                self.builder.build_int_compare(
                    IntPredicate::NE,
                    int_val,
                    int_val.get_type().const_int(0, false),
                    "ternary_cond"
                ).map_err(|e| CodegenError::LlvmError(e.to_string()))?
            } else {
                int_val
            }
        } else {
            self.context.bool_type().const_int(1, false)
        };

        let then_block = self.context.append_basic_block(fn_value, "ternary_then");
        let else_block = self.context.append_basic_block(fn_value, "ternary_else");
        let merge_block = self.context.append_basic_block(fn_value, "ternary_merge");

        self.builder.build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        self.builder.position_at_end(then_block);
        let then_val = self.generate_expr(then_expr)?;
        self.builder.build_unconditional_branch(merge_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let then_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(else_block);
        let else_val = self.generate_expr(else_expr)?;
        self.builder.build_unconditional_branch(merge_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let else_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(merge_block);
        let phi = self.builder.build_phi(then_val.get_type(), "ternary_result")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        phi.add_incoming(&[(&then_val, then_end), (&else_val, else_end)]);

        Ok(phi.as_basic_value())
    }

    fn generate_struct_literal(
        &mut self,
        name: &str,
        fields: &[(Spanned<String>, Spanned<Expr>)],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let struct_type = self.generated_structs.get(name)
            .ok_or_else(|| CodegenError::UndefinedVar(format!("Struct not found: {}", name)))?;

        // Generate field values
        let mut field_values: Vec<BasicValueEnum> = Vec::new();
        for (_, expr) in fields {
            field_values.push(self.generate_expr(&expr.node)?);
        }

        // Create struct value
        let struct_val = struct_type.const_named_struct(&field_values);
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

        // Lookup field index by name
        let field_idx = self.get_field_index(&struct_name, field)?;

        let struct_val = obj_val.into_struct_value();
        self.builder.build_extract_value(struct_val, field_idx, field)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Infers the struct name from an expression (for field access).
    fn infer_struct_name(&self, expr: &Expr) -> CodegenResult<String> {
        match expr {
            Expr::Ident(name) => {
                // Look up variable type - would need type info stored with locals
                // For now, return an error if we can't determine the type
                Err(CodegenError::Unsupported(format!(
                    "Cannot infer struct type for variable: {}. Consider adding type annotations.",
                    name
                )))
            }
            Expr::StructLit { name, .. } => Ok(name.node.clone()),
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
        let merge_block = self.context.append_basic_block(fn_value, &self.fresh_label("match.merge"));

        // Track arm results for phi node: (value, block)
        let mut arm_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();

        // Check if all arms are simple integer literals (can use switch)
        let all_int_literals = arms.iter().all(|arm| {
            matches!(
                &arm.pattern.node,
                Pattern::Literal(Literal::Int(_)) | Pattern::Wildcard
            )
        });

        if all_int_literals && !arms.is_empty() && match_val.is_int_value() {
            // Use LLVM switch instruction for integer pattern matching
            let default_block = self.context.append_basic_block(fn_value, &self.fresh_label("match.default"));

            // Collect cases and find default arm
            let mut cases: Vec<(IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
            let mut default_arm: Option<&MatchArm> = None;
            let mut case_arms: Vec<(&MatchArm, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();

            for arm in arms {
                match &arm.pattern.node {
                    Pattern::Literal(Literal::Int(n)) => {
                        let arm_block = self.context.append_basic_block(fn_value, &self.fresh_label("match.arm"));
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
            let switch = self.builder.build_switch(switch_val, default_block, &cases)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let _ = switch; // Suppress unused variable warning

            // Generate arm bodies for integer cases
            for (arm, arm_block) in case_arms {
                self.builder.position_at_end(arm_block);

                // Handle guard if present
                if let Some(guard) = &arm.guard {
                    let guard_pass = self.context.append_basic_block(fn_value, &self.fresh_label("match.guard.pass"));
                    let guard_fail = self.context.append_basic_block(fn_value, &self.fresh_label("match.guard.fail"));

                    let guard_val = self.generate_expr(&guard.node)?;
                    let guard_bool = if guard_val.is_int_value() {
                        guard_val.into_int_value()
                    } else {
                        self.context.bool_type().const_int(1, false) // Truthy fallback
                    };

                    self.builder.build_conditional_branch(guard_bool, guard_pass, guard_fail)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Guard passed - execute body
                    self.builder.position_at_end(guard_pass);
                    let body_val = self.generate_expr(&arm.body.node)?;
                    self.builder.build_unconditional_branch(merge_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    arm_results.push((body_val, self.builder.get_insert_block().unwrap()));

                    // Guard failed - go to default
                    self.builder.position_at_end(guard_fail);
                    self.builder.build_unconditional_branch(default_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    let body_val = self.generate_expr(&arm.body.node)?;
                    self.builder.build_unconditional_branch(merge_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    arm_results.push((body_val, arm_block));
                }
            }

            // Generate default arm
            self.builder.position_at_end(default_block);
            if let Some(arm) = default_arm {
                let body_val = self.generate_expr(&arm.body.node)?;
                self.builder.build_unconditional_branch(merge_block)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                arm_results.push((body_val, default_block));
            } else {
                // No default arm - return default value (0)
                let default_val = self.context.i64_type().const_int(0, false);
                self.builder.build_unconditional_branch(merge_block)
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
                    self.context.append_basic_block(fn_value, &self.fresh_label("match.check"))
                };
                let arm_body_block = self.context.append_basic_block(fn_value, &self.fresh_label("match.arm"));

                self.builder.position_at_end(current_block);

                // Generate pattern check
                let check_result = self.generate_pattern_check(&arm.pattern, &match_val)?;

                // Handle guard
                if let Some(guard) = &arm.guard {
                    let guard_bind_block = self.context.append_basic_block(fn_value, &self.fresh_label("match.guard.bind"));
                    let guard_check_block = self.context.append_basic_block(fn_value, &self.fresh_label("match.guard.check"));

                    // First check pattern
                    self.builder.build_conditional_branch(check_result, guard_bind_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Bind pattern variables for guard to use
                    self.builder.position_at_end(guard_bind_block);
                    self.generate_pattern_bindings(&arm.pattern, &match_val)?;
                    self.builder.build_unconditional_branch(guard_check_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Then check guard
                    self.builder.position_at_end(guard_check_block);
                    let guard_val = self.generate_expr(&guard.node)?;
                    let guard_bool = if guard_val.is_int_value() {
                        let int_val = guard_val.into_int_value();
                        // Convert i64 to i1 if needed
                        if int_val.get_type().get_bit_width() > 1 {
                            self.builder.build_int_compare(
                                IntPredicate::NE,
                                int_val,
                                self.context.i64_type().const_int(0, false),
                                "guard_bool"
                            ).map_err(|e| CodegenError::LlvmError(e.to_string()))?
                        } else {
                            int_val
                        }
                    } else {
                        self.context.bool_type().const_int(1, false)
                    };
                    self.builder.build_conditional_branch(guard_bool, arm_body_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Generate arm body (bindings already done)
                    self.builder.position_at_end(arm_body_block);
                } else {
                    // No guard - branch based on pattern check
                    self.builder.build_conditional_branch(check_result, arm_body_block, next_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Generate arm body
                    self.builder.position_at_end(arm_body_block);

                    // Bind pattern variables if needed
                    self.generate_pattern_bindings(&arm.pattern, &match_val)?;
                }

                let body_val = self.generate_expr(&arm.body.node)?;
                let body_end_block = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_block)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                arm_results.push((body_val, body_end_block));

                current_block = next_block;
            }

            // Handle the case when no arm matched (last block leads to merge)
            // This should be unreachable for exhaustive matches
        }

        // Merge block with phi node
        self.builder.position_at_end(merge_block);

        if arm_results.is_empty() {
            // No arms - return default value
            Ok(self.context.i64_type().const_int(0, false).into())
        } else {
            // Build phi node
            let first_type = arm_results[0].0.get_type();
            let phi = self.builder.build_phi(first_type, "match_result")
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
            Pattern::Ident(_) => {
                // Identifier pattern always matches (binding)
                Ok(self.context.bool_type().const_int(1, false))
            }
            Pattern::Literal(lit) => {
                match lit {
                    Literal::Int(n) => {
                        let lit_val = self.context.i64_type().const_int(*n as u64, true);
                        let cmp = self.builder.build_int_compare(
                            IntPredicate::EQ,
                            match_val.into_int_value(),
                            lit_val,
                            "pat_eq"
                        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::Bool(b) => {
                        let lit_val = self.context.bool_type().const_int(*b as u64, false);
                        let match_int = match_val.into_int_value();
                        // Convert to same bit width if needed
                        let cmp = self.builder.build_int_compare(
                            IntPredicate::EQ,
                            match_int,
                            lit_val,
                            "pat_eq"
                        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::Float(f) => {
                        let lit_val = self.context.f64_type().const_float(*f);
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::OEQ,
                            match_val.into_float_value(),
                            lit_val,
                            "pat_eq"
                        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        Ok(cmp)
                    }
                    Literal::String(s) => {
                        // String comparison using strcmp
                        // First, create the pattern string constant
                        let pattern_str = self.generate_string_literal(s)?;

                        // Get strcmp function
                        let strcmp_fn = self.module.get_function("strcmp")
                            .unwrap_or_else(|| {
                                let i32_type = self.context.i32_type();
                                let ptr_type = self.context.ptr_type(AddressSpace::default());
                                let fn_type = i32_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
                                self.module.add_function("strcmp", fn_type, None)
                            });

                        // Call strcmp
                        let cmp_result = self.builder.build_call(
                            strcmp_fn,
                            &[(*match_val).into(), pattern_str.into()],
                            "strcmp_result"
                        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        let cmp_int = cmp_result.try_as_basic_value().left()
                            .ok_or_else(|| CodegenError::LlvmError("strcmp returned void".to_string()))?
                            .into_int_value();

                        // Check if strcmp returned 0 (equal)
                        let result = self.builder.build_int_compare(
                            IntPredicate::EQ,
                            cmp_int,
                            self.context.i32_type().const_int(0, false),
                            "str_eq"
                        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                        Ok(result)
                    }
                }
            }
            Pattern::Range { start, end, inclusive } => {
                let mut lower_check = self.context.bool_type().const_int(1, false);
                let mut upper_check = self.context.bool_type().const_int(1, false);

                // Check lower bound
                if let Some(start_pat) = start {
                    if let Pattern::Literal(Literal::Int(n)) = &start_pat.node {
                        let start_val = self.context.i64_type().const_int(*n as u64, true);
                        lower_check = self.builder.build_int_compare(
                            IntPredicate::SGE,
                            match_val.into_int_value(),
                            start_val,
                            "range_lower"
                        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }

                // Check upper bound
                if let Some(end_pat) = end {
                    if let Pattern::Literal(Literal::Int(n)) = &end_pat.node {
                        let end_val = self.context.i64_type().const_int(*n as u64, true);
                        let cmp = if *inclusive { IntPredicate::SLE } else { IntPredicate::SLT };
                        upper_check = self.builder.build_int_compare(
                            cmp,
                            match_val.into_int_value(),
                            end_val,
                            "range_upper"
                        ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }

                // Combine checks
                let result = self.builder.build_and(lower_check, upper_check, "range_check")
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
                    result = self.builder.build_or(result, check, "or_pat")
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
                    let elem_val = self.builder.build_extract_value(struct_val, i as u32, &format!("tuple_{}", i))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    let check = self.generate_pattern_check(pat, &elem_val)?;
                    result = self.builder.build_and(result, check, "tuple_check")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }

                Ok(result)
            }
            Pattern::Variant { name, fields: _ } => {
                // Enum variant pattern: check the tag matches
                // Enum is represented as { i8 tag, i64 data }
                let struct_val = match_val.into_struct_value();
                let tag_val = self.builder.build_extract_value(struct_val, 0, "enum_tag")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    .into_int_value();

                // Find expected tag value
                let expected_tag = self.get_enum_variant_tag(&name.node);

                let result = self.builder.build_int_compare(
                    IntPredicate::EQ,
                    tag_val,
                    self.context.i8_type().const_int(expected_tag as u64, false),
                    "variant_check"
                ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;

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
                        let field_val = self.builder.build_extract_value(struct_val, field_idx, &field_name.node)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        let check = self.generate_pattern_check(pat, &field_val)?;
                        result = self.builder.build_and(result, check, "struct_check")
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
                let alloca = self.builder.build_alloca(var_type, name)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder.build_store(alloca, *match_val)
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
                    let elem_val = self.builder.build_extract_value(struct_val, i as u32, &format!("tuple_{}", i))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.generate_pattern_bindings(pat, &elem_val)?;
                }
                Ok(())
            }
            Pattern::Variant { name: _, fields } => {
                // Bind variant fields
                // Enum is { i8 tag, i64 data } - extract data and bind
                let struct_val = match_val.into_struct_value();
                let data_val = self.builder.build_extract_value(struct_val, 1, "variant_data")
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
                    let field_val = self.builder.build_extract_value(struct_val, field_idx, &field_name.node)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    if let Some(pat) = field_pat {
                        // Pattern specified - bind according to pattern
                        self.generate_pattern_bindings(pat, &field_val)?;
                    } else {
                        // Shorthand: `{x}` means `{x: x}`
                        let var_type = field_val.get_type();
                        let alloca = self.builder.build_alloca(var_type, &field_name.node)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.builder.build_store(alloca, field_val)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        self.locals.insert(field_name.node.clone(), (alloca, var_type));
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

        let result = gen.generate_string_literal("hello").unwrap();
        assert!(result.is_pointer_value());
    }

    #[test]
    fn test_ast_type_to_resolved() {
        let context = Context::create();
        let gen = InkwellCodeGenerator::new(&context, "test");

        // Test basic types
        let i64_type = Type::Named { name: "i64".to_string(), generics: vec![] };
        let resolved = gen.ast_type_to_resolved(&i64_type);
        assert!(matches!(resolved, ResolvedType::I64));

        let bool_type = Type::Named { name: "bool".to_string(), generics: vec![] };
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
        assert!(matches!(gen.get_generic_substitution("T"), Some(ResolvedType::I64)));
        assert!(matches!(gen.get_generic_substitution("U"), Some(ResolvedType::Bool)));
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
