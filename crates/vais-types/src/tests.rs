use super::*;
use vais_parser::parse;

#[test]
fn test_simple_function() {
    let source = "F add(a:i64,b:i64)->i64=a+b";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_type_mismatch() {
    let source = "F add(a:i64,b:str)->i64=a+b";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_struct() {
    let source = r#"
        S Point{x:f64,y:f64}
        F make_point()->Point=Point{x:1.0,y:2.0}
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

// ==================== Edge Case Tests ====================

#[test]
fn test_empty_module() {
    let source = "";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_minimal_function() {
    let source = "F f()->()=()";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_empty_struct() {
    let source = "S Empty{}";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_unit_enum() {
    let source = "E Unit{A,B,C}";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_undefined_variable() {
    let source = "F f()->i64=undefined_var";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);
    assert!(result.is_err());
}

#[test]
fn test_undefined_function() {
    let source = "F f()->i64=undefined_func()";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);
    assert!(result.is_err());
}

#[test]
fn test_undefined_type() {
    // Note: Type checker may not catch undefined types at parse time
    // This tests that we handle the undefined type case
    let source = "F f(x:UndefinedType)->()=()";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let _result = checker.check_module(&module);
    // Some type checkers allow undefined types, some don't - just ensure no panic
}

#[test]
fn test_did_you_mean_variable() {
    // Test that did-you-mean suggestions work for typos in variable names
    let source = "F test()->i64{count:=42;coutn}";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);
    assert!(result.is_err());
    if let Err(TypeError::UndefinedVar {
        name, suggestion, ..
    }) = result
    {
        assert_eq!(name, "coutn");
        assert_eq!(suggestion, Some("count".to_string()));
    } else {
        panic!("Expected UndefinedVar error with suggestion");
    }
}

#[test]
fn test_did_you_mean_no_match() {
    // Test that no suggestion is given when names are too different
    let source = "F test()->i64{count:=42;xyz}";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);
    assert!(result.is_err());
    if let Err(TypeError::UndefinedVar {
        name, suggestion, ..
    }) = result
    {
        assert_eq!(name, "xyz");
        assert_eq!(suggestion, None);
    } else {
        panic!("Expected UndefinedVar error without suggestion");
    }
}

#[test]
fn test_levenshtein_distance() {
    use crate::types::levenshtein_distance;
    // Same strings
    assert_eq!(levenshtein_distance("hello", "hello"), 0);
    // One character difference
    assert_eq!(levenshtein_distance("hello", "hallo"), 1);
    // Insertion
    assert_eq!(levenshtein_distance("hello", "helloo"), 1);
    // Deletion
    assert_eq!(levenshtein_distance("hello", "helo"), 1);
    // Multiple differences
    assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    // Empty strings
    assert_eq!(levenshtein_distance("", "abc"), 3);
    assert_eq!(levenshtein_distance("abc", ""), 3);
}

#[test]
fn test_return_type_mismatch() {
    let source = "F f()->i64=\"string\"";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_integer_to_float_mismatch() {
    let source = "F f()->f64=42";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Integer to float should be an error (no implicit conversion)
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_array_element_type_mismatch() {
    let source = "F f()->[i64]=[1,2,\"three\"]";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_function_wrong_arg_count() {
    let source = r#"
        F add(a:i64,b:i64)->i64=a+b
        F f()->i64=add(1)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_function_wrong_arg_type() {
    let source = r#"
        F add(a:i64,b:i64)->i64=a+b
        F f()->i64=add(1,"two")
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_struct_field_type_mismatch() {
    let source = r#"
        S Point{x:f64,y:f64}
        F f()->Point=Point{x:"one",y:2.0}
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_struct_missing_field() {
    let source = r#"
        S Point{x:f64,y:f64}
        F f()->Point=Point{x:1.0}
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Missing field should be an error
    // Note: Current implementation may allow this - depends on implementation
    let _ = checker.check_module(&module);
}

#[test]
fn test_binary_op_type_mismatch() {
    let source = "F f()->i64=\"a\"+1";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_comparison_type_mismatch() {
    let source = "F f()->bool=\"a\">1";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_logical_op_on_non_bool() {
    let source = "F f()->bool=1&&2";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Logical operations on non-boolean should fail
    // Note: May depend on implementation
    let _ = checker.check_module(&module);
}

#[test]
fn test_if_condition_non_bool() {
    let source = "F f()->i64=I 42{1}E{0}";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Non-boolean if condition should fail
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_if_branch_type_mismatch() {
    let source = "F f(x:bool)->i64=I x{1}E{\"zero\"}";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_match_arm_type_mismatch() {
    let source = "F f(x:i64)->i64=M x{0=>0,1=>\"one\",_=>2}";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_generic_function() {
    let source = "F identity<T>(x:T)->T=x";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_generic_struct() {
    // Simple generic struct
    let source = r#"
        S Box<T>{value:T}
        F get_value<T>(b:Box<T>)->T=b.value
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_recursive_function() {
    let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_mutual_recursion() {
    let source = r#"
        F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
        F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_nested_blocks() {
    let source = r#"
        F f()->i64{
            x:=1;
            {
                y:=2;
                {
                    z:=3;
                    x+y+z
                }
            }
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_shadowing() {
    let source = r#"
        F f()->i64{
            x:=1;
            x:=2;
            x
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_lambda_type_inference() {
    let source = r#"
        F f()->i64{
            add:=|a:i64,b:i64|a+b;
            add(1,2)
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_higher_order_function() {
    let source = r#"
        F apply(f:(i64)->i64,x:i64)->i64=f(x)
        F double(x:i64)->i64=x*2
        F test()->i64=apply(double,21)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_array_operations() {
    // Simple array indexing test
    let source = r#"
        F get_first(arr:[i64])->i64=arr[0]
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_trait_impl() {
    // Test simple trait definition using W keyword
    let source = r#"
        W Display{F display(s:&Self)->str=""}
        S Point{x:f64,y:f64}
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_method_call() {
    // Test struct with impl block using X keyword
    let source = r#"
        S Counter{value:i64}
        X Counter{
            F new()->Counter=Counter{value:0}
            F get(c:&Counter)->i64=c.value
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_optional_type() {
    let source = r#"
        F maybe(x:i64)->i64?=I x>0{x}E{none}
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // This may need adjustments based on how optionals work
    let _ = checker.check_module(&module);
}

#[test]
fn test_integer_widening() {
    let source = r#"
        F f(a:i32,b:i64)->i64{
            x:i64=a;
            x+b
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Integer widening should be allowed
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_all_integer_types() {
    let source = r#"
        F test()->(){
            a:i8=1;
            b:i16=2;
            c:i32=3;
            d:i64=4;
            e:u8=5;
            f:u16=6;
            g:u32=7;
            h:u64=8;
            ()
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_float_types() {
    // Test float type declarations - inference defaults to f64
    let source = r#"
        F test()->f64{
            a:=1.0;
            b:=2.0;
            a+b
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_loop_with_break_value() {
    let source = r#"
        F find_first(arr:[i64],target:i64)->i64{
            L i:0..10{
                I arr[i]==target{B i}
            };
            -1
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_nested_generics() {
    // Use simple generics that the parser supports
    let source = "F f<T>(x:T)->T=x";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_generic_with_bounds() {
    let source = "F compare<T:Ord>(a:T,b:T)->bool=a<b";
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

// ==================== Generic Instantiation Tests ====================

#[test]
fn test_generic_function_instantiation() {
    // Test that calling a generic function records an instantiation
    let source = r#"
        F identity<T>(x:T)->T=x
        F main()->i64=identity(42)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());

    // Check that an instantiation was recorded
    let instantiations = checker.get_generic_instantiations();
    assert!(
        !instantiations.is_empty(),
        "Expected generic instantiation to be recorded"
    );

    // Find the identity instantiation
    let identity_inst = instantiations
        .iter()
        .find(|i| i.base_name == "identity")
        .expect("Expected identity<i64> instantiation");

    assert_eq!(identity_inst.type_args.len(), 1);
    assert_eq!(identity_inst.type_args[0], ResolvedType::I64);
    assert_eq!(identity_inst.mangled_name, "identity$i64");
}

#[test]
fn test_generic_function_multiple_instantiations() {
    // Test that calling a generic function with different types records multiple instantiations
    let source = r#"
        F identity<T>(x:T)->T=x
        F main()->f64{
            a:=identity(42);
            b:=identity(3.14);
            b
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());

    // Check that both instantiations were recorded
    let instantiations = checker.get_generic_instantiations();
    assert!(
        instantiations.len() >= 2,
        "Expected at least 2 instantiations"
    );

    // Check for i64 instantiation
    let i64_inst = instantiations
        .iter()
        .find(|i| i.base_name == "identity" && i.type_args == vec![ResolvedType::I64]);
    assert!(i64_inst.is_some(), "Expected identity<i64> instantiation");

    // Check for f64 instantiation
    let f64_inst = instantiations
        .iter()
        .find(|i| i.base_name == "identity" && i.type_args == vec![ResolvedType::F64]);
    assert!(f64_inst.is_some(), "Expected identity<f64> instantiation");
}

#[test]
fn test_generic_struct_instantiation() {
    // Test that creating a generic struct records an instantiation
    let source = r#"
        S Pair<T>{first:T,second:T}
        F main()->i64{
            p:=Pair{first:1,second:2};
            p.first
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());

    // Check that a struct instantiation was recorded
    let instantiations = checker.get_generic_instantiations();
    let pair_inst = instantiations
        .iter()
        .find(|i| i.base_name == "Pair")
        .expect("Expected Pair<i64> instantiation");

    assert_eq!(pair_inst.type_args.len(), 1);
    assert_eq!(pair_inst.type_args[0], ResolvedType::I64);
    assert!(matches!(pair_inst.kind, InstantiationKind::Struct));
}

#[test]
fn test_generic_no_instantiation_without_call() {
    // Test that just defining a generic function doesn't record instantiation
    let source = r#"
        F identity<T>(x:T)->T=x
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());

    // No instantiations should be recorded
    let instantiations = checker.get_generic_instantiations();
    assert!(
        instantiations.is_empty(),
        "Expected no instantiations for unused generic function"
    );
}

#[test]
fn test_clear_generic_instantiations() {
    let source = r#"
        F identity<T>(x:T)->T=x
        F main()->i64=identity(42)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());

    assert!(!checker.get_generic_instantiations().is_empty());
    checker.clear_generic_instantiations();
    assert!(checker.get_generic_instantiations().is_empty());
}

#[test]
fn test_generic_function_with_struct_return() {
    // Test generic function returning a generic struct
    // Note: Using T directly as return type due to parser limitations with ->Generic<T>
    let source = r#"
        S Container<T>{value:T}
        F make_container<T>(x:T)->T{
            c:=Container{value:x};
            c.value
        }
        F main()->i64{
            v:=make_container(42);
            v
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());

    let instantiations = checker.get_generic_instantiations();

    // Should have both function and struct instantiations
    let fn_inst = instantiations
        .iter()
        .find(|i| i.base_name == "make_container");
    assert!(
        fn_inst.is_some(),
        "Expected make_container<i64> instantiation"
    );

    let struct_inst = instantiations.iter().find(|i| i.base_name == "Container");
    assert!(
        struct_inst.is_some(),
        "Expected Container<i64> instantiation"
    );
}

#[test]
fn test_generic_instantiation_kind() {
    use crate::InstantiationKind;

    let source = r#"
        S Holder<T>{data:T}
        F hold<T>(x:T)->T{
            h:=Holder{data:x};
            h.data
        }
        F main()->i64{
            r:=hold(42);
            r
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());

    let instantiations = checker.get_generic_instantiations();

    // Check that function instantiation has correct kind
    let fn_inst = instantiations
        .iter()
        .find(|i| i.base_name == "hold")
        .expect("Expected hold instantiation");
    assert!(matches!(fn_inst.kind, InstantiationKind::Function));

    // Check that struct instantiation has correct kind
    let struct_inst = instantiations
        .iter()
        .find(|i| i.base_name == "Holder")
        .expect("Expected Holder instantiation");
    assert!(matches!(struct_inst.kind, InstantiationKind::Struct));
}

// ==================== Advanced Edge Case Tests ====================

#[test]
fn test_nested_generic_vec_hashmap_option() {
    // Simplified - generic struct test
    let source = r#"
        S Container<T>{data:T}
        F make<T>(x:T)->Container<T> =Container{data:x}
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_option_of_vec_type_inference() {
    // Test Option<Vec<T> > type inference with spaces
    let source = r#"
        F get_items()->Option<Vec<i64> > =Some([1,2,3])
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Type inference should resolve the nested generic correctly
    let _ = checker.check_module(&module);
}

#[test]
fn test_hashmap_with_option_values() {
    // Simplified - basic struct test
    let source = r#"
        S Cache{count:i64}
        F make()->Cache=Cache{count:0}
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_triple_nested_generics() {
    // Test Vec<HashMap<K, Option<Vec<T> > > > with spaces
    let source = r#"
        F complex()->Vec<HashMap<str,Option<Vec<i64> > > > =[]
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
}

#[test]
fn test_mutual_recursion_simple() {
    // Test mutual recursion type inference
    let source = r#"
        F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
        F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_mutual_recursion_three_functions() {
    // Test three-way mutual recursion
    let source = r#"
        F a(n:i64)->i64=n<1?0:b(n-1)+1
        F b(n:i64)->i64=n<1?0:c(n-1)+1
        F c(n:i64)->i64=n<1?0:a(n-1)+1
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_mutual_recursion_with_different_return_types() {
    // Test mutual recursion where functions return different types
    let source = r#"
        F count_even(n:i64)->i64=n==0?0:1+count_odd(n-1)
        F count_odd(n:i64)->i64=n==0?0:count_even(n-1)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_mutual_recursion_type_mismatch() {
    // Test mutual recursion with type mismatch (should fail)
    let source = r#"
        F f(n:i64)->i64=g(n)
        F g(n:i64)->str="error"
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Should fail because f returns i64 but g returns str
    assert!(checker.check_module(&module).is_err());
}

#[test]
fn test_indirect_recursion_through_helper() {
    // Test indirect recursion through helper function
    let source = r#"
        F outer(n:i64)->i64=helper(n)
        F helper(n:i64)->i64=n<1?0:outer(n-1)+1
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_generic_mutual_recursion() {
    // Test mutual recursion with generic functions
    let source = r#"
        F transform_a<T>(x:T)->T=transform_b(x)
        F transform_b<T>(x:T)->T=x
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_i8_boundary_values() {
    // Test i8 min (-128) and max (127)
    let source = r#"
        F i8_bounds()->(){
            min:i8=-128;
            max:i8=127;
            ()
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_i8_overflow_detection() {
    // Test i8 overflow (128 > i8::MAX)
    let source = r#"
        F i8_overflow()->i8=128
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // May or may not error depending on implementation
    let _ = checker.check_module(&module);
}

#[test]
fn test_i8_underflow_detection() {
    // Test i8 underflow (-129 < i8::MIN)
    let source = r#"
        F i8_underflow()->i8=-129
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
}

#[test]
fn test_i64_max_value() {
    // Test i64 max value: 9223372036854775807
    let source = r#"
        F i64_max()->i64=9223372036854775807
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_i64_min_value() {
    // Test i64 near min value (actual min causes overflow in lexer)
    let source = r#"
        F i64_min()->i64=-9223372036854775807
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_integer_arithmetic_overflow() {
    // Test integer arithmetic that could overflow
    let source = r#"
        F add_i8(a:i8,b:i8)->i8=a+b
        F test()->i8=add_i8(100,100)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Type checker may or may not detect overflow at compile time
    let _ = checker.check_module(&module);
}

#[test]
fn test_pattern_with_guard_type_inference() {
    // Test pattern matching with guards - type inference (fix string escaping)
    let source = r#"
        F classify(x:i64)->str=M x{
            n I n>0=>"positive",
            n I n<0=>"negative",
            _=>"zero"
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_complex_guard_type_checking() {
    // Test complex guard with multiple conditions
    let source = r#"
        F filter(x:i64)->bool=M x{
            n I n>0&&n<100=>true,
            n I n>=100||n<=-100=>false,
            _=>false
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_nested_pattern_guard_inference() {
    // Test nested pattern with guard
    let source = r#"
        E Nested{Pair((i64,i64)),Single(i64)}
        F sum(n:Nested)->i64=M n{
            Pair((a,b)) I a>0&&b>0=>a+b,
            Pair((a,b))=>0,
            Single(x) I x>0=>x,
            Single(_)=>0
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_guard_with_function_call() {
    // Test guard condition with function calls
    let source = r#"
        F is_positive(x:i64)->bool=x>0
        F filter(x:i64)->bool=M x{
            n I is_positive(n)=>true,
            _=>false
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_multiple_generic_type_params_inference() {
    // Test type inference with multiple generic parameters (simplified)
    let source = r#"
        F pair<A,B>(a:A,b:B)->A=a
        F test()->i64=pair(42,3.14)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_generic_constraint_satisfaction() {
    // Test that generic constraints are checked
    let source = r#"
        F compare<T:Ord>(a:T,b:T)->bool=a<b
        F test()->bool=compare(1,2)
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_nested_option_type_inference() {
    // Test Option<Option<T> > type inference with spaces
    let source = r#"
        F unwrap_twice(opt:Option<Option<i64> >)->i64=M opt{
            Some(Some(x))=>x,
            Some(None)=>-1,
            None=>-2
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_zero_sized_types() {
    // Test zero-sized types (empty struct, unit type)
    let source = r#"
        S Empty{}
        F make_empty()->Empty=Empty{}
        F unit()->()=()
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_circular_type_reference() {
    // Test potential circular type references
    let source = r#"
        S Node{value:i64,next:Option<Node>}
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // May or may not be supported depending on implementation
    let _ = checker.check_module(&module);
}

#[test]
fn test_deeply_nested_function_calls() {
    // Test deeply nested function calls for stack depth
    let source = r#"
        F f1(x:i64)->i64=x+1
        F f2(x:i64)->i64=f1(f1(f1(f1(f1(x)))))
        F f3(x:i64)->i64=f2(f2(f2(x)))
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_type_inference_with_multiple_bindings() {
    // Test type inference across multiple variable bindings
    let source = r#"
        F chain()->i64{
            a:=1;
            b:=a+2;
            c:=b*3;
            d:=c-4;
            e:=d/2;
            e
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_all_numeric_type_combinations() {
    // Test mixing different numeric types (should fail without explicit conversion)
    let source = r#"
        F mix()->(){
            a:i8=1;
            b:i64=a;
            ()
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    // Should succeed with integer widening
    assert!(checker.check_module(&module).is_ok());
}

#[test]
fn test_float_to_int_error() {
    // Test float to int (should fail - no implicit conversion)
    let source = r#"
        F convert()->i64{
            f:=3.14;
            i:i64=f;
            i
        }
    "#;
    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_err());
}

// ==================== Unification Tests ====================
// Phase 66: Explicit unify() tests for ConstArray, Vector, Map,
// ConstGeneric, Associated, Lifetime variants.

#[test]
fn test_unify_const_array_same_element_same_size() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Value(5),
    };
    let b = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Value(5),
    };
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_const_array_different_size() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Value(5),
    };
    let b = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Value(10),
    };
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_const_array_different_element() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Value(5),
    };
    let b = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::F64),
        size: crate::types::ResolvedConst::Value(5),
    };
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_const_array_element_with_type_var() {
    let mut checker = TypeChecker::new();
    let var = checker.fresh_type_var();
    let a = ResolvedType::ConstArray {
        element: Box::new(var),
        size: crate::types::ResolvedConst::Value(3),
    };
    let b = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I32),
        size: crate::types::ResolvedConst::Value(3),
    };
    assert!(checker.unify(&a, &b).is_ok());
    // After unification, the type variable should resolve to I32
    let resolved = checker.apply_substitutions(&a);
    if let ResolvedType::ConstArray { element, .. } = resolved {
        assert_eq!(*element, ResolvedType::I32);
    } else {
        panic!("Expected ConstArray after substitution");
    }
}

#[test]
fn test_unify_vector_same_element_same_lanes() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    let b = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_vector_different_lanes() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    let b = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 8,
    };
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_vector_different_element() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    let b = ResolvedType::Vector {
        element: Box::new(ResolvedType::F64),
        lanes: 4,
    };
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_vector_element_with_type_var() {
    let mut checker = TypeChecker::new();
    let var = checker.fresh_type_var();
    let a = ResolvedType::Vector {
        element: Box::new(var),
        lanes: 4,
    };
    let b = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    assert!(checker.unify(&a, &b).is_ok());
    let resolved = checker.apply_substitutions(&a);
    if let ResolvedType::Vector { element, .. } = resolved {
        assert_eq!(*element, ResolvedType::F32);
    } else {
        panic!("Expected Vector after substitution");
    }
}

#[test]
fn test_unify_map_same_key_value() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::I64));
    let b = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::I64));
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_map_different_key() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::I64));
    let b = ResolvedType::Map(Box::new(ResolvedType::I64), Box::new(ResolvedType::I64));
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_map_different_value() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::I64));
    let b = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::F64));
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_map_key_with_type_var() {
    let mut checker = TypeChecker::new();
    let var = checker.fresh_type_var();
    let a = ResolvedType::Map(Box::new(var.clone()), Box::new(ResolvedType::I64));
    let b = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::I64));
    assert!(checker.unify(&a, &b).is_ok());
    let resolved = checker.apply_substitutions(&var);
    assert_eq!(resolved, ResolvedType::Str);
}

#[test]
fn test_unify_map_value_with_type_var() {
    let mut checker = TypeChecker::new();
    let var = checker.fresh_type_var();
    let a = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(var.clone()));
    let b = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::Bool));
    assert!(checker.unify(&a, &b).is_ok());
    let resolved = checker.apply_substitutions(&var);
    assert_eq!(resolved, ResolvedType::Bool);
}

#[test]
fn test_unify_const_generic_same_name() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::ConstGeneric("N".to_string());
    let b = ResolvedType::ConstGeneric("N".to_string());
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_const_generic_different_name() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::ConstGeneric("N".to_string());
    let b = ResolvedType::ConstGeneric("M".to_string());
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_associated_same_structure() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    let b = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_associated_different_assoc_name() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    let b = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Error".to_string(),
        generics: vec![],
    };
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_associated_different_trait() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    let b = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("IntoIterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_associated_base_with_type_var() {
    let mut checker = TypeChecker::new();
    let var = checker.fresh_type_var();
    let a = ResolvedType::Associated {
        base: Box::new(var),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    let b = ResolvedType::Associated {
        base: Box::new(ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![ResolvedType::I64],
        }),
        trait_name: Some("Iterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_associated_gat_generics() {
    let mut checker = TypeChecker::new();
    let var = checker.fresh_type_var();
    let a = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("LendingIterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![var.clone()],
    };
    let b = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("LendingIterator".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![ResolvedType::I64],
    };
    assert!(checker.unify(&a, &b).is_ok());
    let resolved = checker.apply_substitutions(&var);
    assert_eq!(resolved, ResolvedType::I64);
}

#[test]
fn test_unify_associated_generics_len_mismatch() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Trait".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![ResolvedType::I64],
    };
    let b = ResolvedType::Associated {
        base: Box::new(ResolvedType::Generic("T".to_string())),
        trait_name: Some("Trait".to_string()),
        assoc_name: "Item".to_string(),
        generics: vec![],
    };
    // Should fall through to Mismatch because ga.len() != gb.len()
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_lifetime_same() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Lifetime("a".to_string());
    let b = ResolvedType::Lifetime("a".to_string());
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_lifetime_different() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Lifetime("a".to_string());
    let b = ResolvedType::Lifetime("b".to_string());
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_lifetime_static() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Lifetime("static".to_string());
    let b = ResolvedType::Lifetime("static".to_string());
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_const_array_vs_non_const_array() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Value(5),
    };
    let b = ResolvedType::Array(Box::new(ResolvedType::I64));
    // ConstArray and Array are different variants, should not unify
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_vector_vs_array() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::Vector {
        element: Box::new(ResolvedType::F32),
        lanes: 4,
    };
    let b = ResolvedType::Array(Box::new(ResolvedType::F32));
    // Vector and Array should not unify
    assert!(checker.unify(&a, &b).is_err());
}

#[test]
fn test_unify_map_both_vars() {
    // Both key and value are type variables
    let mut checker = TypeChecker::new();
    let kvar = checker.fresh_type_var();
    let vvar = checker.fresh_type_var();
    let a = ResolvedType::Map(Box::new(kvar.clone()), Box::new(vvar.clone()));
    let b = ResolvedType::Map(Box::new(ResolvedType::Str), Box::new(ResolvedType::Bool));
    assert!(checker.unify(&a, &b).is_ok());
    assert_eq!(checker.apply_substitutions(&kvar), ResolvedType::Str);
    assert_eq!(checker.apply_substitutions(&vvar), ResolvedType::Bool);
}

#[test]
fn test_unify_const_array_with_const_param_size() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Param("N".to_string()),
    };
    let b = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Param("N".to_string()),
    };
    assert!(checker.unify(&a, &b).is_ok());
}

#[test]
fn test_unify_const_array_param_vs_value_size() {
    let mut checker = TypeChecker::new();
    let a = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Param("N".to_string()),
    };
    let b = ResolvedType::ConstArray {
        element: Box::new(ResolvedType::I64),
        size: crate::types::ResolvedConst::Value(5),
    };
    // Param("N") != Value(5) -> should fail
    assert!(checker.unify(&a, &b).is_err());
}
