# MIR Core Contract

## Status

MIR is part of the compiler pipeline, and it now has a narrow semantic
comparison path for the strict Core fixture subset. It is not yet the
authoritative semantic oracle for all of Vais Core v0.

The current lowering pass intentionally accepts some unsupported AST forms by
lowering them to `Constant::Int(0)` placeholders. That behavior keeps the MIR
optimization and emission pipeline testable, but it means a lowered MIR module
can silently lose source semantics. Until that fallback is removed or converted
to a hard diagnostic for Core constructs, LLVM execution remains a backend
result, not independent proof of language semantics.

## Current Gate

`vais-mir::lower::lower_module_checked` and `vais-mir::validate` provide the
first MIR certification gate:

```bash
cargo test -p vais-mir --test lower_strict_tests --release
cargo test -p vais-mir --test core_strict_fixtures --release
cargo test -p vais-mir --test interpreter_tests --release
cargo test -p vais-mir --test validation_tests --release
```

Strict lowering rejects semantic-loss placeholders before validation.
`tests/core/mir_strict.tsv` lists the Core fixtures currently certified through
that path. `tests/core/mir_deferred.tsv` lists Core fixtures intentionally kept
out of the MIR semantic path until the lowering contract can represent them
without placeholders. The deferred manifest may be empty when every current
positive Core run fixture has a strict MIR contract.

The validator is structural. It checks that MIR consumers do not receive
malformed bodies:

- each body has return local `_0`
- `_0` has the function return type
- parameter locals `_1.._n` exist and match the declared parameter types
- every body has at least one basic block
- every basic block has a terminator
- `goto`, `switch`, `call`, and `assert` targets point to existing blocks
- statement operands, terminator operands, destinations, and indexed places
  reference declared locals
- duplicate function bodies are rejected at module scope

`vais-mir::interpreter` executes the primitive/control/aggregate subset needed
by the strict fixture allowlist. The current strict run fixtures are
`basic_return.vais`, `call_and_block.vais`, `int_bool_string.vais`,
`if_else_while.vais`, `point.vais`, `color_match.vais`, and
`option_match.vais`, `result_match.vais`, and `vec_i64.vais`. `core-certify.sh` also compares MIR
interpreter exit codes with native LLVM execution for the strict Core run
fixtures. This proves semantic agreement for that promoted subset only; future
Core fixtures still need lowering/interpreter promotion before they can be used
as oracle evidence.

`Option<i64>` is represented in strict MIR as a synthetic enum named
`Option<i64>` with variants `None` (discriminant 0, no payload) and `Some`
(discriminant 1, one `i64` payload). Broader `Option<T>` payloads remain outside
the certified subset and must report a strict lowering diagnostic instead of
reusing the `Option<i64>` layout.

`Result<i64,i64>` is represented in strict MIR as a synthetic enum named
`Result<i64,i64>` with variants `Ok` (discriminant 0, one `i64` payload) and
`Err` (discriminant 1, one `i64` payload). Broader `Result<T,E>` payloads remain
outside the certified subset and must report a strict lowering diagnostic
instead of reusing the `Result<i64,i64>` layout. The
`strict_lowering_rejects_uncertified_result_payload_types` test locks this
negative path.

`Vec<i64>` is represented in strict MIR as `MirType::Vec(I64)`.
`vec_new()` lowers to `AggregateKind::Vec`, `push` lowers to `Rvalue::VecPush`
assigned back to the receiver place, `len` lowers to `Rvalue::Len`, and index
reads lower to `Place::Index`. Broader `Vec<T>` element types and arbitrary
methods remain outside the certified subset until they get the same typed
lowering and interpreter contract.

## Core MIR Invariants

For a Core-certified source program, MIR may be treated as a trusted compiler
artifact only when all of these are true:

1. The source passes the Core fixture manifest.
2. Type checking completes with no unresolved `ResolvedType::Unknown` or
   `ResolvedType::Var` in the current source range.
3. Lowering emits no placeholder value for a supported Core construct.
4. `validate_module` succeeds.
5. MIR optimization preserves `validate_module` success.
6. Any unsupported Core construct fails before codegen with a typed compiler
   error instead of producing a placeholder.

## Fallback Audit

`crates/vais-mir/src/lower.rs` still contains legacy `Constant::Int(0)`
defaults. The audit classifies them as follows:

| Class | Status | Rule |
|-------|--------|------|
| `semantic_loss` helper result | blocked in strict path | Every caller must add `MirLowerError`; current callers are unsupported statement, unbound identifier, unsupported expression, unknown enum variant, unsupported enum struct variant literal, unknown struct literal, and unsupported field access. |
| struct literal field completeness diagnostics | blocked in strict path | Duplicate, unknown, or missing struct literal fields are reported through the strict diagnostic helper before any promoted fixture can be certified. |
| unsupported `Option<T>` payload types | blocked in strict path | Only `Option<i64>` has a payload-specific MIR enum contract today. Other payloads must fail strict lowering until their layout and interpreter behavior are certified. |
| unsupported `Result<T, E>` payload types | blocked in strict path | Only `Result<i64, i64>` has a payload-specific MIR enum contract today. Other payloads must fail strict lowering until their layout and interpreter behavior are certified. |
| unsupported `Vec<T>` element types | blocked in strict path | Only `Vec<i64>` has a MIR value/interpreter contract today. Other element types must fail strict lowering until their layout and operations are certified. |
| unsupported method/index forms | blocked in strict path | `Vec<i64>.push`, `Vec<i64>.len`, and `Vec<i64>` index reads are certified. Other `MethodCall`/`Index` forms still go through audited strict diagnostics instead of placeholders. |
| Function block initial accumulator | legacy/default only | A typed Core source should not depend on an empty i64 block becoming 0. Do not add fixtures that require this behavior. |
| Generic statement-list initial accumulator | legacy/default only | Empty expression blocks are not semantic-oracle certified. |
| `return` continuation blocks | allowed unreachable placeholder | Used after an emitted return to keep builder state valid. The interpreter must follow terminators from the entry block. |
| `if` or `else-if` without `else` default | deferred unit/statement compatibility | Valid only for statement-like lowering today; do not certify value semantics until unit-typed MIR is explicit. |
| self-tailcall continuation | allowed unreachable placeholder | Tail call terminates the current block; the placeholder is for the dead continuation only. |
| match identifier binding placeholder | blocked in strict path | Strict lowering emits `MirLowerError`; legacy lowering may bind 0 for old tests only. |

The audit is locked by `lower_strict_tests::strict_lowering_fallback_audit_is_current`.
That test counts strict diagnostic sites, `semantic_loss` callers, and direct
`Operand::Constant(Constant::Int(0))` sites. If any count changes, update this
audit and add a same-class test before accepting the change.

## Reference Interpreter Path

The first reference interpreter path is in place for the strict fixture subset.
The remaining implementation order is:

1. Promote deferred Core constructs into `mir_strict.tsv` one class at a time.
2. Extend the interpreter for the promoted constructs before adding them to the
   semantic comparison gate.
3. Compare interpreter output with native LLVM execution for every promoted
   `run` fixture.
4. Promote MIR from "strict subset oracle" to "Core semantic oracle" only when
   interpreter and LLVM agree on every Core `run` fixture.

Until step 4 is complete, a native binary run outside the strict subset remains
a backend integration check, not a complete semantic proof.

## Vec/Index Contract

`tests/core/positive/collections/vec_i64.vais` is now in `mir_strict.tsv` and
Core `run` comparison. The certified contract covers `Vec<i64>` construction,
`push` mutation, `len` read, and index reads without `Constant::Int(0)`
placeholders. The interpreter runs the same MIR and `core-certify.sh` compares
its exit code with native LLVM.

`lower_strict_tests::strict_lowering_emits_vec_aggregate_push_len_and_index`
locks the emitted MIR shape. `strict_lowering_rejects_uncertified_vec_element_type`
keeps broader `Vec<T>` payloads out of the strict subset until they are
explicitly certified.

## Non-Goals

The MIR validator does not perform source parsing, type inference, borrow
checking, LLVM validation, or runtime evaluation. Those remain separate gates.
