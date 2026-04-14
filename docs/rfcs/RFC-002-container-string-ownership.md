# RFC-002: Container-Owned Strings (Vec<str> and struct str fields)

- **Status**: Draft (Phase 191 #2, 2026-04-14)
- **Author**: Phase 191 #2 harness (Opus direct)
- **Area**: `crates/vais-codegen/src/vtable.rs`, `crates/vais-codegen/src/inkwell/gen_aggregate.rs`, `crates/vais-codegen/src/string_ops.rs`, `crates/vais-codegen/src/state.rs`
- **Requires**: RFC-001 §4 (function-scope ownership model is the base).
- **Blocks**: RFC-001 §8 item "Container-owned strings".

## 1. Problem

RFC-001 defined single-owner semantics for strings **within a function frame**:
every heap `str` has exactly one alloc_slot tracking it, and ownership transfers
cleanly across `return`, `let`, PHI merges, and block exit. Three cases remain
out of scope and are the subject of this RFC:

```vais
# Case A: Vec<str>
v := Vec<str>::new()
v.push("hello " + "world")  # concat returns heap str; pushed into Vec

# Case B: struct with str field
S Person { name: str }
p := Person { name: "Alice " + "Smith" }

# Case C: Vec<struct { s: str }>
v2 := Vec<Person>::new()
v2.push(Person { name: "x" + "y" })
```

In all three cases today, the heap buffer returned by `+` is copied as a
fat-pointer into the container, but **no owner is registered for the container's
elements**. When the container goes out of scope, function-exit cleanup sees
only the alloc_slot (which was reassigned or nulled by the push/store path),
and the heap buffer leaks.

A naïve fix — "have the Vec destructor free every element" — is unsafe because
the Vec's elements can be a mix of:

1. **Literals** (`@str.N` globals) — never free.
2. **Heap owned** (concat results pushed directly) — must free.
3. **Borrowed** (fat-pointer aliases still owned by an outer alloc_slot) — must not free.

The fat-pointer ABI (`{ i8*, i64 }`) has no spare bit for a runtime
heap/literal discriminator; RFC-001 intentionally chose not to introduce one
(§4.3).

## 2. Current state (verified 2026-04-14)

### 2.1 Vec destructor: not emitted

- `crates/vais-codegen/src/vtable.rs:63-97` emits `__drop_TypeName` functions
  **per concrete type implementing the `Drop` trait**. Vec<T> does not
  implement Drop today — it is a generic struct
  (`S Vec<T> { data: i64, len: i64, cap: i64 }`) whose buffer lives on the
  heap but is released by no codegen path. Manual `impl Drop for Vec<T>` is
  not synthesized.

### 2.2 Vec<str>.push lowering

- Vec.push is lowered via intrinsic (likely `__vais_vec_push_*` per element
  class) that memcpy's the fat pointer `{i8*, i64}` into the Vec buffer.
  The alloc_slot that previously owned the buffer is NOT updated — it
  simply points at a now-shared pointer.

### 2.3 Struct literal field init

- `crates/vais-codegen/src/inkwell/gen_advanced.rs:319-420` emits
  `insertvalue` / GEP+store for field initialization. No ownership transfer;
  the alloc_slot of the rvalue keeps the buffer as "its own", even though
  the struct now has a fat-pointer copy.

### 2.4 Struct destructor (`__drop_TypeName`)

- If the struct has `impl Drop`, the user-written `drop(&self) -> i64`
  method is called at scope exit. The method must manually free str fields —
  **but there is no generated code that frees str fields automatically**.
  Users writing `S Person { name: str }` without a Drop impl get a silent
  leak.

### 2.5 Ownership tracker (RFC-001)

- `fn_ctx.string_value_slot: HashMap<ssa_key, slot_name>` — per-function,
  not per-container.
- `fn_ctx.alloc_tracker: Vec<(slot_name, ptr_reg)>` — per-function slots
  that function-exit cleanup scans.
- `fn_ctx.scope_str_stack: Vec<Vec<slot_name>>` — block-scope slots
  (Phase 190.5/190.6/191#5).
- **Nothing tracks ownership AT or BELOW the container level.** A slot is
  "owned by the function until it transfers out via return." Push into a
  Vec is not a transfer today; the slot still appears in alloc_tracker
  and would get freed at function exit, giving a UAF if the Vec escaped
  via return.

## 3. Goals

1. A `Vec<str>` local whose elements are heap-owned must free every element
   when the Vec is dropped (scope exit or explicit).
2. A `struct { f: str }` local whose `f` field is heap-owned must free
   the field when the struct is dropped.
3. Literals pushed into the same Vec/struct **must not be freed**. Mixed
   literal/heap contents must remain safe.
4. Ownership transfers from a function's local-scope alloc_slot into the
   container are **single-owner**: after `v.push(a+b)`, the function-scope
   cleanup must not also try to free that buffer.
5. Moving a container into another function (by return, by `let` that
   crosses a scope) must transfer ownership of ALL its elements, not
   leave dangling alloc_slot entries.
6. No fat-pointer ABI change. No per-element tag bits.
7. Performance: O(n) element free at drop is acceptable; no per-element
   runtime discriminator checks.

## 4. Design

### 4.1 Chosen approach: "heap-promotion on container insertion"

When a value is inserted into a container at a position of type `str`:

- If the inserted value is a **literal**: insert as-is (fat pointer to
  `.rodata`). The container MUST NOT free this at drop.
- If the inserted value is a **heap-owned concat result** or other owned
  buffer: **transfer** the ownership from the function's alloc_slot to the
  container. The alloc_slot is removed from `scope_str_stack` and
  `string_value_slot` (and left in `alloc_tracker` with a null store
  emitted — same trick as Phase 191 #5).
- If the inserted value is a **borrowed** fat pointer (from a `str`
  parameter or another container element): the container becomes a
  co-borrower; neither the push site nor the container drop frees it.

**How does the container know, at drop time, which elements to free?**

The container gets a **per-element ownership bitmap**, stored beside (not
inside) the element buffer. Concretely, the `Vec<str>` layout becomes:

```
struct Vec<str> {
  data: i8*,       // existing: pointer to { i8*, i64 } fat-ptr array
  len: i64,        // existing
  cap: i64,        // existing
  owned: i8*,      // NEW: pointer to a bit-packed i8 array, cap/8 bytes.
                   //      bit i = 1 → element i is heap-owned; 0 → literal/borrowed.
}
```

This preserves the existing `{ i8*, i64, i64 }` ABI for Vec<T> where T is
not `str`. Only Vec<str> specialisations get the fourth field; the
monomorphization path (which already distinguishes Vec<i64> from Vec<f32>)
picks the correct layout.

**Alternative considered**: keep element layout identical and store the
bitmap in a codegen-side side table keyed by the Vec's heap address.
Rejected because the side table must persist across function returns and
introduces synchronization risk under future concurrency.

### 4.2 struct str field ownership

Two sub-cases:

**(a) struct without `impl Drop`**: codegen auto-emits a "shallow drop"
function `__drop_shallow_{Name}` when the struct has any heap-owned
field (determined at monomorphization time from field types). The shallow
drop frees each `str` / container field following the same rules as (4.1)
for containers. Struct literal construction from a heap-owned rvalue
transfers ownership: the source alloc_slot is removed from
`scope_str_stack` + `string_value_slot`, and the struct's "field is owned"
flag is set.

**(b) struct with user `impl Drop`**: the user's Drop is called; codegen
does NOT also call shallow-drop. The user is responsible for freeing str
fields. We document this clearly: "If you write `impl Drop`, you own the
bodies of all heap fields." This matches Rust's model.

**Ownership flag storage**: for structs with per-field heap ownership, the
monomorphized struct layout gains a trailing `ownership_mask: i64` field
(bit i = 1 → field i is heap-owned). 64 heap fields per struct should
cover every realistic case; overflow can be a compile error.

### 4.3 Vec<str>.push / struct-literal emission changes

**Push path** (`inkwell/gen_aggregate.rs` + string_ops.rs):
1. Determine if the rvalue has a known owner (look up in
   `string_value_slot`). If yes → set the owned-bit for the new element
   index; remove the entry from `string_value_slot` and
   `scope_str_stack.last_mut()`. (Alloc_tracker is preserved + null-store
   emitted, matching Phase 191 #5.)
2. If not (literal): leave owned-bit at 0.
3. If borrowed (fat-pointer came from a `str` function parameter):
   leave owned-bit at 0. Detection: the SSA key is not in
   `string_value_slot` AND not a literal constant. In practice,
   the compiler can always conservatively classify "not in
   string_value_slot" as "literal or borrowed" — both map to owned-bit 0.

**Struct literal path** (`gen_aggregate.rs`):
- For each str field initializer, apply the same three-way classification.
- When emitting the struct's `insertvalue` / GEP+store, also emit a
  masked OR into the struct's `ownership_mask` slot if the field is
  heap-owned.

### 4.4 Container drop emission (`vtable.rs`)

Auto-emit `__drop_Vec_str` (and monomorphized equivalents for other
heap-capable element types):

```llvm
define void @__drop_Vec_str(i8* %ptr) {
  %vec = bitcast i8* %ptr to %Vec_str*
  %data = load i8*, i8** %vec.data
  %len = load i64, i64* %vec.len
  %owned = load i8*, i8** %vec.owned
  ; loop i in 0..len:
  ;   if owned[i/8] & (1 << i%8):
  ;     %elem_ptr = (data as {i8*,i64}*)[i].0
  ;     call free(%elem_ptr)
  call void @free(i8* %data)
  call void @free(i8* %owned)
  ret void
}
```

Emitted once per monomorphization. Inserted into the drop_registry for
`Vec<str>`. Scope-exit cleanup calls the registered drop function.

Auto-emit `__drop_shallow_{Struct}` for any struct with heap-owned fields
that lacks a user Drop. Algorithm: for each field i with heap-ownership
potential, check bit i of the struct's ownership_mask and, if set, free
the field (recursively for nested containers).

### 4.5 Transfer on return / let-cross-scope

When a `Vec<str>` or struct-with-heap-fields is returned from a function,
or bound to an outer-scope `let`, RFC-001 §4.5/§4.6's transfer mechanism
is extended:

- `pending_return_skip_slot` already excludes the returned container's
  alloc_slot.
- We add `pending_return_skip_container` tracking the container's
  heap-owned buffer(s) — specifically the Vec's `data` + `owned` buffers,
  or the struct's heap field buffers — so function-exit cleanup does not
  free them before the caller receives them.
- The container's own alloc_slot (for its top-level `data` ptr) is already
  tracked by RFC-001 in `alloc_tracker`; this continues to work.

### 4.6 Rejected alternatives

- **(B) Tag-bit inside the fat pointer** (stash owned-flag in the low bit
  of `i8*`, alignment guarantees 2+ unused bits): Rejected because it
  forces every string operation to mask-off the tag before use, creating
  a pervasive tax on hot paths. Also interacts poorly with future
  small-string-optimization and interning.

- **(C) "Always clone to heap on push"**: Every push would `strdup` its
  input, guaranteeing heap ownership. Rejected because (i) it doubles
  memory for literal-heavy workloads and (ii) it doesn't solve the
  function-return-transfer case on its own.

- **(D) Runtime pointer-provenance check**: Compare the element pointer
  against `.rodata` bounds at drop time. Rejected because
  platform-dependent (.rodata location is an OS/loader detail) and
  fragile under dlopen.

- **(E) Per-element wrapper type**: Have Vec<str> internally hold
  `Vec<{ptr, owned_bit}>` instead of `Vec<{i8*,i64}>`. Rejected because
  the fat-pointer ABI leaks through iter(), slice access, and any user
  code that borrows an element — we'd have to redesign `&str` too.

## 5. Open questions for reviewer

1. **Ownership mask for structs**: Is the 64-field limit acceptable, or
   should we use a growable bitset pointer (matching Vec<str>.owned)?
   Trade-off: fixed i64 is fast/simple; pointer requires extra alloc per
   struct.

2. **User-Drop interaction**: If the user writes `impl Drop for Person`
   and forgets to free `name`, should codegen warn? RFC-001 established
   compile-time-only discrimination; here we could emit a diagnostic
   when we can prove a heap-owned field is never freed in the user's
   Drop body. But dataflow analysis of Vais Drop bodies is out of scope
   for this RFC. Proposal: defer to a separate lint RFC.

3. **Nested containers**: `Vec<Vec<str>>` — the outer Vec's `__drop_Vec_Vec_str`
   must call `__drop_Vec_str` for each element. The monomorphization path
   needs to recurse. Is there a cycle risk (Vec<T> where T references a
   Vec transitively)? Vais does not have cyclic generics today; confirm
   and document.

4. **`Vec<str>` iter borrow**: `for s in vec` — the borrowed `s` must
   not free. The owned-bit belongs to the Vec, not to the iterator's
   fat-pointer copy. Confirmed safe by §4.1 — only at drop time does
   the Vec consult its own owned bitmap.

5. **Migration**: existing code that uses `Vec<str>` will get the new
   layout after rebuild. Is a compat shim needed, or can we rely on
   full recompilation? (Vais is currently pre-1.0; recompilation is
   acceptable.)

## 6. Test plan

New test file `crates/vaisc/tests/e2e/phase191_container_str_drop.rs`
(≥5 cases):

1. `vec_str_push_drop_no_leak`: Vec<str> of concat results, Vec drops,
   leaks --atExit reports 0. (macOS-specific gate; fall back to
   crash-free iteration for CI.)
2. `vec_str_mixed_literal_and_heap`: Vec<str> with both `vec.push("lit")`
   and `vec.push("a"+"b")`. Drop must free only the heap element.
3. `struct_str_field_drop`: `S Person { name: str }; p := Person { name: "a"+"b" }`.
   p drops, leaks=0.
4. `struct_user_drop_takes_ownership`: user `impl Drop for Person` is
   called once, shallow-drop is NOT called.
5. `nested_vec_of_struct_str`: `Vec<Person>` where Person has `name: str`
   field pushed from concat. Outer Vec drop frees all inner names.
6. `vec_str_return_transfers`: `F build() -> Vec<str> { v := Vec::new(); v.push("a"+"b"); v }` —
   caller receives live Vec, drops it, leaks=0. No double-free.

## 7. Implementation phases

Splittable into three sub-phases if complexity warrants:

- **191 #2a**: Vec<str> layout change + ownership mask + auto-emitted
  `__drop_Vec_str`. Tests 1, 2, 6.
- **191 #2b**: Struct shallow-drop synthesis + ownership_mask field.
  Tests 3, 4.
- **191 #2c**: Nested container recursion. Test 5.

Gate: user review of this RFC (§5 open questions answered) before any
code change. Then 2a → 2b → 2c sequentially, each with its own e2e
baseline preservation + team-review.

## 8. Acceptance criteria

- All tests in §6 green on inkwell backend. Text-IR backend: same tests,
  gated on RFC-001 §5.4 parity (Phase 191 #5 is complete as of 2026-04-14).
- E2E baseline (2576 after Phase 191 #5) + new tests all passing.
- RFC-001 §8 checklist item "Container-owned strings" marked complete.
- No fat-pointer ABI change.
- No performance regression >2% on the existing Vec<i64> / Vec<f32>
  monomorphizations (measured via criterion if benches exist).

---

**Next step**: User reviews §5 open questions. After sign-off, draft
implementation plan and split into 191 #2a/#2b/#2c subtasks.
