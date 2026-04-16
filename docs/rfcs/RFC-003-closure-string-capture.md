# RFC-003: Closure String Capture Ownership

- **Status**: Draft
- **Author**: Phase 191 #4 (Opus direct)
- **Area**: `crates/vais-codegen/src/expr_helpers_misc.rs`
- **Requires**: RFC-001 §4 (function-scope ownership model)

## 1. Problem

When a closure captures a heap-owned `str` (e.g., a concat result) by
value, it copies the `{ i8*, i64 }` fat pointer. The raw `i8*` still
points to the buffer tracked by the outer scope's `alloc_tracker`. When
the outer scope exits, the buffer is freed, but the closure holds a
stale pointer → use-after-free.

```vais
F make_greeter() -> i64 {
    greeting := "hello-" + "world"  # heap-owned
    f := |name: str| println(greeting)  # captures greeting by value
    # greeting's buffer is freed at function exit
    # f now holds a dangling pointer
    f("test")  # UAF
    0
}
```

## 2. Design: Clone-on-Capture

When a ByValue capture identifies a variable as heap-owned str (present
in `string_value_slot` or `var_string_slot`), emit `strdup` to clone
the buffer. The closure receives an independent copy. The original
stays tracked by the outer scope.

### Why not transfer?
Transfer (move semantics) would prevent the outer scope from using the
variable after the closure is created. Vais doesn't have Rust's borrow
checker to enforce this. Clone is safer and matches Vais's simple
ownership model.

### Why not reference?
ByRef capture passes a pointer to the outer variable, which suffers
the same lifetime issue when the outer scope exits.

## 3. Implementation

In `expr_helpers_misc.rs:generate_lambda_expr`, at the ByValue capture
path (lines 223-243), when `cap_ty` is `Str`:

1. Load the fat pointer from the local
2. Extract the i8* pointer component
3. Null-check: if null (literal), pass as-is
4. If non-null: call `__vais_str_clone` (or inline strdup + length copy)
5. Build a new fat pointer with the cloned buffer
6. Pass the new fat pointer as the captured value

The cloned buffer ownership transfers to the lambda function. The
lambda's own alloc_tracker handles cleanup at its return.

## 4. Test Plan

1. `closure_str_capture_basic`: Closure captures concat result, called
   after outer scope would have freed → correct output
2. `closure_str_capture_multiple_calls`: Closure called twice, both
   produce correct output
3. `closure_str_literal_no_clone`: Closure captures literal str → no
   unnecessary allocation

## 5. Risks

- Performance: clone-on-capture adds a malloc+memcpy per heap-owned
  str capture. This is the conservative choice; a future borrow checker
  could enable zero-copy transfer.
- The cloned buffer in the lambda body is not tracked by alloc_tracker
  of the lambda → potential leak. This needs careful integration with
  the lambda's own cleanup path.
