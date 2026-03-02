//! Lightweight type resolution for LSP features
//!
//! Resolves expression types from AST without full type checking.
//! Used for type-aware completion, hover, and inlay hints.

use std::collections::HashMap;
use vais_ast::{Expr, FunctionBody, IfElse, Item, Module, Spanned, Stmt, Type};

/// Resolved type information for LSP purposes
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LspType {
    /// Named type (struct, enum, trait, type alias)
    Named(String),
    /// Primitive type
    Primitive(String),
    /// Array type
    Array(Box<LspType>),
    /// Tuple type
    Tuple(Vec<LspType>),
    /// Optional type
    Optional(Box<LspType>),
    /// Result type
    Result(Box<LspType>, Box<LspType>),
    /// Function type
    Function {
        params: Vec<LspType>,
        ret: Box<LspType>,
    },
    /// Range type
    Range,
    /// Unit type
    Unit,
    /// Unknown (could not resolve)
    Unknown,
}

impl LspType {
    pub(crate) fn display_name(&self) -> String {
        match self {
            LspType::Named(name) => name.clone(),
            LspType::Primitive(name) => name.clone(),
            LspType::Array(inner) => format!("[{}]", inner.display_name()),
            LspType::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(|t| t.display_name()).collect();
                format!("({})", inner.join(", "))
            }
            LspType::Optional(inner) => format!("Option<{}>", inner.display_name()),
            LspType::Result(ok, err) => {
                format!("Result<{}, {}>", ok.display_name(), err.display_name())
            }
            LspType::Function { params, ret } => {
                let params_str: Vec<String> = params.iter().map(|p| p.display_name()).collect();
                format!("fn({}) -> {}", params_str.join(", "), ret.display_name())
            }
            LspType::Range => "Range".to_string(),
            LspType::Unit => "()".to_string(),
            LspType::Unknown => "_".to_string(),
        }
    }
}

/// Struct field information
#[derive(Debug, Clone)]
pub(crate) struct FieldInfo {
    pub(crate) name: String,
    pub(crate) ty: LspType,
    pub(crate) type_display: String,
}

/// Method information
#[derive(Debug, Clone)]
pub(crate) struct MethodInfo {
    pub(crate) name: String,
    pub(crate) params: Vec<(String, String)>, // (name, type_display)
    pub(crate) ret_type: Option<String>,
    pub(crate) from_trait: Option<String>,
}

/// Lightweight type context built from AST
pub(crate) struct TypeContext {
    /// Struct name -> fields
    pub(crate) structs: HashMap<String, Vec<FieldInfo>>,
    /// Type name -> methods (from impl blocks)
    pub(crate) type_methods: HashMap<String, Vec<MethodInfo>>,
    /// Trait name -> methods
    pub(crate) trait_methods: HashMap<String, Vec<MethodInfo>>,
    /// Type name -> list of trait names it implements
    pub(crate) type_traits: HashMap<String, Vec<String>>,
    /// Enum name -> variant names
    pub(crate) enum_variants: HashMap<String, Vec<String>>,
    /// Function name -> return type
    pub(crate) function_returns: HashMap<String, LspType>,
    /// Variable name -> type (scope-local, from let bindings)
    pub(crate) variable_types: HashMap<String, LspType>,
}

impl TypeContext {
    /// Build type context from an AST module
    pub(crate) fn from_module(ast: &Module) -> Self {
        let mut ctx = Self {
            structs: HashMap::new(),
            type_methods: HashMap::new(),
            trait_methods: HashMap::new(),
            type_traits: HashMap::new(),
            enum_variants: HashMap::new(),
            function_returns: HashMap::new(),
            variable_types: HashMap::new(),
        };

        for item in &ast.items {
            match &item.node {
                Item::Struct(s) => {
                    let fields: Vec<FieldInfo> = s
                        .fields
                        .iter()
                        .map(|f| FieldInfo {
                            name: f.name.node.clone(),
                            ty: ast_type_to_lsp(&f.ty.node),
                            type_display: format_type(&f.ty.node),
                        })
                        .collect();
                    ctx.structs.insert(s.name.node.clone(), fields);
                }
                Item::Enum(e) => {
                    let variants: Vec<String> =
                        e.variants.iter().map(|v| v.name.node.clone()).collect();
                    ctx.enum_variants.insert(e.name.node.clone(), variants);
                }
                Item::Function(f) => {
                    let ret = f
                        .ret_type
                        .as_ref()
                        .map(|rt| ast_type_to_lsp(&rt.node))
                        .unwrap_or(LspType::Unit);
                    ctx.function_returns.insert(f.name.node.clone(), ret);
                }
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };

                    let trait_name = impl_block.trait_name.as_ref().map(|t| t.node.clone());

                    // Track trait implementations
                    if let Some(ref tn) = trait_name {
                        ctx.type_traits
                            .entry(type_name.clone())
                            .or_default()
                            .push(tn.clone());
                    }

                    let methods: Vec<MethodInfo> = impl_block
                        .methods
                        .iter()
                        .map(|m| {
                            let params: Vec<(String, String)> = m
                                .node
                                .params
                                .iter()
                                .filter(|p| p.name.node != "self")
                                .map(|p| (p.name.node.clone(), format_type(&p.ty.node)))
                                .collect();
                            let ret_type = m.node.ret_type.as_ref().map(|rt| format_type(&rt.node));
                            MethodInfo {
                                name: m.node.name.node.clone(),
                                params,
                                ret_type,
                                from_trait: trait_name.clone(),
                            }
                        })
                        .collect();

                    // Also register return types for methods
                    for method in &impl_block.methods {
                        let ret = method
                            .node
                            .ret_type
                            .as_ref()
                            .map(|rt| ast_type_to_lsp(&rt.node))
                            .unwrap_or(LspType::Unit);
                        ctx.function_returns
                            .insert(method.node.name.node.clone(), ret);
                    }

                    ctx.type_methods
                        .entry(type_name)
                        .or_default()
                        .extend(methods);
                }
                Item::Trait(t) => {
                    let methods: Vec<MethodInfo> = t
                        .methods
                        .iter()
                        .map(|m| {
                            let params: Vec<(String, String)> = m
                                .params
                                .iter()
                                .filter(|p| p.name.node != "self")
                                .map(|p| (p.name.node.clone(), format_type(&p.ty.node)))
                                .collect();
                            let ret_type = m.ret_type.as_ref().map(|rt| format_type(&rt.node));
                            MethodInfo {
                                name: m.name.node.clone(),
                                params,
                                ret_type,
                                from_trait: Some(t.name.node.clone()),
                            }
                        })
                        .collect();
                    ctx.trait_methods.insert(t.name.node.clone(), methods);
                }
                _ => {}
            }
        }

        ctx
    }

    /// Collect variable bindings from a function body up to a given offset
    pub(crate) fn collect_variable_bindings(&mut self, ast: &Module, cursor_offset: usize) {
        for item in &ast.items {
            match &item.node {
                Item::Function(f) => {
                    // Check if cursor is inside this function
                    if item.span.start <= cursor_offset && cursor_offset <= item.span.end {
                        // Add function parameters
                        for param in &f.params {
                            if param.name.node != "self" {
                                let ty = ast_type_to_lsp(&param.ty.node);
                                self.variable_types.insert(param.name.node.clone(), ty);
                            }
                        }
                        // Collect from body
                        match &f.body {
                            FunctionBody::Block(stmts) => {
                                self.collect_bindings_from_stmts(stmts, cursor_offset);
                            }
                            FunctionBody::Expr(expr) => {
                                self.collect_bindings_from_expr(expr, cursor_offset);
                            }
                        }
                    }
                }
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        let method_span = method.span;
                        if method_span.start <= cursor_offset && cursor_offset <= method_span.end {
                            // Add 'self' as the impl target type
                            if let Type::Named { name, .. } = &impl_block.target_type.node {
                                self.variable_types
                                    .insert("self".to_string(), LspType::Named(name.clone()));
                            }
                            // Add method parameters
                            for param in &method.node.params {
                                if param.name.node != "self" {
                                    let ty = ast_type_to_lsp(&param.ty.node);
                                    self.variable_types.insert(param.name.node.clone(), ty);
                                }
                            }
                            match &method.node.body {
                                FunctionBody::Block(stmts) => {
                                    self.collect_bindings_from_stmts(stmts, cursor_offset);
                                }
                                FunctionBody::Expr(expr) => {
                                    self.collect_bindings_from_expr(expr, cursor_offset);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_bindings_from_stmts(&mut self, stmts: &[Spanned<Stmt>], cursor_offset: usize) {
        for stmt in stmts {
            // Only collect bindings that appear before the cursor
            if stmt.span.start > cursor_offset {
                break;
            }
            match &stmt.node {
                Stmt::Let {
                    name, ty, value, ..
                } => {
                    let resolved_ty = if let Some(ty_ann) = ty {
                        if !matches!(ty_ann.node, Type::Infer) {
                            ast_type_to_lsp(&ty_ann.node)
                        } else {
                            self.infer_expr_type(value)
                        }
                    } else {
                        self.infer_expr_type(value)
                    };
                    self.variable_types.insert(name.node.clone(), resolved_ty);
                }
                Stmt::Expr(expr) => {
                    self.collect_bindings_from_expr(expr, cursor_offset);
                }
                _ => {}
            }
        }
    }

    fn collect_bindings_from_expr(&mut self, expr: &Spanned<Expr>, cursor_offset: usize) {
        match &expr.node {
            Expr::Block(stmts) => {
                self.collect_bindings_from_stmts(stmts, cursor_offset);
            }
            Expr::If { then, else_, .. } => {
                self.collect_bindings_from_stmts(then, cursor_offset);
                if let Some(else_branch) = else_ {
                    match else_branch {
                        IfElse::ElseIf(_, stmts, _) => {
                            self.collect_bindings_from_stmts(stmts, cursor_offset);
                        }
                        IfElse::Else(stmts) => {
                            self.collect_bindings_from_stmts(stmts, cursor_offset);
                        }
                    }
                }
            }
            Expr::Loop { body, .. } => {
                self.collect_bindings_from_stmts(body, cursor_offset);
            }
            Expr::While { body, .. } => {
                self.collect_bindings_from_stmts(body, cursor_offset);
            }
            _ => {}
        }
    }

    /// Infer expression type from AST
    pub(crate) fn infer_expr_type(&self, expr: &Spanned<Expr>) -> LspType {
        match &expr.node {
            Expr::Int(_) => LspType::Primitive("i64".to_string()),
            Expr::Float(_) => LspType::Primitive("f64".to_string()),
            Expr::Bool(_) => LspType::Primitive("bool".to_string()),
            Expr::String(_) => LspType::Primitive("str".to_string()),
            Expr::Ident(name) => {
                // Look up variable type
                if let Some(ty) = self.variable_types.get(name) {
                    return ty.clone();
                }
                LspType::Unknown
            }
            Expr::Array(elems) => {
                if let Some(first) = elems.first() {
                    LspType::Array(Box::new(self.infer_expr_type(first)))
                } else {
                    LspType::Array(Box::new(LspType::Unknown))
                }
            }
            Expr::Tuple(elems) => {
                let types: Vec<LspType> = elems.iter().map(|e| self.infer_expr_type(e)).collect();
                LspType::Tuple(types)
            }
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    // Check for constructors
                    match name.as_str() {
                        "Some" => return LspType::Optional(Box::new(LspType::Unknown)),
                        "Ok" => {
                            return LspType::Result(
                                Box::new(LspType::Unknown),
                                Box::new(LspType::Unknown),
                            )
                        }
                        "Err" => {
                            return LspType::Result(
                                Box::new(LspType::Unknown),
                                Box::new(LspType::Unknown),
                            )
                        }
                        "None" => return LspType::Optional(Box::new(LspType::Unknown)),
                        _ => {}
                    }
                    // Look up function return type
                    if let Some(ret) = self.function_returns.get(name) {
                        return ret.clone();
                    }
                    // Check if it's a struct constructor
                    if self.structs.contains_key(name) {
                        return LspType::Named(name.clone());
                    }
                }
                LspType::Unknown
            }
            Expr::StructLit { name, .. } => LspType::Named(name.node.clone()),
            Expr::Field { expr: obj, field } => {
                let obj_type = self.infer_expr_type(obj);
                if let LspType::Named(type_name) = &obj_type {
                    // Look up field type in struct definition
                    if let Some(fields) = self.structs.get(type_name) {
                        for f in fields {
                            if f.name == field.node {
                                return f.ty.clone();
                            }
                        }
                    }
                }
                LspType::Unknown
            }
            Expr::MethodCall {
                receiver, method, ..
            } => {
                let recv_type = self.infer_expr_type(receiver);
                if let LspType::Named(type_name) = &recv_type {
                    // Look up method return type
                    if let Some(methods) = self.type_methods.get(type_name) {
                        for m in methods {
                            if m.name == method.node {
                                if let Some(ref ret_str) = m.ret_type {
                                    return parse_type_string(ret_str);
                                }
                                return LspType::Unit;
                            }
                        }
                    }
                }
                LspType::Unknown
            }
            Expr::Range { .. } => LspType::Range,
            Expr::Block(stmts) => {
                if let Some(last) = stmts.last() {
                    match &last.node {
                        Stmt::Expr(e) => return self.infer_expr_type(e),
                        Stmt::Return(Some(e)) => return self.infer_expr_type(e),
                        _ => {}
                    }
                }
                LspType::Unit
            }
            Expr::If { then, else_, .. } => {
                // Try to infer from then branch's last expression
                if let Some(last) = then.last() {
                    if let Stmt::Expr(e) = &last.node {
                        return self.infer_expr_type(e);
                    }
                }
                if let Some(IfElse::Else(stmts)) = else_ {
                    if let Some(last) = stmts.last() {
                        if let Stmt::Expr(e) = &last.node {
                            return self.infer_expr_type(e);
                        }
                    }
                }
                LspType::Unknown
            }
            _ => LspType::Unknown,
        }
    }

    /// Get completions for a type after a dot
    pub(crate) fn get_dot_completions(&self, type_name: &str) -> Vec<CompletionEntry> {
        let mut completions = Vec::new();

        // Add struct fields
        if let Some(fields) = self.structs.get(type_name) {
            for field in fields {
                completions.push(CompletionEntry {
                    label: field.name.clone(),
                    kind: CompletionKind::Field,
                    detail: field.type_display.clone(),
                    insert_text: field.name.clone(),
                    from_trait: None,
                });
            }
        }

        // Add direct impl methods
        if let Some(methods) = self.type_methods.get(type_name) {
            for method in methods {
                let params_snippet: Vec<String> = method
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, (name, _))| format!("${{{}:{}}}", i + 1, name))
                    .collect();
                let detail = format!(
                    "fn({}){}",
                    method
                        .params
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, t))
                        .collect::<Vec<_>>()
                        .join(", "),
                    method
                        .ret_type
                        .as_ref()
                        .map(|r| format!(" -> {}", r))
                        .unwrap_or_default()
                );
                completions.push(CompletionEntry {
                    label: method.name.clone(),
                    kind: CompletionKind::Method,
                    detail,
                    insert_text: format!("{}({})", method.name, params_snippet.join(", ")),
                    from_trait: method.from_trait.clone(),
                });
            }
        }

        // Add trait methods from implemented traits
        if let Some(traits) = self.type_traits.get(type_name) {
            for trait_name in traits {
                if let Some(methods) = self.trait_methods.get(trait_name) {
                    for method in methods {
                        // Skip if already provided by direct impl
                        if completions.iter().any(|c| c.label == method.name) {
                            continue;
                        }
                        let params_snippet: Vec<String> = method
                            .params
                            .iter()
                            .enumerate()
                            .map(|(i, (name, _))| format!("${{{}:{}}}", i + 1, name))
                            .collect();
                        let detail = format!(
                            "fn({}){}",
                            method
                                .params
                                .iter()
                                .map(|(n, t)| format!("{}: {}", n, t))
                                .collect::<Vec<_>>()
                                .join(", "),
                            method
                                .ret_type
                                .as_ref()
                                .map(|r| format!(" -> {}", r))
                                .unwrap_or_default()
                        );
                        completions.push(CompletionEntry {
                            label: method.name.clone(),
                            kind: CompletionKind::Method,
                            detail,
                            insert_text: format!("{}({})", method.name, params_snippet.join(", ")),
                            from_trait: Some(trait_name.clone()),
                        });
                    }
                }
            }
        }

        completions
    }
}

/// Completion entry for type-aware suggestions
#[derive(Debug, Clone)]
pub(crate) struct CompletionEntry {
    pub(crate) label: String,
    pub(crate) kind: CompletionKind,
    pub(crate) detail: String,
    pub(crate) insert_text: String,
    pub(crate) from_trait: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum CompletionKind {
    Field,
    Method,
}

/// Convert AST Type to LspType
fn ast_type_to_lsp(ty: &Type) -> LspType {
    match ty {
        Type::Named { name, generics } => match name.as_str() {
            "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
            | "f32" | "f64" | "bool" | "str" | "isize" | "usize" | "char" => {
                LspType::Primitive(name.clone())
            }
            "Option" => {
                let inner = generics
                    .first()
                    .map(|g| ast_type_to_lsp(&g.node))
                    .unwrap_or(LspType::Unknown);
                LspType::Optional(Box::new(inner))
            }
            "Result" => {
                let ok = generics
                    .first()
                    .map(|g| ast_type_to_lsp(&g.node))
                    .unwrap_or(LspType::Unknown);
                let err = generics
                    .get(1)
                    .map(|g| ast_type_to_lsp(&g.node))
                    .unwrap_or(LspType::Unknown);
                LspType::Result(Box::new(ok), Box::new(err))
            }
            "Vec" => {
                let inner = generics
                    .first()
                    .map(|g| ast_type_to_lsp(&g.node))
                    .unwrap_or(LspType::Unknown);
                LspType::Array(Box::new(inner))
            }
            _ => LspType::Named(name.clone()),
        },
        Type::Array(inner) => LspType::Array(Box::new(ast_type_to_lsp(&inner.node))),
        Type::Tuple(types) => {
            let inner: Vec<LspType> = types.iter().map(|t| ast_type_to_lsp(&t.node)).collect();
            LspType::Tuple(inner)
        }
        Type::FnPtr { params, ret, .. } | Type::Fn { params, ret } => {
            let param_types: Vec<LspType> =
                params.iter().map(|p| ast_type_to_lsp(&p.node)).collect();
            let ret_type = Box::new(ast_type_to_lsp(&ret.node));
            LspType::Function {
                params: param_types,
                ret: ret_type,
            }
        }
        Type::Optional(inner) => LspType::Optional(Box::new(ast_type_to_lsp(&inner.node))),
        Type::Result(inner) => LspType::Result(
            Box::new(ast_type_to_lsp(&inner.node)),
            Box::new(LspType::Unknown),
        ),
        Type::Ref(inner) | Type::RefMut(inner) => ast_type_to_lsp(&inner.node),
        Type::Pointer(inner) => ast_type_to_lsp(&inner.node),
        Type::Slice(inner) | Type::SliceMut(inner) => {
            LspType::Array(Box::new(ast_type_to_lsp(&inner.node)))
        }
        Type::Unit => LspType::Unit,
        Type::Infer => LspType::Unknown,
        _ => LspType::Unknown,
    }
}

/// Format AST Type as display string
fn format_type(ty: &Type) -> String {
    match ty {
        Type::Named { name, generics } => {
            if generics.is_empty() {
                name.clone()
            } else {
                let gen_strs: Vec<String> = generics.iter().map(|g| format_type(&g.node)).collect();
                format!("{}<{}>", name, gen_strs.join(", "))
            }
        }
        Type::Array(inner) => format!("[{}]", format_type(&inner.node)),
        Type::Tuple(types) => {
            let strs: Vec<String> = types.iter().map(|t| format_type(&t.node)).collect();
            format!("({})", strs.join(", "))
        }
        Type::FnPtr { params, ret, .. } | Type::Fn { params, ret } => {
            let param_strs: Vec<String> = params.iter().map(|p| format_type(&p.node)).collect();
            format!(
                "fn({}) -> {}",
                param_strs.join(", "),
                format_type(&ret.node)
            )
        }
        Type::Unit => "()".to_string(),
        Type::Infer => "_".to_string(),
        Type::Pointer(inner) => format!("*{}", format_type(&inner.node)),
        Type::Ref(inner) => format!("&{}", format_type(&inner.node)),
        Type::RefMut(inner) => format!("&mut {}", format_type(&inner.node)),
        Type::Slice(inner) => format!("&[{}]", format_type(&inner.node)),
        Type::SliceMut(inner) => format!("&mut [{}]", format_type(&inner.node)),
        Type::Optional(inner) => format!("{}?", format_type(&inner.node)),
        Type::Result(inner) => format!("{}!", format_type(&inner.node)),
        _ => format!("{:?}", ty),
    }
}

/// Parse a simple type string back into LspType (for method return types)
fn parse_type_string(s: &str) -> LspType {
    let s = s.trim();
    match s {
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32"
        | "f64" | "bool" | "str" | "isize" | "usize" | "char" => LspType::Primitive(s.to_string()),
        "()" => LspType::Unit,
        _ if s.starts_with("Option<") => LspType::Optional(Box::new(LspType::Unknown)),
        _ if s.starts_with("Result<") => {
            LspType::Result(Box::new(LspType::Unknown), Box::new(LspType::Unknown))
        }
        _ if s.starts_with("Vec<") || s.starts_with('[') => {
            LspType::Array(Box::new(LspType::Unknown))
        }
        _ if s.starts_with("fn(") => LspType::Function {
            params: vec![],
            ret: Box::new(LspType::Unknown),
        },
        _ => LspType::Named(s.to_string()),
    }
}
