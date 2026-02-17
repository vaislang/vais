//! Collection and aggregate type checking (arrays, tuples, structs, ranges, etc.)

use std::collections::HashMap;
use vais_ast::*;
use crate::TypeChecker;
use crate::types::{self, GenericInstantiation, ResolvedType, TypeError, TypeResult};

impl TypeChecker {
    /// Check collection expressions
    pub(crate) fn check_collection_expr(&mut self, expr: &Spanned<Expr>) -> Option<TypeResult<ResolvedType>> {
        match &expr.node {
            Expr::Binary { op, left, right } => {
                let left_type = match self.check_expr(left) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let right_type = match self.check_expr(right) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // Allow string concatenation with +
                        if matches!(op, BinOp::Add) && matches!(left_type, ResolvedType::Str) {
                            if let Err(e) = self.unify(&left_type, &right_type) {
                                return Some(Err(e));
                            }
                            return Some(Ok(ResolvedType::Str));
                        }
                        if !left_type.is_numeric() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            }));
                        }
                        if let Err(e) = self.unify(&left_type, &right_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(left_type))
                    }
                    BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte => {
                        // Allow string comparison with <, >, <=, >=
                        if matches!(left_type, ResolvedType::Str) {
                            if let Err(e) = self.unify(&left_type, &right_type) {
                                return Some(Err(e));
                            }
                            return Some(Ok(ResolvedType::Bool));
                        }
                        if !left_type.is_numeric() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            }));
                        }
                        if let Err(e) = self.unify(&left_type, &right_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(ResolvedType::Bool))
                    }
                    BinOp::Eq | BinOp::Neq => {
                        if let Err(e) = self.unify(&left_type, &right_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(ResolvedType::Bool))
                    }
                    BinOp::And | BinOp::Or => {
                        if let Err(e) = self.unify(&left_type, &ResolvedType::Bool) {
                            return Some(Err(e));
                        }
                        if let Err(e) = self.unify(&right_type, &ResolvedType::Bool) {
                            return Some(Err(e));
                        }
                        Some(Ok(ResolvedType::Bool))
                    }
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                        // Allow bool operands for BitAnd (&) and BitOr (|) as logical and/or
                        if matches!(left_type, ResolvedType::Bool)
                            && matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
                        {
                            if let Err(e) = self.unify(&left_type, &right_type) {
                                return Some(Err(e));
                            }
                            return Some(Ok(ResolvedType::Bool));
                        }
                        if !left_type.is_integer() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            }));
                        }
                        if let Err(e) = self.unify(&left_type, &right_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(left_type))
                    }
                }
            }

            Expr::Unary { op, expr: inner } => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                match op {
                    UnaryOp::Neg => {
                        if !inner_type.is_numeric() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: inner_type.to_string(),
                                span: Some(inner.span),
                            }));
                        }
                        Some(Ok(inner_type))
                    }
                    UnaryOp::Not => {
                        if let Err(e) = self.unify(&inner_type, &ResolvedType::Bool) {
                            return Some(Err(e));
                        }
                        Some(Ok(ResolvedType::Bool))
                    }
                    UnaryOp::BitNot => {
                        if !inner_type.is_integer() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: inner_type.to_string(),
                                span: Some(inner.span),
                            }));
                        }
                        Some(Ok(inner_type))
                    }
                }
            }

            Expr::Field { expr: inner, field } => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };

                // Handle both direct Named types and references to Named types
                let type_name = match &inner_type {
                    ResolvedType::Named { name, .. } => Some(name.clone()),
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                        if let ResolvedType::Named { name, .. } = inner.as_ref() {
                            Some(name.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(name) = type_name.clone() {
                    // Check struct fields
                    if let Some(struct_def) = self.structs.get(&name) {
                        if let Some(field_type) = struct_def.fields.get(&field.node) {
                            return Some(Ok(field_type.clone()));
                        }
                    }
                    // Check union fields
                    if let Some(union_def) = self.unions.get(&name) {
                        if let Some(field_type) = union_def.fields.get(&field.node) {
                            return Some(Ok(field_type.clone()));
                        }
                    }
                }

                // Get field names for did-you-mean suggestion
                let suggestion = if let Some(ref name) = type_name {
                    if let Some(struct_def) = self.structs.get(name) {
                        types::find_similar_name(
                            &field.node,
                            struct_def.fields.keys().map(|s| s.as_str()),
                        )
                    } else if let Some(union_def) = self.unions.get(name) {
                        types::find_similar_name(
                            &field.node,
                            union_def.fields.keys().map(|s| s.as_str()),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                let display_type_name = type_name.unwrap_or_else(|| inner_type.to_string());
                Some(Err(TypeError::NoSuchField {
                    field: field.node.clone(),
                    type_name: display_type_name,
                    suggestion,
                    span: Some(field.span),
                }))
            }

            Expr::Index { expr: inner, index } => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let index_type = match self.check_expr(index) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };

                // Check if this is a slice operation (index is a Range)
                let is_slice = matches!(index.node, Expr::Range { .. });

                match inner_type {
                    ResolvedType::Array(elem_type) => {
                        if is_slice {
                            // Slice returns a pointer to array elements
                            Some(Ok(ResolvedType::Pointer(elem_type)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*elem_type))
                        }
                    }
                    ResolvedType::Map(key_type, value_type) => {
                        if let Err(e) = self.unify(&key_type, &index_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(*value_type))
                    }
                    // Pointers can be indexed like arrays
                    ResolvedType::Pointer(elem_type) => {
                        if is_slice {
                            // Slice of pointer returns a pointer
                            Some(Ok(ResolvedType::Pointer(elem_type)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*elem_type))
                        }
                    }
                    ResolvedType::Slice(elem_type) => {
                        if is_slice {
                            Some(Ok(ResolvedType::Slice(elem_type)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*elem_type))
                        }
                    }
                    ResolvedType::SliceMut(elem_type) => {
                        if is_slice {
                            Some(Ok(ResolvedType::SliceMut(elem_type)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*elem_type))
                        }
                    }
                    _ => Some(Err(TypeError::Mismatch {
                        expected: "indexable type".to_string(),
                        found: inner_type.to_string(),
                        span: Some(expr.span),
                    })),
                }
            }

            Expr::Array(exprs) => {
                if exprs.is_empty() {
                    let var = self.fresh_type_var();
                    // Array literals decay to pointers in Vais
                    return Some(Ok(ResolvedType::Pointer(Box::new(var))));
                }

                // Helper: get element type from an array element (handles Spread)
                let get_elem_type =
                    |checker: &mut Self, e: &Spanned<Expr>| -> TypeResult<ResolvedType> {
                        if let Expr::Spread(inner) = &e.node {
                            let inner_type = checker.check_expr(inner)?;
                            // Spread must be on a pointer/array type
                            match inner_type {
                                ResolvedType::Pointer(elem) => Ok(*elem),
                                ResolvedType::Array(elem) => Ok(*elem),
                                _ => Ok(inner_type),
                            }
                        } else {
                            checker.check_expr(e)
                        }
                    };

                let first_type = match get_elem_type(self, &exprs[0]) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                for expr in &exprs[1..] {
                    let t = match get_elem_type(self, expr) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if let Err(e) = self.unify(&first_type, &t) {
                        return Some(Err(e));
                    }
                }

                // Array literals produce pointers to first element
                Some(Ok(ResolvedType::Pointer(Box::new(first_type))))
            }

            Expr::Tuple(exprs) => {
                let mut types = Vec::new();
                for e in exprs {
                    match self.check_expr(e) {
                        Ok(t) => types.push(t),
                        Err(e) => return Some(Err(e)),
                    }
                }
                Some(Ok(ResolvedType::Tuple(types)))
            }

            Expr::MapLit(pairs) => {
                if pairs.is_empty() {
                    let k = self.fresh_type_var();
                    let v = self.fresh_type_var();
                    return Some(Ok(ResolvedType::Map(Box::new(k), Box::new(v))));
                }
                let first_key_type = match self.check_expr(&pairs[0].0) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let first_val_type = match self.check_expr(&pairs[0].1) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                for (k, v) in &pairs[1..] {
                    let kt = match self.check_expr(k) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    let vt = match self.check_expr(v) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if let Err(e) = self.unify(&first_key_type, &kt) {
                        return Some(Err(e));
                    }
                    if let Err(e) = self.unify(&first_val_type, &vt) {
                        return Some(Err(e));
                    }
                }
                Some(Ok(ResolvedType::Map(
                    Box::new(first_key_type),
                    Box::new(first_val_type),
                )))
            }

            Expr::StructLit { name, fields } => {
                // First check for struct
                if let Some(struct_def) = self.structs.get(&name.node).cloned() {
                    // Create fresh type variables for generic parameters
                    let generic_substitutions: HashMap<String, ResolvedType> = struct_def
                        .generics
                        .iter()
                        .map(|param| (param.clone(), self.fresh_type_var()))
                        .collect();

                    // Check each field and unify with expected type
                    for (field_name, value) in fields {
                        let value_type = match self.check_expr(value) {
                            Ok(t) => t,
                            Err(e) => return Some(Err(e)),
                        };
                        if let Some(expected_type) =
                            struct_def.fields.get(&field_name.node).cloned()
                        {
                            // Substitute generic parameters with type variables
                            let expected_type =
                                self.substitute_generics(&expected_type, &generic_substitutions);
                            if let Err(e) = self.unify(&expected_type, &value_type) {
                                return Some(Err(e));
                            }
                        } else {
                            let suggestion = types::find_similar_name(
                                &field_name.node,
                                struct_def.fields.keys().map(|s| s.as_str()),
                            );
                            return Some(Err(TypeError::UndefinedVar {
                                name: field_name.node.clone(),
                                span: Some(field_name.span),
                                suggestion,
                            }));
                        }
                    }

                    // Apply substitutions to infer concrete generic types
                    let inferred_generics: Vec<_> = struct_def
                        .generics
                        .iter()
                        .map(|param| {
                            let ty = generic_substitutions.get(param)
                                .expect("Internal compiler error: generic parameter should exist in substitutions map");
                            self.apply_substitutions(ty)
                        })
                        .collect();

                    // Record generic struct instantiation if the struct has generic parameters
                    if !struct_def.generics.is_empty() {
                        // Only record if all type arguments are concrete (not type variables)
                        let all_concrete = inferred_generics
                            .iter()
                            .all(|t| !matches!(t, ResolvedType::Var(_)));
                        if all_concrete {
                            let inst = GenericInstantiation::struct_type(
                                &name.node,
                                inferred_generics.clone(),
                            );
                            self.add_instantiation(inst);
                        }
                    }

                    Some(Ok(ResolvedType::Named {
                        name: name.node.clone(),
                        generics: inferred_generics,
                    }))
                // Then check for union (uses same syntax: `UnionName { field: value }`)
                } else if let Some(union_def) = self.unions.get(&name.node).cloned() {
                    // Create fresh type variables for generic parameters
                    let generic_substitutions: HashMap<String, ResolvedType> = union_def
                        .generics
                        .iter()
                        .map(|param| (param.clone(), self.fresh_type_var()))
                        .collect();

                    // Union literal should have exactly one field
                    if fields.len() != 1 {
                        return Some(Err(TypeError::Mismatch {
                            expected: "exactly one field for union initialization".to_string(),
                            found: format!("{} fields", fields.len()),
                            span: Some(expr.span),
                        }));
                    }

                    // Check the field
                    let (field_name, value) = &fields[0];
                    let value_type = match self.check_expr(value) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if let Some(expected_type) = union_def.fields.get(&field_name.node).cloned() {
                        let expected_type =
                            self.substitute_generics(&expected_type, &generic_substitutions);
                        if let Err(e) = self.unify(&expected_type, &value_type) {
                            return Some(Err(e));
                        }
                    } else {
                        let suggestion = types::find_similar_name(
                            &field_name.node,
                            union_def.fields.keys().map(|s| s.as_str()),
                        );
                        return Some(Err(TypeError::UndefinedVar {
                            name: field_name.node.clone(),
                            span: Some(field_name.span),
                            suggestion,
                        }));
                    }

                    // Apply substitutions to infer concrete generic types
                    let inferred_generics: Vec<_> = union_def
                        .generics
                        .iter()
                        .map(|param| {
                            let ty = generic_substitutions.get(param)
                                .expect("Internal compiler error: generic parameter should exist in substitutions map");
                            self.apply_substitutions(ty)
                        })
                        .collect();

                    Some(Ok(ResolvedType::Named {
                        name: name.node.clone(),
                        generics: inferred_generics,
                    }))
                } else {
                    // Get all type names for suggestion
                    let mut type_candidates: Vec<&str> = Vec::new();
                    type_candidates.extend(self.structs.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.enums.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.unions.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.type_aliases.keys().map(|s| s.as_str()));

                    let suggestion =
                        types::find_similar_name(&name.node, type_candidates.into_iter());
                    Some(Err(TypeError::UndefinedType {
                        name: name.node.clone(),
                        span: Some(name.span),
                        suggestion,
                    }))
                }
            }

            Expr::Range {
                start,
                end,
                inclusive: _,
            } => {
                // Infer the element type from start or end expressions
                let elem_type = if let Some(start_expr) = start {
                    let start_type = match self.check_expr(start_expr) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    // Ensure start is a numeric type (integer)
                    if !start_type.is_integer() {
                        return Some(Err(TypeError::Mismatch {
                            expected: "integer type".to_string(),
                            found: start_type.to_string(),
                            span: Some(start_expr.span),
                        }));
                    }

                    // If end is present, unify the types
                    if let Some(end_expr) = end {
                        let end_type = match self.check_expr(end_expr) {
                            Ok(t) => t,
                            Err(e) => return Some(Err(e)),
                        };
                        if !end_type.is_integer() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "integer type".to_string(),
                                found: end_type.to_string(),
                                span: Some(end_expr.span),
                            }));
                        }
                        if let Err(e) = self.unify(&start_type, &end_type) {
                            return Some(Err(e));
                        }
                    }

                    start_type
                } else if let Some(end_expr) = end {
                    // Only end is present (e.g., ..10)
                    let end_type = match self.check_expr(end_expr) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if !end_type.is_integer() {
                        return Some(Err(TypeError::Mismatch {
                            expected: "integer type".to_string(),
                            found: end_type.to_string(),
                            span: Some(end_expr.span),
                        }));
                    }
                    end_type
                } else {
                    // Neither start nor end (e.g., ..) - default to i64
                    ResolvedType::I64
                };

                Some(Ok(ResolvedType::Range(Box::new(elem_type))))
            }

            _ => None,
        }
    }
}
