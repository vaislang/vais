//! Fuzz tests for the parser
//!
//! These tests ensure that the parser never panics on malformed input,
//! even if it returns parsing errors. The goal is robustness - any input
//! should either parse successfully or return an error, but never crash.

use std::panic::{catch_unwind, AssertUnwindSafe};
use vais_parser::parse;

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

    fn next_u8(&mut self) -> u8 {
        (self.next() & 0xFF) as u8
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        if max <= min {
            return min;
        }
        min + (self.next() as usize % (max - min))
    }

    fn gen_ascii_char(&mut self) -> char {
        // Generate printable ASCII (32-126) plus some control chars
        let c = self.next_u8() % 128;
        c as char
    }

    fn gen_unicode_char(&mut self) -> char {
        // Generate various unicode ranges
        let range = self.next_u8() % 10;
        match range {
            0..=5 => (self.next_u8() % 128) as char, // ASCII
            6 => char::from_u32(0x4E00 + (self.next() % 0x9FFF) as u32).unwrap_or('中'), // CJK
            7 => char::from_u32(0x0400 + (self.next() % 256) as u32).unwrap_or('А'), // Cyrillic
            8 => char::from_u32(0x0600 + (self.next() % 256) as u32).unwrap_or('ا'), // Arabic
            _ => '�',                                // Replacement character
        }
    }
}

/// Helper function to run parser with panic catching
fn parse_no_panic(input: &str) -> Result<bool, String> {
    let result = catch_unwind(AssertUnwindSafe(|| parse(input)));

    match result {
        Ok(_) => Ok(true), // Parser either succeeded or returned an error - both are fine
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
fn fuzz_completely_random_bytes() {
    let mut rng = SimpleRng::new(42);
    let mut failures = Vec::new();

    for i in 0..200 {
        // Generate random length string
        let len = rng.next_range(0, 500);
        let bytes: Vec<u8> = (0..len).map(|_| rng.next_u8()).collect();

        // Try to convert to UTF-8, if it fails, skip (parser expects valid UTF-8)
        if let Ok(input) = String::from_utf8(bytes) {
            if let Err(panic_msg) = parse_no_panic(&input) {
                failures.push((i, input.clone(), panic_msg));
                if failures.len() >= 10 {
                    break; // Stop after finding 10 failures
                }
            }
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (idx, input, panic_msg) in &failures {
            eprintln!("\nTest iteration {}: PANIC!", idx);
            eprintln!(
                "Input (first 200 chars): {:?}",
                input.chars().take(200).collect::<String>()
            );
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Parser panicked on {} random byte inputs (see above)",
            failures.len()
        );
    }
}

#[test]
fn fuzz_random_ascii_strings() {
    let mut rng = SimpleRng::new(100);
    let mut failures = Vec::new();

    for i in 0..300 {
        let len = rng.next_range(0, 1000);
        let input: String = (0..len).map(|_| rng.gen_ascii_char()).collect();

        if let Err(panic_msg) = parse_no_panic(&input) {
            failures.push((i, input.clone(), panic_msg));
            if failures.len() >= 10 {
                break;
            }
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (idx, input, panic_msg) in &failures {
            eprintln!("\nTest iteration {}: PANIC!", idx);
            eprintln!(
                "Input (first 200 chars): {:?}",
                input.chars().take(200).collect::<String>()
            );
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!("Parser panicked on {} ASCII string inputs", failures.len());
    }
}

#[test]
fn fuzz_mutated_valid_code() {
    let valid_programs = vec![
        "F add(x:i64,y:i64)->i64=x+y",
        "S Point{x:i64,y:i64}",
        "E Option<T>{Some(T),None}",
        "F main()->(){x:=42;y:=x+1;R()}",
        "T Show{F show(self)->str}",
        "I Show for i64{F show(self)->str=\"number\"}",
    ];

    let mut rng = SimpleRng::new(200);
    let mut failures = Vec::new();

    for (prog_idx, program) in valid_programs.iter().enumerate() {
        for i in 0..50 {
            let mutation_type = rng.next_u8() % 5;
            let mut mutated = program.to_string();

            match mutation_type {
                0 => {
                    // Delete random character
                    if !mutated.is_empty() {
                        let pos = rng.next_range(0, mutated.len());
                        mutated.remove(pos);
                    }
                }
                1 => {
                    // Insert random character
                    let pos = rng.next_range(0, mutated.len() + 1);
                    let ch = rng.gen_ascii_char();
                    mutated.insert(pos, ch);
                }
                2 => {
                    // Swap two adjacent characters
                    if mutated.len() >= 2 {
                        let pos = rng.next_range(0, mutated.len() - 1);
                        let chars: Vec<char> = mutated.chars().collect();
                        mutated = chars[..pos].iter().collect::<String>()
                            + &chars[pos + 1].to_string()
                            + &chars[pos].to_string()
                            + &chars[pos + 2..].iter().collect::<String>();
                    }
                }
                3 => {
                    // Replace random character
                    if !mutated.is_empty() {
                        let pos = rng.next_range(0, mutated.len());
                        let ch = rng.gen_ascii_char();
                        mutated.remove(pos);
                        mutated.insert(pos, ch);
                    }
                }
                _ => {
                    // Duplicate a substring
                    if !mutated.is_empty() {
                        let start = rng.next_range(0, mutated.len());
                        let end = rng.next_range(start, mutated.len() + 1);
                        let substring = mutated[start..end].to_string();
                        let insert_pos = rng.next_range(0, mutated.len() + 1);
                        mutated.insert_str(insert_pos, &substring);
                    }
                }
            }

            if let Err(panic_msg) = parse_no_panic(&mutated) {
                failures.push((prog_idx, i, mutated.clone(), panic_msg));
                if failures.len() >= 10 {
                    break;
                }
            }
        }
        if failures.len() >= 10 {
            break;
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (prog_idx, iter, input, panic_msg) in &failures {
            eprintln!("\nProgram {} iteration {}: PANIC!", prog_idx, iter);
            eprintln!("Input: {:?}", input);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!("Parser panicked on {} mutated inputs", failures.len());
    }
}

#[test]
fn fuzz_very_long_inputs() {
    let mut rng = SimpleRng::new(300);
    let mut failures = Vec::new();

    for i in 0..20 {
        // Generate very long strings (10K - 50K chars)
        let len = rng.next_range(10000, 50000);
        let input: String = (0..len).map(|_| rng.gen_ascii_char()).collect();

        if let Err(panic_msg) = parse_no_panic(&input) {
            failures.push((i, panic_msg));
            if failures.len() >= 5 {
                break;
            }
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (idx, panic_msg) in &failures {
            eprintln!("\nTest iteration {}: PANIC!", idx);
            eprintln!("Input: very long string (10K-50K chars)");
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!("Parser panicked on {} very long inputs", failures.len());
    }
}

#[test]
fn fuzz_deeply_nested_expressions() {
    let mut failures = Vec::new();

    // Test various nesting patterns
    // Note: Deep nesting (30+) causes stack overflow which aborts the process
    // and can't be caught by catch_unwind. We test moderate nesting here.
    // The fact that deep nesting causes stack overflow is a FINDING of this fuzz test.
    // macOS CI has smaller default stack size — keep max depth at 15 to avoid stack overflow
    for depth in [5, 10, 15] {
        // Nested parentheses
        let mut input = String::new();
        for _ in 0..depth {
            input.push('(');
        }
        input.push('1');
        for _ in 0..depth {
            input.push(')');
        }

        if let Err(panic_msg) = parse_no_panic(&input) {
            failures.push((format!("parens depth {}", depth), panic_msg));
        }

        // Nested function calls
        let mut input2 = String::new();
        for _ in 0..depth {
            input2.push_str("f(");
        }
        input2.push('x');
        for _ in 0..depth {
            input2.push(')');
        }

        if let Err(panic_msg) = parse_no_panic(&input2) {
            failures.push((format!("calls depth {}", depth), panic_msg));
        }

        // Nested blocks
        let mut input3 = String::from("F f()->(){ ");
        for _ in 0..depth {
            input3.push('{');
        }
        input3.push('x');
        for _ in 0..depth {
            input3.push('}');
        }
        input3.push('}');

        if let Err(panic_msg) = parse_no_panic(&input3) {
            failures.push((format!("blocks depth {}", depth), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (desc, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", desc);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!("Parser panicked on {} deeply nested inputs", failures.len());
    }
}

#[test]
#[ignore] // Ignored by default since it aborts the process - run with --ignored to test
fn fuzz_extremely_deep_nesting_causes_stack_overflow() {
    // This test documents that deep nesting (100+) causes stack overflow
    // Stack overflow cannot be caught and will abort the process
    // This is a BUG FOUND by fuzzing - parser should handle this gracefully
    let depth = 100;
    let mut input = String::new();
    for _ in 0..depth {
        input.push('(');
    }
    input.push('1');
    for _ in 0..depth {
        input.push(')');
    }

    // This will cause stack overflow and abort
    let _ = parse(&input);
}

#[test]
fn fuzz_many_sequential_statements() {
    let mut failures = Vec::new();

    for count in [100, 500, 1000, 2000] {
        let mut input = String::from("F f()->(){");
        for i in 0..count {
            input.push_str(&format!("x{}:={};", i, i));
        }
        input.push_str("R()}");

        if let Err(panic_msg) = parse_no_panic(&input) {
            failures.push((format!("{} statements", count), panic_msg));
        }

        // Many function definitions
        let mut input2 = String::new();
        for i in 0..count {
            input2.push_str(&format!("F f{}()->i64={};", i, i));
        }

        if let Err(panic_msg) = parse_no_panic(&input2) {
            failures.push((format!("{} functions", count), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (desc, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", desc);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Parser panicked on {} sequential statement inputs",
            failures.len()
        );
    }
}

#[test]
fn fuzz_unicode_and_special_chars() {
    let mut rng = SimpleRng::new(400);
    let mut failures = Vec::new();

    for i in 0..200 {
        let len = rng.next_range(10, 500);
        let input: String = (0..len).map(|_| rng.gen_unicode_char()).collect();

        if let Err(panic_msg) = parse_no_panic(&input) {
            failures.push((i, input.clone(), panic_msg));
            if failures.len() >= 10 {
                break;
            }
        }
    }

    // Test null bytes
    let null_inputs = vec![
        "\0",
        "F test\0()->i64=42",
        "\0\0\0\0",
        "F\0f\0()\0-\0>i64=42",
    ];

    for (i, input) in null_inputs.iter().enumerate() {
        if let Err(panic_msg) = parse_no_panic(input) {
            failures.push((1000 + i, input.to_string(), panic_msg));
        }
    }

    // Test control characters
    let mut control_test = String::new();
    for i in 0..32 {
        control_test.push(char::from_u32(i).unwrap_or(' '));
    }

    if let Err(panic_msg) = parse_no_panic(&control_test) {
        failures.push((2000, control_test.clone(), panic_msg));
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (idx, input, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!(
                "Input (first 100 chars): {:?}",
                input.chars().take(100).collect::<String>()
            );
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Parser panicked on {} unicode/special char inputs",
            failures.len()
        );
    }
}

#[test]
fn fuzz_empty_and_minimal_inputs() {
    let test_cases = vec![
        "", " ", "\n", "\t", "\r\n", "F", "S", "E", "T", "I", "(", ")", "{", "}", "[", "]", ":",
        ";", ",", "=", "->", "<>", "F()", "F f", "F f(", "F f)", "F f()=", "F f()->", "S{", "E{",
    ];

    let mut failures = Vec::new();

    for (i, input) in test_cases.iter().enumerate() {
        if let Err(panic_msg) = parse_no_panic(input) {
            failures.push((i, input.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (idx, input, panic_msg) in &failures {
            eprintln!("\nTest {}: PANIC!", idx);
            eprintln!("Input: {:?}", input);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!("Parser panicked on {} minimal inputs", failures.len());
    }
}

#[test]
fn fuzz_pathological_patterns() {
    let mut failures = Vec::new();

    let patterns = vec![
        // Repeated operators
        ("++++++++++++++++++++", "repeated plus"),
        ("--------------------", "repeated minus"),
        ("********************", "repeated star"),
        ("////////////////////", "repeated slash"),
        ("====================", "repeated equals"),
        ("<<<<<<<<<<<<<<<<<<<<", "repeated less-than"),
        (">>>>>>>>>>>>>>>>>>>>", "repeated greater-than"),
        // Mismatched delimiters
        ("((((((((", "unmatched open parens"),
        ("))))))))", "unmatched close parens"),
        ("{{{{{{{{", "unmatched open braces"),
        ("}}}}}}}}", "unmatched close braces"),
        ("[[[[[[[[", "unmatched open brackets"),
        ("]]]]]]]]", "unmatched close brackets"),
        // Mixed delimiters
        ("({[<>]})", "mixed delimiters"),
        ("(((((]]]]]]", "crossed delimiters"),
        ("{{{{{)))))", "crossed braces/parens"),
        // Keyword spam
        ("F F F F F F F F F F", "repeated F"),
        ("S S S S S S S S S S", "repeated S"),
        ("R R R R R R R R R R", "repeated R"),
        ("if if if if if if", "repeated if"),
        // Type syntax abuse
        ("<><><><><><><>", "repeated angle brackets"),
        ("->->->->->->->", "repeated arrows"),
        ("::::::::::::", "repeated colons"),
        (",,,,,,,,,,,,", "repeated commas"),
        // Number/identifier edge cases
        ("123456789012345678901234567890", "very long number"),
        (
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "very long identifier",
        ),
        ("___________________________________", "all underscores"),
    ];

    for (input, desc) in patterns {
        if let Err(panic_msg) = parse_no_panic(input) {
            failures.push((desc.to_string(), input.to_string(), panic_msg));
        }
    }

    if !failures.is_empty() {
        eprintln!("\n=== PARSER PANICS FOUND ===");
        for (desc, input, panic_msg) in &failures {
            eprintln!("\n{}: PANIC!", desc);
            eprintln!("Input: {:?}", input);
            eprintln!("Panic message: {}", panic_msg);
        }
        panic!(
            "Parser panicked on {} pathological patterns",
            failures.len()
        );
    }
}
