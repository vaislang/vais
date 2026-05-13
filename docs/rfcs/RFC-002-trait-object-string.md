# RFC-002-trait-object-string: Trait Object String Return Ownership

- **Status**: Draft
- **Author**: Phase 191 #3 (Opus direct)
- **Area**: `crates/vais-codegen/src/vtable.rs`, `crates/vais-codegen/src/trait_dispatch.rs`
- **Requires**: RFC-001 §4 (function-scope ownership model)
- **Blocks**: RFC-001 §8 item "Trait object str return"

## 1. Problem

When a `dyn Trait` method returns `str`, the current vtable dispatch
hardcodes all non-void returns as `i64`. Since `str` is a fat pointer
`{ i8*, i64 }` (16 bytes), this truncates the return value and loses
the length component. Additionally, there is no ownership tracking for
heap-owned strings returned through the vtable boundary.

```vais
W Describable {
    F describe(&self) -> str
}

S Cat { name: str }
X Cat: Describable {
    F describe(&self) -> str {
        "cat:" + self.name  # heap-allocated concat result
    }
}

F print_desc(d: dyn Describable) -> i64 {
    s := d.describe()  # Currently broken: returns i64, not {i8*, i64}
    println(s)
    0
}
```

## 2. Current State

### vtable_struct_type (vtable.rs:186-191)
All non-void returns hardcoded to `"i64"`. Function pointer type in vtable
is `i64(i8*, ...)`.

### generate_vtable_global (vtable.rs:233-238)
Same `"i64"` hardcoding for concrete→vtable bitcast.

### generate_dynamic_call (vtable.rs:416-420)
fn_type uses `ret_type` parameter which comes from trait_dispatch.rs.
Call arguments all typed as `i64`.

### trait_dispatch.rs:236-240
`ret_type` determination: Unit→"void", else→"i64".

## 3. Design

### 3.1 Return Type Detection

Replace the blanket `"i64"` with type-aware detection:
- `ResolvedType::Str` → `"{ i8*, i64 }"` (fat pointer)
- `ResolvedType::Unit` → `"void"`
- Everything else → `"i64"` (unchanged)

This applies consistently to:
1. vtable struct type generation
2. vtable global constant emission
3. dynamic call generation
4. trait dispatch return type determination

### 3.2 Ownership Transfer

When a dynamic call returns `{ i8*, i64 }`:
1. Extract the raw `i8*` via `extractvalue ... 0`
2. Track it via `track_alloc_with_slot` (same as concat results)
3. Register in `string_value_slot` with the fat pointer SSA name
4. Register in `scope_str_stack` top frame

This follows the exact same pattern as RFC-001 §4.3 (concat returns).
The callee is responsible for allocating; the caller takes ownership.

### 3.3 Callee Convention

The concrete method implementation (e.g., `Cat_describe`) already returns
`{ i8*, i64 }` because it's a normal function that happens to return `str`.
The vtable just needs to use the correct type in its function pointer
instead of truncating to `i64`.

No callee changes needed — the fix is entirely in the vtable dispatch path.

## 4. Implementation

### Files Changed
- `crates/vais-codegen/src/vtable.rs`: 3 sites (vtable_struct_type,
  generate_vtable_global, generate_dynamic_call)
- `crates/vais-codegen/src/trait_dispatch.rs`: 1 site (ret_type detection)
- Ownership tracking: at the call site in expression codegen where
  `generate_dyn_method_call` result is used

### Helper Function
```
fn vtable_ret_type(ret: &ResolvedType) -> &str {
    match ret {
        ResolvedType::Unit => "void",
        ResolvedType::Str => "{ i8*, i64 }",
        _ => "i64",
    }
}
```

## 5. Test Plan

1. `trait_str_return_basic`: Trait method returns concat result, caller
   prints it → correct output + exit 0
2. `trait_str_return_no_leak`: Trait method returns concat, caller drops
   scope → no leak
3. `trait_str_return_literal`: Trait method returns literal str → no
   crash (literal is not heap-owned, should not be freed)

## 6. Risks

- **ABI change**: vtable layout for traits with str-returning methods
  changes from `i64(i8*)*` to `{ i8*, i64 }(i8*)*`. This is safe
  because Vais is pre-1.0 and requires full recompilation (RFC-002 §5.5).
- **Existing traits**: Any trait that already has a str-returning method
  will now work correctly instead of silently truncating. No regression
  risk for traits that don't return str.
