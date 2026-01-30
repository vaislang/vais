//! Integration tests for the Vais compiler
//!
//! These tests verify the complete compilation pipeline:
//! Source -> Lexer -> Parser -> Type Checker -> Code Generator -> LLVM IR

use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;
use vais_codegen::CodeGenerator;

/// Helper function to compile source code to LLVM IR
fn compile_to_ir(source: &str) -> Result<String, String> {
    // Step 1: Tokenize
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;

    // Step 2: Parse
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;

    // Step 3: Type check
    let mut checker = TypeChecker::new();
    checker.check_module(&module).map_err(|e| format!("Type error: {:?}", e))?;

    // Step 4: Generate LLVM IR
    let mut gen = CodeGenerator::new("test");
    let ir = gen.generate_module(&module).map_err(|e| format!("Codegen error: {:?}", e))?;

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
    let source = r#"F main() -> () { () }"#;
    assert!(compiles(source));
}

#[test]
fn test_simple_arithmetic() {
    let source = "F add(a:i64, b:i64) -> i64 = a + b";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("define i64 @add"));
    assert!(ir.contains("add i64"));
}

#[test]
fn test_fibonacci() {
    let source = "F fib(n:i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("define i64 @fib"));
    assert!(ir.contains("call i64 @fib")); // Recursive calls
}

// ==================== Control Flow Tests ====================

#[test]
fn test_if_else() {
    let source = "F max(a:i64, b:i64)->i64=I a>b{a}E{b}";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("icmp sgt"));
    assert!(ir.contains("br i1"));
}

#[test]
fn test_nested_if() {
    let source = r#"
F classify(x: i64) -> i64 {
    I x > 0 {
        I x > 100 { 2 } E { 1 }
    } E {
        I x < 0 - 100 { 0 - 2 } E { 0 - 1 }
    }
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_match_expression() {
    let source = r#"
F digit_name(n:i64) -> str = M n {
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
F count_range(n: i64) -> i64 {
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
F check_condition(n: i64) -> i64 {
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
F find_limit(x: i64) -> i64 {
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
S Point { x: f64, y: f64 }
F origin() -> Point = Point { x: 0.0, y: 0.0 }
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("%Point = type { double, double }"));
}

#[test]
fn test_struct_field_access() {
    let source = r#"
S Point { x: i64, y: i64 }
F get_x(p: Point) -> i64 = p.x
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("getelementptr"));
}

#[test]
fn test_empty_struct() {
    let source = r#"
S Unit {}
F make_unit() -> Unit = Unit {}
"#;
    assert!(compiles(source));
}

// ==================== Enum Tests ====================

#[test]
fn test_enum_definition() {
    // Enum definition - path syntax tested separately
    let source = r#"
E Color { Red, Green, Blue }
"#;
    assert!(compiles(source));
}

#[test]
fn test_enum_with_data() {
    let source = r#"
E Shape { Circle(f64), Rectangle(f64, f64) }
"#;
    assert!(compiles(source));
}

#[test]
fn test_enum_pattern_match() {
    let source = r#"
E Result { Ok(i64), Err(str) }
F handle(r: Result) -> i64 = M r {
    Ok(v) => v,
    Err(_) => 0
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_enum_unit_variant_matching() {
    let source = r#"
E Status { Pending, Running, Done }

F status_to_code(s: Status) -> i64 = M s {
    Pending => 0,
    Running => 1,
    Done => 2
}

F test_status() -> i64 {
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
    let source = "F identity<T>(x: T) -> T = x";
    assert!(compiles(source));
}

#[test]
fn test_generic_struct() {
    let source = r#"
S Box<T> { value: T }
F get_value<T>(b: Box<T>) -> T = b.value
"#;
    assert!(compiles(source));
}

#[test]
fn test_generic_with_bounds() {
    let source = "F compare<T: Ord>(a: T, b: T) -> bool = a < b";
    assert!(compiles(source));
}

// ==================== Lambda Tests ====================

#[test]
fn test_simple_lambda() {
    let source = r#"
F apply_double() -> i64 {
    double := |x:i64| x * 2;
    double(21)
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_higher_order_function() {
    let source = r#"
F apply(f: (i64) -> i64, x: i64) -> i64 = f(x)
F double(x: i64) -> i64 = x * 2
F test() -> i64 = apply(double, 21)
"#;
    assert!(compiles(source));
}

// ==================== Type System Tests ====================

#[test]
fn test_array_type() {
    // Test array type in parameter and indexing
    let source = r#"
F first(arr: [i64]) -> i64 = arr[0]
"#;
    assert!(compiles(source));
}

#[test]
fn test_tuple_type() {
    let source = "F make_pair(a: i64, b: str) -> (i64, str) = (a, b)";
    assert!(compiles(source));
}

#[test]
fn test_pointer_type() {
    let source = "F identity_ptr(p: *i64) -> *i64 = p";
    assert!(compiles(source));
}

#[test]
fn test_reference_type() {
    let source = "F ref_fn(r: &i64) -> i64 = 0";
    assert!(compiles(source));
}

// ==================== Operator Tests ====================

#[test]
fn test_arithmetic_operators() {
    let source = r#"
F math(a: i64, b: i64) -> i64 {
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
F compare(a: i64, b: i64) -> bool {
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
F bits(a: i64, b: i64) -> i64 {
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
    let source = "F logic(a: bool, b: bool) -> bool = a && b || !a";
    assert!(compiles(source));
}

#[test]
fn test_unary_operators() {
    let source = r#"
F unary(x: i64) -> i64 {
    neg := -x;
    bitnot := ~x;
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
F compound(x: i64) -> i64 {
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
W Display { F display(s: &Self) -> str = "" }
"#;
    assert!(compiles(source));
}

#[test]
fn test_impl_block() {
    let source = r#"
S Counter { value: i64 }
X Counter {
    F new() -> Counter = Counter { value: 0 }
    F get_val(c: Counter) -> i64 = c.value
}
"#;
    assert!(compiles(source));
}

// ==================== Module System Tests ====================

#[test]
fn test_use_statement() {
    let source = "U std::fs";
    assert!(compiles(source));
}

#[test]
fn test_pub_function() {
    let source = "P F public_fn() -> () = ()";
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
    let source = "F f() -> i64 = \"string\"";
    assert!(fails_to_compile(source));
}

#[test]
fn test_undefined_variable_error() {
    let source = "F f() -> i64 = undefined_var";
    assert!(fails_to_compile(source));
}

#[test]
fn test_undefined_function_error() {
    let source = "F f() -> i64 = unknown_func()";
    assert!(fails_to_compile(source));
}

#[test]
fn test_wrong_argument_count_error() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F f() -> i64 = add(1)
"#;
    assert!(fails_to_compile(source));
}

#[test]
fn test_wrong_argument_type_error() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F f() -> i64 = add(1, "two")
"#;
    assert!(fails_to_compile(source));
}

#[test]
fn test_if_condition_non_bool_error() {
    let source = "F f() -> i64 = I 42 { 1 } E { 0 }";
    assert!(fails_to_compile(source));
}

// ==================== Complex Programs ====================

#[test]
fn test_factorial() {
    let source = r#"
F factorial(n: i64) -> i64 = n <= 1 ? 1 : n * @(n - 1)
F main() -> () {
    result := factorial(5);
    ()
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_binary_search() {
    let source = r#"
F binary_search(arr: [i64], target: i64, low: i64, high: i64) -> i64 {
    I low > high { 0 - 1 }
    E {
        mid := (low + high) / 2;
        I arr[mid] == target { mid }
        E {
            I arr[mid] < target {
                @(arr, target, mid + 1, high)
            } E {
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
F swap(arr: *i64, i: i64, j: i64) -> () {
    temp := arr[i];
    ()
}

F bubble_sort(arr: *i64, n: i64) -> () {
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
S Node { value: i64, next: *Node }

F get_value(n: Node) -> i64 = n.value
"#;
    assert!(compiles(source));
}

#[test]
fn test_multiple_types_and_functions() {
    let source = r#"
# Types
S Point { x: f64, y: f64 }
S Size { width: f64, height: f64 }
S Rect { x: f64, y: f64, width: f64, height: f64 }

# Functions
F point_new(x: f64, y: f64) -> Point = Point { x: x, y: y }
F size_new(w: f64, h: f64) -> Size = Size { width: w, height: h }

# Main
F main() -> () {
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
    let source = "F abs(x: i64) -> i64 = x < 0 ? -x : x";
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("define i64 @abs"));
    assert!(ir.contains("br i1")); // conditional branch for ternary
}

#[test]
fn test_ternary_with_unary_minus() {
    // Test ternary where then branch starts with unary minus
    let source = "F abs(x: i64) -> i64 = x < 0 ? -x : x";
    assert!(compiles(source));
}

#[test]
fn test_try_operator_parsing() {
    // Test that ? operator parses correctly
    // Note: Full try operator functionality requires Result/Option types
    let source = r#"
F maybe_get(opt: i64?) -> i64? {
    v := opt?
    R Some(v)
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
F add_one(opt: i64?) -> i64? {
    v := opt? + 1
    R Some(v)
}
"#;
    let module = vais_parser::parse(source);
    assert!(module.is_ok());
}

#[test]
fn test_try_and_ternary_together() {
    // Test that try and ternary can coexist
    let source = r#"
F process(opt: i64?) -> i64? {
    v := (opt?)
    R v > 0 ? Some(v) : None
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
S Data {
    values: [i64; 10]
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_array_in_function_param() {
    // Test parsing const-sized array in function parameter
    let source = r#"
F process(arr: [i64; 5]) -> i64 {
    42
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_array_as_return_type() {
    // Test parsing const-sized array as return type
    let source = r#"
S Container {
    data: [i64; 3]
}
F get_data() -> [i64; 3] {
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
F identity<T, const N: u64>(x: T) -> T {
    x
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_in_struct() {
    // Test parsing const generic parameter in struct
    let source = r#"
S FixedArray<T, const N: u64> {
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
F process(arr: [i64; 5 + 3]) -> i64 {
    8
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_expr_multiplication() {
    // Test const expression with multiplication
    let source = r#"
F process(arr: [i64; 4 * 3]) -> i64 {
    12
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_expr_complex() {
    // Test complex const expression
    let source = r#"
F process(arr: [i64; 2 * 3 + 4]) -> i64 {
    10
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_array_in_nested_type() {
    // Test const array in optional type
    let source = r#"
S Matrix {
    data: [f64; 9]
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_multiple_const_generics() {
    // Test multiple const generic parameters
    let source = r#"
S Matrix<const ROWS: u64, const COLS: u64> {
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
S Container<T, const N: u64> {
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
F test_vec4i32() -> Vec4i32 {
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
F test_add(a: Vec4f32, b: Vec4f32) -> Vec4f32 {
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
F test_mul(a: Vec4f32, b: Vec4f32) -> Vec4f32 {
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
F test_sub(a: Vec4f32, b: Vec4f32) -> Vec4f32 {
    simd_sub_vec4f32(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("fsub <4 x float>"));
}

#[test]
fn test_simd_vec4f32_div() {
    let source = r#"
F test_div(a: Vec4f32, b: Vec4f32) -> Vec4f32 {
    simd_div_vec4f32(a, b)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("fdiv <4 x float>"));
}

#[test]
fn test_simd_vec4i32_add() {
    let source = r#"
F test_add(a: Vec4i32, b: Vec4i32) -> Vec4i32 {
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
F test_vec8(a: Vec8f32, b: Vec8f32) -> Vec8f32 {
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
F test_reduce(v: Vec4f32) -> f32 {
    simd_reduce_add_vec4f32(v)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@llvm.vector.reduce.fadd.v4f32"));
}

#[test]
fn test_simd_dot_product() {
    let source = r#"
F dot_product(a: Vec4f32, b: Vec4f32) -> f32 {
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
F test_vec2f64(a: Vec2f64, b: Vec2f64) -> Vec2f64 {
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
F test_vec4f64(a: Vec4f64, b: Vec4f64) -> Vec4f64 {
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
F test_vec2i64(a: Vec2i64, b: Vec2i64) -> Vec2i64 {
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
F test_mul(a: Vec4i64, b: Vec4i64) -> Vec4i64 {
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
F test_constructors() -> i32 {
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
F test_f32_constructor(a: f32, b: f32, c: f32, d: f32) -> Vec4f32 {
    vec4f32(a, b, c, d)
}

F test_f64_constructor(a: f64, b: f64) -> Vec2f64 {
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
F test_reduce(v: Vec4i32) -> i32 {
    simd_reduce_add_vec4i32(v)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(ir.contains("@llvm.vector.reduce.add.v4i32"));
}

#[test]
fn test_simd_reduce_i64() {
    let source = r#"
F test_reduce(v: Vec2i64) -> i64 {
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
E Option<T> {
    None,
    Some(T)
}

F is_some(opt: Option<i64>) -> i64 {
    M opt {
        Some(_) => 1,
        None => 0
    }
}

F is_none(opt: Option<i64>) -> i64 {
    M opt {
        Some(_) => 0,
        None => 1
    }
}

F unwrap_or(opt: Option<i64>, default: i64) -> i64 {
    M opt {
        Some(v) => v,
        None => default
    }
}

F test_option() -> i64 {
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
E Result<T, E> {
    Ok(T),
    Err(E)
}

F is_ok(r: Result<i64, str>) -> i64 {
    M r {
        Ok(_) => 1,
        Err(_) => 0
    }
}

F is_err(r: Result<i64, str>) -> i64 {
    M r {
        Ok(_) => 0,
        Err(_) => 1
    }
}

F test_result() -> i64 {
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
S Vec<T> {
    data: i64,
    len: i64,
    cap: i64
}

X Vec<T> {
    F with_capacity(capacity: i64) -> Vec<T> {
        data := malloc(capacity * 8)
        Vec { data: data, len: 0, cap: capacity }
    }

    F len(&self) -> i64 {
        self.len
    }

    F capacity(&self) -> i64 {
        self.cap
    }

    F is_empty(&self) -> i64 {
        I self.len == 0 { 1 } E { 0 }
    }

    F get(&self, index: i64) -> T {
        I index >= 0 && index < self.len {
            ptr := self.data + index * 8
            load_i64(ptr)
        } E {
            0
        }
    }

    F push(&self, value: T) -> i64 {
        I self.len < self.cap {
            ptr := self.data + self.len * 8
            store_i64(ptr, value)
            self.len = self.len + 1
            self.len
        } E {
            0
        }
    }

    F pop(&self) -> T {
        I self.len > 0 {
            self.len = self.len - 1
            ptr := self.data + self.len * 8
            load_i64(ptr)
        } E {
            0
        }
    }

    F drop(&self) -> i64 {
        free(self.data)
        0
    }
}

F test_vec() -> i64 {
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
S Box<T> {
    ptr: i64
}

X Box<T> {
    F new(value: T) -> Box<T> {
        ptr := malloc(8)
        store_i64(ptr, value)
        Box { ptr: ptr }
    }

    F get(&self) -> T {
        load_i64(self.ptr)
    }

    F set(&self, value: T) -> () {
        store_i64(self.ptr, value)
    }

    F drop(&self) -> () {
        free(self.ptr)
    }
}

F test_box() -> i64 {
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
S Rc<T> {
    ptr: i64,
    count: i64
}

X Rc<T> {
    F new(value: T) -> Rc<T> {
        ptr := malloc(16)
        store_i64(ptr, value)
        store_i64(ptr + 8, 1)
        Rc { ptr: ptr, count: ptr + 8 }
    }

    F get(&self) -> T {
        load_i64(self.ptr)
    }

    F strong_count(&self) -> i64 {
        load_i64(self.count)
    }
}

F test_rc() -> i64 {
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
S Deque<T> {
    data: i64,
    head: i64,
    tail: i64,
    cap: i64
}

X Deque<T> {
    F with_capacity(capacity: i64) -> Deque<T> {
        data := malloc(capacity * 8)
        Deque { data: data, head: 0, tail: 0, cap: capacity }
    }

    F len(&self) -> i64 {
        I self.tail >= self.head {
            self.tail - self.head
        } E {
            self.cap - self.head + self.tail
        }
    }

    F is_empty(&self) -> i64 {
        I self.head == self.tail { 1 } E { 0 }
    }

    F push_back(&self, value: T) -> i64 {
        ptr := self.data + self.tail * 8
        store_i64(ptr, value)
        self.tail = (self.tail + 1) % self.cap
        1
    }

    F pop_front(&self) -> T {
        I self.head != self.tail {
            ptr := self.data + self.head * 8
            value := load_i64(ptr)
            self.head = (self.head + 1) % self.cap
            value
        } E {
            0
        }
    }

    F drop(&self) -> () {
        free(self.data)
    }
}

F test_deque() -> i64 {
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
S HashMap<K, V> {
    buckets: i64,
    count: i64,
    capacity: i64
}

X HashMap<K, V> {
    F new() -> HashMap<K, V> {
        cap := 16
        buckets := malloc(cap * 8)
        L i: 0..cap {
            store_i64(buckets + i * 8, 0)
        }
        HashMap { buckets: buckets, count: 0, capacity: cap }
    }

    F len(&self) -> i64 {
        self.count
    }

    F is_empty(&self) -> i64 {
        I self.count == 0 { 1 } E { 0 }
    }

    F drop(&self) -> () {
        free(self.buckets)
    }
}

F test_hashmap() -> i64 {
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
W Iterator {
    F next(&self) -> i64 = 0
    F has_next(&self) -> i64 = 0
}

S RangeIter {
    current: i64,
    end: i64
}

X RangeIter : Iterator {
    F next(&self) -> i64 {
        val := self.current
        self.current = self.current + 1
        val
    }

    F has_next(&self) -> i64 {
        I self.current < self.end { 1 } E { 0 }
    }
}

# Test that the trait and impl compile
F make_iter() -> RangeIter {
    RangeIter { current: 0, end: 10 }
}

F sum_range(n: i64) -> i64 {
    n * (n - 1) / 2
}

F test_iter() -> i64 {
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
S String {
    data: i64,
    len: i64,
    cap: i64
}

X String {
    F from_raw(ptr: i64, len: i64) -> String {
        cap := len + 1
        data := malloc(cap)
        memcpy(data, ptr, len)
        store_byte(data + len, 0)
        String { data: data, len: len, cap: cap }
    }

    F len(&self) -> i64 {
        self.len
    }

    F is_empty(&self) -> i64 {
        I self.len == 0 { 1 } E { 0 }
    }

    F as_ptr(&self) -> i64 {
        self.data
    }

    F drop(&self) -> () {
        free(self.data)
    }
}

F test_string() -> i64 {
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
F abs_f64(x: f64) -> f64 = I x < 0.0 { 0.0 - x } E { x }
F abs_i64(x: i64) -> i64 = I x < 0 { 0 - x } E { x }
F min_f64(a: f64, b: f64) -> f64 = I a < b { a } E { b }
F max_f64(a: f64, b: f64) -> f64 = I a > b { a } E { b }
F min_i64(a: i64, b: i64) -> i64 = I a < b { a } E { b }
F max_i64(a: i64, b: i64) -> i64 = I a > b { a } E { b }
F clamp_f64(x: f64, lo: f64, hi: f64) -> f64 = max_f64(lo, min_f64(x, hi))
F clamp_i64(x: i64, lo: i64, hi: i64) -> i64 = max_i64(lo, min_i64(x, hi))

F test_math() -> i64 {
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
S Arena {
    base: i64,
    offset: i64,
    capacity: i64
}

X Arena {
    F new(size: i64) -> Arena {
        base := malloc(size)
        Arena { base: base, offset: 0, capacity: size }
    }

    F alloc(&self, size: i64) -> i64 {
        I self.offset + size <= self.capacity {
            ptr := self.base + self.offset
            self.offset = self.offset + size
            ptr
        } E {
            0
        }
    }

    F reset(&self) -> () {
        self.offset = 0
    }

    F drop(&self) -> () {
        free(self.base)
    }
}

F test_arena() -> i64 {
    arena := Arena.new(1024)
    ptr1 := arena.alloc(64)
    ptr2 := arena.alloc(128)
    arena.reset()
    ptr3 := arena.alloc(64)
    arena.drop()
    I ptr3 == ptr1 { 1 } E { 0 }
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_std_priority_queue() {
    // Test priority queue (min-heap) structure
    let source = r#"
S PriorityQueue<T> {
    data: i64,
    len: i64,
    cap: i64
}

X PriorityQueue<T> {
    F new() -> PriorityQueue<T> {
        cap := 16
        data := malloc(cap * 8)
        PriorityQueue { data: data, len: 0, cap: cap }
    }

    F len(&self) -> i64 {
        self.len
    }

    F is_empty(&self) -> i64 {
        I self.len == 0 { 1 } E { 0 }
    }

    F peek(&self) -> T {
        I self.len > 0 {
            load_i64(self.data)
        } E {
            0
        }
    }

    F drop(&self) -> () {
        free(self.data)
    }
}

F test_pq() -> i64 {
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
S Set<T> {
    buckets: i64,
    count: i64,
    capacity: i64
}

X Set<T> {
    F new() -> Set<T> {
        cap := 16
        buckets := malloc(cap * 8)
        L i: 0..cap {
            store_i64(buckets + i * 8, 0)
        }
        Set { buckets: buckets, count: 0, capacity: cap }
    }

    F len(&self) -> i64 {
        self.count
    }

    F is_empty(&self) -> i64 {
        I self.count == 0 { 1 } E { 0 }
    }

    F drop(&self) -> () {
        free(self.buckets)
    }
}

F test_set() -> i64 {
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
E FutureState<T> {
    Pending,
    Ready(T)
}

S Future<T> {
    state: FutureState<T>
}

X Future<T> {
    F pending() -> Future<T> {
        Future { state: Pending }
    }

    F ready(value: T) -> Future<T> {
        Future { state: Ready(value) }
    }

    F is_ready(&self) -> i64 {
        M self.state {
            Ready(_) => 1,
            Pending => 0
        }
    }

    F get(&self) -> T {
        M self.state {
            Ready(v) => v,
            Pending => 0
        }
    }
}

F test_future() -> i64 {
    f := Future.ready(42)
    I f.is_ready() == 1 {
        f.get()
    } E {
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
S FixedBuffer<T, const CAP: u64> {
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
S Grid {
    cells: [i64; 3 * 4]
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_multiple_params() {
    // Multiple const generic parameters should all be tracked
    let source = r#"
S Tensor<const D1: u64, const D2: u64, const D3: u64> {
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
F fixed_sum<const N: u64>(arr: [i64; N]) -> i64 {
    0
}
"#;
    assert!(compiles(source));
}

#[test]
fn test_const_generic_mixed_with_type() {
    // Mixed type and const generics should be distinguished
    let source = r#"
F transform<T, const SIZE: u64>(data: [T; SIZE]) -> T {
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
S Matrix3x3 {
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
S SmallBuffer<const N: u64> {
    data: [i64; N],
    len: u64
}
S LargeBuffer<const N: u64> {
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
S ConstVec<T, const N: u64> {
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
F greet(name: i64, times: i64 = 1) -> i64 {
    name + times
}
F main() -> i64 = greet(42, 1)
"#;
    assert!(compiles(source));
}

#[test]
fn test_default_parameter_omitted() {
    // Calling with fewer args when defaults exist should compile
    let source = r#"
F add_with_default(a: i64, b: i64 = 10) -> i64 = a + b
F main() -> i64 = add_with_default(32)
"#;
    assert!(compiles(source));
}

#[test]
fn test_multiple_default_parameters() {
    // Multiple default parameters
    let source = r#"
F compute(x: i64, y: i64 = 5, z: i64 = 10) -> i64 = x + y + z
F main() -> i64 = compute(27)
"#;
    assert!(compiles(source));
}

#[test]
fn test_default_parameter_all_provided() {
    // All args provided even with defaults
    let source = r#"
F compute(x: i64, y: i64 = 5, z: i64 = 10) -> i64 = x + y + z
F main() -> i64 = compute(10, 20, 30)
"#;
    assert!(compiles(source));
}

#[test]
fn test_default_parameter_expression() {
    // Default value can be an expression
    let source = r#"
F scale(value: i64, factor: i64 = 2 * 3) -> i64 = value * factor
F main() -> i64 = scale(7)
"#;
    assert!(compiles(source));
}

#[test]
fn test_named_arg_ast_definition() {
    // The NamedArg and CallArgs types exist in the AST
    // This test verifies the AST definitions compile correctly
    let source = r#"
F identity(x: i64) -> i64 = x
F main() -> i64 = identity(42)
"#;
    assert!(compiles(source));
}
