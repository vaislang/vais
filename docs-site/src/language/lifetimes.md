# Lifetimes & Borrow Checking

## Overview
Vais implements Non-Lexical Lifetimes (NLL) with control-flow-graph (CFG) based dataflow analysis to ensure memory safety without garbage collection. The borrow checker prevents common memory errors at compile time.

## Borrow Rules

Vais enforces strict borrowing rules:

1. **Multiple shared references** — Any number of `&T` (immutable borrows) allowed simultaneously
2. **Single mutable reference** — Only one `&mut T` (mutable borrow) at a time
3. **Exclusivity** — Cannot have `&T` and `&mut T` simultaneously on the same value

### Valid Borrowing
```vais
x := 10
a := &x         # &i64 — immutable borrow
b := &x         # &i64 — OK: multiple immutable borrows
val := *a + *b  # OK
```

### Invalid Borrowing
```vais
x := mut 10
m := &mut x      # &mut i64 — mutable borrow
# n := &x        # ERROR: cannot borrow as immutable while mutably borrowed
*m = 20          # OK
```

## Borrow Checker Errors

The Vais borrow checker detects six categories of memory safety violations:

| Code | Error | Description |
|------|-------|-------------|
| E100 | UseAfterMove | Using a value after it has been moved |
| E101 | DoubleFree | Attempting to free memory twice |
| E102 | UseAfterFree | Using memory after it has been freed |
| E103 | MutableBorrowConflict | Multiple mutable borrows of the same value |
| E104 | BorrowWhileMutablyBorrowed | Shared borrow while mutable borrow exists |
| E105 | MoveWhileBorrowed | Moving a value while it's borrowed |
| E106 | LifetimeViolation | Reference outlives the data it points to |

### Example: E100 UseAfterMove
```vais
F take_ownership(x: Vec<i64>) {
    # x is consumed here
}

v := Vec::<i64>::new()
take_ownership(v)
# v.push(1)        # ERROR E100: v was moved
```

### Example: E103 MutableBorrowConflict
```vais
x := mut [1, 2, 3]
a := &mut x
# b := &mut x      # ERROR E103: cannot borrow as mutable more than once
*a = [4, 5, 6]
```

### Example: E106 LifetimeViolation
```vais
F dangling_ref() -> &i64 {
    x := 42
    &x               # ERROR E106: returns reference to local variable
}
```

## Non-Lexical Lifetimes (NLL)

Vais implements NLL, meaning borrow lifetimes are determined by actual usage, not lexical scope:

### Before NLL (lexical scopes)
```vais
x := mut [1, 2, 3]
{
    r := &x          # Borrow starts
    print_i64(r[0])
}                     # Borrow ends at scope exit

x.push(4)            # OK: borrow ended
```

### With NLL (usage-based)
```vais
x := mut [1, 2, 3]
r := &x              # Borrow starts
print_i64(r[0])      # Last use of r — borrow expires here

x.push(4)            # OK: r is no longer used
```

The borrow expires after `r`'s last use, not at the end of the scope.

## Two-Phase Borrows

Vais supports two-phase mutable borrows for method chaining and mutation:

```vais
v := mut Vec::<i64>::new()
v.push(1)
v.push(2)            # OK: each mutable borrow is reserved then activated
```

The borrow checker uses `BorrowKind::ReservedMutable` internally to allow this pattern.

## Strict Borrow Checking

Enable strict borrow checking with the `--strict-borrow` flag:

```bash
vaisc --strict-borrow file.vais
```

This enforces stricter rules:
- All potential moves must be explicit
- Unused borrows trigger warnings
- Conservative dataflow joins (if one branch moves, value is considered moved)

## Lifetime Annotations

For complex cases where the compiler cannot infer lifetimes, use explicit lifetime annotations:

### Basic Syntax
```vais
F longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    I x.len() > y.len() { x }
    E { y }
}
```

The `'a` annotation declares that:
- Both `x` and `y` must live at least as long as `'a`
- The returned reference is valid for lifetime `'a`

### Multiple Lifetimes
```vais
F first<'a, 'b>(x: &'a i64, y: &'b i64) -> &'a i64 {
    x    # Return value tied to x's lifetime
}
```

### Struct Lifetimes
```vais
S Ref<'a> {
    data: &'a i64
}

F create_ref<'a>(x: &'a i64) -> Ref<'a> {
    Ref { data: x }
}
```

## Lifetime Elision

Vais automatically infers lifetimes in simple cases (lifetime elision rules):

### Rule 1: Single input reference
```vais
# Explicit:  F foo<'a>(x: &'a i64) -> &'a i64
F foo(x: &i64) -> &i64 {    # Inferred: output lifetime = input lifetime
    x
}
```

### Rule 2: Multiple inputs with self
```vais
X MyStruct {
    # Explicit:  F get<'a>(&'a self) -> &'a i64
    F get(&self) -> &i64 {    # Inferred: output lifetime = self lifetime
        &self.field
    }
}
```

### Rule 3: No inference possible
```vais
# ERROR: cannot infer lifetime
# F choose(x: &i64, y: &i64) -> &i64 { ... }

# Must annotate explicitly:
F choose<'a>(x: &'a i64, y: &'a i64) -> &'a i64 { ... }
```

## Control Flow Analysis

The borrow checker uses CFG-based dataflow analysis:

1. **Build CFG** — construct control flow graph from MIR
2. **Worklist algorithm** — iteratively propagate borrow state
3. **Join states** — conservatively merge at control flow joins (if-else, loops)
4. **Fixpoint** — iterate until no state changes

### Join Rules
At control flow joins (e.g., after if-else):
- If a value is `Moved` or `Dropped` in any branch, it's considered `Moved`
- Borrows are expired if they don't appear in all branches
- Conservative approach prevents use-after-move

```vais
x := mut [1, 2, 3]
I condition {
    take(x)         # x moved in this branch
} E {
    print(x[0])     # x borrowed here
}
# x is considered moved here (conservative join)
```

## MIR Representation

Lifetimes are tracked in the Mid-level Intermediate Representation (MIR):

```rust
// MirType variants
RefLifetime(String, Box<MirType>)      // &'a T
RefMutLifetime(String, Box<MirType>)   // &'a mut T

// LocalDecl
struct LocalDecl {
    lifetime: Option<String>,  // e.g., Some("'a")
    // ...
}

// Body
struct Body {
    lifetime_params: Vec<String>,            // ['a, 'b, ...]
    lifetime_bounds: Vec<(String, String)>,  // [('a, 'b), ...] means 'a: 'b
    // ...
}
```

## Best Practices

### 1. Prefer immutable borrows
```vais
F read_only(data: &[i64]) { ... }    # Good: immutable borrow
```

### 2. Keep mutable borrows short
```vais
{
    m := &mut x
    *m = 10
}    # Borrow ends
x.other_operation()
```

### 3. Use lifetime elision
Let the compiler infer lifetimes when possible:
```vais
F process(&self, data: &Vec<i64>) -> &i64 {
    &data[0]    # Lifetime inferred
}
```

### 4. Split borrows
Borrow different parts of a struct independently:
```vais
S Pair { first: i64, second: i64 }

p := mut Pair { first: 1, second: 2 }
a := &mut p.first
b := &mut p.second    # OK: non-overlapping borrows
```

## Implementation Details

- **CFG Construction** — MIR blocks form nodes, terminators form edges
- **Worklist** — VecDeque-based iterative dataflow
- **Liveness Analysis** — tracks last use of each local variable
- **Expiration** — borrows expire when their target's last use is reached
- **Max Iterations** — 1000 iterations to detect infinite loops in fixpoint

## See Also

- [Slice Types](./slices.md)
- [Memory Management](../api/memory.md)
- [MIR Design](../compiler/architecture.md)
- [Type Inference](./type-inference.md)
