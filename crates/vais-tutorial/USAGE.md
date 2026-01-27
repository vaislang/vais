# Vais Tutorial Usage Guide

## Quick Start

### Run Interactive Tutorial

```bash
# Using cargo run
cargo run -p vais-tutorial --bin vais-tutorial

# Or using the example
cargo run -p vais-tutorial --example tutorial_interactive
```

### Run Demo

```bash
cargo run -p vais-tutorial --example tutorial_demo
```

## Tutorial Structure

### 5 Comprehensive Chapters

1. **Chapter 1: Basic Syntax**
   - Variables and Bindings
   - Functions
   - Basic Types

2. **Chapter 2: Control Flow**
   - If Expressions
   - Loops
   - Pattern Matching

3. **Chapter 3: Collections**
   - Vectors
   - Hash Maps
   - Hash Sets

4. **Chapter 4: Error Handling**
   - Option Type
   - Result Type
   - Error Combinators

5. **Chapter 5: Structs and Traits**
   - Structures
   - Traits
   - Generic Types

## Interactive Commands

### Navigation

- `chapters` or `ch` - List all chapters
- `lessons` or `ls [chapter]` - List lessons in a chapter
- `start [chapter] [lesson]` - Start a specific lesson
- `next` or `n` - Move to the next lesson

### Learning Assistance

- `hint` or `h` - Get a hint for the current lesson
- `solution` or `sol` - Show the complete solution

### Code Verification

- `check <file>` - Verify code from a file
- `verify <code>` - Verify inline code

### Progress Management

- `progress` or `p` - Show your learning progress
- `reset confirm` - Reset all progress

### Utility

- `help` - Show all available commands
- `quit`, `exit`, or `q` - Exit the tutorial

## Example Session

```
>>> chapters
Available Chapters:
  0. Chapter 1: Basic Syntax [0/3]
     Learn variables, functions, and basic types in Vais
  ...

>>> start 0 0
═══════════════════════════════════════════════════════════
Chapter 0 - Lesson 1: Variables and Bindings
═══════════════════════════════════════════════════════════

Learn how to declare and use variables

In Vais, variables are declared using the 'let' keyword:

    let x = 42;
    let name = "Vais";
...

>>> hint
Hint: Use the 'let' keyword to declare a variable

>>> solution
Solution:
──────────────────────────────────────────────────────────
let answer = 42;
──────────────────────────────────────────────────────────

>>> check my_solution.vais
✓ All tests passed!
Tests: 1/1
🎉 Lesson completed!

>>> next
```

## Writing Solutions

### Save to File

Create a file (e.g., `my_solution.vais`):

```vais
let answer = 42;
```

Then check it:

```
>>> check my_solution.vais
```

### Inline Verification

```
>>> verify let answer = 42;
```

## Progress Tracking

Progress is automatically saved to `~/.vais_tutorial_progress.json`. This includes:

- Completed lessons
- Current chapter and lesson position
- Number of hints used per lesson

## Programmatic Usage

### Using Tutorial API

```rust
use vais_tutorial::Tutorial;

// Create a new tutorial
let mut tutorial = Tutorial::new();

// List chapters
tutorial.list_chapters();

// Get a lesson
if let Some(lesson) = tutorial.get_lesson(0, 0) {
    println!("Lesson: {}", lesson.title);

    // Validate code
    let result = tutorial.validate_code(&lesson.solution, lesson);
    println!("Valid: {}", result.is_ok());
}

// Track progress
tutorial.mark_lesson_complete("ch1_variables");
tutorial.save_progress().unwrap();
```

### Custom Progress File

```rust
use vais_tutorial::Tutorial;

let tutorial = Tutorial::with_progress_file("my_progress.json");
```

## Testing

```bash
# Run all tests
cargo test -p vais-tutorial

# Run specific test suite
cargo test -p vais-tutorial --test integration_tests
cargo test -p vais-tutorial --test lesson_validation_tests

# Run unit tests only
cargo test -p vais-tutorial --lib
```

## Tips

1. **Use Hints Wisely**: Try to solve each lesson on your own first. Use hints only when stuck.

2. **Understand, Don't Memorize**: Focus on understanding the concepts rather than memorizing syntax.

3. **Practice**: After completing a lesson, try variations of the solution to deepen your understanding.

4. **Progress at Your Pace**: The tutorial saves your progress, so you can stop and resume anytime.

5. **Experiment**: The tutorial validates your code, so feel free to experiment with different solutions.

## Troubleshooting

### Tutorial Won't Start

```bash
# Check if the crate builds
cargo build -p vais-tutorial
```

### Progress Not Saving

Check permissions for the home directory:
```bash
ls -la ~/.vais_tutorial_progress.json
```

### Validation Errors

If code validation fails unexpectedly:
1. Check for syntax errors
2. Compare with the solution
3. Try the next lesson and come back

## Advanced Features

### Custom Lessons

You can extend the tutorial by creating custom lessons:

```rust
use vais_tutorial::{Lesson, TestCase};

let custom_lesson = Lesson {
    id: "custom_lesson".to_string(),
    title: "My Custom Lesson".to_string(),
    description: "Learn something new".to_string(),
    content: "Lesson content here...".to_string(),
    code_template: "// Your code here\n".to_string(),
    solution: "let x = 42;\n".to_string(),
    test_cases: vec![
        TestCase {
            description: "Code should compile".to_string(),
            expected_output: None,
            should_compile: true,
            validation_fn: None,
        }
    ],
    hints: vec![
        "Hint 1".to_string(),
        "Hint 2".to_string(),
    ],
};
```

## Integration with IDE

The tutorial can be integrated into your development workflow:

1. Run the tutorial in a separate terminal
2. Edit solutions in your IDE
3. Use `check <file>` to validate from the tutorial REPL

## Contributing

To add new lessons or improve existing ones:

1. Edit `src/lessons.rs`
2. Add lessons to the appropriate chapter
3. Run tests to validate: `cargo test -p vais-tutorial`
4. Submit a pull request

## Resources

- Vais Language Documentation: See main README
- Tutorial Source: `crates/vais-tutorial/`
- Examples: `crates/vais-tutorial/examples/`
