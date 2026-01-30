//! Tests for struct value passing in function arguments

#[cfg(test)]
mod tests {
    use crate::CodeGenerator;
    use vais_ast::*;

    /// Helper to create a simple Point struct definition
    fn create_point_struct() -> Struct {
        Struct {
            name: Spanned::new("Point".to_string(), Span::default()),
            generics: vec![],
            fields: vec![
                Field {
                    name: Spanned::new("x".to_string(), Span::default()),
                    ty: Spanned::new(Type::Named { name: "i64".to_string(), generics: vec![] }, Span::default()),
                    is_pub: true,
                },
                Field {
                    name: Spanned::new("y".to_string(), Span::default()),
                    ty: Spanned::new(Type::Named { name: "i64".to_string(), generics: vec![] }, Span::default()),
                    is_pub: true,
                },
            ],
            methods: vec![],
            is_pub: true,
            attributes: vec![],
        }
    }

    /// Helper to create a function that takes a Point by value
    fn create_point_consumer_fn() -> Function {
        Function {
            name: Spanned::new("consume_point".to_string(), Span::default()),
            generics: vec![],
            params: vec![
                Param {
                    name: Spanned::new("p".to_string(), Span::default()),
                    ty: Spanned::new(Type::Named { name: "Point".to_string(), generics: vec![] }, Span::default()),
                    is_mut: false,
                    is_vararg: false,
                    ownership: Ownership::Regular,
                    default_value: None,
                }
            ],
            ret_type: Some(Spanned::new(Type::Named { name: "i64".to_string(), generics: vec![] }, Span::default())),
            body: FunctionBody::Expr(
                Box::new(Spanned::new(
                    Expr::Int(42),
                    Span::default()
                ))
            ),
            is_pub: true,
            is_async: false,
            attributes: vec![],
        }
    }

    #[test]
    fn test_struct_argument_passing_generates_load() {
        let mut codegen = CodeGenerator::new("test");

        // Register the Point struct
        let point_struct = create_point_struct();
        codegen.register_struct(&point_struct).unwrap();

        // Register the function that takes Point by value
        let point_fn = create_point_consumer_fn();
        codegen.register_function(&point_fn).unwrap();

        // Generate the function
        let fn_ir = codegen.generate_function(&point_fn).unwrap();

        // The function should declare the parameter as a value type (not pointer)
        assert!(fn_ir.contains("%Point %p"),
            "Function should declare Point parameter as value type, got:\n{}", fn_ir);

        // Now test calling the function with a struct literal
        let mut counter = 0;
        let call_expr = Expr::Call {
            func: Box::new(Spanned::new(Expr::Ident("consume_point".to_string()), Span::default())),
            args: vec![
                Spanned::new(
                    Expr::StructLit {
                        name: Spanned::new("Point".to_string(), Span::default()),
                        fields: vec![
                            (
                                Spanned::new("x".to_string(), Span::default()),
                                Spanned::new(Expr::Int(10), Span::default())
                            ),
                            (
                                Spanned::new("y".to_string(), Span::default()),
                                Spanned::new(Expr::Int(20), Span::default())
                            ),
                        ],
                    },
                    Span::default()
                )
            ],
        };

        let (_, call_ir) = codegen.generate_expr(&Spanned::new(call_expr, Span::default()), &mut counter).unwrap();

        // The call should:
        // 1. Allocate the struct on stack (alloca)
        // 2. Store field values
        // 3. LOAD the struct value before passing it
        // 4. Call with the loaded value

        assert!(call_ir.contains("alloca %Point"),
            "Should allocate Point on stack, got:\n{}", call_ir);
        assert!(call_ir.contains("load %Point, %Point*"),
            "Should load Point value from pointer before passing to function, got:\n{}", call_ir);
        assert!(call_ir.contains("call i64 @consume_point(%Point"),
            "Should call function with Point value (not pointer), got:\n{}", call_ir);
    }

    #[test]
    fn test_struct_literal_creates_pointer() {
        let mut codegen = CodeGenerator::new("test");

        // Register the Point struct
        let point_struct = create_point_struct();
        codegen.register_struct(&point_struct).unwrap();

        let mut counter = 0;
        let struct_lit = Expr::StructLit {
            name: Spanned::new("Point".to_string(), Span::default()),
            fields: vec![
                (
                    Spanned::new("x".to_string(), Span::default()),
                    Spanned::new(Expr::Int(1), Span::default())
                ),
                (
                    Spanned::new("y".to_string(), Span::default()),
                    Spanned::new(Expr::Int(2), Span::default())
                ),
            ],
        };

        let (val, ir) = codegen.generate_expr(&Spanned::new(struct_lit, Span::default()), &mut counter).unwrap();

        // Struct literal should allocate on stack
        assert!(ir.contains("alloca %Point"),
            "Struct literal should use alloca, got:\n{}", ir);

        // The returned value should be a pointer (temp variable)
        assert!(val.starts_with('%'),
            "Struct literal should return a pointer (temp var), got: {}", val);
    }

    #[test]
    fn test_multiple_struct_arguments() {
        let mut codegen = CodeGenerator::new("test");

        // Register the Point struct
        let point_struct = create_point_struct();
        codegen.register_struct(&point_struct).unwrap();

        // Create a function that takes two Point arguments
        let fn_def = Function {
            name: Spanned::new("add_points".to_string(), Span::default()),
            generics: vec![],
            params: vec![
                Param {
                    name: Spanned::new("p1".to_string(), Span::default()),
                    ty: Spanned::new(Type::Named { name: "Point".to_string(), generics: vec![] }, Span::default()),
                    is_mut: false,
                    is_vararg: false,
                    ownership: Ownership::Regular,
                    default_value: None,
                },
                Param {
                    name: Spanned::new("p2".to_string(), Span::default()),
                    ty: Spanned::new(Type::Named { name: "Point".to_string(), generics: vec![] }, Span::default()),
                    is_mut: false,
                    is_vararg: false,
                    ownership: Ownership::Regular,
                    default_value: None,
                }
            ],
            ret_type: Some(Spanned::new(Type::Named { name: "Point".to_string(), generics: vec![] }, Span::default())),
            body: FunctionBody::Expr(
                Box::new(Spanned::new(Expr::Ident("p1".to_string()), Span::default()))
            ),
            is_pub: true,
            is_async: false,
            attributes: vec![],
        };

        codegen.register_function(&fn_def).unwrap();
        let fn_ir = codegen.generate_function(&fn_def).unwrap();

        // Both parameters should be value types
        assert!(fn_ir.contains("%Point %p1"),
            "First parameter should be value type, got:\n{}", fn_ir);
        assert!(fn_ir.contains("%Point %p2"),
            "Second parameter should be value type, got:\n{}", fn_ir);

        // Test calling with two struct literals
        let mut counter = 0;
        let call_expr = Expr::Call {
            func: Box::new(Spanned::new(Expr::Ident("add_points".to_string()), Span::default())),
            args: vec![
                Spanned::new(
                    Expr::StructLit {
                        name: Spanned::new("Point".to_string(), Span::default()),
                        fields: vec![
                            (Spanned::new("x".to_string(), Span::default()), Spanned::new(Expr::Int(1), Span::default())),
                            (Spanned::new("y".to_string(), Span::default()), Spanned::new(Expr::Int(2), Span::default())),
                        ],
                    },
                    Span::default()
                ),
                Spanned::new(
                    Expr::StructLit {
                        name: Spanned::new("Point".to_string(), Span::default()),
                        fields: vec![
                            (Spanned::new("x".to_string(), Span::default()), Spanned::new(Expr::Int(3), Span::default())),
                            (Spanned::new("y".to_string(), Span::default()), Spanned::new(Expr::Int(4), Span::default())),
                        ],
                    },
                    Span::default()
                ),
            ],
        };

        let (_, call_ir) = codegen.generate_expr(&Spanned::new(call_expr, Span::default()), &mut counter).unwrap();

        // Both arguments should be loaded before passing
        let load_count = call_ir.matches("load %Point, %Point*").count();
        assert_eq!(load_count, 2,
            "Should load both Point values before passing, found {} loads in:\n{}", load_count, call_ir);
    }
}
