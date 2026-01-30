# Migration Guide

## v0.1.0 → v0.2.0

### Breaking Changes
None. v0.2.0 is fully backwards-compatible with v0.1.0.

### New Features Available
- **Generic functions now fully work at runtime** - Previously only type-checked, now monomorphized to concrete implementations
- **Trait dynamic dispatch** - `&dyn Trait` parameters now work with vtable-based dispatch
- **print/println** - Use `print("x = {}", x)` instead of manual FFI calls
- **String operations** - `+` for concatenation, comparison operators, `.len()`, `.contains()`, etc.
- **Array mutation** - `arr[i] = val` now works
- **format()** - `format("hello {}", name)` returns formatted strings

### Recommended Updates
1. Replace manual printf FFI calls with built-in `print()`/`println()`
2. Use string methods instead of manual C string operations
3. Use generic functions for type-safe polymorphism
4. Use `&dyn Trait` for runtime polymorphism where needed
