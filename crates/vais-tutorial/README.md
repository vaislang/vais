# Vais Tutorial System

Interactive tutorial system for learning the Vais programming language, inspired by the Rust Book.

## Features

- **5 Comprehensive Chapters**: From basic syntax to advanced traits and generics
- **Interactive REPL**: Learn by doing with immediate feedback
- **Progress Tracking**: Automatic saving of completed lessons and progress
- **Hint System**: Get help when you're stuck
- **Code Validation**: Verify your solutions with instant feedback
- **Examples and Solutions**: Learn from clear examples and reference solutions

## Chapters

1. **Basic Syntax**: Variables, functions, and types
2. **Control Flow**: Conditionals, loops, and pattern matching
3. **Collections**: Vectors, hash maps, and sets
4. **Error Handling**: Option and Result types
5. **Structs and Traits**: Custom types and shared behavior

## Usage

### Run Interactive Tutorial

```bash
cargo run --example tutorial_interactive
```

### Run Demo

```bash
cargo run --example tutorial_demo
```

### Available Commands

- `help` - Show available commands
- `chapters` / `ch` - List all chapters
- `lessons` / `ls [chapter]` - List lessons in a chapter
- `start [chapter] [lesson]` - Start a specific lesson
- `next` / `n` - Move to the next lesson
- `hint` / `h` - Show a hint for the current lesson
- `solution` / `sol` - Show the solution
- `check <file>` - Check code from a file
- `verify <code>` - Verify inline code
- `progress` / `p` - Show learning progress
- `reset confirm` - Reset all progress
- `quit` / `exit` / `q` - Exit the tutorial

## Example Session

```
>>> chapters
Available Chapters:
  0. Chapter 1: Basic Syntax [0/3]
     Learn variables, functions, and basic types in Vais
  1. Chapter 2: Control Flow [0/3]
     Master conditionals, loops, and pattern matching
  ...

>>> start 0 0
═════════════════════════════════════════════════════════════
Chapter 0 - Lesson 1: Variables and Bindings
═════════════════════════════════════════════════════════════

Learn how to declare and use variables

[lesson content...]

>>> hint
Hint: Use the 'let' keyword to declare a variable

>>> check my_solution.vais
✓ All tests passed!
🎉 Lesson completed!
```

## Testing

Run tests with:

```bash
cargo test
```

## Integration

Use the tutorial system in your own projects:

```rust
use vais_tutorial::Tutorial;

let mut tutorial = Tutorial::new();
tutorial.list_chapters();

if let Some(lesson) = tutorial.get_lesson(0, 0) {
    println!("Lesson: {}", lesson.title);
}
```

## Progress File

Progress is automatically saved to `~/.vais_tutorial_progress.json`. You can specify a custom location:

```rust
let tutorial = Tutorial::with_progress_file("my_progress.json");
```
