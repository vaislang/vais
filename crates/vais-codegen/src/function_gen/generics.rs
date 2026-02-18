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

        // Generate parameters with substituted types
        let params: Vec<_> = generic_fn
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                let concrete_ty = vais_types::substitute_type(&ty, &self.generics.substitutions);
                let llvm_ty = self.type_to_llvm(&concrete_ty);

                // Register parameter as local
                self.fn_ctx.locals.insert(
                    p.name.node.to_string(),
                    LocalVar::param(concrete_ty, p.name.node.to_string()),
                );

                format!("{} %{}", llvm_ty, p.name.node)
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
