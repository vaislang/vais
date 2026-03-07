use super::*;

#[test]
fn test_const_generation() {
    let mut gen = JsCodeGenerator::new();
    let c = ConstDef {
        name: Spanned::new("MAX_SIZE".to_string(), Span::new(0, 8)),
        ty: Spanned::new(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            Span::new(10, 13),
        ),
        value: Spanned::new(Expr::Int(1024), Span::new(16, 20)),
        is_pub: true,
        attributes: vec![],
    };
    let result = gen.generate_const(&c).unwrap();
    assert_eq!(result, "export const MAX_SIZE = 1024;\n");
}

#[test]
fn test_enum_generation() {
    let mut gen = JsCodeGenerator::new();
    let e = Enum {
        name: Spanned::new("Color".to_string(), Span::new(0, 5)),
        generics: vec![],
        variants: vec![
            Variant {
                name: Spanned::new("Red".to_string(), Span::new(6, 9)),
                fields: VariantFields::Unit,
            },
            Variant {
                name: Spanned::new("Rgb".to_string(), Span::new(11, 14)),
                fields: VariantFields::Tuple(vec![
                    Spanned::new(
                        Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        Span::new(15, 18),
                    ),
                    Spanned::new(
                        Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        Span::new(20, 23),
                    ),
                    Spanned::new(
                        Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        Span::new(25, 28),
                    ),
                ]),
            },
        ],
        is_pub: false,
        attributes: vec![],
    };
    let result = gen.generate_enum(&e).unwrap();
    assert!(result.contains("const Color = Object.freeze"));
    assert!(result.contains("Red: Object.freeze"));
    assert!(result.contains("Rgb(__0, __1, __2)"));
}

#[test]
fn test_result_enum_helpers() {
    let mut gen = JsCodeGenerator::new();
    let e = Enum {
        name: Spanned::new("Result".to_string(), Span::new(0, 6)),
        generics: vec![],
        variants: vec![
            Variant {
                name: Spanned::new("Ok".to_string(), Span::new(7, 9)),
                fields: VariantFields::Tuple(vec![Spanned::new(
                    Type::Infer,
                    Span::new(10, 11),
                )]),
            },
            Variant {
                name: Spanned::new("Err".to_string(), Span::new(12, 15)),
                fields: VariantFields::Tuple(vec![Spanned::new(
                    Type::Infer,
                    Span::new(16, 17),
                )]),
            },
        ],
        is_pub: true,
        attributes: vec![],
    };
    let result = gen.generate_enum(&e).unwrap();
    assert!(result.contains("Result.is_Ok = function"));
    assert!(result.contains("Result.is_Err = function"));
    assert!(result.contains("Result.unwrap = function"));
    assert!(result.contains("Result.unwrap_or = function"));
    assert!(result.contains("Result.map = function"));
}

#[test]
fn test_option_enum_helpers() {
    let mut gen = JsCodeGenerator::new();
    let e = Enum {
        name: Spanned::new("Option".to_string(), Span::new(0, 6)),
        generics: vec![],
        variants: vec![
            Variant {
                name: Spanned::new("Some".to_string(), Span::new(7, 11)),
                fields: VariantFields::Tuple(vec![Spanned::new(
                    Type::Infer,
                    Span::new(12, 13),
                )]),
            },
            Variant {
                name: Spanned::new("None".to_string(), Span::new(14, 18)),
                fields: VariantFields::Unit,
            },
        ],
        is_pub: true,
        attributes: vec![],
    };
    let result = gen.generate_enum(&e).unwrap();
    assert!(result.contains("Option.is_Some = function"));
    assert!(result.contains("Option.is_None = function"));
    assert!(result.contains("Option.unwrap = function"));
    assert!(result.contains("Option.unwrap_or = function"));
    assert!(result.contains("Option.map = function"));
}

#[test]
fn test_generic_function_comment() {
    use vais_ast::{GenericParam, GenericParamKind, Variance};

    let mut gen = JsCodeGenerator::new();
    let func = Function {
        name: Spanned::new("identity".to_string(), Span::new(0, 8)),
        generics: vec![GenericParam {
            name: Spanned::new("T".to_string(), Span::new(9, 10)),
            bounds: vec![],
            kind: GenericParamKind::Type { bounds: vec![] },
            variance: Variance::Invariant,
        }],
        params: vec![Param {
            name: Spanned::new("x".to_string(), Span::new(11, 12)),
            ty: Spanned::new(Type::Infer, Span::new(14, 15)),
            is_mut: false,
            is_vararg: false,
            ownership: Ownership::Regular,
            default_value: None,
        }],
        ret_type: Some(Spanned::new(Type::Infer, Span::new(20, 21))),
        body: FunctionBody::Expr(Box::new(Spanned::new(
            Expr::Ident("x".to_string()),
            Span::new(22, 23),
        ))),
        is_pub: false,
        is_async: false,
        attributes: vec![],
        where_clause: vec![],
    };
    let result = gen.generate_function(&func, false).unwrap();
    assert!(result.contains("/* <T> */"));
    assert!(result.contains("function identity(x)"));
}

#[test]
fn test_generic_struct_comment() {
    use vais_ast::{GenericParam, GenericParamKind, Variance};

    let mut gen = JsCodeGenerator::new();
    let s = Struct {
        name: Spanned::new("Box".to_string(), Span::new(0, 3)),
        generics: vec![GenericParam {
            name: Spanned::new("T".to_string(), Span::new(4, 5)),
            bounds: vec![],
            kind: GenericParamKind::Type { bounds: vec![] },
            variance: Variance::Invariant,
        }],
        fields: vec![Field {
            name: Spanned::new("value".to_string(), Span::new(6, 11)),
            ty: Spanned::new(Type::Infer, Span::new(13, 14)),
            is_pub: true,
        }],
        methods: vec![],
        is_pub: true,
        attributes: vec![],
        where_clause: vec![],
    };
    let result = gen.generate_struct(&s).unwrap();
    assert!(result.contains("/* <T> */"));
    assert!(result.contains("class Box"));
}

#[test]
fn test_trait_impl_tracking() {
    let mut gen = JsCodeGenerator::new();

    // First create a simple struct to impl on
    let s = Struct {
        name: Spanned::new("MyType".to_string(), Span::new(0, 6)),
        generics: vec![],
        fields: vec![],
        methods: vec![],
        is_pub: false,
        attributes: vec![],
        where_clause: vec![],
    };
    gen.generate_struct(&s).unwrap();

    // Now create an impl block for a trait
    let imp = Impl {
        trait_name: Some(Spanned::new("Display".to_string(), Span::new(0, 7))),
        target_type: Spanned::new(
            Type::Named {
                name: "MyType".to_string(),
                generics: vec![],
            },
            Span::new(12, 18),
        ),
        generics: vec![],
        associated_types: vec![],
        methods: vec![Spanned::new(
            Function {
                name: Spanned::new("to_string".to_string(), Span::new(19, 28)),
                generics: vec![],
                params: vec![Param {
                    name: Spanned::new("self".to_string(), Span::new(29, 33)),
                    ty: Spanned::new(Type::Infer, Span::new(35, 36)),
                    is_mut: false,
                    is_vararg: false,
                    ownership: Ownership::Regular,
                    default_value: None,
                }],
                ret_type: Some(Spanned::new(
                    Type::Named {
                        name: "str".to_string(),
                        generics: vec![],
                    },
                    Span::new(41, 44),
                )),
                body: FunctionBody::Expr(Box::new(Spanned::new(
                    Expr::String("MyType".to_string()),
                    Span::new(45, 53),
                ))),
                is_pub: false,
                is_async: false,
                attributes: vec![],
                where_clause: vec![],
            },
            Span::new(19, 53),
        )],
    };
    let result = gen.generate_impl(&imp).unwrap();
    assert!(result.contains("MyType.__implements = new Set()"));
    assert!(result.contains("MyType.__implementsTrait = function"));
    assert!(result.contains("MyType.__implements.add(\"Display\")"));
    assert!(result.contains("MyType.prototype.to_string"));
}

#[test]
fn test_const_private() {
    let mut gen = JsCodeGenerator::new();
    let c = ConstDef {
        name: Spanned::new("PI".to_string(), Span::new(0, 2)),
        ty: Spanned::new(
            Type::Named {
                name: "f64".to_string(),
                generics: vec![],
            },
            Span::new(4, 7),
        ),
        value: Spanned::new(Expr::Float(3.14), Span::new(10, 14)),
        is_pub: false,
        attributes: vec![],
    };
    let result = gen.generate_const(&c).unwrap();
    assert!(result.starts_with("const PI = "));
    assert!(!result.contains("export"));
}

#[test]
fn test_global_generation() {
    let mut gen = JsCodeGenerator::new();
    let g = GlobalDef {
        name: Spanned::new("counter".to_string(), Span::new(0, 7)),
        ty: Spanned::new(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            Span::new(9, 12),
        ),
        value: Spanned::new(Expr::Int(0), Span::new(15, 16)),
        is_pub: false,
        is_mutable: true,
    };
    let result = gen.generate_global(&g).unwrap();
    assert_eq!(result, "let counter = 0;\n");
}

#[test]
fn test_global_public() {
    let mut gen = JsCodeGenerator::new();
    let g = GlobalDef {
        name: Spanned::new("VERSION".to_string(), Span::new(0, 7)),
        ty: Spanned::new(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            Span::new(9, 12),
        ),
        value: Spanned::new(Expr::Int(1), Span::new(15, 16)),
        is_pub: true,
        is_mutable: false,
    };
    let result = gen.generate_global(&g).unwrap();
    assert_eq!(result, "export let VERSION = 1;\n");
}

#[test]
fn test_extern_block_generation() {
    let mut gen = JsCodeGenerator::new();
    let ext = ExternBlock {
        abi: "C".to_string(),
        functions: vec![
            ExternFunction {
                name: Spanned::new("malloc".to_string(), Span::new(0, 6)),
                params: vec![],
                ret_type: None,
                is_vararg: false,
                attributes: vec![],
            },
            ExternFunction {
                name: Spanned::new("free".to_string(), Span::new(10, 14)),
                params: vec![],
                ret_type: None,
                is_vararg: false,
                attributes: vec![],
            },
        ],
    };
    let result = gen.generate_extern_block(&ext).unwrap();
    assert!(result.contains("/* extern: malloc"));
    assert!(result.contains("/* extern: free"));
    assert!(result.contains("must be provided at runtime"));
}

#[test]
fn test_union_generation() {
    let mut gen = JsCodeGenerator::new();
    let u = Union {
        name: Spanned::new("Data".to_string(), Span::new(0, 4)),
        generics: vec![],
        fields: vec![
            Field {
                name: Spanned::new("int_val".to_string(), Span::new(5, 12)),
                ty: Spanned::new(
                    Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    Span::new(14, 17),
                ),
                is_pub: true,
            },
            Field {
                name: Spanned::new("float_val".to_string(), Span::new(19, 28)),
                ty: Spanned::new(
                    Type::Named {
                        name: "f64".to_string(),
                        generics: vec![],
                    },
                    Span::new(30, 33),
                ),
                is_pub: true,
            },
        ],
        is_pub: false,
    };
    let result = gen.generate_union(&u).unwrap();
    assert!(result.contains("class Data"));
    assert!(result.contains("constructor(value)"));
    assert!(result.contains("get int_val()"));
    assert!(result.contains("set int_val(v)"));
    assert!(result.contains("get float_val()"));
    assert!(result.contains("set float_val(v)"));
}

#[test]
fn test_union_public() {
    let mut gen = JsCodeGenerator::new();
    let u = Union {
        name: Spanned::new("PubUnion".to_string(), Span::new(0, 8)),
        generics: vec![],
        fields: vec![Field {
            name: Spanned::new("val".to_string(), Span::new(9, 12)),
            ty: Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                Span::new(14, 17),
            ),
            is_pub: true,
        }],
        is_pub: true,
    };
    let result = gen.generate_union(&u).unwrap();
    assert!(result.contains("export class PubUnion"));
}

#[test]
fn test_enum_struct_variant() {
    let mut gen = JsCodeGenerator::new();
    let e = Enum {
        name: Spanned::new("Shape".to_string(), Span::new(0, 5)),
        generics: vec![],
        variants: vec![Variant {
            name: Spanned::new("Circle".to_string(), Span::new(6, 12)),
            fields: VariantFields::Struct(vec![Field {
                name: Spanned::new("radius".to_string(), Span::new(13, 19)),
                ty: Spanned::new(
                    Type::Named {
                        name: "f64".to_string(),
                        generics: vec![],
                    },
                    Span::new(21, 24),
                ),
                is_pub: true,
            }]),
        }],
        is_pub: false,
        attributes: vec![],
    };
    let result = gen.generate_enum(&e).unwrap();
    assert!(result.contains("Circle(radius)"));
    assert!(result.contains("__tag: \"Circle\""));
}

#[test]
fn test_enum_public() {
    let mut gen = JsCodeGenerator::new();
    let e = Enum {
        name: Spanned::new("Dir".to_string(), Span::new(0, 3)),
        generics: vec![],
        variants: vec![
            Variant {
                name: Spanned::new("Up".to_string(), Span::new(4, 6)),
                fields: VariantFields::Unit,
            },
            Variant {
                name: Spanned::new("Down".to_string(), Span::new(8, 12)),
                fields: VariantFields::Unit,
            },
        ],
        is_pub: true,
        attributes: vec![],
    };
    let result = gen.generate_enum(&e).unwrap();
    assert!(result.contains("export const Dir"));
}

#[test]
fn test_type_alias_empty() {
    let mut gen = JsCodeGenerator::new();
    let item = Item::TypeAlias(TypeAlias {
        name: Spanned::new("Num".to_string(), Span::new(0, 3)),
        generics: vec![],
        ty: Spanned::new(
            Type::Named {
                name: "i64".to_string(),
                generics: vec![],
            },
            Span::new(6, 9),
        ),
        is_pub: false,
    });
    let result = gen.generate_item(&item).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_error_item() {
    let mut gen = JsCodeGenerator::new();
    let item = Item::Error {
        message: "parse failed".to_string(),
        skipped_tokens: vec![],
    };
    let result = gen.generate_item(&item).unwrap();
    assert!(result.contains("error"));
    assert!(result.contains("parse failed"));
}

#[test]
fn test_async_function() {
    let mut gen = JsCodeGenerator::new();
    let func = Function {
        name: Spanned::new("fetch_data".to_string(), Span::new(0, 10)),
        generics: vec![],
        params: vec![],
        ret_type: None,
        body: FunctionBody::Expr(Box::new(Spanned::new(Expr::Int(42), Span::new(0, 2)))),
        is_pub: false,
        is_async: true,
        attributes: vec![],
        where_clause: vec![],
    };
    let result = gen.generate_function(&func, false).unwrap();
    assert!(result.contains("async function fetch_data()"));
}

#[test]
fn test_function_with_default_param() {
    let mut gen = JsCodeGenerator::new();
    let func = Function {
        name: Spanned::new("greet".to_string(), Span::new(0, 5)),
        generics: vec![],
        params: vec![Param {
            name: Spanned::new("name".to_string(), Span::new(6, 10)),
            ty: Spanned::new(
                Type::Named {
                    name: "str".to_string(),
                    generics: vec![],
                },
                Span::new(12, 15),
            ),
            is_mut: false,
            is_vararg: false,
            ownership: Ownership::Regular,
            default_value: Some(Box::new(Spanned::new(
                Expr::String("world".to_string()),
                Span::new(18, 25),
            ))),
        }],
        ret_type: None,
        body: FunctionBody::Expr(Box::new(Spanned::new(
            Expr::Ident("name".to_string()),
            Span::new(28, 32),
        ))),
        is_pub: false,
        is_async: false,
        attributes: vec![],
        where_clause: vec![],
    };
    let result = gen.generate_function(&func, false).unwrap();
    assert!(result.contains("name = \"world\""));
}

#[test]
fn test_struct_with_method() {
    let mut gen = JsCodeGenerator::new();
    let s = Struct {
        name: Spanned::new("Counter".to_string(), Span::new(0, 7)),
        generics: vec![],
        fields: vec![Field {
            name: Spanned::new("count".to_string(), Span::new(8, 13)),
            ty: Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                Span::new(15, 18),
            ),
            is_pub: true,
        }],
        methods: vec![Spanned::new(
            Function {
                name: Spanned::new("inc".to_string(), Span::new(20, 23)),
                generics: vec![],
                params: vec![],
                ret_type: None,
                body: FunctionBody::Expr(Box::new(Spanned::new(
                    Expr::Int(1),
                    Span::new(26, 27),
                ))),
                is_pub: false,
                is_async: false,
                attributes: vec![],
                where_clause: vec![],
            },
            Span::new(20, 28),
        )],
        is_pub: false,
        attributes: vec![],
        where_clause: vec![],
    };
    let result = gen.generate_struct(&s).unwrap();
    assert!(result.contains("class Counter"));
    assert!(result.contains("constructor(count)"));
    assert!(result.contains("this.count = count;"));
    assert!(result.contains("inc()"));
}

#[test]
fn test_trait_with_default_method() {
    let mut gen = JsCodeGenerator::new();
    let t = Trait {
        name: Spanned::new("Greet".to_string(), Span::new(0, 5)),
        generics: vec![],
        super_traits: vec![],
        methods: vec![TraitMethod {
            name: Spanned::new("greet".to_string(), Span::new(6, 11)),
            generics: vec![],
            params: vec![],
            ret_type: None,
            is_async: false,
            is_const: false,
            default_body: Some(FunctionBody::Expr(Box::new(Spanned::new(
                Expr::String("hello".to_string()),
                Span::new(14, 21),
            )))),
        }],
        associated_types: vec![],
        is_pub: false,
        where_clause: vec![],
    };
    let result = gen.generate_trait(&t).unwrap();
    assert!(result.contains("class Greet"));
    assert!(result.contains("greet()"));
    assert!(result.contains("return \"hello\""));
}

#[test]
fn test_trait_abstract_method() {
    let mut gen = JsCodeGenerator::new();
    let t = Trait {
        name: Spanned::new("Drawable".to_string(), Span::new(0, 8)),
        generics: vec![],
        super_traits: vec![],
        methods: vec![TraitMethod {
            name: Spanned::new("draw".to_string(), Span::new(9, 13)),
            generics: vec![],
            params: vec![],
            ret_type: None,
            is_async: false,
            is_const: false,
            default_body: None,
        }],
        associated_types: vec![],
        is_pub: false,
        where_clause: vec![],
    };
    let result = gen.generate_trait(&t).unwrap();
    assert!(result.contains("draw()"));
    assert!(result.contains("throw new Error"));
    assert!(result.contains("Drawable.draw not implemented"));
}

#[test]
fn test_trait_public() {
    let mut gen = JsCodeGenerator::new();
    let t = Trait {
        name: Spanned::new("PubTrait".to_string(), Span::new(0, 8)),
        generics: vec![],
        super_traits: vec![],
        methods: vec![],
        associated_types: vec![],
        is_pub: true,
        where_clause: vec![],
    };
    let result = gen.generate_trait(&t).unwrap();
    assert!(result.contains("export class PubTrait"));
}

#[test]
fn test_impl_static_method() {
    let mut gen = JsCodeGenerator::new();
    let imp = Impl {
        trait_name: None,
        target_type: Spanned::new(
            Type::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            Span::new(0, 3),
        ),
        generics: vec![],
        associated_types: vec![],
        methods: vec![Spanned::new(
            Function {
                name: Spanned::new("new".to_string(), Span::new(4, 7)),
                generics: vec![],
                params: vec![], // No self param = static method
                ret_type: None,
                body: FunctionBody::Expr(Box::new(Spanned::new(
                    Expr::Array(vec![]),
                    Span::new(10, 12),
                ))),
                is_pub: false,
                is_async: false,
                attributes: vec![],
                where_clause: vec![],
            },
            Span::new(4, 13),
        )],
    };
    let result = gen.generate_impl(&imp).unwrap();
    assert!(result.contains("Vec._new = function()"));
    // Should NOT be on prototype (no self param)
    assert!(!result.contains("prototype._new"));
}

#[test]
fn test_impl_without_trait() {
    let mut gen = JsCodeGenerator::new();
    let imp = Impl {
        trait_name: None,
        target_type: Spanned::new(
            Type::Named {
                name: "Point".to_string(),
                generics: vec![],
            },
            Span::new(0, 5),
        ),
        generics: vec![],
        associated_types: vec![],
        methods: vec![],
    };
    let result = gen.generate_impl(&imp).unwrap();
    // No trait = no __implements tracking
    assert!(!result.contains("__implements"));
}

#[test]
fn test_impl_non_named_type() {
    let mut gen = JsCodeGenerator::new();
    let imp = Impl {
        trait_name: None,
        target_type: Spanned::new(Type::Unit, Span::new(0, 2)),
        generics: vec![],
        associated_types: vec![],
        methods: vec![],
    };
    let result = gen.generate_impl(&imp).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_macro_item_empty() {
    let mut gen = JsCodeGenerator::new();
    let item = Item::Macro(MacroDef {
        name: Spanned::new("my_macro".to_string(), Span::new(0, 8)),
        rules: vec![],
        is_pub: false,
    });
    let result = gen.generate_item(&item).unwrap();
    assert_eq!(result, "");
}
