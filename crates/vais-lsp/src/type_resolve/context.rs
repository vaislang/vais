//! TypeContext construction and variable binding collection

use super::*;

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
}
