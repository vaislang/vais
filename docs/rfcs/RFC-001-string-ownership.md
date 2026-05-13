# RFC-001: String Ownership Model and Drop-Tracking

- **Status**: Draft (Phase 190.5, 2026-04-13)
- **Author**: Phase 190.5 harness (Opus direct)
- **Area**: `crates/vais-codegen/src/string_ops.rs`, `crates/vais-codegen/src/vtable.rs`, `crates/vais-codegen/src/stmt.rs`
- **Requires**: CLAUDE.md §"Type Conversion Rules" (Phase 158) style of enforcement — this RFC establishes an analogous locked-in rule for string ownership.

## 1. Problem

In long-running programs (e.g. `vais-apps/monitor` running over minutes), memory grows monotonically when user code concatenates strings via `+`:

```vais
msg := a + b + c + d
```

Each intermediate (`a+b`, `(a+b)+c`) calls the runtime helper `__vais_str_concat`, which `malloc`s a new buffer and returns a fat pointer. The caller never frees any intermediate — only the final rvalue pointer is tracked in the function's `alloc_slot`, and even then only as "the latest stored pointer," not as a set of owned allocations.

A naive fix ("just `free` the intermediate buffers") is **not safe**, because the same fat-pointer representation is used for:

1. **Literals** — `@str.N` globals in `.rodata`. Freeing these is UB.
2. **Heap-owned** — result of `__vais_str_concat`, `__vais_str_substring`, `strdup`-equivalents. These MUST be freed exactly once.
3. **Borrowed** — fat pointers that alias heap-owned allocations (e.g., a `str` parameter forwarded to another call). Freeing these here double-frees.

The codegen currently has **no discriminator** between these three cases. This is the root cause.

## 2. Current state (as of 2026-04-13)

Research findings (verified by file read, not memory):

### 2.1 String representation
- Runtime layout: `{ i8*, i64 }` fat pointer (`data`, `len`). Literals point into `.rodata`; concat/substring results point to `malloc`'d buffers.
- **No tag bit or discriminator** is currently stored to distinguish literal vs heap.

### 2.2 Drop infrastructure that already exists
- **Vec drop**: `crates/vais-codegen/src/vtable.rs:63` — `VtableGenerator::generate_drop_function`.
- **Struct drop**: `crates/vais-codegen/src/stmt.rs:824` — `generate_drop_cleanup`. Consults `drop_registry`, emits `Drop::drop()` in LIFO order.
- **`alloc_slot` lifecycle**: entry-block `alloca i8**` per tracked allocation, null-initialized, cleared at function exit after defer/drop cleanup. `track_alloc` at `stmt.rs:811-818`.
- **`str` has no destructor entry** in the vtable. `str` is modeled as immutable fat pointer; there is no RAII hook for it.

### 2.3 Concat lowering (the bug)
- Entry: `string_ops.rs:112` (`generate_string_binary_op`, `BinOp::Add`).
- Runtime helper: `string_ops.rs:434` (`__vais_str_concat`). Allocates `alen + blen + 1` via `malloc`, `memcpy`s both inputs, returns fat pointer. **Does not free either input.**
- Emission pattern:
  ```llvm
  %result = call { i8*, i64 } @__vais_str_concat(i8* %left_ptr, i8* %right_ptr)
  %raw = extractvalue { i8*, i64 } %result, 0
  store i8* %raw, i8** %__alloc_slot_N
  ```
- The `store` overwrites the slot. If the slot previously held `a+b`'s heap pointer and now receives `(a+b)+c`'s heap pointer, the earlier allocation is leaked. The slot is a **shared reuse slot**, not a list of live allocations.

### 2.4 Existing tests
- `crates/vaisc/tests/e2e/phase134_string.rs`
- `crates/vaisc/tests/e2e/phase90_strings.rs`
- `crates/vaisc/tests/e2e/phase183_option_result_struct.rs`
- `crates/vaisc/tests/e2e/phase145_r4_drop.rs` — defer + Drop trait integration (closest existing coverage).
- **No leak test** for string concat chains currently exists.

## 3. Design principles (locked-in, mirror of Phase 158)

These principles have the same "locked-in" status as Phase 158's type-conversion rules: changing them requires a new RFC + E2E protection test update.

1. **Single owner per heap string.** Every heap-allocated string buffer has exactly one owning location at any instant. Transferring ownership moves the pointer; borrowing it does not.
2. **Literals never enter the drop set.** `.rodata` pointers are statically distinguishable at codegen time by **call site**, not by runtime tag.
3. **Ownership is tracked per-allocation, not per-slot.** The existing shared `alloc_slot` is a reuse buffer and does not reflect ownership. A new per-scope "owned string list" is introduced.
4. **Drop emission piggy-backs on existing RAII.** `str` gets an entry in the drop path used by Vec/struct (`stmt.rs:generate_drop_cleanup`). No new backend-specific drop code path.
5. **No runtime tag bit.** The fat-pointer ABI stays `{ i8*, i64 }`. Discrimination is compile-time only.

## 4. Proposed model

### 4.1 Ownership categories (compile-time only)

Every `str`-typed SSA value in codegen carries, as part of its codegen-side metadata (not LLVM IR), one of:

| Category | Origin | Drop at scope exit? |
|----------|--------|-------|
| `Literal` | `@str.N` global, string literal expression | No |
| `Owned` | `__vais_str_concat`, `__vais_str_substring`, `strdup`-equivalents, fresh heap allocation | **Yes** |
| `Borrowed` | Function parameter of type `str`, field load, element of `&[str]`, return value of a function returning a borrowed `str` (future) | No |

Today every concat result is tagged `Owned`; every string literal is `Literal`; every `str` parameter is `Borrowed`. No new language-level type distinction is introduced — categories exist only inside `gen_expr`.

### 4.2 Per-scope owned string list

Augment `FnCodegenCtx` (the codegen-side function context that already holds `drop_registry` and defer state) with:

```rust
// pseudo-code, exact field name TBD during implementation
owned_strings: Vec<ScopeStrList>,  // stack, one entry per lexical scope
```

Each `ScopeStrList` is a `Vec<PointerValue>` of heap pointers produced in that scope and not yet transferred out. Operations:

- **Push**: right after a call to `__vais_str_concat` / `__vais_str_substring`, the result pointer is pushed onto the innermost scope's list.
- **Transfer out (return / bind to outer-scope variable)**: the pointer is removed from the current scope's list and pushed onto the target scope's list. If the target is the function's return value, it is removed entirely (caller takes ownership — see §4.4).
- **Drop at scope exit**: on block end, emit `call void @free(i8* %ptr)` for every pointer remaining in the list, then pop the list. Emission order: LIFO.

### 4.3 Concat chain `a + b + c + d` walk-through

Pseudo-LLVM with the new model:

```llvm
; a, b are Borrowed (literals or parameters); %t1 is Owned
%t1 = call { i8*, i64 } @__vais_str_concat(i8* %a, i8* %b)
; push %t1.ptr to owned_strings[current_scope]

; %t2 is Owned, %t1 consumed (still tracked — see §4.4 on consumption)
%t2 = call { i8*, i64 } @__vais_str_concat(i8* %t1.ptr, i8* %c)
; push %t2.ptr

%t3 = call { i8*, i64 } @__vais_str_concat(i8* %t2.ptr, i8* %d)
; push %t3.ptr

; bind: let msg = %t3  — msg adopts %t3; %t3 stays in list because its binding lives to scope end
; at scope exit:
;   free(%t3.ptr)   ; msg's backing buffer
;   free(%t2.ptr)   ; intermediate
;   free(%t1.ptr)   ; intermediate
```

### 4.4 The "consumption" question

`__vais_str_concat(x, y)` **reads but does not consume** x and y. Both `x` and `y` remain valid pointers after the call (the runtime memcpy'd them). Therefore ownership of x and y is **unchanged** by the call — each intermediate remains owned by its original scope's list and is freed at scope exit.

This means `a + b + c + d` produces **three** owned allocations (`a+b`, `(a+b)+c`, `(a+b)+c+d`), all dropped at scope exit. The total peak memory for the chain is `O(sum of lengths)`, which is the best we can do without a dedicated N-ary concat intrinsic (see §7 Future work).

### 4.5 Assignment and rebinding

```vais
x := a + b      // x owns T1
x = a + b + c   // x now owns T2; T1 must be freed immediately
```

On re-assignment of a `str` local that was tracked as `Owned`:
- Before the new store, emit `free(old_ptr)` and remove the old pointer from the scope list.
- Push the new pointer.

This is consistent with Rust's `Drop`-on-reassignment semantics.

### 4.6 Return value ownership transfer

```vais
F build() -> str {
  return a + b  // T1 escapes this function
}
```

When the return expression produces an `Owned` string:
- Remove its pointer from `owned_strings`.
- The **caller's** scope list receives the pointer (same mechanism as a regular concat result).

Caller side: the return value of a function returning `str` is treated as `Owned` iff the function's signature is annotated (implicitly, by the codegen analyzer) as returning an owned string. For now, **every function returning `str` is treated as returning `Owned`**, matching current de-facto behavior (there is no way to return a borrowed `str` today without introducing `&'a str` lifetimes, which is out of scope).

### 4.7 Parameters

`str` parameters are `Borrowed`. The callee does not add them to `owned_strings` and does not free them. This matches existing lowering — no change.

## 5. Interaction with existing features

### 5.1 `defer` ordering
Current order at scope exit (per `stmt.rs:generate_drop_cleanup`):
1. defer blocks run (LIFO).
2. Vec/struct drop cleanup runs (LIFO).

New step inserted **after** Vec/struct drop and **before** slot clearing:

3. **String drop**: `free` every pointer in the innermost scope's `owned_strings` list (LIFO).

Rationale: if a Vec/struct destructor needed to read a `str` field (e.g., log it), the str must still be live. By placing string drop after destructor emission, destructors always see valid strings. This is analogous to Rust where `String` fields of a struct are dropped **by** the struct's generated `Drop`, not before it.

**Complication**: heap strings stored *inside* a Vec or struct field are owned by that container, not by the function scope. For Phase 190.5, strings stored in structs/Vecs are **outside the scope** of this RFC — they are already handled (incorrectly, possibly also leaking) by container destructors. A follow-up RFC will address container-owned strings. **This RFC covers only strings owned by local scopes via `alloc_slot`-tracked temporaries.**

### 5.2 `alloc_slot` relationship
The existing `alloc_slot` remains for pointer-reuse / dominance reasons (loops). It is orthogonal to ownership: a slot may point to a literal, to a heap allocation currently owned by `owned_strings`, or to `null`. The drop is driven by `owned_strings`, not by the slot. **The slot is not modified by this RFC.**

### 5.3 Trait objects returning `str`
Out of scope. Follow-up RFC.

### 5.4 Text-IR backend vs inkwell backend
The research confirms concat emission lives in `string_ops.rs` and is shared. `generate_drop_cleanup` in `stmt.rs` is the single cleanup point. No divergence between backends. **One implementation path, not two.**

## 6. Implementation plan

1. **Instrument `__vais_str_concat` call sites** (`string_ops.rs:112`, and the substring path) to push result pointers onto `owned_strings`.
2. **Extend `FnCodegenCtx`** with `owned_strings: Vec<Vec<PointerValue>>` and scope push/pop helpers.
3. **Hook into `generate_drop_cleanup`** (`stmt.rs:824`): after existing Vec/struct drop emission, emit `free` for each remaining heap string in LIFO order.
4. **Reassignment path**: in the `str` local store lowering, emit `free` for the previous `Owned` pointer (if any) before the new store.
5. **Return path**: in the return-expression lowering, when the returned value is an `Owned` string, remove it from the list before emitting the `ret`.
6. **E2E test**: `crates/vaisc/tests/e2e/phase190_str_concat_drop.rs`:
   - Compile a program that does `a + b + c + d` in a tight loop (e.g., 10_000 iterations).
   - Use `assert_exit_code` + spawn the binary under `leaks` (macOS) or ASan (Linux).
   - Assert: 0 leaks, RSS plateau within O(1) MB after warmup.
7. **Full E2E rerun**: 2563 passing, 0 failing. Special attention to `phase134_string`, `phase90_strings`, `phase183_option_result_struct`, `phase145_r4_drop`.

## 7. Risks & open items

| Risk | Mitigation |
|------|------------|
| Pointer escapes via raw cast (`as` of `str` to `i64`) | Forbidden by Phase 158 rules — no risk today. |
| Loop body with concat: each iteration leaks until block exit | Fine: scope list drop fires at block exit (end of loop body), so per-iteration frees happen correctly. |
| Closures capturing `str` | Out of scope for Phase 190.5. Closures today capture by copy of fat pointer; this can alias an owned buffer. Must be addressed in follow-up RFC before closures + long-running concat are safe. |
| Panic / early return inside the scope | LLVM landingpad/cleanup blocks must run the same `free` sequence. Vais today uses `abort`-on-panic (no unwinding), so this is not a concern in 190.5. Will be re-examined when unwinding lands. |

## 8. Future work (explicitly not in 190.5)

- Container-owned strings (Vec<str>, struct fields).
- Trait objects returning `str`.
- N-ary concat intrinsic that avoids intermediate allocations (`__vais_str_concat_n`).
- Small-string optimization.
- String interning.

## 9. Testing & verification gate

- `cargo test -p vaisc --test e2e` → 2564 passing (2563 baseline + 1 new).
- `leaks --atExit -- target/debug/phase190_str_concat_drop` (macOS) → `0 leaks`.
- `vais-apps/monitor` 5-minute smoke: RSS plateau. Measurement script to live under `vais-apps/monitor/bench/rss_plateau.sh`.

## 10. Sign-off

- [x] User approval to proceed with implementation (2026-04-13, "근본적 해결 진행해줘").
- [x] §4.4 decision: **aggressive free on next concat** (LHS consumed → immediate
      free) combined with **per-scope drop** for the final un-consumed result.
      Correctness gained by tracking ownership via alloc_slot + variable-name
      binding; no new aliasing proof required.
- [x] Baseline re-confirmed: `cargo test -p vaisc --test e2e` = 2563 passed /
      0 failed (2026-04-13).
- [x] Team review (2026-04-14) found a Critical UAF in 1st-pass implementation
      (`let msg = a+b; return msg` patterns — SSA mismatch on load-from-alloca).
      Phase 190.6 added variable-name tracking (`var_string_slot` +
      `var_string_slots_multi` + PHI merge) and regression tests.
- [x] 2nd-pass verification: `phase190_str_concat_drop` (8 tests) + full E2E
      (2563 + 8) all pass (2026-04-14).
- [ ] (Out of scope — tracked in follow-up RFC): container-owned strings
      (`Vec<str>`, struct fields), trait objects returning `str`, closures,
      text-IR backend full scope-drop parity.
