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

use vais_ast::{self as ast, BinaryOp, Expr, ExprKind, Literal, MatchArm, Module as VaisModule, Pattern, Spanned, Stmt, UnaryOp};
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

        self.type_mapper.register_struct(&e.name, enum_type);
        self.generated_structs.insert(e.name.clone(), enum_type);

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
        match &expr.kind {
            ExprKind::Literal(lit) => self.generate_literal(lit),
            ExprKind::Var(name) => self.generate_var(name),
            ExprKind::Binary(op, lhs, rhs) => self.generate_binary(*op, lhs, rhs),
            ExprKind::Unary(op, operand) => self.generate_unary(*op, operand),
            ExprKind::Call(callee, args) => self.generate_call(callee, args),
            ExprKind::Block(stmts) => self.generate_block(stmts),
            ExprKind::If(if_else) => self.generate_if(if_else),
            ExprKind::StructLiteral(name, fields) => self.generate_struct_literal(name, fields),
            ExprKind::FieldAccess(obj, field) => self.generate_field_access(obj, field),
            ExprKind::Match { expr: match_expr, arms } => self.generate_match(match_expr, arms),
            _ => Err(CodegenError::Unsupported(format!(
                "Expression kind not yet implemented: {:?}",
                expr.kind
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
        args: &[Expr],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Get function name
        let fn_name = match &callee.kind {
            ExprKind::Var(name) => name.clone(),
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
            .map(|arg| self.generate_expr(arg).map(|v| v.into()))
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

    fn generate_block(&mut self, stmts: &[Stmt]) -> CodegenResult<BasicValueEnum<'ctx>> {
        let mut last_value: BasicValueEnum = self.context.struct_type(&[], false).const_zero().into();

        for stmt in stmts {
            last_value = self.generate_stmt(stmt)?;
        }

        Ok(last_value)
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> CodegenResult<BasicValueEnum<'ctx>> {
        match stmt {
            Stmt::Let { name, ty, value, .. } => {
                let val = self.generate_expr(value)?;
                let var_type = self.type_mapper.map_type(ty);
                let alloca = self.builder.build_alloca(var_type, name)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder.build_store(alloca, val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.locals.insert(name.clone(), (alloca, var_type));
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            Stmt::Expr(expr) => self.generate_expr(expr),
            Stmt::Return(expr) => {
                let val = self.generate_expr(expr)?;
                self.builder.build_return(Some(&val))
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                Ok(self.context.struct_type(&[], false).const_zero().into())
            }
            _ => Err(CodegenError::Unsupported(format!("Statement: {:?}", stmt))),
        }
    }

    fn generate_if(&mut self, if_else: &ast::IfElse) -> CodegenResult<BasicValueEnum<'ctx>> {
        let fn_value = self.current_function.ok_or_else(|| {
            CodegenError::LlvmError("No current function for if expression".to_string())
        })?;

        // Generate condition
        let cond = self.generate_expr(&if_else.condition)?;
        let cond_bool = cond.into_int_value();

        // Create blocks
        let then_block = self.context.append_basic_block(fn_value, "then");
        let else_block = self.context.append_basic_block(fn_value, "else");
        let merge_block = self.context.append_basic_block(fn_value, "merge");

        // Conditional branch
        self.builder.build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Then block
        self.builder.position_at_end(then_block);
        let then_val = self.generate_expr(&if_else.then_branch)?;
        self.builder.build_unconditional_branch(merge_block)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let then_end_block = self.builder.get_insert_block().unwrap();

        // Else block
        self.builder.position_at_end(else_block);
        let else_val = if let Some(else_expr) = &if_else.else_branch {
            self.generate_expr(else_expr)?
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

    fn generate_struct_literal(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let struct_type = self.generated_structs.get(name)
            .ok_or_else(|| CodegenError::UndefinedVar(format!("Struct not found: {}", name)))?;

        // Generate field values
        let mut field_values: Vec<BasicValueEnum> = Vec::new();
        for (_, expr) in fields {
            field_values.push(self.generate_expr(expr)?);
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
        match &expr.kind {
            ExprKind::Var(name) => {
                // Look up variable type - would need type info stored with locals
                // For now, return an error if we can't determine the type
                Err(CodegenError::Unsupported(format!(
                    "Cannot infer struct type for variable: {}. Consider adding type annotations.",
                    name
                )))
            }
            ExprKind::StructLiteral(name, _) => Ok(name.clone()),
            _ => Err(CodegenError::Unsupported(format!(
                "Cannot infer struct type for expression: {:?}",
                expr.kind
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

    /// Gets the tag value for an enum variant.
    fn get_enum_variant_tag(&self, _variant_name: &str) -> i32 {
        // TODO: Implement proper enum variant tag lookup
        // For now, return 0 as a placeholder
        0
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
}
