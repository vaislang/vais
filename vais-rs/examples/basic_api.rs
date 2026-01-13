//! Basic Vais API Usage Example
//! Run: cargo run --example basic_api

use vais_ir::Value;
use vais_lowering::Lowerer;
use vais_parser::parse;
use vais_vm::execute_function;

fn main() {
    println!("=== Vais Basic API Examples ===\n");

    // Example 1: Simple arithmetic
    example_simple_arithmetic();

    // Example 2: Recursive function (factorial)
    example_factorial();

    // Example 3: Collection operations
    example_collections();

    // Example 4: Pattern matching
    example_pattern_matching();

    // Example 5: Struct operations
    example_structs();
}

fn example_simple_arithmetic() {
    println!("1. Simple Arithmetic");
    println!("{}", "-".repeat(40));

    let source = r#"
        add(a, b) = a + b
        mul(a, b) = a * b
        calc(x, y, z) = add(x, y) * z
    "#;

    // Parse source code
    let program = parse(source).expect("Failed to parse");

    // Lower to bytecode
    let functions = Lowerer::new().lower_program(&program).expect("Failed to lower");

    // Execute function
    let result = execute_function(
        functions.clone(),
        "calc",
        vec![Value::Int(2), Value::Int(3), Value::Int(4)],
    );

    println!("calc(2, 3, 4) = {:?}", result);
    println!();
}

fn example_factorial() {
    println!("2. Recursive Factorial");
    println!("{}", "-".repeat(40));

    let source = "factorial(n) = n < 2 ? 1 : n * $(n - 1)";

    let program = parse(source).expect("Failed to parse");
    let functions = Lowerer::new().lower_program(&program).expect("Failed to lower");

    for n in [5, 10, 15] {
        let result = execute_function(functions.clone(), "factorial", vec![Value::Int(n)]);
        println!("factorial({}) = {:?}", n, result);
    }
    println!();
}

fn example_collections() {
    println!("3. Collection Operations");
    println!("{}", "-".repeat(40));

    let source = r#"
        double(arr) = arr.@(_ * 2)
        evens(arr) = arr.?(_ % 2 == 0)
        sum(arr) = arr./+
        process(arr) = arr.?(_ > 0).@(_ * 2)./+
    "#;

    let program = parse(source).expect("Failed to parse");
    let functions = Lowerer::new().lower_program(&program).expect("Failed to lower");

    let input = Value::Array(std::rc::Rc::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
        Value::Int(5),
    ]));

    let doubled = execute_function(functions.clone(), "double", vec![input.clone()]);
    println!("double([1,2,3,4,5]) = {:?}", doubled);

    let evens = execute_function(functions.clone(), "evens", vec![input.clone()]);
    println!("evens([1,2,3,4,5]) = {:?}", evens);

    let sum = execute_function(functions.clone(), "sum", vec![input.clone()]);
    println!("sum([1,2,3,4,5]) = {:?}", sum);

    let mixed = Value::Array(std::rc::Rc::new(vec![
        Value::Int(-1),
        Value::Int(2),
        Value::Int(-3),
        Value::Int(4),
    ]));
    let processed = execute_function(functions.clone(), "process", vec![mixed]);
    println!("process([-1,2,-3,4]) = {:?}", processed);
    println!();
}

fn example_pattern_matching() {
    println!("4. Pattern Matching");
    println!("{}", "-".repeat(40));

    let source = r#"
        describe(n) = match n {
            0 => "zero",
            1 => "one",
            _ => "many"
        }
    "#;

    let program = parse(source).expect("Failed to parse");
    let functions = Lowerer::new().lower_program(&program).expect("Failed to lower");

    for n in [0, 1, 5] {
        let result = execute_function(functions.clone(), "describe", vec![Value::Int(n)]);
        println!("describe({}) = {:?}", n, result);
    }
    println!();
}

fn example_structs() {
    println!("5. Struct Operations");
    println!("{}", "-".repeat(40));

    let source = r#"
        make_point(x, y) = { x: x, y: y }
        get_x(p) = p.x
        get_y(p) = p.y
    "#;

    let program = parse(source).expect("Failed to parse");
    let functions = Lowerer::new().lower_program(&program).expect("Failed to lower");

    let point = execute_function(
        functions.clone(),
        "make_point",
        vec![Value::Int(10), Value::Int(20)],
    );
    println!("make_point(10, 20) = {:?}", point);

    if let Ok(ref p) = point {
        let x = execute_function(functions.clone(), "get_x", vec![p.clone()]);
        let y = execute_function(functions.clone(), "get_y", vec![p.clone()]);
        println!("get_x(point) = {:?}", x);
        println!("get_y(point) = {:?}", y);
    }
    println!();
}
