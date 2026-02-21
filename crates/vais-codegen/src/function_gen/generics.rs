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

        let llvm_fields: Vec<String> = fields.iter().map(|(_, ty)| self.type_to_llvm(ty)).collect();

        ir.push_str(&format!(
            "%{} = type {{ {} }}\n",
            inst.mangled_name,
            llvm_fields.join(", ")
        ));

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

    /// Generate a specialized function from a generic function template
    pub(crate) fn generate_specialized_function(
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

        // Skip if already generated
        if self
            .generics
            .generated_functions
            .contains_key(&inst.mangled_name)
        {
            return Ok(String::new());
        }
        self.generics
            .generated_functions
            .insert(inst.mangled_name.clone(), true);

        // Create substitution map from generic params to concrete types
        // Filter out lifetime params (they don't have runtime representation)
        let type_params: Vec<_> = generic_fn
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

        self.initialize_function_state(&inst.mangled_name);

        // Collect param info (name, concrete type) first — needed for both signature and alloca
        let param_infos: Vec<(String, ResolvedType)> = generic_fn
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                let concrete_ty = vais_types::substitute_type(&ty, &self.generics.substitutions);
                (p.name.node.to_string(), concrete_ty)
            })
            .collect();

        // Build LLVM parameter list and register locals (initially as SSA params)
        let params: Vec<String> = param_infos
            .iter()
            .map(|(name, concrete_ty)| {
                let llvm_ty = self.type_to_llvm(concrete_ty);
                // Register parameter as local initially (may be updated below for struct params)
                self.fn_ctx.locals.insert(
                    name.to_string(),
                    LocalVar::param(concrete_ty.clone(), name.to_string()),
                );
                format!("{} %{}", llvm_ty, name)
            })
            .collect();

        let ret_type = if let Some(t) = generic_fn.ret_type.as_ref() {
            let ty = self.ast_type_to_resolved(&t.node);
            vais_types::substitute_type(&ty, &self.generics.substitutions)
        } else {
            self.types
                .functions
                .get(&generic_fn.name.node)
                .map(|info| &info.signature.ret)
                .cloned() // explicit single clone at end
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
        ir.push_str(&format!(
            "define {} @{}({}) {{\n",
            ret_llvm,
            inst.mangled_name,
            params.join(", ")
        ));
        ir.push_str("entry:\n");
        self.fn_ctx.current_block = "entry".to_string();

        // For struct-by-value parameters, alloca+store so field access (GEP) works correctly.
        // Without this, the param is an SSA struct value and GEP requires a pointer.
        for (name, concrete_ty) in &param_infos {
            if matches!(concrete_ty, ResolvedType::Named { .. }) {
                let llvm_ty = self.type_to_llvm(concrete_ty);
                let param_ptr = format!("__{}_ptr", name);
                ir.push_str(&format!("  %{} = alloca {}\n", param_ptr, llvm_ty));
                ir.push_str(&format!(
                    "  store {} %{}, {}* %{}\n",
                    llvm_ty, name, llvm_ty, param_ptr
                ));
                // Update locals to use the alloca pointer as an SSA value so field access works
                self.fn_ctx.locals.insert(
                    name.to_string(),
                    LocalVar::ssa(concrete_ty.clone(), format!("%{}", param_ptr)),
                );
            }
        }

        // Generate function body
        let mut counter = 0;
        match &generic_fn.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut counter)?;
                ir.push_str(&expr_ir);
                if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, ret_llvm, ret_llvm, value
                    ));
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, loaded));
                } else {
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, value));
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir) = self.generate_block(stmts, &mut counter)?;
                ir.push_str(&block_ir);
                if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, ret_llvm, ret_llvm, value
                    ));
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, loaded));
                } else {
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, value));
                }
            }
        }

        ir.push_str("}\n");

        // Restore state
        self.generics.substitutions = old_subst;
        self.fn_ctx.current_function = None;

        Ok(ir)
    }
}
