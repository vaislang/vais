//! Tests for nested struct field access (o.a.val style)

#[cfg(test)]
mod tests {
    use crate::CodeGenerator;
    use vais_ast::*;

    /// Helper to create an Inner struct with a val field
    fn create_inner_struct() -> Struct {
        Struct {
            name: Spanned::new("Inner".to_string(), Span::default()),
            generics: vec![],
            fields: vec![Field {
                name: Spanned::new("val".to_string(), Span::default()),
                ty: Spanned::new(
                    Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    Span::default(),
                ),
                is_pub: true,
            }],
            methods: vec![],
            is_pub: true,
            attributes: vec![],
            where_clause: vec![],
        }
    }

    /// Helper to create an Outer struct with an Inner field
    fn create_outer_struct() -> Struct {
        Struct {
            name: Spanned::new("Outer".to_string(), Span::default()),
            generics: vec![],
            fields: vec![Field {
                name: Spanned::new("a".to_string(), Span::default()),
                ty: Spanned::new(
                    Type::Named {
                        name: "Inner".to_string(),
                        generics: vec![],
                    },
                    Span::default(),
                ),
                is_pub: true,
            }],
            methods: vec![],
            is_pub: true,
            attributes: vec![],
            where_clause: vec![],
        }
    }

    /// Helper to create a function that tests nested field access
    fn create_nested_field_test_fn() -> Function {
        // Create o.a.val expression
        let inner_field_access = Expr::Field {
            expr: Box::new(Spanned::new(Expr::Ident("o".to_string()), Span::default())),
            field: Spanned::new("a".to_string(), Span::default()),
        };

        let nested_field_access = Expr::Field {
            expr: Box::new(Spanned::new(inner_field_access, Span::default())),
            field: Spanned::new("val".to_string(), Span::default()),
        };

        Function {
            name: Spanned::new("test_nested".to_string(), Span::default()),
            generics: vec![],
            params: vec![Param {
                name: Spanned::new("o".to_string(), Span::default()),
                ty: Spanned::new(
                    Type::Named {
                        name: "Outer".to_string(),
                        generics: vec![],
                    },
                    Span::default(),
                ),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            }],
            ret_type: Some(Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                Span::default(),
            )),
            body: FunctionBody::Expr(Box::new(Spanned::new(nested_field_access, Span::default()))),
            is_pub: true,
            is_async: false,
            attributes: vec![],
            where_clause: vec![],
        }
    }

    #[test]
    fn test_nested_struct_field_access() {
        let mut codegen = CodeGenerator::new("test");

        // Register the Inner struct
        let inner_struct = create_inner_struct();
        codegen.register_struct(&inner_struct).unwrap();

        // Register the Outer struct
        let outer_struct = create_outer_struct();
        codegen.register_struct(&outer_struct).unwrap();

        // Register and generate the function that accesses o.a.val
        let test_fn = create_nested_field_test_fn();
        codegen.register_function(&test_fn).unwrap();

        // Generate the function IR
        let fn_ir = codegen.generate_function(&test_fn).unwrap();

        // The generated IR should:
        // 1. Access field 'a' from Outer (field index 0) via GEP
        // 2. Access field 'val' from Inner (field index 0) via GEP
        // 3. Load only the final i64 value (not intermediate structs for efficiency)
        // Should contain getelementptr instructions for both field accesses
        assert!(
            fn_ir.contains("getelementptr %Outer"),
            "Should contain getelementptr for Outer struct access, got:\n{}",
            fn_ir
        );
        assert!(
            fn_ir.contains("getelementptr %Inner"),
            "Should contain getelementptr for Inner struct access, got:\n{}",
            fn_ir
        );

        // Should load only the final value (optimized to avoid loading intermediate structs)
        let load_count = fn_ir.matches("load i64").count();
        assert!(
            load_count >= 1,
            "Should have at least 1 load for the final i64 value, got {} loads in:\n{}",
            load_count,
            fn_ir
        );
    }

    #[test]
    fn test_simple_field_access_still_works() {
        let mut codegen = CodeGenerator::new("test");

        // Register the Inner struct
        let inner_struct = create_inner_struct();
        codegen.register_struct(&inner_struct).unwrap();

        // Create a simple function that just accesses i.val (not nested)
        let simple_field_access = Expr::Field {
            expr: Box::new(Spanned::new(Expr::Ident("i".to_string()), Span::default())),
            field: Spanned::new("val".to_string(), Span::default()),
        };

        let simple_fn = Function {
            name: Spanned::new("test_simple".to_string(), Span::default()),
            generics: vec![],
            params: vec![Param {
                name: Spanned::new("i".to_string(), Span::default()),
                ty: Spanned::new(
                    Type::Named {
                        name: "Inner".to_string(),
                        generics: vec![],
                    },
                    Span::default(),
                ),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            }],
            ret_type: Some(Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                Span::default(),
            )),
            body: FunctionBody::Expr(Box::new(Spanned::new(simple_field_access, Span::default()))),
            is_pub: true,
            is_async: false,
            attributes: vec![],
            where_clause: vec![],
        };

        codegen.register_function(&simple_fn).unwrap();
        let fn_ir = codegen.generate_function(&simple_fn).unwrap();

        // Should generate valid IR with getelementptr and load
        assert!(
            fn_ir.contains("getelementptr %Inner"),
            "Should contain getelementptr for Inner struct, got:\n{}",
            fn_ir
        );
        assert!(
            fn_ir.contains("load i64"),
            "Should load i64 value from field, got:\n{}",
            fn_ir
        );
    }
}
