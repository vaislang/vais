# RFC-002: Container-Owned Strings (Vec<str> and struct str fields)

- **Status**: Re-review required (scope correction 2026-04-14, see §9)
  - Prior status: Approved (user sign-off 2026-04-14, Phase 191 #2, commit e1edb7bb)
  - Blocker: §2 "Current state" drift discovered during 191 #2a pre-implementation
    verification. See §9 for the specific corrections and the amended design.
- **Author**: Phase 191 #2 harness (Opus direct)
- **Area**: `crates/vais-codegen/src/vtable.rs`, `crates/vais-codegen/src/inkwell/gen_aggregate.rs`, `crates/vais-codegen/src/string_ops.rs`, `crates/vais-codegen/src/state.rs`, `std/vec.vais` (amended scope 2026-04-14)
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

## 2. Current state (verified 2026-04-14, **corrected 2026-04-14** — see §9)

> ⚠️ **Correction notice**: The original §2.1/§2.2 (preserved below with
> strikethrough intent) misstated Vec's layout and how `push`/`drop` are
> lowered. The corrected picture — which the amended design in §4 and §9
> is built on — is given first.

### 2.1 Vec<T> is a generic stdlib struct with user-defined Drop

Actual definition in `std/vec.vais:51-56`:

```vais
S Vec<T> {
    data: i64,      # Pointer to element array (stored as i64 pointer-int)
    len: i64,       # Current number of elements
    cap: i64,       # Allocated capacity
    elem_size: i64  # sizeof(T) — used as stride for load_typed/store_typed
}
```

Four fields, not three. `elem_size` is the stride for typed element access.
The struct has a user-defined method `F drop(&self) -> i64` (lines 238-244)
that already calls `free(self.data)` and zeroes `len`/`cap`. This drop is
registered for scope-exit cleanup by the normal Named-type destructor path.

**Consequence for design**: `__drop_Vec_str` cannot simply be "auto-emitted
by codegen" as a replacement — it would collide with the existing user
drop. The correct integration point is either (a) monomorphization-specific
**augmentation** of the existing Vec<str>.drop (a codegen-synthesized
prelude that frees owned elements before user code frees `self.data`), or
(b) an amendment to `std/vec.vais` that gives `Vec<T>` an `elem_drop_hook`
that monomorphization fills in for T=str. §4 (corrected) picks (a).

### 2.2 Vec<T>.push is a user method, not a codegen intrinsic

`F push(&self, value: T) -> i64` (lines 186-194) is plain Vais:

```vais
F push(&self, value: T) -> i64 {
    I self.len >= self.cap { @.grow() }
    ptr := self.data + self.len * self.elem_size
    store_typed(ptr, value)   # writes { i8*, i64 } fat pointer whole
    self.len = self.len + 1
    self.len
}
```

For `Vec<str>`, `store_typed` writes the fat pointer `{ i8*, i64 }` into
the element slot. There is no codegen hook at the push call site today:
the alloc_slot that owned the buffer is not updated, the owned-bit array
(proposed in §4.1) does not yet exist, and the `grow()` method `memcpy`s
fat pointers wholesale (which is fine — pointer aliases don't double-free
a buffer, the RFC-001 single-owner invariant is what matters).

**Consequence for design**: ownership transfer at push time requires
codegen to recognise the `Vec<str>.push(heap_concat_result)` pattern and
inject (i) ownership-bit set + (ii) alloc_slot removal **around** the
call, since we cannot edit user method bodies to add that logic
generically without type-specialisation. Two viable paths, discussed in
§4.3 (corrected):
  - **(α)** Codegen call-site wrapping: at every `Vec<str>.push` call
    emit the bit-set + slot-transfer IR before the call and let the
    stdlib body run unchanged.
  - **(β)** Stdlib amendment: split `Vec<T>` into `Vec<T: Copy>` and
    `Vec<T: Heap>` with different push/drop bodies. Significantly higher
    blast radius; rejected in §9.3.

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
inside) the element buffer. The corrected `Vec<str>` layout — accounting
for the existing 4-field `{data, len, cap, elem_size}` from `std/vec.vais`
— adds a **fifth** field:

```
struct Vec<str> {
  data: i64,       // existing: pointer-int to { i8*, i64 } fat-ptr array
  len: i64,        // existing
  cap: i64,        // existing
  elem_size: i64,  // existing: sizeof({i8*,i64}) = 16 on 64-bit targets
  owned: i64,      // NEW: pointer-int to bit-packed i8 array (cap/8 bytes, ceil).
                   //      bit i = 1 → element i is heap-owned; 0 → literal/borrowed.
                   //      Stored as i64 to match the other pointer-int fields'
                   //      storage convention in std/vec.vais.
}
```

This preserves the existing 4-field ABI for `Vec<T>` where T is
not `str`. Only `Vec<str>` specialisations get the fifth field; the
monomorphization path (which already distinguishes `Vec<i64>` from
`Vec<f32>` via `elem_size`) picks the correct layout and initialises
`owned = 0` (null) on construction, lazily allocating the bitmap on
first heap-owned `push`.

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

**(b) struct with user `impl Drop`** — **Option D (double-safe)**:

User `impl Drop` is treated as a **pre-drop hook** for domain logic
(logging, closing external handles, decrementing counters), not a
memory-reclamation hook. The sequence at container destruction is:

1. User `drop(&self)` runs. User code **cannot free heap fields** — the
   language provides no `free_field(&mut self, name)` primitive, and
   field pointers are read-only through `&self`.
2. shallow-drop runs **unconditionally** immediately after. It consults
   the struct's `ownership_mask` and frees only fields whose bit is
   still set.
3. If the user wants to transfer ownership of a field out of the struct
   during `drop`, they call the explicit helper `take_field!(self, name)`
   which clears the corresponding bit in `ownership_mask`. shallow-drop
   will then skip that field.

This design makes **double-free impossible** (only shallow-drop can
free, and it consults a monotonic-cleared bitmap) and **leak
impossible** (every bit set at drop time produces exactly one free).
User Drop has a clean orthogonal role and cannot accidentally corrupt
the memory model. The trade-off is that users cannot directly `free`
fields from Drop — but they also cannot forget to free them.

Note: `take_field!` is a macro/builtin (not a function) because it
needs to return the moved value and simultaneously clear the bit; its
ABI and syntax are out-of-scope for this RFC and will be specified in
an implementation follow-up. Until it exists, transferring ownership
out of a struct Drop is not supported — users must use a dedicated
`into_parts() -> (T1, T2, ...)` method pattern instead.

**Ownership flag storage**: for structs with per-field heap ownership, the
monomorphized struct layout gains a trailing `ownership_mask: i64` field
(bit i = 1 → field i is heap-owned). 64 heap fields per struct is
accepted as the hard cap (Q1 resolved 2026-04-14); overflow is a
compile error. Fixed `i64` is preferred over a dynamic bitmap pointer
because it avoids an extra per-struct heap allocation and keeps the
struct layout fully stack-allocatable.

### 4.3 Vec<str>.push / struct-literal emission changes

**Push path** (corrected 2026-04-14 per §2.2 — call-site wrapping
approach α):

At every *call site* of `Vec<str>.push(rvalue)`, codegen emits:

1. Pre-call owned-bitmap growth: if `self.owned == 0` or the bitmap's
   capacity < `self.cap`, reallocate (mirroring what `grow()` does for
   `data`). This can be a small runtime helper
   `__vais_vec_str_owned_grow(Vec<str>*, new_cap)`.
2. Rvalue classification (same three-way split):
   - Known owner (in `string_value_slot`) → **before** the call,
     compute the new element index `i = self.len` and emit
     `__vais_vec_str_owned_set(self, i)` (sets bit i). Then remove the
     slot entry from `string_value_slot` and
     `scope_str_stack.last_mut()`. Keep the alloc_tracker entry and
     emit a null store (matches Phase 191 #5).
   - Literal → do nothing (bit stays 0).
   - Borrowed (not in `string_value_slot` AND not a literal constant)
     → do nothing. Conservative: "not in string_value_slot" maps to 0.
3. The stdlib `push` body itself is unchanged — it still does
   `store_typed(ptr, value)` to write the fat pointer.

The `__vais_vec_str_owned_set` / `__vais_vec_str_owned_grow` helpers
are emitted once by codegen alongside `__drop_Vec_str` (see §4.4).

**Struct literal path** (`gen_aggregate.rs`):
- For each str field initializer, apply the same three-way classification.
- When emitting the struct's `insertvalue` / GEP+store, also emit a
  masked OR into the struct's `ownership_mask` slot if the field is
  heap-owned.

### 4.4 Container drop emission (`vtable.rs`)

**Corrected 2026-04-14**: `Vec<T>` already has a user-defined
`F drop(&self) -> i64` in `std/vec.vais` that frees `self.data`. We do
NOT replace it. Instead, codegen emits a **prelude helper**
`__vais_vec_str_shallow_free(%Vec_str*)` for the `Vec<str>`
monomorphization, and splices it into the scope-exit drop sequence
*before* the user drop runs:

```
scope-exit for a Vec<str> local:
  1. call __vais_vec_str_shallow_free(%v)   // NEW: frees heap elements + bitmap
  2. call Vec.drop(%v)                       // EXISTING user method: free(self.data)
```

The prelude helper is defined as:

```llvm
define void @__vais_vec_str_shallow_free(%Vec_str* %v) {
entry:
  %len     = load i64, i64* %v.len
  %data_i  = load i64, i64* %v.data
  %owned_i = load i64, i64* %v.owned
  %has_owned = icmp ne i64 %owned_i, 0
  br i1 %has_owned, label %free_elems, label %done
free_elems:
  %data   = inttoptr i64 %data_i   to { i8*, i64 }*
  %owned  = inttoptr i64 %owned_i  to i8*
  ; loop i in 0..len:
  ;   byte = owned[i/8]
  ;   if (byte >> (i%8)) & 1:
  ;     elem   = load { i8*, i64 }, ptr data + i
  ;     eptr   = extractvalue elem, 0   ; the buffer pointer
  ;     call void @free(i8* eptr)
  call void @free(i8* %owned)
  br label %done
done:
  ret void
}
```

Why a prelude helper rather than auto-emitting `__drop_Vec_str` that
also frees `self.data`? Because the user drop already does the latter,
and replacing it would silently drop any user-authored side effects in
`Vec<T>.drop` (none today, but we keep the user drop authoritative).
The prelude is a strict addition, not a replacement.

Emitted once per monomorphization of `Vec<str>` (and equivalents for
other heap-capable element types). Inserted into the drop_registry
alongside the user `Vec<str>.drop` entry. Scope-exit cleanup calls
both in order.

Auto-emit `__drop_shallow_{Struct}` for any struct with heap-owned fields
that lacks a user Drop. If the struct has a user Drop, the same ordering
rule applies: `__drop_shallow_{Struct}` runs **after** user drop (per
§4.2 Option D — memory reclaim is strictly post-user-drop for structs).
Note that this is the *opposite* order from Vec<str> because for structs
the bitmap consultation happens after user logic (user drop is a
pre-drop hook), while for Vec<str> the existing user drop releases the
backing buffer itself and must run last. See §9.4 for the rationale
reconciliation.

Algorithm: for each field i with heap-ownership potential, check bit i
of the struct's ownership_mask and, if set, free the field
(recursively for nested containers).

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

## 5. Resolved design decisions (user review 2026-04-14)

1. **Struct ownership mask — RESOLVED**: Fixed `i64` (64-field cap). An
   overflow is a compile error. Dynamic bitmap pointer rejected to keep
   structs stack-allocatable and avoid an extra per-struct heap alloc.

2. **User Drop interaction — RESOLVED (Option D)**: User `impl Drop` is
   a pre-drop hook for domain logic. Users **cannot free heap fields**
   from Drop — only shallow-drop (consulting the ownership bitmap) may
   free. Ownership transfer out of a struct during Drop requires an
   explicit `take_field!` primitive that clears the corresponding bit.
   Eliminates double-free and leak-through-forgotten-free by
   construction. `take_field!` syntax/ABI deferred to impl follow-up.

3. **Nested containers — deferred to implementation**: `Vec<Vec<str>>`
   requires `__drop_Vec_Vec_str` to recurse into `__drop_Vec_str` per
   element. Vais does not have cyclic generics today (confirmed in
   RFC-001 context); revisit if future language changes allow them.

4. **`Vec<str>` iter borrow — confirmed safe by §4.1**: Only the Vec
   consults its owned bitmap at drop; iterator fat-pointer copies cannot
   trigger a free path.

5. **Migration — RESOLVED**: Full recompilation required (Vais is
   pre-1.0). No compat shim. selfhost/std rebuild is part of the
   implementation plan.

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

**Sign-off (2026-04-14)**: All five review questions resolved. Safety
invariants (no double-free, no silent leak) made structural via Option
D + bitmap-as-single-source-of-truth. Implementation can proceed as
191 #2a → #2b → #2c, each with its own e2e baseline gate + team-review.

---

## 9. Scope correction (2026-04-14, pre-#2a)

During 191 #2a pre-implementation verification, reading `std/vec.vais`
and `crates/vais-codegen/src/vtable.rs` revealed that §2.1 and §2.2
(the "Current state" section) misstated the Vec<T> layout and the
push/drop lowering model. This section consolidates the corrections and
the design adjustments they force.

### 9.1 Drift found

| Claim in original RFC | Reality (verified 2026-04-14) |
|-----------------------|-------------------------------|
| `Vec<T>` has 3 fields `{ i8*, i64, i64 }` | `Vec<T>` has **4** fields `{data: i64, len: i64, cap: i64, elem_size: i64}` (`std/vec.vais:51`) |
| Vec<T> does not implement Drop; codegen auto-emits `__drop_Vec_str` to replace nothing | Vec<T> **already has** a user-defined `F drop(&self)` at `std/vec.vais:238-244` that frees `self.data` |
| `push` is a codegen intrinsic (`__vais_vec_push_*`) | `push` is a plain Vais method at `std/vec.vais:186-194` using `store_typed` |
| "Add 4th field `owned: i8*`" | Must be a **5th** field `owned: i64` to coexist with `elem_size` |
| `__drop_{Type}` handles element freeing | `__drop_{Type}` today just calls `free(%ptr)` unconditionally (`vtable.rs:71-97`); needs iteration logic or a separate helper |

### 9.2 Design adjustments (reflected inline in §2, §4.1, §4.3, §4.4)

- **Vec<str> layout** (§4.1): 5 fields `{data, len, cap, elem_size, owned}`.
  All i64. Non-str Vec<T> keeps the existing 4-field layout. Mono-
  morphization picks the layout by T.
- **Push integration** (§4.3): *call-site wrapping* (path α). Codegen
  emits owned-bit set + slot transfer IR around each
  `Vec<str>.push(...)` call; the stdlib method body is unchanged. A
  small runtime helper `__vais_vec_str_owned_set(self, i)` performs
  the bit flip, and `__vais_vec_str_owned_grow(self, new_cap)` mirrors
  the existing `grow()` reallocation for the owned bitmap.
- **Drop integration** (§4.4): add **prelude** helper
  `__vais_vec_str_shallow_free(%Vec_str*)` that frees heap-owned
  elements and the owned bitmap. Spliced into scope-exit *before* the
  existing user `Vec.drop` call. Vec's user drop stays authoritative
  for `self.data` itself. No collision, no replacement.
- **Struct drop ordering** (§4.4): still runs *after* user drop (Option
  D pre-drop-hook semantics). The per-container choice of pre vs post
  is justified in §9.4.

### 9.3 Rejected during correction: stdlib Vec<T> split

Path β ("split Vec<T> into Vec<T: Copy> and Vec<T: Heap> in stdlib")
was considered and rejected:

- Requires introducing Copy/Heap trait bounds which Vais does not have
  today — would pull in RFC-scale language work on its own.
- Breaks every selfhost+stdlib caller (Vec is used in hundreds of
  files). Blast radius dwarfs the bug being fixed.
- The call-site-wrapping approach (α) achieves the same safety with
  zero stdlib churn and one new monomorphized helper pair per element
  type.

### 9.4 Drop-ordering asymmetry: why Vec uses prelude but Struct uses postlude

- **Vec<str>**: elements live inside `self.data` (a heap block).
  User `Vec.drop` frees that block. If we freed individual elements
  *after* `Vec.drop`, their pointers would already be reclaimed —
  UAF. So element-freeing must precede user drop → **prelude**.
- **Struct with `name: str` + user Drop**: fields are embedded in the
  struct's own memory. User Drop sees them alive regardless of
  ordering. Option D places shallow-drop *after* user Drop so that
  user Drop can run as a domain-logic pre-hook without accidentally
  double-freeing fields → **postlude**.

Both orderings converge on the same safety invariant (exactly one
free per heap allocation), just with different splice points.

### 9.5 Monomorphization note

Vais' existing monomorphization path for `Vec<T>` (which specialises
`elem_size` per T) is the integration point for:
- Choosing the 4-field vs 5-field layout.
- Emitting the `__vais_vec_str_{owned_set, owned_grow, shallow_free}`
  trio iff T is `str` (or, later, any heap-capable element).
- Registering the shallow-free prelude in the drop sequence.

The exact codegen hook (`vais-types` monomorphization pass vs
`vais-codegen` post-IR synthesis) will be fixed during #2a
implementation and recorded as an implementation note; both are
feasible and do not change the user-visible model.

### 9.6 Re-review requested

Because §2, §4.1, §4.3, §4.4 changed materially, user re-sign-off is
requested before #2a implementation begins. The five original review
questions (§5) remain resolved — the corrections do not disturb
Option D, the i64 ownership_mask for structs, the full-recompilation
migration stance, or the nested-container deferral.

**Diff summary for review**:
- §1, §3, §6 — unchanged.
- §2 — rewritten to reflect actual Vec<T> layout + user drop.
- §4.1 — 4-field → 5-field Vec<str> layout (`+elem_size, +owned`).
- §4.3 — push path becomes call-site wrapping, not intrinsic rewrite.
- §4.4 — prelude helper for Vec<str>, strict addition over existing
  user drop; struct postlude unchanged.
- §5, §7, §8 — unchanged.
- §9 — NEW (this section).

**Re-sign-off line** (fill in when approved):
`Re-approved 2026-04-14 after §9 scope correction: user (via "전체 그냥 계속해서 진행해줄수없어?" directive in Session 3 — auto-progress resume)`

### 9.7 New blocker — %Vec generic fallback vs %Vec$str specialized layout

Discovered during #2a pre-implementation survey (Session 3, iter 8): the
text-IR codegen uses **both** `%Vec` (a generic 4-field fallback type)
and `%Vec$T` (specialized per monomorphization) **simultaneously**.
Numerous GEP sites (`method_call.rs:650`, `helpers.rs:438`,
`loops.rs:289` + ~10 others) emit `getelementptr %Vec, %Vec* ...` on
Vec receivers regardless of their concrete T — relying on the shared
4-field layout.

If `%Vec$str` has a 5th `owned` field, passing a `%Vec$str*` through a
`%Vec*`-shaped GEP path is *safe* only up through field index 3
(elem_size). Reading field index 4 (owned) via the generic `%Vec`
path is **OOB**. Conversely, writing to the owned field requires the
specialized layout to be known at the GEP emission site.

Three resolution paths:

- **(i) Full audit**: every `%Vec` GEP site learns to emit
  `%Vec$T` when the receiver's concrete type is known. Most invasive;
  safest invariant.
- **(ii) Shared 5-field layout**: `%Vec` itself becomes 5 fields; non-str
  Vec<T> pays an 8-byte dead-field cost. Violates §4.1's "non-str Vec
  ABI unchanged" goal but requires zero GEP-site changes.
- **(iii) Sidecar table**: keep `%Vec` 4 fields; for Vec<str>, allocate
  the owned bitmap in a codegen-maintained side table keyed by the
  Vec's runtime `data` pointer (or by its stack address). No layout
  change, but reintroduces the side-table concurrency risk §4.1
  rejected.

**Decision required** before #2a implementation resumes. Recommended:
**(ii)** — the 8-byte cost is negligible for a stdlib type used
everywhere, and §4.1's "ABI unchanged" goal was aspirational relative
to user code, not the runtime struct layout. Non-str Vec just stores
0 in the `owned` slot and `__vais_vec_str_shallow_free` is only
registered for Vec<str> specifically.

User action: choose (i)/(ii)/(iii), then the author amends §4.1 + §4.4
accordingly and proceeds to #2a.
