//! Lightweight type resolution for LSP features
//!
//! Resolves expression types from AST without full type checking.
//! Used for type-aware completion, hover, and inlay hints.
//!
//! # Submodules
//!
//! - `helpers`: AST type conversion and formatting functions

mod helpers;

use helpers::{ast_type_to_lsp, format_type, parse_type_string};
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

#[cfg(test)]
mod tests {
    use super::*;
    use vais_ast::Span;

    // ========== LspType::display_name tests ==========

    #[test]
    fn test_display_name_named() {
        assert_eq!(
            LspType::Named("MyStruct".to_string()).display_name(),
            "MyStruct"
        );
    }

    #[test]
    fn test_display_name_primitive_i64() {
        assert_eq!(LspType::Primitive("i64".to_string()).display_name(), "i64");
    }

    #[test]
    fn test_display_name_primitive_bool() {
        assert_eq!(
            LspType::Primitive("bool".to_string()).display_name(),
            "bool"
        );
    }

    #[test]
    fn test_display_name_primitive_str() {
        assert_eq!(LspType::Primitive("str".to_string()).display_name(), "str");
    }

    #[test]
    fn test_display_name_array() {
        let t = LspType::Array(Box::new(LspType::Primitive("i64".to_string())));
        assert_eq!(t.display_name(), "[i64]");
    }

    #[test]
    fn test_display_name_array_nested() {
        let t = LspType::Array(Box::new(LspType::Array(Box::new(LspType::Primitive(
            "f64".to_string(),
        )))));
        assert_eq!(t.display_name(), "[[f64]]");
    }

    #[test]
    fn test_display_name_tuple_empty() {
        let t = LspType::Tuple(vec![]);
        assert_eq!(t.display_name(), "()");
    }

    #[test]
    fn test_display_name_tuple_single() {
        let t = LspType::Tuple(vec![LspType::Primitive("i64".to_string())]);
        assert_eq!(t.display_name(), "(i64)");
    }

    #[test]
    fn test_display_name_tuple_multi() {
        let t = LspType::Tuple(vec![
            LspType::Primitive("i64".to_string()),
            LspType::Primitive("bool".to_string()),
            LspType::Primitive("str".to_string()),
        ]);
        assert_eq!(t.display_name(), "(i64, bool, str)");
    }

    #[test]
    fn test_display_name_optional() {
        let t = LspType::Optional(Box::new(LspType::Primitive("i64".to_string())));
        assert_eq!(t.display_name(), "Option<i64>");
    }

    #[test]
    fn test_display_name_result() {
        let t = LspType::Result(
            Box::new(LspType::Primitive("i64".to_string())),
            Box::new(LspType::Primitive("str".to_string())),
        );
        assert_eq!(t.display_name(), "Result<i64, str>");
    }

    #[test]
    fn test_display_name_function() {
        let t = LspType::Function {
            params: vec![
                LspType::Primitive("i64".to_string()),
                LspType::Primitive("bool".to_string()),
            ],
            ret: Box::new(LspType::Primitive("str".to_string())),
        };
        assert_eq!(t.display_name(), "fn(i64, bool) -> str");
    }

    #[test]
    fn test_display_name_function_no_params() {
        let t = LspType::Function {
            params: vec![],
            ret: Box::new(LspType::Unit),
        };
        assert_eq!(t.display_name(), "fn() -> ()");
    }

    #[test]
    fn test_display_name_range() {
        assert_eq!(LspType::Range.display_name(), "Range");
    }

    #[test]
    fn test_display_name_unit() {
        assert_eq!(LspType::Unit.display_name(), "()");
    }

    #[test]
    fn test_display_name_unknown() {
        assert_eq!(LspType::Unknown.display_name(), "_");
    }

    // ========== LspType equality tests ==========

    #[test]
    fn test_lsp_type_eq_primitives() {
        assert_eq!(
            LspType::Primitive("i64".to_string()),
            LspType::Primitive("i64".to_string())
        );
        assert_ne!(
            LspType::Primitive("i64".to_string()),
            LspType::Primitive("f64".to_string())
        );
    }

    #[test]
    fn test_lsp_type_eq_named() {
        assert_eq!(
            LspType::Named("Foo".to_string()),
            LspType::Named("Foo".to_string())
        );
        assert_ne!(
            LspType::Named("Foo".to_string()),
            LspType::Named("Bar".to_string())
        );
    }

    #[test]
    fn test_lsp_type_eq_unit_vs_unknown() {
        assert_ne!(LspType::Unit, LspType::Unknown);
    }

    #[test]
    fn test_lsp_type_eq_range() {
        assert_eq!(LspType::Range, LspType::Range);
    }

    // ========== parse_type_string tests ==========

    #[test]
    fn test_parse_type_string_i64() {
        assert_eq!(
            parse_type_string("i64"),
            LspType::Primitive("i64".to_string())
        );
    }

    #[test]
    fn test_parse_type_string_f64() {
        assert_eq!(
            parse_type_string("f64"),
            LspType::Primitive("f64".to_string())
        );
    }

    #[test]
    fn test_parse_type_string_bool() {
        assert_eq!(
            parse_type_string("bool"),
            LspType::Primitive("bool".to_string())
        );
    }

    #[test]
    fn test_parse_type_string_str() {
        assert_eq!(
            parse_type_string("str"),
            LspType::Primitive("str".to_string())
        );
    }

    #[test]
    fn test_parse_type_string_all_int_types() {
        for ty in [
            "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128",
        ] {
            assert_eq!(parse_type_string(ty), LspType::Primitive(ty.to_string()));
        }
    }

    #[test]
    fn test_parse_type_string_float_types() {
        assert_eq!(
            parse_type_string("f32"),
            LspType::Primitive("f32".to_string())
        );
        assert_eq!(
            parse_type_string("f64"),
            LspType::Primitive("f64".to_string())
        );
    }

    #[test]
    fn test_parse_type_string_size_types() {
        assert_eq!(
            parse_type_string("isize"),
            LspType::Primitive("isize".to_string())
        );
        assert_eq!(
            parse_type_string("usize"),
            LspType::Primitive("usize".to_string())
        );
    }

    #[test]
    fn test_parse_type_string_char() {
        assert_eq!(
            parse_type_string("char"),
            LspType::Primitive("char".to_string())
        );
    }

    #[test]
    fn test_parse_type_string_unit() {
        assert_eq!(parse_type_string("()"), LspType::Unit);
    }

    #[test]
    fn test_parse_type_string_option() {
        assert!(matches!(
            parse_type_string("Option<i64>"),
            LspType::Optional(_)
        ));
    }

    #[test]
    fn test_parse_type_string_result() {
        assert!(matches!(
            parse_type_string("Result<i64, str>"),
            LspType::Result(_, _)
        ));
    }

    #[test]
    fn test_parse_type_string_vec() {
        assert!(matches!(parse_type_string("Vec<i64>"), LspType::Array(_)));
    }

    #[test]
    fn test_parse_type_string_array_bracket() {
        assert!(matches!(parse_type_string("[i64]"), LspType::Array(_)));
    }

    #[test]
    fn test_parse_type_string_fn() {
        assert!(matches!(
            parse_type_string("fn(i64) -> bool"),
            LspType::Function { .. }
        ));
    }

    #[test]
    fn test_parse_type_string_custom_named() {
        assert_eq!(
            parse_type_string("MyStruct"),
            LspType::Named("MyStruct".to_string())
        );
    }

    #[test]
    fn test_parse_type_string_trimming() {
        assert_eq!(
            parse_type_string("  i64  "),
            LspType::Primitive("i64".to_string())
        );
    }

    // ========== ast_type_to_lsp tests ==========

    #[test]
    fn test_ast_type_to_lsp_named_primitive() {
        let ty = Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        };
        assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive("i64".to_string()));
    }

    #[test]
    fn test_ast_type_to_lsp_named_all_primitives() {
        for name in [
            "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64",
            "bool", "str", "isize", "usize", "char",
        ] {
            let ty = Type::Named {
                name: name.to_string(),
                generics: vec![],
            };
            assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive(name.to_string()));
        }
    }

    #[test]
    fn test_ast_type_to_lsp_named_custom() {
        let ty = Type::Named {
            name: "MyStruct".to_string(),
            generics: vec![],
        };
        assert_eq!(ast_type_to_lsp(&ty), LspType::Named("MyStruct".to_string()));
    }

    #[test]
    fn test_ast_type_to_lsp_option() {
        use vais_ast::Spanned;
        let ty = Type::Named {
            name: "Option".to_string(),
            generics: vec![Spanned {
                node: Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span: Span::new(0, 1),
            }],
        };
        let result = ast_type_to_lsp(&ty);
        assert!(matches!(result, LspType::Optional(_)));
    }

    #[test]
    fn test_ast_type_to_lsp_result() {
        use vais_ast::Spanned;
        let ty = Type::Named {
            name: "Result".to_string(),
            generics: vec![
                Spanned {
                    node: Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    span: Span::new(0, 1),
                },
                Spanned {
                    node: Type::Named {
                        name: "str".to_string(),
                        generics: vec![],
                    },
                    span: Span::new(2, 3),
                },
            ],
        };
        let result = ast_type_to_lsp(&ty);
        assert!(matches!(result, LspType::Result(_, _)));
    }

    #[test]
    fn test_ast_type_to_lsp_vec() {
        use vais_ast::Spanned;
        let ty = Type::Named {
            name: "Vec".to_string(),
            generics: vec![Spanned {
                node: Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span: Span::new(0, 1),
            }],
        };
        let result = ast_type_to_lsp(&ty);
        assert!(matches!(result, LspType::Array(_)));
    }

    #[test]
    fn test_ast_type_to_lsp_unit() {
        assert_eq!(ast_type_to_lsp(&Type::Unit), LspType::Unit);
    }

    #[test]
    fn test_ast_type_to_lsp_infer() {
        assert_eq!(ast_type_to_lsp(&Type::Infer), LspType::Unknown);
    }

    #[test]
    fn test_ast_type_to_lsp_array() {
        use vais_ast::Spanned;
        let ty = Type::Array(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        let result = ast_type_to_lsp(&ty);
        assert!(matches!(result, LspType::Array(_)));
    }

    #[test]
    fn test_ast_type_to_lsp_tuple() {
        use vais_ast::Spanned;
        let ty = Type::Tuple(vec![
            Spanned {
                node: Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span: Span::new(0, 1),
            },
            Spanned {
                node: Type::Named {
                    name: "bool".to_string(),
                    generics: vec![],
                },
                span: Span::new(2, 3),
            },
        ]);
        let result = ast_type_to_lsp(&ty);
        assert!(matches!(result, LspType::Tuple(_)));
        if let LspType::Tuple(types) = result {
            assert_eq!(types.len(), 2);
        }
    }

    #[test]
    fn test_ast_type_to_lsp_optional() {
        use vais_ast::Spanned;
        let ty = Type::Optional(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert!(matches!(ast_type_to_lsp(&ty), LspType::Optional(_)));
    }

    #[test]
    fn test_ast_type_to_lsp_result_shorthand() {
        use vais_ast::Spanned;
        let ty = Type::Result(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        let result = ast_type_to_lsp(&ty);
        assert!(matches!(result, LspType::Result(_, _)));
    }

    #[test]
    fn test_ast_type_to_lsp_ref() {
        use vais_ast::Spanned;
        let ty = Type::Ref(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive("i64".to_string()));
    }

    #[test]
    fn test_ast_type_to_lsp_ref_mut() {
        use vais_ast::Spanned;
        let ty = Type::RefMut(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive("i64".to_string()));
    }

    #[test]
    fn test_ast_type_to_lsp_pointer() {
        use vais_ast::Spanned;
        let ty = Type::Pointer(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(ast_type_to_lsp(&ty), LspType::Primitive("i64".to_string()));
    }

    #[test]
    fn test_ast_type_to_lsp_slice() {
        use vais_ast::Spanned;
        let ty = Type::Slice(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert!(matches!(ast_type_to_lsp(&ty), LspType::Array(_)));
    }

    #[test]
    fn test_ast_type_to_lsp_slice_mut() {
        use vais_ast::Spanned;
        let ty = Type::SliceMut(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert!(matches!(ast_type_to_lsp(&ty), LspType::Array(_)));
    }

    #[test]
    fn test_ast_type_to_lsp_option_no_generic() {
        let ty = Type::Named {
            name: "Option".to_string(),
            generics: vec![],
        };
        let result = ast_type_to_lsp(&ty);
        if let LspType::Optional(inner) = result {
            assert_eq!(*inner, LspType::Unknown);
        } else {
            panic!("Expected Optional");
        }
    }

    // ========== format_type tests ==========

    #[test]
    fn test_format_type_named_simple() {
        let ty = Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        };
        assert_eq!(format_type(&ty), "i64");
    }

    #[test]
    fn test_format_type_named_with_generics() {
        use vais_ast::Spanned;
        let ty = Type::Named {
            name: "Vec".to_string(),
            generics: vec![Spanned {
                node: Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span: Span::new(0, 1),
            }],
        };
        assert_eq!(format_type(&ty), "Vec<i64>");
    }

    #[test]
    fn test_format_type_named_multi_generics() {
        use vais_ast::Spanned;
        let ty = Type::Named {
            name: "HashMap".to_string(),
            generics: vec![
                Spanned {
                    node: Type::Named {
                        name: "str".to_string(),
                        generics: vec![],
                    },
                    span: Span::new(0, 1),
                },
                Spanned {
                    node: Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    span: Span::new(2, 3),
                },
            ],
        };
        assert_eq!(format_type(&ty), "HashMap<str, i64>");
    }

    #[test]
    fn test_format_type_array() {
        use vais_ast::Spanned;
        let ty = Type::Array(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(format_type(&ty), "[i64]");
    }

    #[test]
    fn test_format_type_tuple() {
        use vais_ast::Spanned;
        let ty = Type::Tuple(vec![
            Spanned {
                node: Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span: Span::new(0, 1),
            },
            Spanned {
                node: Type::Named {
                    name: "bool".to_string(),
                    generics: vec![],
                },
                span: Span::new(2, 3),
            },
        ]);
        assert_eq!(format_type(&ty), "(i64, bool)");
    }

    #[test]
    fn test_format_type_unit() {
        assert_eq!(format_type(&Type::Unit), "()");
    }

    #[test]
    fn test_format_type_infer() {
        assert_eq!(format_type(&Type::Infer), "_");
    }

    #[test]
    fn test_format_type_pointer() {
        use vais_ast::Spanned;
        let ty = Type::Pointer(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(format_type(&ty), "*i64");
    }

    #[test]
    fn test_format_type_ref() {
        use vais_ast::Spanned;
        let ty = Type::Ref(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(format_type(&ty), "&i64");
    }

    #[test]
    fn test_format_type_ref_mut() {
        use vais_ast::Spanned;
        let ty = Type::RefMut(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(format_type(&ty), "&mut i64");
    }

    #[test]
    fn test_format_type_slice() {
        use vais_ast::Spanned;
        let ty = Type::Slice(Box::new(Spanned {
            node: Type::Named {
                name: "u8".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(format_type(&ty), "&[u8]");
    }

    #[test]
    fn test_format_type_slice_mut() {
        use vais_ast::Spanned;
        let ty = Type::SliceMut(Box::new(Spanned {
            node: Type::Named {
                name: "u8".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(format_type(&ty), "&mut [u8]");
    }

    #[test]
    fn test_format_type_optional() {
        use vais_ast::Spanned;
        let ty = Type::Optional(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(format_type(&ty), "i64?");
    }

    #[test]
    fn test_format_type_result_shorthand() {
        use vais_ast::Spanned;
        let ty = Type::Result(Box::new(Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }));
        assert_eq!(format_type(&ty), "i64!");
    }

    // ========== TypeContext::infer_expr_type tests ==========

    fn make_ctx() -> TypeContext {
        TypeContext {
            structs: HashMap::new(),
            type_methods: HashMap::new(),
            trait_methods: HashMap::new(),
            type_traits: HashMap::new(),
            enum_variants: HashMap::new(),
            function_returns: HashMap::new(),
            variable_types: HashMap::new(),
        }
    }

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned {
            node,
            span: Span::new(0, 1),
        }
    }

    #[test]
    fn test_infer_int() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Int(42));
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("i64".to_string())
        );
    }

    #[test]
    fn test_infer_float() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Float(3.14));
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("f64".to_string())
        );
    }

    #[test]
    fn test_infer_bool() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Bool(true));
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("bool".to_string())
        );
    }

    #[test]
    fn test_infer_string() {
        let ctx = make_ctx();
        let expr = spanned(Expr::String("hello".to_string()));
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("str".to_string())
        );
    }

    #[test]
    fn test_infer_ident_known() {
        let mut ctx = make_ctx();
        ctx.variable_types
            .insert("x".to_string(), LspType::Primitive("i64".to_string()));
        let expr = spanned(Expr::Ident("x".to_string()));
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("i64".to_string())
        );
    }

    #[test]
    fn test_infer_ident_unknown() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Ident("x".to_string()));
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
    }

    #[test]
    fn test_infer_array_empty() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Array(vec![]));
        let result = ctx.infer_expr_type(&expr);
        assert!(matches!(result, LspType::Array(_)));
    }

    #[test]
    fn test_infer_array_with_elements() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Array(vec![
            spanned(Expr::Int(1)),
            spanned(Expr::Int(2)),
        ]));
        let result = ctx.infer_expr_type(&expr);
        if let LspType::Array(inner) = result {
            assert_eq!(*inner, LspType::Primitive("i64".to_string()));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_infer_tuple() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Tuple(vec![
            spanned(Expr::Int(1)),
            spanned(Expr::Bool(true)),
        ]));
        let result = ctx.infer_expr_type(&expr);
        if let LspType::Tuple(types) = result {
            assert_eq!(types.len(), 2);
            assert_eq!(types[0], LspType::Primitive("i64".to_string()));
            assert_eq!(types[1], LspType::Primitive("bool".to_string()));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_infer_call_some() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("Some".to_string()))),
            args: vec![spanned(Expr::Int(42))],
        });
        assert!(matches!(ctx.infer_expr_type(&expr), LspType::Optional(_)));
    }

    #[test]
    fn test_infer_call_ok() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("Ok".to_string()))),
            args: vec![spanned(Expr::Int(42))],
        });
        assert!(matches!(ctx.infer_expr_type(&expr), LspType::Result(_, _)));
    }

    #[test]
    fn test_infer_call_err() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("Err".to_string()))),
            args: vec![spanned(Expr::String("error".to_string()))],
        });
        assert!(matches!(ctx.infer_expr_type(&expr), LspType::Result(_, _)));
    }

    #[test]
    fn test_infer_call_none() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("None".to_string()))),
            args: vec![],
        });
        assert!(matches!(ctx.infer_expr_type(&expr), LspType::Optional(_)));
    }

    #[test]
    fn test_infer_call_known_function() {
        let mut ctx = make_ctx();
        ctx.function_returns
            .insert("foo".to_string(), LspType::Primitive("bool".to_string()));
        let expr = spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("foo".to_string()))),
            args: vec![],
        });
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("bool".to_string())
        );
    }

    #[test]
    fn test_infer_call_struct_constructor() {
        let mut ctx = make_ctx();
        ctx.structs.insert("Point".to_string(), vec![]);
        let expr = spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("Point".to_string()))),
            args: vec![],
        });
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Named("Point".to_string())
        );
    }

    #[test]
    fn test_infer_call_unknown_function() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("unknown_fn".to_string()))),
            args: vec![],
        });
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
    }

    #[test]
    fn test_infer_struct_lit() {
        let ctx = make_ctx();
        let expr = spanned(Expr::StructLit {
            name: spanned("Point".to_string()),
            fields: vec![],
        });
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Named("Point".to_string())
        );
    }

    #[test]
    fn test_infer_field_access() {
        let mut ctx = make_ctx();
        ctx.structs.insert(
            "Point".to_string(),
            vec![
                FieldInfo {
                    name: "x".to_string(),
                    ty: LspType::Primitive("i64".to_string()),
                    type_display: "i64".to_string(),
                },
                FieldInfo {
                    name: "y".to_string(),
                    ty: LspType::Primitive("f64".to_string()),
                    type_display: "f64".to_string(),
                },
            ],
        );
        ctx.variable_types
            .insert("p".to_string(), LspType::Named("Point".to_string()));
        let expr = spanned(Expr::Field {
            expr: Box::new(spanned(Expr::Ident("p".to_string()))),
            field: spanned("x".to_string()),
        });
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("i64".to_string())
        );
    }

    #[test]
    fn test_infer_field_access_unknown_field() {
        let mut ctx = make_ctx();
        ctx.structs.insert(
            "Point".to_string(),
            vec![FieldInfo {
                name: "x".to_string(),
                ty: LspType::Primitive("i64".to_string()),
                type_display: "i64".to_string(),
            }],
        );
        ctx.variable_types
            .insert("p".to_string(), LspType::Named("Point".to_string()));
        let expr = spanned(Expr::Field {
            expr: Box::new(spanned(Expr::Ident("p".to_string()))),
            field: spanned("z".to_string()),
        });
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
    }

    #[test]
    fn test_infer_method_call() {
        let mut ctx = make_ctx();
        ctx.type_methods.insert(
            "Vec".to_string(),
            vec![MethodInfo {
                name: "len".to_string(),
                params: vec![],
                ret_type: Some("i64".to_string()),
                from_trait: None,
            }],
        );
        ctx.variable_types
            .insert("v".to_string(), LspType::Named("Vec".to_string()));
        let expr = spanned(Expr::MethodCall {
            receiver: Box::new(spanned(Expr::Ident("v".to_string()))),
            method: spanned("len".to_string()),
            args: vec![],
        });
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("i64".to_string())
        );
    }

    #[test]
    fn test_infer_range() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Range {
            start: Some(Box::new(spanned(Expr::Int(0)))),
            end: Some(Box::new(spanned(Expr::Int(10)))),
            inclusive: false,
        });
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Range);
    }

    #[test]
    fn test_infer_block_empty() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Block(vec![]));
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Unit);
    }

    #[test]
    fn test_infer_block_with_expr() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Block(vec![spanned(Stmt::Expr(Box::new(spanned(
            Expr::Int(42),
        ))))]));
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("i64".to_string())
        );
    }

    #[test]
    fn test_infer_block_with_return() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Block(vec![spanned(Stmt::Return(Some(Box::new(
            spanned(Expr::Bool(true)),
        ))))]));
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("bool".to_string())
        );
    }

    // ========== TypeContext::from_module tests ==========

    fn parse_module(source: &str) -> Module {
        vais_parser::parse(source).expect("parse failed")
    }

    #[test]
    fn test_from_module_function() {
        let ast = parse_module("F foo() -> i64 { 42 }");
        let ctx = TypeContext::from_module(&ast);
        assert!(ctx.function_returns.contains_key("foo"));
    }

    #[test]
    fn test_from_module_struct() {
        let ast = parse_module("S Point { x: i64, y: i64 }");
        let ctx = TypeContext::from_module(&ast);
        assert!(ctx.structs.contains_key("Point"));
        assert_eq!(ctx.structs["Point"].len(), 2);
        assert_eq!(ctx.structs["Point"][0].name, "x");
        assert_eq!(ctx.structs["Point"][1].name, "y");
    }

    #[test]
    fn test_from_module_enum() {
        let ast = parse_module("E Color { Red, Green, Blue }");
        let ctx = TypeContext::from_module(&ast);
        assert!(ctx.enum_variants.contains_key("Color"));
        assert_eq!(ctx.enum_variants["Color"], vec!["Red", "Green", "Blue"]);
    }

    #[test]
    fn test_from_module_impl() {
        let ast = parse_module("S Point { x: i64 }\nX Point { F get_x(self) -> i64 { self.x } }");
        let ctx = TypeContext::from_module(&ast);
        assert!(ctx.type_methods.contains_key("Point"));
        assert_eq!(ctx.type_methods["Point"].len(), 1);
        assert_eq!(ctx.type_methods["Point"][0].name, "get_x");
    }

    #[test]
    fn test_from_module_trait() {
        let ast = parse_module("W Printable { F print(self) -> i64 }");
        let ctx = TypeContext::from_module(&ast);
        assert!(ctx.trait_methods.contains_key("Printable"));
    }

    #[test]
    fn test_from_module_trait_impl() {
        let ast = parse_module("W Printable { F print(self) -> i64 }\nS Foo { x: i64 }\nX Foo: Printable { F print(self) -> i64 { self.x } }");
        let ctx = TypeContext::from_module(&ast);
        assert!(ctx.type_traits.contains_key("Foo"));
        assert!(ctx.type_traits["Foo"].contains(&"Printable".to_string()));
    }

    #[test]
    fn test_from_module_function_no_return_type() {
        let ast = parse_module("F noop() { }");
        let ctx = TypeContext::from_module(&ast);
        if let Some(ret) = ctx.function_returns.get("noop") {
            assert_eq!(*ret, LspType::Unit);
        }
    }

    // ========== TypeContext::get_dot_completions tests ==========

    #[test]
    fn test_dot_completions_struct_fields() {
        let mut ctx = make_ctx();
        ctx.structs.insert(
            "Point".to_string(),
            vec![
                FieldInfo {
                    name: "x".to_string(),
                    ty: LspType::Primitive("i64".to_string()),
                    type_display: "i64".to_string(),
                },
                FieldInfo {
                    name: "y".to_string(),
                    ty: LspType::Primitive("i64".to_string()),
                    type_display: "i64".to_string(),
                },
            ],
        );
        let completions = ctx.get_dot_completions("Point");
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.label == "x"));
        assert!(completions.iter().any(|c| c.label == "y"));
    }

    #[test]
    fn test_dot_completions_methods() {
        let mut ctx = make_ctx();
        ctx.type_methods.insert(
            "Vec".to_string(),
            vec![
                MethodInfo {
                    name: "len".to_string(),
                    params: vec![],
                    ret_type: Some("i64".to_string()),
                    from_trait: None,
                },
                MethodInfo {
                    name: "push".to_string(),
                    params: vec![("item".to_string(), "T".to_string())],
                    ret_type: None,
                    from_trait: None,
                },
            ],
        );
        let completions = ctx.get_dot_completions("Vec");
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.label == "len"));
        assert!(completions.iter().any(|c| c.label == "push"));
    }

    #[test]
    fn test_dot_completions_trait_methods() {
        let mut ctx = make_ctx();
        ctx.type_traits
            .insert("Foo".to_string(), vec!["ToString".to_string()]);
        ctx.trait_methods.insert(
            "ToString".to_string(),
            vec![MethodInfo {
                name: "to_string".to_string(),
                params: vec![],
                ret_type: Some("str".to_string()),
                from_trait: Some("ToString".to_string()),
            }],
        );
        let completions = ctx.get_dot_completions("Foo");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "to_string");
        assert_eq!(completions[0].from_trait, Some("ToString".to_string()));
    }

    #[test]
    fn test_dot_completions_no_duplicates() {
        let mut ctx = make_ctx();
        ctx.type_methods.insert(
            "Foo".to_string(),
            vec![MethodInfo {
                name: "display".to_string(),
                params: vec![],
                ret_type: Some("str".to_string()),
                from_trait: None,
            }],
        );
        ctx.type_traits
            .insert("Foo".to_string(), vec!["Display".to_string()]);
        ctx.trait_methods.insert(
            "Display".to_string(),
            vec![MethodInfo {
                name: "display".to_string(),
                params: vec![],
                ret_type: Some("str".to_string()),
                from_trait: Some("Display".to_string()),
            }],
        );
        let completions = ctx.get_dot_completions("Foo");
        // Should only have 1 "display" entry (direct impl takes priority)
        assert_eq!(
            completions.iter().filter(|c| c.label == "display").count(),
            1
        );
    }

    #[test]
    fn test_dot_completions_empty_type() {
        let ctx = make_ctx();
        let completions = ctx.get_dot_completions("NonExistent");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_dot_completions_fields_and_methods() {
        let mut ctx = make_ctx();
        ctx.structs.insert(
            "MyStruct".to_string(),
            vec![FieldInfo {
                name: "value".to_string(),
                ty: LspType::Primitive("i64".to_string()),
                type_display: "i64".to_string(),
            }],
        );
        ctx.type_methods.insert(
            "MyStruct".to_string(),
            vec![MethodInfo {
                name: "get_value".to_string(),
                params: vec![],
                ret_type: Some("i64".to_string()),
                from_trait: None,
            }],
        );
        let completions = ctx.get_dot_completions("MyStruct");
        assert_eq!(completions.len(), 2);
        let field = completions.iter().find(|c| c.label == "value").unwrap();
        assert!(matches!(field.kind, CompletionKind::Field));
        let method = completions.iter().find(|c| c.label == "get_value").unwrap();
        assert!(matches!(method.kind, CompletionKind::Method));
    }

    // ========== Additional LspType display_name edge cases ==========

    #[test]
    fn test_display_name_optional_nested() {
        let t = LspType::Optional(Box::new(LspType::Optional(Box::new(LspType::Primitive(
            "i64".to_string(),
        )))));
        assert_eq!(t.display_name(), "Option<Option<i64>>");
    }

    #[test]
    fn test_display_name_result_nested() {
        let t = LspType::Result(
            Box::new(LspType::Array(Box::new(LspType::Primitive(
                "i64".to_string(),
            )))),
            Box::new(LspType::Primitive("str".to_string())),
        );
        assert_eq!(t.display_name(), "Result<[i64], str>");
    }

    #[test]
    fn test_display_name_function_complex() {
        let t = LspType::Function {
            params: vec![
                LspType::Array(Box::new(LspType::Primitive("i64".to_string()))),
                LspType::Optional(Box::new(LspType::Primitive("str".to_string()))),
            ],
            ret: Box::new(LspType::Result(
                Box::new(LspType::Primitive("bool".to_string())),
                Box::new(LspType::Primitive("str".to_string())),
            )),
        };
        assert_eq!(
            t.display_name(),
            "fn([i64], Option<str>) -> Result<bool, str>"
        );
    }

    #[test]
    fn test_display_name_tuple_with_array() {
        let t = LspType::Tuple(vec![
            LspType::Array(Box::new(LspType::Primitive("i64".to_string()))),
            LspType::Primitive("bool".to_string()),
        ]);
        assert_eq!(t.display_name(), "([i64], bool)");
    }

    #[test]
    fn test_display_name_array_of_tuples() {
        let t = LspType::Array(Box::new(LspType::Tuple(vec![
            LspType::Primitive("i64".to_string()),
            LspType::Primitive("str".to_string()),
        ])));
        assert_eq!(t.display_name(), "[(i64, str)]");
    }

    // ========== Additional LspType equality edge cases ==========

    #[test]
    fn test_lsp_type_clone() {
        let t = LspType::Array(Box::new(LspType::Primitive("i64".to_string())));
        let cloned = t.clone();
        assert_eq!(t, cloned);
    }

    #[test]
    fn test_lsp_type_debug() {
        let t = LspType::Primitive("i64".to_string());
        let debug = format!("{:?}", t);
        assert!(debug.contains("Primitive"));
        assert!(debug.contains("i64"));
    }

    #[test]
    fn test_lsp_type_ne_different_variants() {
        assert_ne!(
            LspType::Primitive("i64".to_string()),
            LspType::Named("i64".to_string())
        );
        assert_ne!(LspType::Unit, LspType::Range);
        assert_ne!(LspType::Array(Box::new(LspType::Unit)), LspType::Unit);
    }

    // ========== Additional parse_type_string edge cases ==========

    #[test]
    fn test_parse_type_string_tuple() {
        // parse_type_string doesn't parse tuple syntax, falls through to Named
        let result = parse_type_string("(i64, bool)");
        assert!(matches!(result, LspType::Named(_)));
    }

    #[test]
    fn test_parse_type_string_hashmap() {
        // HashMap is not a known generic, should be Named
        let result = parse_type_string("HashMap<str, i64>");
        assert!(matches!(result, LspType::Named(_)));
    }

    #[test]
    fn test_parse_type_string_empty_string() {
        // Empty string falls through to Named("")
        let result = parse_type_string("");
        assert_eq!(result, LspType::Named("".to_string()));
    }

    #[test]
    fn test_parse_type_string_whitespace_only() {
        // Trimmed whitespace becomes empty string, falls through to Named
        let result = parse_type_string("   ");
        assert_eq!(result, LspType::Named("".to_string()));
    }

    // ========== Additional ast_type_to_lsp edge cases ==========

    #[test]
    fn test_ast_type_to_lsp_nested_generics() {
        use vais_ast::Spanned;
        let ty = Type::Named {
            name: "Vec".to_string(),
            generics: vec![Spanned {
                node: Type::Named {
                    name: "Option".to_string(),
                    generics: vec![Spanned {
                        node: Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        span: Span::new(0, 1),
                    }],
                },
                span: Span::new(0, 1),
            }],
        };
        let result = ast_type_to_lsp(&ty);
        if let LspType::Array(inner) = result {
            assert!(matches!(*inner, LspType::Optional(_)));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_ast_type_to_lsp_result_with_generics() {
        use vais_ast::Spanned;
        let ty = Type::Named {
            name: "Result".to_string(),
            generics: vec![
                Spanned {
                    node: Type::Named {
                        name: "Vec".to_string(),
                        generics: vec![Spanned {
                            node: Type::Named {
                                name: "i64".to_string(),
                                generics: vec![],
                            },
                            span: Span::new(0, 1),
                        }],
                    },
                    span: Span::new(0, 1),
                },
                Spanned {
                    node: Type::Named {
                        name: "str".to_string(),
                        generics: vec![],
                    },
                    span: Span::new(2, 3),
                },
            ],
        };
        let result = ast_type_to_lsp(&ty);
        if let LspType::Result(ok, _err) = result {
            assert!(matches!(*ok, LspType::Array(_)));
        } else {
            panic!("Expected Result");
        }
    }

    // ========== Additional format_type edge cases ==========

    #[test]
    fn test_format_type_nested_generics() {
        use vais_ast::Spanned;
        let ty = Type::Named {
            name: "Vec".to_string(),
            generics: vec![Spanned {
                node: Type::Named {
                    name: "Vec".to_string(),
                    generics: vec![Spanned {
                        node: Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        span: Span::new(0, 1),
                    }],
                },
                span: Span::new(0, 1),
            }],
        };
        assert_eq!(format_type(&ty), "Vec<Vec<i64>>");
    }

    #[test]
    fn test_format_type_empty_tuple() {
        let ty = Type::Tuple(vec![]);
        assert_eq!(format_type(&ty), "()");
    }

    #[test]
    fn test_format_type_single_element_tuple() {
        use vais_ast::Spanned;
        let ty = Type::Tuple(vec![Spanned {
            node: Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            span: Span::new(0, 1),
        }]);
        assert_eq!(format_type(&ty), "(i64)");
    }

    // ========== Additional TypeContext::infer_expr_type edge cases ==========

    #[test]
    fn test_infer_nested_call() {
        let mut ctx = make_ctx();
        ctx.function_returns
            .insert("inner".to_string(), LspType::Primitive("i64".to_string()));
        let expr = spanned(Expr::Call {
            func: Box::new(spanned(Expr::Ident("outer".to_string()))),
            args: vec![spanned(Expr::Call {
                func: Box::new(spanned(Expr::Ident("inner".to_string()))),
                args: vec![],
            })],
        });
        // outer is unknown, so result is Unknown
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
    }

    #[test]
    fn test_infer_struct_lit_with_fields() {
        let ctx = make_ctx();
        let expr = spanned(Expr::StructLit {
            name: spanned("Config".to_string()),
            fields: vec![
                (spanned("debug".to_string()), spanned(Expr::Bool(true))),
                (spanned("level".to_string()), spanned(Expr::Int(3))),
            ],
        });
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Named("Config".to_string())
        );
    }

    #[test]
    fn test_infer_method_call_trait_not_resolved() {
        // infer_expr_type only checks type_methods, not trait_methods
        // so trait-only methods return Unknown
        let mut ctx = make_ctx();
        ctx.type_traits
            .insert("MyType".to_string(), vec!["Display".to_string()]);
        ctx.trait_methods.insert(
            "Display".to_string(),
            vec![MethodInfo {
                name: "to_string".to_string(),
                params: vec![],
                ret_type: Some("str".to_string()),
                from_trait: Some("Display".to_string()),
            }],
        );
        ctx.variable_types
            .insert("x".to_string(), LspType::Named("MyType".to_string()));
        let expr = spanned(Expr::MethodCall {
            receiver: Box::new(spanned(Expr::Ident("x".to_string()))),
            method: spanned("to_string".to_string()),
            args: vec![],
        });
        // Returns Unknown because infer_expr_type doesn't look up trait_methods
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
    }

    #[test]
    fn test_infer_method_call_unknown_receiver() {
        let ctx = make_ctx();
        let expr = spanned(Expr::MethodCall {
            receiver: Box::new(spanned(Expr::Ident("unknown".to_string()))),
            method: spanned("foo".to_string()),
            args: vec![],
        });
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
    }

    #[test]
    fn test_infer_range_partial() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Range {
            start: Some(Box::new(spanned(Expr::Int(0)))),
            end: None,
            inclusive: false,
        });
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Range);
    }

    #[test]
    fn test_infer_block_with_let() {
        let ctx = make_ctx();
        let expr = spanned(Expr::Block(vec![
            spanned(Stmt::Let {
                name: spanned("x".to_string()),
                ty: None,
                value: Box::new(spanned(Expr::Int(42))),
                is_mut: false,
                ownership: vais_ast::Ownership::Regular,
            }),
            spanned(Stmt::Expr(Box::new(spanned(Expr::Ident("x".to_string()))))),
        ]));
        // Block returns the type of the last stmt
        let result = ctx.infer_expr_type(&expr);
        // Since block processing uses last stmt, ident "x" won't be in scope at inference time
        // (infer_expr_type doesn't add let bindings to its scope) -- this is expected Unknown
        assert_eq!(result, LspType::Unknown);
    }

    // ========== Additional TypeContext::from_module edge cases ==========

    #[test]
    fn test_from_module_multiple_functions() {
        let ast = parse_module("F foo() -> i64 { 0 }\nF bar() -> bool { true }");
        let ctx = TypeContext::from_module(&ast);
        assert!(ctx.function_returns.contains_key("foo"));
        assert!(ctx.function_returns.contains_key("bar"));
    }

    #[test]
    fn test_from_module_struct_with_typed_fields() {
        let ast = parse_module("S Config { debug: bool, level: i64, name: str }");
        let ctx = TypeContext::from_module(&ast);
        assert_eq!(ctx.structs["Config"].len(), 3);
        assert_eq!(ctx.structs["Config"][0].name, "debug");
        assert_eq!(ctx.structs["Config"][1].name, "level");
        assert_eq!(ctx.structs["Config"][2].name, "name");
    }

    #[test]
    fn test_from_module_empty() {
        let ast = parse_module("");
        let ctx = TypeContext::from_module(&ast);
        assert!(ctx.structs.is_empty());
        assert!(ctx.function_returns.is_empty());
        assert!(ctx.enum_variants.is_empty());
    }

    #[test]
    fn test_from_module_multiple_enums() {
        let ast = parse_module("E Color { Red, Blue }\nE Size { Small, Big }");
        let ctx = TypeContext::from_module(&ast);
        assert_eq!(ctx.enum_variants.len(), 2);
        assert_eq!(ctx.enum_variants["Color"].len(), 2);
        assert_eq!(ctx.enum_variants["Size"].len(), 2);
    }

    #[test]
    fn test_from_module_impl_multiple_methods() {
        let ast = parse_module("S Vec { len: i64 }\nX Vec { F push(self, x: i64) -> i64 { 0 }\nF pop(self) -> i64 { 0 } }");
        let ctx = TypeContext::from_module(&ast);
        assert_eq!(ctx.type_methods["Vec"].len(), 2);
    }

    // ========== Additional TypeContext::get_dot_completions edge cases ==========

    #[test]
    fn test_dot_completions_method_detail() {
        let mut ctx = make_ctx();
        ctx.type_methods.insert(
            "Vec".to_string(),
            vec![MethodInfo {
                name: "len".to_string(),
                params: vec![],
                ret_type: Some("i64".to_string()),
                from_trait: None,
            }],
        );
        let completions = ctx.get_dot_completions("Vec");
        assert_eq!(completions[0].label, "len");
        assert!(completions[0].detail.contains("i64"));
    }

    #[test]
    fn test_dot_completions_field_detail() {
        let mut ctx = make_ctx();
        ctx.structs.insert(
            "Point".to_string(),
            vec![FieldInfo {
                name: "x".to_string(),
                ty: LspType::Primitive("f64".to_string()),
                type_display: "f64".to_string(),
            }],
        );
        let completions = ctx.get_dot_completions("Point");
        assert_eq!(completions[0].label, "x");
        assert!(completions[0].detail.contains("f64"));
    }

    #[test]
    fn test_dot_completions_multiple_traits() {
        let mut ctx = make_ctx();
        ctx.type_traits.insert(
            "MyType".to_string(),
            vec!["Display".to_string(), "Debug".to_string()],
        );
        ctx.trait_methods.insert(
            "Display".to_string(),
            vec![MethodInfo {
                name: "display".to_string(),
                params: vec![],
                ret_type: Some("str".to_string()),
                from_trait: Some("Display".to_string()),
            }],
        );
        ctx.trait_methods.insert(
            "Debug".to_string(),
            vec![MethodInfo {
                name: "debug".to_string(),
                params: vec![],
                ret_type: Some("str".to_string()),
                from_trait: Some("Debug".to_string()),
            }],
        );
        let completions = ctx.get_dot_completions("MyType");
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.label == "display"));
        assert!(completions.iter().any(|c| c.label == "debug"));
    }

    // ========== FieldInfo and MethodInfo tests ==========

    #[test]
    fn test_field_info_clone() {
        let fi = FieldInfo {
            name: "x".to_string(),
            ty: LspType::Primitive("i64".to_string()),
            type_display: "i64".to_string(),
        };
        let cloned = fi.clone();
        assert_eq!(fi.name, cloned.name);
        assert_eq!(fi.type_display, cloned.type_display);
    }

    #[test]
    fn test_method_info_clone() {
        let mi = MethodInfo {
            name: "foo".to_string(),
            params: vec![("x".to_string(), "i64".to_string())],
            ret_type: Some("bool".to_string()),
            from_trait: Some("MyTrait".to_string()),
        };
        let cloned = mi.clone();
        assert_eq!(mi.name, cloned.name);
        assert_eq!(mi.params, cloned.params);
        assert_eq!(mi.ret_type, cloned.ret_type);
        assert_eq!(mi.from_trait, cloned.from_trait);
    }

    #[test]
    fn test_method_info_no_return() {
        let mi = MethodInfo {
            name: "set".to_string(),
            params: vec![("val".to_string(), "i64".to_string())],
            ret_type: None,
            from_trait: None,
        };
        assert!(mi.ret_type.is_none());
        assert!(mi.from_trait.is_none());
    }

    #[test]
    fn test_completion_entry_field_kind() {
        let entry = CompletionEntry {
            label: "x".to_string(),
            kind: CompletionKind::Field,
            detail: "i64".to_string(),
            insert_text: "x".to_string(),
            from_trait: None,
        };
        assert!(matches!(entry.kind, CompletionKind::Field));
        assert!(entry.from_trait.is_none());
    }

    #[test]
    fn test_completion_entry_method_kind() {
        let entry = CompletionEntry {
            label: "len".to_string(),
            kind: CompletionKind::Method,
            detail: "() -> i64".to_string(),
            insert_text: "len()".to_string(),
            from_trait: Some("Sized".to_string()),
        };
        assert!(matches!(entry.kind, CompletionKind::Method));
        assert_eq!(entry.from_trait, Some("Sized".to_string()));
    }

    #[test]
    fn test_completion_entry_clone() {
        let entry = CompletionEntry {
            label: "test".to_string(),
            kind: CompletionKind::Field,
            detail: "detail".to_string(),
            insert_text: "test".to_string(),
            from_trait: None,
        };
        let cloned = entry.clone();
        assert_eq!(entry.label, cloned.label);
        assert_eq!(entry.detail, cloned.detail);
    }

    // ========== infer_expr_type — If expression ==========

    #[test]
    fn test_infer_if_then_branch() {
        let ctx = TypeContext {
            structs: HashMap::new(),
            type_methods: HashMap::new(),
            trait_methods: HashMap::new(),
            type_traits: HashMap::new(),
            enum_variants: HashMap::new(),
            function_returns: HashMap::new(),
            variable_types: HashMap::new(),
        };
        // If with then branch returning an int
        let expr = spanned(Expr::If {
            cond: Box::new(spanned(Expr::Bool(true))),
            then: vec![spanned(Stmt::Expr(Box::new(spanned(Expr::Int(42)))))],
            else_: None,
        });
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("i64".to_string())
        );
    }

    #[test]
    fn test_infer_if_else_branch() {
        let ctx = TypeContext {
            structs: HashMap::new(),
            type_methods: HashMap::new(),
            trait_methods: HashMap::new(),
            type_traits: HashMap::new(),
            enum_variants: HashMap::new(),
            function_returns: HashMap::new(),
            variable_types: HashMap::new(),
        };
        // If with empty then, but else branch returning a string
        let expr = spanned(Expr::If {
            cond: Box::new(spanned(Expr::Bool(true))),
            then: vec![],
            else_: Some(IfElse::Else(vec![spanned(Stmt::Expr(Box::new(spanned(
                Expr::String("hello".to_string()),
            ))))])),
        });
        assert_eq!(
            ctx.infer_expr_type(&expr),
            LspType::Primitive("str".to_string())
        );
    }

    #[test]
    fn test_infer_if_empty_both() {
        let ctx = TypeContext {
            structs: HashMap::new(),
            type_methods: HashMap::new(),
            trait_methods: HashMap::new(),
            type_traits: HashMap::new(),
            enum_variants: HashMap::new(),
            function_returns: HashMap::new(),
            variable_types: HashMap::new(),
        };
        let expr = spanned(Expr::If {
            cond: Box::new(spanned(Expr::Bool(true))),
            then: vec![],
            else_: None,
        });
        assert_eq!(ctx.infer_expr_type(&expr), LspType::Unknown);
    }

    // ========== parse_type_string — additional patterns ==========

    #[test]
    fn test_parse_type_string_option_generic() {
        let t = parse_type_string("Option<i64>");
        assert!(matches!(t, LspType::Optional(_)));
    }

    #[test]
    fn test_parse_type_string_result_generic() {
        let t = parse_type_string("Result<i64, str>");
        assert!(matches!(t, LspType::Result(_, _)));
    }

    #[test]
    fn test_parse_type_string_array_bracket_notation() {
        let t = parse_type_string("[i64]");
        assert!(matches!(t, LspType::Array(_)));
    }

    #[test]
    fn test_parse_type_string_vec_generic() {
        let t = parse_type_string("Vec<bool>");
        assert!(matches!(t, LspType::Array(_)));
    }

    // ========== format_type — additional edge cases ==========

    #[test]
    fn test_format_type_optional_shorthand() {
        // Type::Optional produces "type?" format
        let ty = Type::Optional(Box::new(spanned(Type::Named {
            name: "i64".to_string(),
            generics: vec![],
        })));
        assert_eq!(format_type(&ty), "i64?");
    }

    #[test]
    fn test_format_type_result_shorthand_explicit() {
        // Type::Result produces "type!" format
        let ty = Type::Result(Box::new(spanned(Type::Named {
            name: "str".to_string(),
            generics: vec![],
        })));
        assert_eq!(format_type(&ty), "str!");
    }
}
