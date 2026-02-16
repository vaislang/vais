//! Positive integration tests for the Vais parser.
//!
//! Tests that valid Vais source code parses correctly and produces expected AST structures.

use vais_ast::*;
use vais_lexer::tokenize;
use vais_parser::Parser;

// =============================================================================
// Functions
// =============================================================================

#[test]
fn test_parse_simple_function() {
    let source = "F add(a: i64, b: i64) -> i64 = a + b";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "add");
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name.node, "a");
            assert_eq!(f.params[1].name.node, "b");
            assert!(f.ret_type.is_some());
            assert!(matches!(f.body, FunctionBody::Expr(_)));
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_function_with_block_body() {
    let source = r#"
        F factorial(n: i64) -> i64 {
            I n <= 1 {
                R 1
            } E {
                R n * @(n - 1)
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "factorial");
            assert_eq!(f.params.len(), 1);
            assert!(matches!(f.body, FunctionBody::Block(_)));
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_generic_function() {
    let source = "F identity<T>(x: T) -> T = x";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "identity");
            assert_eq!(f.generics.len(), 1);
            assert_eq!(f.generics[0].name.node, "T");
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_public_function() {
    let source = "P F hello() -> () = ()";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "hello");
            assert!(f.is_pub);
        }
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Structs
// =============================================================================

#[test]
fn test_parse_struct() {
    let source = r#"
        S Point {
            x: i64,
            y: i64
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Struct(s) => {
            assert_eq!(s.name.node, "Point");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name.node, "x");
            assert_eq!(s.fields[1].name.node, "y");
        }
        _ => panic!("Expected Struct"),
    }
}

#[test]
fn test_parse_generic_struct() {
    let source = r#"
        S Container<T> {
            value: T
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Struct(s) => {
            assert_eq!(s.name.node, "Container");
            assert_eq!(s.generics.len(), 1);
            assert_eq!(s.generics[0].name.node, "T");
            assert_eq!(s.fields.len(), 1);
        }
        _ => panic!("Expected Struct"),
    }
}

#[test]
fn test_parse_public_struct() {
    let source = r#"
        P S Visible {
            id: i64
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Struct(s) => {
            assert_eq!(s.name.node, "Visible");
            assert!(s.is_pub);
            assert_eq!(s.fields.len(), 1);
        }
        _ => panic!("Expected Struct"),
    }
}

// =============================================================================
// Enums
// =============================================================================

#[test]
fn test_parse_enum() {
    let source = r#"
        E Option<T> {
            None,
            Some(T)
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Enum(e) => {
            assert_eq!(e.name.node, "Option");
            assert_eq!(e.generics.len(), 1);
            assert_eq!(e.variants.len(), 2);
            assert_eq!(e.variants[0].name.node, "None");
            assert_eq!(e.variants[1].name.node, "Some");
        }
        _ => panic!("Expected Enum"),
    }
}

#[test]
fn test_parse_enum_with_struct_variant() {
    let source = r#"
        E Message {
            Text(str),
            Command { name: str, args: Vec<str> }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Enum(e) => {
            assert_eq!(e.name.node, "Message");
            assert_eq!(e.variants.len(), 2);
            match &e.variants[1].fields {
                VariantFields::Struct(fields) => {
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].name.node, "name");
                }
                _ => panic!("Expected Struct variant"),
            }
        }
        _ => panic!("Expected Enum"),
    }
}

// =============================================================================
// Unions
// =============================================================================

#[test]
fn test_parse_union() {
    let source = r#"
        O Data {
            i: i64,
            f: f64
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Union(u) => {
            assert_eq!(u.name.node, "Data");
            assert_eq!(u.fields.len(), 2);
        }
        _ => panic!("Expected Union"),
    }
}

// =============================================================================
// Type Aliases
// =============================================================================

#[test]
fn test_parse_type_alias() {
    let source = "T MyInt = i64";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::TypeAlias(ta) => {
            assert_eq!(ta.name.node, "MyInt");
            match &ta.ty.node {
                Type::Named { name, .. } => assert_eq!(name, "i64"),
                _ => panic!("Expected Named type"),
            }
        }
        _ => panic!("Expected TypeAlias"),
    }
}

// =============================================================================
// Imports
// =============================================================================

#[test]
fn test_parse_use_statement() {
    let source = "U std::collections::HashMap";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Use(u) => {
            assert_eq!(u.path.len(), 3);
            assert_eq!(u.path[0].node, "std");
            assert_eq!(u.path[1].node, "collections");
            assert_eq!(u.path[2].node, "HashMap");
            assert!(u.items.is_none());
        }
        _ => panic!("Expected Use"),
    }
}

#[test]
fn test_parse_use_selective_single() {
    let source = "U std/string.Str";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Use(u) => {
            assert_eq!(u.path.len(), 2);
            assert_eq!(u.path[0].node, "std");
            assert_eq!(u.path[1].node, "string");
            let items = u.items.as_ref().expect("Should have items");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].node, "Str");
        }
        _ => panic!("Expected Use"),
    }
}

#[test]
fn test_parse_use_selective_multi() {
    let source = "U std/option.{Option, Some, None}";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Use(u) => {
            assert_eq!(u.path.len(), 2);
            assert_eq!(u.path[0].node, "std");
            assert_eq!(u.path[1].node, "option");
            let items = u.items.as_ref().expect("Should have items");
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].node, "Option");
            assert_eq!(items[1].node, "Some");
            assert_eq!(items[2].node, "None");
        }
        _ => panic!("Expected Use"),
    }
}

#[test]
fn test_parse_use_with_semicolon() {
    let source = "U std/option;\nF main() -> i64 = 42";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 2);
    match &module.items[0].node {
        Item::Use(u) => {
            assert_eq!(u.path.len(), 2);
            assert_eq!(u.path[0].node, "std");
            assert_eq!(u.path[1].node, "option");
            assert!(u.items.is_none());
        }
        _ => panic!("Expected Use"),
    }
}

#[test]
fn test_parse_use_selective_with_semicolon() {
    let source = "U std/option.{Option, None};\nF main() -> i64 = 42";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 2);
    match &module.items[0].node {
        Item::Use(u) => {
            let items = u.items.as_ref().expect("Should have items");
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].node, "Option");
            assert_eq!(items[1].node, "None");
        }
        _ => panic!("Expected Use"),
    }
}

#[test]
fn test_parse_use_trailing_comma() {
    let source = "U std/option.{Option, Some,}";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Use(u) => {
            let items = u.items.as_ref().expect("Should have items");
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].node, "Option");
            assert_eq!(items[1].node, "Some");
        }
        _ => panic!("Expected Use"),
    }
}

// =============================================================================
// Traits
// =============================================================================

#[test]
fn test_parse_trait() {
    let source = r#"
        W Display {
            F fmt(self) -> str
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Trait(t) => {
            assert_eq!(t.name.node, "Display");
            assert_eq!(t.methods.len(), 1);
            assert_eq!(t.methods[0].name.node, "fmt");
        }
        _ => panic!("Expected Trait"),
    }
}

#[test]
fn test_parse_trait_with_associated_type() {
    let source = r#"
        W Iterator {
            T Item
            F next(self) -> Option<Item>
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Trait(t) => {
            assert_eq!(t.name.node, "Iterator");
            assert_eq!(t.associated_types.len(), 1);
            assert_eq!(t.associated_types[0].name.node, "Item");
            assert_eq!(t.methods.len(), 1);
        }
        _ => panic!("Expected Trait"),
    }
}

// =============================================================================
// Impl Blocks
// =============================================================================

#[test]
fn test_parse_impl_block() {
    let source = r#"
        X Point: Display {
            F fmt(self) -> str {
                R "Point"
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Impl(i) => {
            assert!(i.trait_name.is_some());
            assert_eq!(i.trait_name.as_ref().unwrap().node, "Display");
            assert_eq!(i.methods.len(), 1);
        }
        _ => panic!("Expected Impl"),
    }
}

#[test]
fn test_parse_inherent_impl() {
    let source = r#"
        X Point {
            F new(x: i64, y: i64) -> Point {
                R Point { x: x, y: y }
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Impl(i) => {
            assert!(i.trait_name.is_none());
            assert_eq!(i.methods.len(), 1);
            assert_eq!(i.methods[0].node.name.node, "new");
        }
        _ => panic!("Expected Impl"),
    }
}

// =============================================================================
// Variable Bindings
// =============================================================================

#[test]
fn test_parse_let_binding() {
    let source = r#"
        F test() -> i64 {
            x := 42
            R x
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => {
                assert_eq!(stmts.len(), 2);
                match &stmts[0].node {
                    Stmt::Let { name, is_mut, .. } => {
                        assert_eq!(name.node, "x");
                        assert!(!is_mut);
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_mut_binding() {
    let source = r#"
        F test() -> i64 {
            x := mut 0
            x = 42
            R x
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Let { name, is_mut, .. } => {
                    assert_eq!(name.node, "x");
                    assert!(is_mut);
                }
                _ => panic!("Expected Let statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_typed_binding() {
    let source = r#"
        F test() -> i64 {
            x: i64 = 42
            R x
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Let { name, ty, .. } => {
                    assert_eq!(name.node, "x");
                    assert!(ty.is_some());
                }
                _ => panic!("Expected Let statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Control Flow
// =============================================================================

#[test]
fn test_parse_if_else() {
    let source = r#"
        F abs(x: i64) -> i64 {
            I x < 0 {
                R -x
            } E {
                R x
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Expr(expr) => match &expr.node {
                    Expr::If { cond, then, else_ } => {
                        assert!(matches!(cond.node, Expr::Binary { .. }));
                        assert!(!then.is_empty());
                        assert!(else_.is_some());
                    }
                    _ => panic!("Expected If expression"),
                },
                _ => panic!("Expected Expr statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_loop() {
    let source = r#"
        F infinite() -> () {
            L {
                print("loop")
                B
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Expr(expr) => match &expr.node {
                    Expr::Loop {
                        pattern,
                        iter,
                        body,
                    } => {
                        assert!(pattern.is_none());
                        assert!(iter.is_none());
                        assert!(!body.is_empty());
                    }
                    _ => panic!("Expected Loop expression"),
                },
                _ => panic!("Expected Expr statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_for_loop() {
    let source = r#"
        F sum(items: Vec<i64>) -> i64 {
            total := mut 0
            L x: items {
                total = total + x
            }
            R total
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => {
                assert!(stmts.len() >= 2);
                match &stmts[1].node {
                    Stmt::Expr(expr) => match &expr.node {
                        Expr::Loop { pattern, iter, .. } => {
                            assert!(pattern.is_some());
                            assert!(iter.is_some());
                        }
                        _ => panic!("Expected Loop expression"),
                    },
                    _ => panic!("Expected Expr statement"),
                }
            }
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_match() {
    let source = r#"
        F describe(x: i64) -> str {
            M x {
                0 => "zero",
                1 => "one",
                _ => "other"
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Expr(expr) => match &expr.node {
                    Expr::Match { expr, arms } => {
                        assert!(matches!(expr.node, Expr::Ident(_)));
                        assert_eq!(arms.len(), 3);
                    }
                    _ => panic!("Expected Match expression"),
                },
                _ => panic!("Expected Expr statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_match_with_enum() {
    let source = r#"
        F unwrap<T>(opt: Option<T>) -> T {
            M opt {
                Some(x) => x,
                None => panic("unwrap on None")
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "unwrap");
            assert_eq!(f.generics.len(), 1);
            match &f.body {
                FunctionBody::Block(stmts) => match &stmts[0].node {
                    Stmt::Expr(expr) => match &expr.node {
                        Expr::Match { arms, .. } => {
                            assert_eq!(arms.len(), 2);
                        }
                        _ => panic!("Expected Match expression"),
                    },
                    _ => panic!("Expected Expr statement"),
                },
                _ => panic!("Expected Block body"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Closures
// =============================================================================

#[test]
fn test_parse_closure() {
    let source = r#"
        F map_test() -> Vec<i64> {
            items := [1, 2, 3]
            R items.map(|x| x * 2)
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => {
                assert!(stmts.len() >= 2);
                match &stmts[1].node {
                    Stmt::Return(Some(expr)) => match &expr.node {
                        Expr::MethodCall { args, .. } => {
                            assert_eq!(args.len(), 1);
                            assert!(matches!(args[0].node, Expr::Lambda { .. }));
                        }
                        _ => panic!("Expected MethodCall"),
                    },
                    _ => panic!("Expected Return statement"),
                }
            }
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_multi_param_closure() {
    let source = r#"
        F fold_test() -> i64 {
            items := [1, 2, 3, 4]
            R items.fold(0, |acc, x| acc + x)
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => {
                assert!(stmts.len() >= 2);
                match &stmts[1].node {
                    Stmt::Return(Some(expr)) => match &expr.node {
                        Expr::MethodCall { args, .. } => {
                            assert_eq!(args.len(), 2);
                            match &args[1].node {
                                Expr::Lambda { params, .. } => {
                                    assert_eq!(params.len(), 2);
                                }
                                _ => panic!("Expected Lambda"),
                            }
                        }
                        _ => panic!("Expected MethodCall"),
                    },
                    _ => panic!("Expected Return statement"),
                }
            }
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Method Calls
// =============================================================================

#[test]
fn test_parse_method_call() {
    let source = r#"
        F test() -> i64 {
            s := "hello"
            R s.len()
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[1].node {
                Stmt::Return(Some(expr)) => match &expr.node {
                    Expr::MethodCall {
                        receiver, method, ..
                    } => {
                        assert!(matches!(receiver.node, Expr::Ident(_)));
                        assert_eq!(method.node, "len");
                    }
                    _ => panic!("Expected MethodCall"),
                },
                _ => panic!("Expected Return statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Pipe Operator
// =============================================================================

#[test]
fn test_parse_pipe_operator() {
    let source = r#"
        F pipeline() -> i64 {
            R 42 |> double |> inc
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Return(Some(expr)) => {
                    // Pipe operator is transformed into nested Call expressions
                    // 42 |> double |> inc becomes inc(double(42))
                    assert!(matches!(expr.node, Expr::Call { .. }));
                }
                _ => panic!("Expected Return statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Async/Await
// =============================================================================

#[test]
fn test_parse_async_function() {
    let source = r#"
        A F fetch_data() -> str {
            R "data"
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "fetch_data");
            assert!(f.is_async);
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_await_expression() {
    let source = r#"
        A F call_async() -> str {
            result := fetch_data().await
            R result
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Let { value, .. } => {
                    assert!(matches!(value.node, Expr::Await(_)));
                }
                _ => panic!("Expected Let statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Attributes
// =============================================================================

#[test]
fn test_parse_attribute() {
    let source = r#"
        #[inline]
        F fast() -> i64 = 42
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.attributes.len(), 1);
            assert_eq!(f.attributes[0].name, "inline");
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_cfg_attribute() {
    let source = r#"
        #[cfg(target_os = "linux")]
        F linux_only() -> () = ()
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.attributes.len(), 1);
            assert_eq!(f.attributes[0].name, "cfg");
            assert!(!f.attributes[0].args.is_empty());
        }
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Constants & Globals
// =============================================================================

#[test]
fn test_parse_const() {
    let source = "C MAX: i64 = 100";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Const(c) => {
            assert_eq!(c.name.node, "MAX");
            assert!(matches!(c.value.node, Expr::Int(100)));
        }
        _ => panic!("Expected Const"),
    }
}

#[test]
fn test_parse_global() {
    let source = "G counter: i64 = 0";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Global(g) => {
            assert_eq!(g.name.node, "counter");
            assert!(matches!(g.value.node, Expr::Int(0)));
        }
        _ => panic!("Expected Global"),
    }
}

// =============================================================================
// Defer
// =============================================================================

#[test]
fn test_parse_defer() {
    let source = r#"
        F with_cleanup() -> () {
            file := open("test.txt")
            D close(file)
            R ()
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => {
                assert!(stmts.len() >= 2);
                match &stmts[1].node {
                    Stmt::Defer(_) => {}
                    _ => panic!("Expected Defer statement"),
                }
            }
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Slice Types
// =============================================================================

#[test]
fn test_parse_slice_type() {
    let source = "F process(data: &[i64]) -> i64 = data.len()";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.params.len(), 1);
            match &f.params[0].ty.node {
                Type::Slice(_) => {}
                _ => panic!("Expected Slice type"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_mut_slice_type() {
    let source = "F modify(data: &mut [i64]) -> () = ()";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.params.len(), 1);
            match &f.params[0].ty.node {
                Type::SliceMut(_) => {}
                _ => panic!("Expected SliceMut type"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Complex Expressions
// =============================================================================

#[test]
fn test_parse_array_literal() {
    let source = r#"
        F get_array() -> Vec<i64> {
            R [1, 2, 3, 4, 5]
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Return(Some(expr)) => match &expr.node {
                    Expr::Array(items) => {
                        assert_eq!(items.len(), 5);
                    }
                    _ => panic!("Expected Array"),
                },
                _ => panic!("Expected Return statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_tuple_literal() {
    let source = r#"
        F get_tuple() -> (i64, str) {
            R (42, "hello")
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Return(Some(expr)) => match &expr.node {
                    Expr::Tuple(items) => {
                        assert_eq!(items.len(), 2);
                    }
                    _ => panic!("Expected Tuple"),
                },
                _ => panic!("Expected Return statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_struct_literal() {
    let source = r#"
        F make_point() -> Point {
            R Point { x: 10, y: 20 }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Return(Some(expr)) => match &expr.node {
                    Expr::StructLit { name, fields } => {
                        assert_eq!(name.node, "Point");
                        assert_eq!(fields.len(), 2);
                    }
                    _ => panic!("Expected StructLit"),
                },
                _ => panic!("Expected Return statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_range() {
    let source = r#"
        F get_range() -> () {
            L i: 0..10 {
                print(i)
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Expr(expr) => match &expr.node {
                    Expr::Loop { iter, .. } => {
                        assert!(iter.is_some());
                        assert!(matches!(iter.as_ref().unwrap().node, Expr::Range { .. }));
                    }
                    _ => panic!("Expected Loop expression"),
                },
                _ => panic!("Expected Expr statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_self_recursion() {
    let source = r#"
        F countdown(n: i64) -> () {
            I n > 0 {
                print(n)
                @(n - 1)
            }
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "countdown");
            // SelfCall (@) will be present in the body
            match &f.body {
                FunctionBody::Block(stmts) => {
                    assert!(!stmts.is_empty());
                }
                _ => panic!("Expected Block body"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_string_interpolation() {
    let source = r#"
        F greet(name: str) -> str {
            R "Hello, ~{name}!"
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Return(Some(expr)) => {
                    // String interpolation with ~{} syntax
                    assert!(matches!(expr.node, Expr::StringInterp(_)));
                }
                _ => panic!("Expected Return statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Error Handling Operators
// =============================================================================

#[test]
fn test_parse_try_operator() {
    let source = r#"
        F read_file(path: str) -> Result<str, Error> {
            content := open(path)?
            R Ok(content)
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Let { value, .. } => {
                    assert!(matches!(value.node, Expr::Try(_)));
                }
                _ => panic!("Expected Let statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_unwrap_operator() {
    let source = r#"
        F get_value(opt: Option<i64>) -> i64 {
            R opt!
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => match &stmts[0].node {
                Stmt::Return(Some(expr)) => {
                    assert!(matches!(expr.node, Expr::Unwrap(_)));
                }
                _ => panic!("Expected Return statement"),
            },
            _ => panic!("Expected Block body"),
        },
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Combined Features
// =============================================================================

#[test]
fn test_parse_complex_generic_function_with_trait_bounds() {
    let source = r#"
        F process<T>(items: Vec<T>) -> Vec<T> {
            result := items.map(|x| x)
            R result
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "process");
            assert_eq!(f.generics.len(), 1);
            assert_eq!(f.generics[0].name.node, "T");
        }
        _ => panic!("Expected Function"),
    }
}

// =============================================================================
// Closure Capture Modes
// =============================================================================

#[test]
fn test_parse_lambda_default_capture() {
    let source = "F main() { f := |x| x + 1 }";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0].node {
                    Stmt::Let { value, .. } => match &value.node {
                        Expr::Lambda { capture_mode, .. } => {
                            assert_eq!(*capture_mode, CaptureMode::ByValue);
                        }
                        _ => panic!("Expected Lambda"),
                    },
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected Block"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_lambda_move_capture() {
    let source = "F main() { f := move |x| x + 1 }";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Function(f) => match &f.body {
            FunctionBody::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0].node {
                    Stmt::Let { value, .. } => match &value.node {
                        Expr::Lambda { capture_mode, .. } => {
                            assert_eq!(*capture_mode, CaptureMode::Move);
                        }
                        _ => panic!("Expected Lambda"),
                    },
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected Block"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_lambda_move_with_captures() {
    let source = r#"
        F main() {
            x := 42
            f := move |y| x + y
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Function(f) => {
            match &f.body {
                FunctionBody::Block(stmts) => {
                    assert_eq!(stmts.len(), 2);
                    // Check second statement (lambda)
                    match &stmts[1].node {
                        Stmt::Let { value, .. } => match &value.node {
                            Expr::Lambda {
                                capture_mode,
                                params,
                                ..
                            } => {
                                assert_eq!(*capture_mode, CaptureMode::Move);
                                assert_eq!(params.len(), 1);
                                assert_eq!(params[0].name.node, "y");
                            }
                            _ => panic!("Expected Lambda"),
                        },
                        _ => panic!("Expected Let statement"),
                    }
                }
                _ => panic!("Expected Block"),
            }
        }
        _ => panic!("Expected Function"),
    }
}
