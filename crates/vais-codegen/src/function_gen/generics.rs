//! Generic function and struct specialization

use crate::types::{LocalVar, StructInfo};
use crate::{CodeGenerator, CodegenResult};
use std::collections::HashMap;
use vais_ast::{Function, FunctionBody, GenericParamKind, Struct};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate a specialized struct type from a generic struct template
    pub(crate) fn generate_specialized_struct_type(
        &mut self,
        generic_struct: &Struct,
        inst: &vais_types::GenericInstantiation,
        ir: &mut String,
    ) -> CodegenResult<()> {
        // Skip instantiations with non-concrete type args (e.g., Container$T from inside a
        // generic function body where T hasn't been substituted yet). These would produce
        // incorrect LLVM type names like `%Container$T` instead of `%Container$i64`.
        if inst
            .type_args
            .iter()
            .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
        {
            return Ok(());
        }

        // Skip if already generated
        if self
            .generics
            .generated_structs
            .contains_key(&inst.mangled_name)
        {
            return Ok(());
        }
        self.generics
            .generated_structs
            .insert(inst.mangled_name.to_string(), true); // explicit to_string instead of clone

        // Create substitution map from generic params to concrete types
        // Filter out lifetime params (they don't have runtime representation)
        let type_params: Vec<_> = generic_struct
            .generics
            .iter()
            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
            .collect();
        let substitutions: HashMap<String, ResolvedType> = type_params
            .iter()
            .zip(inst.type_args.iter())
            .map(|(g, t)| (g.name.node.to_string(), t.clone())) // clone required: type_args is &[ResolvedType]
            .collect();

        // Save and set generic substitutions
        let old_subst = std::mem::replace(&mut self.generics.substitutions, substitutions);

        // Generate field types with substitutions
        let fields: Vec<(String, ResolvedType)> = generic_struct
            .fields
            .iter()
            .map(|f| {
                let ty = self.ast_type_to_resolved(&f.ty.node);
                let concrete_ty = vais_types::substitute_type(&ty, &self.generics.substitutions);
                (f.name.node.to_string(), concrete_ty)
            })
            .collect();

        let llvm_fields: Vec<String> = fields
            .iter()
            .map(|(_, ty)| {
                let llvm_ty = self.type_to_llvm(ty);
                // void is not valid as a struct field type — use i8 for Unit fields
                if llvm_ty == "void" { "i8".to_string() } else { llvm_ty }
            })
            .collect();

        write_ir!(
            ir,
            "%{} = type {{ {} }}",
            inst.mangled_name,
            llvm_fields.join(", ")
        );

        // Register the specialized struct
        let struct_info = StructInfo {
            _name: inst.mangled_name.to_string(),
            fields,
            _repr_c: false,
            _invariants: Vec::new(),
        };
        self.types
            .structs
            .insert(inst.mangled_name.to_string(), struct_info);

        // Register a name mapping from base name to mangled name
        // so struct literals and field accesses in generic impl methods can resolve it
        self.generics
            .struct_aliases
            .insert(inst.base_name.to_string(), inst.mangled_name.to_string());

        // Restore old substitutions
        self.generics.substitutions = old_subst;

        Ok(())
    }

    /// Maximum monomorphization depth to prevent infinite recursive instantiation.
    ///
    /// This guards against patterns like `F foo<T>() -> Wrapper<Wrapper<T>>` which
    /// could trigger unbounded instantiation chains. The type checker normally
    /// prevents these, but this is a safety net at the codegen level.
    const MAX_MONOMORPHIZATION_DEPTH: usize = 64;

    /// Generate a specialized function from a generic function template
    pub(crate) fn generate_specialized_function(
        &mut self,
        generic_fn: &Function,
        inst: &vais_types::GenericInstantiation,
    ) -> CodegenResult<String> {
        // Use stacker to handle deep specialization chains.
        // Catch panics from stack overflow and convert to recoverable error.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            stacker::maybe_grow(4 * 1024 * 1024, 16 * 1024 * 1024, || {
                self.enter_type_recursion("generate_specialized_function")?;
                let result = self.generate_specialized_function_inner(generic_fn, inst);
                self.exit_type_recursion();
                result
            })
        }));
        match result {
            Ok(r) => r,
            Err(_) => {
                eprintln!("[WARN] Stack overflow during specialization of '{}' — skipping", inst.mangled_name);
                Ok(String::new()) // Return empty IR, function will be undefined
            }
        }
    }

    #[inline(never)]
    fn generate_specialized_function_inner(
        &mut self,
        generic_fn: &Function,
        inst: &vais_types::GenericInstantiation,
    ) -> CodegenResult<String> {
        // Skip instantiations with non-concrete type args (e.g., make_container$T from inside
        // a generic function body). These would produce unresolved generic IR.
        if inst
            .type_args
            .iter()
            .any(|t| matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)))
        {
            return Ok(String::new());
        }

        // Skip specialization for deeply nested struct types to prevent stack overflow.
        // store_typed/load_typed have been extracted to #[inline(never)] methods, so the
        // threshold is relaxed from >2 fields to >6 fields. Only skip if the struct has
        // deeply nested Named fields (depth >= 2), as single-level Named fields are fine.
        let has_complex_type = inst.type_args.iter().any(|t| {
            if let ResolvedType::Named { name, .. } = t {
                let fields = self
                    .types
                    .structs
                    .get(name)
                    .map(|s| &s.fields[..])
                    .unwrap_or(&[]);
                let generic_fields = self
                    .generics
                    .struct_defs
                    .get(name)
                    .map(|s| s.fields.len())
                    .unwrap_or(0);
                // Check for deeply nested Named types (depth 2: Named field whose own fields
                // are also Named). Single-level Named fields (e.g., Vec<Point>) are allowed.
                let has_deeply_nested = fields.iter().any(|(_, ty)| {
                    if let ResolvedType::Named {
                        name: inner_name, ..
                    } = ty
                    {
                        self.types
                            .structs
                            .get(inner_name)
                            .map(|s| {
                                s.fields
                                    .iter()
                                    .any(|(_, ft)| matches!(ft, ResolvedType::Named { .. }))
                            })
                            .unwrap_or(false)
                    } else {
                        false
                    }
                });
                // Complex if: >6 fields, OR deeply nested Named fields (depth >= 2)
                fields.len() > 6 || generic_fields > 6 || has_deeply_nested
            } else {
                false
            }
        });
        if has_complex_type {
            self.generics
                .generated_functions
                .insert(inst.mangled_name.clone(), true);
            return Ok(String::new());
        }

        // Skip if already generated
        if self
            .generics
            .generated_functions
            .contains_key(&inst.mangled_name)
        {
            return Ok(String::new());
        }

        // Guard against infinite recursive monomorphization.
        // Count how many specializations have been generated for this base function
        // name. If it exceeds the limit, it's likely an unbounded instantiation chain.
        let specialization_count = self
            .generics
            .generated_functions
            .keys()
            .filter(|k| k.starts_with(&inst.base_name))
            .count();
        if specialization_count >= Self::MAX_MONOMORPHIZATION_DEPTH {
            return Err(crate::CodegenError::RecursionLimitExceeded(format!(
                "Monomorphization depth limit ({}) exceeded for generic function '{}'. \
                 This may indicate infinite recursive type instantiation.",
                Self::MAX_MONOMORPHIZATION_DEPTH,
                inst.base_name
            )));
        }

        self.generics
            .generated_functions
            .insert(inst.mangled_name.clone(), true);

        // Create substitution map from generic params to concrete types
        // Filter out lifetime params (they don't have runtime representation)
        // NOTE: For method specializations, also sets Self → concrete struct type
        let type_params: Vec<_> = generic_fn
            .generics
            .iter()
            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
            .collect();
        let mut substitutions: HashMap<String, ResolvedType> = type_params
            .iter()
            .zip(inst.type_args.iter())
            .map(|(g, t)| (g.name.node.to_string(), t.clone()))
            .collect();

        // For method specializations (base_name contains '_'), set Self → concrete struct type
        if let Some(underscore_pos) = inst.base_name.find('_') {
            let struct_name = &inst.base_name[..underscore_pos];
            if self.types.structs.contains_key(struct_name)
                || self.generics.struct_defs.contains_key(struct_name)
                || self.types.enums.contains_key(struct_name)
            {
                substitutions.insert(
                    "Self".to_string(),
                    ResolvedType::Named {
                        name: struct_name.to_string(),
                        generics: inst.type_args.clone(),
                    },
                );
                // If the method has no generics of its own (e.g., Vec<T>.push where T comes
                // from the struct), use the struct's generic parameters for substitution.
                if type_params.is_empty() && !inst.type_args.is_empty() {
                    if let Some(struct_def) = self.generics.struct_defs.get(struct_name) {
                        let struct_type_params: Vec<_> = struct_def
                            .generics
                            .iter()
                            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                            .collect();
                        for (g, t) in struct_type_params.iter().zip(inst.type_args.iter()) {
                            substitutions.insert(g.name.node.to_string(), t.clone());
                        }
                    }
                }
            }
        }

        // Save and set generic substitutions
        let old_subst = std::mem::replace(&mut self.generics.substitutions, substitutions);

        self.initialize_function_state(&inst.mangled_name);

        // Collect param info (name, concrete type) first — needed for both signature and alloca
        let param_infos: Vec<(String, ResolvedType)> = generic_fn
            .params
            .iter()
            .map(|p| {
                let name = p.name.node.to_string();
                // Special case: `self` parameter has Type::Infer in AST, which resolves
                // to Unknown. Use the Self substitution directly instead.
                if name == "self" {
                    if let Some(self_ty) = self.generics.substitutions.get("Self").cloned() {
                        return (name, self_ty);
                    }
                }
                let ty = self.ast_type_to_resolved(&p.ty.node);
                let concrete_ty = vais_types::substitute_type(&ty, &self.generics.substitutions);
                (name, concrete_ty)
            })
            .collect();

        // Build LLVM parameter list and register locals (initially as SSA params)
        let params: Vec<String> = param_infos
            .iter()
            .map(|(name, concrete_ty)| {
                let llvm_ty = self.type_to_llvm(concrete_ty);
                // void is invalid as a parameter type — use i8 for Unit parameters
                let llvm_ty = if llvm_ty == "void" { "i8".to_string() } else { llvm_ty };
                let llvm_name = crate::helpers::sanitize_param_name(name);
                // For struct-type self parameter, pass as pointer (matches unspecialized convention)
                let (actual_ty, actual_resolved) = if name == "self" && matches!(concrete_ty, ResolvedType::Named { .. }) {
                    (format!("{}*", llvm_ty), ResolvedType::Ref(Box::new(concrete_ty.clone())))
                } else {
                    (llvm_ty, concrete_ty.clone())
                };
                // Register parameter as local initially (may be updated below for struct params)
                self.fn_ctx.locals.insert(
                    name.to_string(),
                    LocalVar::param(actual_resolved, llvm_name.to_string()),
                );
                format!("{} %{}", actual_ty, llvm_name)
            })
            .collect();

        let ret_type = if let Some(t) = generic_fn.ret_type.as_ref() {
            let ty = self.ast_type_to_resolved(&t.node);
            vais_types::substitute_type(&ty, &self.generics.substitutions)
        } else {
            // Bug fix: try multiple lookup strategies in order since specialized
            // functions may be registered under the mangled name (e.g., "Cell_get$bool")
            // rather than the base name (e.g., "Cell_get").
            //
            // Strategy 1: base name (original behavior)
            let by_base = self
                .types
                .functions
                .get(&generic_fn.name.node)
                .map(|info| info.signature.ret.clone());

            // Strategy 2: mangled name (specialized function registration)
            let by_mangled = || {
                self.types
                    .functions
                    .get(&inst.mangled_name)
                    .map(|info| info.signature.ret.clone())
            };

            // Strategy 3: resolved_function_sigs from the type checker (base name, then mangled)
            let by_resolved = || {
                self.types
                    .resolved_function_sigs
                    .get(&generic_fn.name.node)
                    .map(|sig| sig.ret.clone())
                    .or_else(|| {
                        self.types
                            .resolved_function_sigs
                            .get(&inst.mangled_name)
                            .map(|sig| sig.ret.clone())
                    })
            };

            // Use the first non-Unit result, or fall back to Unit
            by_base
                .filter(|t| *t != ResolvedType::Unit)
                .or_else(by_mangled)
                .or_else(by_resolved)
                .unwrap_or(ResolvedType::Unit)
        };

        let ret_llvm = self.type_to_llvm(&ret_type);

        let mut ir = format!(
            "; Specialized function: {} from {}<{}>\n",
            inst.mangled_name,
            inst.base_name,
            inst.type_args
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        write_ir!(
            ir,
            "define {} @{}({}) {{",
            ret_llvm,
            inst.mangled_name,
            params.join(", ")
        );
        ir.push_str("entry:\n");
        self.fn_ctx.current_block = "entry".to_string();

        // For struct parameters, ensure field access (GEP) works correctly.
        // - `self` passed as pointer: use the pointer directly (no copy needed)
        // - other struct params passed by value: alloca+store so GEP works
        for (name, concrete_ty) in &param_infos {
            if matches!(concrete_ty, ResolvedType::Named { .. }) {
                let llvm_ty = self.type_to_llvm(concrete_ty);
                let src_llvm_name = crate::helpers::sanitize_param_name(name);

                if name == "self" {
                    // self is already a pointer (%Vec* %self) — use directly
                    // This ensures mutations (self.len += 1) affect the original struct
                    self.fn_ctx.locals.insert(
                        name.to_string(),
                        LocalVar::ssa(concrete_ty.clone(), format!("%{}", src_llvm_name)),
                    );
                } else {
                    let param_ptr = format!("__{}_ptr", name);
                    write_ir!(ir, "  %{} = alloca {}", param_ptr, llvm_ty);
                    write_ir!(
                        ir,
                        "  store {} %{}, {}* %{}",
                        llvm_ty,
                        src_llvm_name,
                        llvm_ty,
                        param_ptr
                    );
                    // Update locals to use the alloca pointer as an SSA value so field access works
                    self.fn_ctx.locals.insert(
                        name.to_string(),
                        LocalVar::ssa(concrete_ty.clone(), format!("%{}", param_ptr)),
                    );
                }
            }
        }

        // Set current return type so R (return) statements emit correct type
        self.fn_ctx.current_return_type = Some(ret_type.clone());

        // Generate function body
        let mut counter = 0;
        match &generic_fn.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut counter)?;
                ir.push_str(&expr_ir);
                // Bug 2 safety check: if ret_type resolved to Unit but the generated
                // function signature uses a non-void return type (detected via ret_llvm),
                // prefer the signature type to avoid LLVM IR validation failures.
                // Both ret_type and ret_llvm are derived from the same source, so this
                // check catches any future divergence between them.
                let effective_ret_llvm = &ret_llvm;
                if ret_type == ResolvedType::Unit && effective_ret_llvm != "void" {
                    // Signature says non-void but ret_type says Unit — trust the signature.
                    // Attempt to return the body value as the declared type.
                    write_ir!(ir, "  ret {} {}", effective_ret_llvm, value);
                } else if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    let loaded = format!("%ret.{}", counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        ret_llvm,
                        ret_llvm,
                        value
                    );
                    write_ir!(ir, "  ret {} {}", ret_llvm, loaded);
                } else {
                    // Coerce body value to return type if needed (e.g., i64 → double)
                    let coerced = self.coerce_specialized_return(&value, &ret_llvm, &ret_type, &mut counter, &mut ir);
                    write_ir!(ir, "  ret {} {}", ret_llvm, coerced);
                }
            }
            FunctionBody::Block(stmts) => {
                // Use generate_block_stmts (visitor-based) which produces the
                // single-pointer alloca pattern consistent with generate_ident_expr.
                // The old generate_block path creates double-pointer allocas for
                // struct locals which causes type mismatches on return.
                let (value, block_ir, terminated) =
                    self.generate_block_stmts(stmts, &mut counter)?;
                ir.push_str(&block_ir);
                // If the block already contains a terminator (e.g., explicit `R 42`),
                // do not emit a duplicate ret instruction.
                if !terminated {
                    // Bug 2 safety check: if ret_type resolved to Unit but the generated
                    // function signature uses a non-void return type, trust the signature.
                    if ret_type == ResolvedType::Unit && ret_llvm != "void" {
                        write_ir!(ir, "  ret {} {}", ret_llvm, value);
                    } else if ret_type == ResolvedType::Unit {
                        ir.push_str("  ret void\n");
                    } else if matches!(ret_type, ResolvedType::Named { .. }) {
                        if self.is_block_result_value(stmts) {
                            // Check if value needs coercion (e.g., i64 from generic call
                            // but ret_llvm is a struct type like %Vec$u64)
                            let val_ty = self.llvm_type_of(&value);
                            if val_ty == "i64" && ret_llvm.starts_with('%') && !ret_llvm.ends_with('*') {
                                let coerced = self.coerce_specialized_return(&value, &ret_llvm, &ret_type, &mut counter, &mut ir);
                                write_ir!(ir, "  ret {} {}", ret_llvm, coerced);
                            } else {
                                write_ir!(ir, "  ret {} {}", ret_llvm, value);
                            }
                        } else {
                            let loaded = format!("%ret.{}", counter);
                            write_ir!(
                                ir,
                                "  {} = load {}, {}* {}",
                                loaded,
                                ret_llvm,
                                ret_llvm,
                                value
                            );
                            write_ir!(ir, "  ret {} {}", ret_llvm, loaded);
                        }
                    } else {
                        // Coerce body value to return type if needed
                        let coerced = self.coerce_specialized_return(&value, &ret_llvm, &ret_type, &mut counter, &mut ir);
                        write_ir!(ir, "  ret {} {}", ret_llvm, coerced);
                    }
                }
            }
        }

        // Splice entry-block allocas into the function
        self.splice_entry_allocas(&mut ir);

        ir.push_str("}\n");

        // Restore state
        self.generics.substitutions = old_subst;
        self.fn_ctx.current_function = None;

        Ok(ir)
    }

    /// Coerce a value to match the specialized function return type.
    /// Handles cases like i64→double, i64→float when generic functions
    /// return i64 internally but the specialized version declares a concrete return type.
    fn coerce_specialized_return(
        &mut self,
        value: &str,
        ret_llvm: &str,
        _ret_type: &ResolvedType,
        counter: &mut usize,
        ir: &mut String,
    ) -> String {
        // If value looks like an i64 but return type is float/double, bitcast
        if ret_llvm == "double" {
            // Assume value is i64 from generic body — bitcast to double
            let tmp = self.next_temp(counter);
            write_ir!(ir, "  {} = bitcast i64 {} to double", tmp, value);
            return tmp;
        }
        if ret_llvm == "float" {
            let tmp1 = self.next_temp(counter);
            let tmp2 = self.next_temp(counter);
            write_ir!(ir, "  {} = bitcast i64 {} to double", tmp1, value);
            write_ir!(ir, "  {} = fptrunc double {} to float", tmp2, tmp1);
            return tmp2;
        }
        // For small int types (i8, i16, i32), truncate from i64
        if ret_llvm == "i8" || ret_llvm == "i16" || ret_llvm == "i32" {
            let tmp = self.next_temp(counter);
            write_ir!(ir, "  {} = trunc i64 {} to {}", tmp, value, ret_llvm);
            return tmp;
        }
        // For struct return types where body returns i64 (generic erasure)
        // Use inttoptr+load to reinterpret the i64 as a struct pointer
        let val_llvm = self.llvm_type_of(value);
        if val_llvm == "i64"
            && ret_llvm.starts_with('%')
            && !ret_llvm.ends_with('*')
            && value.starts_with('%')
        {
            let tmp_ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = inttoptr i64 {} to {}*", tmp_ptr, value, ret_llvm);
            let loaded = self.next_temp(counter);
            write_ir!(ir, "  {} = load {}, {}* {}", loaded, ret_llvm, ret_llvm, tmp_ptr);
            return loaded;
        }
        // Default: return as-is
        value.to_string()
    }
}
