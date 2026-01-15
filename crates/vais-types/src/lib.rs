//! Vais 2.0 Type System
//!
//! Static type checking with inference for AI-optimized code generation.

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    Mismatch { expected: String, found: String },

    #[error("Undefined variable: {0}")]
    UndefinedVar(String),

    #[error("Undefined type: {0}")]
    UndefinedType(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Cannot call non-function type: {0}")]
    NotCallable(String),

    #[error("Wrong number of arguments: expected {expected}, got {got}")]
    ArgCount { expected: usize, got: usize },

    #[error("Cannot infer type")]
    CannotInfer,

    #[error("Duplicate definition: {0}")]
    Duplicate(String),

    #[error("Cannot assign to immutable variable: {0}")]
    ImmutableAssign(String),
}

type TypeResult<T> = Result<T, TypeError>;

/// Resolved type in the type system
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedType {
    // Primitives
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Str,
    Unit,

    // Compound types
    Array(Box<ResolvedType>),
    Map(Box<ResolvedType>, Box<ResolvedType>),
    Tuple(Vec<ResolvedType>),
    Optional(Box<ResolvedType>),
    Result(Box<ResolvedType>),
    Pointer(Box<ResolvedType>),
    Ref(Box<ResolvedType>),
    RefMut(Box<ResolvedType>),

    // Function type
    Fn {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
    },

    // Named type (struct/enum)
    Named {
        name: String,
        generics: Vec<ResolvedType>,
    },

    // Type variable for inference
    Var(usize),

    // Unknown/Error type
    Unknown,
}

impl ResolvedType {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
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
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
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
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, ResolvedType::F32 | ResolvedType::F64)
    }
}

impl std::fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::I8 => write!(f, "i8"),
            ResolvedType::I16 => write!(f, "i16"),
            ResolvedType::I32 => write!(f, "i32"),
            ResolvedType::I64 => write!(f, "i64"),
            ResolvedType::I128 => write!(f, "i128"),
            ResolvedType::U8 => write!(f, "u8"),
            ResolvedType::U16 => write!(f, "u16"),
            ResolvedType::U32 => write!(f, "u32"),
            ResolvedType::U64 => write!(f, "u64"),
            ResolvedType::U128 => write!(f, "u128"),
            ResolvedType::F32 => write!(f, "f32"),
            ResolvedType::F64 => write!(f, "f64"),
            ResolvedType::Bool => write!(f, "bool"),
            ResolvedType::Str => write!(f, "str"),
            ResolvedType::Unit => write!(f, "()"),
            ResolvedType::Array(t) => write!(f, "[{}]", t),
            ResolvedType::Map(k, v) => write!(f, "[{}:{}]", k, v),
            ResolvedType::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            ResolvedType::Optional(t) => write!(f, "{}?", t),
            ResolvedType::Result(t) => write!(f, "{}!", t),
            ResolvedType::Pointer(t) => write!(f, "*{}", t),
            ResolvedType::Ref(t) => write!(f, "&{}", t),
            ResolvedType::RefMut(t) => write!(f, "&mut {}", t),
            ResolvedType::Fn { params, ret } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")->{}", ret)
            }
            ResolvedType::Named { name, generics } => {
                write!(f, "{}", name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Var(id) => write!(f, "?{}", id),
            ResolvedType::Unknown => write!(f, "?"),
        }
    }
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<(String, ResolvedType, bool)>, // (name, type, is_mut)
    pub ret: ResolvedType,
    pub is_async: bool,
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
    pub methods: HashMap<String, FunctionSig>,
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: HashMap<String, Vec<ResolvedType>>,
}

/// Variable info
#[derive(Debug, Clone)]
struct VarInfo {
    ty: ResolvedType,
    is_mut: bool,
}

/// Type checker
pub struct TypeChecker {
    // Type environment
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
    functions: HashMap<String, FunctionSig>,
    type_aliases: HashMap<String, ResolvedType>,

    // Scope stack
    scopes: Vec<HashMap<String, VarInfo>>,

    // Current function context
    current_fn_ret: Option<ResolvedType>,
    current_fn_name: Option<String>,

    // Type variable counter for inference
    next_type_var: usize,

    // Type substitutions
    substitutions: HashMap<usize, ResolvedType>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            type_aliases: HashMap::new(),
            scopes: vec![HashMap::new()],
            current_fn_ret: None,
            current_fn_name: None,
            next_type_var: 0,
            substitutions: HashMap::new(),
        }
    }

    /// Type check a module
    pub fn check_module(&mut self, module: &Module) -> TypeResult<()> {
        // First pass: collect all type definitions
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => self.register_struct(s)?,
                Item::Enum(e) => self.register_enum(e)?,
                Item::TypeAlias(t) => self.register_type_alias(t)?,
                Item::Use(_) => {} // TODO: Handle imports
            }
        }

        // Second pass: check function bodies
        for item in &module.items {
            if let Item::Function(f) = &item.node {
                self.check_function(f)?;
            }
        }

        Ok(())
    }

    /// Register a function signature
    fn register_function(&mut self, f: &Function) -> TypeResult<()> {
        let name = f.name.node.clone();
        if self.functions.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.resolve_type(&p.ty.node);
                (p.name.node.clone(), ty, p.is_mut)
            })
            .collect();

        let ret = f
            .ret_type
            .as_ref()
            .map(|t| self.resolve_type(&t.node))
            .unwrap_or(ResolvedType::Unit);

        self.functions.insert(
            name.clone(),
            FunctionSig {
                name,
                generics: f.generics.iter().map(|g| g.node.clone()).collect(),
                params,
                ret,
                is_async: f.is_async,
            },
        );

        Ok(())
    }

    /// Register a struct
    fn register_struct(&mut self, s: &Struct) -> TypeResult<()> {
        let name = s.name.node.clone();
        if self.structs.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        let mut fields = HashMap::new();
        for field in &s.fields {
            fields.insert(field.name.node.clone(), self.resolve_type(&field.ty.node));
        }

        let mut methods = HashMap::new();
        for method in &s.methods {
            let params: Vec<_> = method
                .node
                .params
                .iter()
                .map(|p| {
                    let ty = self.resolve_type(&p.ty.node);
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = method
                .node
                .ret_type
                .as_ref()
                .map(|t| self.resolve_type(&t.node))
                .unwrap_or(ResolvedType::Unit);

            methods.insert(
                method.node.name.node.clone(),
                FunctionSig {
                    name: method.node.name.node.clone(),
                    generics: method.node.generics.iter().map(|g| g.node.clone()).collect(),
                    params,
                    ret,
                    is_async: method.node.is_async,
                },
            );
        }

        self.structs.insert(
            name.clone(),
            StructDef {
                name,
                generics: s.generics.iter().map(|g| g.node.clone()).collect(),
                fields,
                methods,
            },
        );

        Ok(())
    }

    /// Register an enum
    fn register_enum(&mut self, e: &Enum) -> TypeResult<()> {
        let name = e.name.node.clone();
        if self.enums.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        let mut variants = HashMap::new();
        for variant in &e.variants {
            let types = match &variant.fields {
                VariantFields::Unit => vec![],
                VariantFields::Tuple(ts) => ts.iter().map(|t| self.resolve_type(&t.node)).collect(),
                VariantFields::Struct(_) => vec![], // TODO: Handle struct variants
            };
            variants.insert(variant.name.node.clone(), types);
        }

        self.enums.insert(
            name.clone(),
            EnumDef {
                name,
                generics: e.generics.iter().map(|g| g.node.clone()).collect(),
                variants,
            },
        );

        Ok(())
    }

    /// Register a type alias
    fn register_type_alias(&mut self, t: &TypeAlias) -> TypeResult<()> {
        let name = t.name.node.clone();
        if self.type_aliases.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        let resolved = self.resolve_type(&t.ty.node);
        self.type_aliases.insert(name, resolved);

        Ok(())
    }

    /// Check a function body
    fn check_function(&mut self, f: &Function) -> TypeResult<()> {
        self.push_scope();

        // Add parameters to scope
        for param in &f.params {
            let ty = self.resolve_type(&param.ty.node);
            self.define_var(&param.name.node, ty, param.is_mut);
        }

        // Set current function context
        self.current_fn_ret = Some(
            f.ret_type
                .as_ref()
                .map(|t| self.resolve_type(&t.node))
                .unwrap_or(ResolvedType::Unit),
        );
        self.current_fn_name = Some(f.name.node.clone());

        // Check body
        let body_type = match &f.body {
            FunctionBody::Expr(expr) => self.check_expr(expr)?,
            FunctionBody::Block(stmts) => self.check_block(stmts)?,
        };

        // Check return type
        let expected_ret = self.current_fn_ret.clone().unwrap();
        self.unify(&expected_ret, &body_type)?;

        self.current_fn_ret = None;
        self.current_fn_name = None;
        self.pop_scope();

        Ok(())
    }

    /// Check a block of statements
    fn check_block(&mut self, stmts: &[Spanned<Stmt>]) -> TypeResult<ResolvedType> {
        let mut last_type = ResolvedType::Unit;

        for stmt in stmts {
            last_type = self.check_stmt(stmt)?;
        }

        Ok(last_type)
    }

    /// Check a statement
    fn check_stmt(&mut self, stmt: &Spanned<Stmt>) -> TypeResult<ResolvedType> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
            } => {
                let value_type = self.check_expr(value)?;
                let var_type = if let Some(ty) = ty {
                    let expected = self.resolve_type(&ty.node);
                    self.unify(&expected, &value_type)?;
                    expected
                } else {
                    value_type
                };
                self.define_var(&name.node, var_type, *is_mut);
                Ok(ResolvedType::Unit)
            }
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Return(expr) => {
                let ret_type = if let Some(expr) = expr {
                    self.check_expr(expr)?
                } else {
                    ResolvedType::Unit
                };
                if let Some(expected) = self.current_fn_ret.clone() {
                    self.unify(&expected, &ret_type)?;
                }
                Ok(ResolvedType::Unit)
            }
            Stmt::Break(_) | Stmt::Continue => Ok(ResolvedType::Unit),
        }
    }

    /// Check an expression
    fn check_expr(&mut self, expr: &Spanned<Expr>) -> TypeResult<ResolvedType> {
        match &expr.node {
            Expr::Int(_) => Ok(ResolvedType::I64),
            Expr::Float(_) => Ok(ResolvedType::F64),
            Expr::Bool(_) => Ok(ResolvedType::Bool),
            Expr::String(_) => Ok(ResolvedType::Str),
            Expr::Unit => Ok(ResolvedType::Unit),

            Expr::Ident(name) => self.lookup_var(name),

            Expr::SelfCall => {
                // @ refers to current function
                if let Some(name) = &self.current_fn_name {
                    if let Some(sig) = self.functions.get(name) {
                        return Ok(ResolvedType::Fn {
                            params: sig.params.iter().map(|(_, t, _)| t.clone()).collect(),
                            ret: Box::new(sig.ret.clone()),
                        });
                    }
                }
                Err(TypeError::UndefinedFunction("@".to_string()))
            }

            Expr::Binary { op, left, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if !left_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(left_type)
                    }
                    BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte => {
                        if !left_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::Eq | BinOp::Neq => {
                        self.unify(&left_type, &right_type)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::And | BinOp::Or => {
                        self.unify(&left_type, &ResolvedType::Bool)?;
                        self.unify(&right_type, &ResolvedType::Bool)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                        if !left_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: left_type.to_string(),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(left_type)
                    }
                }
            }

            Expr::Unary { op, expr: inner } => {
                let inner_type = self.check_expr(inner)?;
                match op {
                    UnaryOp::Neg => {
                        if !inner_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: inner_type.to_string(),
                            });
                        }
                        Ok(inner_type)
                    }
                    UnaryOp::Not => {
                        self.unify(&inner_type, &ResolvedType::Bool)?;
                        Ok(ResolvedType::Bool)
                    }
                    UnaryOp::BitNot => {
                        if !inner_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: inner_type.to_string(),
                            });
                        }
                        Ok(inner_type)
                    }
                }
            }

            Expr::Ternary { cond, then, else_ } => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                let then_type = self.check_expr(then)?;
                let else_type = self.check_expr(else_)?;
                self.unify(&then_type, &else_type)?;

                Ok(then_type)
            }

            Expr::If { cond, then, else_ } => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                self.push_scope();
                let then_type = self.check_block(then)?;
                self.pop_scope();

                if let Some(else_branch) = else_ {
                    let else_type = self.check_if_else(else_branch)?;
                    self.unify(&then_type, &else_type)?;
                    Ok(then_type)
                } else {
                    Ok(ResolvedType::Unit)
                }
            }

            Expr::Loop { pattern, iter, body } => {
                self.push_scope();

                if let (Some(pattern), Some(iter)) = (pattern, iter) {
                    let iter_type = self.check_expr(iter)?;
                    // TODO: Proper iterator type inference
                    if let ResolvedType::Array(elem_type) = iter_type {
                        if let Pattern::Ident(name) = &pattern.node {
                            self.define_var(name, *elem_type, false);
                        }
                    }
                }

                self.check_block(body)?;
                self.pop_scope();

                Ok(ResolvedType::Unit)
            }

            Expr::Match { expr, arms } => {
                let expr_type = self.check_expr(expr)?;
                let mut result_type: Option<ResolvedType> = None;

                for arm in arms {
                    // TODO: Proper pattern type checking
                    self.push_scope();
                    let arm_type = self.check_expr(&arm.body)?;
                    self.pop_scope();

                    if let Some(ref prev) = result_type {
                        self.unify(prev, &arm_type)?;
                    } else {
                        result_type = Some(arm_type);
                    }
                }

                Ok(result_type.unwrap_or(ResolvedType::Unit))
            }

            Expr::Call { func, args } => {
                let func_type = self.check_expr(func)?;

                match func_type {
                    ResolvedType::Fn { params, ret } => {
                        if params.len() != args.len() {
                            return Err(TypeError::ArgCount {
                                expected: params.len(),
                                got: args.len(),
                            });
                        }

                        for (param_type, arg) in params.iter().zip(args) {
                            let arg_type = self.check_expr(arg)?;
                            self.unify(param_type, &arg_type)?;
                        }

                        Ok(*ret)
                    }
                    _ => Err(TypeError::NotCallable(func_type.to_string())),
                }
            }

            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let receiver_type = self.check_expr(receiver)?;

                if let ResolvedType::Named { name, .. } = &receiver_type {
                    if let Some(struct_def) = self.structs.get(name).cloned() {
                        if let Some(method_sig) = struct_def.methods.get(&method.node).cloned() {
                            // Skip self parameter
                            let param_types: Vec<_> =
                                method_sig.params.iter().skip(1).map(|(_, t, _)| t.clone()).collect();

                            if param_types.len() != args.len() {
                                return Err(TypeError::ArgCount {
                                    expected: param_types.len(),
                                    got: args.len(),
                                });
                            }

                            for (param_type, arg) in param_types.iter().zip(args) {
                                let arg_type = self.check_expr(arg)?;
                                self.unify(&param_type, &arg_type)?;
                            }

                            return Ok(method_sig.ret.clone());
                        }
                    }
                }

                Err(TypeError::UndefinedFunction(method.node.clone()))
            }

            Expr::Field { expr: inner, field } => {
                let inner_type = self.check_expr(inner)?;

                if let ResolvedType::Named { name, .. } = &inner_type {
                    if let Some(struct_def) = self.structs.get(name) {
                        if let Some(field_type) = struct_def.fields.get(&field.node) {
                            return Ok(field_type.clone());
                        }
                    }
                }

                Err(TypeError::UndefinedVar(field.node.clone()))
            }

            Expr::Index { expr: inner, index } => {
                let inner_type = self.check_expr(inner)?;
                let index_type = self.check_expr(index)?;

                match inner_type {
                    ResolvedType::Array(elem_type) => {
                        if !index_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                            });
                        }
                        Ok(*elem_type)
                    }
                    ResolvedType::Map(key_type, value_type) => {
                        self.unify(&key_type, &index_type)?;
                        Ok(*value_type)
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "indexable type".to_string(),
                        found: inner_type.to_string(),
                    }),
                }
            }

            Expr::Array(exprs) => {
                if exprs.is_empty() {
                    let var = self.fresh_type_var();
                    return Ok(ResolvedType::Array(Box::new(var)));
                }

                let first_type = self.check_expr(&exprs[0])?;
                for expr in &exprs[1..] {
                    let t = self.check_expr(expr)?;
                    self.unify(&first_type, &t)?;
                }

                Ok(ResolvedType::Array(Box::new(first_type)))
            }

            Expr::Tuple(exprs) => {
                let types: Result<Vec<_>, _> = exprs.iter().map(|e| self.check_expr(e)).collect();
                Ok(ResolvedType::Tuple(types?))
            }

            Expr::StructLit { name, fields } => {
                if let Some(struct_def) = self.structs.get(&name.node).cloned() {
                    for (field_name, value) in fields {
                        let value_type = self.check_expr(value)?;
                        if let Some(expected_type) = struct_def.fields.get(&field_name.node).cloned() {
                            self.unify(&expected_type, &value_type)?;
                        } else {
                            return Err(TypeError::UndefinedVar(field_name.node.clone()));
                        }
                    }
                    Ok(ResolvedType::Named {
                        name: name.node.clone(),
                        generics: vec![],
                    })
                } else {
                    Err(TypeError::UndefinedType(name.node.clone()))
                }
            }

            Expr::Range { .. } => {
                // TODO: Implement range type
                Ok(ResolvedType::Named {
                    name: "Range".to_string(),
                    generics: vec![ResolvedType::I64],
                })
            }

            Expr::Block(stmts) => {
                self.push_scope();
                let result = self.check_block(stmts);
                self.pop_scope();
                result
            }

            Expr::Await(inner) => {
                // TODO: Proper async type checking
                self.check_expr(inner)
            }

            Expr::Try(inner) => {
                let inner_type = self.check_expr(inner)?;
                if let ResolvedType::Result(ok_type) = inner_type {
                    Ok(*ok_type)
                } else {
                    Err(TypeError::Mismatch {
                        expected: "Result type".to_string(),
                        found: inner_type.to_string(),
                    })
                }
            }

            Expr::Unwrap(inner) => {
                let inner_type = self.check_expr(inner)?;
                match inner_type {
                    ResolvedType::Optional(inner) | ResolvedType::Result(inner) => Ok(*inner),
                    _ => Err(TypeError::Mismatch {
                        expected: "Optional or Result".to_string(),
                        found: inner_type.to_string(),
                    }),
                }
            }

            Expr::Ref(inner) => {
                let inner_type = self.check_expr(inner)?;
                Ok(ResolvedType::Ref(Box::new(inner_type)))
            }

            Expr::Deref(inner) => {
                let inner_type = self.check_expr(inner)?;
                match inner_type {
                    ResolvedType::Ref(t) | ResolvedType::RefMut(t) | ResolvedType::Pointer(t) => {
                        Ok(*t)
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "reference or pointer".to_string(),
                        found: inner_type.to_string(),
                    }),
                }
            }

            Expr::Assign { target, value } => {
                // Check target is mutable
                if let Expr::Ident(name) = &target.node {
                    let var_info = self.lookup_var_info(name)?;
                    if !var_info.is_mut {
                        return Err(TypeError::ImmutableAssign(name.clone()));
                    }
                }

                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;
                self.unify(&target_type, &value_type)?;
                Ok(ResolvedType::Unit)
            }

            Expr::AssignOp { op, target, value } => {
                // Similar to assign
                if let Expr::Ident(name) = &target.node {
                    let var_info = self.lookup_var_info(name)?;
                    if !var_info.is_mut {
                        return Err(TypeError::ImmutableAssign(name.clone()));
                    }
                }

                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;
                self.unify(&target_type, &value_type)?;
                Ok(ResolvedType::Unit)
            }

            Expr::Lambda { params, body } => {
                self.push_scope();

                let param_types: Vec<_> = params
                    .iter()
                    .map(|p| {
                        let ty = self.resolve_type(&p.ty.node);
                        self.define_var(&p.name.node, ty.clone(), p.is_mut);
                        ty
                    })
                    .collect();

                let ret_type = self.check_expr(body)?;
                self.pop_scope();

                Ok(ResolvedType::Fn {
                    params: param_types,
                    ret: Box::new(ret_type),
                })
            }

            Expr::Spawn(inner) => {
                let inner_type = self.check_expr(inner)?;
                // Return a future/task type
                Ok(ResolvedType::Named {
                    name: "Task".to_string(),
                    generics: vec![inner_type],
                })
            }
        }
    }

    /// Check if-else branch
    fn check_if_else(&mut self, branch: &IfElse) -> TypeResult<ResolvedType> {
        match branch {
            IfElse::ElseIf(cond, then, else_) => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                self.push_scope();
                let then_type = self.check_block(then)?;
                self.pop_scope();

                if let Some(else_branch) = else_ {
                    let else_type = self.check_if_else(else_branch)?;
                    self.unify(&then_type, &else_type)?;
                }

                Ok(then_type)
            }
            IfElse::Else(stmts) => {
                self.push_scope();
                let result = self.check_block(stmts);
                self.pop_scope();
                result
            }
        }
    }

    /// Resolve AST type to internal type
    fn resolve_type(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, generics } => {
                let resolved_generics: Vec<_> =
                    generics.iter().map(|g| self.resolve_type(&g.node)).collect();

                match name.as_str() {
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
                        if let Some(alias) = self.type_aliases.get(name) {
                            alias.clone()
                        } else {
                            ResolvedType::Named {
                                name: name.clone(),
                                generics: resolved_generics,
                            }
                        }
                    }
                }
            }
            Type::Array(inner) => ResolvedType::Array(Box::new(self.resolve_type(&inner.node))),
            Type::Map(key, value) => ResolvedType::Map(
                Box::new(self.resolve_type(&key.node)),
                Box::new(self.resolve_type(&value.node)),
            ),
            Type::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.resolve_type(&t.node)).collect())
            }
            Type::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.resolve_type(&inner.node)))
            }
            Type::Result(inner) => ResolvedType::Result(Box::new(self.resolve_type(&inner.node))),
            Type::Pointer(inner) => ResolvedType::Pointer(Box::new(self.resolve_type(&inner.node))),
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.resolve_type(&inner.node))),
            Type::RefMut(inner) => ResolvedType::RefMut(Box::new(self.resolve_type(&inner.node))),
            Type::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.resolve_type(&p.node)).collect(),
                ret: Box::new(self.resolve_type(&ret.node)),
            },
            Type::Unit => ResolvedType::Unit,
            Type::Infer => self.fresh_type_var(),
        }
    }

    /// Unify two types
    fn unify(&mut self, expected: &ResolvedType, found: &ResolvedType) -> TypeResult<()> {
        let expected = self.apply_substitutions(expected);
        let found = self.apply_substitutions(found);

        if expected == found {
            return Ok(());
        }

        match (&expected, &found) {
            (ResolvedType::Var(id), t) | (t, ResolvedType::Var(id)) => {
                self.substitutions.insert(*id, t.clone());
                Ok(())
            }
            (ResolvedType::Array(a), ResolvedType::Array(b)) => self.unify(a, b),
            (ResolvedType::Optional(a), ResolvedType::Optional(b)) => self.unify(a, b),
            (ResolvedType::Result(a), ResolvedType::Result(b)) => self.unify(a, b),
            (ResolvedType::Ref(a), ResolvedType::Ref(b)) => self.unify(a, b),
            (ResolvedType::RefMut(a), ResolvedType::RefMut(b)) => self.unify(a, b),
            (ResolvedType::Pointer(a), ResolvedType::Pointer(b)) => self.unify(a, b),
            (ResolvedType::Tuple(a), ResolvedType::Tuple(b)) if a.len() == b.len() => {
                for (ta, tb) in a.iter().zip(b.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            (
                ResolvedType::Fn {
                    params: pa,
                    ret: ra,
                },
                ResolvedType::Fn {
                    params: pb,
                    ret: rb,
                },
            ) if pa.len() == pb.len() => {
                for (ta, tb) in pa.iter().zip(pb.iter()) {
                    self.unify(ta, tb)?;
                }
                self.unify(ra, rb)
            }
            _ => Err(TypeError::Mismatch {
                expected: expected.to_string(),
                found: found.to_string(),
            }),
        }
    }

    /// Apply substitutions to a type
    fn apply_substitutions(&self, ty: &ResolvedType) -> ResolvedType {
        match ty {
            ResolvedType::Var(id) => {
                if let Some(subst) = self.substitutions.get(id) {
                    self.apply_substitutions(subst)
                } else {
                    ty.clone()
                }
            }
            ResolvedType::Array(inner) => {
                ResolvedType::Array(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Result(inner) => {
                ResolvedType::Result(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.apply_substitutions(t)).collect())
            }
            ResolvedType::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                ret: Box::new(self.apply_substitutions(ret)),
            },
            _ => ty.clone(),
        }
    }

    /// Create a fresh type variable
    fn fresh_type_var(&self) -> ResolvedType {
        // Note: This should be mutable, but for simplicity we'll use a workaround
        ResolvedType::Var(0)
    }

    // === Scope management ===

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_var(&mut self, name: &str, ty: ResolvedType, is_mut: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), VarInfo { ty, is_mut });
        }
    }

    fn lookup_var(&self, name: &str) -> TypeResult<ResolvedType> {
        self.lookup_var_info(name).map(|v| v.ty)
    }

    fn lookup_var_info(&self, name: &str) -> TypeResult<VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Ok(info.clone());
            }
        }

        // Check if it's a function
        if let Some(sig) = self.functions.get(name) {
            return Ok(VarInfo {
                ty: ResolvedType::Fn {
                    params: sig.params.iter().map(|(_, t, _)| t.clone()).collect(),
                    ret: Box::new(sig.ret.clone()),
                },
                is_mut: false,
            });
        }

        Err(TypeError::UndefinedVar(name.to_string()))
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
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
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let source = "F add(a:i64,b:str)->i64=a+b";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_struct() {
        let source = r#"
            S Point{x:f64,y:f64}
            F make_point()->Point=Point{x:1.0,y:2.0}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }
}
