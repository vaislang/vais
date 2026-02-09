# Type Inference Safety

Vais features a powerful type inference system that reduces boilerplate while maintaining type safety. Since Phase 61, the compiler enforces stricter rules to prevent ambiguous type inference scenarios.

## Inference Rules

### Automatic Type Inference

When type information can be unambiguously determined from context, Vais infers types automatically:

```vais
F add(a: i64, b: i64) -> i64 {
    a + b
}

F main() -> i64 {
    x := 5          # i64 inferred from literal
    y := add(x, 3)  # i64 inferred from function signature
    0
}
```

### Unconstrained Type Error (E032)

When the compiler cannot infer types from context, it raises an `InferFailed` error (E032):

```vais
# ERROR: Cannot infer parameter types
F add(a, b) {
    a + b
}
# Error E032: Type inference failed: unconstrained type parameters
```

This prevents the compiler from making arbitrary assumptions about types, which could lead to runtime errors or unexpected behavior.

**Fix:** Provide explicit type annotations:

```vais
F add(a: i64, b: i64) -> i64 {
    a + b
}
```

## Recursive Functions

### Return Type Required

Functions that use self-recursion (via `@`) **must** specify an explicit return type:

```vais
# ERROR: Missing return type in recursive function
F fib(n: i64) {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}
# Error E032: Recursive function requires explicit return type annotation
```

**Why?** The compiler needs the return type to properly type-check recursive calls before the function body is fully analyzed.

**Fix:** Add the return type annotation:

```vais
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}
```

### Self-Recursion Operator

The `@` operator invokes the current function recursively:

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { 1 }
    E { n * @(n - 1) }
}
```

## Struct Type Inference

Struct field types are inferred from the struct definition:

```vais
S Point { x: f64, y: f64 }

F make_point() -> Point {
    Point { x: 3.0, y: 4.0 }  # Field types inferred from struct definition
}
```

## Generic Type Inference

Generic type parameters are inferred from usage:

```vais
F identity<T>(x: T) -> T {
    x
}

F main() -> i64 {
    a := identity(42)      # T = i64 inferred
    b := identity("hello") # T = str inferred
    0
}
```

## Error Messages

The compiler provides detailed error messages when type inference fails:

```
Error E032: Type inference failed: unconstrained type parameters

  ┌─ example.vais:1:1
  │
1 │ F add(a, b) {
  │ ^^^^^^^^^^^^^ Cannot infer types for parameters 'a' and 'b'
  │
  = help: Add explicit type annotations: F add(a: i64, b: i64) -> i64
```

## Best Practices

1. **Annotate public APIs** — Always provide explicit types for function signatures in public APIs
2. **Use inference for locals** — Let the compiler infer types for local variables
3. **Annotate recursive functions** — Always specify return types for recursive functions
4. **Document complex types** — Use type aliases for complex generic types

```vais
# Good: Clear public API
F process(input: str, count: i64) -> Result<Vec<str>, Error> {
    # Local variables use inference
    lines := input.split("\n")
    results := Vec::new()
    # ...
}
```

## See Also

- [Generics](./generics.md) — Generic type parameters
- [Iterator Type Inference](./iterator-type-inference.md) — Specialized inference for iterators
- [Language Specification](./language-spec.md) — Complete language reference
