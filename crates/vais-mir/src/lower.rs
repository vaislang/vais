//! AST → MIR lowering pass.
//!
//! Converts the typed AST to MIR for optimization. Currently handles a core
//! subset of the language: functions, variables, binary/unary ops, if/else,
//! ternary, function calls, and return.
//!
//! `lower_module` preserves the legacy permissive behavior used by older tests:
//! expressions that are not yet supported fall back to opaque `Constant::Int(0)`
//! placeholders, allowing the rest of the MIR pipeline to proceed.
//!
//! New certification work must use `lower_module_checked`, which reports those
//! semantic-loss fallbacks as errors instead of treating the MIR as trustworthy.

use crate::builder::MirBuilder;
use crate::types::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use vais_ast::{
    BinOp as AstBinOp, Expr, FunctionBody, IfElse, Item, Literal, Module, Pattern, Span, Spanned,
    Stmt, Type as AstType, UnaryOp as AstUnOp, VariantFields,
};

/// A MIR lowering error produced by strict lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirLowerError {
    pub span: Span,
    pub message: String,
}

impl MirLowerError {
    fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

impl fmt::Display for MirLowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.span.file_id != 0 || self.span.start != 0 || self.span.end != 0 {
            write!(
                f,
                "{}..{}: {}",
                self.span.start, self.span.end, self.message
            )
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for MirLowerError {}

/// Lower an AST module to MIR.
pub fn lower_module(module: &Module) -> MirModule {
    lower_module_internal(module, false).0
}

/// Lower an AST module to MIR, rejecting semantic-loss placeholders.
pub fn lower_module_checked(module: &Module) -> Result<MirModule, Vec<MirLowerError>> {
    let (module, errors) = lower_module_internal(module, true);
    if errors.is_empty() {
        Ok(module)
    } else {
        Err(errors)
    }
}

fn lower_module_internal(module: &Module, strict: bool) -> (MirModule, Vec<MirLowerError>) {
    let mut mir_module = MirModule::new("main");
    let mut errors = Vec::new();
    let enum_names: HashSet<String> = module
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Enum(e) => Some(e.name.node.clone()),
            _ => None,
        })
        .collect();

    for item in &module.items {
        match &item.node {
            Item::Struct(s) => {
                let mir_fields: Vec<(String, MirType)> = s
                    .fields
                    .iter()
                    .map(|f| {
                        (
                            f.name.node.clone(),
                            ast_type_to_mir_strict(
                                &f.ty.node,
                                f.ty.span,
                                strict,
                                &mut errors,
                                &enum_names,
                            ),
                        )
                    })
                    .collect();
                mir_module.structs.insert(s.name.node.clone(), mir_fields);
            }
            Item::Enum(e) => {
                let variants = e
                    .variants
                    .iter()
                    .map(|variant| {
                        let fields = match &variant.fields {
                            VariantFields::Unit => vec![],
                            VariantFields::Tuple(items) => items
                                .iter()
                                .map(|t| {
                                    ast_type_to_mir_strict(
                                        &t.node,
                                        t.span,
                                        strict,
                                        &mut errors,
                                        &enum_names,
                                    )
                                })
                                .collect(),
                            VariantFields::Struct(fields) => fields
                                .iter()
                                .map(|f| {
                                    ast_type_to_mir_strict(
                                        &f.ty.node,
                                        f.ty.span,
                                        strict,
                                        &mut errors,
                                        &enum_names,
                                    )
                                })
                                .collect(),
                        };
                        (variant.name.node.clone(), fields)
                    })
                    .collect();
                mir_module.enums.insert(e.name.node.clone(), variants);
            }
            _ => {}
        }
    }
    mir_module
        .enums
        .entry(option_i64_enum_name())
        .or_insert_with(builtin_option_i64_variants);
    mir_module
        .enums
        .entry(result_i64_i64_enum_name())
        .or_insert_with(builtin_result_i64_i64_variants);

    for item in &module.items {
        match &item.node {
            Item::Function(func) => {
                let param_types: Vec<MirType> = func
                    .params
                    .iter()
                    .map(|p| {
                        ast_type_to_mir_strict(
                            &p.ty.node,
                            p.ty.span,
                            strict,
                            &mut errors,
                            &enum_names,
                        )
                    })
                    .collect();
                let ret_type = match &func.ret_type {
                    Some(t) => {
                        ast_type_to_mir_strict(&t.node, t.span, strict, &mut errors, &enum_names)
                    }
                    None => {
                        if strict {
                            errors.push(MirLowerError::new(
                                item.span,
                                "strict MIR lowering requires an explicit function return type",
                            ));
                        }
                        MirType::I64
                    }
                };

                let func_name = func.name.node.clone();
                let mut lowerer = FunctionLowerer::new(
                    &func_name,
                    param_types,
                    ret_type,
                    strict,
                    mir_module.structs.clone(),
                    mir_module.enums.clone(),
                );

                // Register parameter names
                for (i, param) in func.params.iter().enumerate() {
                    lowerer.bind_var(&param.name.node, Local((i + 1) as u32));
                }

                // Lower function body
                match &func.body {
                    FunctionBody::Expr(expr) => {
                        let result = lowerer.lower_expr(expr);
                        lowerer
                            .builder
                            .assign(lowerer.builder.return_place(), Rvalue::Use(result));
                        lowerer.emit_drops();
                        lowerer.builder.return_();
                    }
                    FunctionBody::Block(stmts) => {
                        let mut last_operand = Operand::Constant(Constant::Int(0));
                        for stmt in stmts {
                            last_operand = lowerer.lower_stmt(stmt);
                        }
                        lowerer
                            .builder
                            .assign(lowerer.builder.return_place(), Rvalue::Use(last_operand));
                        lowerer.emit_drops();
                        lowerer.builder.return_();
                    }
                }

                errors.extend(lowerer.errors);
                mir_module.bodies.push(lowerer.builder.build());
            }
            Item::Struct(_) | Item::Enum(_) => {}
            Item::Use(_) => {}
            _ => {
                if strict {
                    errors.push(MirLowerError::new(
                        item.span,
                        format!(
                            "unsupported top-level item for strict MIR lowering: {:?}",
                            item.node
                        ),
                    ));
                }
            }
        }
    }

    (mir_module, errors)
}

fn ast_type_to_mir_strict(
    ty: &AstType,
    span: Span,
    strict: bool,
    errors: &mut Vec<MirLowerError>,
    enum_names: &HashSet<String>,
) -> MirType {
    match ty {
        AstType::Named { name, generics } if name == "Option" => {
            let option_type = if generics.len() == 1 {
                let payload_ty = ast_type_to_mir_strict(
                    &generics[0].node,
                    generics[0].span,
                    strict,
                    errors,
                    enum_names,
                );
                option_enum_name(&payload_ty)
            } else {
                None
            };

            if let Some(option_type) = option_type {
                MirType::Enum(option_type)
            } else {
                if strict {
                    errors.push(MirLowerError::new(
                        span,
                        "unsupported Option payload type for strict MIR lowering; only Option<i64> is certified",
                    ));
                }
                MirType::Enum(option_i64_enum_name())
            }
        }
        AstType::Named { name, generics } if name == "Result" => {
            let result_type = if generics.len() == 2 {
                let ok_ty = ast_type_to_mir_strict(
                    &generics[0].node,
                    generics[0].span,
                    strict,
                    errors,
                    enum_names,
                );
                let err_ty = ast_type_to_mir_strict(
                    &generics[1].node,
                    generics[1].span,
                    strict,
                    errors,
                    enum_names,
                );
                result_enum_name(&ok_ty, &err_ty)
            } else {
                None
            };

            if let Some(result_type) = result_type {
                MirType::Enum(result_type)
            } else {
                if strict {
                    errors.push(MirLowerError::new(
                        span,
                        "unsupported Result payload types for strict MIR lowering; only Result<i64,i64> is certified",
                    ));
                }
                MirType::Enum(result_i64_i64_enum_name())
            }
        }
        AstType::Named { name, generics } if name == "Vec" => {
            let vec_type = if generics.len() == 1 {
                let elem_ty = ast_type_to_mir_strict(
                    &generics[0].node,
                    generics[0].span,
                    strict,
                    errors,
                    enum_names,
                );
                (elem_ty == MirType::I64).then_some(elem_ty)
            } else {
                None
            };

            if let Some(elem_ty) = vec_type {
                MirType::Vec(Box::new(elem_ty))
            } else {
                if strict {
                    errors.push(MirLowerError::new(
                        span,
                        "unsupported Vec element type for strict MIR lowering; only Vec<i64> is certified",
                    ));
                }
                MirType::Vec(Box::new(MirType::I64))
            }
        }
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
            other if enum_names.contains(other) => MirType::Enum(other.to_string()),
            other => MirType::Struct(other.to_string()),
        },
        AstType::Array(inner) => MirType::Array(Box::new(ast_type_to_mir_strict(
            &inner.node,
            inner.span,
            strict,
            errors,
            enum_names,
        ))),
        AstType::Tuple(elems) => MirType::Tuple(
            elems
                .iter()
                .map(|e| ast_type_to_mir_strict(&e.node, e.span, strict, errors, enum_names))
                .collect(),
        ),
        AstType::Fn { params, ret } | AstType::FnPtr { params, ret, .. } => MirType::Function {
            params: params
                .iter()
                .map(|p| ast_type_to_mir_strict(&p.node, p.span, strict, errors, enum_names))
                .collect(),
            ret: Box::new(ast_type_to_mir_strict(
                &ret.node, ret.span, strict, errors, enum_names,
            )),
        },
        AstType::Ref(inner) | AstType::RefMut(inner) => MirType::Ref(Box::new(
            ast_type_to_mir_strict(&inner.node, inner.span, strict, errors, enum_names),
        )),
        AstType::RefLifetime { lifetime, inner } => MirType::RefLifetime {
            lifetime: lifetime.clone(),
            inner: Box::new(ast_type_to_mir_strict(
                &inner.node,
                inner.span,
                strict,
                errors,
                enum_names,
            )),
        },
        AstType::RefMutLifetime { lifetime, inner } => MirType::RefMutLifetime {
            lifetime: lifetime.clone(),
            inner: Box::new(ast_type_to_mir_strict(
                &inner.node,
                inner.span,
                strict,
                errors,
                enum_names,
            )),
        },
        AstType::Pointer(inner) => MirType::Pointer(Box::new(ast_type_to_mir_strict(
            &inner.node,
            inner.span,
            strict,
            errors,
            enum_names,
        ))),
        AstType::Unit => MirType::Unit,
        _ => {
            if strict {
                errors.push(MirLowerError::new(
                    span,
                    format!("unsupported type for strict MIR lowering: {:?}", ty),
                ));
            }
            MirType::I64
        }
    }
}

fn option_i64_enum_name() -> String {
    "Option<i64>".to_string()
}

fn option_enum_name(payload_ty: &MirType) -> Option<String> {
    match payload_ty {
        MirType::I64 => Some(option_i64_enum_name()),
        _ => None,
    }
}

fn builtin_option_i64_variants() -> Vec<(String, Vec<MirType>)> {
    vec![
        ("None".to_string(), vec![]),
        ("Some".to_string(), vec![MirType::I64]),
    ]
}

fn result_i64_i64_enum_name() -> String {
    "Result<i64,i64>".to_string()
}

fn result_enum_name(ok_ty: &MirType, err_ty: &MirType) -> Option<String> {
    match (ok_ty, err_ty) {
        (MirType::I64, MirType::I64) => Some(result_i64_i64_enum_name()),
        _ => None,
    }
}

fn builtin_result_i64_i64_variants() -> Vec<(String, Vec<MirType>)> {
    vec![
        ("Ok".to_string(), vec![MirType::I64]),
        ("Err".to_string(), vec![MirType::I64]),
    ]
}

struct FunctionLowerer {
    builder: MirBuilder,
    vars: HashMap<String, Local>,
    func_name: String,
    local_types: HashMap<Local, MirType>,
    structs: HashMap<String, Vec<(String, MirType)>>,
    enums: HashMap<String, Vec<(String, Vec<MirType>)>>,
    enum_names: HashSet<String>,
    moved_locals: HashSet<Local>,
    strict: bool,
    errors: Vec<MirLowerError>,
}

impl FunctionLowerer {
    fn new(
        name: &str,
        params: Vec<MirType>,
        ret_type: MirType,
        strict: bool,
        structs: HashMap<String, Vec<(String, MirType)>>,
        enums: HashMap<String, Vec<(String, Vec<MirType>)>>,
    ) -> Self {
        let builder = MirBuilder::new(name, params.clone(), ret_type);
        let mut local_types = HashMap::new();
        let enum_names = enums.keys().cloned().collect();

        // Register parameter types (_1, _2, ...)
        for (i, ty) in params.iter().enumerate() {
            local_types.insert(Local((i + 1) as u32), ty.clone());
        }

        Self {
            builder,
            vars: HashMap::new(),
            func_name: name.to_string(),
            local_types,
            structs,
            enums,
            enum_names,
            moved_locals: HashSet::new(),
            strict,
            errors: Vec::new(),
        }
    }

    fn semantic_loss(&mut self, span: Span, message: impl Into<String>) -> Operand {
        if self.strict {
            self.errors.push(MirLowerError::new(span, message));
        }
        Operand::Constant(Constant::Int(0))
    }

    fn strict_error(&mut self, span: Span, message: impl Into<String>) {
        if self.strict {
            self.errors.push(MirLowerError::new(span, message));
        }
    }

    fn bind_var(&mut self, name: &str, local: Local) {
        self.vars.insert(name.to_string(), local);
    }

    fn new_temp(&mut self, ty: MirType) -> Local {
        let local = self.builder.new_local(ty.clone(), None);
        self.local_types.insert(local, ty);
        local
    }

    /// Emit Drop statements for non-Copy locals that haven't been moved yet.
    /// Call this before function return to clean up remaining values.
    fn emit_drops(&mut self) {
        for (&local, ty) in &self.local_types {
            if !ty.is_copy() && !self.moved_locals.contains(&local) {
                self.builder.drop(Place::local(local));
            }
        }
    }

    /// Lower a statement.
    fn lower_stmt(&mut self, stmt: &Spanned<Stmt>) -> Operand {
        match &stmt.node {
            Stmt::Let {
                name, value, ty, ..
            } => {
                let mir_type = ty
                    .as_ref()
                    .map(|t| self.ast_type_to_mir(&t.node, t.span))
                    .unwrap_or(MirType::I64);
                let val = self.lower_expr_with_expected(value, Some(&mir_type));
                let local = self.new_temp(mir_type);
                self.builder.assign(Place::local(local), Rvalue::Use(val));
                self.bind_var(&name.node, local);
                Operand::Constant(Constant::Unit)
            }
            Stmt::Expr(expr) => self.lower_expr(expr),
            Stmt::Return(Some(expr)) => {
                let val = self.lower_expr(expr);
                self.builder
                    .assign(self.builder.return_place(), Rvalue::Use(val));
                self.emit_drops();
                self.builder.return_();
                let dead_bb = self.builder.new_block();
                self.builder.switch_to_block(dead_bb);
                Operand::Constant(Constant::Int(0))
            }
            Stmt::Return(None) => {
                self.emit_drops();
                self.builder.return_();
                let dead_bb = self.builder.new_block();
                self.builder.switch_to_block(dead_bb);
                Operand::Constant(Constant::Int(0))
            }
            _ => self.semantic_loss(
                stmt.span,
                format!(
                    "unsupported statement for strict MIR lowering: {:?}",
                    stmt.node
                ),
            ),
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

    fn ast_type_to_mir(&mut self, ty: &AstType, span: Span) -> MirType {
        ast_type_to_mir_strict(ty, span, self.strict, &mut self.errors, &self.enum_names)
    }

    fn lower_expr_with_expected(
        &mut self,
        expr: &Spanned<Expr>,
        expected: Option<&MirType>,
    ) -> Operand {
        if let Some(MirType::Enum(enum_name)) = expected {
            match &expr.node {
                Expr::Ident(variant) if self.enum_variant_info(enum_name, variant).is_some() => {
                    return self.lower_enum_variant_literal(expr.span, enum_name, variant, None);
                }
                Expr::EnumAccess {
                    enum_name: access_enum,
                    variant,
                    data,
                } if access_enum == enum_name => {
                    return self.lower_enum_variant_literal(
                        expr.span,
                        access_enum,
                        variant,
                        data.as_deref(),
                    );
                }
                Expr::Call { func, args } => {
                    if let Expr::Ident(variant) = &func.node {
                        if self.enum_variant_info(enum_name, variant).is_some() {
                            let args = args.iter().collect::<Vec<_>>();
                            return self
                                .lower_enum_variant_args(expr.span, enum_name, variant, &args);
                        }
                    }
                }
                Expr::EnumAccess {
                    enum_name: access_enum,
                    variant,
                    ..
                } => {
                    self.strict_error(
                        expr.span,
                        format!(
                            "enum variant `{}::{}` does not match expected enum `{}`",
                            access_enum, variant, enum_name
                        ),
                    );
                }
                _ => {}
            }
        }

        if let Some(MirType::Vec(elem_ty)) = expected {
            if let Expr::Call { func, args } = &expr.node {
                if matches!(&func.node, Expr::Ident(name) if name == "vec_new") {
                    if !args.is_empty() {
                        self.strict_error(
                            expr.span,
                            format!(
                                "vec_new for strict MIR expects 0 arguments, got {}",
                                args.len()
                            ),
                        );
                    }
                    return self.lower_vec_new(expr.span, elem_ty.as_ref());
                }
            }
        }

        self.lower_expr(expr)
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
                    let ty = self
                        .local_types
                        .get(&local)
                        .cloned()
                        .unwrap_or(MirType::I64);
                    if ty.is_copy() {
                        Operand::Copy(Place::local(local))
                    } else {
                        self.moved_locals.insert(local);
                        Operand::Move(Place::local(local))
                    }
                } else {
                    self.semantic_loss(
                        expr.span,
                        format!("unbound identifier `{}` during MIR lowering", name),
                    )
                }
            }

            Expr::Binary { op, left, right } => {
                let lhs = self.lower_expr(left);
                let rhs = self.lower_expr(right);
                let mir_op = ast_binop_to_mir(op);
                let result = self.new_temp(self.infer_binary_type(op, left, right));
                self.builder.assign_binop(result, mir_op, lhs, rhs);
                Operand::Copy(Place::local(result))
            }

            Expr::Unary { op, expr: inner } => {
                let operand = self.lower_expr(inner);
                let mir_op = match op {
                    AstUnOp::Neg => UnOp::Neg,
                    AstUnOp::Not | AstUnOp::BitNot => UnOp::Not,
                };
                let result = self.new_temp(self.infer_unary_type(op, inner));
                self.builder
                    .assign(Place::local(result), Rvalue::UnaryOp(mir_op, operand));
                Operand::Copy(Place::local(result))
            }

            Expr::Block(stmts) => self.lower_stmts(stmts),

            Expr::If { cond, then, else_ } => {
                let cond_val = self.lower_expr(cond);
                let result = self.new_temp(MirType::I64);

                let bb_then = self.builder.new_block();
                let bb_else = self.builder.new_block();
                let bb_merge = self.builder.new_block();

                self.builder
                    .switch_int(cond_val, vec![(1, bb_then)], bb_else);

                // Then branch (block of statements)
                self.builder.switch_to_block(bb_then);
                let then_val = self.lower_stmts(then);
                self.builder
                    .assign(Place::local(result), Rvalue::Use(then_val));
                self.builder.goto(bb_merge);

                // Else branch
                self.builder.switch_to_block(bb_else);
                if let Some(else_branch) = else_ {
                    let else_val = self.lower_if_else(else_branch);
                    self.builder
                        .assign(Place::local(result), Rvalue::Use(else_val));
                } else {
                    self.builder.assign(
                        Place::local(result),
                        Rvalue::Use(Operand::Constant(Constant::Int(0))),
                    );
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

                self.builder
                    .switch_int(cond_val, vec![(1, bb_then)], bb_else);

                self.builder.switch_to_block(bb_then);
                let then_val = self.lower_expr(then);
                self.builder
                    .assign(Place::local(result), Rvalue::Use(then_val));
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_else);
                let else_val = self.lower_expr(else_);
                self.builder
                    .assign(Place::local(result), Rvalue::Use(else_val));
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_merge);
                Operand::Copy(Place::local(result))
            }

            Expr::While { condition, body } => {
                let bb_header = self.builder.new_block();
                let bb_body = self.builder.new_block();
                let bb_exit = self.builder.new_block();

                self.builder.goto(bb_header);

                self.builder.switch_to_block(bb_header);
                let cond_val = self.lower_expr(condition);
                self.builder
                    .switch_int(cond_val, vec![(1, bb_body)], bb_exit);

                self.builder.switch_to_block(bb_body);
                self.lower_stmts(body);
                self.builder.goto(bb_header);

                self.builder.switch_to_block(bb_exit);
                Operand::Constant(Constant::Unit)
            }

            Expr::StructLit {
                name,
                fields,
                enum_name,
            } => self.lower_struct_literal(expr.span, name, fields, enum_name.as_deref()),

            Expr::EnumAccess {
                enum_name,
                variant,
                data,
            } => self.lower_enum_variant_literal(expr.span, enum_name, variant, data.as_deref()),

            Expr::Field { expr: base, field } => self.lower_field_access(expr.span, base, field),

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
                        _ => {
                            if self.strict {
                                self.errors.push(MirLowerError::new(
                                    func.span,
                                    "unsupported non-identifier call target for strict MIR lowering",
                                ));
                            }
                            "unknown".to_string()
                        }
                    };
                    let mir_args: Vec<Operand> = args.iter().map(|a| self.lower_expr(a)).collect();
                    let result = self.new_temp(MirType::I64);
                    let next_bb = self.builder.new_block();

                    self.builder
                        .call(&func_name, mir_args, Place::local(result), next_bb);

                    self.builder.switch_to_block(next_bb);
                    Operand::Copy(Place::local(result))
                }
            }

            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.lower_method_call(expr.span, receiver, method, args),

            Expr::Index { expr: base, index } => self.lower_index_access(expr.span, base, index),

            Expr::Match {
                expr: match_expr,
                arms,
            } => {
                let place_match = self.lower_place_expr(match_expr);
                let match_ty = place_match
                    .as_ref()
                    .map(|(_, ty)| ty.clone())
                    .unwrap_or_else(|| self.infer_expr_type(match_expr));
                let enum_match_place = match &place_match {
                    Some((place, MirType::Enum(_))) => Some(place.clone()),
                    _ => None,
                };
                let disc = if let Some((place, MirType::Enum(_))) = &place_match {
                    let disc_local = self.new_temp(MirType::I64);
                    self.builder.assign(
                        Place::local(disc_local),
                        Rvalue::Discriminant(place.clone()),
                    );
                    Operand::Copy(Place::local(disc_local))
                } else {
                    self.lower_expr(match_expr)
                };
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
                        pattern if self.enum_pattern_discriminant(&match_ty, pattern).is_some() => {
                            let discriminant =
                                self.enum_pattern_discriminant(&match_ty, pattern).unwrap();
                            targets.push((discriminant as i64, bb));
                        }
                        Pattern::Wildcard => {
                            otherwise_block = Some(bb);
                        }
                        Pattern::Ident(name) => {
                            self.strict_error(
                                arm.pattern.span,
                                format!(
                                    "match binding pattern `{}` would be lowered through a placeholder",
                                    name
                                ),
                            );
                            otherwise_block = Some(bb);
                        }
                        _ => {
                            self.strict_error(
                                arm.pattern.span,
                                format!(
                                    "unsupported match pattern for strict MIR lowering: {:?}",
                                    arm.pattern.node
                                ),
                            );
                            if otherwise_block.is_none() {
                                otherwise_block = Some(bb);
                            }
                        }
                    }
                }

                let otherwise = otherwise_block.unwrap_or(bb_merge);
                self.builder.switch_int(disc, targets, otherwise);

                let vars_before_match = self.vars.clone();
                for (i, arm) in arms.iter().enumerate() {
                    self.builder.switch_to_block(arm_blocks[i]);
                    self.vars = vars_before_match.clone();

                    if let Some(match_place) = &enum_match_place {
                        self.bind_enum_payload_pattern(
                            arm.pattern.span,
                            &match_ty,
                            &arm.pattern.node,
                            match_place,
                        );
                    }

                    if let Pattern::Ident(name) = &arm.pattern.node {
                        if self
                            .enum_pattern_discriminant(&match_ty, &arm.pattern.node)
                            .is_none()
                        {
                            let bound = self.new_temp(MirType::I64);
                            self.builder.assign(
                                Place::local(bound),
                                Rvalue::Use(Operand::Constant(Constant::Int(0))),
                            );
                            self.bind_var(name, bound);
                        }
                    }

                    let arm_val = self.lower_expr(&arm.body);
                    self.builder
                        .assign(Place::local(result), Rvalue::Use(arm_val));
                    self.builder.goto(bb_merge);
                }
                self.vars = vars_before_match;

                self.builder.switch_to_block(bb_merge);
                Operand::Copy(Place::local(result))
            }

            Expr::Assign { target, value } => {
                let val = self.lower_expr(value);
                if let Expr::Ident(name) = &target.node {
                    if let Some(&local) = self.vars.get(name) {
                        self.builder.assign(Place::local(local), Rvalue::Use(val));
                    } else if self.strict {
                        self.errors.push(MirLowerError::new(
                            target.span,
                            format!("assignment target `{}` is not a declared local", name),
                        ));
                    }
                } else if self.strict {
                    self.errors.push(MirLowerError::new(
                        target.span,
                        "unsupported assignment target for strict MIR lowering",
                    ));
                }
                Operand::Constant(Constant::Unit)
            }

            // Unsupported expressions fall back to a constant
            _ => self.semantic_loss(
                expr.span,
                format!(
                    "unsupported expression for strict MIR lowering: {:?}",
                    expr.node
                ),
            ),
        }
    }

    /// Lower an IfElse branch.
    fn lower_if_else(&mut self, if_else: &IfElse) -> Operand {
        match if_else {
            IfElse::Else(stmts) => self.lower_stmts(stmts),
            IfElse::ElseIf(cond, then_stmts, else_branch) => {
                let cond_val = self.lower_expr(cond);
                let result = self.new_temp(MirType::I64);

                let bb_then = self.builder.new_block();
                let bb_else = self.builder.new_block();
                let bb_merge = self.builder.new_block();

                self.builder
                    .switch_int(cond_val, vec![(1, bb_then)], bb_else);

                self.builder.switch_to_block(bb_then);
                let then_val = self.lower_stmts(then_stmts);
                self.builder
                    .assign(Place::local(result), Rvalue::Use(then_val));
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_else);
                if let Some(else_b) = else_branch {
                    let else_val = self.lower_if_else(else_b);
                    self.builder
                        .assign(Place::local(result), Rvalue::Use(else_val));
                } else {
                    self.builder.assign(
                        Place::local(result),
                        Rvalue::Use(Operand::Constant(Constant::Int(0))),
                    );
                }
                self.builder.goto(bb_merge);

                self.builder.switch_to_block(bb_merge);
                Operand::Copy(Place::local(result))
            }
        }
    }

    fn lower_vec_new(&mut self, _span: Span, elem_ty: &MirType) -> Operand {
        let result = self.new_temp(MirType::Vec(Box::new(elem_ty.clone())));
        self.builder.assign(
            Place::local(result),
            Rvalue::Aggregate(AggregateKind::Vec, vec![]),
        );
        Operand::Move(Place::local(result))
    }

    fn lower_method_call(
        &mut self,
        span: Span,
        receiver: &Spanned<Expr>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
    ) -> Operand {
        let Some((receiver_place, receiver_ty)) = self.lower_place_expr(receiver) else {
            return self.semantic_loss(
                span,
                format!(
                    "unsupported MethodCall receiver for strict MIR lowering: {:?}",
                    receiver.node
                ),
            );
        };

        let MirType::Vec(elem_ty) = receiver_ty else {
            return self.semantic_loss(
                span,
                format!(
                    "unsupported MethodCall `{}` for strict MIR lowering on type {:?}",
                    method.node, receiver_ty
                ),
            );
        };

        match method.node.as_str() {
            "push" => {
                if args.len() != 1 {
                    self.strict_error(
                        method.span,
                        format!("Vec<i64>.push expects 1 argument, got {}", args.len()),
                    );
                }
                let value = args
                    .first()
                    .map(|arg| self.lower_expr_with_expected(arg, Some(elem_ty.as_ref())))
                    .unwrap_or(Operand::Constant(Constant::Unit));
                self.builder.assign(
                    receiver_place.clone(),
                    Rvalue::VecPush(receiver_place, value),
                );
                Operand::Constant(Constant::Unit)
            }
            "len" => {
                if !args.is_empty() {
                    self.strict_error(
                        method.span,
                        format!("Vec<i64>.len expects 0 arguments, got {}", args.len()),
                    );
                }
                let result = self.new_temp(MirType::I64);
                self.builder
                    .assign(Place::local(result), Rvalue::Len(receiver_place));
                Operand::Copy(Place::local(result))
            }
            _ => self.semantic_loss(
                method.span,
                format!(
                    "unsupported MethodCall `{}` for strict MIR lowering on Vec<i64>",
                    method.node
                ),
            ),
        }
    }

    fn lower_index_access(
        &mut self,
        span: Span,
        base: &Spanned<Expr>,
        index: &Spanned<Expr>,
    ) -> Operand {
        let Some((base_place, base_ty)) = self.lower_place_expr(base) else {
            return self.semantic_loss(
                span,
                format!(
                    "unsupported Index base for strict MIR lowering: {:?}",
                    base.node
                ),
            );
        };

        let MirType::Vec(elem_ty) = base_ty else {
            return self.semantic_loss(
                span,
                format!(
                    "unsupported Index for strict MIR lowering on type {:?}",
                    base_ty
                ),
            );
        };

        let index_operand = self.lower_expr(index);
        let index_local = self.new_temp(MirType::I64);
        self.builder
            .assign(Place::local(index_local), Rvalue::Use(index_operand));
        let indexed_place = base_place.index(index_local);

        if elem_ty.is_copy() {
            Operand::Copy(indexed_place)
        } else {
            Operand::Move(indexed_place)
        }
    }

    fn lower_enum_variant_literal(
        &mut self,
        span: Span,
        enum_name: &str,
        variant: &str,
        data: Option<&Spanned<Expr>>,
    ) -> Operand {
        let args = data.into_iter().collect::<Vec<_>>();
        self.lower_enum_variant_args(span, enum_name, variant, &args)
    }

    fn lower_enum_variant_args(
        &mut self,
        span: Span,
        enum_name: &str,
        variant: &str,
        args: &[&Spanned<Expr>],
    ) -> Operand {
        let Some((variant_index, fields)) = self.enum_variant_info(enum_name, variant) else {
            return self.semantic_loss(
                span,
                format!(
                    "unknown enum variant `{}::{}` during MIR lowering",
                    enum_name, variant
                ),
            );
        };

        if fields.is_empty() && !args.is_empty() {
            self.strict_error(
                span,
                format!(
                    "unit enum variant `{}::{}` cannot carry payload data",
                    enum_name, variant
                ),
            );
        } else if fields.len() != args.len() {
            self.strict_error(
                span,
                format!(
                    "payload enum variant `{}::{}` expects {} payload field(s), got {}",
                    enum_name,
                    variant,
                    fields.len(),
                    args.len()
                ),
            );
        }

        let operands = fields
            .iter()
            .enumerate()
            .map(|(index, field_ty)| {
                args.get(index)
                    .map(|arg| self.lower_expr_with_expected(arg, Some(field_ty)))
                    .unwrap_or(Operand::Constant(Constant::Unit))
            })
            .collect();
        let result = self.new_temp(MirType::Enum(enum_name.to_string()));
        self.builder.assign(
            Place::local(result),
            Rvalue::Aggregate(
                AggregateKind::Enum(enum_name.to_string(), variant_index),
                operands,
            ),
        );
        Operand::Move(Place::local(result))
    }

    fn enum_variant_info(&self, enum_name: &str, variant: &str) -> Option<(u32, Vec<MirType>)> {
        self.enums
            .get(enum_name)?
            .iter()
            .enumerate()
            .find_map(|(index, (candidate, fields))| {
                (candidate == variant).then(|| (index as u32, fields.clone()))
            })
    }

    fn enum_pattern_discriminant(&self, match_ty: &MirType, pattern: &Pattern) -> Option<u32> {
        let MirType::Enum(enum_name) = match_ty else {
            return None;
        };

        match pattern {
            Pattern::Ident(name) => {
                let (index, fields) = self.enum_variant_info(enum_name, name)?;
                fields.is_empty().then_some(index)
            }
            Pattern::Variant { name, fields } => {
                let (index, variant_fields) = self.enum_variant_info(enum_name, &name.node)?;
                (variant_fields.len() == fields.len()).then_some(index)
            }
            _ => None,
        }
    }

    fn bind_enum_payload_pattern(
        &mut self,
        span: Span,
        match_ty: &MirType,
        pattern: &Pattern,
        match_place: &Place,
    ) {
        let MirType::Enum(enum_name) = match_ty else {
            return;
        };
        let Pattern::Variant { name, fields } = pattern else {
            return;
        };
        let Some((_index, variant_fields)) = self.enum_variant_info(enum_name, &name.node) else {
            return;
        };

        if variant_fields.len() != fields.len() {
            self.strict_error(
                span,
                format!(
                    "payload enum variant `{}::{}` expects {} payload field(s), got {}",
                    enum_name,
                    name.node,
                    variant_fields.len(),
                    fields.len()
                ),
            );
            return;
        }

        for (index, field_pattern) in fields.iter().enumerate() {
            let Pattern::Ident(binding_name) = &field_pattern.node else {
                self.strict_error(
                    field_pattern.span,
                    format!(
                        "unsupported enum payload pattern for strict MIR lowering: {:?}",
                        field_pattern.node
                    ),
                );
                continue;
            };

            let field_ty = variant_fields[index].clone();
            let bound = self.new_temp(field_ty.clone());
            let field_place = match_place.clone().field(index as u32);
            let operand = if field_ty.is_copy() {
                Operand::Copy(field_place)
            } else {
                Operand::Move(field_place)
            };
            self.builder
                .assign(Place::local(bound), Rvalue::Use(operand));
            self.bind_var(binding_name, bound);
        }
    }

    fn lower_struct_literal(
        &mut self,
        span: Span,
        name: &Spanned<String>,
        fields: &[(Spanned<String>, Spanned<Expr>)],
        enum_name: Option<&str>,
    ) -> Operand {
        if let Some(enum_name) = enum_name {
            return self.semantic_loss(
                span,
                format!(
                    "enum struct variant literal `{}.{}` is not strict-MIR-certified",
                    enum_name, name.node
                ),
            );
        }

        let Some(declared_fields) = self.structs.get(&name.node).cloned() else {
            return self.semantic_loss(
                name.span,
                format!("unknown struct `{}` during MIR lowering", name.node),
            );
        };

        let mut provided = HashMap::new();
        for (field_name, field_value) in fields {
            if provided
                .insert(field_name.node.clone(), field_value)
                .is_some()
            {
                self.strict_error(
                    field_name.span,
                    format!(
                        "duplicate field `{}` in struct literal `{}`",
                        field_name.node, name.node
                    ),
                );
            }
        }

        for provided_name in provided.keys() {
            if !declared_fields
                .iter()
                .any(|(declared_name, _)| declared_name == provided_name)
            {
                self.strict_error(
                    name.span,
                    format!(
                        "unknown field `{}` in struct literal `{}`",
                        provided_name, name.node
                    ),
                );
            }
        }

        let mut operands = Vec::with_capacity(declared_fields.len());
        for (field_name, _field_ty) in &declared_fields {
            if let Some(field_value) = provided.get(field_name) {
                operands.push(self.lower_expr(field_value));
            } else {
                self.strict_error(
                    name.span,
                    format!(
                        "missing field `{}` in struct literal `{}`",
                        field_name, name.node
                    ),
                );
                operands.push(Operand::Constant(Constant::Unit));
            }
        }

        let result = self.new_temp(MirType::Struct(name.node.clone()));
        self.builder.assign(
            Place::local(result),
            Rvalue::Aggregate(AggregateKind::Struct(name.node.clone()), operands),
        );
        Operand::Move(Place::local(result))
    }

    fn lower_field_access(
        &mut self,
        span: Span,
        base: &Spanned<Expr>,
        field: &Spanned<String>,
    ) -> Operand {
        let Some((place, field_ty)) = self.lower_field_place(base, field) else {
            return self.semantic_loss(
                span,
                format!(
                    "unsupported field access `{}` for strict MIR lowering",
                    field.node
                ),
            );
        };

        if field_ty.is_copy() {
            Operand::Copy(place)
        } else {
            Operand::Move(place)
        }
    }

    fn lower_place_expr(&self, expr: &Spanned<Expr>) -> Option<(Place, MirType)> {
        match &expr.node {
            Expr::Ident(name) => {
                let local = self.vars.get(name)?;
                let ty = self.local_types.get(local).cloned().unwrap_or(MirType::I64);
                Some((Place::local(*local), ty))
            }
            Expr::Field { expr: base, field } => self.lower_field_place(base, field),
            _ => None,
        }
    }

    fn lower_field_place(
        &self,
        base: &Spanned<Expr>,
        field: &Spanned<String>,
    ) -> Option<(Place, MirType)> {
        let (base_place, base_ty) = self.lower_place_expr(base)?;
        let (field_index, field_ty) = self.struct_field_info(&base_ty, &field.node)?;
        Some((base_place.field(field_index), field_ty))
    }

    fn struct_field_info(&self, base_ty: &MirType, field: &str) -> Option<(u32, MirType)> {
        let MirType::Struct(struct_name) = base_ty else {
            return None;
        };
        let fields = self.structs.get(struct_name)?;
        fields
            .iter()
            .enumerate()
            .find_map(|(index, (name, ty))| (name == field).then(|| (index as u32, ty.clone())))
    }

    fn infer_expr_type(&self, expr: &Spanned<Expr>) -> MirType {
        match &expr.node {
            Expr::Int(_) => MirType::I64,
            Expr::Float(_) => MirType::F64,
            Expr::Bool(_) => MirType::Bool,
            Expr::String(_) => MirType::Str,
            Expr::Unit => MirType::Unit,
            Expr::Ident(name) => self
                .vars
                .get(name)
                .and_then(|local| self.local_types.get(local))
                .cloned()
                .unwrap_or(MirType::I64),
            Expr::StructLit {
                name, enum_name, ..
            } => enum_name
                .as_ref()
                .map(|enum_name| MirType::Enum(enum_name.clone()))
                .unwrap_or_else(|| MirType::Struct(name.node.clone())),
            Expr::Field { expr: base, field } => self
                .infer_field_type(base, &field.node)
                .unwrap_or(MirType::I64),
            Expr::Binary { op, left, right } => self.infer_binary_type(op, left, right),
            Expr::Unary { op, expr } => self.infer_unary_type(op, expr),
            Expr::Assign { .. } => MirType::Unit,
            _ => MirType::I64,
        }
    }

    fn infer_field_type(&self, base: &Spanned<Expr>, field: &str) -> Option<MirType> {
        let base_ty = self.infer_expr_type(base);
        self.struct_field_info(&base_ty, field).map(|(_, ty)| ty)
    }

    fn infer_binary_type(
        &self,
        op: &AstBinOp,
        left: &Spanned<Expr>,
        _right: &Spanned<Expr>,
    ) -> MirType {
        match op {
            AstBinOp::Eq
            | AstBinOp::Neq
            | AstBinOp::Lt
            | AstBinOp::Lte
            | AstBinOp::Gt
            | AstBinOp::Gte
            | AstBinOp::And
            | AstBinOp::Or => MirType::Bool,
            AstBinOp::Add
            | AstBinOp::Sub
            | AstBinOp::Mul
            | AstBinOp::Div
            | AstBinOp::Mod
            | AstBinOp::BitAnd
            | AstBinOp::BitOr
            | AstBinOp::BitXor
            | AstBinOp::Shl
            | AstBinOp::Shr => self.infer_expr_type(left),
        }
    }

    fn infer_unary_type(&self, op: &AstUnOp, expr: &Spanned<Expr>) -> MirType {
        match op {
            AstUnOp::Neg | AstUnOp::BitNot => self.infer_expr_type(expr),
            AstUnOp::Not => MirType::Bool,
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

        let before_stmts: usize = mir.bodies[0]
            .basic_blocks
            .iter()
            .map(|bb| bb.statements.len())
            .sum();

        crate::optimize::optimize_mir_module(&mut mir);

        let after_stmts: usize = mir.bodies[0]
            .basic_blocks
            .iter()
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

    #[test]
    fn test_lower_ternary() {
        let source = "F max(a: i64, b: i64) -> i64 = a > b ? a : b";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        assert_eq!(mir.bodies.len(), 1);
        // Should have entry + then + else + merge blocks
        assert!(mir.bodies[0].basic_blocks.len() >= 4);

        let display = mir.bodies[0].display();
        assert!(display.contains("switchInt"));
    }

    #[test]
    fn test_lower_match_expression() {
        let source = r#"
            F classify(x: i64) -> i64 = M x {
                0 => 100,
                1 => 200,
                _ => 999
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        assert_eq!(mir.bodies.len(), 1);
        // Should have multiple blocks for match arms
        assert!(mir.bodies[0].basic_blocks.len() >= 4);

        let display = mir.bodies[0].display();
        assert!(display.contains("switchInt"));
    }

    #[test]
    fn test_lower_recursive_call() {
        let source = "F factorial(n: i64) -> i64 = I n == 0 { 1 } E { n * @(n - 1) }";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        assert_eq!(mir.bodies.len(), 1);
        let display = mir.bodies[0].display();
        // Tail call should be converted to TailCall terminator
        assert!(display.contains("tailcall") || display.contains("factorial"));
    }

    #[test]
    fn test_lower_unary_operations() {
        let source = "F negate_and_flip(x: i64) -> i64 = { y := -x; !y }";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        assert_eq!(mir.bodies.len(), 1);
        let display = mir.bodies[0].display();
        // Should have UnaryOp for negation and bitwise not
        assert!(display.contains("Neg") || display.contains("Not"));
    }

    #[test]
    fn test_lower_multiple_functions() {
        let source = r#"
            F double(x: i64) -> i64 = x * 2
            F triple(x: i64) -> i64 = x * 3
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        assert_eq!(mir.bodies.len(), 2);
        assert_eq!(mir.bodies[0].name, "double");
        assert_eq!(mir.bodies[1].name, "triple");
    }

    #[test]
    fn test_copy_type_operand() {
        // All i64 types should use Operand::Copy (not Move)
        let source = "F add(x: i64, y: i64) -> i64 = x + y";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        let display = mir.bodies[0].display();
        // Should use "Copy(" for i64 parameters (debug format)
        assert!(display.contains("Copy("));
        assert!(!display.contains("Move("));
    }

    #[test]
    fn test_move_type_operand() {
        // String type uses Copy (str is Copy since Phase 13)
        let source = r#"F take_string(s: str) -> str = s"#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        let display = mir.bodies[0].display();
        // Should use "Copy(" for str parameter (debug format)
        assert!(display.contains("Copy("));
    }

    #[test]
    fn test_drop_insertion() {
        // Copy types (including str) should NOT have Drop statements
        let source = r#"
            F use_string() -> i64 {
                s: str = "hello"
                42
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        let display = mir.bodies[0].display();
        // The string local should NOT be dropped (str is Copy)
        assert!(!display.contains("drop("));
    }

    #[test]
    fn test_no_drop_for_copy_types() {
        // Copy types should NOT have Drop statements
        let source = r#"
            F use_int() -> i64 = {
                x := 42
                y := 100
                x + y
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        let display = mir.bodies[0].display();
        // No drops for i64 locals
        assert!(!display.contains("drop("));
    }

    #[test]
    fn test_move_prevents_drop() {
        // str is Copy, so it uses Copy instead of Move
        let source = r#"F return_string(s: str) -> str = s"#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower_module(&module);

        let display = mir.bodies[0].display();
        // The parameter is copied to return place (str is Copy)
        assert!(display.contains("Copy("));
        // Copy types are not dropped
        assert!(!display.contains("drop("));
    }
}
