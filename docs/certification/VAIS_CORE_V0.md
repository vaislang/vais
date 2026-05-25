# Vais Core v0 Certification Spec

## Purpose

Vais Core v0 is the first correctness target for the language and compiler. A feature is not considered guaranteed because it appears in README files, old roadmap entries, or examples. It is guaranteed only when it is listed here and covered by the Core certification gate.

The goal is a small language that can be parsed, type-checked, lowered or validated, code-generated, linked, and run without relying on unstable inference or backend fallbacks.

## Certification Rule

For every Core construct, the project must define:

- accepted syntax
- type rule
- required compiler stage
- positive fixture
- negative fixture where applicable
- expected failure stage and error code for invalid programs

The required stage names are the stages from `compiler/docs/COMPILER_STAGES.md`: Lex, Parse, Type check, Codegen, Link, Run.

## Core Surface

### Source Model

| Construct | Status | Required stage | Notes |
|---|---|---:|---|
| UTF-8 source files | Core | Parse | Invalid text must fail before type checking. |
| `#` line comments | Core | Parse | Comments do not affect token spans beyond line mapping. |
| identifiers | Core | Parse | ASCII identifiers are the Core minimum. |
| integer literals | Core | Run | Decimal literals with explicit inferred or annotated type. |
| string literals | Core | Run | `str` is allowed, but advanced ownership behavior is outside Core. |
| boolean literals | Core | Run | `true`, `false`. |

### Declarations

| Construct | Status | Required stage | Notes |
|---|---|---:|---|
| function declaration `F name(...) -> T { ... }` | Core | Run | Parameters and return type must be explicit. |
| expression-body function `F name(...) -> T = expr` | Core | Run | Allowed only when the expression type is fully resolved. |
| `S` struct declaration | Core | Run | Named fields only. Tuple structs are Deferred. |
| `EN` enum declaration | Core | Run | Prefer `EN`; legacy context-dependent `E` is Deferred for new Core fixtures. |
| `U` import | Core-minimal | Codegen | Only for Core std modules and local Core fixtures. |

### Statements and Expressions

| Construct | Status | Required stage | Notes |
|---|---|---:|---|
| local binding with explicit type | Core | Run | Example: `x: i64 := 1`. |
| local binding with obvious literal inference | Core-minimal | Run | Allowed for primitive literals only. |
| assignment to mutable local | Core | Run | Mutation must be explicit with `mut` where required by current language rules. |
| arithmetic on integers | Core | Run | `+`, `-`, `*`, `/`, `%` for same concrete numeric type. |
| boolean comparison | Core | Run | Comparison returns `bool`; no bool-to-int coercion. |
| `I ... { ... } EL { ... }` | Core | Run | Core fixtures use `bool` conditions. The full compiler currently accepts integer truthy conditions for legacy code, so a stricter Core-only rejection requires a future core-mode gate. Branch types must match exactly after allowed explicit casts. |
| `LW condition { ... }` | Core | Run | Core fixtures use `bool` conditions. Integer truthy conditions are outside Core style but still accepted by the full compiler for compatibility. |
| simple `L` loop with `B` / `C` | Core | Codegen | Run fixtures added only after deterministic examples exist. |
| `M` match over Core enum | Core | Run | Exhaustiveness checking required where currently implemented. |
| `R` return | Core | Run | Return value type must match function signature. |
| explicit cast `as` | Core | Run | Only documented safe primitive casts. |

### Types

| Type | Status | Required stage | Notes |
|---|---|---:|---|
| `unit` / `()` | Core | Run | Function bodies without meaningful value. |
| `bool` | Core | Run | No implicit integer conversion. |
| `i64` | Core | Run | Primary integer type for Core fixtures. |
| `u8`, `u64` | Core-minimal | Run | Include only where existing compiler behavior is stable. |
| `str` | Core-minimal | Run | String construction and passing only. Complex mutation is Deferred. |
| `Vec<T>` | Core-minimal | Codegen | Creation, push, indexing only for certified element types. |
| `Option<T>` | Core-minimal | Codegen | `Some` / `None` and match over certified `T`. |
| `Result<T,E>` | Core-minimal | Codegen | Basic construction and match in Core v0. Product-level A2-01/A2-02 now certify bounded `?` propagation in `A2_SUBSETS.md`, but that does not widen this frozen Core v0 manifest unless Core fixtures are added here. |

## Relationship To W1 A2 Certified Subsets

W1-A through W1-C added product-level language/compiler certifications adjacent
to Core v0:

- A2-01/A2-02: bounded Core-typed `Result`/`Option` `?` propagation.
- A2-03: narrow dyn/trait dispatch with visible impls and invalid non-impl
  rejection.
- A2-04: inline/no-escape closures.
- A2-05: bounded named-function pointer parameters.

Those subsets are documented and gated in `A2_SUBSETS.md`. They may be cited by
product language docs and DB-required profiles, but they are not automatically
part of the frozen Core v0 fixture manifest. Moving any of them into Core v0
requires the promotion rule below and an explicit `core-certify.sh` fixture
change.

## Core Std Modules

Only the following std modules are eligible for Core v0 fixtures unless Task 1 discovers a hard compiler dependency:

| Module | Status | Notes |
|---|---|---|
| `option.vais` | Core-minimal | Required for Option fixtures. |
| `result.vais` | Core-minimal | Required for Result fixtures. |
| `vec.vais` | Core-minimal | Only certified methods are allowed. |
| `string.vais` | Core-minimal | Construction and read-only use only. |
| `core.vais` / primitive helpers | Core-minimal | Include only if imported by existing fixtures. |

Everything else in `compiler/std/` starts as Deferred or Experimental until promoted by fixture coverage.

## Boundary Invariants

Core v0 has these non-negotiable compiler invariants:

1. Parse success means the AST contains no placeholder syntax nodes.
2. Type-check success for a Core fixture means every expression has a resolved type.
3. Codegen must not receive `ResolvedType::Unknown`, `ResolvedType::Var`, or unresolved `Infer` for Core fixtures.
4. Codegen must not use `i64` as a fallback for unknown Core types.
5. Every invalid Core fixture fails before or at the declared stage with a stable error code and a structured `error[CODE]` diagnostic header.
6. LLVM verifier or clang failures are compiler bugs for Core fixtures, not acceptable user errors.

## Promotion Rule

A Deferred or Experimental feature can enter Core only after:

1. the syntax and type rule are documented here,
2. positive and negative fixtures are added,
3. the feature passes the required stage in `core-certify.sh`,
4. compiler boundary invariants still pass,
5. same-class audit confirms no hidden fallback path was added.

## Initial Core Fixture Set

Task 2 should create fixtures for these cases first:

| Fixture group | Positive examples | Negative examples |
|---|---|---|
| functions | explicit params, expression body, block body | missing return, return type mismatch |
| primitives | integer arithmetic, bool comparison, string return | bool-int coercion, int-str coercion |
| control flow | if/else branch match, while loop | string/non-predicate condition, mismatched branch types |
| structs | literal construction, field read/write | unknown field, wrong field type |
| enums | Option-like enum, Result-like enum, match | non-exhaustive match if supported, bad variant field |
| Vec minimal | create, push, index | index element type leak, unsupported field access |

## Current Certification State

Status: Active draft with an executable Core gate.

`compiler/scripts/core-certify.sh` reads `compiler/tests/core/manifest.tsv` and
runs the initial Core positive and negative fixture set. MIR structural
validation is documented in `MIR_CONTRACT.md`; the reference interpreter path is
implemented for the current strict MIR subset and compared with native LLVM for
strict Core run fixtures.

The promoted strict enum subset includes unit enum matching, `Option<i64>`
payload matching, and `Result<i64,i64>` payload matching. Broader generic
payloads remain outside strict MIR until explicitly promoted.

The current dated gate evidence is recorded in `CURRENT_STATUS.md`. Use that
file and the root workspace `ROADMAP.md` for live status. Historical roadmap
counts and archived phase notes are not certification evidence unless they are
re-run by the current gate.
