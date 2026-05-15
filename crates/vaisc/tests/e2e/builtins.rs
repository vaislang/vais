use super::helpers::*;

// ============================================================
// Real-World Project Tests (Phase 13 P2 - Business Logic)
// ============================================================
// Note: fibonacci (fib(10)=55) and factorial (factorial(5)=120) tests
// are covered in execution_tests.rs (exec_recursion_fibonacci / exec_recursion_factorial).

#[test]
fn e2e_project_gcd_algorithm() {
    let source = r#"
fn gcd(a: i64, b: i64) -> i64 = I b == 0 { a } else { gcd(b, a % b) }
fn main() -> i64 = gcd(48, 18)
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_project_lcm_algorithm() {
    let source = r#"
fn gcd(a: i64, b: i64) -> i64 = I b == 0 { a } else { gcd(b, a % b) }
fn lcm(a: i64, b: i64) -> i64 = a / gcd(a, b) * b
fn main() -> i64 = lcm(12, 8)
"#;
    assert_exit_code(source, 24);
}

#[test]
fn e2e_project_power_function() {
    let source = r#"
fn power(base: i64, exp: i64) -> i64 =
    I exp == 0 { 1 }
    else I exp == 1 { base }
    else { base * @(base, exp - 1) }
fn main() -> i64 = power(2, 10) % 256
"#;
    // 2^10 = 1024, 1024 % 256 = 0
    assert_exit_code(source, 0);
}

#[test]
fn e2e_project_is_prime() {
    let source = r#"
fn is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    else I n % d == 0 { 0 }
    else { @(n, d + 2) }
fn is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    else I n < 4 { 1 }
    else I n % 2 == 0 { 0 }
    else { is_prime_helper(n, 3) }
fn main() -> i64 = is_prime(97)
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_project_count_primes() {
    let source = r#"
fn is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    else I n % d == 0 { 0 }
    else { @(n, d + 2) }
fn is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    else I n < 4 { 1 }
    else I n % 2 == 0 { 0 }
    else { is_prime_helper(n, 3) }
fn count_primes_helper(n: i64, current: i64, count: i64) -> i64 =
    I current > n { count }
    else { @(n, current + 1, count + is_prime(current)) }
fn count_primes(n: i64) -> i64 = count_primes_helper(n, 2, 0)
fn main() -> i64 = count_primes(100)
"#;
    // 25 primes under 100
    assert_exit_code(source, 25);
}

#[test]
fn e2e_project_integer_sqrt() {
    let source = r#"
fn isqrt_helper(n: i64, guess: i64) -> i64 {
    next := (guess + n / guess) / 2
    I next == guess { guess }
    else I next > guess { guess }
    else { @(n, next) }
}
fn isqrt(n: i64) -> i64 = I n < 2 { n } else { isqrt_helper(n, n / 2) }
fn main() -> i64 = isqrt(144)
"#;
    assert_exit_code(source, 12);
}

#[test]
fn e2e_project_sum_to_n_tail_recursive() {
    let source = r#"
fn sum_to_acc(n: i64, acc: i64) -> i64 =
    I n == 0 { acc } else { @(n - 1, acc + n) }
fn sum_to(n: i64) -> i64 = sum_to_acc(n, 0)
fn main() -> i64 = sum_to(100) % 256
"#;
    // sum(1..100) = 5050, 5050 % 256 = 186
    assert_exit_code(source, 186);
}

#[test]
fn e2e_project_array_statistics() {
    let source = r#"
fn array_sum(arr: *i64, len: i64, idx: i64) -> i64 =
    I idx == len { 0 }
    else { arr[idx] + @(arr, len, idx + 1) }
fn array_min(arr: *i64, len: i64, idx: i64, current_min: i64) -> i64 =
    I idx == len { current_min }
    else I arr[idx] < current_min { @(arr, len, idx + 1, arr[idx]) }
    else { @(arr, len, idx + 1, current_min) }
fn array_max(arr: *i64, len: i64, idx: i64, current_max: i64) -> i64 =
    I idx == len { current_max }
    else I arr[idx] > current_max { @(arr, len, idx + 1, arr[idx]) }
    else { @(arr, len, idx + 1, current_max) }
fn main() -> i64 {
    data: *i64 = [42, 17, 93, 5, 68]
    sum := array_sum(data, 5, 0)
    min := array_min(data, 5, 1, data[0])
    max := array_max(data, 5, 1, data[0])
    I sum == 225 { I min == 5 { I max == 93 { 42 } else { 2 } } else { 1 } } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_project_nth_prime() {
    let source = r#"
fn is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    else I n % d == 0 { 0 }
    else { @(n, d + 2) }
fn is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    else I n < 4 { 1 }
    else I n % 2 == 0 { 0 }
    else { is_prime_helper(n, 3) }
fn nth_prime_helper(target: i64, current: i64, found: i64) -> i64 =
    I found == target { current - 1 }
    else I is_prime(current) == 1 { @(target, current + 1, found + 1) }
    else { @(target, current + 1, found) }
fn nth_prime(n: i64) -> i64 = nth_prime_helper(n, 2, 0)
fn main() -> i64 = nth_prime(10)
"#;
    // 10th prime is 29
    assert_exit_code(source, 29);
}

#[test]
fn e2e_project_math_cli_output() {
    let source = r#"
fn fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
fn print_digit(d: i64) -> i64 { putchar(d + 48); 0 }
fn print_num_helper(n: i64) -> i64 =
    I n < 10 { print_digit(n) }
    else { print_num_helper(n / 10); print_digit(n % 10) }
fn print_num(n: i64) -> i64 =
    I n == 0 { print_digit(0) }
    else { print_num_helper(n) }
fn main() -> i64 {
    puts("fib(10)=")
    print_num(fib(10))
    putchar(10)
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("fib(10)="), "should print label");
    assert!(result.stdout.contains("55"), "should print fib(10)=55");
}

#[test]
fn e2e_project_collatz_conjecture() {
    let source = r#"
fn collatz_steps(n: i64, steps: i64) -> i64 =
    I n == 1 { steps }
    else I n % 2 == 0 { @(n / 2, steps + 1) }
    else { @(3 * n + 1, steps + 1) }
fn main() -> i64 = collatz_steps(27, 0)
"#;
    // Collatz sequence for 27 takes 111 steps
    assert_exit_code(source, 111);
}

#[test]
fn e2e_project_binary_search() {
    let source = r#"
fn binary_search(arr: *i64, target: i64, lo: i64, hi: i64) -> i64 =
    I lo > hi { 0 - 1 }
    else {
        mid := (lo + hi) / 2;
        I arr[mid] == target { mid }
        else I arr[mid] < target { @(arr, target, mid + 1, hi) }
        else { @(arr, target, lo, mid - 1) }
    }
fn main() -> i64 {
    sorted: *i64 = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100]
    binary_search(sorted, 70, 0, 9)
}
"#;
    // 70 is at index 6
    assert_exit_code(source, 6);
}

#[test]
fn e2e_project_count_array_elements() {
    let source = r#"
fn count_greater(arr: *i64, len: i64, idx: i64, threshold: i64) -> i64 =
    I idx == len { 0 }
    else I arr[idx] > threshold { 1 + @(arr, len, idx + 1, threshold) }
    else { @(arr, len, idx + 1, threshold) }
fn main() -> i64 {
    data: *i64 = [42, 17, 93, 5, 68, 31, 85, 12, 76, 54]
    count_greater(data, 10, 0, 50)
}
"#;
    // Elements > 50: 93, 68, 85, 76, 54 = 5
    assert_exit_code(source, 5);
}

// === print/println built-in tests ===

#[test]
fn e2e_println_simple_string() {
    let source = r#"
fn main() -> i64 {
    println("Hello, World!")
    0
}
"#;
    assert_stdout_contains(source, "Hello, World!");
}

#[test]
fn e2e_print_simple_string() {
    let source = r#"
fn main() -> i64 {
    print("Hello")
    0
}
"#;
    assert_stdout_contains(source, "Hello");
}

#[test]
fn e2e_println_format_integer() {
    let source = r#"
fn main() -> i64 {
    x: i64 = 42
    println("x = {}", x)
    0
}
"#;
    assert_stdout_contains(source, "x = 42");
}

#[test]
fn e2e_println_format_multiple() {
    let source = r#"
fn main() -> i64 {
    a: i64 = 10
    b: i64 = 20
    println("{} + {} = {}", a, b, a + b)
    0
}
"#;
    assert_stdout_contains(source, "10 + 20 = 30");
}

#[test]
fn e2e_println_format_string_arg() {
    let source = r#"
fn main() -> i64 {
    println("name: {}", "Vais")
    0
}
"#;
    assert_stdout_contains(source, "name: Vais");
}

#[test]
fn e2e_print_no_newline() {
    let source = r#"
fn main() -> i64 {
    print("AB")
    print("CD")
    putchar(10)
    0
}
"#;
    assert_stdout_contains(source, "ABCD");
}

#[test]
fn e2e_println_no_args() {
    let source = r#"
fn main() -> i64 {
    println("done")
    0
}
"#;
    assert_stdout_contains(source, "done");
}

// ==================== Format Function ====================

#[test]
fn e2e_format_simple() {
    let source = r#"
fn main() -> i64 {
    s: str = format("hello {}", 42)
    println(s)
    0
}
"#;
    assert_stdout_contains(source, "hello 42");
}

#[test]
fn e2e_format_multiple_args() {
    let source = r#"
fn main() -> i64 {
    s: str = format("{} + {} = {}", 1, 2, 3)
    println(s)
    0
}
"#;
    assert_stdout_contains(source, "1 + 2 = 3");
}

#[test]
fn e2e_format_no_args() {
    let source = r#"
fn main() -> i64 {
    s: str = format("plain text")
    println(s)
    0
}
"#;
    assert_stdout_contains(source, "plain text");
}

// ==================== Stdlib Utility Functions ====================

#[test]
fn e2e_atoi() {
    let source = r#"
fn main() -> i64 {
    x: i32 = atoi("42")
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_atol() {
    let source = r#"
fn main() -> i64 {
    atol("99")
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_labs() {
    let source = r#"
fn main() -> i64 {
    labs(-42)
}
"#;
    assert_exit_code(source, 42);
}
