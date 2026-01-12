//! Expression type inference
//!
//! Infers types of expressions and validates operator usage.

use aoel_ast::{BinaryOp, Expr, LiteralKind, Type, UnaryOp};
use aoel_lexer::Span;

use crate::error::TypeCheckError;
use crate::symbol::{SymbolKind, SymbolTable};
use crate::types::{
    bool_type, float64_type, int64_type, is_bool_type, is_numeric_type, string_type,
    type_to_string, void_type,
};

/// Type inferrer for expressions
pub struct TypeInferrer<'a> {
    symbols: &'a SymbolTable,
}

impl<'a> TypeInferrer<'a> {
    pub fn new(symbols: &'a SymbolTable) -> Self {
        Self { symbols }
    }

    /// Infer the type of an expression
    pub fn infer(&self, expr: &Expr) -> Result<Type, TypeCheckError> {
        match expr {
            Expr::Literal(lit) => Ok(self.infer_literal(lit)),
            Expr::Ident(ident) => self.infer_ident(ident),
            Expr::ExternalRef(ext) => Ok(self.infer_external_ref(ext)),
            Expr::FieldAccess(access) => self.infer_field_access(access),
            Expr::Binary(binary) => self.infer_binary(binary),
            Expr::Unary(unary) => self.infer_unary(unary),
            Expr::Call(call) => self.infer_call(call),
            Expr::Index(index) => self.infer_index(index),
            Expr::Grouped(grouped) => self.infer(&grouped.inner),
        }
    }

    fn infer_literal(&self, lit: &aoel_ast::Literal) -> Type {
        match &lit.kind {
            LiteralKind::Integer(_) => int64_type(lit.span),
            LiteralKind::Float(_) => float64_type(lit.span),
            LiteralKind::String(_) => string_type(lit.span),
            LiteralKind::Bool(_) => bool_type(lit.span),
            LiteralKind::Regex(_) => string_type(lit.span),
            LiteralKind::Duration(_) => string_type(lit.span), // Duration as string for now
            LiteralKind::Size(_) => string_type(lit.span),     // Size as string for now
            LiteralKind::Void => void_type(lit.span),
        }
    }

    fn infer_ident(&self, ident: &aoel_ast::Ident) -> Result<Type, TypeCheckError> {
        if let Some(symbol) = self.symbols.lookup(&ident.name) {
            if let Some(ty) = &symbol.ty {
                return Ok(ty.clone());
            }
        }
        // If we can't find a type, return a placeholder (will be caught by reference checking)
        Ok(void_type(ident.span))
    }

    fn infer_external_ref(&self, ext: &aoel_ast::ExternalRef) -> Type {
        // External references have unknown type at this stage
        // They would need to be resolved against a schema registry
        Type::Ref(ext.clone())
    }

    fn infer_field_access(
        &self,
        access: &aoel_ast::FieldAccess,
    ) -> Result<Type, TypeCheckError> {
        // Handle input.field and output.field patterns
        if let Expr::Ident(base) = &access.base {
            match base.name.as_str() {
                "input" => {
                    if let Some(symbol) = self.symbols.lookup(&access.field.name) {
                        if matches!(symbol.kind, SymbolKind::InputField) {
                            if let Some(ty) = &symbol.ty {
                                return Ok(ty.clone());
                            }
                        }
                    }
                }
                "output" => {
                    if let Some(symbol) = self.symbols.lookup(&access.field.name) {
                        if matches!(symbol.kind, SymbolKind::OutputField) {
                            if let Some(ty) = &symbol.ty {
                                return Ok(ty.clone());
                            }
                        }
                    }
                }
                "INPUT" => {
                    // Same as "input" (case variation in FLOW block)
                    if let Some(symbol) = self.symbols.lookup(&access.field.name) {
                        if matches!(symbol.kind, SymbolKind::InputField) {
                            if let Some(ty) = &symbol.ty {
                                return Ok(ty.clone());
                            }
                        }
                    }
                }
                "OUTPUT" => {
                    // Same as "output" (case variation in FLOW block)
                    if let Some(symbol) = self.symbols.lookup(&access.field.name) {
                        if matches!(symbol.kind, SymbolKind::OutputField) {
                            if let Some(ty) = &symbol.ty {
                                return Ok(ty.clone());
                            }
                        }
                    }
                }
                _ => {
                    // Could be a node reference or struct field access
                    let base_type = self.infer(&access.base)?;
                    if let Type::Struct(s) = &base_type {
                        for field in &s.fields {
                            if field.name.name == access.field.name {
                                return Ok(field.ty.clone());
                            }
                        }
                    }
                }
            }
        } else {
            // Nested field access
            let base_type = self.infer(&access.base)?;
            if let Type::Struct(s) = &base_type {
                for field in &s.fields {
                    if field.name.name == access.field.name {
                        return Ok(field.ty.clone());
                    }
                }
            }
        }

        // Return void as placeholder; actual error is caught by reference checking
        Ok(void_type(access.span))
    }

    fn infer_binary(&self, binary: &aoel_ast::BinaryExpr) -> Result<Type, TypeCheckError> {
        let left_type = self.infer(&binary.left)?;
        let right_type = self.infer(&binary.right)?;
        let span = binary.span;

        match binary.op {
            // Arithmetic operators: numeric -> numeric
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                if !is_numeric_type(&left_type) {
                    return Err(TypeCheckError::InvalidOperandType {
                        operator: binary.op.as_str().to_string(),
                        expected: "numeric".to_string(),
                        found: type_to_string(&left_type),
                        span: binary.left.span(),
                    });
                }
                if !is_numeric_type(&right_type) {
                    return Err(TypeCheckError::InvalidOperandType {
                        operator: binary.op.as_str().to_string(),
                        expected: "numeric".to_string(),
                        found: type_to_string(&right_type),
                        span: binary.right.span(),
                    });
                }
                // Return the wider type (simplified: always float64 if either is float)
                if matches!(&left_type, Type::Primitive(p) if matches!(p.kind, aoel_ast::PrimitiveKind::Float32 | aoel_ast::PrimitiveKind::Float64))
                    || matches!(&right_type, Type::Primitive(p) if matches!(p.kind, aoel_ast::PrimitiveKind::Float32 | aoel_ast::PrimitiveKind::Float64))
                {
                    Ok(float64_type(span))
                } else {
                    Ok(int64_type(span))
                }
            }

            // Comparison operators: compatible types -> BOOL
            BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Lte | BinaryOp::Gte => {
                // We allow comparison of compatible types
                Ok(bool_type(span))
            }

            // Logical operators: BOOL -> BOOL
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor | BinaryOp::Implies => {
                if !is_bool_type(&left_type) {
                    return Err(TypeCheckError::InvalidOperandType {
                        operator: binary.op.as_str().to_string(),
                        expected: "BOOL".to_string(),
                        found: type_to_string(&left_type),
                        span: binary.left.span(),
                    });
                }
                if !is_bool_type(&right_type) {
                    return Err(TypeCheckError::InvalidOperandType {
                        operator: binary.op.as_str().to_string(),
                        expected: "BOOL".to_string(),
                        found: type_to_string(&right_type),
                        span: binary.right.span(),
                    });
                }
                Ok(bool_type(span))
            }

            // IN operator: element IN collection -> BOOL
            BinaryOp::In => Ok(bool_type(span)),

            // MATCH operator: STRING MATCH pattern -> BOOL
            BinaryOp::Match => Ok(bool_type(span)),
        }
    }

    fn infer_unary(&self, unary: &aoel_ast::UnaryExpr) -> Result<Type, TypeCheckError> {
        let operand_type = self.infer(&unary.operand)?;
        let span = unary.span;

        match unary.op {
            UnaryOp::Not => {
                if !is_bool_type(&operand_type) {
                    return Err(TypeCheckError::InvalidOperandType {
                        operator: "NOT".to_string(),
                        expected: "BOOL".to_string(),
                        found: type_to_string(&operand_type),
                        span: unary.operand.span(),
                    });
                }
                Ok(bool_type(span))
            }
            UnaryOp::Neg => {
                if !is_numeric_type(&operand_type) {
                    return Err(TypeCheckError::InvalidOperandType {
                        operator: "-".to_string(),
                        expected: "numeric".to_string(),
                        found: type_to_string(&operand_type),
                        span: unary.operand.span(),
                    });
                }
                Ok(operand_type)
            }
        }
    }

    fn infer_call(&self, call: &aoel_ast::CallExpr) -> Result<Type, TypeCheckError> {
        // Look up the function
        if let Some(symbol) = self.symbols.lookup(&call.name.name) {
            if let SymbolKind::BuiltinFunction { returns_bool, .. } = &symbol.kind {
                let span = call.span;
                if *returns_bool {
                    return Ok(bool_type(span));
                } else {
                    // For non-bool returning functions, we need to infer based on context
                    // For now, return int64 for aggregate functions
                    return Ok(int64_type(span));
                }
            }
        }

        // Unknown function - return void as placeholder
        Ok(void_type(call.span))
    }

    fn infer_index(&self, index: &aoel_ast::IndexExpr) -> Result<Type, TypeCheckError> {
        let base_type = self.infer(&index.base)?;

        match &base_type {
            Type::Array(arr) => Ok(arr.element_type.clone()),
            Type::Map(map) => Ok(map.value_type.clone()),
            _ => Ok(void_type(index.span)),
        }
    }
}

/// Infer expression type with given symbol table
pub fn infer_expr_type(expr: &Expr, symbols: &SymbolTable) -> Result<Type, TypeCheckError> {
    TypeInferrer::new(symbols).infer(expr)
}
