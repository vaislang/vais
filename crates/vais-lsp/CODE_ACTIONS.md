# Code Actions Documentation

This document describes the Code Actions feature implemented in the Vais LSP server.

## Overview

Code Actions provide quick fixes and refactoring suggestions to improve code quality and fix common issues. They appear as lightbulb suggestions in LSP-compatible editors like VS Code.

## Implemented Code Actions

### 1. Quick Fix: Create Variable

**Trigger:** When you reference an undefined variable

**Action:** Creates a new variable declaration with a default value

**Example:**
```vais
F main() = puts(message)  // 'message' is undefined
```

**Suggested Fix:**
```vais
L message: i64 = 0
F main() = puts(message)
```

### 2. Quick Fix: Import Missing Module

**Trigger:** When you call an undefined function that exists in the standard library

**Action:** Adds the appropriate import statement

**Supported Functions:**
- Math functions: `sqrt`, `sin`, `cos`, `tan`, `pow`, `log`, `exp`, `floor`, `ceil`, `abs`, `abs_i64`, `min`, `max`
- I/O functions: `read_i64`, `read_f64`, `read_line`, `read_char`

**Example:**
```vais
F main() = print_i64(sqrt(16))  // 'sqrt' is undefined
```

**Suggested Fix:**
```vais
U std/math
F main() = print_i64(sqrt(16))
```

### 3. Quick Fix: Type Cast

**Trigger:** When there's a type mismatch between i64 and f64

**Action:** Adds an explicit type cast

**Example:**
```vais
F double(x: f64) -> f64 = x * 2.0
F main() = double(42)  // Type mismatch: expected f64, found i64
```

**Suggested Fix:**
```vais
F double(x: f64) -> f64 = x * 2.0
F main() = double(42 as f64)
```

### 4. Refactor: Extract to Variable

**Trigger:** When you select a single-line expression

**Action:** Extracts the selected expression into a local variable

**Example:**
Select: `1 + 2 + 3`
```vais
F main() = print_i64(1 + 2 + 3)
```

**Suggested Refactoring:**
```vais
F main() {
    L value: _ = 1 + 2 + 3
    print_i64(value)
}
```

### 5. Refactor: Extract to Function

**Trigger:** When you select multiple lines or a complex expression (> 30 characters)

**Action:** Extracts the selected code into a new function

**Example:**
Select the calculation lines:
```vais
F main() {
    L x: i64 = 10
    L y: i64 = x * 2 + 5
    print_i64(y)
}
```

**Suggested Refactoring:**
```vais
F extracted_function() -> _ {
    L x: i64 = 10
    L y: i64 = x * 2 + 5
    print_i64(y)
}

F main() {
    extracted_function()
}
```

## Usage in Editors

### VS Code

1. Place your cursor on an error or select code
2. Click the lightbulb icon or press `Cmd+.` (Mac) / `Ctrl+.` (Windows/Linux)
3. Select the desired code action from the menu

### Other Editors

Most LSP-compatible editors support code actions. Refer to your editor's documentation for the specific keybinding.

## Implementation Details

### Capability Registration

The LSP server registers `code_action_provider` capability during initialization:

```rust
code_action_provider: Some(CodeActionProviderCapability::Simple(true))
```

### Code Action Method

The `code_action` method in `LanguageServer` trait implementation:
- Receives `CodeActionParams` with range and context
- Analyzes diagnostics in the context
- Examines selected text range
- Returns `Vec<CodeActionOrCommand>`

### Action Kinds

- `CodeActionKind::QUICKFIX` - For diagnostic-based quick fixes
- `CodeActionKind::REFACTOR_EXTRACT` - For extract variable/function refactorings

## Future Enhancements

Potential improvements for future versions:

1. **Add Import from Custom Modules**
   - Suggest imports from user-defined modules
   - Auto-detect available functions in the project

2. **Implement Trait**
   - Generate stub implementations for trait methods

3. **Add Missing Match Arms**
   - Auto-generate missing pattern match cases

4. **Inline Variable/Function**
   - Reverse of extract refactorings

5. **Convert For to While**
   - Refactor between loop types

6. **Add Documentation Comments**
   - Generate documentation templates

7. **Fix Immutability Issues**
   - Suggest making variables mutable when needed

8. **Organize Imports**
   - Sort and deduplicate import statements

## Testing

Run the test suite:
```bash
cargo test -p vais-lsp code_actions
```

## Contributing

To add a new code action:

1. Identify the trigger condition (diagnostic or selection)
2. Implement the logic in `backend.rs` `code_action` method
3. Create a `CodeAction` with appropriate `title`, `kind`, and `edit`
4. Add tests in `tests/code_actions_test.rs`
5. Update this documentation
