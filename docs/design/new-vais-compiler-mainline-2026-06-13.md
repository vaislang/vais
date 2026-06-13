# New Vais Compiler Mainline Plan (2026-06-13)

## Decision

New Vais / Vais is the user-facing language name. The `nl` repository name and
`.nl` extension remain transitional implementation names. The legacy bootstrap
adapter's canonical name is `legacy_vais_bootstrap.py`; `nl2vais.py` remains as
a compatibility wrapper for old calls.

Legacy Vais (`/Users/sswoo/study/projects/vais/compiler`) stays as:
- bootstrap backend for the current validated pipeline,
- oracle for value-correctness comparison,
- regression source when New Vais exposes backend/compiler bugs.

It is not the owner of New Vais semantics.

## Goal

Build the New Vais native compiler as the mainline:

```
New Vais source (.vais, with .nl accepted during transition)
  -> New Vais lexer/parser/typecheck
  -> direct LLVM IR emitter
  -> clang/native execution
```

The native path must eventually replace the transitional path:

```
New Vais source (.nl)
  -> legacy_vais_bootstrap.py
  -> Legacy Vais
  -> vaisc
  -> clang/native execution
```

## Non-Goals

- Do not rename `projects/nl` or `.nl` without a dedicated migration gate.
- Do not delete the `nl2vais.py` compatibility wrapper until old external calls
  are migrated.
- Do not delete the Legacy Vais bootstrap path before the native compiler has
  its own parity gate.
- Do not attempt L4 ecosystem/product distribution in this phase.

## Mainline Contract

The native compiler owns these from day one:

- P1/P2/P3: one-token/one-meaning, one syntax per operation, no ambiguity shortcuts.
- P4: diagnostics include `help:` and a concrete correction when a known Rust/Vais habit is detected.
- P7/P7b: coercion is centralized and every feature is value-tested, not just compile-tested.
- P8: callable values use closure object semantics (`{code, env}`), not bare function pointers for captured closures.
- P9: examples are first-class regression assets.

## Bootstrap Contract

Current green gates are the baseline:

- `bash scripts/test-vaisc.sh` = New Vais `vaisc` smoke OK
- `bash scripts/test-vaisc-front.sh` = New Vais day-1 front contract OK
- `bash scripts/test-vaisc-direct.sh` = New Vais direct emitter OK
- `bash scripts/test-vaisc-errors.sh` = New Vais native P4 diagnostics OK
- `bash scripts/test-vaisc-parity.sh` = New Vais native parity manifest OK
- `bash scripts/test.sh` = 112/112
- `bash scripts/test-fixpoint-full.sh` = self-host e2e OK
- `bash scripts/test-fixpoint-full-self.sh` = full-source stage compare OK
- `python3 tests/transpiler_test.py` = 59/59
- `python3 tests/nl_check_test.py` = 40/40

Every native-compiler slice must either:
- pass the relevant subset and preserve these gates, or
- add a tracked gap with file/line diagnosis and leave the existing green path intact.

## Slice Order

### NV-C0: Product Boundary

Define the native compiler CLI and artifact contract.

Status: done as of 2026-06-13.

Command contract:
- `scripts/vaisc emit-ir SOURCE.vais -o OUT.ll`
- `scripts/vaisc build SOURCE.vais -o OUT`
- `scripts/vaisc run SOURCE.vais`

Transition rules:
- `SOURCE.vais` is the user-facing source spelling.
- `.nl` remains accepted as the transitional extension.
- repo-local `scripts/vaisc` avoids colliding with the Legacy `vaisc` binary while the bootstrap remains active.
- Legacy `vaisc` is resolved only as an internal bootstrap/oracle backend.

Gate:
- `bash scripts/test-vaisc.sh`

Done when:
- a tiny native compiler entrypoint has a documented command shape,
- at least one smoke program emits LLVM IR through that entrypoint,
- generated IR builds/runs with `clang`,
- New Vais `build`/`run` commands return the expected value,
- Legacy bootstrap oracle returns the same value,
- verification is scriptable.

### NV-C1: Front Contract

Freeze the day-1 subset that the native compiler must parse/typecheck first.

Status: done as of 2026-06-13.

Accepted day-1 native subset:
- `fn main() -> Int`,
- helper `fn name(a: Int, ...) -> Int`,
- `let` / `let mut`,
- integer literals, arithmetic, comparison values,
- `return`,
- `if` / `else`,
- `while`,
- function calls,
- `print("...{x}...")` interpolation and `putchar(Int)` IO calls,
- simple `struct` literals and field access,
- payload-free enum tags and simple return-arm `match`,
- small Int-coded payload enum/match slices,
- single-Int closure return lowering,
- `List<Int>` local push/growth, `.len()`, `.sum()`, and index access.

Rejected with P4-style `help:` diagnostics:
- missing or non-`Int` `fn main` entrypoint,
- helper functions without `name: Int` params and `-> Int` return,
- `for`, broader payload enum/match, broader closures, traits/impls,
- strings/chars/bools, `Map`/`Option`/`Result`, `?`,
- method calls beyond `.push()`/`.len()`/`.sum()`,
- Rust-habit spellings such as `&&`, `||`, `!`, `as`, `::`, `Vec`, `HashMap`, `String`, compound assignment.

Gate:
- `bash scripts/test-vaisc-front.sh`

Done when:
- subset is documented,
- unsupported constructs fail with P4-style diagnostics,
- a focused gate covers accepted and rejected examples.

### NV-C2: Direct LLVM IR Emitter

Separate direct LLVM IR emission from the Legacy Vais transpiler path.

Status: done as of 2026-06-13 for the first minimal direct slice.

Command contract:
- `scripts/vaisc emit-ir SOURCE.vais --engine direct -o OUT.ll`
- `scripts/vaisc build SOURCE.vais --engine direct -o OUT`
- `scripts/vaisc run SOURCE.vais --engine direct`

Current direct subset:
- exactly one `fn main() -> Int`,
- body is a single `return <expr>`,
- `<expr>` supports Int literals, parentheses, `+`, `-`, `*`, `/`, `%`,
  and integer comparisons lowered through `icmp`/`zext`.

The default engine remains `bootstrap` while direct parity grows.

Gate:
- `bash scripts/test-vaisc-direct.sh`

Done when:
- simple arithmetic and return emit valid LLVM IR,
- generated IR builds with `clang`,
- result matches bootstrap path for the same source.

### NV-C3: P4 Diagnostics

Move known `nl-check` correction knowledge into the native compiler path.

Status: done as of 2026-06-13 for the native day-1 diagnostic slice.

Diagnostic contract:
- include New Vais source coordinates,
- print the source line and a caret at the reported column,
- include `help:`,
- include `fix:` for known correction patterns.

Covered day-1 patterns:
- `&&` -> `and`,
- `||` -> `or`,
- `x as Type` -> `Type(x)`,
- `Path::Name` -> `Path.Name`,
- Rust scalar type names such as `i32`/`f64` -> New Vais type names,
- turbofish constructors such as `Vec<Int>::new()` -> literals,
- direct emitter parse failures such as identifiers in literal-only return expressions.

Gate:
- `bash scripts/test-vaisc-errors.sh`

Done when:
- at least `&&`, `||`, `as`, `::`, Rust type names, and turbofish produce `help:` suggestions,
- diagnostics use New Vais source coordinates.

### NV-C4: Parity Gate

Create a native compiler gate that runs a growing subset of `examples/`.

Status: done as of 2026-06-13 for the manifest-backed first gate.

Contract:
- `tools/vaisc-parity.tsv` records each added source as `native-supported`, `bootstrap-only`, or `tracked`,
- `native-supported` entries must build/run through New Vais `scripts/vaisc` and Legacy `scripts/build.sh`,
- both paths must match the source `# expect:` value,
- trusted `compiler/self/*` tier sources may bypass the narrow user-front preflight, but remain guarded by
  manifest parity plus the long self-host gates,
- `bootstrap-only` entries must remain Legacy-green and be rejected by the native front,
- `tracked` entries must remain Legacy-green and are expected not to pass natively yet; if one starts passing, the gate fails so it can be promoted.

Current coverage:
- `native-supported=37`,
- `bootstrap-only=0`,
- `tracked=0`.

Promoted native slices after the first gate:
- `%` term operator tokenization and `srem` lowering, covering `gcd`, Collatz,
  and digit-sum examples.
- Bitwise builtin calls, `Int(...)`, generic marker skip for Int helpers,
  string literal equality, and single-byte char literals.
- `print` interpolation and `putchar` IO calls, covering `examples/e14_print.nl`.
- Simple struct field access and List push/growth/index access, covering
  `examples/c4.nl` and `examples/e75_list_push.nl`.
- List literal `.sum()` support, covering `examples/c2.nl`.
- Payload-free enum tags and simple return-arm match, covering `examples/c1.nl`.
- Payload-free enum dispatch and small Int/self-recursive payload enum lowering, covering
  `examples/e22_enum_dispatch.nl`, `examples/e35_calc_dispatch.nl`,
  `examples/e30_enum_payload_match.nl`, and `examples/e50_ast_eval.nl`.
- Single-Int closure return lowering, covering `examples/e80_closure_return.nl`.
- Trusted self-host tier sources, covering `compiler/self/fixpoint.nl`,
  `compiler/self/fixpoint2.nl`, `compiler/self/fixpoint3.nl`, and
  `compiler/self/fixpoint_full.nl`.

Gate:
- `bash scripts/test-vaisc-parity.sh`

Done when:
- native gate runs separately from `scripts/test.sh`,
- each added example records whether it is native-supported, bootstrap-only, or tracked,
- native-supported examples are value-correct.

## Migration Rule

NV-C4 now covers the core examples and self-host tiers. The adapter rename is
done; the remaining physical rename decisions are:

- `projects/nl` -> `projects/vais`,
- optional `.nl` -> `.vais`,
- eventual removal of the `nl2vais.py` compatibility wrapper.

Until then, `nl` is a stable implementation code name, not the language brand.
