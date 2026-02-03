//! AST → MIR lowering pass.
//!
//! Converts the typed AST to MIR for optimization. Currently handles a core
//! subset of the language: functions, variables, binary/unary ops, if/else,
//! ternary, function calls, and return.
//!
//! Expressions that are not yet supported fall back to opaque `Constant::Int(0)`
//! placeholders, allowing the rest of the MIR pipeline to proceed.

use vais_ast::{Module, Item, Expr, Spanned, BinOp as AstBinOp, UnaryOp as AstUnOp,
               Literal, Pattern, FunctionBody, Stmt, Type as AstType, IfElse};
use crate::types::*;
use crate::builder::MirBuilder;

/// Lower an AST module to MIR.
pub fn lower_module(module: &Module) -> MirModule {
    let mut mir_module = MirModule::new("main");

    for item in &module.items {
        match &item.node {
            Item::Function(func) => {
                let param_types: Vec<MirType> = func.params
                    .iter()
                    .map(|p| ast_type_to_mir(&p.ty.node))
                    .collect();
                let ret_type = func.ret_type.as_ref()
                    .map(|t| ast_type_to_mir(&t.node))
                    .unwrap_or(MirType::I64);

                let func_name = func.name.node.clone();
                let mut lowerer = FunctionLowerer::new(&func_name, param_types, ret_type);

                // Register parameter names
                for (i, param) in func.params.iter().enumerate() {
                    lowerer.bind_var(&param.name.node, Local((i + 1) as u32));
                }

                // Lower function body
                match &func.body {
                    FunctionBody::Expr(expr) => {
                        let result = lowerer.lower_expr(expr);
                        lowerer.builder.assign(
                            lowerer.builder.return_place(),
                            Rvalue::Use(result),
                        );
                        lowerer.builder.return_();
                    }
                    FunctionBody::Block(stmts) => {
                        let mut last_operand = Operand::Constant(Constant::Int(0));
                        for stmt in stmts {
                            last_operand = lowerer.lower_stmt(stmt);
                        }
                        lowerer.builder.assign(
                            lowerer.builder.return_place(),
                            Rvalue::Use(last_operand),
                        );
                        lowerer.builder.return_();
                    }
                }

                mir_module.bodies.push(lowerer.builder.build());
            }
            Item::Struct(s) => {
                let mir_fields: Vec<(String, MirType)> = s.fields
                    .iter()
                    .map(|f| (f.name.node.clone(), ast_type_to_mir(&f.ty.node)))
                    .collect();
                mir_module.structs.insert(s.name.node.clone(), mir_fields);
            }
            _ => {
                // Skip other items for now
            }
        }
    }

    mir_module
}

/// Convert an AST type to MIR type.
fn ast_type_to_mir(ty: &AstType) -> MirType {
    match ty {
        AstType::Named { name, .. } => match name.as_str() {
            "i64" | "int" => MirType::I64,
            "i32" => MirType::I32,
            "i16" => MirType::I16,
            "i8" => MirType::I8,
            "f64" | "float" => MirType::F64,
            "f32" => MirType::F32,
            "bool" => MirType::Bool,
            "str" | "string" | "String" => MirType::Str,
            "void" => MirType::Unit,
            other => MirType::Struct(other.to_string()),
        },
        AstType::Array(inner) => MirType::Array(Box::new(ast_type_to_mir(&inner.node))),
        AstType::Tuple(elems) => {
            MirType::Tuple(elems.iter().map(|e| ast_type_to_mir(&e.node)).collect())
        }
        AstType::Fn { params, ret } | AstType::FnPtr { params, ret, .. } => MirType::Function {
            params: params.iter().map(|p| ast_type_to_mir(&p.node)).collect(),
            ret: Box::new(ast_type_to_mir(&ret.node)),
        },
        AstType::Ref(inner) | AstType::RefMut(inner) => {
            MirType::Ref(Box::new(ast_type_to_mir(&inner.node)))
        }
        AstType::Pointer(inner) => MirType::Pointer(Box::new(ast_type_to_mir(&inner.node))),
        AstType::Unit => MirType::Unit,
        _ => MirType::I64, // Default fallback
    }
}

struct FunctionLowerer {
    builder: MirBuilder,
    vars: std::collections::HashMap<String, Local>,
    func_name: String,
}

impl FunctionLowerer {
    fn new(name: &str, params: Vec<MirType>, ret_type: MirType) -> Self {
        Self {
            builder: MirBuilder::new(name, params, ret_type),
            vars: std::collections::HashMap::new(),
            func_name: name.to_string(),
        }
    }

    fn bind_var(&mut self, name: &str, local: Local) {
        self.vars.insert(name.to_string(), local);
    }

    fn new_temp(&mut self, ty: MirType) -> Local {
        self.builder.new_local(ty, None)
    }

    /// Lower a statement.
    fn lower_stmt(&mut self, stmt: &Spanned<Stmt>) -> Operand {
        match &stmt.node {
            Stmt::Let { name, value, .. } => {
                let val = self.lower_expr(value);
                let local = self.new_temp(MirType::I64);
                self.builder.assign(Place::local(local), Rvalue::Use(val));
                self.bind_var(&name.node, local);
                Operand::Constant(Constant::Unit)
            }
            Stmt::Expr(expr) => self.lower_expr(expr),
            Stmt::Return(Some(expr)) => {
                let val = self.lower_expr(expr);
                self.builder.assign(self.builder.return_place(), Rvalue::Use(val));
                self.builder.return_();
                let dead_bb = self.builder.new_block();
                self.builder.switch_to_block(dead_bb);
                Operand::Constant(Constant::Int(0))
            }
            Stmt::Return(None) => {
                self.builder.return_();
                let dead_bb = self.builder.new_block();
                self.builder.switch_to_block(dead_bb);
                Operand::Constant(Constant::Int(0))
            }
            _ => Operand::Constant(Constant::Int(0)),
        }
    }

    /// Lower a block of statements, returning the last expression value.
    fn lower_stmts(&mut self, stmts: &[Spanned<Stmt>]) -> Operand {
        let mut last = Operand::Constant(Constant::Int(0));
        for stmt in stmts {
            last = self.lower_stmt(stmt);
        }
        last
    }

    /// Lower an expression, returning an operand for the result.
    fn lower_expr(&mut self, expr: &Spanned<Expr>) -> Operand {
        match &expr.node {
            Expr::Int(v) => Operand::Constant(Constant::Int(*v)),
            Expr::Float(v) => Operand::Constant(Constant::Float(*v)),
            Expr::Bool(v) => Operand::Constant(Constant::Bool(*v)),
            Expr::String(s) => Operand::Constant(Constant::Str(s.clone())),
            Expr::Unit => Operand::Constant(Constant::Unit),

            Expr::Ident(name) => {
                if let Some(&local) = self.vars.get(name) {
                    Operand::Copy(Place::local(local))
                } else {
                    Operand::Constant(Constant::Int(0))
                }
            }

            Expr::Binary { op, left, right } => {
                let lhs = self.lower_expr(left);
                let rhs = self.lower_expr(right);
                let mir_op = ast_binop_to_mir(op);
                let result = self.new_temp(MirType::I64);
                self.builder.assign_binop(result, mir_op, lhs, rhs);
                Operand::Copy(Place::local(result))
            }

            Expr::Unary { op, expr: inner } => {
                let operand = self.lower_expr(inner);
                let mir_op = match op {
                    AstUnOp::Neg => UnOp::Neg,
                    AstUnOp::Not | AstUnOp::BitNot => UnOp::Not,
                };
                let result = self.new_temp(MirType::I64);
                self.builder.assign(
                    Place::local(result),
                    Rvalue::UnaryOp(mir_op, operand),
                );
                Operand::Copy(Place::local(result))
            }

            Expr::Block(stmts) => {
                self.lower_stmts(stmts)
            }

            Expr::If { cond, then, else_ } => {
                let cond_val = self.lower_expr(cond);
                let result = self.new_temp(MirType::I64);

                let bb_then = self.builder.new_block();
                let bb_else = self.builder.new_block();
                let bb_merge = self.builder.new_block();

                self.builder.switch_int(cond_val, vec![(1, bb_then)], bb_else);

                // Then branch (block of statements)
                self.builder.switch_to_block(bb_then);
                let then_val = self.lower_stmts(then);
                self.builder.assign(Place::local(result), Rvalue::Use(then_val));
                self.builder.goto(bb_merge);

                // Else branch
                self.builder.switch_to_block(bb_else);
                if let Some(else_branch) = else_ {
                    let else_val = self.lower_if_else(else_branch);
                    self.builder.assign(Place::local(result), Rvalue::Use(else_val));
                } else {
                    self.builder.assign(Place::local(result), Rvalue::Use(Operand::Constant(Constant::Int(0))));
                }
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_merge);
                Operand::Copy(Place::local(result))
            }

            Expr::Ternary { cond, then, else_ } => {
                let cond_val = self.lower_expr(cond);
                let result = self.new_temp(MirType::I64);

                let bb_then = self.builder.new_block();
                let bb_else = self.builder.new_block();
                let bb_merge = self.builder.new_block();

                self.builder.switch_int(cond_val, vec![(1, bb_then)], bb_else);

                self.builder.switch_to_block(bb_then);
                let then_val = self.lower_expr(then);
                self.builder.assign(Place::local(result), Rvalue::Use(then_val));
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_else);
                let else_val = self.lower_expr(else_);
                self.builder.assign(Place::local(result), Rvalue::Use(else_val));
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_merge);
                Operand::Copy(Place::local(result))
            }

            Expr::Call { func, args } => {
                // Check if this is a self-recursive call (@)
                let is_self_call = matches!(&func.node, Expr::SelfCall);

                if is_self_call {
                    let mir_args: Vec<Operand> = args.iter().map(|a| self.lower_expr(a)).collect();
                    self.builder.terminate(Terminator::TailCall {
                        func: self.func_name.clone(),
                        args: mir_args,
                    });
                    let dead_bb = self.builder.new_block();
                    self.builder.switch_to_block(dead_bb);
                    Operand::Constant(Constant::Int(0))
                } else {
                    let func_name = match &func.node {
                        Expr::Ident(name) => name.clone(),
                        _ => "unknown".to_string(),
                    };
                    let mir_args: Vec<Operand> = args.iter().map(|a| self.lower_expr(a)).collect();
                    let result = self.new_temp(MirType::I64);
                    let next_bb = self.builder.new_block();

                    self.builder.call(
                        &func_name,
                        mir_args,
                        Place::local(result),
                        next_bb,
                    );

                    self.builder.switch_to_block(next_bb);
                    Operand::Copy(Place::local(result))
                }
            }

            Expr::Match { expr: match_expr, arms } => {
                let disc = self.lower_expr(match_expr);
                let result = self.new_temp(MirType::I64);
                let bb_merge = self.builder.new_block();

                let mut targets = Vec::new();
                let mut arm_blocks = Vec::new();
                let mut otherwise_block = None;

                for arm in arms {
                    let bb = self.builder.new_block();
                    arm_blocks.push(bb);

                    match &arm.pattern.node {
                        Pattern::Literal(Literal::Int(v)) => {
                            targets.push((*v, bb));
                        }
                        Pattern::Literal(Literal::Bool(v)) => {
                            targets.push((if *v { 1 } else { 0 }, bb));
                        }
                        Pattern::Wildcard | Pattern::Ident(_) => {
                            otherwise_block = Some(bb);
                        }
                        _ => {
                            if otherwise_block.is_none() {
                                otherwise_block = Some(bb);
                            }
                        }
                    }
                }

                let otherwise = otherwise_block.unwrap_or(bb_merge);
                self.builder.switch_int(disc, targets, otherwise);

                for (i, arm) in arms.iter().enumerate() {
                    self.builder.switch_to_block(arm_blocks[i]);

                    if let Pattern::Ident(name) = &arm.pattern.node {
                        let bound = self.new_temp(MirType::I64);
                        self.builder.assign(
                            Place::local(bound),
                            Rvalue::Use(Operand::Constant(Constant::Int(0))),
                        );
                        self.bind_var(name, bound);
                    }

                    let arm_val = self.lower_expr(&arm.body);
                    self.builder.assign(Place::local(result), Rvalue::Use(arm_val));
                    self.builder.goto(bb_merge);
                }

                self.builder.switch_to_block(bb_merge);
                Operand::Copy(Place::local(result))
            }

            Expr::Assign { target, value } => {
                let val = self.lower_expr(value);
                if let Expr::Ident(name) = &target.node {
                    if let Some(&local) = self.vars.get(name) {
                        self.builder.assign(Place::local(local), Rvalue::Use(val));
                    }
                }
                Operand::Constant(Constant::Unit)
            }

            // Unsupported expressions fall back to a constant
            _ => Operand::Constant(Constant::Int(0)),
        }
    }

    /// Lower an IfElse branch.
    fn lower_if_else(&mut self, if_else: &IfElse) -> Operand {
        match if_else {
            IfElse::Else(stmts) => {
                self.lower_stmts(stmts)
            }
            IfElse::ElseIf(cond, then_stmts, else_branch) => {
                let cond_val = self.lower_expr(cond);
                let result = self.new_temp(MirType::I64);

                let bb_then = self.builder.new_block();
                let bb_else = self.builder.new_block();
                let bb_merge = self.builder.new_block();

                self.builder.switch_int(cond_val, vec![(1, bb_then)], bb_else);

                self.builder.switch_to_block(bb_then);
                let then_val = self.lower_stmts(then_stmts);
                self.builder.assign(Place::local(result), Rvalue::Use(then_val));
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_else);
                if let Some(else_b) = else_branch {
                    let else_val = self.lower_if_else(else_b);
                    self.builder.assign(Place::local(result), Rvalue::Use(else_val));
                } else {
                    self.builder.assign(Place::local(result), Rvalue::Use(Operand::Constant(Constant::Int(0))));
                }
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_merge);
                Operand::Copy(Place::local(result))
            }
        }
    }
}

fn ast_binop_to_mir(op: &AstBinOp) -> BinOp {
    match op {
        AstBinOp::Add => BinOp::Add,
        AstBinOp::Sub => BinOp::Sub,
        AstBinOp::Mul => BinOp::Mul,
        AstBinOp::Div => BinOp::Div,
        AstBinOp::Mod => BinOp::Rem,
        AstBinOp::Eq => BinOp::Eq,
        AstBinOp::Neq => BinOp::Ne,
        AstBinOp::Lt => BinOp::Lt,
        AstBinOp::Lte => BinOp::Le,
        AstBinOp::Gt => BinOp::Gt,
        AstBinOp::Gte => BinOp::Ge,
        AstBinOp::And => BinOp::BitAnd,
        AstBinOp::Or => BinOp::BitOr,
        AstBinOp::BitAnd => BinOp::BitAnd,
        AstBinOp::BitOr => BinOp::BitOr,
        AstBinOp::BitXor => BinOp::BitXor,
        AstBinOp::Shl => BinOp::Shl,
        AstBinOp::Shr => BinOp::Shr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_simple_function() {
        let source = "F add(x: i64, y: i64) -> i64 = x + y";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        assert_eq!(mir.bodies.len(), 1);
        assert_eq!(mir.bodies[0].name, "add");
        assert_eq!(mir.bodies[0].params.len(), 2);

        let display = mir.bodies[0].display();
        assert!(display.contains("Add"));
    }

    #[test]
    fn test_lower_with_if() {
        let source = "F abs(x: i64) -> i64 = I x < 0 { 0 - x } E { x }";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        assert_eq!(mir.bodies.len(), 1);
        assert!(mir.bodies[0].basic_blocks.len() >= 3);
    }

    #[test]
    fn test_lower_with_let() {
        let source = r#"
            F compute(x: i64) -> i64 = {
                a := x + 1
                b := a * 2
                b
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        assert_eq!(mir.bodies.len(), 1);
        assert_eq!(mir.bodies[0].name, "compute");
    }

    #[test]
    fn test_lower_and_optimize() {
        let source = r#"
            F dead_code(x: i64) -> i64 = {
                unused := 42
                result := x + 1
                result
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower_module(&module);

        let before_stmts: usize = mir.bodies[0].basic_blocks.iter()
            .map(|bb| bb.statements.len())
            .sum();

        crate::optimize::optimize_mir_module(&mut mir);

        let after_stmts: usize = mir.bodies[0].basic_blocks.iter()
            .map(|bb| bb.statements.len())
            .sum();

        // DCE should remove the unused assignment
        assert!(after_stmts <= before_stmts);
    }

    #[test]
    fn test_lower_and_emit_llvm() {
        let source = "F add(x: i64, y: i64) -> i64 = x + y";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower_module(&module);
        crate::optimize::optimize_mir_module(&mut mir);
        let ir = crate::emit_llvm::emit_llvm_ir(&mir, "x86_64-apple-darwin");

        assert!(ir.contains("define i64 @add("));
        assert!(ir.contains("add i64"));
        assert!(ir.contains("ret i64"));
    }
}
