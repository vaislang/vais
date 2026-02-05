//! Fuzz tests for the type checker
//!
//! These tests ensure that the type checker never panics on any AST input,
//! even if it returns type errors. The goal is robustness - any AST should
//! either type check successfully or return an error, but never crash.

use std::panic::{catch_unwind, AssertUnwindSafe};
use vais_parser::parse;
use vais_types::TypeChecker;

/// Simple Linear Congruential Generator for reproducible random numbers
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        if max <= min {
            return min;
        }
        min + (self.next() as usize % (max - min))
    }

    fn gen_type(&mut self) -> &'static str {
        let types = ["i64", "f64", "bool", "str", "()", "i32", "u64"];
        types[self.next_range(0, types.len())]
    }

    fn gen_identifier(&mut self) -> String {
        let len = self.next_range(1, 20);
        let mut id = String::new();
        for i in 0..len {
            let c = if i == 0 {
                // First char must be letter or underscore
                let chars = "abcdefghijklmnopqrstuvwxyz_";
                chars.chars().nth(self.next_range(0, chars.len())).unwrap()
            } else {
                // Subsequent chars can be alphanumeric or underscore
                let chars = "abcdefghijklmnopqrstuvwxyz0123456789_";
                chars.chars().nth(self.next_range(0, chars.len())).unwrap()
            };
            id.push(c);
        }
        id
    }
}

/// Helper function to run type checker with panic catching
fn type_check_no_panic(source: &str) -> Result<bool, String> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        // First parse the source
        let module = match parse(source) {
            Ok(m) => m,
            Err(_) => return, // Parse error is fine, not testing parser here
        };

        // Then type check
        let mut checker = TypeChecker::new();
        let _ = checker.check_module(&module); // Ignore the result - we just care about panics
    }));

    match result {
        Ok(_) => Ok(true), // Type checker either succeeded or returned an error - both are fine
        Err(panic_info) => {
            // Extract panic message
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            Err(msg)
        }
    }
}

#[test]
fn fuzz_functions_with_many_parameters() {
    let mut failures = Vec::new();

    for param_count in [10, 50, 100, 200] {
        // Generate function with many parameters
        let mut source = String::from("F test(");
        for i in 0..param_count {
            if i > 0 {
                source.push_str(", ");
            }
            source.push_str(&format!("p{}:i64", i));
        }
        source.push_str(")->i64=42");

        if let Err(panic_msg) = type_check_no_panic(&source) {
            failures.push((format!("{} params", param_count), panic_msg));
        }

        // Function call with many arguments
        let mut call_source = String::from("F f(");
        for i in 0..param_count {
            if i > 0 {
                call_source.push_str(", ");
            }
            call_source.push_str(&format!("p{}:i64", i));
        }
        call_source.push_str(")->i64=0 F test()->i64=f(");
        for i in 0..param_count {
            if i > 0 {
                call_source.push_str(", ");
            }
            call_source.push_str(&i.to_string());
        }
        call_source.push(')');

        if let Err(panic_msg) = type_check_no_panic(&call_source) {
            failures.push((format!("{} args in call", param_count), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (desc, panic_msg) in &failures {
            eprintln!("\n{}: PANIC!", desc);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} many-parameter tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_deeply_nested_types() {
    let mut failures = Vec::new();

    for depth in [5, 10, 20, 50] {
        // Nested Option types: Option<Option<Option<...>>>
        let mut type_str = String::from("i64");
        for _ in 0..depth {
            type_str = format!("Option<{}>", type_str);
        }
        let source = format!("E Option<T>{{Some(T),None}} F test()->{}=None", type_str);

        if let Err(panic_msg) = type_check_no_panic(&source) {
            failures.push((format!("Option depth {}", depth), panic_msg));
        }

        // Nested tuple types: ((((i64, i64), i64), i64), i64)
        let mut tuple_type = String::from("i64");
        for _ in 0..depth {
            tuple_type = format!("({}, i64)", tuple_type);
        }
        let source2 = format!("F test()->{}=(42, 42)", tuple_type);

        if let Err(panic_msg) = type_check_no_panic(&source2) {
            failures.push((format!("Tuple depth {}", depth), panic_msg));
        }

        // Nested function types: fn(fn(fn(i64)->i64)->i64)->i64
        let mut fn_type = String::from("i64");
        for _ in 0..depth {
            fn_type = format!("fn({})->i64", fn_type);
        }
        let source3 = format!("F test(f:{})->i64=42", fn_type);

        if let Err(panic_msg) = type_check_no_panic(&source3) {
            failures.push((format!("Function type depth {}", depth), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (desc, panic_msg) in &failures {
            eprintln!("\n{}: PANIC!", desc);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} deeply nested type tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_circular_type_references() {
    let test_cases = vec![
        // Type alias referring to itself
        "Y A = A",
        // Mutual type alias references
        "Y A = B Y B = A",
        // Struct with field of its own type
        "S Node { value: i64, next: Node }",
        // Mutual struct references
        "S A { b: B } S B { a: A }",
        // Enum with recursive variant
        "E List { Cons(i64, List), Nil }",
        // Type alias chain
        "Y A = B Y B = C Y C = A",
        // Complex circular dependency
        "S Foo { bar: Bar } S Bar { baz: Baz } S Baz { foo: Foo }",
        // Self-referential generic
        "E Tree<T> { Node(T, Tree<Tree<T>>), Leaf }",
    ];

    let mut failures = Vec::new();

    for (i, source) in test_cases.iter().enumerate() {
        if let Err(panic_msg) = type_check_no_panic(source) {
            failures.push((i, source.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (idx, source, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!("Source: {}", source);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} circular type tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_empty_trait_implementations() {
    let test_cases = vec![
        // Empty trait
        "T Empty {}",
        // Empty trait impl
        "T Show { F show(self)->str } I Show for i64 {}",
        // Trait with methods, empty impl
        "T Display { F fmt(self)->str F debug(self)->str } I Display for bool {}",
        // Multiple empty impls
        "T T1 {} T T2 {} I T1 for i64 {} I T2 for i64 {} I T1 for bool {} I T2 for bool {}",
        // Empty generic trait
        "T Convert<T> {} I Convert<i64> for bool {}",
        // Trait impl for non-existent type
        "T Show { F show(self)->str } I Show for NonExistent {}",
        // Duplicate trait impls
        "T Show { F show(self)->str } I Show for i64 {} I Show for i64 {}",
        // Impl for builtin type without trait definition
        "I SomeTrait for i64 {}",
    ];

    let mut failures = Vec::new();

    for (i, source) in test_cases.iter().enumerate() {
        if let Err(panic_msg) = type_check_no_panic(source) {
            failures.push((i, source.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (idx, source, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!("Source: {}", source);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} empty trait impl tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_functions_with_no_body() {
    let test_cases = vec![
        // These should parse but might have issues in type checking
        "F test()->i64",
        "F test(x:i64)->i64",
        "F test<T>(x:T)->T",
        // Function with just type signature
        "F add(x:i64, y:i64)->i64",
        // Generic function without body
        "F identity<T>(x:T)->T",
        // Multiple functions without bodies
        "F f1()->i64 F f2()->bool F f3()->str",
    ];

    let mut failures = Vec::new();

    for (i, source) in test_cases.iter().enumerate() {
        if let Err(panic_msg) = type_check_no_panic(source) {
            failures.push((i, source.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (idx, source, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!("Source: {}", source);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} no-body function tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_very_long_identifiers() {
    let mut failures = Vec::new();

    for len in [100, 500, 1000, 5000] {
        let long_id = "x".repeat(len);

        // Long variable name
        let source1 = format!("F test()->i64{{{}:=42;R {}}}", long_id, long_id);
        if let Err(panic_msg) = type_check_no_panic(&source1) {
            failures.push((format!("var name len {}", len), panic_msg));
        }

        // Long function name
        let source2 = format!("F {}()->i64=42", long_id);
        if let Err(panic_msg) = type_check_no_panic(&source2) {
            failures.push((format!("func name len {}", len), panic_msg));
        }

        // Long type name
        let source3 = format!("S {}{{{}: i64}}", long_id, long_id);
        if let Err(panic_msg) = type_check_no_panic(&source3) {
            failures.push((format!("type name len {}", len), panic_msg));
        }

        // Long field name
        let source4 = format!("S Point{{{}: i64, y: i64}}", long_id);
        if let Err(panic_msg) = type_check_no_panic(&source4) {
            failures.push((format!("field name len {}", len), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (desc, panic_msg) in &failures {
            eprintln!("\n{}: PANIC!", desc);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} long identifier tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_type_mismatches() {
    let test_cases = vec![
        // Wrong return type
        "F test()->i64=true",
        "F test()->bool=42",
        "F test()->str=123",
        // Wrong parameter type
        "F add(x:i64,y:i64)->i64=x+y F test()->i64=add(true, false)",
        // Type mismatch in assignment
        "F test()->i64{x:i64=true;R x}",
        // Multiple type errors
        "F test()->i64{x:bool=42;y:str=true;R false}",
        // Unresolved type variable
        "F test<T>(x:T)->i64=x",
        // Wrong number of type arguments
        "E Option<T>{Some(T),None} F test()->Option=None",
        // Conflicting type constraints
        "F test<T>(x:T, y:T)->T{R x} F main()->i64=test(42, true)",
    ];

    let mut failures = Vec::new();

    for (i, source) in test_cases.iter().enumerate() {
        if let Err(panic_msg) = type_check_no_panic(source) {
            failures.push((i, source.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (idx, source, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!("Source: {}", source);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} type mismatch tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_undefined_references() {
    let test_cases = vec![
        // Undefined variable
        "F test()->i64=undefined_var",
        // Undefined function
        "F test()->i64=undefined_func()",
        // Undefined type
        "F test()->UndefinedType=42",
        // Undefined struct field
        "S Point{x:i64} F test()->i64{p:=Point{x:1};R p.undefined_field}",
        // Undefined enum variant
        "E Option<T>{Some(T),None} F test()->Option<i64>=Option::Undefined",
        // Undefined trait
        "I UndefinedTrait for i64 {}",
        // Undefined generic parameter
        "F test()->T=42",
        // Referencing non-existent module
        "F test()->i64=some_module::some_func()",
    ];

    let mut failures = Vec::new();

    for (i, source) in test_cases.iter().enumerate() {
        if let Err(panic_msg) = type_check_no_panic(source) {
            failures.push((i, source.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (idx, source, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!("Source: {}", source);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} undefined reference tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_malformed_generics() {
    let test_cases = vec![
        // Too many type arguments
        "E Option<T>{Some(T),None} F test()->Option<i64,bool>=None",
        // Too few type arguments
        "S Pair<A,B>{first:A,second:B} F test()->Pair<i64>=Pair{first:1,second:2}",
        // Generic without definition
        "F test()->Vec<i64>=42",
        // Nested unresolved generics
        "F test<T>()->Option<Option<Option<T>>>=None",
        // Conflicting generic bounds
        "T Trait1 {} T Trait2 {} F test<T: Trait1 + Trait2>(x:T)->T=x F main()->i64=test(42)",
        // Generic type in wrong position
        "F test()->i64{T:=42;R T}",
        // Multiple definitions with same generic name
        "F test<T,T>(x:T,y:T)->T=x",
    ];

    let mut failures = Vec::new();

    for (i, source) in test_cases.iter().enumerate() {
        if let Err(panic_msg) = type_check_no_panic(source) {
            failures.push((i, source.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (idx, source, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!("Source: {}", source);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} malformed generic tests",
            failures.len()
        );
    }
}

#[test]
fn fuzz_generated_valid_programs() {
    let mut rng = SimpleRng::new(500);
    let mut failures = Vec::new();

    for i in 0..100 {
        let mut source = String::new();

        // Generate random number of definitions
        let def_count = rng.next_range(1, 20);

        for _ in 0..def_count {
            let def_type = rng.next_range(0, 4);

            match def_type {
                0 => {
                    // Generate function
                    let name = rng.gen_identifier();
                    let param_count = rng.next_range(0, 5);
                    source.push_str(&format!("F {}(", name));

                    for j in 0..param_count {
                        if j > 0 {
                            source.push_str(", ");
                        }
                        let param_name = rng.gen_identifier();
                        let param_type = rng.gen_type();
                        source.push_str(&format!("{}:{}", param_name, param_type));
                    }

                    let ret_type = rng.gen_type();
                    source.push_str(&format!(")->{}", ret_type));

                    // Simple body
                    if ret_type == "()" {
                        source.push_str("=()");
                    } else if ret_type == "bool" {
                        source.push_str("=true");
                    } else if ret_type == "str" {
                        source.push_str("=\"test\"");
                    } else {
                        source.push_str("=42");
                    }

                    source.push(' ');
                }
                1 => {
                    // Generate struct
                    let name = rng.gen_identifier();
                    let field_count = rng.next_range(0, 5);

                    source.push_str(&format!("S {}", name));
                    if field_count == 0 {
                        source.push_str(" ");
                    } else {
                        source.push('{');
                        for j in 0..field_count {
                            if j > 0 {
                                source.push_str(", ");
                            }
                            let field_name = rng.gen_identifier();
                            let field_type = rng.gen_type();
                            source.push_str(&format!("{}:{}", field_name, field_type));
                        }
                        source.push_str("} ");
                    }
                }
                2 => {
                    // Generate enum
                    let name = rng.gen_identifier();
                    let variant_count = rng.next_range(1, 4);

                    source.push_str(&format!("E {}", name));
                    source.push('{');
                    for j in 0..variant_count {
                        if j > 0 {
                            source.push(',');
                        }
                        let variant_name = rng.gen_identifier();
                        // Simple variant without fields
                        source.push_str(&variant_name);
                    }
                    source.push_str("} ");
                }
                _ => {
                    // Generate type alias
                    let name = rng.gen_identifier();
                    let target_type = rng.gen_type();
                    source.push_str(&format!("Y {}={} ", name, target_type));
                }
            }
        }

        if let Err(panic_msg) = type_check_no_panic(&source) {
            failures.push((i, source.clone(), panic_msg));
            if failures.len() >= 10 {
                break;
            }
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (idx, source, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!(
                "Source (first 200 chars): {}",
                source.chars().take(200).collect::<String>()
            );
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} generated programs",
            failures.len()
        );
    }
}

#[test]
fn fuzz_edge_case_expressions() {
    let test_cases = vec![
        // Division by zero
        "F test()->i64=42/0",
        // Overflow in literals
        "F test()->i64=99999999999999999999999999999",
        // Deep expression nesting
        "F test()->i64=((((((((((1+2)+3)+4)+5)+6)+7)+8)+9)+10)+11)",
        // Many chained operations
        "F test()->i64=1+2+3+4+5+6+7+8+9+10+11+12+13+14+15+16+17+18+19+20",
        // Complex boolean expression
        "F test()->bool=true&&false||true&&false||true&&false||true",
        // Deeply nested field access (if struct exists)
        "S A{b:B} S B{c:C} S C{d:D} S D{val:i64} F test(a:A)->i64=a.b.c.d.val",
        // Empty string operations
        "F test()->str=\"\"",
        // Negative numbers
        "F test()->i64=-42",
        "F test()->i64=-(-(-42))",
        // Mixed operations
        "F test()->i64=(1+2)*3-4/2",
    ];

    let mut failures = Vec::new();

    for (i, source) in test_cases.iter().enumerate() {
        if let Err(panic_msg) = type_check_no_panic(source) {
            failures.push((i, source.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== TYPE CHECKER PANICS FOUND ===");
        for (idx, source, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!("Source: {}", source);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Type checker panicked on {} edge case expression tests",
            failures.len()
        );
    }
}
