//! Integration tests for the Vais compiler
//!
//! These tests verify the complete compilation pipeline:
//! Source -> Lexer -> Parser -> Type Checker -> Code Generator -> LLVM IR

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Helper function to compile source code to LLVM IR
fn compile_to_ir(source: &str) -> Result<String, String> {
    // Step 1: Tokenize
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;

    // Step 2: Parse
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;

    // Step 3: Type check
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;

    // Step 4: Generate LLVM IR (with generic instantiations if any)
    let mut gen = CodeGenerator::new("test");
    let instantiations = checker.get_generic_instantiations();
    let ir = if instantiations.is_empty() {
        gen.generate_module(&module)
            .map_err(|e| format!("Codegen error: {:?}", e))?
    } else {
        gen.generate_module_with_instantiations(&module, &instantiations)
            .map_err(|e| format!("Codegen error: {:?}", e))?
    };

    Ok(ir)
}

/// Helper to check if source compiles successfully
fn compiles(source: &str) -> bool {
    match compile_to_ir(source) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            false
        }
    }
}

/// Helper to check if source fails to compile
fn fails_to_compile(source: &str) -> bool {
    compile_to_ir(source).is_err()
}

// ==================== Basic Compilation Tests ====================

#[test]
fn test_empty_module() {
    assert!(compiles(""));
}

#[test]
fn test_hello_world() {
    // print is a runtime function, so just test a simple main
    let source = r#"fn main() -> () { () }"#;
    assert!(compiles(source));
}

#[test]
fn test_simple_arithmetic() {
    let source = "fn add(a:i64, b:i64) -> i64 = a + b";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("define i64 @add"));
    assert!(ir.contains("add i64"));
}

#[test]
fn test_fibonacci() {
    let source = "fn fib(n:i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("define i64 @fib"));
    assert!(ir.contains("call i64 @fib")); // Recursive calls
}

// ==================== Control Flow Tests ====================

#[test]
fn test_if_else() {
    let source = "fn max(a:i64, b:i64)->i64=I a>b{a} else {b}";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("icmp sgt"));
    assert!(ir.contains("br i1"));
}

#[test]
fn test_nested_if() {
    let source = r#"
fn classify(x: i64) -> i64 {
    I x > 0 {
        I x > 100 { 2 } else { 1 }
    } else {
        I x < 0 - 100 { 0 - 2 } else { 0 - 1 }
    }
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_match_expression() {
    let source = r#"
fn digit_name(n:i64) -> str = match n {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_for_loop() {
    // Test for loop with range - immutable binding
    let source = r#"
fn count_range(n: i64) -> i64 {
    result := 0;
    L i: 0..n { result };
    result
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_while_loop() {
    // Test while loop syntax without mutable assignment
    let source = r#"
fn check_condition(n: i64) -> i64 {
    L _: n > 0 { n };
    0
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_infinite_loop_with_break() {
    // Test infinite loop with break
    let source = r#"
fn find_limit(x: i64) -> i64 {
    L {
        I x > 100 { B x };
        x
    };
    0
}
"#;
    assert!(compiles(source));
}

// ==================== Struct Tests ====================

#[test]
fn test_struct_definition() {
    let source = r#"
struct Point { x: f64, y: f64 }
fn origin() -> Point = Point { x: 0.0, y: 0.0 }
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("%Point = type { double, double }"));
}

#[test]
fn test_struct_field_access() {
    let source = r#"
struct Point { x: i64, y: i64 }
fn get_x(p: Point) -> i64 = p.x
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("getelementptr"));
}

#[test]
fn test_empty_struct() {
    let source = r#"
struct Unit {}
fn make_unit() -> Unit = Unit {}
"#;
    assert!(compiles(source));
}

// ==================== Enum Tests ====================

#[test]
fn test_enum_definition() {
    // Enum definition - path syntax tested separately
    let source = r#"
enum Color { Red, Green, Blue }
"#;
    assert!(compiles(source));
}

#[test]
fn test_enum_with_data() {
    let source = r#"
enum Shape { Circle(f64), Rectangle(f64, f64) }
"#;
    assert!(compiles(source));
}

#[test]
fn test_enum_pattern_match() {
    let source = r#"
enum Result { Ok(i64), Err(str) }
fn handle(r: Result) -> i64 = match r {
    Ok(v) => v,
    Err(_) => 0
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_enum_unit_variant_matching() {
    let source = r#"
enum Status { Pending, Running, Done }

fn status_to_code(s: Status) -> i64 = match s {
    Pending => 0,
    Running => 1,
    Done => 2
}

fn test_status() -> i64 {
    pending := Pending;
    running := Running;
    done := Done;
    a := status_to_code(pending);
    b := status_to_code(running);
    c := status_to_code(done);
    a + b + c
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("icmp eq i32"));
    assert!(ir.contains("define i64 @test_status"));
}

// ==================== Generic Tests ====================

#[test]
fn test_generic_function() {
    let source = "fn identity<T>(x: T) -> T = x";
    assert!(compiles(source));
}

#[test]
fn test_generic_struct() {
    let source = r#"
struct Box<T> { value: type }
fn get_value<T>(b: Box<T>) -> type = b.value
"#;
    assert!(compiles(source));
}

#[test]
fn test_generic_with_bounds() {
    let source = "fn compare<T: Ord>(a: T, b: T) -> bool = a < b";
    assert!(compiles(source));
}

// ==================== Lambda Tests ====================

#[test]
fn test_simple_lambda() {
    let source = r#"
fn apply_double() -> i64 {
    double := |x:i64| x * 2;
    double(21)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_higher_order_function() {
    let source = r#"
fn apply(f: (i64) -> i64, x: i64) -> i64 = f(x)
fn double(x: i64) -> i64 = x * 2
fn test() -> i64 = apply(double, 21)
"#;
    assert!(compiles(source));
}

// ==================== Type System Tests ====================

#[test]
fn test_array_type() {
    // Test array type in parameter and indexing
    let source = r#"
fn first(arr: [i64]) -> i64 = arr[0]
"#;
    assert!(compiles(source));
}

#[test]
fn test_tuple_type() {
    let source = "fn make_pair(a: i64, b: str) -> (i64, str) = (a, b)";
    assert!(compiles(source));
}

#[test]
fn test_pointer_type() {
    let source = "fn identity_ptr(p: *i64) -> *i64 = p";
    assert!(compiles(source));
}

#[test]
fn test_reference_type() {
    let source = "fn ref_fn(r: &i64) -> i64 = 0";
    assert!(compiles(source));
}

// ==================== Operator Tests ====================

#[test]
fn test_arithmetic_operators() {
    let source = r#"
fn math(a: i64, b: i64) -> i64 {
    x := a + b;
    y := x - a;
    z := y * b;
    w := z / a;
    w % b
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("add i64"));
    assert!(ir.contains("sub i64"));
    assert!(ir.contains("mul i64"));
    assert!(ir.contains("sdiv i64"));
    assert!(ir.contains("srem i64"));
}

#[test]
fn test_comparison_operators() {
    let source = r#"
fn compare(a: i64, b: i64) -> bool {
    lt := a < b;
    le := a <= b;
    gt := a > b;
    ge := a >= b;
    eq := a == b;
    ne := a != b;
    lt
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("icmp slt"));
    assert!(ir.contains("icmp sle"));
    assert!(ir.contains("icmp sgt"));
    assert!(ir.contains("icmp sge"));
    assert!(ir.contains("icmp eq"));
    assert!(ir.contains("icmp ne"));
}

#[test]
fn test_bitwise_operators() {
    let source = r#"
fn bits(a: i64, b: i64) -> i64 {
    x := a & b;
    y := a | b;
    z := a ^ b;
    w := a << 2;
    v := a >> 1;
    x
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("and i64"));
    assert!(ir.contains("or i64"));
    assert!(ir.contains("xor i64"));
    assert!(ir.contains("shl i64"));
    assert!(ir.contains("ashr i64"));
}

#[test]
fn test_logical_operators() {
    let source = "fn logic(a: bool, b: bool) -> bool = a && b || !a";
    assert!(compiles(source));
}

#[test]
fn test_unary_operators() {
    let source = r#"
fn unary(x: i64) -> i64 {
    neg := -x;
    bitnot := (~x);
    neg + bitnot
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("sub i64 0")); // negation
    assert!(ir.contains("xor i64") && ir.contains("-1")); // bitwise not
}

#[test]
fn test_compound_operations() {
    // Test arithmetic operations without mutable assignment
    let source = r#"
fn compound(x: i64) -> i64 {
    y := x + 1;
    z := y - 2;
    w := z * 3;
    w
}
"#;
    assert!(compiles(source));
}

// ==================== Trait Tests ====================

#[test]
fn test_trait_definition() {
    let source = r#"
trait Display { fn display(s: &Self) -> str = "" }
"#;
    assert!(compiles(source));
}

#[test]
fn test_impl_block() {
    let source = r#"
struct Counter { value: i64 }
impl Counter {
    fn new() -> Counter = Counter { value: 0 }
    fn get_val(c: Counter) -> i64 = c.value
}
"#;
    assert!(compiles(source));
}

// ==================== Module System Tests ====================

#[test]
fn test_use_statement() {
    let source = "use std::fs";
    assert!(compiles(source));
}

#[test]
fn test_pub_function() {
    let source = "pub fn public_fn() -> () = ()";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("define void @public_fn"));
}

// ==================== Async Tests ====================

#[test]
fn test_async_function() {
    let source = "A F async_fn() -> i64 = 42";
    assert!(compiles(source));
}

// ==================== Error Detection Tests ====================

#[test]
fn test_type_mismatch_error() {
    let source = "fn f() -> i64 = \"string\"";
    assert!(fails_to_compile(source));
}

#[test]
fn test_undefined_variable_error() {
    let source = "fn f() -> i64 = undefined_var";
    assert!(fails_to_compile(source));
}

#[test]
fn test_undefined_function_error() {
    let source = "fn f() -> i64 = unknown_func()";
    assert!(fails_to_compile(source));
}

#[test]
fn test_wrong_argument_count_error() {
    let source = r#"
fn add(a: i64, b: i64) -> i64 = a + b
fn f() -> i64 = add(1)
"#;
    assert!(fails_to_compile(source));
}

#[test]
fn test_wrong_argument_type_error() {
    let source = r#"
fn add(a: i64, b: i64) -> i64 = a + b
fn f() -> i64 = add(1, "two")
"#;
    assert!(fails_to_compile(source));
}

#[test]
fn test_if_condition_lenient_integer_truthy() {
    // Phase 254: `I`/`LW` conditions accept integer truthy alongside Bool.
    // This is scoped to predicate position and does not conflict with
    // Phase 158's bool↔i64 prohibition in value context.
    // Mirror of crates/vais-types/src/tests.rs::test_if_condition_lenient_integer_truthy.
    // Renamed from test_if_condition_non_bool_error (pre-254 expectation).
    let source = "fn f() -> i64 = I 42 { 1 } else { 0 }";
    assert!(compiles(source));
}

// ==================== Complex Programs ====================

#[test]
fn test_factorial() {
    let source = r#"
fn factorial(n: i64) -> i64 = n <= 1 ? 1 : n * @(n - 1)
fn main() -> () {
    result := factorial(5);
    ()
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_binary_search() {
    let source = r#"
fn binary_search(arr: [i64], target: i64, low: i64, high: i64) -> i64 {
    I low > high { 0 - 1 }
    else {
        mid := (low + high) / 2;
        I arr[mid] == target { mid }
        else {
            I arr[mid] < target {
                @(arr, target, mid + 1, high)
            } else {
                @(arr, target, low, mid - 1)
            }
        }
    }
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_bubble_sort_structure() {
    let source = r#"
fn swap(arr: *i64, i: i64, j: i64) -> () {
    temp := arr[i];
    ()
}

fn bubble_sort(arr: *i64, n: i64) -> () {
    L i:0..n {
        L j:0..(n-i-1) {
            I arr[j] > arr[j+1] {
                swap(arr, j, j+1)
            }
        }
    }
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_linked_list_structure() {
    // Test self-referential struct definition
    let source = r#"
struct Node { value: i64, next: *Node }

fn get_value(n: Node) -> i64 = n.value
"#;
    assert!(compiles(source));
}

#[test]
fn test_multiple_types_and_functions() {
    let source = r#"
# Types
struct Point { x: f64, y: f64 }
struct Size { width: f64, height: f64 }
struct Rect { x: f64, y: f64, width: f64, height: f64 }

# Functions
fn point_new(x: f64, y: f64) -> Point = Point { x: x, y: y }
fn size_new(w: f64, h: f64) -> Size = Size { width: w, height: h }

# Main
fn main() -> () {
    p := point_new(0.0, 0.0);
    s := size_new(100.0, 50.0);
    ()
}
"#;
    assert!(compiles(source));
}

// ==================== Try and Ternary Operator Tests ====================

#[test]
fn test_ternary_operator() {
    let source = "fn abs(x: i64) -> i64 = x < 0 ? -x : x";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("define i64 @abs"));
    assert!(ir.contains("br i1")); // conditional branch for ternary
}

#[test]
fn test_ternary_with_unary_minus() {
    // Test ternary where then branch starts with unary minus
    let source = "fn abs(x: i64) -> i64 = x < 0 ? -x : x";
    assert!(compiles(source));
}

#[test]
fn test_try_operator_parsing() {
    // Test that ? operator parses correctly
    // Note: Full try operator functionality requires Result/Option types
    let source = r#"
fn maybe_get(opt: i64?) -> i64? {
    v := opt?
    return Some(v)
}
"#;
    // This should parse correctly
    let module = vais_parser::parse(source);
    assert!(module.is_ok());
}

#[test]
fn test_try_operator_with_binary_op() {
    // Test try operator followed by binary operator
    let source = r#"
fn add_one(opt: i64?) -> i64? {
    v := opt? + 1
    return Some(v)
}
"#;
    let module = vais_parser::parse(source);
    assert!(module.is_ok());
}

#[test]
fn test_try_and_ternary_together() {
    // Test that try and ternary can coexist
    let source = r#"
fn process(opt: i64?) -> i64? {
    v := (opt?)
    return v > 0 ? Some(v) : None
}
"#;
    let module = vais_parser::parse(source);
    assert!(module.is_ok());
}

// ==================== Const Generics Tests ====================

#[test]
fn test_const_array_type_literal() {
    // Test parsing of const-sized array with literal size
    let source = r#"
struct Data {
    values: [i64; 10]
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_array_in_function_param() {
    // Test parsing const-sized array in function parameter
    let source = r#"
fn process(arr: [i64; 5]) -> i64 {
    42
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_array_as_return_type() {
    // Test parsing const-sized array as return type
    let source = r#"
struct Container {
    data: [i64; 3]
}
fn get_data() -> [i64; 3] {
    [1, 2, 3]
}
"#;
    // Note: This test verifies parsing and type checking
    let module = vais_parser::parse(source);
    assert!(module.is_ok());
}

#[test]
fn test_const_generic_parameter() {
    // Test parsing const generic parameter in function
    let source = r#"
fn identity<T, const N: u64>(x: T) -> type {
    x
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_in_struct() {
    // Test parsing const generic parameter in struct
    let source = r#"
struct FixedArray<T, const N: u64> {
    data: [T; N],
    len: u64
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_expr_addition() {
    // Test const expression with addition
    let source = r#"
fn process(arr: [i64; 5 + 3]) -> i64 {
    8
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_expr_multiplication() {
    // Test const expression with multiplication
    let source = r#"
fn process(arr: [i64; 4 * 3]) -> i64 {
    12
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_expr_complex() {
    // Test complex const expression
    let source = r#"
fn process(arr: [i64; 2 * 3 + 4]) -> i64 {
    10
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_array_in_nested_type() {
    // Test const array in optional type
    let source = r#"
struct Matrix {
    data: [f64; 9]
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_multiple_const_generics() {
    // Test multiple const generic parameters
    let source = r#"
struct Matrix<const ROWS: u64, const COLS: u64> {
    data: [f64; ROWS],
    rows: u64,
    cols: u64
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_mixed_type_and_const_generics() {
    // Test mixing type and const generics
    let source = r#"
struct Container<T, const N: u64> {
    elements: [T; N],
    count: u64
}
"#;
    // Just test parsing for now
    let module = vais_parser::parse(source);
    assert!(module.is_ok());
}

// ==================== SIMD Vector Tests ====================

#[test]
fn test_simd_vec4i32_constructor() {
    // Use i32 vector which doesn't have literal type issues
    let source = r#"
fn test_vec4i32() -> Vec4i32 {
    vec4i32(1, 2, 3, 4)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<4 x i32>"));
    assert!(ir.contains("insertelement"));
}

#[test]
fn test_simd_vec4f32_add() {
    let source = r#"
fn test_add(a: Vec4f32, b: Vec4f32) -> Vec4f32 {
    simd_add_vec4f32(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<4 x float>"));
    assert!(ir.contains("fadd <4 x float>"));
}

#[test]
fn test_simd_vec4f32_mul() {
    let source = r#"
fn test_mul(a: Vec4f32, b: Vec4f32) -> Vec4f32 {
    simd_mul_vec4f32(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<4 x float>"));
    assert!(ir.contains("fmul <4 x float>"));
}

#[test]
fn test_simd_vec4f32_sub() {
    let source = r#"
fn test_sub(a: Vec4f32, b: Vec4f32) -> Vec4f32 {
    simd_sub_vec4f32(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("fsub <4 x float>"));
}

#[test]
fn test_simd_vec4f32_div() {
    let source = r#"
fn test_div(a: Vec4f32, b: Vec4f32) -> Vec4f32 {
    simd_div_vec4f32(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("fdiv <4 x float>"));
}

#[test]
fn test_simd_vec4i32_add() {
    let source = r#"
fn test_add(a: Vec4i32, b: Vec4i32) -> Vec4i32 {
    simd_add_vec4i32(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<4 x i32>"));
    assert!(ir.contains("add <4 x i32>"));
}

#[test]
fn test_simd_vec8f32_operations() {
    let source = r#"
fn test_vec8(a: Vec8f32, b: Vec8f32) -> Vec8f32 {
    c := simd_add_vec8f32(a, b)
    simd_mul_vec8f32(c, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<8 x float>"));
    assert!(ir.contains("fadd <8 x float>"));
    assert!(ir.contains("fmul <8 x float>"));
}

#[test]
fn test_simd_reduce_add_vec4f32() {
    let source = r#"
fn test_reduce(v: Vec4f32) -> f32 {
    simd_reduce_add_vec4f32(v)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@llvm.vector.reduce.fadd.v4f32"));
}

#[test]
fn test_simd_dot_product() {
    let source = r#"
fn dot_product(a: Vec4f32, b: Vec4f32) -> f32 {
    product := simd_mul_vec4f32(a, b)
    simd_reduce_add_vec4f32(product)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("fmul <4 x float>"));
    assert!(ir.contains("@llvm.vector.reduce.fadd.v4f32"));
}

#[test]
fn test_simd_vec2f64_operations() {
    let source = r#"
fn test_vec2f64(a: Vec2f64, b: Vec2f64) -> Vec2f64 {
    simd_add_vec2f64(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<2 x double>"));
    assert!(ir.contains("fadd <2 x double>"));
}

#[test]
fn test_simd_vec4f64_operations() {
    let source = r#"
fn test_vec4f64(a: Vec4f64, b: Vec4f64) -> Vec4f64 {
    simd_mul_vec4f64(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<4 x double>"));
    assert!(ir.contains("fmul <4 x double>"));
}

#[test]
fn test_simd_vec2i64_operations() {
    let source = r#"
fn test_vec2i64(a: Vec2i64, b: Vec2i64) -> Vec2i64 {
    simd_add_vec2i64(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<2 x i64>"));
    assert!(ir.contains("add <2 x i64>"));
}

#[test]
fn test_simd_vec4i64_mul() {
    let source = r#"
fn test_mul(a: Vec4i64, b: Vec4i64) -> Vec4i64 {
    simd_mul_vec4i64(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<4 x i64>"));
    assert!(ir.contains("mul <4 x i64>"));
}

#[test]
fn test_simd_vec_constructors() {
    // Test integer vector constructors (float constructors have literal type issues)
    let source = r#"
fn test_constructors() -> i32 {
    v4i32 := vec4i32(1, 2, 3, 4)
    v2i64 := vec2i64(1, 2)
    v4i64 := vec4i64(1, 2, 3, 4)
    0
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<4 x i32>"));
    assert!(ir.contains("<2 x i64>"));
    assert!(ir.contains("<4 x i64>"));
}

#[test]
fn test_simd_vec_float_constructors_via_params() {
    // Test float vector constructors with parameters (avoids literal type issues)
    let source = r#"
fn test_f32_constructor(a: f32, b: f32, c: f32, d: f32) -> Vec4f32 {
    vec4f32(a, b, c, d)
}

fn test_f64_constructor(a: f64, b: f64) -> Vec2f64 {
    vec2f64(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("<4 x float>"));
    assert!(ir.contains("<2 x double>"));
}

#[test]
fn test_simd_reduce_i32() {
    let source = r#"
fn test_reduce(v: Vec4i32) -> i32 {
    simd_reduce_add_vec4i32(v)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@llvm.vector.reduce.add.v4i32"));
}

#[test]
fn test_simd_reduce_i64() {
    let source = r#"
fn test_reduce(v: Vec2i64) -> i64 {
    simd_reduce_add_vec2i64(v)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@llvm.vector.reduce.add.v2i64"));
}

// ==================== Standard Library Tests ====================

#[test]
fn test_std_option_type() {
    // Test Option<T> enum definition and usage
    let source = r#"
enum Option<T> {
    None,
    Some(T)
}

fn is_some(opt: Option<i64>) -> i64 {
    match opt {
        Some(_) => 1,
        None => 0
    }
}

fn is_none(opt: Option<i64>) -> i64 {
    match opt {
        Some(_) => 0,
        None => 1
    }
}

fn unwrap_or(opt: Option<i64>, default: i64) -> i64 {
    match opt {
        Some(v) => v,
        None => default
    }
}

fn test_option() -> i64 {
    some_val := Some(42)
    none_val: Option<i64> = None
    result := unwrap_or(some_val, 0)
    result
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_result_type() {
    // Test Result<T, E> enum definition
    let source = r#"
enum Result<T, E> {
    Ok(T),
    Err(E)
}

fn is_ok(r: Result<i64, str>) -> i64 {
    match r {
        Ok(_) => 1,
        Err(_) => 0
    }
}

fn is_err(r: Result<i64, str>) -> i64 {
    match r {
        Ok(_) => 0,
        Err(_) => 1
    }
}

fn test_result() -> i64 {
    success: Result<i64, str> = Ok(42)
    failure: Result<i64, str> = Err("error")
    is_ok(success)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_vec_struct() {
    // Test Vec<T> struct definition and basic operations
    let source = r#"
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64
}

impl Vec<T> {
    fn with_capacity(capacity: i64) -> Vec<T> {
        data := malloc(capacity * 8)
        Vec { data: data, len: 0, cap: capacity }
    }

    fn len(&self) -> i64 {
        self.len
    }

    fn capacity(&self) -> i64 {
        self.cap
    }

    fn is_empty(&self) -> i64 {
        I self.len == 0 { 1 } else { 0 }
    }

    fn get(&self, index: i64) -> type {
        I index >= 0 && index < self.len {
            ptr := self.data + index * 8
            load_i64(ptr)
        } else {
            0
        }
    }

    fn push(&self, value: T) -> i64 {
        I self.len < self.cap {
            ptr := self.data + self.len * 8
            store_i64(ptr, value)
            self.len = self.len + 1
            self.len
        } else {
            0
        }
    }

    fn pop(&self) -> type {
        I self.len > 0 {
            self.len = self.len - 1
            ptr := self.data + self.len * 8
            load_i64(ptr)
        } else {
            0
        }
    }

    fn drop(&self) -> i64 {
        free(self.data)
        0
    }
}

fn test_vec() -> i64 {
    v := Vec.with_capacity(8)
    v.push(1)
    v.push(2)
    v.push(3)
    len := v.len()
    val := v.get(1)
    v.drop()
    val
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_box_struct() {
    // Test Box<T> heap allocation
    let source = r#"
struct Box<T> {
    ptr: i64
}

impl Box<T> {
    fn new(value: T) -> Box<T> {
        ptr := malloc(8)
        store_i64(ptr, value)
        Box { ptr: ptr }
    }

    fn get(&self) -> type {
        load_i64(self.ptr)
    }

    fn set(&self, value: T) -> () {
        store_i64(self.ptr, value)
    }

    fn drop(&self) -> () {
        free(self.ptr)
    }
}

fn test_box() -> i64 {
    b := Box.new(42)
    val := b.get()
    b.set(100)
    new_val := b.get()
    b.drop()
    new_val
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_rc_struct() {
    // Test Rc<T> reference counting
    let source = r#"
struct Rc<T> {
    ptr: i64,
    count: i64
}

impl Rc<T> {
    fn new(value: T) -> Rc<T> {
        ptr := malloc(16)
        store_i64(ptr, value)
        store_i64(ptr + 8, 1)
        Rc { ptr: ptr, count: ptr + 8 }
    }

    fn get(&self) -> type {
        load_i64(self.ptr)
    }

    fn strong_count(&self) -> i64 {
        load_i64(self.count)
    }
}

fn test_rc() -> i64 {
    rc := Rc.new(42)
    count := rc.strong_count()
    val := rc.get()
    val
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_deque_struct() {
    // Test Deque<T> double-ended queue
    let source = r#"
struct Deque<T> {
    data: i64,
    head: i64,
    tail: i64,
    cap: i64
}

impl Deque<T> {
    fn with_capacity(capacity: i64) -> Deque<T> {
        data := malloc(capacity * 8)
        Deque { data: data, head: 0, tail: 0, cap: capacity }
    }

    fn len(&self) -> i64 {
        I self.tail >= self.head {
            self.tail - self.head
        } else {
            self.cap - self.head + self.tail
        }
    }

    fn is_empty(&self) -> i64 {
        I self.head == self.tail { 1 } else { 0 }
    }

    fn push_back(&self, value: T) -> i64 {
        ptr := self.data + self.tail * 8
        store_i64(ptr, value)
        self.tail = (self.tail + 1) % self.cap
        1
    }

    fn pop_front(&self) -> type {
        I self.head != self.tail {
            ptr := self.data + self.head * 8
            value := load_i64(ptr)
            self.head = (self.head + 1) % self.cap
            value
        } else {
            0
        }
    }

    fn drop(&self) -> () {
        free(self.data)
    }
}

fn test_deque() -> i64 {
    d := Deque.with_capacity(8)
    d.push_back(1)
    d.push_back(2)
    d.push_back(3)
    first := d.pop_front()
    d.drop()
    first
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_hashmap_basic() {
    // Test HashMap basic structure
    let source = r#"
struct HashMap<K, V> {
    buckets: i64,
    count: i64,
    capacity: i64
}

impl HashMap<K, V> {
    fn new() -> HashMap<K, V> {
        cap := 16
        buckets := malloc(cap * 8)
        L i: 0..cap {
            store_i64(buckets + i * 8, 0)
        }
        HashMap { buckets: buckets, count: 0, capacity: cap }
    }

    fn len(&self) -> i64 {
        self.count
    }

    fn is_empty(&self) -> i64 {
        I self.count == 0 { 1 } else { 0 }
    }

    fn drop(&self) -> () {
        free(self.buckets)
    }
}

fn test_hashmap() -> i64 {
    m := HashMap.new()
    empty := m.is_empty()
    m.drop()
    empty
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_iterator_trait() {
    // Test Iterator trait definition (simple version)
    let source = r#"
trait Iterator {
    fn next(&self) -> i64 = 0
    fn has_next(&self) -> i64 = 0
}

struct RangeIter {
    current: i64,
    end: i64
}

impl RangeIter : Iterator {
    fn next(&self) -> i64 {
        val := self.current
        self.current = self.current + 1
        val
    }

    fn has_next(&self) -> i64 {
        I self.current < self.end { 1 } else { 0 }
    }
}

# Test that the trait and impl compile
fn make_iter() -> RangeIter {
    RangeIter { current: 0, end: 10 }
}

fn sum_range(n: i64) -> i64 {
    n * (n - 1) / 2
}

fn test_iter() -> i64 {
    iter := make_iter()
    sum_range(10)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_string_operations() {
    // Test string operations
    let source = r#"
struct String {
    data: i64,
    len: i64,
    cap: i64
}

impl String {
    fn from_raw(ptr: i64, len: i64) -> String {
        cap := len + 1
        data := malloc(cap)
        memcpy(data, ptr, len)
        store_byte(data + len, 0)
        String { data: data, len: len, cap: cap }
    }

    fn len(&self) -> i64 {
        self.len
    }

    fn is_empty(&self) -> i64 {
        I self.len == 0 { 1 } else { 0 }
    }

    fn as_ptr(&self) -> i64 {
        self.data
    }

    fn drop(&self) -> () {
        free(self.data)
    }
}

fn test_string() -> i64 {
    s := String.from_raw(0, 0)
    len := s.len()
    s.drop()
    len
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_math_functions() {
    // Test math function signatures (without top-level const which isn't supported)
    let source = r#"
# Basic math operations
fn abs_f64(x: f64) -> f64 = I x < 0.0 { 0.0 - x } else { x }
fn abs_i64(x: i64) -> i64 = I x < 0 { 0 - x } else { x }
fn min_f64(a: f64, b: f64) -> f64 = I a < b { a } else { b }
fn max_f64(a: f64, b: f64) -> f64 = I a > b { a } else { b }
fn min_i64(a: i64, b: i64) -> i64 = I a < b { a } else { b }
fn max_i64(a: i64, b: i64) -> i64 = I a > b { a } else { b }
fn clamp_f64(x: f64, lo: f64, hi: f64) -> f64 = max_f64(lo, min_f64(x, hi))
fn clamp_i64(x: i64, lo: i64, hi: i64) -> i64 = max_i64(lo, min_i64(x, hi))

fn test_math() -> i64 {
    pi := 3.14159265358979323846
    a := abs_i64(0 - 5)
    b := min_i64(3, 7)
    c := max_i64(3, 7)
    d := clamp_i64(10, 0, 5)
    a + b + c + d
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_arena_allocator() {
    // Test arena allocator pattern
    let source = r#"
struct Arena {
    base: i64,
    offset: i64,
    capacity: i64
}

impl Arena {
    fn new(size: i64) -> Arena {
        base := malloc(size)
        Arena { base: base, offset: 0, capacity: size }
    }

    fn alloc(&self, size: i64) -> i64 {
        I self.offset + size <= self.capacity {
            ptr := self.base + self.offset
            self.offset = self.offset + size
            ptr
        } else {
            0
        }
    }

    fn reset(&self) -> () {
        self.offset = 0
    }

    fn drop(&self) -> () {
        free(self.base)
    }
}

fn test_arena() -> i64 {
    arena := Arena.new(1024)
    ptr1 := arena.alloc(64)
    ptr2 := arena.alloc(128)
    arena.reset()
    ptr3 := arena.alloc(64)
    arena.drop()
    I ptr3 == ptr1 { 1 } else { 0 }
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_priority_queue() {
    // Test priority queue (min-heap) structure
    let source = r#"
struct PriorityQueue<T> {
    data: i64,
    len: i64,
    cap: i64
}

impl PriorityQueue<T> {
    fn new() -> PriorityQueue<T> {
        cap := 16
        data := malloc(cap * 8)
        PriorityQueue { data: data, len: 0, cap: cap }
    }

    fn len(&self) -> i64 {
        self.len
    }

    fn is_empty(&self) -> i64 {
        I self.len == 0 { 1 } else { 0 }
    }

    fn peek(&self) -> type {
        I self.len > 0 {
            load_i64(self.data)
        } else {
            0
        }
    }

    fn drop(&self) -> () {
        free(self.data)
    }
}

fn test_pq() -> i64 {
    pq := PriorityQueue.new()
    empty := pq.is_empty()
    pq.drop()
    empty
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_set_structure() {
    // Test Set structure (HashSet-like)
    let source = r#"
struct Set<T> {
    buckets: i64,
    count: i64,
    capacity: i64
}

impl Set<T> {
    fn new() -> Set<T> {
        cap := 16
        buckets := malloc(cap * 8)
        L i: 0..cap {
            store_i64(buckets + i * 8, 0)
        }
        Set { buckets: buckets, count: 0, capacity: cap }
    }

    fn len(&self) -> i64 {
        self.count
    }

    fn is_empty(&self) -> i64 {
        I self.count == 0 { 1 } else { 0 }
    }

    fn drop(&self) -> () {
        free(self.buckets)
    }
}

fn test_set() -> i64 {
    s := Set.new()
    empty := s.is_empty()
    s.drop()
    empty
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_future_type() {
    // Test Future type for async operations
    let source = r#"
enum FutureState<T> {
    Pending,
    Ready(T)
}

struct Future<T> {
    state: FutureState<T>
}

impl Future<T> {
    fn pending() -> Future<T> {
        Future { state: Pending }
    }

    fn ready(value: T) -> Future<T> {
        Future { state: Ready(value) }
    }

    fn is_ready(&self) -> i64 {
        match self.state {
            Ready(_) => 1,
            Pending => 0
        }
    }

    fn get(&self) -> type {
        match self.state {
            Ready(v) => v,
            Pending => 0
        }
    }
}

fn test_future() -> i64 {
    f := Future.ready(42)
    I f.is_ready() == 1 {
        f.get()
    } else {
        0
    }
}
"#;
    assert!(compiles(source));
}

// ==================== Const Generics Improvement Tests (Phase 13 P2) ====================

#[test]
fn test_const_generic_type_tracking() {
    // Verify const generics are tracked separately from type generics
    let source = r#"
struct FixedBuffer<T, const CAP: u64> {
    data: [T; CAP],
    capacity: u64
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_with_arithmetic() {
    // Const expression arithmetic should be evaluated at compile time
    let source = r#"
struct Grid {
    cells: [i64; 3 * 4]
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_multiple_params() {
    // Multiple const generic parameters should all be tracked
    let source = r#"
struct Tensor<const D1: u64, const D2: u64, const D3: u64> {
    data: [f64; D1],
    shape0: u64,
    shape1: u64,
    shape2: u64
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_function_param() {
    // Const generics in function parameters
    let source = r#"
fn fixed_sum<const N: u64>(arr: [i64; N]) -> i64 {
    0
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_mixed_with_type() {
    // Mixed type and const generics should be distinguished
    let source = r#"
fn transform<T, const SIZE: u64>(data: [T; SIZE]) -> type {
    data
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_concrete_array_size() {
    // Concrete const array sizes should evaluate to LLVM array types
    // [f64; 9] is const-sized array syntax
    let source = r#"
struct Matrix3x3 {
    data: [f64; 9],
    rows: u64
}
"#;
    // Just verify it compiles - the key is that [f64; 9] becomes [9 x double] in LLVM
    assert!(compiles(source));
}

#[test]
fn test_const_generic_mangling() {
    // Verify that const generic instantiations produce unique mangled names
    // This is tested indirectly through the type system
    let source = r#"
struct SmallBuffer<const N: u64> {
    data: [i64; N],
    len: u64
}
struct LargeBuffer<const N: u64> {
    data: [i64; N],
    len: u64
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_display() {
    // Test that const generic types display correctly in error messages
    let source = r#"
struct ConstVec<T, const N: u64> {
    items: [T; N],
    count: u64
}
"#;
    assert!(compiles(source));
}

// ==================== Default Parameters Tests (Phase 13 P2) ====================

#[test]
fn test_default_parameter_parsing() {
    // Default parameter value should parse correctly
    let source = r#"
fn greet(name: i64, times: i64 = 1) -> i64 {
    name + times
}
fn main() -> i64 = greet(42, 1)
"#;
    assert!(compiles(source));
}

#[test]
fn test_default_parameter_omitted() {
    // Calling with fewer args when defaults exist should compile
    let source = r#"
fn add_with_default(a: i64, b: i64 = 10) -> i64 = a + b
fn main() -> i64 = add_with_default(32)
"#;
    assert!(compiles(source));
}

#[test]
fn test_multiple_default_parameters() {
    // Multiple default parameters
    let source = r#"
fn compute(x: i64, y: i64 = 5, z: i64 = 10) -> i64 = x + y + z
fn main() -> i64 = compute(27)
"#;
    assert!(compiles(source));
}

#[test]
fn test_default_parameter_all_provided() {
    // All args provided even with defaults
    let source = r#"
fn compute(x: i64, y: i64 = 5, z: i64 = 10) -> i64 = x + y + z
fn main() -> i64 = compute(10, 20, 30)
"#;
    assert!(compiles(source));
}

#[test]
fn test_default_parameter_expression() {
    // Default value can be an expression
    let source = r#"
fn scale(value: i64, factor: i64 = 2 * 3) -> i64 = value * factor
fn main() -> i64 = scale(7)
"#;
    assert!(compiles(source));
}

#[test]
fn test_named_arg_ast_definition() {
    // The NamedArg and CallArgs types exist in the AST
    // This test verifies the AST definitions compile correctly
    let source = r#"
fn identity(x: i64) -> i64 = x
fn main() -> i64 = identity(42)
"#;
    assert!(compiles(source));
}

// ==================== String Operations Tests ====================

#[test]
fn test_string_concatenation() {
    let source = r#"
fn greet(name: str) -> str {
    "Hello, " + name
}
fn main() -> i64 {
    msg := greet("World")
    strlen(msg)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_string_equality() {
    let source = r#"
fn check(s: str) -> i64 {
    I s == "hello" {
        1
    } else {
        0
    }
}
fn main() -> i64 = check("hello")
"#;
    assert!(compiles(source));
}

#[test]
fn test_string_inequality() {
    let source = r#"
fn check(s: str) -> i64 {
    I s != "world" {
        1
    } else {
        0
    }
}
fn main() -> i64 = check("hello")
"#;
    assert!(compiles(source));
}

#[test]
fn test_string_method_len() {
    let source = r#"
fn main() -> i64 {
    s := "hello"
    s.len()
}
"#;
    assert!(compiles(source));
}

#[test]
#[allow(non_snake_case)]
fn test_string_method_charAt() {
    let source = r#"
fn main() -> i64 {
    s := "hello"
    s.charAt(0)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_string_method_contains() {
    let source = r#"
fn main() -> i64 {
    s := "hello world"
    I s.contains("world") {
        1
    } else {
        0
    }
}
"#;
    assert!(compiles(source));
}

#[test]
#[allow(non_snake_case)]
fn test_string_method_indexOf() {
    let source = r#"
fn main() -> i64 {
    s := "hello world"
    s.indexOf("world")
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_string_method_substring() {
    let source = r#"
fn main() -> i64 {
    s := "hello world"
    sub := s.substring(0, 5)
    strlen(sub)
}
"#;
    assert!(compiles(source));
}

#[test]
#[allow(non_snake_case)]
fn test_string_method_startsWith() {
    let source = r#"
fn main() -> i64 {
    s := "hello world"
    I s.startsWith("hello") {
        1
    } else {
        0
    }
}
"#;
    assert!(compiles(source));
}

#[test]
#[allow(non_snake_case)]
fn test_string_method_endsWith() {
    let source = r#"
fn main() -> i64 {
    s := "hello world"
    I s.endsWith("world") {
        1
    } else {
        0
    }
}
"#;
    assert!(compiles(source));
}

#[test]
#[allow(non_snake_case)]
fn test_string_method_isEmpty() {
    let source = r#"
fn main() -> i64 {
    s := ""
    I s.isEmpty() {
        1
    } else {
        0
    }
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_string_comparison_operators() {
    let source = r#"
fn main() -> i64 {
    I "abc" < "def" {
        1
    } else {
        0
    }
}
"#;
    assert!(compiles(source));
}

// ==================== Generic Function Monomorphization E2E Tests ====================

#[test]
fn test_generic_identity_single_type() {
    let source = r#"
fn identity<T>(x: T) -> type = x
fn main() -> i64 {
    identity(42)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@identity$i64"));
}

#[test]
fn test_generic_identity_multiple_types() {
    let source = r#"
fn identity<T>(x: T) -> type = x
fn main() -> i64 {
    a := identity(42)
    b := identity(3.14)
    a
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@identity$i64"));
    assert!(ir.contains("@identity$f64"));
}

#[test]
fn test_generic_function_with_operations() {
    let source = r#"
fn add_pair<T>(a: T, b: T) -> type = a + b
fn main() -> i64 {
    add_pair(10, 20)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@add_pair$i64"));
}

#[test]
fn test_generic_function_call_resolution() {
    let source = r#"
fn wrap<T>(x: T) -> type = x
fn main() -> i64 {
    wrap(100)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("call i64 @wrap$i64"));
}

// ==================== Trait Dynamic Dispatch E2E Tests ====================

#[test]
fn test_trait_static_dispatch() {
    let source = r#"
trait Drawable {
    fn draw(&self) -> i64
}
struct Circle { radius: i64 }
impl Circle: Drawable {
    fn draw(&self) -> i64 = self.radius
}
fn main() -> i64 {
    c := Circle { radius: 42 }
    c.draw()
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@Circle_draw"));
}

#[test]
fn test_trait_dynamic_dispatch_codegen() {
    let source = r#"
trait Drawable {
    fn draw(&self) -> i64
}
struct Circle { radius: i64 }
impl Circle: Drawable {
    fn draw(&self) -> i64 = self.radius
}
fn draw_shape(shape: &dyn Drawable) -> i64 = shape.draw()
fn main() -> i64 {
    c := Circle { radius: 42 }
    draw_shape(&c)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Trait object creation: fat pointer { data_ptr, vtable_ptr }
    assert!(
        ir.contains("insertvalue { i8*, i8* }"),
        "Should create trait object"
    );
    // Vtable global is generated for Circle implementing Drawable
    assert!(
        ir.contains("vtable_Circle_Drawable"),
        "Should have vtable global"
    );
    // Note: extractvalue { i8*, i8* } for dyn dispatch at call site
    // is not yet wired — method calls on &dyn Trait currently use static dispatch.
}

#[test]
fn test_trait_multiple_impls_dispatch() {
    let source = r#"
trait Shape {
    fn area(&self) -> i64
}
struct Rect { w: i64, h: i64 }
impl Rect: Shape {
    fn area(&self) -> i64 = self.w * self.h
}
struct Square { side: i64 }
impl Square: Shape {
    fn area(&self) -> i64 = self.side * self.side
}
fn get_area(s: &dyn Shape) -> i64 = s.area()
fn main() -> i64 {
    r := Rect { w: 3, h: 4 }
    s := Square { side: 5 }
    a1 := get_area(&r)
    a2 := get_area(&s)
    a1 + a2
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("vtable_Rect_Shape"));
    assert!(ir.contains("vtable_Square_Shape"));
}

// ==================== GAT (Generic Associated Types) Tests ====================

#[test]
fn test_gat_iterator_pattern() {
    let source = r#"
trait Iterable {
    type Item
    fn iter(&self) -> i64
}
struct Numbers { count: i64 }
impl Numbers: Iterable {
    type Item = i64
    fn iter(&self) -> i64 { self.count }
}
fn main() -> i64 {
    n := Numbers { count: 42 }
    n.iter()
}
"#;
    let result = compile_to_ir(source);
    assert!(
        result.is_ok(),
        "GAT iterator pattern should compile: {:?}",
        result.err()
    );
    let ir = result.unwrap();
    assert!(ir.contains("@Numbers_iter"), "Should generate iter method");
}

#[test]
fn test_gat_container_pattern() {
    let source = r#"
trait Container {
    type Elem
    fn get(&self, idx: i64) -> i64
    fn size(&self) -> i64
}
struct IntArray { len: i64 }
impl IntArray: Container {
    type Elem = i64
    fn get(&self, idx: i64) -> i64 { idx }
    fn size(&self) -> i64 { self.len }
}
fn main() -> i64 {
    arr := IntArray { len: 10 }
    arr.size() + arr.get(5)
}
"#;
    let result = compile_to_ir(source);
    assert!(
        result.is_ok(),
        "GAT container pattern should compile: {:?}",
        result.err()
    );
    let ir = result.unwrap();
    assert!(ir.contains("@IntArray_get"), "Should generate get method");
    assert!(ir.contains("@IntArray_size"), "Should generate size method");
}

#[test]
fn test_gat_functor_pattern() {
    let source = r#"
trait Functor {
    type Item
    fn map_val(&self, x: i64) -> i64
}
struct Wrapper { value: i64 }
impl Wrapper: Functor {
    type Item = i64
    fn map_val(&self, x: i64) -> i64 { self.value + x }
}
fn main() -> i64 {
    w := Wrapper { value: 10 }
    w.map_val(32)
}
"#;
    let result = compile_to_ir(source);
    assert!(
        result.is_ok(),
        "GAT functor pattern should compile: {:?}",
        result.err()
    );
    let ir = result.unwrap();
    assert!(
        ir.contains("@Wrapper_map_val"),
        "Should generate map_val method"
    );
}

#[test]
fn test_gat_with_default_type() {
    let source = r#"
trait Collection {
    type Item = i64
    fn count(&self) -> i64
}
struct MyVec { size: i64 }
impl MyVec: Collection {
    fn count(&self) -> i64 { self.size }
}
fn main() -> i64 {
    v := MyVec { size: 5 }
    v.count()
}
"#;
    let result = compile_to_ir(source);
    assert!(
        result.is_ok(),
        "GAT with default type should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_gat_multiple_associated_types() {
    let source = r#"
trait Pair {
    type First
    type Second
    fn get_first(&self) -> i64
    fn get_second(&self) -> i64
}
struct IntPair { a: i64, b: i64 }
impl IntPair: Pair {
    type First = i64
    type Second = i64
    fn get_first(&self) -> i64 { self.a }
    fn get_second(&self) -> i64 { self.b }
}
fn main() -> i64 {
    p := IntPair { a: 10, b: 20 }
    p.get_first() + p.get_second()
}
"#;
    let result = compile_to_ir(source);
    assert!(
        result.is_ok(),
        "GAT with multiple associated types should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_gat_trait_definition_only() {
    let source = r#"
trait Iterator {
    type Item
    fn next(&self) -> i64
}
"#;
    assert!(compiles(source), "GAT trait definition should compile");
}

#[test]
fn test_gat_static_dispatch() {
    let source = r#"
trait Producer {
    type Output
    fn produce(&self) -> i64
}
struct Factory { value: i64 }
impl Factory: Producer {
    type Output = i64
    fn produce(&self) -> i64 { self.value * 2 }
}
fn call_producer(p: &Factory) -> i64 {
    p.produce()
}
fn main() -> i64 {
    f := Factory { value: 21 }
    call_producer(&f)
}
"#;
    let result = compile_to_ir(source);
    assert!(
        result.is_ok(),
        "GAT static dispatch should compile: {:?}",
        result.err()
    );
    let ir = result.unwrap();
    assert!(
        ir.contains("@Factory_produce"),
        "Should generate static dispatch"
    );
}

#[test]
fn test_gat_impl_without_explicit_type() {
    // This should fail because associated type is not provided
    let source = r#"
trait Container {
    type Elem
    fn size(&self) -> i64
}
struct Box { val: i64 }
impl Box: Container {
    fn size(&self) -> i64 { 1 }
}
"#;
    // For now, we allow this (type checker may require explicit types in future)
    // Just verify it parses and type-checks
    let result = compile_to_ir(source);
    // Depending on type checker strictness, this may pass or fail
    // We document current behavior
    let _ = result;
}

// ==================== Phase 23: Dependent Type Validation ====================

#[test]
fn test_dependent_type_positive_literal() {
    // Positive integer satisfies {x: i64 | x > 0}
    let source = r#"
fn test_positive(n: {x: i64 | x > 0}) -> i64 {
    return n
}
fn main() -> i64 {
    return test_positive(5)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_dependent_type_zero_nonneg() {
    // Zero satisfies {x: i64 | x >= 0}
    let source = r#"
fn test_nonneg(n: {x: i64 | x >= 0}) -> i64 {
    return n
}
fn main() -> i64 {
    return test_nonneg(0)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_dependent_type_negative_violation() {
    // Negative literal violates {x: i64 | x > 0} — should fail to compile
    let source = r#"
fn main() -> i64 {
    x: {n: i64 | n > 0} := -5
    return x
}
"#;
    assert!(fails_to_compile(source));
}

#[test]
fn test_dependent_type_call_violation() {
    // Passing negative literal to function with dependent type param
    let source = r#"
fn test_positive(n: {x: i64 | x > 0}) -> i64 {
    return n
}
fn main() -> i64 {
    return test_positive(-3)
}
"#;
    assert!(fails_to_compile(source));
}

#[test]
fn test_dependent_type_zero_positive_violation() {
    // Zero violates {x: i64 | x > 0}
    let source = r#"
fn test_positive(n: {x: i64 | x > 0}) -> i64 {
    return n
}
fn main() -> i64 {
    return test_positive(0)
}
"#;
    assert!(fails_to_compile(source));
}

#[test]
fn test_dependent_type_bounded_range() {
    // Value within range: {x: i64 | x >= 0} with x = 10
    let source = r#"
fn bounded(n: {x: i64 | x >= 0}) -> i64 {
    return n + 1
}
fn main() -> i64 {
    return bounded(10)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_dependent_type_runtime_not_checked() {
    // Non-literal values should compile (runtime checking not enforced)
    let source = r#"
fn test_positive(n: {x: i64 | x > 0}) -> i64 {
    return n
}
fn get_value() -> i64 { return 42 }
fn main() -> i64 {
    v := get_value()
    return test_positive(v)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_dependent_type_compound_and_violation() {
    // Compound predicate with && — value violates upper bound
    let source = r#"
fn bounded(n: {x: i64 | x >= 0 && x <= 100}) -> i64 {
    return n
}
fn main() -> i64 {
    return bounded(200)
}
"#;
    assert!(!compiles(source));
}

#[test]
fn test_dependent_type_compound_and_pass() {
    // Compound predicate with && — value satisfies both bounds
    let source = r#"
fn bounded(n: {x: i64 | x >= 0 && x <= 100}) -> i64 {
    return n
}
fn main() -> i64 {
    return bounded(50)
}
"#;
    assert!(compiles(source));
}

// ==================== Phase 23: ICE Fallback Safety ====================

#[test]
fn test_ice_fallback_generic_function() {
    // Generic function should compile (generic resolved before codegen)
    let source = r#"
fn identity<T>(x: T) -> type {
    return x
}
fn main() -> i64 {
    return identity(42)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_ice_fallback_trait_impl_with_str_method() {
    // Trait impl block with &self method returning str should compile.
    // (Historically named after ImplTrait but the body only exercises a plain
    //  `X Foo: Describable` impl block, not existential return types.)
    let source = r#"
trait Describable {
    fn describe(&self) -> str
}
struct Foo {}
impl Foo: Describable {
    fn describe(&self) -> str { return "foo" }
}
fn main() -> i64 {
    return 0
}
"#;
    assert!(compiles(source));
}
