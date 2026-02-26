use crate::{Chapter, Lesson, TestCase};

pub fn create_chapters() -> Vec<Chapter> {
    vec![
        create_chapter1_basics(),
        create_chapter2_control_flow(),
        create_chapter3_collections(),
        create_chapter4_error_handling(),
        create_chapter5_structs_traits(),
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
In Vais, variables are declared using the 'let' keyword:

    let x = 42;
    let name = "Vais";

Variables are immutable by default. To make them mutable, use 'mut':

    let mut count = 0;
    count = count + 1;

Type annotations are optional when the type can be inferred:

    let x: i32 = 42;
    let y = 3.14;  // inferred as f64
"#
                .to_string(),
                code_template: r#"// Create a variable named 'answer' with value 42
// Your code here
"#
                .to_string(),
                solution: r#"let answer = 42;
"#
                .to_string(),
                test_cases: vec![TestCase {
                    description: "Code should compile".to_string(),
                    expected_output: None,
                    should_compile: true,
                    validation_fn: None,
                }],
                hints: vec![
                    "Use the 'let' keyword to declare a variable".to_string(),
                    "Variable syntax: let name = value;".to_string(),
                    "The solution is: let answer = 42;".to_string(),
                ],
            },
            Lesson {
                id: "ch1_functions".to_string(),
                title: "Functions".to_string(),
                description: "Learn how to define and call functions".to_string(),
                content: r#"
Functions are declared using the 'fn' keyword:

    fn greet() {
        print("Hello, world!");
    }

Functions can take parameters and return values:

    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

The last expression in a function is automatically returned:

    fn double(x: i32) -> i32 {
        x * 2  // no semicolon = return value
    }
"#
                .to_string(),
                code_template:
                    r#"// Write a function named 'square' that takes an i32 and returns its square
// Your code here
"#
                    .to_string(),
                solution: r#"fn square(x: i32) -> i32 {
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
                    "Function syntax: fn name(params) -> return_type { body }".to_string(),
                    "Multiply x by itself".to_string(),
                    "The solution is: fn square(x: i32) -> i32 { x * x }".to_string(),
                ],
            },
            Lesson {
                id: "ch1_types".to_string(),
                title: "Basic Types".to_string(),
                description: "Understand Vais's type system".to_string(),
                content: r#"
Vais has several primitive types:

Integers: i8, i16, i32, i64, u8, u16, u32, u64
    let x: i32 = -42;
    let y: u64 = 100;

Floating point: f32, f64
    let pi: f64 = 3.14159;

Boolean: bool
    let is_active: bool = true;

String: str (string slice) and String (owned string)
    let s: str = "hello";
    let owned: String = String::from("world");

Characters: char
    let c: char = 'A';
"#
                .to_string(),
                code_template: r#"// Create variables with different types:
// - An i32 named 'age' with value 25
// - A bool named 'is_student' with value true
// - A str named 'name' with value "Alice"
// Your code here
"#
                .to_string(),
                solution: r#"let age: i32 = 25;
let is_student: bool = true;
let name: str = "Alice";
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
                    "Use type annotations: let name: type = value;".to_string(),
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
The 'if' expression allows conditional execution:

    if x > 0 {
        print("positive");
    }

You can add 'else' and 'else if' branches:

    if x > 0 {
        print("positive");
    } else if x < 0 {
        print("negative");
    } else {
        print("zero");
    }

Since 'if' is an expression, it returns a value:

    let abs = if x >= 0 { x } else { -x };
"#
                .to_string(),
                code_template:
                    r#"// Write a function 'max' that returns the larger of two i32 values
// Use an if expression
// Your code here
"#
                    .to_string(),
                solution: r#"fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
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
                    "Use if-else as an expression to return a value".to_string(),
                    "Compare a and b with the > operator".to_string(),
                ],
            },
            Lesson {
                id: "ch2_loops".to_string(),
                title: "Loops".to_string(),
                description: "Understand different loop constructs".to_string(),
                content: r#"
Vais has several loop constructs:

Infinite loop:
    loop {
        // runs forever unless broken
        break;
    }

While loop:
    while condition {
        // runs while condition is true
    }

For loop with ranges:
    for i in 0..10 {
        // i goes from 0 to 9
    }

For loop with iterators:
    for item in collection {
        // process each item
    }

Use 'break' to exit and 'continue' to skip:
    for i in 0..10 {
        if i == 5 { break; }
        if i % 2 == 0 { continue; }
    }
"#
                .to_string(),
                code_template: r#"// Write a function 'sum_range' that sums numbers from 1 to n
// Use a for loop with a range
// Your code here
"#
                .to_string(),
                solution: r#"fn sum_range(n: i32) -> i32 {
    let mut sum = 0;
    for i in 1..=n {
        sum = sum + i;
    }
    sum
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
                    "Use 1..=n for an inclusive range (1 to n)".to_string(),
                    "Accumulate the sum in a mutable variable".to_string(),
                    "Return the sum at the end".to_string(),
                ],
            },
            Lesson {
                id: "ch2_match".to_string(),
                title: "Pattern Matching".to_string(),
                description: "Master the match expression".to_string(),
                content: r#"
The 'match' expression is powerful for pattern matching:

    match value {
        1 => print("one"),
        2 => print("two"),
        _ => print("other"),
    }

Match must be exhaustive (cover all cases):

    match option {
        Some(x) => x,
        None => 0,
    }

You can match ranges and use guards:

    match age {
        0..=12 => "child",
        13..=19 => "teen",
        _ if age >= 65 => "senior",
        _ => "adult",
    }
"#
                .to_string(),
                code_template:
                    r#"// Write a function 'describe_number' that takes an i32 and returns:
// - "zero" if 0
// - "positive" if > 0
// - "negative" if < 0
// Use a match expression
// Your code here
"#
                    .to_string(),
                solution: r#"fn describe_number(n: i32) -> str {
    match n {
        0 => "zero",
        x if x > 0 => "positive",
        _ => "negative",
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
                    "Use guards with 'if' to check conditions".to_string(),
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
        description: "Work with vectors, hash maps, and sets".to_string(),
        lessons: vec![
            Lesson {
                id: "ch3_vectors".to_string(),
                title: "Vectors".to_string(),
                description: "Learn to use dynamic arrays".to_string(),
                content: r#"
Vectors are growable arrays:

Creating vectors:
    let v = Vec::new();
    let v = vec![1, 2, 3];

Adding elements:
    v.push(4);
    v.push(5);

Accessing elements:
    let first = v[0];
    let maybe = v.get(10);  // returns Option

Iterating:
    for item in v {
        print(item);
    }

Common operations:
    v.len()     // length
    v.is_empty() // check if empty
    v.pop()     // remove and return last element
"#
                .to_string(),
                code_template: r#"// Write a function 'create_range_vec' that creates a vector
// containing numbers from 1 to n
// Your code here
"#
                .to_string(),
                solution: r#"fn create_range_vec(n: i32) -> Vec<i32> {
    let mut v = Vec::new();
    for i in 1..=n {
        v.push(i);
    }
    v
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
                    "Create an empty vector with Vec::new()".to_string(),
                    "Use a for loop to add elements".to_string(),
                    "Return the vector at the end".to_string(),
                ],
            },
            Lesson {
                id: "ch3_hashmaps".to_string(),
                title: "Hash Maps".to_string(),
                description: "Store key-value pairs".to_string(),
                content: r#"
Hash maps store key-value pairs:

Creating a hash map:
    let mut scores = HashMap::new();

Inserting values:
    scores.insert("Blue", 10);
    scores.insert("Red", 50);

Accessing values:
    let score = scores.get("Blue");  // returns Option
    let score = scores["Blue"];      // panics if not found

Iterating:
    for (key, value) in scores {
        print(key, value);
    }

Updating values:
    scores.insert("Blue", 25);  // overwrites
    scores.entry("Yellow").or_insert(50);
"#
                .to_string(),
                code_template: r#"// Write a function 'word_count' that takes a vector of strings
// and returns a HashMap with word counts
// Your code here
"#
                .to_string(),
                solution: r#"fn word_count(words: Vec<str>) -> HashMap<str, i32> {
    let mut counts = HashMap::new();
    for word in words {
        let count = counts.entry(word).or_insert(0);
        *count = *count + 1;
    }
    counts
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
                    "Use HashMap::new() to create a new map".to_string(),
                    "Use entry().or_insert() to initialize counts".to_string(),
                    "Increment the count for each word".to_string(),
                ],
            },
            Lesson {
                id: "ch3_sets".to_string(),
                title: "Hash Sets".to_string(),
                description: "Work with unique collections".to_string(),
                content: r#"
Hash sets store unique values:

Creating a set:
    let mut set = HashSet::new();

Adding values:
    set.insert(1);
    set.insert(2);
    set.insert(2);  // ignored, already exists

Checking membership:
    if set.contains(&1) {
        print("found");
    }

Set operations:
    let a = hashset![1, 2, 3];
    let b = hashset![2, 3, 4];

    a.union(&b)         // {1, 2, 3, 4}
    a.intersection(&b)  // {2, 3}
    a.difference(&b)    // {1}
"#
                .to_string(),
                code_template: r#"// Write a function 'unique_elements' that takes a Vec<i32>
// and returns a HashSet with unique elements
// Your code here
"#
                .to_string(),
                solution: r#"fn unique_elements(nums: Vec<i32>) -> HashSet<i32> {
    let mut set = HashSet::new();
    for num in nums {
        set.insert(num);
    }
    set
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
                    "Create an empty HashSet".to_string(),
                    "Insert each element from the vector".to_string(),
                    "Duplicates are automatically handled".to_string(),
                ],
            },
        ],
    }
}

fn create_chapter4_error_handling() -> Chapter {
    Chapter {
        id: 3,
        title: "Chapter 4: Error Handling".to_string(),
        description: "Handle errors with Option and Result".to_string(),
        lessons: vec![
            Lesson {
                id: "ch4_option".to_string(),
                title: "Option Type".to_string(),
                description: "Handle optional values".to_string(),
                content: r#"
The Option type represents an optional value:

    enum Option<T> {
        Some(T),
        None,
    }

Creating Options:
    let some_number = Some(5);
    let no_number: Option<i32> = None;

Pattern matching:
    match some_number {
        Some(x) => print(x),
        None => print("no value"),
    }

Useful methods:
    option.is_some()           // true if Some
    option.is_none()           // true if None
    option.unwrap()            // get value or panic
    option.unwrap_or(default)  // get value or default
    option.map(|x| x * 2)      // transform if Some
"#
                .to_string(),
                code_template: r#"// Write a function 'safe_divide' that returns Option<i32>
// Return Some(a/b) if b != 0, None otherwise
// Your code here
"#
                .to_string(),
                solution: r#"fn safe_divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
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
                    "Check if b is zero first".to_string(),
                    "Return None for division by zero".to_string(),
                    "Return Some(result) for valid division".to_string(),
                ],
            },
            Lesson {
                id: "ch4_result".to_string(),
                title: "Result Type".to_string(),
                description: "Handle operations that can fail".to_string(),
                content: r#"
The Result type represents success or error:

    enum Result<T, E> {
        Ok(T),
        Err(E),
    }

Creating Results:
    let success: Result<i32, str> = Ok(10);
    let failure: Result<i32, str> = Err("error");

Pattern matching:
    match result {
        Ok(value) => print("Success:", value),
        Err(e) => print("Error:", e),
    }

The ? operator propagates errors:
    fn try_operation() -> Result<i32, str> {
        let x = might_fail()?;  // returns early if Err
        Ok(x * 2)
    }

Useful methods:
    result.is_ok()
    result.is_err()
    result.unwrap()
    result.unwrap_or(default)
"#
                .to_string(),
                code_template: r#"// Write a function 'parse_positive' that parses a string to i32
// Return Ok(number) if positive, Err("not positive") otherwise
// Assume parse() works and returns a valid i32
// Your code here
"#
                .to_string(),
                solution: r#"fn parse_positive(s: str) -> Result<i32, str> {
    let num = parse_int(s);
    if num > 0 {
        Ok(num)
    } else {
        Err("not positive")
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
                    "Parse the string first".to_string(),
                    "Check if the parsed number is positive".to_string(),
                    "Return Ok or Err based on the check".to_string(),
                ],
            },
            Lesson {
                id: "ch4_combinators".to_string(),
                title: "Error Combinators".to_string(),
                description: "Chain error handling operations".to_string(),
                content: r#"
Combinators allow chaining operations:

map - transform the success value:
    Some(5).map(|x| x * 2)  // Some(10)
    None.map(|x| x * 2)     // None

and_then (flatMap) - chain operations:
    Some(5).and_then(|x| Some(x * 2))  // Some(10)
    Some(5).and_then(|x| None)         // None

or_else - provide alternative:
    None.or_else(|| Some(0))  // Some(0)

unwrap_or_else - compute default:
    result.unwrap_or_else(|e| {
        print("Error:", e);
        0
    })
"#
                .to_string(),
                code_template: r#"// Write a function 'double_if_positive' that takes Option<i32>
// Returns Some(x*2) if x > 0, None otherwise
// Use map and filter-like logic
// Your code here
"#
                .to_string(),
                solution: r#"fn double_if_positive(opt: Option<i32>) -> Option<i32> {
    opt.and_then(|x| {
        if x > 0 {
            Some(x * 2)
        } else {
            None
        }
    })
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
                    "Use and_then to handle the Option".to_string(),
                    "Check if the value is positive inside".to_string(),
                    "Return Some or None based on the check".to_string(),
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
Structs group related data:

    struct Point {
        x: i32,
        y: i32,
    }

Creating instances:
    let p = Point { x: 10, y: 20 };

Accessing fields:
    let x_coord = p.x;

Tuple structs:
    struct Color(i32, i32, i32);
    let black = Color(0, 0, 0);

Methods are defined in impl blocks:
    impl Point {
        fn distance(&self) -> f64 {
            ((self.x * self.x + self.y * self.y) as f64).sqrt()
        }
    }
"#
                .to_string(),
                code_template: r#"// Define a struct 'Rectangle' with width and height (both i32)
// Add a method 'area' that returns i32
// Your code here
"#
                .to_string(),
                solution: r#"struct Rectangle {
    width: i32,
    height: i32,
}

impl Rectangle {
    fn area(&self) -> i32 {
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
                    "Define the struct with two fields".to_string(),
                    "Use impl Rectangle to add methods".to_string(),
                    "Multiply width and height for area".to_string(),
                ],
            },
            Lesson {
                id: "ch5_traits".to_string(),
                title: "Traits".to_string(),
                description: "Define shared behavior".to_string(),
                content: r#"
Traits define shared behavior:

    trait Printable {
        fn print(&self);
    }

Implementing traits:
    impl Printable for Point {
        fn print(&self) {
            print("Point(", self.x, ",", self.y, ")");
        }
    }

Traits can have default implementations:
    trait Describable {
        fn description(&self) -> str {
            "A describable object"
        }
    }

Trait bounds constrain generics:
    fn print_all<T: Printable>(items: Vec<T>) {
        for item in items {
            item.print();
        }
    }
"#
                .to_string(),
                code_template: r#"// Define a trait 'Shape' with a method 'area' that returns f64
// Implement it for a struct 'Circle' with radius: f64
// Your code here
"#
                .to_string(),
                solution: r#"trait Shape {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        3.14159 * self.radius * self.radius
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
                    "Define the trait with one method".to_string(),
                    "Create a Circle struct with radius field".to_string(),
                    "Calculate area using pi * r^2".to_string(),
                ],
            },
            Lesson {
                id: "ch5_generics".to_string(),
                title: "Generic Types".to_string(),
                description: "Write reusable code with generics".to_string(),
                content: r#"
Generics allow code reuse across types:

Generic functions:
    fn first<T>(list: Vec<T>) -> Option<T> {
        if list.is_empty() {
            None
        } else {
            Some(list[0])
        }
    }

Generic structs:
    struct Pair<T, U> {
        first: T,
        second: U,
    }

    let pair = Pair { first: 1, second: "hello" };

Generic implementations:
    impl<T> Pair<T, T> {
        fn new(first: T, second: T) -> Self {
            Pair { first, second }
        }
    }
"#
                .to_string(),
                code_template: r#"// Write a generic function 'swap' that takes Pair<T, U>
// and returns Pair<U, T>
// Your code here
"#
                .to_string(),
                solution: r#"fn swap<T, U>(pair: Pair<T, U>) -> Pair<U, T> {
    Pair {
        first: pair.second,
        second: pair.first,
    }
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
                    "Use type parameters <T, U>".to_string(),
                    "Swap the first and second fields".to_string(),
                    "Return a new Pair with swapped types".to_string(),
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
        assert_eq!(chapters.len(), 5);
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
        assert_eq!(total, 15);
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
            assert!(chapter.title.len() > 5, "Chapter title too short: {}", chapter.title);
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
