//! Statement code generation for Vais compiler
//!
//! This module handles generation of LLVM IR for Vais statements (Let, Return, Break, Continue, etc.)

use crate::{CodeGenerator, CodegenError, CodegenResult};
use crate::types::LocalVar;
use vais_ast::{Spanned, Stmt, Expr};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate LLVM IR for a block of statements
    pub(crate) fn generate_block(
        &mut self,
        stmts: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let mut last_value = "void".to_string();

        for stmt in stmts {
            let (value, stmt_ir) = self.generate_stmt(stmt, counter)?;
            ir.push_str(&stmt_ir);
            last_value = value;
        }

        Ok((last_value, ir))
    }

    /// Generate LLVM IR for a single statement
    pub(crate) fn generate_stmt(
        &mut self,
        stmt: &Spanned<Stmt>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut: _,
            } => {
                // Infer type BEFORE generating code, so we can use function return types
                let inferred_ty = self.infer_expr_type(value);

                // Check if this is a struct literal - handle specially
                let is_struct_lit = matches!(&value.node, Expr::StructLit { .. });

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
                let is_unit_variant = if let Expr::Ident(name) = &value.node {
                    self.is_unit_enum_variant(name)
                } else {
                    false
                };

                let (val, val_ir) = self.generate_expr(value, counter)?;

                let resolved_ty = ty
                    .as_ref()
                    .map(|t| self.ast_type_to_resolved(&t.node))
                    .unwrap_or(inferred_ty); // Use inferred type if not specified

                // Generate unique LLVM name for this variable (to handle loops)
                let llvm_name = format!("{}.{}", name.node, counter);
                *counter += 1;

                self.locals.insert(
                    name.node.clone(),
                    LocalVar {
                        ty: resolved_ty.clone(),
                        is_param: false, // alloca'd variable
                        llvm_name: llvm_name.clone(),
                    },
                );

                let mut ir = val_ir;
                let llvm_ty = self.type_to_llvm(&resolved_ty);

                // For struct literals and enum variant constructors, the value is already an alloca'd pointer
                // We store the pointer to the struct/enum (i.e., %Point*, %Option*)
                if is_struct_lit || is_enum_variant_call || is_unit_variant {
                    // The val is already a pointer to the struct/enum (%1, %2, etc)
                    // Allocate space for a pointer and store it
                    ir.push_str(&format!(
                        "  %{} = alloca {}*\n",
                        llvm_name, llvm_ty
                    ));
                    ir.push_str(&format!(
                        "  store {}* {}, {}** %{}\n",
                        llvm_ty, val, llvm_ty, llvm_name
                    ));
                } else if matches!(resolved_ty, ResolvedType::Named { .. }) {
                    // For struct values (e.g., from function returns),
                    // alloca struct, store value, then store pointer to it
                    // This keeps all struct variables as pointers for consistency
                    let tmp_ptr = format!("%{}.struct", llvm_name);
                    ir.push_str(&format!(
                        "  {} = alloca {}\n",
                        tmp_ptr, llvm_ty
                    ));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        llvm_ty, val, llvm_ty, tmp_ptr
                    ));
                    ir.push_str(&format!(
                        "  %{} = alloca {}*\n",
                        llvm_name, llvm_ty
                    ));
                    ir.push_str(&format!(
                        "  store {}* {}, {}** %{}\n",
                        llvm_ty, tmp_ptr, llvm_ty, llvm_name
                    ));
                } else {
                    // Allocate and store
                    ir.push_str(&format!(
                        "  %{} = alloca {}\n",
                        llvm_name, llvm_ty
                    ));
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
            Stmt::Expr(expr) => self.generate_expr(expr, counter),
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    let (val, ir) = self.generate_expr(expr, counter)?;
                    Ok((val, ir))
                } else {
                    Ok(("void".to_string(), String::new()))
                }
            }
            Stmt::Break(value) => {
                if let Some(labels) = self.loop_stack.last() {
                    let break_label = labels.break_label.clone();
                    let mut ir = String::new();
                    if let Some(expr) = value {
                        let (val, expr_ir) = self.generate_expr(expr, counter)?;
                        ir.push_str(&expr_ir);
                        // Store break value if needed (for loop expressions)
                        ir.push_str(&format!("  br label %{}\n", break_label));
                        Ok((val, ir))
                    } else {
                        ir.push_str(&format!("  br label %{}\n", break_label));
                        Ok(("void".to_string(), ir))
                    }
                } else {
                    Err(CodegenError::Unsupported("break outside of loop".to_string()))
                }
            }
            Stmt::Continue => {
                if let Some(labels) = self.loop_stack.last() {
                    let continue_label = labels.continue_label.clone();
                    let ir = format!("  br label %{}\n", continue_label);
                    Ok(("void".to_string(), ir))
                } else {
                    Err(CodegenError::Unsupported("continue outside of loop".to_string()))
                }
            }
        }
    }
}
