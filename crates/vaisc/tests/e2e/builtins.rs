use super::helpers::*;

// ============================================================
// Real-World Project Tests (Phase 13 P2 - Business Logic)
// ============================================================

#[test]
fn e2e_project_fibonacci_computation() {
    let source = r#"
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
F main() -> i64 = fib(10)
"#;
    assert_exit_code(source, 55);
}

#[test]
fn e2e_project_factorial_computation() {
    let source = r#"
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)
F main() -> i64 = factorial(5)
"#;
    assert_exit_code(source, 120);
}

#[test]
fn e2e_project_gcd_algorithm() {
    let source = r#"
F gcd(a: i64, b: i64) -> i64 = I b == 0 { a } E { gcd(b, a % b) }
F main() -> i64 = gcd(48, 18)
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_project_lcm_algorithm() {
    let source = r#"
F gcd(a: i64, b: i64) -> i64 = I b == 0 { a } E { gcd(b, a % b) }
F lcm(a: i64, b: i64) -> i64 = a / gcd(a, b) * b
F main() -> i64 = lcm(12, 8)
"#;
    assert_exit_code(source, 24);
}

#[test]
fn e2e_project_power_function() {
    let source = r#"
F power(base: i64, exp: i64) -> i64 =
    I exp == 0 { 1 }
    E I exp == 1 { base }
    E { base * @(base, exp - 1) }
F main() -> i64 = power(2, 10) % 256
"#;
    // 2^10 = 1024, 1024 % 256 = 0
    assert_exit_code(source, 0);
}

#[test]
fn e2e_project_is_prime() {
    let source = r#"
F is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    E I n % d == 0 { 0 }
    E { @(n, d + 2) }
F is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    E I n < 4 { 1 }
    E I n % 2 == 0 { 0 }
    E { is_prime_helper(n, 3) }
F main() -> i64 = is_prime(97)
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_project_count_primes() {
    let source = r#"
F is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    E I n % d == 0 { 0 }
    E { @(n, d + 2) }
F is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    E I n < 4 { 1 }
    E I n % 2 == 0 { 0 }
    E { is_prime_helper(n, 3) }
F count_primes_helper(n: i64, current: i64, count: i64) -> i64 =
    I current > n { count }
    E { @(n, current + 1, count + is_prime(current)) }
F count_primes(n: i64) -> i64 = count_primes_helper(n, 2, 0)
F main() -> i64 = count_primes(100)
"#;
    // 25 primes under 100
    assert_exit_code(source, 25);
}

#[test]
fn e2e_project_integer_sqrt() {
    let source = r#"
F isqrt_helper(n: i64, guess: i64) -> i64 {
    next := (guess + n / guess) / 2
    I next == guess { guess }
    E I next > guess { guess }
    E { @(n, next) }
}
F isqrt(n: i64) -> i64 = I n < 2 { n } E { isqrt_helper(n, n / 2) }
F main() -> i64 = isqrt(144)
"#;
    assert_exit_code(source, 12);
}

#[test]
fn e2e_project_sum_to_n_tail_recursive() {
    let source = r#"
F sum_to_acc(n: i64, acc: i64) -> i64 =
    I n == 0 { acc } E { @(n - 1, acc + n) }
F sum_to(n: i64) -> i64 = sum_to_acc(n, 0)
F main() -> i64 = sum_to(100) % 256
"#;
    // sum(1..100) = 5050, 5050 % 256 = 186
    assert_exit_code(source, 186);
}

#[test]
fn e2e_project_array_statistics() {
    let source = r#"
F array_sum(arr: *i64, len: i64, idx: i64) -> i64 =
    I idx == len { 0 }
    E { arr[idx] + @(arr, len, idx + 1) }
F array_min(arr: *i64, len: i64, idx: i64, current_min: i64) -> i64 =
    I idx == len { current_min }
    E I arr[idx] < current_min { @(arr, len, idx + 1, arr[idx]) }
    E { @(arr, len, idx + 1, current_min) }
F array_max(arr: *i64, len: i64, idx: i64, current_max: i64) -> i64 =
    I idx == len { current_max }
    E I arr[idx] > current_max { @(arr, len, idx + 1, arr[idx]) }
    E { @(arr, len, idx + 1, current_max) }
F main() -> i64 {
    data: *i64 = [42, 17, 93, 5, 68]
    sum := array_sum(data, 5, 0)
    min := array_min(data, 5, 1, data[0])
    max := array_max(data, 5, 1, data[0])
    I sum == 225 { I min == 5 { I max == 93 { 42 } E { 2 } } E { 1 } } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_project_nth_prime() {
    let source = r#"
F is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    E I n % d == 0 { 0 }
    E { @(n, d + 2) }
F is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    E I n < 4 { 1 }
    E I n % 2 == 0 { 0 }
    E { is_prime_helper(n, 3) }
F nth_prime_helper(target: i64, current: i64, found: i64) -> i64 =
    I found == target { current - 1 }
    E I is_prime(current) == 1 { @(target, current + 1, found + 1) }
    E { @(target, current + 1, found) }
F nth_prime(n: i64) -> i64 = nth_prime_helper(n, 2, 0)
F main() -> i64 = nth_prime(10)
"#;
    // 10th prime is 29
    assert_exit_code(source, 29);
}

#[test]
fn e2e_project_math_cli_output() {
    let source = r#"
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
F print_digit(d: i64) -> i64 { putchar(d + 48); 0 }
F print_num_helper(n: i64) -> i64 =
    I n < 10 { print_digit(n) }
    E { print_num_helper(n / 10); print_digit(n % 10) }
F print_num(n: i64) -> i64 =
    I n == 0 { print_digit(0) }
    E { print_num_helper(n) }
F main() -> i64 {
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
F collatz_steps(n: i64, steps: i64) -> i64 =
    I n == 1 { steps }
    E I n % 2 == 0 { @(n / 2, steps + 1) }
    E { @(3 * n + 1, steps + 1) }
F main() -> i64 = collatz_steps(27, 0)
"#;
    // Collatz sequence for 27 takes 111 steps
    assert_exit_code(source, 111);
}

#[test]
fn e2e_project_binary_search() {
    let source = r#"
F binary_search(arr: *i64, target: i64, lo: i64, hi: i64) -> i64 =
    I lo > hi { 0 - 1 }
    E {
        mid := (lo + hi) / 2;
        I arr[mid] == target { mid }
        E I arr[mid] < target { @(arr, target, mid + 1, hi) }
        E { @(arr, target, lo, mid - 1) }
    }
F main() -> i64 {
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
F count_greater(arr: *i64, len: i64, idx: i64, threshold: i64) -> i64 =
    I idx == len { 0 }
    E I arr[idx] > threshold { 1 + @(arr, len, idx + 1, threshold) }
    E { @(arr, len, idx + 1, threshold) }
F main() -> i64 {
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
F main() -> i64 {
    println("Hello, World!")
    0
}
"#;
    assert_stdout_contains(source, "Hello, World!");
}

#[test]
fn e2e_print_simple_string() {
    let source = r#"
F main() -> i64 {
    print("Hello")
    0
}
"#;
    assert_stdout_contains(source, "Hello");
}

#[test]
fn e2e_println_format_integer() {
    let source = r#"
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
    println("name: {}", "Vais")
    0
}
"#;
    assert_stdout_contains(source, "name: Vais");
}

#[test]
fn e2e_print_no_newline() {
    let source = r#"
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
    x: i32 = atoi("42")
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_atol() {
    let source = r#"
F main() -> i64 {
    atol("99")
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_labs() {
    let source = r#"
F main() -> i64 {
    labs(-42)
}
"#;
    assert_exit_code(source, 42);
}
