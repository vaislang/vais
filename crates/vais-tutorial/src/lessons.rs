use crate::{Chapter, Lesson, TestCase};

pub fn create_chapters() -> Vec<Chapter> {
    vec![
        create_chapter1_basics(),
        create_chapter2_control_flow(),
        create_chapter3_collections(),
        create_chapter4_error_handling(),
        create_chapter5_structs_traits(),
        create_chapter6_closures_iterators(),
        create_chapter7_async_concurrency(),
        create_chapter8_ffi_wasm(),
    ]
}

fn create_chapter1_basics() -> Chapter {
    Chapter {
        id: 0,
        title: "Chapter 1: Basic Syntax".to_string(),
        description: "Learn variables, functions, and basic types in Vais".to_string(),
        lessons: vec![
            Lesson {
                id: "ch1_variables".to_string(),
                title: "Variables and Bindings".to_string(),
                description: "Learn how to declare and use variables".to_string(),
                content: r#"
In Vais, variables are declared using the ':=' operator:

    x := 42
    name := "Vais"

Variables are immutable by default. To make them mutable, use 'mut':

    count := mut 0
    count = count + 1

Type annotations are optional when the type can be inferred:

    x: i64 = 42
    y := 3.14  # inferred as f64
"#
                .to_string(),
                code_template: r#"# Create a variable named 'answer' with value 42
# Your code here
"#
                .to_string(),
                solution: r#"answer := 42
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Code should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use the ':=' operator to declare a variable".to_string(),
                    "Variable syntax: name := value".to_string(),
                    "The solution is: answer := 42".to_string(),
                ],
            },
            Lesson {
                id: "ch1_functions".to_string(),
                title: "Functions".to_string(),
                description: "Learn how to define and call functions".to_string(),
                content: r#"
Functions are declared using the 'F' keyword:

    F greet() {
        puts("Hello, world!")
    }

Functions can take parameters and return values:

    F add(a: i64, b: i64) -> i64 {
        a + b
    }

The last expression in a function is automatically returned:

    F double(x: i64) -> i64 {
        x * 2
    }

Single-expression functions use '=':

    F triple(x: i64) -> i64 = x * 3
"#
                .to_string(),
                code_template:
                    r#"# Write a function named 'square' that takes an i64 and returns its square
# Your code here
"#
                    .to_string(),
                solution: r#"F square(x: i64) -> i64 {
    x * x
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Function syntax: F name(params) -> return_type { body }".to_string(),
                    "Multiply x by itself".to_string(),
                    "The solution is: F square(x: i64) -> i64 { x * x }".to_string(),
                ],
            },
            Lesson {
                id: "ch1_types".to_string(),
                title: "Basic Types".to_string(),
                description: "Understand Vais's type system".to_string(),
                content: r#"
Vais has several primitive types:

Integers: i8, i16, i32, i64, u8, u16, u32, u64
    x: i64 = -42
    y: u64 = 100

Floating point: f32, f64
    pi: f64 = 3.14159

Boolean: bool
    is_active: bool = true

String: str
    s: str = "hello"

The default integer type is i64.
"#
                .to_string(),
                code_template: r#"# Create variables with different types:
# - An i64 named 'age' with value 25
# - A bool named 'is_student' with value true
# - A str named 'name' with value "Alice"
# Your code here
"#
                .to_string(),
                solution: r#"age: i64 = 25
is_student: bool = true
name: str = "Alice"
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "All variables should be declared with correct types".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Declare each variable on a separate line".to_string(),
                    "Use type annotations: name: type = value".to_string(),
                    "String literals use double quotes".to_string(),
                ],
            },
        ],
    }
}

fn create_chapter2_control_flow() -> Chapter {
    Chapter {
        id: 1,
        title: "Chapter 2: Control Flow".to_string(),
        description: "Master conditionals, loops, and pattern matching".to_string(),
        lessons: vec![
            Lesson {
                id: "ch2_if_else".to_string(),
                title: "If Expressions".to_string(),
                description: "Learn conditional branching".to_string(),
                content: r#"
In Vais, 'I' is the if keyword and 'E' is else:

    I x > 0 {
        puts("positive")
    }

You can add else and else-if branches:

    I x > 0 {
        puts("positive")
    } E I x < 0 {
        puts("negative")
    } E {
        puts("zero")
    }

You can also use the ternary operator:

    abs := x >= 0 ? x : 0 - x
"#
                .to_string(),
                code_template:
                    r#"# Write a function 'max' that returns the larger of two i64 values
# Use an I/E expression or ternary operator
# Your code here
"#
                    .to_string(),
                solution: r#"F max(a: i64, b: i64) -> i64 {
    I a > b { R a } E { R b }
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use I/E (if/else) or the ternary operator ? :".to_string(),
                    "Compare a and b with the > operator".to_string(),
                ],
            },
            Lesson {
                id: "ch2_loops".to_string(),
                title: "Loops".to_string(),
                description: "Understand different loop constructs".to_string(),
                content: r#"
Vais uses 'L' for all loop constructs:

Infinite loop:
    L {
        # runs forever unless broken
        B   # break
    }

Conditional loop (while):
    L condition {
        # runs while condition is true
    }

Range loop (for):
    L i:0..10 {
        # i goes from 0 to 9
    }

Use 'B' to break and 'C' to continue:
    L i:0..10 {
        I i == 5 { B }
        I i % 2 == 0 { C }
    }
"#
                .to_string(),
                code_template: r#"# Write a function 'sum_range' that sums numbers from 1 to n
# Use L (loop) with a range
# Your code here
"#
                .to_string(),
                solution: r#"F sum_range(n: i64) -> i64 {
    sum := mut 0
    L i:1..n {
        sum = sum + i
    }
    sum + n
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use L i:1..n for a range loop (1 to n-1)".to_string(),
                    "Accumulate the sum in a mutable variable (mut)".to_string(),
                    "Return the sum at the end".to_string(),
                ],
            },
            Lesson {
                id: "ch2_match".to_string(),
                title: "Pattern Matching".to_string(),
                description: "Master the match expression".to_string(),
                content: r#"
In Vais, 'M' is the match keyword:

    M value {
        1 => puts("one"),
        2 => puts("two"),
        _ => puts("other")
    }

Match must be exhaustive (cover all cases):

    M option {
        Some(x) => x,
        None => 0
    }

You can use guards with 'I' (if):

    M age {
        0 => "baby",
        x I x > 0 => "positive",
        _ => "negative"
    }
"#
                .to_string(),
                code_template:
                    r#"# Write a function 'describe_number' that takes an i64 and returns:
# - "zero" if 0
# - "positive" if > 0
# - "negative" if < 0
# Use M (match)
# Your code here
"#
                    .to_string(),
                solution: r#"F describe_number(n: i64) -> str {
    M n {
        0 => "zero",
        x I x > 0 => "positive",
        _ => "negative"
    }
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use guards with 'I' (if) to check conditions".to_string(),
                    "The _ pattern matches anything".to_string(),
                    "Return string literals for each case".to_string(),
                ],
            },
        ],
    }
}

fn create_chapter3_collections() -> Chapter {
    Chapter {
        id: 2,
        title: "Chapter 3: Collections".to_string(),
        description: "Work with arrays and memory in Vais".to_string(),
        lessons: vec![
            Lesson {
                id: "ch3_vectors".to_string(),
                title: "Arrays".to_string(),
                description: "Learn to use arrays and pointers".to_string(),
                content: r#"
Arrays are fixed-size collections:

Creating arrays:
    arr := [1, 2, 3, 4, 5]

Accessing elements by index:
    first := arr[0]
    third := arr[2]

Using pointer arithmetic:
    buf: *i64 = [10, 20, 30]
    val := load_byte(buf, 0)

Array with explicit size:
    F sum(arr: [i64; 5]) -> i64 {
        result := mut 0
        L i:0..5 {
            result = result + arr[i]
        }
        result
    }
"#
                .to_string(),
                code_template: r#"# Write a function 'sum_array' that takes an array of 5 i64
# and returns their sum using a loop
# Your code here
"#
                .to_string(),
                solution: r#"F sum_array(arr: [i64; 5]) -> i64 {
    result := mut 0
    L i:0..5 {
        result = result + arr[i]
    }
    result
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use [i64; 5] for an array of 5 elements".to_string(),
                    "Use L i:0..5 to loop over indices".to_string(),
                    "Return the accumulated result".to_string(),
                ],
            },
            Lesson {
                id: "ch3_hashmaps".to_string(),
                title: "HashMap".to_string(),
                description: "Store key-value pairs".to_string(),
                content: r#"
Vais supports HashMap<K,V> for key-value storage:

Creating a hash map:
    scores := HashMap<str, i64>::new()

Inserting values:
    scores.insert("Blue", 10)
    scores.insert("Red", 50)

Accessing values:
    score := scores.get("Blue")

Common operations:
    scores.len()       # number of entries
    scores.contains_key("Blue")

Note: HashMap requires 'U std/hashmap' import in
real Vais programs.
"#
                .to_string(),
                code_template: r#"# Write a function that creates a HashMap and inserts
# two key-value pairs, then returns the map
# Your code here
"#
                .to_string(),
                solution: r#"F create_scores() -> HashMap<str, i64> {
    scores := HashMap<str, i64>::new()
    scores.insert("Alice", 95)
    scores.insert("Bob", 87)
    scores
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use HashMap<str, i64>::new() to create a map".to_string(),
                    "Use .insert(key, value) to add entries".to_string(),
                    "Return the map at the end".to_string(),
                ],
            },
            Lesson {
                id: "ch3_sets".to_string(),
                title: "Memory and Pointers".to_string(),
                description: "Work with manual memory management".to_string(),
                content: r#"
Vais provides low-level memory operations:

Allocating memory:
    N "C" { F malloc(size: i64) -> i64 }
    buf := malloc(64)

Storing and loading bytes:
    store_byte(buf, 0, 65)   # store 'A' at offset 0
    val := load_byte(buf, 0) # read byte at offset 0

Freeing memory:
    N "C" { F free(ptr: i64) -> i64 }
    free(buf)

String interpolation with ~{}:
    x := 42
    puts("Value: ~{x}")
"#
                .to_string(),
                code_template: r#"# Write a function that allocates a buffer, stores 3 bytes,
# reads them back, and frees the buffer
# Return the sum of the 3 bytes
# Your code here
"#
                .to_string(),
                solution: r#"F store_and_read() -> i64 {
    N "C" { F malloc(size: i64) -> i64 }
    N "C" { F free(ptr: i64) -> i64 }
    buf := malloc(8)
    store_byte(buf, 0, 10)
    store_byte(buf, 1, 20)
    store_byte(buf, 2, 30)
    a := load_byte(buf, 0)
    b := load_byte(buf, 1)
    c := load_byte(buf, 2)
    free(buf)
    a + b + c
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use malloc to allocate memory".to_string(),
                    "Use store_byte and load_byte for byte operations".to_string(),
                    "Always free allocated memory".to_string(),
                ],
            },
        ],
    }
}

fn create_chapter4_error_handling() -> Chapter {
    Chapter {
        id: 3,
        title: "Chapter 4: Error Handling".to_string(),
        description: "Handle errors with Option, Result, and the ? operator".to_string(),
        lessons: vec![
            Lesson {
                id: "ch4_option".to_string(),
                title: "Option Type".to_string(),
                description: "Handle optional values".to_string(),
                content: r#"
The Option type represents an optional value:

    E Option<T> {
        Some(T),
        None
    }

Creating Options:
    some_number := Some(5)
    no_number: Option<i64> = None

Pattern matching with M:
    M some_number {
        Some(x) => puts("~{x}"),
        None => puts("no value")
    }

The ! operator unwraps an Option (returns value or panics):
    val := some_number!   # unwraps to 5
"#
                .to_string(),
                code_template: r#"# Write a function 'safe_divide' that returns i64
# Return a/b if b != 0, -1 otherwise (error sentinel)
# Your code here
"#
                .to_string(),
                solution: r#"F safe_divide(a: i64, b: i64) -> i64 {
    I b == 0 {
        R -1
    }
    R a / b
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Check if b is zero first with I (if)".to_string(),
                    "Return -1 for division by zero".to_string(),
                    "Return a / b for valid division".to_string(),
                ],
            },
            Lesson {
                id: "ch4_result".to_string(),
                title: "Result Type".to_string(),
                description: "Handle operations that can fail".to_string(),
                content: r#"
The Result type represents success or error:

    E Result<T, E> {
        Ok(T),
        Err(E)
    }

Creating Results:
    success: Result<i64, str> = Ok(10)
    failure: Result<i64, str> = Err("error")

Pattern matching:
    M result {
        Ok(value) => puts("Success: ~{value}"),
        Err(e) => puts("Error")
    }

The ? operator propagates errors:
    F try_operation() -> Result<i64, str> {
        x := might_fail()?   # returns early if Err
        Ok(x * 2)
    }

The ! operator unwraps (panics on Err):
    val := result!
"#
                .to_string(),
                code_template: r#"# Write a function 'check_positive' that takes i64
# Return the number if positive, or -1 if not
# Your code here
"#
                .to_string(),
                solution: r#"F check_positive(n: i64) -> i64 {
    I n > 0 {
        R n
    } E {
        R -1
    }
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use I (if) to check if n is positive".to_string(),
                    "Return n for positive numbers".to_string(),
                    "Return -1 for non-positive numbers".to_string(),
                ],
            },
            Lesson {
                id: "ch4_combinators".to_string(),
                title: "Closures and Pipe Operator".to_string(),
                description: "Chain operations with closures and pipes".to_string(),
                content: r#"
Closures (lambdas) use |params| syntax:

    double := |x| x * 2
    add := |a, b| a + b

The pipe operator |> chains function calls:

    result := 5 |> |x| x * 2 |> |x| x + 1
    # result = 11

Multi-step pipeline:
    F process(x: i64) -> i64 {
        x |> |n| n * 3 |> |n| n + 10
    }

Closures can capture outer variables:
    multiplier := 3
    scale := |x| x * multiplier
"#
                .to_string(),
                code_template: r#"# Write a function that uses closures and |> pipe
# to double a number, then add 10
# Your code here
"#
                .to_string(),
                solution: r#"F double_and_add(x: i64) -> i64 {
    x |> |n| n * 2 |> |n| n + 10
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use |> to pipe the value through closures".to_string(),
                    "First closure: |n| n * 2 (double)".to_string(),
                    "Second closure: |n| n + 10 (add 10)".to_string(),
                ],
            },
        ],
    }
}

fn create_chapter5_structs_traits() -> Chapter {
    Chapter {
        id: 4,
        title: "Chapter 5: Structs and Traits".to_string(),
        description: "Define custom types and behaviors".to_string(),
        lessons: vec![
            Lesson {
                id: "ch5_structs".to_string(),
                title: "Structures".to_string(),
                description: "Create custom data types".to_string(),
                content: r#"
Structs group related data using 'S':

    S Point {
        x: i64
        y: i64
    }

Creating instances:
    p := Point { x: 10, y: 20 }

Accessing fields:
    x_coord := p.x

Methods are defined in 'X' (impl) blocks:
    X Point {
        F sum(&self) -> i64 {
            self.x + self.y
        }
    }

Method call:
    result := p.sum()
"#
                .to_string(),
                code_template: r#"# Define a struct 'Rectangle' with width and height (both i64)
# Add a method 'area' that returns i64
# Your code here
"#
                .to_string(),
                solution: r#"S Rectangle {
    width: i64
    height: i64
}

X Rectangle {
    F area(&self) -> i64 {
        self.width * self.height
    }
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Struct and method should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use S to define the struct with two fields".to_string(),
                    "Use X Rectangle to add methods".to_string(),
                    "Multiply width and height for area".to_string(),
                ],
            },
            Lesson {
                id: "ch5_traits".to_string(),
                title: "Traits".to_string(),
                description: "Define shared behavior".to_string(),
                content: r#"
Traits define shared behavior using 'W':

    W Printable {
        F show(&self) -> i64
    }

Implementing traits with 'X ... : Trait':
    X Point: Printable {
        F show(&self) -> i64 {
            puts("Point: ~{self.x}, ~{self.y}")
            0
        }
    }

Trait bounds constrain generics:
    F print_item<T: Printable>(item: T) -> i64 {
        item.show()
    }
"#
                .to_string(),
                code_template: r#"# Define a trait 'Shape' with a method 'area' that returns i64
# Implement it for a struct 'Circle' with radius: i64
# Your code here
"#
                .to_string(),
                solution: r#"W Shape {
    F area(&self) -> i64
}

S Circle {
    radius: i64
}

X Circle: Shape {
    F area(&self) -> i64 {
        self.radius * self.radius * 3
    }
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Trait and implementation should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use W to define the trait".to_string(),
                    "Use S to create the Circle struct".to_string(),
                    "Use X Circle: Shape to implement the trait".to_string(),
                ],
            },
            Lesson {
                id: "ch5_generics".to_string(),
                title: "Generic Types".to_string(),
                description: "Write reusable code with generics".to_string(),
                content: r#"
Generics allow code reuse across types:

Generic functions:
    F identity<T>(x: T) -> T = x

    F max<T>(a: T, b: T) -> T {
        I a > b { R a } E { R b }
    }

Generic structs:
    S Pair<T, U> {
        first: T
        second: U
    }

    pair := Pair<i64, str> { first: 1, second: "hello" }

Generic implementations:
    X Pair<T, U> {
        F get_first(&self) -> T = self.first
    }

Self-recursion with generics:
    F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)
"#
                .to_string(),
                code_template: r#"# Write a generic function 'first_or_second' that takes
# two values of the same type and a boolean flag,
# returns the first if flag is true, second otherwise
# Your code here
"#
                .to_string(),
                solution: r#"F first_or_second<T>(a: T, b: T, flag: bool) -> T {
    I flag { R a } E { R b }
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Generic function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use type parameter <T> for both arguments".to_string(),
                    "Use I/E (if/else) to select which value to return".to_string(),
                    "Return a or b based on the flag".to_string(),
                ],
            },
        ],
    }
}

fn create_chapter6_closures_iterators() -> Chapter {
    Chapter {
        id: 5,
        title: "Chapter 6: Closures and Iterators".to_string(),
        description: "Master closures, higher-order functions, and iterator patterns".to_string(),
        lessons: vec![
            Lesson {
                id: "ch6_closures".to_string(),
                title: "Closures In Depth".to_string(),
                description: "Learn closure syntax, capture modes, and higher-order functions"
                    .to_string(),
                content: r#"
Closures capture variables from their enclosing scope:

    x := 10
    add_x := |n| n + x    # captures x by value

Multi-line closures:
    transform := |a, b| {
        sum := a + b
        sum * 2
    }

Passing closures to functions:
    F apply(f: |i64| -> i64, val: i64) -> i64 {
        f(val)
    }

    result := apply(|x| x * 3, 5)  # result = 15

Returning closures:
    F make_adder(n: i64) -> |i64| -> i64 {
        |x| x + n
    }
    add5 := make_adder(5)
    result := add5(10)  # result = 15
"#
                .to_string(),
                code_template: r#"# Write a function 'make_multiplier' that takes an i64 and
# returns a closure that multiplies its argument by that value.
# Then use it: make_multiplier(3)(7) should return 21
# Your code here
"#
                .to_string(),
                solution: r#"F make_multiplier(factor: i64) -> |i64| -> i64 {
    |x| x * factor
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Closure factory should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Return a closure that captures the factor parameter".to_string(),
                    "Closure syntax: |param| expression".to_string(),
                    "The closure captures 'factor' from the outer function".to_string(),
                ],
            },
            Lesson {
                id: "ch6_higher_order".to_string(),
                title: "Higher-Order Functions".to_string(),
                description: "Use functions as arguments and return values".to_string(),
                content: r#"
Higher-order functions take or return other functions:

    F twice(f: |i64| -> i64, x: i64) -> i64 {
        f(f(x))
    }

    result := twice(|x| x + 1, 5)  # result = 7

Composing functions:
    F compose(f: |i64| -> i64, g: |i64| -> i64) -> |i64| -> i64 {
        |x| f(g(x))
    }

    double := |x| x * 2
    inc := |x| x + 1
    double_then_inc := compose(inc, double)
    result := double_then_inc(5)  # 5*2=10, 10+1=11

The pipe operator chains transformations:
    result := 5 |> |x| x * 2 |> |x| x + 1  # 11
"#
                .to_string(),
                code_template:
                    r#"# Write a function 'apply_twice' that takes a closure and a value,
# applies the closure twice, and returns the result.
# Example: apply_twice(|x| x * 2, 3) -> 12
# Your code here
"#
                    .to_string(),
                solution: r#"F apply_twice(f: |i64| -> i64, x: i64) -> i64 {
    f(f(x))
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Higher-order function should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Call f once on x, then call f again on the result".to_string(),
                    "Function parameter syntax: f: |ParamType| -> ReturnType".to_string(),
                    "Solution: f(f(x))".to_string(),
                ],
            },
            Lesson {
                id: "ch6_iterators".to_string(),
                title: "Iterator Patterns".to_string(),
                description: "Build iterator-like patterns with closures and loops".to_string(),
                content: r#"
Vais uses loop + closure patterns for iteration:

Summing with accumulation:
    F fold(arr: [i64; 5], init: i64, f: |i64, i64| -> i64) -> i64 {
        acc := mut init
        L i:0..5 {
            acc = f(acc, arr[i])
        }
        acc
    }

    nums := [1, 2, 3, 4, 5]
    sum := fold(nums, 0, |acc, x| acc + x)  # 15

Mapping over arrays:
    F map5(arr: [i64; 5], f: |i64| -> i64) -> [i64; 5] {
        result := mut [0, 0, 0, 0, 0]
        L i:0..5 {
            result[i] = f(arr[i])
        }
        result
    }

Filtering with predicates:
    F count_if(arr: [i64; 5], pred: |i64| -> bool) -> i64 {
        count := mut 0
        L i:0..5 {
            I pred(arr[i]) { count = count + 1 }
        }
        count
    }
"#
                .to_string(),
                code_template: r#"# Write a function 'sum_mapped' that takes an array of 5 i64,
# a mapping closure, and returns the sum of applying the closure
# to each element.
# Example: sum_mapped([1,2,3,4,5], |x| x * x) -> 55
# Your code here
"#
                .to_string(),
                solution: r#"F sum_mapped(arr: [i64; 5], f: |i64| -> i64) -> i64 {
    total := mut 0
    L i:0..5 {
        total = total + f(arr[i])
    }
    total
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Iterator pattern should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Loop over the array indices with L i:0..5".to_string(),
                    "Apply f to each element and accumulate the result".to_string(),
                    "Use a mutable accumulator: total := mut 0".to_string(),
                ],
            },
        ],
    }
}

fn create_chapter7_async_concurrency() -> Chapter {
    Chapter {
        id: 6,
        title: "Chapter 7: Async and Concurrency".to_string(),
        description: "Write asynchronous and concurrent programs in Vais".to_string(),
        lessons: vec![
            Lesson {
                id: "ch7_async_basics".to_string(),
                title: "Async Functions".to_string(),
                description: "Learn async/await syntax and the Future type".to_string(),
                content: r#"
Async functions use 'A' (async) and 'Y' (await):

    A F fetch_data() -> i64 {
        # Simulated async work
        result := Y compute_value()
        result * 2
    }

    A F compute_value() -> i64 {
        42
    }

Async functions return a Future:
    future := fetch_data()   # doesn't run yet
    value := Y future        # runs and waits for result

Key rules:
- 'A' marks a function as async
- 'Y' awaits a Future inside an async function
- Async functions can only be awaited inside other async functions
"#
                .to_string(),
                code_template: r#"# Write an async function 'async_add' that takes two i64 values,
# and returns their sum. Then write an async function 'double_add'
# that calls async_add and doubles the result.
# Your code here
"#
                .to_string(),
                solution: r#"A F async_add(a: i64, b: i64) -> i64 {
    a + b
}

A F double_add(a: i64, b: i64) -> i64 {
    sum := Y async_add(a, b)
    sum * 2
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Async functions should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use 'A F' to declare an async function".to_string(),
                    "Use 'Y' to await the result of an async call".to_string(),
                    "Async functions can call other async functions with Y".to_string(),
                ],
            },
            Lesson {
                id: "ch7_channels".to_string(),
                title: "Communication Patterns".to_string(),
                description: "Coordinate async tasks and cleanup".to_string(),
                content: r#"
Async tasks can communicate through return values and shared state:

Using return values (recommended):
    A F fan_out(inputs: [i64; 3]) -> i64 {
        r0 := Y process(inputs[0])
        r1 := Y process(inputs[1])
        r2 := Y process(inputs[2])
        r0 + r1 + r2
    }

Defer for cleanup (like Go's defer):
    F with_resource() -> i64 {
        res := acquire()
        D release(res)    # runs when function exits
        use_resource(res)
    }

Yield for cooperative multitasking:
    A F generator() -> i64 {
        yield 1
        yield 2
        yield 3
        0
    }

Common patterns:
- Fan-out: await N tasks in sequence, combine results
- Pipeline: chain async operations with |>
- Defer: ensure cleanup runs on exit
"#
                .to_string(),
                code_template: r#"# Write a function 'with_cleanup' that:
# 1. Creates a resource (just use an i64 value)
# 2. Uses D (defer) to ensure cleanup
# 3. Returns the resource value multiplied by 2
# Your code here
"#
                .to_string(),
                solution: r#"F acquire() -> i64 { 42 }
F release(r: i64) -> i64 { r }

F with_cleanup() -> i64 {
    res := acquire()
    D release(res)
    res * 2
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Defer pattern should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "D (defer) schedules a call to run when the function exits".to_string(),
                    "Place the defer right after acquiring the resource".to_string(),
                    "The deferred call runs regardless of how the function exits".to_string(),
                ],
            },
        ],
    }
}

fn create_chapter8_ffi_wasm() -> Chapter {
    Chapter {
        id: 7,
        title: "Chapter 8: FFI and WebAssembly".to_string(),
        description: "Interface with C libraries and compile to WebAssembly".to_string(),
        lessons: vec![
            Lesson {
                id: "ch8_ffi_basics".to_string(),
                title: "Foreign Function Interface".to_string(),
                description: "Call C functions from Vais using extern blocks".to_string(),
                content: r#"
The 'N' (extern) keyword declares foreign functions:

Calling C standard library:
    N "C" {
        F malloc(size: i64) -> i64
        F free(ptr: i64) -> i64
        F printf(fmt: str) -> i64
    }

Using extern functions:
    F allocate_buffer(size: i64) -> i64 {
        buf := malloc(size)
        I buf == 0 {
            R -1   # allocation failed
        }
        buf
    }

Calling math functions:
    N "C" {
        F abs(x: i64) -> i64
        F sqrt(x: f64) -> f64
    }

    F distance(x: f64, y: f64) -> f64 {
        sqrt(x * x + y * y)
    }

FFI guidelines:
- Always check return values from C functions
- Free allocated memory when done
- Use D (defer) for automatic cleanup
"#
                .to_string(),
                code_template: r#"# Write a function that uses extern malloc/free to:
# 1. Allocate 16 bytes
# 2. Store a value at offset 0
# 3. Read the value back
# 4. Free the memory
# 5. Return the read value
# Your code here
"#
                .to_string(),
                solution: r#"N "C" {
    F malloc(size: i64) -> i64
    F free(ptr: i64) -> i64
}

F alloc_and_read() -> i64 {
    buf := malloc(16)
    store_byte(buf, 0, 99)
    val := load_byte(buf, 0)
    free(buf)
    val
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "FFI code should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Declare extern functions with N \"C\" { ... }".to_string(),
                    "Use store_byte and load_byte for memory access".to_string(),
                    "Always free allocated memory".to_string(),
                ],
            },
            Lesson {
                id: "ch8_wasm_basics".to_string(),
                title: "WebAssembly Basics".to_string(),
                description: "Compile Vais code to WebAssembly".to_string(),
                content: r#"
Vais can compile to WebAssembly (WASM):

    vaisc --target wasm32-unknown-unknown myfile.vais

Exporting functions to WASM:
    #[wasm_export("add")]
    F add(a: i64, b: i64) -> i64 {
        a + b
    }

Importing from the WASM host:
    #[wasm_import("env", "log")]
    N "C" { F log(value: i64) -> i64 }

WASM constraints:
- No direct filesystem access
- No raw pointers to host memory
- Integer and float types only at the boundary
- Use i32 for WASM-native integers

A minimal WASM module:
    #[wasm_export("main")]
    F main() -> i64 {
        result := compute(10, 20)
        result
    }

    F compute(a: i64, b: i64) -> i64 {
        a * b + a + b
    }
"#
                .to_string(),
                code_template: r#"# Write a WASM-exportable function 'fibonacci' that computes
# the nth Fibonacci number. Use #[wasm_export("fibonacci")].
# Your code here
"#
                .to_string(),
                solution: r#"#[wasm_export("fibonacci")]
F fibonacci(n: i64) -> i64 {
    I n < 2 { R n }
    a := mut 0
    b := mut 1
    L i:2..n {
        temp := b
        b = a + b
        a = temp
    }
    a + b
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "WASM export should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use #[wasm_export(\"name\")] attribute before the function".to_string(),
                    "Use an iterative approach for Fibonacci (not recursive)".to_string(),
                    "Track two variables (a, b) and update in a loop".to_string(),
                ],
            },
            Lesson {
                id: "ch8_wasm_js_interop".to_string(),
                title: "WASM-JavaScript Interop".to_string(),
                description: "Bridge Vais WASM modules with JavaScript".to_string(),
                content: r#"
Vais can also compile to JavaScript ESM:

    vaisc --target js myfile.vais  # produces myfile.mjs

JavaScript codegen produces clean ESM modules:
    # Vais source:
    F add(a: i64, b: i64) -> i64 = a + b

    // Generated JavaScript (myfile.mjs):
    // export function add(a, b) { return a + b; }

WASM + JS bridging pattern:
    # math.vais
    #[wasm_export("multiply")]
    F multiply(a: i64, b: i64) -> i64 = a * b

    #[wasm_export("power")]
    F power(base: i64, exp: i64) -> i64 {
        result := mut 1
        L i:0..exp {
            result = result * base
        }
        result
    }

    // JavaScript usage:
    // const wasm = await WebAssembly.instantiate(buffer);
    // console.log(wasm.instance.exports.multiply(3, 4)); // 12
    // console.log(wasm.instance.exports.power(2, 10));   // 1024

Import functions from JS host:
    #[wasm_import("env", "console_log")]
    N "C" { F console_log(val: i64) -> i64 }
"#
                .to_string(),
                code_template: r#"# Write two WASM-exportable functions:
# 1. 'square' that returns x * x
# 2. 'sum_of_squares' that returns the sum of squares from 1 to n
# Your code here
"#
                .to_string(),
                solution: r#"#[wasm_export("square")]
F square(x: i64) -> i64 = x * x

#[wasm_export("sum_of_squares")]
F sum_of_squares(n: i64) -> i64 {
    total := mut 0
    L i:1..n {
        total = total + i * i
    }
    total + n * n
}
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "WASM interop functions should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use #[wasm_export(\"name\")] for each exported function".to_string(),
                    "Single-expression functions can use = syntax".to_string(),
                    "Loop from 1 to n and accumulate squares".to_string(),
                ],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_chapters_created() {
        let chapters = create_chapters();
        assert_eq!(chapters.len(), 8);
    }

    #[test]
    fn test_chapter1_structure() {
        let chapter = create_chapter1_basics();
        assert_eq!(chapter.id, 0);
        assert_eq!(chapter.lessons.len(), 3);
        assert!(!chapter.lessons[0].hints.is_empty());
    }

    #[test]
    fn test_all_lessons_have_content() {
        let chapters = create_chapters();
        for chapter in chapters {
            for lesson in chapter.lessons {
                assert!(!lesson.title.is_empty());
                assert!(!lesson.content.is_empty());
                assert!(!lesson.solution.is_empty());
                assert!(!lesson.test_cases.is_empty());
            }
        }
    }

    #[test]
    fn test_lesson_ids_unique() {
        let chapters = create_chapters();
        let mut ids = std::collections::HashSet::new();

        for chapter in chapters {
            for lesson in chapter.lessons {
                assert!(
                    ids.insert(lesson.id.clone()),
                    "Duplicate lesson ID: {}",
                    lesson.id
                );
            }
        }
    }

    #[test]
    fn test_chapter_ids_sequential() {
        let chapters = create_chapters();
        for (i, chapter) in chapters.iter().enumerate() {
            assert_eq!(chapter.id, i);
        }
    }

    #[test]
    fn test_all_lessons_have_hints() {
        let chapters = create_chapters();
        for chapter in chapters {
            for lesson in chapter.lessons {
                assert!(
                    !lesson.hints.is_empty(),
                    "Lesson {} has no hints",
                    lesson.id
                );
            }
        }
    }

    #[test]
    fn test_all_lessons_have_test_cases() {
        let chapters = create_chapters();
        for chapter in chapters {
            for lesson in chapter.lessons {
                assert!(
                    !lesson.test_cases.is_empty(),
                    "Lesson {} has no test cases",
                    lesson.id
                );
            }
        }
    }

    #[test]
    fn test_chapter1_lesson_count() {
        let chapter = create_chapter1_basics();
        assert_eq!(chapter.lessons.len(), 3);
        assert_eq!(chapter.lessons[0].id, "ch1_variables");
        assert_eq!(chapter.lessons[1].id, "ch1_functions");
        assert_eq!(chapter.lessons[2].id, "ch1_types");
    }

    #[test]
    fn test_chapter2_lesson_count() {
        let chapter = create_chapter2_control_flow();
        assert_eq!(chapter.lessons.len(), 3);
        assert_eq!(chapter.lessons[0].id, "ch2_if_else");
        assert_eq!(chapter.lessons[1].id, "ch2_loops");
        assert_eq!(chapter.lessons[2].id, "ch2_match");
    }

    #[test]
    fn test_chapter3_lesson_count() {
        let chapter = create_chapter3_collections();
        assert_eq!(chapter.lessons.len(), 3);
    }

    #[test]
    fn test_chapter4_lesson_count() {
        let chapter = create_chapter4_error_handling();
        assert_eq!(chapter.lessons.len(), 3);
    }

    #[test]
    fn test_chapter5_lesson_count() {
        let chapter = create_chapter5_structs_traits();
        assert_eq!(chapter.lessons.len(), 3);
    }

    #[test]
    fn test_total_lessons() {
        let chapters = create_chapters();
        let total: usize = chapters.iter().map(|c| c.lessons.len()).sum();
        assert_eq!(total, 23);
    }

    #[test]
    fn test_all_chapters_have_descriptions() {
        let chapters = create_chapters();
        for chapter in chapters {
            assert!(!chapter.title.is_empty());
            assert!(!chapter.description.is_empty());
        }
    }

    #[test]
    fn test_lesson_code_templates_not_empty() {
        let chapters = create_chapters();
        for chapter in chapters {
            for lesson in chapter.lessons {
                assert!(
                    !lesson.code_template.is_empty(),
                    "Lesson {} has empty code template",
                    lesson.id
                );
            }
        }
    }

    #[test]
    fn test_lesson_descriptions_not_empty() {
        let chapters = create_chapters();
        for chapter in chapters {
            for lesson in chapter.lessons {
                assert!(
                    !lesson.description.is_empty(),
                    "Lesson {} has empty description",
                    lesson.id
                );
            }
        }
    }

    #[test]
    fn test_test_cases_should_compile() {
        let chapters = create_chapters();
        for chapter in chapters {
            for lesson in chapter.lessons {
                // All test cases in the tutorial should have should_compile = true
                for tc in &lesson.test_cases {
                    assert!(
                        tc.should_compile,
                        "Lesson {} has test case that shouldn't compile",
                        lesson.id
                    );
                }
            }
        }
    }

    #[test]
    fn test_all_solutions_not_empty() {
        let chapters = create_chapters();
        for chapter in chapters {
            for lesson in chapter.lessons {
                assert!(
                    !lesson.solution.is_empty(),
                    "Lesson {} has empty solution",
                    lesson.id
                );
            }
        }
    }

    #[test]
    fn test_chapter_ids_start_at_zero() {
        let chapters = create_chapters();
        assert_eq!(chapters[0].id, 0);
    }

    #[test]
    fn test_lesson_ids_have_chapter_prefix() {
        let chapters = create_chapters();
        for (i, chapter) in chapters.iter().enumerate() {
            let prefix = format!("ch{}_", i + 1);
            for lesson in &chapter.lessons {
                assert!(
                    lesson.id.starts_with(&prefix),
                    "Lesson {} should start with prefix {}",
                    lesson.id,
                    prefix
                );
            }
        }
    }

    #[test]
    fn test_chapter_titles_non_trivial() {
        let chapters = create_chapters();
        for chapter in &chapters {
            assert!(
                chapter.title.len() > 5,
                "Chapter title too short: {}",
                chapter.title
            );
        }
    }

    #[test]
    fn test_chapter3_topics() {
        let chapter = create_chapter3_collections();
        let ids: Vec<&str> = chapter.lessons.iter().map(|l| l.id.as_str()).collect();
        assert!(ids.contains(&"ch3_vectors"));
    }

    #[test]
    fn test_chapter4_topics() {
        let chapter = create_chapter4_error_handling();
        let ids: Vec<&str> = chapter.lessons.iter().map(|l| l.id.as_str()).collect();
        assert!(ids.contains(&"ch4_result"));
    }

    #[test]
    fn test_chapter5_topics() {
        let chapter = create_chapter5_structs_traits();
        let ids: Vec<&str> = chapter.lessons.iter().map(|l| l.id.as_str()).collect();
        assert!(ids.contains(&"ch5_structs"));
    }

    #[test]
    fn test_chapter6_lesson_count() {
        let chapter = create_chapter6_closures_iterators();
        assert_eq!(chapter.lessons.len(), 3);
        assert_eq!(chapter.id, 5);
        assert_eq!(chapter.lessons[0].id, "ch6_closures");
        assert_eq!(chapter.lessons[1].id, "ch6_higher_order");
        assert_eq!(chapter.lessons[2].id, "ch6_iterators");
    }

    #[test]
    fn test_chapter6_topics() {
        let chapter = create_chapter6_closures_iterators();
        let ids: Vec<&str> = chapter.lessons.iter().map(|l| l.id.as_str()).collect();
        assert!(ids.contains(&"ch6_closures"));
        assert!(ids.contains(&"ch6_higher_order"));
        assert!(ids.contains(&"ch6_iterators"));
    }

    #[test]
    fn test_chapter7_lesson_count() {
        let chapter = create_chapter7_async_concurrency();
        assert_eq!(chapter.lessons.len(), 2);
        assert_eq!(chapter.id, 6);
        assert_eq!(chapter.lessons[0].id, "ch7_async_basics");
        assert_eq!(chapter.lessons[1].id, "ch7_channels");
    }

    #[test]
    fn test_chapter7_topics() {
        let chapter = create_chapter7_async_concurrency();
        let ids: Vec<&str> = chapter.lessons.iter().map(|l| l.id.as_str()).collect();
        assert!(ids.contains(&"ch7_async_basics"));
        assert!(ids.contains(&"ch7_channels"));
    }

    #[test]
    fn test_chapter8_lesson_count() {
        let chapter = create_chapter8_ffi_wasm();
        assert_eq!(chapter.lessons.len(), 3);
        assert_eq!(chapter.id, 7);
        assert_eq!(chapter.lessons[0].id, "ch8_ffi_basics");
        assert_eq!(chapter.lessons[1].id, "ch8_wasm_basics");
        assert_eq!(chapter.lessons[2].id, "ch8_wasm_js_interop");
    }

    #[test]
    fn test_chapter8_topics() {
        let chapter = create_chapter8_ffi_wasm();
        let ids: Vec<&str> = chapter.lessons.iter().map(|l| l.id.as_str()).collect();
        assert!(ids.contains(&"ch8_ffi_basics"));
        assert!(ids.contains(&"ch8_wasm_basics"));
        assert!(ids.contains(&"ch8_wasm_js_interop"));
    }

    #[test]
    fn test_lesson_hints_are_strings() {
        let chapters = create_chapters();
        for chapter in &chapters {
            for lesson in &chapter.lessons {
                for hint in &lesson.hints {
                    assert!(hint.len() > 0, "Hint in {} is empty", lesson.id);
                }
            }
        }
    }

    #[test]
    fn test_chapters_have_increasing_ids() {
        let chapters = create_chapters();
        for i in 1..chapters.len() {
            assert!(chapters[i].id > chapters[i - 1].id);
        }
    }
}
