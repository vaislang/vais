# Vais TODO - Terminal TODO Manager

A lightweight, fast, and colorful terminal TODO management tool built with the Vais programming language.

## Features

- **Add TODOs**: Quickly add new tasks with descriptions
- **Mark as Done**: Complete tasks with a simple command
- **Remove Items**: Delete tasks you no longer need
- **Colored Output**: Visual distinction between pending (yellow) and completed (green) tasks
- **Persistent Storage**: All data saved to `~/.vais-todo.json`
- **Filtering**: View all, only pending, or only completed tasks
- **Summary Stats**: See your progress at a glance

## Installation

### From Source

```bash
# Navigate to the project directory
cd /Users/sswoo/study/projects/vais/projects/vais-todo

# Compile the project (assuming Vais compiler is available)
vaisc src/main.vais -o vais-todo

# Optionally, move to a directory in your PATH
sudo mv vais-todo /usr/local/bin/
```

## Usage

### Basic Commands

```bash
# Show help
vais-todo help

# Add a new TODO item
vais-todo add "Buy groceries"
vais-todo add "Write documentation"
vais-todo add "Review pull requests"

# List all TODO items
vais-todo list

# List only pending items
vais-todo list --pending

# List only completed items
vais-todo list --done

# Mark item as done (by ID)
vais-todo done 1

# Remove an item (by ID)
vais-todo remove 2
```

### Example Session

```bash
$ vais-todo add "Learn Vais programming"
✓ TODO item added!

$ vais-todo add "Build a CLI tool"
✓ TODO item added!

$ vais-todo list
TODO List:

[ ] 1. Learn Vais programming
[ ] 2. Build a CLI tool

─────────────────────────────
Total: 2  |  Pending: 2  |  Done: 0

$ vais-todo done 1
✓ TODO item marked as done!

$ vais-todo list
TODO List:

[✓] 1. Learn Vais programming
[ ] 2. Build a CLI tool

─────────────────────────────
Total: 2  |  Pending: 1  |  Done: 1

$ vais-todo list --done
TODO List:

[✓] 1. Learn Vais programming
```

## Architecture

### Project Structure

```
vais-todo/
├── vais.toml           # Package manifest
├── README.md           # This file
└── src/
    ├── main.vais       # CLI entry point and command routing
    ├── todo.vais       # TODO data model and business logic
    ├── storage.vais    # JSON file persistence
    └── display.vais    # Colored terminal output
```

### Data Model

```vais
S Todo {
    id: i64,           # Unique identifier
    description: str,   # Task description
    done: i64,         # 0 = pending, 1 = done
    created_at: i64    # Unix timestamp
}

S TodoList {
    items_ptr: i64,    # Pointer to Todo array
    count: i64,        # Number of items
    next_id: i64       # Next ID to assign
}
```

### Storage Format

TODO items are stored as JSON in `~/.vais-todo.json`:

```json
{
  "version": "1.0",
  "todos": [
    {
      "id": 1,
      "description": "Learn Vais programming",
      "done": 1,
      "created_at": 1704067200
    },
    {
      "id": 2,
      "description": "Build a CLI tool",
      "done": 0,
      "created_at": 1704070800
    }
  ]
}
```

## Implementation Details

### Extern Functions

The tool uses standard C library functions through Vais's FFI:

- **I/O**: `fopen`, `fclose`, `fputs`, `fgets`
- **String**: `strcmp`, `strlen`, `strcpy`, `strcat`, `atoi`
- **Memory**: `malloc`, `free`
- **System**: `time`, `getenv`, `exit`
- **Output**: `puts`, `printf`, `sprintf`

### Color Support

ANSI escape codes provide colorful terminal output:

- **Green**: Completed tasks
- **Yellow**: Pending tasks
- **Red**: Error messages
- **Blue**: Section headers
- **Gray**: Metadata and dividers

### Error Handling

All functions return status codes:
- `0`: Success
- `-1`: Error (not found, file I/O failure, etc.)

## Development

### Building from Source

```bash
# Type check
cargo check

# Build the Vais compiler
cargo build --release

# Compile the TODO app
./target/release/vaisc projects/vais-todo/src/main.vais -o vais-todo
```

### Testing

```bash
# Test basic functionality
./vais-todo help
./vais-todo add "Test item"
./vais-todo list
./vais-todo done 1
./vais-todo remove 1
```

## Roadmap

Future enhancements:

- [ ] Due dates and reminders
- [ ] Priority levels (high, medium, low)
- [ ] Categories/tags
- [ ] Search and filtering
- [ ] Multiple lists/projects
- [ ] Sync with cloud storage
- [ ] Export to various formats (Markdown, CSV)
- [ ] Undo/redo functionality
- [ ] Recurring tasks

## License

MIT License - See package manifest for details.

## Contributing

This is a demonstration project for the Vais programming language. Contributions are welcome!

## About Vais

Vais is an AI-optimized systems programming language featuring:

- Single-character keywords for compact code
- Full type inference
- LLVM backend for native performance
- Rust-based compiler infrastructure

Learn more at the main Vais repository.
