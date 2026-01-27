# Vais Tutorial - Quick Start

## Installation

The tutorial is part of the Vais workspace. No additional installation needed.

## Running the Tutorial

```bash
# Interactive mode (recommended)
cargo run -p vais-tutorial --bin vais-tutorial

# Or using examples
cargo run -p vais-tutorial --example tutorial_interactive

# Demo mode (see features)
cargo run -p vais-tutorial --example tutorial_demo
```

## Essential Commands

```
chapters    - List all chapters
start 0 0   - Start Chapter 0, Lesson 0
hint        - Get a hint
solution    - Show the solution
next        - Move to next lesson
progress    - View your progress
quit        - Exit tutorial
```

## Your First Lesson

1. **Start the tutorial:**
   ```bash
   cargo run -p vais-tutorial --bin vais-tutorial
   ```

2. **Begin Chapter 1:**
   ```
   >>> start 0 0
   ```

3. **Read the lesson content** and try to solve it

4. **Get help if needed:**
   ```
   >>> hint
   ```

5. **Check your solution:**
   ```
   >>> check my_solution.vais
   ```
   Or verify inline:
   ```
   >>> verify let answer = 42;
   ```

6. **Move to the next lesson:**
   ```
   >>> next
   ```

## Learning Path

1. **Chapter 1: Basics** (3 lessons) - Variables, functions, types
2. **Chapter 2: Control Flow** (3 lessons) - If, loops, match
3. **Chapter 3: Collections** (3 lessons) - Vec, HashMap, Set
4. **Chapter 4: Error Handling** (3 lessons) - Option, Result
5. **Chapter 5: Advanced** (3 lessons) - Structs, traits, generics

**Total: 15 lessons**

## Tips

- Try solving each lesson yourself before using hints
- Progress is automatically saved
- You can quit anytime and resume later
- Experiment with different solutions

## Getting Help

```
>>> help
```

Shows all available commands and their usage.

## Testing Your Knowledge

After each chapter, try to:
1. Write code without looking at solutions
2. Explain concepts to yourself
3. Create variations of the examples

## Next Steps

After completing the tutorial:
1. Explore the Vais examples directory
2. Read the language documentation
3. Build your own projects
4. Contribute to Vais!

---

**Happy Learning!**
