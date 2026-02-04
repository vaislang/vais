# Vais TODO - Technical Documentation

## Architecture Overview

### Module Breakdown

```
┌─────────────────────────────────────────────┐
│              main.vais                      │
│  (CLI argument parsing & command routing)   │
└───────────────┬─────────────────────────────┘
                │
        ┌───────┴──────┬──────────┬───────────┐
        │              │          │           │
        ▼              ▼          ▼           ▼
┌──────────────┐ ┌─────────┐ ┌──────────┐ ┌──────────┐
│  todo.vais   │ │storage  │ │ display  │ │  libc    │
│  (data model)│ │ (JSON)  │ │ (colors) │ │  (FFI)   │
└──────────────┘ └─────────┘ └──────────┘ └──────────┘
```

## Data Structures

### Todo Item

```vais
S Todo {
    id: i64,           # Unique identifier (1, 2, 3, ...)
    description: str,   # Task description (max ~512 chars)
    done: i64,         # Status: 0 = pending, 1 = completed
    created_at: i64    # Unix timestamp (seconds since epoch)
}
```

**Memory Layout**: 32 bytes (approx)
- `id`: 8 bytes
- `description`: 8 bytes (pointer)
- `done`: 8 bytes
- `created_at`: 8 bytes

### TodoList Container

```vais
S TodoList {
    items_ptr: i64,    # Pointer to dynamically allocated array
    count: i64,        # Current number of items
    next_id: i64       # Next ID to assign (monotonic counter)
}
```

**Memory Layout**: 24 bytes
- Initial capacity: 10 items (320 bytes allocated)
- Growth strategy: Simple fixed allocation (no dynamic resize in v1.0)

## Core Functions

### todo.vais - Business Logic

#### `todo_new(id: i64, desc: str) -> Todo`
Creates a new Todo instance with current timestamp.

```vais
F todo_new(id: i64, desc: str) -> Todo {
    C timestamp := time(0)
    R Todo {
        id: id,
        description: desc,
        done: 0,
        created_at: timestamp
    }
}
```

**Time Complexity**: O(1)

#### `todolist_add(list: TodoList, desc: str) -> TodoList`
Adds a new item to the list and increments next_id.

**Time Complexity**: O(1)
**Space Complexity**: O(1) - no reallocation in v1.0

#### `todolist_done(list: TodoList, id: i64) -> i64`
Marks an item as completed by ID.

**Time Complexity**: O(n) - linear search
**Return**: 0 on success, -1 if not found

#### `todolist_remove(list: TodoList, id: i64) -> i64`
Removes an item and shifts remaining items.

**Time Complexity**: O(n) - linear scan + shift
**Return**: 0 on success, -1 if not found

### storage.vais - Persistence Layer

#### JSON Format

```json
{
  "version": "1.0",
  "todos": [
    {
      "id": 1,
      "description": "Task description",
      "done": 0,
      "created_at": 1704067200
    }
  ]
}
```

#### `storage_save(list: TodoList, path: str) -> i64`
Serializes TodoList to JSON file.

**Algorithm**:
1. Open file for writing (truncate existing)
2. Write JSON header
3. Iterate through items, writing each as JSON object
4. Close file

**Time Complexity**: O(n)
**I/O Operations**: 3n + 4 writes (header + items + footer)

#### `storage_load(path: str) -> TodoList`
Deserializes JSON file to TodoList.

**Algorithm**:
1. Try to open file
2. If file doesn't exist, return empty list
3. Parse JSON (simplified line-by-line in v1.0)
4. Reconstruct TodoList

**Time Complexity**: O(n)
**Error Handling**: Returns empty list on file not found

### display.vais - User Interface

#### Color System

```vais
C COLOR_RESET  := "\x1b[0m"
C COLOR_GREEN  := "\x1b[32m"   # Completed tasks
C COLOR_YELLOW := "\x1b[33m"   # Pending tasks
C COLOR_RED    := "\x1b[31m"   # Errors
C COLOR_BLUE   := "\x1b[34m"   # Headers
C COLOR_GRAY   := "\x1b[90m"   # Metadata
C COLOR_BOLD   := "\x1b[1m"    # Emphasis
```

#### `display_todo(todo: Todo) -> i64`
Renders a single TODO item with color coding.

**Output Format**:
```
[✓] 1. Completed task    # Green
[ ] 2. Pending task      # Yellow
```

#### `display_list(list: TodoList) -> i64`
Renders full list with summary statistics.

**Output Format**:
```
TODO List:

[✓] 1. Task one
[ ] 2. Task two

─────────────────────────────
Total: 2  |  Pending: 1  |  Done: 1
```

**Time Complexity**: O(n) - iterate through all items

### main.vais - Command Router

#### Command Dispatch Table

| Command | Args | Function | Effect |
|---------|------|----------|--------|
| `help` | 0 | `display_help()` | Show usage |
| `list` | 0-1 | `display_list()` | Show all/filtered |
| `add` | 1+ | `todolist_add()` | Create new item |
| `done` | 1 | `todolist_done()` | Mark complete |
| `remove` | 1 | `todolist_remove()` | Delete item |

#### `execute_command(cmd: str, argc: i64, argv: str, list: TodoList, path: str) -> i64`
Main command dispatcher using string comparison.

**Algorithm**:
```
1. Compare cmd with each known command (strcmp)
2. Validate argc (argument count)
3. Call appropriate function
4. Save state if modified
5. Return exit code (0 = success, 1 = error)
```

**Time Complexity**: O(1) - constant number of string comparisons

## Memory Management

### Allocation Strategy

```vais
# Initial allocation
F todolist_new() -> TodoList {
    C initial_capacity := 10
    C size := initial_capacity * 32
    C ptr := malloc(size)
    R TodoList { items_ptr: ptr, count: 0, next_id: 1 }
}

# Cleanup
F todolist_free(list: TodoList) -> i64 {
    I list.items_ptr != 0 {
        free(list.items_ptr)
    }
    R 0
}
```

### Memory Lifecycle

```
main() entry
    ↓
storage_load() → malloc() for list
    ↓
[operations: add/done/remove]
    ↓
storage_save() → write to disk
    ↓
todolist_free() → free() memory
    ↓
exit
```

### Memory Footprint

- Empty list: 24 bytes (struct) + 320 bytes (buffer) = 344 bytes
- Per TODO: 32 bytes (struct) + description string
- Average 50 TODOs: 344 + 50 × (32 + 50) = ~4KB

## Error Handling

### Error Code Convention

```vais
# Success: 0
# Failure: -1
# Exit codes: 0 (success), 1 (error)
```

### Error Scenarios

| Scenario | Detection | Response |
|----------|-----------|----------|
| File not found | `fopen() == 0` | Create new empty list |
| Invalid ID | Linear search fails | Return -1, show error |
| Missing args | `argc < expected` | Show usage, exit 1 |
| Write failure | `fputs() < 0` | Return -1, show error |
| Malloc failure | `malloc() == 0` | (Not handled in v1.0) |

## Performance Characteristics

### Time Complexity Summary

| Operation | Best | Average | Worst |
|-----------|------|---------|-------|
| Add | O(1) | O(1) | O(1) |
| Done | O(1) | O(n) | O(n) |
| Remove | O(1) | O(n) | O(n) |
| List | O(n) | O(n) | O(n) |
| Save | O(n) | O(n) | O(n) |
| Load | O(n) | O(n) | O(n) |

### Space Complexity

- Storage: O(n) - linear in number of items
- Runtime: O(1) - fixed overhead for operations

### Scalability Limits

- Max items: Limited by initial allocation (10 in v1.0)
- Max description: ~512 characters (no enforced limit)
- File size: Unbounded (JSON grows linearly)

## FFI (Foreign Function Interface)

### C Standard Library Bindings

#### I/O Functions
```vais
X F fopen(path: str, mode: str) -> i64
X F fclose(file: i64) -> i64
X F fputs(s: str, file: i64) -> i64
X F fgets(buf: str, size: i64, file: i64) -> str
X F fprintf(file: i64, fmt: str) -> i64
X F fscanf(file: i64, fmt: str) -> i64
```

#### String Functions
```vais
X F strcmp(a: str, b: str) -> i64
X F strlen(s: str) -> i64
X F strcpy(dest: str, src: str) -> str
X F strcat(dest: str, src: str) -> str
X F atoi(s: str) -> i64
```

#### Memory Functions
```vais
X F malloc(size: i64) -> i64
X F free(ptr: i64) -> i64
```

#### System Functions
```vais
X F time(ptr: i64) -> i64
X F getenv(name: str) -> str
X F exit(code: i64) -> i64
```

#### Output Functions
```vais
X F puts(s: str) -> i64
X F printf(fmt: str) -> i64
X F sprintf(buf: str, fmt: str) -> i64
```

### ABI Compatibility

- Calling convention: C ABI (System V on Unix, MS on Windows)
- String passing: Null-terminated char* pointers
- Return values: Direct register return (i64)
- No exception handling: Return codes only

## Compilation

### LLVM IR Generation

```
Vais Source → AST → Type Check → LLVM IR → Object File → Executable
```

### Optimization Flags

```bash
# Debug build
vaisc src/main.vais -o vais-todo

# Optimized build (future)
vaisc src/main.vais -o vais-todo -O3

# With debug symbols
vaisc src/main.vais -o vais-todo -g
```

### Link-Time Dependencies

```
vais-todo executable
├── libc.so.6 (C standard library)
├── libm.so.6 (Math library)
└── ld-linux.so.2 (Dynamic linker)
```

## Testing Strategy

### Unit Tests (Future)

```vais
# Test todo creation
F test_todo_new() -> i64 {
    C todo := todo_new(1, "Test")
    R todo.id == 1 && todo.done == 0 ? 0 : -1
}

# Test list operations
F test_todolist_add() -> i64 {
    C list := todolist_new()
    C list := todolist_add(list, "Test")
    R list.count == 1 ? 0 : -1
}
```

### Integration Tests

```bash
#!/bin/bash
# Test CLI functionality

# Test add
./vais-todo add "Test task"
[ $? -eq 0 ] || exit 1

# Test list
output=$(./vais-todo list)
echo "$output" | grep -q "Test task" || exit 1

# Test done
./vais-todo done 1
[ $? -eq 0 ] || exit 1

# Test remove
./vais-todo remove 1
[ $? -eq 0 ] || exit 1

echo "All tests passed!"
```

### Performance Benchmarks

```bash
# Benchmark add operations
time for i in {1..1000}; do
    ./vais-todo add "Task $i"
done

# Benchmark list rendering
time ./vais-todo list

# Benchmark file I/O
time ./vais-todo list > /dev/null
```

## Future Enhancements

### v1.1 - Dynamic Growth
- Implement list resizing when capacity exceeded
- Use realloc() for efficient memory management

### v1.2 - Better Parsing
- Proper JSON parser (not line-by-line)
- Handle escaped characters in descriptions

### v1.3 - Search
- Linear search by substring
- Filter by date range

### v1.4 - Sorting
- Sort by ID, date, or completion status
- Configurable sort order

### v2.0 - Advanced Features
- Due dates and priorities
- Categories and tags
- Full-text search
- Undo/redo capability

## Debug Guide

### Common Issues

#### Segmentation Fault
```bash
# Run with debugger
gdb ./vais-todo
(gdb) run list
(gdb) backtrace
```

#### Memory Leaks
```bash
# Check with valgrind
valgrind --leak-check=full ./vais-todo list
```

#### File Corruption
```bash
# Validate JSON
python -m json.tool ~/.vais-todo.json

# Reset if corrupted
rm ~/.vais-todo.json
./vais-todo list
```

## References

- Vais Language Specification: `docs/language-spec.md`
- LLVM IR Reference: https://llvm.org/docs/LangRef.html
- C Standard Library: https://en.cppreference.com/w/c
- ANSI Escape Codes: https://en.wikipedia.org/wiki/ANSI_escape_code
