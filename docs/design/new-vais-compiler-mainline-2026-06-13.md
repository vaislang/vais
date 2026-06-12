# New Vais Compiler Mainline Plan (2026-06-13)

## Decision

New Vais / Vais is the user-facing language name. The `nl` repository name,
`.nl` extension, and `nl2vais.py` remain transitional implementation names
until the native compiler reaches parity with the current bootstrap gates.

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
  -> nl2vais.py
  -> Legacy Vais
  -> vaisc
  -> clang/native execution
```

## Non-Goals

- Do not rename `projects/nl`, `.nl`, or `nl2vais.py` before parity.
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

Initial subset:
- `fn main() -> Int`,
- `let` / `let mut`,
- integer literals and arithmetic,
- `return`,
- `if` / `else`,
- `while`,
- function calls.

Done when:
- subset is documented,
- unsupported constructs fail with P4-style diagnostics,
- a focused gate covers accepted and rejected examples.

### NV-C2: Direct LLVM IR Emitter

Separate direct LLVM IR emission from the Legacy Vais transpiler path.

Done when:
- simple arithmetic and return emit valid LLVM IR,
- generated IR builds with `clang`,
- result matches bootstrap path for the same source.

### NV-C3: P4 Diagnostics

Move known `nl-check` correction knowledge into the native compiler path.

Done when:
- at least `&&`, `||`, `as`, `::`, Rust type names, and turbofish produce `help:` suggestions,
- diagnostics use New Vais source coordinates.

### NV-C4: Parity Gate

Create a native compiler gate that runs a growing subset of `examples/`.

Done when:
- native gate runs separately from `scripts/test.sh`,
- each added example records whether it is native-supported, bootstrap-only, or tracked,
- native-supported examples are value-correct.

## Migration Rule

Only after NV-C4 covers the core examples and self-host tiers should the project
consider physical rename:

- `projects/nl` -> `projects/vais`,
- optional `.nl` -> `.vais`,
- `nl2vais.py` -> bootstrap/legacy adapter name.

Until then, `nl` is a stable implementation code name, not the language brand.
