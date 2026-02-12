//! Statement Visitor implementation for CodeGenerator
//!
//! This module implements the StmtVisitor trait for CodeGenerator,
//! providing a clean separation of statement code generation logic.

use crate::types::LocalVar;
use crate::visitor::{BlockResult, GenResult, StmtVisitor};
use crate::{CodeGenerator, CodegenError};
use vais_ast::{Expr, Spanned, Stmt};
use vais_types::ResolvedType;

impl StmtVisitor for CodeGenerator {
    fn visit_stmt(&mut self, stmt: &Spanned<Stmt>, counter: &mut usize) -> GenResult {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ..
            } => self.generate_let_stmt(name, ty.as_ref(), value, *is_mut, counter),
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
                ..
            } => self.generate_let_destructure(pattern, value, *is_mut, counter),
            Stmt::Expr(expr) => self.generate_expr(expr, counter),
            Stmt::Return(expr) => self.generate_return_stmt(expr.as_ref().map(|e| &**e), counter),
            Stmt::Break(value) => self.generate_break_stmt(value.as_ref().map(|v| &**v), counter),
            Stmt::Continue => self.generate_continue_stmt(),
            Stmt::Defer(expr) => {
                // Add the deferred expression to the stack
                // It will be executed when the function exits (in LIFO order)
                self.defer_stack.push(expr.as_ref().clone());
                // No IR generated here - defer is processed at function exit
                Ok(("void".to_string(), String::new()))
            }
            Stmt::Error { message, .. } => {
                // Error nodes should not reach codegen - they indicate parsing failures
                Err(CodegenError::Unsupported(format!(
                    "Parse error in statement: {}",
                    message
                )))
            }
        }
    }

    fn visit_block_stmts(&mut self, stmts: &[Spanned<Stmt>], counter: &mut usize) -> BlockResult {
        let mut ir = String::new();
        let mut last_value = "0".to_string();
        let mut terminated = false;

        for stmt in stmts {
            if terminated {
                break;
            }

            let (value, stmt_ir) = self.visit_stmt(stmt, counter)?;
            ir.push_str(&stmt_ir);
            last_value = value;

            match &stmt.node {
                Stmt::Break(_) | Stmt::Continue | Stmt::Return(_) => {
                    terminated = true;
                }
                _ => {}
            }
        }

        Ok((last_value, ir, terminated))
    }
}

impl CodeGenerator {
    /// Generate let statement with SSA optimization for immutable simple types
    fn generate_let_stmt(
        &mut self,
        name: &Spanned<String>,
        ty: Option<&Spanned<vais_ast::Type>>,
        value: &Spanned<Expr>,
        is_mut: bool,
        counter: &mut usize,
    ) -> GenResult {
        // Infer type BEFORE generating code, so we can use function return types
        let inferred_ty = self.infer_expr_type(value);

        // Check if this is a struct literal - handle specially
        // Also detect struct tuple literal: Point(40, 2) where "Point" is a known struct
        let is_struct_lit = matches!(&value.node, Expr::StructLit { .. })
            || if let Expr::Call { func, .. } = &value.node {
                if let Expr::Ident(fn_name) = &func.node {
                    let resolved = self.resolve_struct_name(fn_name);
                    self.structs.contains_key(&resolved) && !self.functions.contains_key(fn_name)
                } else {
                    false
                }
            } else {
                false
            };

        // Check if this is an enum variant constructor call (e.g., Some(42))
        let is_enum_variant_call = if let Expr::Call { func, .. } = &value.node {
            if let Expr::Ident(fn_name) = &func.node {
                self.get_tuple_variant_info(fn_name).is_some()
            } else {
                false
            }
        } else {
            false
        };

        // Check if this is a unit enum variant (e.g., None)
        let is_unit_variant = if let Expr::Ident(ident_name) = &value.node {
            self.is_unit_enum_variant(ident_name)
        } else {
            false
        };

        let (val, val_ir) = self.generate_expr(value, counter)?;

        let resolved_ty = ty
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(inferred_ty);

        // Determine if we can use SSA style (no alloca) to reduce stack usage
        // Conditions for SSA:
        // 1. Not mutable (is_mut == false)
        // 2. Not a struct literal (needs pointer semantics)
        // 3. Not an enum variant (needs pointer semantics)
        // 4. Not a Named type (struct/enum values need special handling)
        // 5. Simple primitive types (i64, i32, bool, etc.)
        let is_simple_type = matches!(
            resolved_ty,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::I128
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
                | ResolvedType::U128
                | ResolvedType::F32
                | ResolvedType::F64
                | ResolvedType::Bool
                | ResolvedType::Str
                | ResolvedType::Pointer(_)
        );

        let use_ssa = !is_mut
            && !is_struct_lit
            && !is_enum_variant_call
            && !is_unit_variant
            && !matches!(resolved_ty, ResolvedType::Named { .. })
            && is_simple_type;

        if use_ssa {
            // SSA style: directly alias the value, no alloca needed
            // This significantly reduces stack usage
            self.locals.insert(
                name.node.clone(),
                LocalVar::ssa(resolved_ty.clone(), val.clone()),
            );
            // If this was a lambda with captures, register the closure info
            if let Some(closure_info) = self.last_lambda_info.take() {
                self.closures.insert(name.node.clone(), closure_info);
            }
            // Return just the expression IR, no alloca/store needed
            Ok(("void".to_string(), val_ir))
        } else {
            // Traditional alloca style
            // Generate unique LLVM name for this variable (to handle loops)
            let llvm_name = format!("{}.{}", name.node, counter);
            *counter += 1;

            self.locals.insert(
                name.node.clone(),
                LocalVar::alloca(resolved_ty.clone(), llvm_name.clone()),
            );

            let mut ir = val_ir;
            let llvm_ty = self.type_to_llvm(&resolved_ty);

            // For struct literals and enum variant constructors, the value is already an alloca'd pointer
            if is_struct_lit || is_enum_variant_call || is_unit_variant {
                ir.push_str(&format!("  %{} = alloca {}*\n", llvm_name, llvm_ty));
                ir.push_str(&format!(
                    "  store {}* {}, {}** %{}\n",
                    llvm_ty, val, llvm_ty, llvm_name
                ));
            } else if matches!(resolved_ty, ResolvedType::Named { .. }) {
                // For struct values (e.g., from function returns)
                let tmp_ptr = format!("%{}.struct", llvm_name);
                ir.push_str(&format!("  {} = alloca {}\n", tmp_ptr, llvm_ty));
                ir.push_str(&format!(
                    "  store {} {}, {}* {}\n",
                    llvm_ty, val, llvm_ty, tmp_ptr
                ));
                ir.push_str(&format!("  %{} = alloca {}*\n", llvm_name, llvm_ty));
                ir.push_str(&format!(
                    "  store {}* {}, {}** %{}\n",
                    llvm_ty, tmp_ptr, llvm_ty, llvm_name
                ));
            } else {
                ir.push_str(&format!("  %{} = alloca {}\n", llvm_name, llvm_ty));
                ir.push_str(&format!(
                    "  store {} {}, {}* %{}\n",
                    llvm_ty, val, llvm_ty, llvm_name
                ));
            }

            // If this was a lambda with captures, register the closure info
            if let Some(closure_info) = self.last_lambda_info.take() {
                self.closures.insert(name.node.clone(), closure_info);
            }

            Ok(("void".to_string(), ir))
        }
    }

    /// Generate break statement
    fn generate_break_stmt(
        &mut self,
        value: Option<&Spanned<Expr>>,
        counter: &mut usize,
    ) -> GenResult {
        if let Some(labels) = self.loop_stack.last() {
            let break_label = labels.break_label.clone();
            let mut ir = String::new();
            if let Some(expr) = value {
                let (val, expr_ir) = self.generate_expr(expr, counter)?;
                ir.push_str(&expr_ir);
                ir.push_str(&format!("  br label %{}\n", break_label));
                Ok((val, ir))
            } else {
                ir.push_str(&format!("  br label %{}\n", break_label));
                Ok(("void".to_string(), ir))
            }
        } else {
            Err(CodegenError::Unsupported(
                "break outside of loop".to_string(),
            ))
        }
    }

    /// Generate continue statement
    fn generate_continue_stmt(&mut self) -> GenResult {
        if let Some(labels) = self.loop_stack.last() {
            let continue_label = labels.continue_label.clone();
            let ir = format!("  br label %{}\n", continue_label);
            Ok(("void".to_string(), ir))
        } else {
            Err(CodegenError::Unsupported(
                "continue outside of loop".to_string(),
            ))
        }
    }

    /// Generate return statement with actual ret instruction
    fn generate_return_stmt(
        &mut self,
        expr: Option<&Spanned<Expr>>,
        counter: &mut usize,
    ) -> GenResult {
        let mut ir = String::new();

        // Execute deferred expressions before return (LIFO order)
        let defer_ir = self.generate_defer_cleanup(counter)?;
        ir.push_str(&defer_ir);

        if let Some(expr) = expr {
            let (val, expr_ir) = self.generate_expr(expr, counter)?;
            ir.push_str(&expr_ir);

            // Get return type from current function context
            let ret_type = self
                .current_return_type
                .clone()
                .unwrap_or(ResolvedType::I64);
            let llvm_ty = self.type_to_llvm(&ret_type);

            // For struct types, may need to load from pointer
            if matches!(ret_type, ResolvedType::Named { .. }) && !self.is_expr_value(expr) {
                let loaded = format!("%ret.{}", counter);
                *counter += 1;
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    loaded, llvm_ty, llvm_ty, val
                ));
                ir.push_str(&format!("  ret {} {}\n", llvm_ty, loaded));
            } else {
                ir.push_str(&format!("  ret {} {}\n", llvm_ty, val));
            }
        } else {
            ir.push_str("  ret void\n");
        }

        Ok(("void".to_string(), ir))
    }
}
