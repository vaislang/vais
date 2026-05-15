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
        let mut source = String::from("fn test(");
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
        let mut call_source = String::from("fn f(");
        for i in 0..param_count {
            if i > 0 {
                call_source.push_str(", ");
            }
            call_source.push_str(&format!("p{}:i64", i));
        }
        call_source.push_str(")->i64=0 fn test()->i64=f(");
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
    // Run in a thread with larger stack size to accommodate ASan's overhead
    let handle = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024) // 16MB stack
        .spawn(|| {
            let mut failures = Vec::new();

            // Reduced depth limits for ASan compatibility (ASan uses more stack)
            // Check for ASan via RUSTFLAGS environment variable (set during build)
            let is_asan = option_env!("RUSTFLAGS")
                .map(|flags| flags.contains("sanitizer=address"))
                .unwrap_or(false);
            let depths = if is_asan {
                vec![5, 10, 15]
            } else {
                vec![5, 10, 20, 50]
            };

            for depth in depths {
                // Nested Option types: Option<Option<Option<...>>>
                let mut type_str = String::from("i64");
                for _ in 0..depth {
                    type_str = format!("Option<{}>", type_str);
                }
                let source = format!(
                    "enum Option<T>{{Some(T),None}} fn test()->{}=None",
                    type_str
                );

                if let Err(panic_msg) = type_check_no_panic(&source) {
                    failures.push((format!("Option depth {}", depth), panic_msg));
                }

                // Nested tuple types: ((((i64, i64), i64), i64), i64)
                let mut tuple_type = String::from("i64");
                for _ in 0..depth {
                    tuple_type = format!("({}, i64)", tuple_type);
                }
                let source2 = format!("fn test()->{}=(42, 42)", tuple_type);

                if let Err(panic_msg) = type_check_no_panic(&source2) {
                    failures.push((format!("Tuple depth {}", depth), panic_msg));
                }

                // Nested function types: fn(fn(fn(i64)->i64)->i64)->i64
                let mut fn_type = String::from("i64");
                for _ in 0..depth {
                    fn_type = format!("fn({})->i64", fn_type);
                }
                let source3 = format!("fn test(f:{})->i64=42", fn_type);

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
        })
        .expect("Failed to spawn thread");

    handle.join().expect("Thread panicked");
}

#[test]
fn fuzz_circular_type_references() {
    let test_cases = [
        // Type alias referring to itself
        "Y A = A",
        // Mutual type alias references
        "Y A = B Y B = A",
        // Struct with field of its own type
        "struct Node { value: i64, next: Node }",
        // Mutual struct references
        "struct A { b: B } struct B { a: A }",
        // Enum with recursive variant
        "enum List { Cons(i64, List), Nil }",
        // Type alias chain
        "Y A = B Y B = C Y C = A",
        // Complex circular dependency
        "struct Foo { bar: Bar } struct Bar { baz: Baz } struct Baz { foo: Foo }",
        // Self-referential generic
        "enum Tree<T> { Node(T, Tree<Tree<T>>), Leaf }",
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
    let test_cases = [
        // Empty trait
        "trait Empty {}",
        // Empty trait impl
        "trait Show { fn show(self)->str } impl i64: Show {}",
        // Trait with methods, empty impl
        "trait Display { fn fmt(self)->str fn debug(self)->str } impl bool: Display {}",
        // Multiple empty impls
        "trait T1 {} trait T2 {} impl i64: T1 {} impl i64: T2 {} impl bool: T1 {} impl bool: T2 {}",
        // Empty generic trait
        "trait Convert<T> {} impl bool: Convert<i64> {}",
        // Trait impl for non-existent type
        "trait Show { fn show(self)->str } impl NonExistent: Show {}",
        // Duplicate trait impls
        "trait Show { fn show(self)->str } impl i64: Show {} impl i64: Show {}",
        // Impl for builtin type without trait definition
        "impl i64: SomeTrait {}",
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
    let test_cases = [
        // These should parse but might have issues in type checking
        "fn test()->i64",
        "fn test(x:i64)->i64",
        "fn test<T>(x:T)->T",
        // Function with just type signature
        "fn add(x:i64, y:i64)->i64",
        // Generic function without body
        "fn identity<T>(x:T)->T",
        // Multiple functions without bodies
        "fn f1()->i64 fn f2()->bool fn f3()->str",
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
        let source1 = format!("fn test()->i64{{{}:=42;return {}}}", long_id, long_id);
        if let Err(panic_msg) = type_check_no_panic(&source1) {
            failures.push((format!("var name len {}", len), panic_msg));
        }

        // Long function name
        let source2 = format!("fn {}()->i64=42", long_id);
        if let Err(panic_msg) = type_check_no_panic(&source2) {
            failures.push((format!("func name len {}", len), panic_msg));
        }

        // Long type name
        let source3 = format!("struct {}{{{}: i64}}", long_id, long_id);
        if let Err(panic_msg) = type_check_no_panic(&source3) {
            failures.push((format!("type name len {}", len), panic_msg));
        }

        // Long field name
        let source4 = format!("struct Point{{{}: i64, y: i64}}", long_id);
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
        "fn test()->i64=true",
        "fn test()->bool=42",
        "fn test()->str=123",
        // Wrong parameter type
        "fn add(x:i64,y:i64)->i64=x+y fn test()->i64=add(true, false)",
        // Type mismatch in assignment
        "fn test()->i64{x:i64=true;return x}",
        // Multiple type errors
        "fn test()->i64{x:bool=42;y:str=true;return false}",
        // Unresolved type variable
        "fn test<T>(x:T)->i64=x",
        // Wrong number of type arguments
        "enum Option<T>{Some(T),None} fn test()->Option=None",
        // Conflicting type constraints
        "fn test<T>(x:T, y:T)->T{return x} fn main()->i64=test(42, true)",
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
    let test_cases = [
        // Undefined variable
        "fn test()->i64=undefined_var",
        // Undefined function
        "fn test()->i64=undefined_func()",
        // Undefined type
        "fn test()->UndefinedType=42",
        // Undefined struct field
        "struct Point{x:i64} fn test()->i64{p:=Point{x:1};return p.undefined_field}",
        // Undefined enum variant
        "enum Option<T>{Some(T),None} fn test()->Option<i64>=Option::Undefined",
        // Undefined trait
        "impl i64: UndefinedTrait {}",
        // Undefined generic parameter
        "fn test()->T=42",
        // Referencing non-existent module
        "fn test()->i64=some_module::some_func()",
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
    let test_cases = [
        // Too many type arguments
        "enum Option<T>{Some(T),None} fn test()->Option<i64,bool>=None",
        // Too few type arguments
        "struct Pair<A,B>{first:A,second:B} fn test()->Pair<i64>=Pair{first:1,second:2}",
        // Generic without definition
        "fn test()->Vec<i64>=42",
        // Nested unresolved generics
        "fn test<T>()->Option<Option<Option<T>>>=None",
        // Conflicting generic bounds
        "trait Trait1 {} trait Trait2 {} fn test<T: Trait1 + Trait2>(x:T)->T=x fn main()->i64=test(42)",
        // Generic type in wrong position
        "fn test()->i64{T:=42;return T}",
        // Multiple definitions with same generic name
        "fn test<T,T>(x:T,y:T)->T=x",
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
    // Run in a thread with larger stack size to accommodate ASan's overhead
    let handle = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024) // 16MB stack
        .spawn(|| {
            let mut rng = SimpleRng::new(500);
            let mut failures = Vec::new();

            // Reduced iteration count for ASan (ASan is slower and uses more memory)
            let is_asan = option_env!("RUSTFLAGS")
                .map(|flags| flags.contains("sanitizer=address"))
                .unwrap_or(false);
            let iterations = if is_asan { 30 } else { 100 };

            for i in 0..iterations {
                let mut source = String::new();

                // Generate random number of definitions (fewer under ASan)
                let def_count = if is_asan {
                    rng.next_range(1, 10)
                } else {
                    rng.next_range(1, 20)
                };

                for _ in 0..def_count {
                    let def_type = rng.next_range(0, 4);

                    match def_type {
                        0 => {
                            // Generate function
                            let name = rng.gen_identifier();
                            let param_count = rng.next_range(0, 5);
                            source.push_str(&format!("fn {}(", name));

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

                            source.push_str(&format!("struct {}", name));
                            if field_count == 0 {
                                source.push(' ');
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

                            source.push_str(&format!("enum {}", name));
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
        })
        .expect("Failed to spawn thread");

    handle.join().expect("Thread panicked");
}

#[test]
fn fuzz_edge_case_expressions() {
    // Run in a thread with larger stack size to accommodate ASan's overhead
    let handle = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024) // 16MB stack
        .spawn(|| {
            let test_cases = vec![
                // Division by zero
                "fn test()->i64=42/0",
                // Overflow in literals
                "fn test()->i64=99999999999999999999999999999",
                // Deep expression nesting
                "fn test()->i64=((((((((((1+2)+3)+4)+5)+6)+7)+8)+9)+10)+11)",
                // Many chained operations
                "fn test()->i64=1+2+3+4+5+6+7+8+9+10+11+12+13+14+15+16+17+18+19+20",
                // Complex boolean expression
                "fn test()->bool=true&&false||true&&false||true&&false||true",
                // Deeply nested field access (if struct exists)
                "struct A{b:B} struct B{c:C} struct C{d:D} struct D{val:i64} fn test(a:A)->i64=a.b.c.d.val",
                // Empty string operations
                "fn test()->str=\"\"",
                // Negative numbers
                "fn test()->i64=-42",
                "fn test()->i64=-(-(-42))",
                // Mixed operations
                "fn test()->i64=(1+2)*3-4/2",
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
        })
        .expect("Failed to spawn thread");

    handle.join().expect("Thread panicked");
}
