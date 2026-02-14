//! Derive macro framework
//!
//! Provides automatic code generation for common traits via #[derive(...)] attributes.
//! Scans struct and enum definitions for `#[derive(...)]` and generates corresponding
//! trait impl blocks that are injected back into the module AST.
//!
//! Supported derive macros:
//! - Clone: Generate clone implementation (returns new struct with same field values)
//! - PartialEq: Generate equality comparison (field-by-field comparison)
//! - Default: Generate default value constructor (all fields zero/empty)
//! - Debug: Recognized but generates no-op impl (debug formatting requires runtime support)
//! - Hash: Recognized but generates no-op impl (hash requires runtime support)
//! - Error: Recognized but generates no-op impl (error display requires runtime support)

use std::collections::HashMap;
use vais_ast::{
    Attribute, BinOp, Expr, Field, Function, FunctionBody, Impl, Item, Module, Ownership, Param,
    Span, Spanned, Stmt, Struct, Type,
};

/// Result type for derive macro operations
pub type DeriveResult<T> = Result<T, DeriveError>;

/// Error type for derive macro failures
#[derive(Debug, Clone)]
pub enum DeriveError {
    /// Unsupported derive macro
    UnsupportedDerive(String),
    /// Cannot derive for this type
    CannotDerive { derive: String, reason: String },
    /// Missing required trait bound
    MissingBound {
        derive: String,
        field: String,
        bound: String,
    },
}

impl std::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeriveError::UnsupportedDerive(name) => {
                write!(f, "Unsupported derive macro: '{}'. Supported: Debug, Clone, PartialEq, Default, Hash, Error", name)
            }
            DeriveError::CannotDerive { derive, reason } => {
                write!(f, "Cannot derive '{}': {}", derive, reason)
            }
            DeriveError::MissingBound {
                derive,
                field,
                bound,
            } => {
                write!(
                    f,
                    "Cannot derive '{}': field '{}' requires '{}' bound",
                    derive, field, bound
                )
            }
        }
    }
}

impl std::error::Error for DeriveError {}

/// Derive macro registry
///
/// Maps derive macro names to their generators.
#[derive(Default)]
pub struct DeriveRegistry {
    supported: HashMap<String, ()>,
}

impl DeriveRegistry {
    /// Create a new registry with built-in derive macros
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.supported.insert("Debug".to_string(), ());
        registry.supported.insert("Clone".to_string(), ());
        registry.supported.insert("PartialEq".to_string(), ());
        registry.supported.insert("Default".to_string(), ());
        registry.supported.insert("Hash".to_string(), ());
        registry.supported.insert("Error".to_string(), ());
        registry
    }

    /// Check if a derive macro is supported
    pub fn is_supported(&self, name: &str) -> bool {
        self.supported.contains_key(name)
    }
}

/// Trait for derive macro generators
pub trait DeriveGenerator: Send + Sync {
    /// Get the trait name this derive generates
    fn trait_name(&self) -> &str;

    /// Get required bounds for fields
    fn required_bounds(&self) -> Vec<String> {
        vec![self.trait_name().to_string()]
    }
}

/// Helper to create a spanned node with dummy span
fn sp<T>(node: T) -> Spanned<T> {
    Spanned {
        node,
        span: Span::default(),
    }
}

/// Make a Param with defaults for generated code
fn make_param(name: &str, ty: Type) -> Param {
    Param {
        name: sp(name.to_string()),
        ty: sp(ty),
        is_mut: false,
        is_vararg: false,
        ownership: Ownership::Regular,
        default_value: None,
    }
}

/// Make a Function with defaults for generated code
fn make_fn(name: &str, params: Vec<Param>, ret_type: Option<Type>, body_expr: Expr) -> Function {
    Function {
        name: sp(name.to_string()),
        generics: vec![],
        params,
        ret_type: ret_type.map(sp),
        body: FunctionBody::Block(vec![sp(Stmt::Expr(Box::new(sp(body_expr))))]),
        is_pub: false,
        is_async: false,
        attributes: vec![],
        where_clause: vec![],
    }
}

/// Extract derive trait names from attributes
fn extract_derives(attributes: &[Attribute]) -> Vec<String> {
    let mut derives = Vec::new();
    for attr in attributes {
        if attr.name == "derive" {
            for arg in &attr.args {
                derives.push(arg.clone());
            }
        }
    }
    derives
}

/// Generate a Clone impl block for a struct
fn generate_clone_impl(s: &Struct) -> Impl {
    let name = &s.name.node;

    // Build clone body: StructName { field1: self.field1, field2: self.field2, ... }
    let fields: Vec<(Spanned<String>, Spanned<Expr>)> = s
        .fields
        .iter()
        .map(|f| {
            let fname = f.name.node.clone();
            let field_access = Expr::Field {
                expr: Box::new(sp(Expr::Ident("self".to_string()))),
                field: sp(fname.clone()),
            };
            (sp(fname), sp(field_access))
        })
        .collect();

    let clone_body = Expr::StructLit {
        name: sp(name.clone()),
        fields,
    };

    let self_param = make_param(
        "self",
        Type::Ref(Box::new(sp(Type::Named {
            name: name.clone(),
            generics: vec![],
        }))),
    );

    let ret = Type::Named {
        name: name.clone(),
        generics: vec![],
    };

    let clone_fn = make_fn("clone", vec![self_param], Some(ret), clone_body);

    Impl {
        target_type: sp(Type::Named {
            name: name.clone(),
            generics: vec![],
        }),
        trait_name: Some(sp("Clone".to_string())),
        generics: vec![],
        associated_types: vec![],
        methods: vec![sp(clone_fn)],
    }
}

/// Generate a PartialEq impl block for a struct
fn generate_partial_eq_impl(s: &Struct) -> Impl {
    let name = &s.name.node;

    let eq_body = if s.fields.is_empty() {
        Expr::Bool(true)
    } else {
        let mut comparisons: Vec<Expr> = s
            .fields
            .iter()
            .map(|f| {
                let fname = f.name.node.clone();
                let self_field = Expr::Field {
                    expr: Box::new(sp(Expr::Ident("self".to_string()))),
                    field: sp(fname.clone()),
                };
                let other_field = Expr::Field {
                    expr: Box::new(sp(Expr::Ident("other".to_string()))),
                    field: sp(fname),
                };
                Expr::Binary {
                    op: BinOp::Eq,
                    left: Box::new(sp(self_field)),
                    right: Box::new(sp(other_field)),
                }
            })
            .collect();

        if comparisons.len() == 1 {
            comparisons.remove(0)
        } else {
            let mut result = comparisons.remove(0);
            for cmp in comparisons {
                result = Expr::Binary {
                    op: BinOp::And,
                    left: Box::new(sp(result)),
                    right: Box::new(sp(cmp)),
                };
            }
            result
        }
    };

    let self_param = make_param(
        "self",
        Type::Ref(Box::new(sp(Type::Named {
            name: name.clone(),
            generics: vec![],
        }))),
    );
    let other_param = make_param(
        "other",
        Type::Ref(Box::new(sp(Type::Named {
            name: name.clone(),
            generics: vec![],
        }))),
    );
    let ret = Type::Named {
        name: "bool".to_string(),
        generics: vec![],
    };

    let eq_fn = make_fn("eq", vec![self_param, other_param], Some(ret), eq_body);

    Impl {
        target_type: sp(Type::Named {
            name: name.clone(),
            generics: vec![],
        }),
        trait_name: Some(sp("PartialEq".to_string())),
        generics: vec![],
        associated_types: vec![],
        methods: vec![sp(eq_fn)],
    }
}

/// Generate a Default impl block for a struct
fn generate_default_impl(s: &Struct) -> Impl {
    let name = &s.name.node;

    let fields: Vec<(Spanned<String>, Spanned<Expr>)> = s
        .fields
        .iter()
        .map(|f| {
            let fname = f.name.node.clone();
            let default_val = default_value_for_field(f);
            (sp(fname), sp(default_val))
        })
        .collect();

    let default_body = Expr::StructLit {
        name: sp(name.clone()),
        fields,
    };

    let ret = Type::Named {
        name: name.clone(),
        generics: vec![],
    };

    let default_fn = make_fn("default", vec![], Some(ret), default_body);

    Impl {
        target_type: sp(Type::Named {
            name: name.clone(),
            generics: vec![],
        }),
        trait_name: Some(sp("Default".to_string())),
        generics: vec![],
        associated_types: vec![],
        methods: vec![sp(default_fn)],
    }
}

/// Get default value expression for a field based on its type.
/// Only primitive types are supported. Generic/complex types default to 0 (i64).
fn default_value_for_field(f: &Field) -> Expr {
    match &f.ty.node {
        Type::Named { name, generics } => {
            // Generic types (Vec<T>, Option<T>, etc.) are not supported for Default derive
            if !generics.is_empty() {
                return Expr::Int(0);
            }
            match name.as_str() {
                "bool" => Expr::Bool(false),
                "str" => Expr::String(String::new()),
                "f32" | "f64" => Expr::Float(0.0),
                "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64"
                | "u128" => Expr::Int(0),
                _ => Expr::Int(0),
            }
        }
        _ => Expr::Int(0),
    }
}

/// Process derive attributes on a module and generate impl blocks
///
/// Scans all struct and enum items for `#[derive(...)]` attributes.
/// For each recognized derive, generates a corresponding impl block
/// and appends it to the module's item list.
pub fn process_derives(module: &mut Module) -> DeriveResult<()> {
    let registry = DeriveRegistry::new();
    let mut new_impls: Vec<Item> = Vec::new();

    for item in &module.items {
        match &item.node {
            Item::Struct(s) => {
                let derives = extract_derives(&s.attributes);
                for derive_name in &derives {
                    if !registry.is_supported(derive_name) {
                        return Err(DeriveError::UnsupportedDerive(derive_name.clone()));
                    }
                    // Generic structs are not yet supported for derive macros
                    if !s.generics.is_empty() {
                        continue;
                    }
                    match derive_name.as_str() {
                        "Clone" => new_impls.push(Item::Impl(generate_clone_impl(s))),
                        "PartialEq" => new_impls.push(Item::Impl(generate_partial_eq_impl(s))),
                        "Default" => new_impls.push(Item::Impl(generate_default_impl(s))),
                        // Debug, Hash, Error: recognized but no-op
                        // (require runtime string/hash support not yet available)
                        _ => {}
                    }
                }
            }
            Item::Enum(e) => {
                let derives = extract_derives(&e.attributes);
                for derive_name in &derives {
                    if !registry.is_supported(derive_name) {
                        return Err(DeriveError::UnsupportedDerive(derive_name.clone()));
                    }
                    // Enum derive is recognized but not yet implemented
                    // (requires variant matching codegen)
                }
            }
            _ => {}
        }
    }

    for impl_item in new_impls {
        module.items.push(sp(impl_item));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_registry() {
        let registry = DeriveRegistry::new();
        assert!(registry.is_supported("Debug"));
        assert!(registry.is_supported("Clone"));
        assert!(registry.is_supported("PartialEq"));
        assert!(registry.is_supported("Default"));
        assert!(registry.is_supported("Hash"));
        assert!(registry.is_supported("Error"));
        assert!(!registry.is_supported("Custom"));
    }

    #[test]
    fn test_derive_error_display_unsupported() {
        let err = DeriveError::UnsupportedDerive("Custom".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Unsupported derive macro"));
        assert!(msg.contains("Custom"));
        assert!(msg.contains("Debug"));
    }

    #[test]
    fn test_derive_error_display_cannot_derive() {
        let err = DeriveError::CannotDerive {
            derive: "Clone".to_string(),
            reason: "field is not cloneable".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Cannot derive 'Clone'"));
        assert!(msg.contains("field is not cloneable"));
    }

    #[test]
    fn test_derive_error_display_missing_bound() {
        let err = DeriveError::MissingBound {
            derive: "Debug".to_string(),
            field: "inner".to_string(),
            bound: "Debug".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Cannot derive 'Debug'"));
        assert!(msg.contains("field 'inner'"));
        assert!(msg.contains("requires 'Debug' bound"));
    }

    #[test]
    fn test_process_derives_empty_module() {
        let mut module = Module {
            items: vec![],
            modules_map: Default::default(),
        };
        let result = process_derives(&mut module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_derives_struct_with_clone() {
        let mut module = Module {
            items: vec![sp(Item::Struct(Struct {
                name: sp("Point".to_string()),
                generics: vec![],
                fields: vec![
                    Field {
                        name: sp("x".to_string()),
                        ty: sp(Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        }),
                        is_pub: false,
                    },
                    Field {
                        name: sp("y".to_string()),
                        ty: sp(Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        }),
                        is_pub: false,
                    },
                ],
                methods: vec![],
                is_pub: false,
                attributes: vec![Attribute {
                    name: "derive".to_string(),
                    args: vec!["Clone".to_string()],
                    expr: None,
                }],
                where_clause: vec![],
            }))],
            modules_map: Default::default(),
        };

        let result = process_derives(&mut module);
        assert!(result.is_ok());
        assert_eq!(module.items.len(), 2);
        match &module.items[1].node {
            Item::Impl(impl_block) => {
                assert_eq!(impl_block.trait_name.as_ref().unwrap().node, "Clone");
                assert_eq!(impl_block.methods.len(), 1);
                assert_eq!(impl_block.methods[0].node.name.node, "clone");
            }
            _ => panic!("Expected Impl item"),
        }
    }

    #[test]
    fn test_process_derives_multiple() {
        let mut module = Module {
            items: vec![sp(Item::Struct(Struct {
                name: sp("Color".to_string()),
                generics: vec![],
                fields: vec![Field {
                    name: sp("r".to_string()),
                    ty: sp(Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    }),
                    is_pub: false,
                }],
                methods: vec![],
                is_pub: false,
                attributes: vec![Attribute {
                    name: "derive".to_string(),
                    args: vec![
                        "Clone".to_string(),
                        "PartialEq".to_string(),
                        "Default".to_string(),
                    ],
                    expr: None,
                }],
                where_clause: vec![],
            }))],
            modules_map: Default::default(),
        };

        let result = process_derives(&mut module);
        assert!(result.is_ok());
        assert_eq!(module.items.len(), 4);
    }

    #[test]
    fn test_process_derives_unsupported() {
        let mut module = Module {
            items: vec![sp(Item::Struct(Struct {
                name: sp("Foo".to_string()),
                generics: vec![],
                fields: vec![],
                methods: vec![],
                is_pub: false,
                attributes: vec![Attribute {
                    name: "derive".to_string(),
                    args: vec!["CustomTrait".to_string()],
                    expr: None,
                }],
                where_clause: vec![],
            }))],
            modules_map: Default::default(),
        };

        let result = process_derives(&mut module);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_derives() {
        let attrs = vec![
            Attribute {
                name: "derive".to_string(),
                args: vec!["Clone".to_string(), "Debug".to_string()],
                expr: None,
            },
            Attribute {
                name: "inline".to_string(),
                args: vec![],
                expr: None,
            },
        ];
        let derives = extract_derives(&attrs);
        assert_eq!(derives, vec!["Clone", "Debug"]);
    }

    #[test]
    fn test_derive_registry_default() {
        let registry = DeriveRegistry::default();
        assert!(registry.supported.is_empty());

        let registry = DeriveRegistry::new();
        assert!(registry.is_supported("Debug"));
        assert!(registry.is_supported("Clone"));
    }

    struct TestDeriveGenerator;

    impl DeriveGenerator for TestDeriveGenerator {
        fn trait_name(&self) -> &str {
            "TestTrait"
        }
    }

    #[test]
    fn test_derive_generator_trait_name() {
        let gen = TestDeriveGenerator;
        assert_eq!(gen.trait_name(), "TestTrait");
    }

    #[test]
    fn test_derive_generator_required_bounds() {
        let gen = TestDeriveGenerator;
        let bounds = gen.required_bounds();
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0], "TestTrait");
    }
}
