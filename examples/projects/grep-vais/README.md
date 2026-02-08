# grep-vais

A simplified recursive file search tool written in Vais.

## Project Status

This project demonstrates the challenges and learnings when working with Vais' type system, particularly around string handling:

### Key Learnings

1. **String Types**: Vais has two representations for strings:
   - `str` - Compile-time string literals (e.g., "hello")
   - `i64` - Runtime string pointers (from function returns, argv, etc.)

2. **Type System Limitations**:
   - Cannot freely convert between `str` and `i64` pointer types
   - Function parameters must specify exact type
   - Need separate function declarations for literal vs pointer strings

3. **Extern Function Signatures**:
   - `printf`, `puts` require careful argument counting
   - Variadic functions (`...`) not fully supported in extern declarations
   - Need separate declarations for different arities (printf1, printf2, etc.)

4. **Builtin Functions**:
   - `store_byte(ptr, val)` takes 2 args, not 3 - pointer arithmetic needed
   - `load_byte(ptr)` takes 1 arg - add offset to pointer directly
   - `load_i64(ptr)` for reading pointers from arrays (like argv)

5. **Command Line Arguments**:
   - `main(argc: i64, argv: i64)` signature required
   - Access args via `load_i64(argv + (index * 8))`
   - Returned pointers are i64, not str

6. **Ownership System**:
   - May need `--no-ownership-check` flag for complex pointer manipulations
   - String parameters can trigger use-after-move errors

## Implementation

The project includes three files:

- **main.vais** - CLI entry point with argument parsing
- **search.vais** - Core search logic (partial implementation)
- **simple_grep.vais** - Simplified single-file grep

### Current Limitations

Due to the type system challenges above, the full recursive directory search implementation is incomplete. The code demonstrates:

- ✅ File reading and line-by-line processing
- ✅ Pattern matching using `__str_contains`
- ✅ Command-line argument handling
- ✅ Result formatting and output
- ❌ Full recursive directory traversal (type system limitations)
- ❌ Binary file detection
- ❌ Subdirectory recursion (str/i64 conversion issues)

## Building

```bash
# Attempt to compile (will encounter type errors)
cargo run --bin vaisc -- examples/projects/grep-vais/main.vais -o grep-vais

# Compile without ownership checks (may have linker errors)
cargo run --bin vaisc -- examples/projects/grep-vais/main.vais -o grep-vais --no-ownership-check
```

## Lessons for Future Vais Development

This project highlights areas where the Vais language could be improved:

1. **String Interoperability**: Allow safer conversion between `str` literals and `i64` pointers
2. **Variadic Functions**: Better support for variable-argument extern functions
3. **Type Inference**: More flexible type inference for compatible pointer types
4. **Standard Library**: Built-in file I/O and string manipulation utilities
5. **Error Messages**: More specific error locations when type mismatches occur

## Alternative Approaches

For a working file search tool in Vais, consider:

1. Use `std/fs.vais` utilities if available
2. Shell out to system `grep` command via extern
3. Implement simpler single-file search (no recursion)
4. Use Vais for high-level logic, C for low-level file operations

## Code Structure

```
grep-vais/
├── main.vais           # CLI and main entry (~145 lines)
├── search.vais         # Search logic (~198 lines)
├── simple_grep.vais    # Simplified version (~110 lines)
└── README.md           # This file

Total: ~450 lines of Vais code demonstrating language features and limitations
```

## Educational Value

While not a complete working implementation, this project serves as:

- A practical example of Vais syntax and idioms
- Documentation of current language limitations
- Test case for future language improvements
- Reference for handling C FFI in Vais

## Future Work

Once Vais addresses the string type system issues, this project could be completed with:

- Full recursive directory traversal
- File pattern filtering (*.vais, *.rs, etc.)
- Case-insensitive search option
- Context lines (before/after matches)
- Colored output
- Regex support

## Contributing

If you solve the string type system challenges, please contribute your solution! Key areas:

1. Make `opendir`/`fopen` work with both `str` literals and `i64` pointers
2. Enable pointer arithmetic on `str` types for iteration
3. Implement safe str ↔ i64 conversion functions

---

**Note**: This project is primarily educational and demonstrates real-world challenges when building systems-level tools in a new programming language.
