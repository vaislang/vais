//! Special expressions and builtin functions.
//!
//! Handles string interpolation, destructuring, method declarations,
//! impl blocks, and builtin functions (println, store_i64, etc.).


use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;

use vais_ast::{self as ast, Expr, Pattern, Spanned, Stmt, StringInterpPart, Type};
use vais_types::ResolvedType;

use super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn generate_string_interp(
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

    pub(super) fn generate_let_destructure(
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
    pub(super) fn extract_struct_type_name(&self, ty: &Type) -> Option<String> {
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
    pub(super) fn infer_value_struct_type(&self, expr: &Expr) -> Option<String> {
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
                                // Only use LLVM type match if unambiguous (exactly one match)
                                let matches: Vec<_> = self
                                    .generated_structs
                                    .iter()
                                    .filter(|(_, st)| **st == struct_type)
                                    .collect();
                                if matches.len() == 1 {
                                    return Some(matches[0].0.clone());
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
                // First check our explicit function->struct return type map
                if let Some(sn) = self.function_return_structs.get(&qualified) {
                    return Some(sn.clone());
                }
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
                            // Only use LLVM type match if unambiguous (exactly one match)
                            let matches: Vec<_> = self
                                .generated_structs
                                .iter()
                                .filter(|(_, st)| **st == struct_type)
                                .collect();
                            if matches.len() == 1 {
                                return Some(matches[0].0.clone());
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
    pub(super) fn declare_method(
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

        let ret_resolved = if let Some(t) = func.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else if let Some(resolved_sig) = self.resolved_function_sigs.get(&method_name) {
            resolved_sig.ret.clone()
        } else if let Some(resolved_sig) = self.resolved_function_sigs.get(&func.name.node) {
            resolved_sig.ret.clone()
        } else {
            ResolvedType::Unit
        };
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
    pub(super) fn generate_method(&mut self, type_name: &str, func: &ast::Function) -> CodegenResult<()> {
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
        let ret_resolved = if let Some(t) = func.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else if let Some(resolved_sig) = self.resolved_function_sigs.get(&method_name) {
            resolved_sig.ret.clone()
        } else if let Some(resolved_sig) = self.resolved_function_sigs.get(&func.name.node) {
            resolved_sig.ret.clone()
        } else {
            ResolvedType::Unit
        };
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
    pub(super) fn get_impl_type_name(ty: &Type) -> Option<String> {
        match ty {
            Type::Named { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    // ========== Built-in pseudo-functions ==========

    pub(super) fn generate_println_call(
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

    pub(super) fn generate_print_call(
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

    pub(super) fn generate_format_call(
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

    pub(super) fn generate_store_i64(
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

    pub(super) fn generate_load_i64(&mut self, args: &[Spanned<Expr>]) -> CodegenResult<BasicValueEnum<'ctx>> {
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

    pub(super) fn generate_store_byte(
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

    pub(super) fn generate_load_byte(
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

    pub(super) fn generate_store_f64(
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

    pub(super) fn generate_load_f64(&mut self, args: &[Spanned<Expr>]) -> CodegenResult<BasicValueEnum<'ctx>> {
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
