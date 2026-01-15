//! Vais 2.0 LLVM Code Generator
//!
//! Generates LLVM IR from typed AST for native code generation.
//!
//! Note: This is a placeholder structure. Full LLVM integration requires
//! the inkwell crate and LLVM installation.

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::{*, IfElse};
use vais_types::ResolvedType;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Undefined variable: {0}")]
    UndefinedVar(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("LLVM error: {0}")]
    LlvmError(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

type CodegenResult<T> = Result<T, CodegenError>;

/// LLVM IR Code Generator for Vais 2.0
///
/// Generates LLVM IR text from typed AST for native code generation via clang.
pub struct CodeGenerator {
    // Module name
    module_name: String,

    // Function signatures for lookup
    functions: HashMap<String, FunctionInfo>,

    // Struct definitions
    structs: HashMap<String, StructInfo>,

    // Current function being compiled
    current_function: Option<String>,

    // Local variables in current function
    locals: HashMap<String, LocalVar>,

    // Label counter for unique basic block names
    label_counter: usize,

    // Stack of loop labels for break/continue
    loop_stack: Vec<LoopLabels>,
}

#[derive(Debug, Clone)]
struct LoopLabels {
    continue_label: String,
    break_label: String,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    params: Vec<(String, ResolvedType)>,
    ret_type: ResolvedType,
    is_extern: bool,
}

#[derive(Debug, Clone)]
struct StructInfo {
    #[allow(dead_code)]
    name: String,
    fields: Vec<(String, ResolvedType)>,
}

#[derive(Debug, Clone)]
struct LocalVar {
    ty: ResolvedType,
    /// True if this is a function parameter (SSA value), false if alloca'd
    is_param: bool,
}

impl CodeGenerator {
    pub fn new(module_name: &str) -> Self {
        Self {
            module_name: module_name.to_string(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            current_function: None,
            locals: HashMap::new(),
            label_counter: 0,
            loop_stack: Vec::new(),
        }
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let label = format!("{}{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Generate LLVM IR for a module
    pub fn generate_module(&mut self, module: &Module) -> CodegenResult<String> {
        let mut ir = String::new();

        // Header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");

        // Note: target triple and data layout are omitted to let clang auto-detect
        ir.push('\n');

        // First pass: collect declarations
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => self.register_struct(s)?,
                _ => {}
            }
        }

        // Generate struct types
        for (name, info) in &self.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations
        for (_name, info) in &self.functions {
            if info.is_extern {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
            }
        }

        // Second pass: generate function bodies
        for item in &module.items {
            if let Item::Function(f) = &item.node {
                ir.push_str(&self.generate_function(f)?);
                ir.push('\n');
            }
        }

        Ok(ir)
    }

    fn register_function(&mut self, f: &Function) -> CodegenResult<()> {
        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                (p.name.node.clone(), ty)
            })
            .collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        self.functions.insert(
            f.name.node.clone(),
            FunctionInfo {
                name: f.name.node.clone(),
                params,
                ret_type,
                is_extern: false,
            },
        );

        Ok(())
    }

    fn register_struct(&mut self, s: &Struct) -> CodegenResult<()> {
        let fields: Vec<_> = s
            .fields
            .iter()
            .map(|f| {
                let ty = self.ast_type_to_resolved(&f.ty.node);
                (f.name.node.clone(), ty)
            })
            .collect();

        self.structs.insert(
            s.name.node.clone(),
            StructInfo {
                name: s.name.node.clone(),
                fields,
            },
        );

        Ok(())
    }

    fn generate_struct_type(&self, name: &str, info: &StructInfo) -> String {
        let fields: Vec<_> = info
            .fields
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty))
            .collect();

        format!("%{} = type {{ {} }}", name, fields.join(", "))
    }

    fn generate_extern_decl(&self, info: &FunctionInfo) -> String {
        let params: Vec<_> = info
            .params
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty))
            .collect();

        let ret = self.type_to_llvm(&info.ret_type);

        format!("declare {} @{}({})", ret, info.name, params.join(", "))
    }

    fn generate_function(&mut self, f: &Function) -> CodegenResult<String> {
        self.current_function = Some(f.name.node.clone());
        self.locals.clear();
        self.label_counter = 0;
        self.loop_stack.clear();

        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                let llvm_ty = self.type_to_llvm(&ty);

                // Register parameter as local (SSA value, not alloca)
                self.locals.insert(
                    p.name.node.clone(),
                    LocalVar {
                        ty: ty.clone(),
                        is_param: true,
                    },
                );

                format!("{} %{}", llvm_ty, p.name.node)
            })
            .collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        let ret_llvm = self.type_to_llvm(&ret_type);

        let mut ir = format!(
            "define {} @{}({}) {{\n",
            ret_llvm,
            f.name.node,
            params.join(", ")
        );

        ir.push_str("entry:\n");

        // Generate body
        match &f.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut 0)?;
                ir.push_str(&expr_ir);
                if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else {
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, value));
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir) = self.generate_block(stmts, &mut 0)?;
                ir.push_str(&block_ir);
                if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else {
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, value));
                }
            }
        }

        ir.push_str("}\n");

        self.current_function = None;
        Ok(ir)
    }

    fn generate_block(
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

    fn generate_stmt(
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
                let (val, val_ir) = self.generate_expr(value, counter)?;

                let resolved_ty = ty
                    .as_ref()
                    .map(|t| self.ast_type_to_resolved(&t.node))
                    .unwrap_or(ResolvedType::I64); // Default to i64

                self.locals.insert(
                    name.node.clone(),
                    LocalVar {
                        ty: resolved_ty.clone(),
                        is_param: false, // alloca'd variable
                    },
                );

                let llvm_ty = self.type_to_llvm(&resolved_ty);
                let mut ir = val_ir;

                // Allocate and store
                ir.push_str(&format!(
                    "  %{} = alloca {}\n",
                    name.node, llvm_ty
                ));
                ir.push_str(&format!(
                    "  store {} {}, {}* %{}\n",
                    llvm_ty, val, llvm_ty, name.node
                ));

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

    fn generate_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Int(n) => Ok((n.to_string(), String::new())),
            Expr::Float(n) => Ok((format!("{:e}", n), String::new())),
            Expr::Bool(b) => Ok((if *b { "1" } else { "0" }.to_string(), String::new())),
            Expr::String(s) => {
                // TODO: Proper string handling
                Ok((format!("\"{}\"", s), String::new()))
            }
            Expr::Unit => Ok(("void".to_string(), String::new())),

            Expr::Ident(name) => {
                if let Some(local) = self.locals.get(name) {
                    if local.is_param {
                        // Parameters are SSA values, use directly
                        Ok((format!("%{}", name), String::new()))
                    } else {
                        // Local variables need to be loaded from alloca
                        let tmp = self.next_temp(counter);
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        let ir = format!(
                            "  {} = load {}, {}* %{}\n",
                            tmp, llvm_ty, llvm_ty, name
                        );
                        Ok((tmp, ir))
                    }
                } else if name == "self" {
                    // Handle self reference
                    Ok(("%self".to_string(), String::new()))
                } else {
                    // Might be a function reference
                    Ok((format!("@{}", name), String::new()))
                }
            }

            Expr::SelfCall => {
                // @ refers to current function
                if let Some(fn_name) = &self.current_function {
                    Ok((format!("@{}", fn_name), String::new()))
                } else {
                    Err(CodegenError::UndefinedFunction("@".to_string()))
                }
            }

            Expr::Binary { op, left, right } => {
                let (left_val, left_ir) = self.generate_expr(left, counter)?;
                let (right_val, right_ir) = self.generate_expr(right, counter)?;

                let tmp = self.next_temp(counter);
                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "sdiv",
                    BinOp::Mod => "srem",
                    BinOp::Lt => "icmp slt",
                    BinOp::Lte => "icmp sle",
                    BinOp::Gt => "icmp sgt",
                    BinOp::Gte => "icmp sge",
                    BinOp::Eq => "icmp eq",
                    BinOp::Neq => "icmp ne",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::BitAnd => "and",
                    BinOp::BitOr => "or",
                    BinOp::BitXor => "xor",
                    BinOp::Shl => "shl",
                    BinOp::Shr => "ashr",
                };

                let mut ir = left_ir;
                ir.push_str(&right_ir);
                ir.push_str(&format!(
                    "  {} = {} i64 {}, {}\n",
                    tmp, op_str, left_val, right_val
                ));

                Ok((tmp, ir))
            }

            Expr::Unary { op, expr: inner } => {
                let (val, val_ir) = self.generate_expr(inner, counter)?;
                let tmp = self.next_temp(counter);

                let mut ir = val_ir;
                match op {
                    UnaryOp::Neg => {
                        ir.push_str(&format!("  {} = sub i64 0, {}\n", tmp, val));
                    }
                    UnaryOp::Not => {
                        ir.push_str(&format!("  {} = xor i1 {}, 1\n", tmp, val));
                    }
                    UnaryOp::BitNot => {
                        ir.push_str(&format!("  {} = xor i64 {}, -1\n", tmp, val));
                    }
                }

                Ok((tmp, ir))
            }

            Expr::Ternary { cond, then, else_ } => {
                // Use proper branching for lazy evaluation
                let then_label = self.next_label("ternary.then");
                let else_label = self.next_label("ternary.else");
                let merge_label = self.next_label("ternary.merge");

                // Generate condition
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Conditional branch
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_val, then_label, else_label
                ));

                // Then branch
                ir.push_str(&format!("{}:\n", then_label));
                let (then_val, then_ir) = self.generate_expr(then, counter)?;
                ir.push_str(&then_ir);
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Else branch
                ir.push_str(&format!("{}:\n", else_label));
                let (else_val, else_ir) = self.generate_expr(else_, counter)?;
                ir.push_str(&else_ir);
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Merge with phi
                ir.push_str(&format!("{}:\n", merge_label));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
                    result, then_val, then_label, else_val, else_label
                ));

                Ok((result, ir))
            }

            Expr::Call { func, args } => {
                let mut ir = String::new();
                let mut arg_vals = Vec::new();

                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    arg_vals.push(format!("i64 {}", val));
                }

                let fn_name = if let Expr::Ident(name) = &func.node {
                    name.clone()
                } else if let Expr::SelfCall = &func.node {
                    self.current_function.clone().unwrap_or_default()
                } else {
                    return Err(CodegenError::Unsupported("indirect call".to_string()));
                };

                let tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @{}({})\n",
                    tmp,
                    fn_name,
                    arg_vals.join(", ")
                ));

                Ok((tmp, ir))
            }

            // If/Else expression with basic blocks
            Expr::If { cond, then, else_ } => {
                let then_label = self.next_label("then");
                let else_label = self.next_label("else");
                let merge_label = self.next_label("merge");

                // Generate condition
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Conditional branch
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_val, then_label, else_label
                ));

                // Then block
                ir.push_str(&format!("{}:\n", then_label));
                let (then_val, then_ir) = self.generate_block_stmts(then, counter)?;
                ir.push_str(&then_ir);
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Else block
                ir.push_str(&format!("{}:\n", else_label));
                let (else_val, else_ir) = if let Some(else_branch) = else_ {
                    self.generate_if_else(else_branch, counter, &merge_label)?
                } else {
                    ("0".to_string(), String::new())
                };
                ir.push_str(&else_ir);
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Merge block with phi node
                ir.push_str(&format!("{}:\n", merge_label));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
                    result, then_val, then_label, else_val, else_label
                ));

                Ok((result, ir))
            }

            // Loop expression
            Expr::Loop { pattern: _, iter, body } => {
                let loop_start = self.next_label("loop.start");
                let loop_body = self.next_label("loop.body");
                let loop_end = self.next_label("loop.end");

                // Push loop labels for break/continue
                self.loop_stack.push(LoopLabels {
                    continue_label: loop_start.clone(),
                    break_label: loop_end.clone(),
                });

                let mut ir = String::new();

                // Check if this is a conditional loop (L cond { body }) or infinite loop
                if let Some(iter_expr) = iter {
                    // Conditional loop: L condition { body }
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                    ir.push_str(&format!("{}:\n", loop_start));

                    // Evaluate condition
                    let (cond_val, cond_ir) = self.generate_expr(iter_expr, counter)?;
                    ir.push_str(&cond_ir);
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        cond_val, loop_body, loop_end
                    ));

                    // Loop body
                    ir.push_str(&format!("{}:\n", loop_body));
                    let (_body_val, body_ir) = self.generate_block_stmts(body, counter)?;
                    ir.push_str(&body_ir);
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                } else {
                    // Infinite loop: L { body } - must use break to exit
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                    ir.push_str(&format!("{}:\n", loop_start));
                    let (_body_val, body_ir) = self.generate_block_stmts(body, counter)?;
                    ir.push_str(&body_ir);
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                }

                // Loop end
                ir.push_str(&format!("{}:\n", loop_end));

                self.loop_stack.pop();

                // Loop returns void by default (use break with value for expression)
                Ok(("0".to_string(), ir))
            }

            // Block expression
            Expr::Block(stmts) => self.generate_block_stmts(stmts, counter),

            // Assignment expression
            Expr::Assign { target, value } => {
                let (val, val_ir) = self.generate_expr(value, counter)?;
                let mut ir = val_ir;

                if let Expr::Ident(name) = &target.node {
                    if let Some(local) = self.locals.get(name) {
                        if !local.is_param {
                            let llvm_ty = self.type_to_llvm(&local.ty);
                            ir.push_str(&format!(
                                "  store {} {}, {}* %{}\n",
                                llvm_ty, val, llvm_ty, name
                            ));
                        }
                    }
                }

                Ok((val, ir))
            }

            // Compound assignment (+=, -=, etc.)
            Expr::AssignOp { op, target, value } => {
                // First load current value
                let (current_val, load_ir) = self.generate_expr(target, counter)?;
                let (rhs_val, rhs_ir) = self.generate_expr(value, counter)?;

                let mut ir = load_ir;
                ir.push_str(&rhs_ir);

                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "sdiv",
                    BinOp::Mod => "srem",
                    BinOp::BitAnd => "and",
                    BinOp::BitOr => "or",
                    BinOp::BitXor => "xor",
                    BinOp::Shl => "shl",
                    BinOp::Shr => "ashr",
                    _ => return Err(CodegenError::Unsupported(format!("compound {:?}", op))),
                };

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = {} i64 {}, {}\n",
                    result, op_str, current_val, rhs_val
                ));

                // Store back
                if let Expr::Ident(name) = &target.node {
                    if let Some(local) = self.locals.get(name.as_str()) {
                        if !local.is_param {
                            let llvm_ty = self.type_to_llvm(&local.ty);
                            ir.push_str(&format!(
                                "  store {} {}, {}* %{}\n",
                                llvm_ty, result, llvm_ty, name
                            ));
                        }
                    }
                }

                Ok((result, ir))
            }

            // TODO: Implement remaining expression types
            _ => Err(CodegenError::Unsupported(format!("{:?}", expr.node))),
        }
    }

    fn next_temp(&self, counter: &mut usize) -> String {
        let tmp = format!("%{}", counter);
        *counter += 1;
        tmp
    }

    /// Generate code for a block expression (used in if/else branches)
    #[allow(dead_code)]
    fn generate_block_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Block(stmts) => self.generate_block_stmts(stmts, counter),
            _ => self.generate_expr(expr, counter),
        }
    }

    /// Generate code for a block of statements
    fn generate_block_stmts(
        &mut self,
        stmts: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let mut last_value = "0".to_string();

        for stmt in stmts {
            let (value, stmt_ir) = self.generate_stmt(stmt, counter)?;
            ir.push_str(&stmt_ir);
            last_value = value;
        }

        Ok((last_value, ir))
    }

    /// Generate code for if-else branches
    fn generate_if_else(
        &mut self,
        if_else: &IfElse,
        counter: &mut usize,
        _merge_label: &str,
    ) -> CodegenResult<(String, String)> {
        match if_else {
            IfElse::Else(stmts) => {
                self.generate_block_stmts(stmts, counter)
            }
            IfElse::ElseIf(cond, then_stmts, else_branch) => {
                // Generate nested if-else
                let then_label = self.next_label("elseif.then");
                let else_label = self.next_label("elseif.else");
                let local_merge = self.next_label("elseif.merge");

                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_val, then_label, else_label
                ));

                // Then branch
                ir.push_str(&format!("{}:\n", then_label));
                let (then_val, then_ir) = self.generate_block_stmts(then_stmts, counter)?;
                ir.push_str(&then_ir);
                ir.push_str(&format!("  br label %{}\n", local_merge));

                // Else branch
                ir.push_str(&format!("{}:\n", else_label));
                let (else_val, else_ir) = if let Some(nested) = else_branch {
                    self.generate_if_else(nested, counter, &local_merge)?
                } else {
                    ("0".to_string(), String::new())
                };
                ir.push_str(&else_ir);
                ir.push_str(&format!("  br label %{}\n", local_merge));

                // Merge
                ir.push_str(&format!("{}:\n", local_merge));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
                    result, then_val, then_label, else_val, else_label
                ));

                Ok((result, ir))
            }
        }
    }

    fn type_to_llvm(&self, ty: &ResolvedType) -> String {
        match ty {
            ResolvedType::I8 => "i8".to_string(),
            ResolvedType::I16 => "i16".to_string(),
            ResolvedType::I32 => "i32".to_string(),
            ResolvedType::I64 => "i64".to_string(),
            ResolvedType::I128 => "i128".to_string(),
            ResolvedType::U8 => "i8".to_string(),
            ResolvedType::U16 => "i16".to_string(),
            ResolvedType::U32 => "i32".to_string(),
            ResolvedType::U64 => "i64".to_string(),
            ResolvedType::U128 => "i128".to_string(),
            ResolvedType::F32 => "float".to_string(),
            ResolvedType::F64 => "double".to_string(),
            ResolvedType::Bool => "i1".to_string(),
            ResolvedType::Str => "i8*".to_string(),
            ResolvedType::Unit => "void".to_string(),
            ResolvedType::Array(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Pointer(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Ref(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::RefMut(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Named { name, .. } => format!("%{}*", name),
            _ => "i64".to_string(), // Default fallback
        }
    }

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
                _ => ResolvedType::Named {
                    name: name.clone(),
                    generics: generics
                        .iter()
                        .map(|g| self.ast_type_to_resolved(&g.node))
                        .collect(),
                },
            },
            Type::Array(inner) => {
                ResolvedType::Array(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Unit => ResolvedType::Unit,
            _ => ResolvedType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_parser::parse;

    #[test]
    fn test_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @add"));
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_fibonacci() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @fib"));
        assert!(ir.contains("call i64 @fib"));
    }

    #[test]
    fn test_if_else() {
        // I cond { then } E { else }
        let source = "F max(a:i64,b:i64)->i64{I a>b{R a}E{R b}}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @max"));
        assert!(ir.contains("br i1"));
        assert!(ir.contains("then"));
        assert!(ir.contains("else"));
    }

    #[test]
    fn test_loop_with_condition() {
        // L pattern:iter { body } - `L _:condition{body}` for while loop
        let source = "F countdown(n:i64)->i64{x:=n;L _:x>0{x=x-1};x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @countdown"));
        assert!(ir.contains("loop.start"));
        assert!(ir.contains("loop.body"));
        assert!(ir.contains("loop.end"));
    }
}
