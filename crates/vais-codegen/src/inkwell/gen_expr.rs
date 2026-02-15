//! Expression code generation.
//!
//! Handles generation of all expression types: literals, variables,
//! binary/unary operations, function calls, etc.

use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, IntValue};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};

use vais_ast::{BinOp, Expr, Literal, Spanned, UnaryOp};

use super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_expr(&mut self, expr: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
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
                capture_mode,
            } => self.generate_lambda(params, &body.node, captures, *capture_mode),

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
                let ptr_val = if ptr.is_pointer_value() {
                    ptr.into_pointer_value()
                } else {
                    // IntValue (i64) → PointerValue via inttoptr
                    let int_val = ptr.into_int_value();
                    self.builder
                        .build_int_to_ptr(
                            int_val,
                            self.context
                                .i64_type()
                                .ptr_type(inkwell::AddressSpace::default()),
                            "deref_ptr",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                };
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

            // Lazy: create deferred evaluation thunk
            Expr::Lazy(inner) => self.generate_lazy(&inner.node),

            // Await: evaluate the inner expression (async functions compile as synchronous
            // in Inkwell backend, so await is effectively identity — the function has
            // already completed and returned its result directly)
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

            // Force: evaluate a lazy value (check computed flag, call thunk if needed)
            Expr::Force(inner) => self.generate_force(&inner.node),

            // Spawn: evaluate inner expression to create a concurrent task.
            // In Inkwell backend, async functions compile as synchronous, so spawn
            // evaluates the inner expression immediately (eager evaluation).
            Expr::Spawn(inner) => self.generate_expr(&inner.node),

            // Yield: evaluate the inner expression and return its value.
            // Yields the value to the generator's caller. In the current synchronous
            // model, this evaluates and returns the value directly.
            Expr::Yield(inner) => self.generate_expr(&inner.node),

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

    /// Generate LLVM value for a literal (helper for expression generation).
    #[allow(dead_code)]
    pub(super) fn generate_literal(
        &mut self,
        lit: &Literal,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        match lit {
            Literal::Int(n) => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
            Literal::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            Literal::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Literal::String(s) => self.generate_string_literal(s),
        }
    }

    pub(super) fn generate_string_literal(
        &mut self,
        s: &str,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
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

    pub(super) fn generate_var(&mut self, name: &str) -> CodegenResult<BasicValueEnum<'ctx>> {
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

    pub(super) fn generate_binary(
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

    pub(super) fn generate_int_binary(
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

    pub(super) fn generate_float_binary(
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

    pub(super) fn generate_unary(
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

    pub(super) fn generate_call(
        &mut self,
        callee: &Expr,
        args: &[Spanned<Expr>],
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Struct tuple literal: `Response(200, 1)` → desugar to StructLit
        if let Expr::Ident(name) = callee {
            if self.generated_structs.contains_key(name.as_str())
                && self.module.get_function(name).is_none()
            {
                if let Some(field_names) = self.struct_fields.get(name.as_str()).cloned() {
                    let fields: Vec<_> = field_names
                        .iter()
                        .zip(args.iter())
                        .map(|(fname, val)| {
                            (vais_ast::Spanned::new(fname.clone(), val.span), val.clone())
                        })
                        .collect();
                    return self.generate_struct_literal(name, &fields);
                }
            }
        }

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
            "swap" => return self.generate_swap(args),
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
            "ptr_to_str" => {
                // ptr_to_str(i64) -> ptr: convert i64 to i8*
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "ptr_to_str requires 1 arg".to_string(),
                    ));
                }
                let arg = self.generate_expr(&args[0].node)?;
                if arg.is_int_value() {
                    let i8_ptr_type = self
                        .context
                        .i8_type()
                        .ptr_type(inkwell::AddressSpace::default());
                    let result = self
                        .builder
                        .build_int_to_ptr(arg.into_int_value(), i8_ptr_type, "ptr_to_str_result")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    return Ok(result.into());
                } else {
                    // Already a pointer
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

    /// Generate lazy expression: creates a thunk function and returns { i1, T, ptr } struct
    pub(super) fn generate_lazy(
        &mut self,
        inner: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // For Inkwell, we generate the thunk function and build the lazy struct.
        // The thunk captures free variables from the current scope.

        let thunk_name = format!("__lazy_thunk_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Find captured variables
        let used_idents = Self::collect_idents(inner);
        let captured_vars: Vec<(String, BasicValueEnum<'ctx>, inkwell::types::BasicTypeEnum<'ctx>)> =
            used_idents
                .iter()
                .filter_map(|name| {
                    self.locals.get(name).map(|(ptr, var_type)| {
                        let val = self
                            .builder
                            .build_load(*var_type, *ptr, &format!("lazy_cap_{}", name))
                            .unwrap_or_else(|_| self.context.i64_type().const_int(0, false).into());
                        (name.clone(), val, *var_type)
                    })
                })
                .collect();

        // Build thunk parameter types (captured vars only)
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = captured_vars
            .iter()
            .map(|(_, _, cap_type)| (*cap_type).into())
            .collect();

        // Infer the inner type (approximate: use i64 as fallback)
        let inner_type = self.context.i64_type();
        let fn_type = inner_type.fn_type(&param_types, false);
        let thunk_fn = self.module.add_function(&thunk_name, fn_type, None);

        // Save state
        let saved_function = self.current_function;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_insert_block = self.builder.get_insert_block();

        // Set up thunk context
        self.current_function = Some(thunk_fn);
        let entry = self.context.append_basic_block(thunk_fn, "entry");
        self.builder.position_at_end(entry);

        // Register captured vars as params in thunk
        for (idx, (cap_name, _, cap_type)) in captured_vars.iter().enumerate() {
            let param_val = thunk_fn.get_nth_param(idx as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(*cap_type, &format!("__cap_{}", cap_name))
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals.insert(cap_name.clone(), (alloca, *cap_type));
        }

        // Generate thunk body
        let body_val = self.generate_expr(inner)?;
        self.builder
            .build_return(Some(&body_val))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Restore context
        self.current_function = saved_function;
        self.locals = saved_locals;
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }
        self.functions.insert(thunk_name.clone(), thunk_fn);

        // Build lazy struct: { i1 false, T zeroinit, ptr thunk_fn }
        let bool_type = self.context.bool_type();
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let lazy_struct_type = self.context.struct_type(
            &[bool_type.into(), inner_type.into(), ptr_type.into()],
            false,
        );

        let lazy_alloca = self
            .builder
            .build_alloca(lazy_struct_type, "lazy_struct")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store computed = false
        let computed_ptr = self
            .builder
            .build_struct_gep(lazy_struct_type, lazy_alloca, 0, "computed_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(computed_ptr, bool_type.const_int(0, false))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store zero value
        let value_ptr = self
            .builder
            .build_struct_gep(lazy_struct_type, lazy_alloca, 1, "value_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(value_ptr, inner_type.const_int(0, false))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store thunk function pointer (cast to i8*)
        let thunk_ptr_val = thunk_fn.as_global_value().as_pointer_value();
        let thunk_as_i8_ptr = self
            .builder
            .build_bitcast(thunk_ptr_val, ptr_type, "thunk_i8ptr")
            .map_err(|e: inkwell::builder::BuilderError| CodegenError::LlvmError(e.to_string()))?;
        let thunk_slot = self
            .builder
            .build_struct_gep(lazy_struct_type, lazy_alloca, 2, "thunk_ptr")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder
            .build_store(thunk_slot, thunk_as_i8_ptr)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        // Store captured values for force to use later
        let cap_for_binding: Vec<(String, BasicValueEnum<'ctx>)> = captured_vars
            .iter()
            .map(|(name, val, _)| (name.clone(), *val))
            .collect();
        self._last_lazy_info = Some((thunk_name, cap_for_binding));

        // Load and return the lazy struct
        let result = self
            .builder
            .build_load(lazy_struct_type, lazy_alloca, "lazy_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(result)
    }

    /// Generate force expression: check computed flag, call thunk if needed, cache result
    pub(super) fn generate_force(
        &mut self,
        inner: &Expr,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        // Look up lazy binding info if the inner is an identifier
        let lazy_info = if let Expr::Ident(name) = inner {
            self.lazy_bindings.get(name).cloned()
        } else {
            None
        };

        let val = self.generate_expr(inner)?;

        // Check if the value is a lazy struct (has struct type with 3 fields)
        if !val.is_struct_value() {
            return Ok(val);
        }

        let struct_val = val.into_struct_value();
        let struct_type = struct_val.get_type();

        // Verify this is a lazy struct (3 fields: i1, T, ptr)
        if struct_type.count_fields() != 3 {
            let result = self
                .builder
                .build_extract_value(struct_val, 1, "force_val")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            return Ok(result);
        }

        // If we have thunk info, generate proper conditional evaluation
        if let Some((thunk_name, captured_vals)) = lazy_info {
            let current_fn = self
                .current_function
                .ok_or_else(|| CodegenError::LlvmError("No current function".to_string()))?;

            let bool_type = self.context.bool_type();
            let inner_type = self.context.i64_type();

            // Spill lazy struct to alloca so we can update computed/value fields
            let lazy_alloca = self
                .builder
                .build_alloca(struct_type, "lazy_spill")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(lazy_alloca, struct_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Load computed flag (field 0)
            let computed_ptr = self
                .builder
                .build_struct_gep(struct_type, lazy_alloca, 0, "computed_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let computed_flag = self
                .builder
                .build_load(bool_type, computed_ptr, "computed")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .into_int_value();

            // Create basic blocks for conditional evaluation
            let cached_bb = self.context.append_basic_block(current_fn, "lazy.cached");
            let compute_bb = self.context.append_basic_block(current_fn, "lazy.compute");
            let merge_bb = self.context.append_basic_block(current_fn, "lazy.merge");

            self.builder
                .build_conditional_branch(computed_flag, cached_bb, compute_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Cached path: load value from struct field 1
            self.builder.position_at_end(cached_bb);
            let value_ptr = self
                .builder
                .build_struct_gep(struct_type, lazy_alloca, 1, "cached_val_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            let cached_val = self
                .builder
                .build_load(inner_type, value_ptr, "cached_val")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Compute path: call thunk, store result, set computed=true
            self.builder.position_at_end(compute_bb);

            // Build thunk call arguments (captured values)
            let thunk_fn = self.functions.get(&thunk_name).copied().ok_or_else(|| {
                CodegenError::LlvmError(format!("Thunk function '{}' not found", thunk_name))
            })?;

            let call_args: Vec<inkwell::values::BasicMetadataValueEnum> = captured_vals
                .iter()
                .map(|(_, val)| (*val).into())
                .collect();

            let computed_val = self
                .builder
                .build_call(thunk_fn, &call_args, "thunk_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                .try_as_basic_value()
                .left()
                .unwrap_or_else(|| inner_type.const_int(0, false).into());

            // Store computed value into lazy struct
            let store_val_ptr = self
                .builder
                .build_struct_gep(struct_type, lazy_alloca, 1, "store_val_ptr")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(store_val_ptr, computed_val)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Set computed = true
            self.builder
                .build_store(computed_ptr, bool_type.const_int(1, false))
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

            // Merge: phi node selects cached or computed value
            self.builder.position_at_end(merge_bb);
            let phi = self
                .builder
                .build_phi(inner_type, "force_result")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            phi.add_incoming(&[
                (&cached_val, cached_bb),
                (&computed_val, compute_bb),
            ]);

            return Ok(phi.as_basic_value());
        }

        // Fallback: no thunk info, just extract the value field (index 1)
        let result = self
            .builder
            .build_extract_value(struct_val, 1, "force_val")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        Ok(result)
    }
}
