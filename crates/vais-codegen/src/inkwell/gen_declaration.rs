//! Declaration phase code generation.
//!
//! Handles function declarations, struct/enum/union definitions,
//! extern blocks, and const definitions.


use inkwell::types::{BasicMetadataTypeEnum, BasicType, StructType};
use inkwell::values::{BasicValueEnum, FunctionValue};

use vais_ast::{self as ast, Expr};

use super::generator::InkwellCodeGenerator;
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn declare_function(&mut self, func: &ast::Function) -> CodegenResult<FunctionValue<'ctx>> {
        let fn_name = &func.name.node;

        // If already declared, return existing
        if let Some(existing) = self.functions.get(fn_name) {
            return Ok(*existing);
        }

        // Build parameter types
        let mut param_types = Vec::new();
        for param in &func.params {
            let resolved = self.ast_type_to_resolved(&param.ty.node);
            let param_type = self.type_mapper.map_type(&resolved);
            param_types.push(param_type.into());
        }

        // Build return type
        let ret_type = if let Some(ref ty) = func.ret_type {
            let resolved = self.ast_type_to_resolved(&ty.node);
            self.type_mapper.map_type(&resolved)
        } else {
            // Default to i64 if return type is not specified
            self.context.i64_type().into()
        };

        // Create function type
        let fn_type = ret_type.fn_type(&param_types, false);

        // Add function to module
        let fn_value = self.module.add_function(fn_name, fn_type, None);

        // Store in functions map
        self.functions.insert(fn_name.clone(), fn_value);

        // Set parameter names
        for (i, param) in func.params.iter().enumerate() {
            fn_value.get_nth_param(i as u32).map(|p| {
                p.set_name(&param.name.node);
            });
        }

        Ok(fn_value)
    }

    pub(super) fn define_struct(&mut self, s: &ast::Struct) -> CodegenResult<StructType<'ctx>> {
        let struct_name = &s.name.node;

        // If already defined, return existing
        if let Some(existing) = self.generated_structs.get(struct_name) {
            return Ok(*existing);
        }

        // Build field types
        let mut field_types = Vec::new();
        let mut field_names = Vec::new();
        let mut field_type_names = Vec::new();

        for field in &s.fields {
            let resolved = self.ast_type_to_resolved(&field.ty.node);
            let field_type = self.type_mapper.map_type(&resolved);
            field_types.push(field_type);
            field_names.push(field.name.node.clone());

            // Extract type name for nested field access
            let type_name = match &field.ty.node {
                ast::Type::Named { name, .. } => name.clone(),
                _ => String::from("unknown"),
            };
            field_type_names.push((field.name.node.clone(), type_name));
        }

        // Create opaque struct
        let struct_type = self.context.opaque_struct_type(struct_name);

        // Set body
        struct_type.set_body(&field_types, false);

        // Store in maps
        self.generated_structs
            .insert(struct_name.clone(), struct_type);
        self.struct_fields.insert(struct_name.clone(), field_names);
        self.struct_field_type_names
            .insert(struct_name.clone(), field_type_names);

        Ok(struct_type)
    }

    pub(super) fn define_enum(&mut self, e: &ast::Enum) -> CodegenResult<StructType<'ctx>> {
        let enum_name = &e.name.node;

        // If already defined, return existing
        if let Some(existing) = self.generated_structs.get(enum_name) {
            return Ok(*existing);
        }

        // Store variant tags
        for (i, variant) in e.variants.iter().enumerate() {
            self.enum_variants.insert(
                (enum_name.clone(), variant.name.node.clone()),
                i as i32,
            );
        }

        // Create enum as struct: { i32 tag, i64 data }
        let enum_type = self
            .context
            .struct_type(&[self.context.i32_type().into()], false);

        self.generated_structs
            .insert(enum_name.clone(), enum_type);

        Ok(enum_type)
    }

    pub(super) fn declare_extern_block(&mut self, extern_block: &ast::ExternBlock) -> CodegenResult<()> {
        for func in &extern_block.functions {
            let fn_name = &func.name.node;

            // Build parameter types
            let param_types: Vec<BasicMetadataTypeEnum> = func
                .params
                .iter()
                .map(|p| {
                    let resolved = self.ast_type_to_resolved(&p.ty.node);
                    self.type_mapper.map_type(&resolved).into()
                })
                .collect();

            // Build return type
            let ret_type = if let Some(ref ty) = func.ret_type {
                let resolved = self.ast_type_to_resolved(&ty.node);
                self.type_mapper.map_type(&resolved)
            } else {
                self.context.i64_type().into()
            };

            // Create function type
            let fn_type = ret_type.fn_type(&param_types, false);

            // Add function to module
            let fn_value = self.module.add_function(fn_name, fn_type, None);

            // Store in functions map
            self.functions.insert(fn_name.clone(), fn_value);
        }
        Ok(())
    }

    pub(super) fn define_union(&mut self, u: &ast::Union) -> CodegenResult<StructType<'ctx>> {
        let union_name = &u.name.node;

        // If already defined, return existing
        if let Some(existing) = self.generated_structs.get(union_name) {
            return Ok(*existing);
        }

        // Union is represented as a struct with a single byte array field
        // sized to the largest variant
        let union_type = self
            .context
            .struct_type(&[self.context.i64_type().into()], false);

        self.generated_structs
            .insert(union_name.clone(), union_type);

        Ok(union_type)
    }

    pub(super) fn define_const(&mut self, const_def: &ast::ConstDef) -> CodegenResult<()> {
        let const_name = &const_def.name.node;
        let value = self.evaluate_const_expr(&const_def.value.node)?;
        self.constants.insert(const_name.clone(), value);
        Ok(())
    }

    pub(super) fn evaluate_const_expr(&self, expr: &Expr) -> CodegenResult<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Int(i) => Ok(self.context.i64_type().const_int(*i as u64, true).into()),
            Expr::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            Expr::String(s) => {
                // String literal as const requires global
                let bytes = s.as_bytes();
                let mut bytes_with_null = bytes.to_vec();
                bytes_with_null.push(0);
                let str_val = self.context.const_string(&bytes_with_null, false);
                Ok(str_val.into())
            }
            Expr::Bool(b) => Ok(self
                .context
                .bool_type()
                .const_int(if *b { 1 } else { 0 }, false)
                .into()),
            Expr::Binary { left, op, right } => {
                let lhs_val = self.evaluate_const_expr(&left.node)?;
                let rhs_val = self.evaluate_const_expr(&right.node)?;
                // Simple int arithmetic only
                let lhs_int = lhs_val.into_int_value();
                let rhs_int = rhs_val.into_int_value();
                use vais_ast::BinOp;
                let result = match op {
                    BinOp::Add => lhs_int.const_add(rhs_int),
                    BinOp::Sub => lhs_int.const_sub(rhs_int),
                    BinOp::Mul => lhs_int.const_mul(rhs_int),
                    // Division not currently supported in const context
                    _ => {
                        return Err(CodegenError::Unsupported(format!(
                            "Const expr binary op: {:?}",
                            op
                        )))
                    }
                };
                Ok(result.into())
            }
            _ => Err(CodegenError::Unsupported(format!(
                "Const expr: {:?}",
                expr
            ))),
        }
    }
}
