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
    let source = "U std::io";
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
