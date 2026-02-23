//! Module generation with generic instantiations.
//!
//! Handles compilation of modules that include generic function and struct
//! specializations, producing monomorphized LLVM IR for each concrete
//! type argument combination.

use super::*;

impl CodeGenerator {
    pub fn generate_module_with_instantiations(
        &mut self,
        module: &Module,
        instantiations: &[vais_types::GenericInstantiation],
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        self.emit_module_header(&mut ir);

        // First pass: collect declarations (including generic templates)
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    // Track this function name (generic or not)
                    self.types.declared_functions.insert(f.name.node.clone());

                    if !f.generics.is_empty() {
                        // Store generic function for later specialization
                        self.generics
                            .function_templates
                            .insert(f.name.node.clone(), std::rc::Rc::new(f.clone()));
                    } else {
                        self.register_function(f)?;
                    }
                }
                Item::Struct(s) => {
                    if !s.generics.is_empty() {
                        // Store generic struct for later specialization
                        self.generics
                            .struct_defs
                            .insert(s.name.node.clone(), std::rc::Rc::new(s.clone()));
                    }
                    // Always register the struct (including generic ones) so that
                    // struct literals using the base name can find it. For generic
                    // structs, unresolved type params fall back to i64.
                    self.register_struct(s)?;
                    for method in &s.methods {
                        self.register_method(&s.name.node, &method.node)?;
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                    }
                    // Register trait impl for vtable generation
                    if let Some(ref trait_name) = impl_block.trait_name {
                        let mut method_impls = HashMap::new();
                        for method in &impl_block.methods {
                            let fn_name = format!("{}_{}", type_name, method.node.name.node);
                            method_impls.insert(method.node.name.node.clone(), fn_name);
                        }
                        self.register_trait_impl(&type_name, &trait_name.node, method_impls);
                    }
                }
                Item::Trait(trait_def) => {
                    self.register_trait_from_ast(trait_def);
                }
                Item::ExternBlock(extern_block) => {
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    self.register_global(global_def)?;
                }
                Item::TraitAlias(ta) => {
                    let bounds: Vec<String> = ta.bounds.iter().map(|b| b.node.clone()).collect();
                    self.types
                        .trait_aliases
                        .insert(ta.name.node.clone(), bounds);
                }
                Item::Use(_) | Item::TypeAlias(_) | Item::Macro(_) | Item::Error { .. } => {}
            }
        }

        // Build generic function instantiation mapping and register specialized function signatures.
        // Only process concrete instantiations (all type args are resolved, non-generic).
        // Non-concrete instantiations (e.g., make_container$T from inside a generic function body)
        // are skipped — they would produce unresolved generic IR like `@identity$T`.
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                // Skip instantiations with non-concrete type args
                if inst
                    .type_args
                    .iter()
                    .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                {
                    continue;
                }
                if let Some(generic_fn) = self
                    .generics
                    .function_templates
                    .get(&inst.base_name)
                    .cloned()
                {
                    // Build instantiation mapping: base_name -> [(type_args, mangled_name)]
                    self.generics
                        .fn_instantiations
                        .entry(inst.base_name.clone())
                        .or_default()
                        .push((inst.type_args.clone(), inst.mangled_name.clone()));

                    // Register the specialized function signature so call codegen can find it
                    let substitutions: HashMap<String, ResolvedType> = generic_fn
                        .generics
                        .iter()
                        .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                        .zip(inst.type_args.iter())
                        .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                        .collect();

                    let params: Vec<_> = generic_fn
                        .params
                        .iter()
                        .map(|p| {
                            let ty = self.ast_type_to_resolved(&p.ty.node);
                            let concrete_ty = vais_types::substitute_type(&ty, &substitutions);
                            (p.name.node.to_string(), concrete_ty, p.is_mut)
                        })
                        .collect();

                    let ret_type = generic_fn
                        .ret_type
                        .as_ref()
                        .map(|t| {
                            let ty = self.ast_type_to_resolved(&t.node);
                            vais_types::substitute_type(&ty, &substitutions)
                        })
                        .unwrap_or(ResolvedType::Unit);

                    self.types.functions.insert(
                        inst.mangled_name.clone(),
                        FunctionInfo {
                            signature: vais_types::FunctionSig {
                                name: inst.mangled_name.clone(),
                                params,
                                ret: ret_type,
                                is_async: generic_fn.is_async,
                                ..Default::default()
                            },
                            is_extern: false,
                            _extern_abi: None,
                        },
                    );
                }
            }
        }

        // Generate specialized struct types from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Struct = inst.kind {
                if let Some(generic_struct) =
                    self.generics.struct_defs.get(&inst.base_name).cloned()
                {
                    self.generate_specialized_struct_type(&generic_struct, inst, &mut ir)?;
                }
            }
        }

        // Synthesize concrete struct instantiations from function instantiations.
        // The type checker sometimes only records a generic struct instantiation (e.g.,
        // Container$T from inside make_container<T>) without recording the concrete one
        // (Container$i64). We detect this by scanning function instantiation return types
        // and parameter types for references to generic structs with concrete type args.
        {
            // Collect synthetic struct instantiations we need to generate
            let mut synthetic_insts: Vec<vais_types::GenericInstantiation> = Vec::new();
            for inst in instantiations {
                if let vais_types::InstantiationKind::Function = inst.kind {
                    // Only process function instantiations with all-concrete type args
                    if inst
                        .type_args
                        .iter()
                        .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                    {
                        continue;
                    }
                    // Build substitution map for this function instantiation
                    if let Some(generic_fn) =
                        self.generics.function_templates.get(&inst.base_name)
                    {
                        let subst: HashMap<String, ResolvedType> = generic_fn
                            .generics
                            .iter()
                            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                            .zip(inst.type_args.iter())
                            .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                            .collect();

                        // Check return type for generic struct references
                        let types_to_check: Vec<vais_ast::Type> = generic_fn
                            .ret_type
                            .as_ref()
                            .map(|t| vec![t.node.clone()])
                            .unwrap_or_default()
                            .into_iter()
                            .chain(generic_fn.params.iter().map(|p| p.ty.node.clone()))
                            .collect();

                        for ast_ty in &types_to_check {
                            if let Type::Named { name: sname, generics: type_params } = ast_ty {
                                if !type_params.is_empty()
                                    && self.generics.struct_defs.contains_key(sname)
                                {
                                    // Substitute type args to get concrete types
                                    let concrete_args: Vec<ResolvedType> = type_params
                                        .iter()
                                        .map(|tp| {
                                            let resolved = self.ast_type_to_resolved(&tp.node);
                                            vais_types::substitute_type(&resolved, &subst)
                                        })
                                        .collect();

                                    // Only create instantiation if all type args are concrete
                                    let all_concrete = concrete_args.iter().all(|t| {
                                        !matches!(
                                            t,
                                            ResolvedType::Generic(_) | ResolvedType::Var(_)
                                        )
                                    });
                                    if all_concrete {
                                        let mangled =
                                            vais_types::mangle_name(sname, &concrete_args);
                                        if !self
                                            .generics
                                            .generated_structs
                                            .contains_key(&mangled)
                                        {
                                            synthetic_insts.push(
                                                vais_types::GenericInstantiation {
                                                    kind: vais_types::InstantiationKind::Struct,
                                                    base_name: sname.clone(),
                                                    mangled_name: mangled,
                                                    type_args: concrete_args,
                                                    const_args: Vec::new(),
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Generate the synthetic struct instantiations
            for inst in synthetic_insts {
                if let Some(generic_struct) =
                    self.generics.struct_defs.get(&inst.base_name).cloned()
                {
                    self.generate_specialized_struct_type(&generic_struct, &inst, &mut ir)?;
                }
            }
        }

        // Generate non-generic struct types (skip already-emitted specialized generics)
        for (name, info) in &self.types.structs {
            if self.generics.generated_structs.contains_key(name) {
                continue;
            }
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate enum types
        for (name, info) in &self.types.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }

        // Generate union types
        for (name, info) in &self.types.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations (extern functions)
        // Prioritize non-aliased functions (key == name) over aliased ones (key != name)
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .types
            .functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (_, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();

        // Generate specialized functions from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                if let Some(generic_fn) = self
                    .generics
                    .function_templates
                    .get(&inst.base_name)
                    .cloned()
                {
                    body_ir.push_str(&self.generate_specialized_function(&generic_fn, inst)?);
                    body_ir.push('\n');
                }
            }
        }

        // Second pass: generate non-generic function bodies.
        // For generic functions that have NO concrete specialized version (e.g., `identity<T>`
        // only appears inside another generic function body), generate a "fallback" un-specialized
        // version. This maintains backward compatibility with generate_module() behavior — such
        // functions work correctly when called with i64 (the default numeric type).
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    if f.generics.is_empty() {
                        body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                        body_ir.push('\n');
                    } else if !self.generics.fn_instantiations.contains_key(&f.name.node)
                        && !self.generics.generated_functions.contains_key(&f.name.node)
                        && is_function_called_in_module(&f.name.node, module)
                    {
                        // Generic function with no concrete instantiation but IS called
                        // from within another function (e.g., identity<T> called from
                        // double<T>). Generate a fallback version with i64.
                        if let Ok(()) = self.register_function(f) {
                            if let Ok(fn_ir) = self.generate_function_with_span(f, item.span) {
                                body_ir.push_str(&fn_ir);
                                body_ir.push('\n');
                            }
                        }
                    }
                }
                Item::Struct(s) => {
                    if s.generics.is_empty() {
                        for method in &s.methods {
                            body_ir.push_str(&self.generate_method_with_span(
                                &s.name.node,
                                &method.node,
                                method.span,
                            )?);
                            body_ir.push('\n');
                        }
                    }
                }
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &type_name,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Enum(_)
                | Item::Union(_)
                | Item::Use(_)
                | Item::Trait(_)
                | Item::TypeAlias(_)
                | Item::TraitAlias(_)
                | Item::Macro(_)
                | Item::ExternBlock(_)
                | Item::Const(_)
                | Item::Global(_)
                | Item::Error { .. } => {}
            }
        }

        self.emit_string_constants(&mut ir, true);
        self.emit_body_lambdas_vtables(&mut ir, &body_ir);

        // Add WASM runtime if targeting WebAssembly
        if self.target.is_wasm() {
            ir.push_str(&self.generate_wasm_runtime());
        }

        // Add helper functions
        if !matches!(self.target, TargetTriple::Wasm32Unknown) {
            ir.push_str(&self.generate_helper_functions());
        }

        // Add string helper functions if needed
        if self.needs_string_helpers {
            ir.push_str(&self.generate_string_helper_functions());
            if !self.target.is_wasm() {
                ir.push_str(&self.generate_string_extern_declarations());
            }
        }

        // Add contract runtime declarations if any contracts are present
        if !self.contracts.contract_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        // Add debug intrinsics if debug info is enabled
        if self.debug_info.is_enabled() && !self.target.is_wasm() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata
        if !self.target.is_wasm() {
            ir.push_str(&self.debug_info.finalize());
        }

        // Add ABI version metadata
        // ABI version is stored in @__vais_abi_version global constant

        Ok(ir)
    }
}

/// Check whether a function with the given name is called anywhere in the module
/// (excluding its own definition). This is used to determine whether an uninstantiated
/// generic function needs a fallback version -- if no other function calls it, it can
/// be safely omitted.
fn is_function_called_in_module(name: &str, module: &Module) -> bool {
    fn expr_calls(name: &str, expr: &Expr) -> bool {
        match expr {
            Expr::Call { func, args } => {
                if let Expr::Ident(n) = &func.node {
                    if n == name {
                        return true;
                    }
                }
                if expr_calls(name, &func.node) {
                    return true;
                }
                args.iter().any(|a| expr_calls(name, &a.node))
            }
            Expr::Block(stmts) => stmts.iter().any(|s| stmt_calls(name, &s.node)),
            Expr::If { cond, then, else_ } => {
                expr_calls(name, &cond.node)
                    || then.iter().any(|s| stmt_calls(name, &s.node))
                    || else_
                        .as_ref()
                        .map(|e| ifelse_calls(name, e))
                        .unwrap_or(false)
            }
            Expr::Binary { left, right, .. } => {
                expr_calls(name, &left.node) || expr_calls(name, &right.node)
            }
            Expr::Unary { expr: inner, .. } => expr_calls(name, &inner.node),
            Expr::MethodCall { receiver, args, .. } => {
                expr_calls(name, &receiver.node)
                    || args.iter().any(|a| expr_calls(name, &a.node))
            }
            Expr::StaticMethodCall { args, .. } => {
                args.iter().any(|a| expr_calls(name, &a.node))
            }
            Expr::Field { expr: inner, .. } => expr_calls(name, &inner.node),
            Expr::Index { expr: inner, index } => {
                expr_calls(name, &inner.node) || expr_calls(name, &index.node)
            }
            Expr::Ref(inner) | Expr::Deref(inner) | Expr::Try(inner) | Expr::Unwrap(inner)
            | Expr::Await(inner) | Expr::Spawn(inner) | Expr::Yield(inner) => {
                expr_calls(name, &inner.node)
            }
            Expr::Assign { target, value } | Expr::AssignOp { target, value, .. } => {
                expr_calls(name, &target.node) || expr_calls(name, &value.node)
            }
            Expr::Range { start, end, .. } => {
                start.as_ref().map(|e| expr_calls(name, &e.node)).unwrap_or(false)
                    || end.as_ref().map(|e| expr_calls(name, &e.node)).unwrap_or(false)
            }
            Expr::Match { expr: scrutinee, arms } => {
                expr_calls(name, &scrutinee.node)
                    || arms.iter().any(|arm| expr_calls(name, &arm.body.node))
            }
            Expr::Lambda { body, .. } => expr_calls(name, &body.node),
            Expr::Ternary { cond, then, else_ } => {
                expr_calls(name, &cond.node)
                    || expr_calls(name, &then.node)
                    || expr_calls(name, &else_.node)
            }
            Expr::Array(elems) | Expr::Tuple(elems) => {
                elems.iter().any(|e| expr_calls(name, &e.node))
            }
            Expr::StructLit { fields, .. } => {
                fields.iter().any(|(_, e)| expr_calls(name, &e.node))
            }
            Expr::Cast { expr: inner, .. } | Expr::Comptime { body: inner } => {
                expr_calls(name, &inner.node)
            }
            Expr::Loop { iter, body, .. } => {
                iter.as_ref().map(|e| expr_calls(name, &e.node)).unwrap_or(false)
                    || body.iter().any(|s| stmt_calls(name, &s.node))
            }
            Expr::While { condition, body } => {
                expr_calls(name, &condition.node)
                    || body.iter().any(|s| stmt_calls(name, &s.node))
            }
            _ => false,
        }
    }

    fn ifelse_calls(name: &str, ie: &IfElse) -> bool {
        match ie {
            IfElse::Else(stmts) => stmts.iter().any(|s| stmt_calls(name, &s.node)),
            IfElse::ElseIf(cond, then_stmts, else_opt) => {
                expr_calls(name, &cond.node)
                    || then_stmts.iter().any(|s| stmt_calls(name, &s.node))
                    || else_opt
                        .as_ref()
                        .map(|e| ifelse_calls(name, e))
                        .unwrap_or(false)
            }
        }
    }

    fn stmt_calls(name: &str, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e) => expr_calls(name, &e.node),
            Stmt::Let { value, .. } => expr_calls(name, &value.node),
            Stmt::LetDestructure { value, .. } => expr_calls(name, &value.node),
            Stmt::Return(Some(e)) | Stmt::Break(Some(e)) | Stmt::Defer(e) => {
                expr_calls(name, &e.node)
            }
            _ => false,
        }
    }

    fn body_calls(name: &str, body: &FunctionBody) -> bool {
        match body {
            FunctionBody::Expr(e) => expr_calls(name, &e.node),
            FunctionBody::Block(stmts) => stmts.iter().any(|s| stmt_calls(name, &s.node)),
        }
    }

    for item in &module.items {
        match &item.node {
            Item::Function(f) => {
                // Don't check the function's own body
                if f.name.node == name {
                    continue;
                }
                if body_calls(name, &f.body) {
                    return true;
                }
            }
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    if body_calls(name, &method.node.body) {
                        return true;
                    }
                }
            }
            Item::Struct(s) => {
                for method in &s.methods {
                    if body_calls(name, &method.node.body) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}
