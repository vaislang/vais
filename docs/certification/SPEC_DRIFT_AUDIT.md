# Core Spec Drift Audit

Date: 2026-05-01

## Scope

This audit compares the live Core certification boundary with the broad language
documentation and README claims. It does not add language features or widen
Core. Its purpose is to prevent old implementation notes from being read as
current guarantees.

## Sources Checked

- `tests/core/manifest.tsv`
- `tests/core/mir_strict.tsv`
- `tests/core/mir_deferred.tsv`
- `docs/certification/VAIS_CORE_V0.md`
- `docs/certification/MIR_CONTRACT.md`
- `docs/certification/EXCLUDED_FEATURES.md`
- `docs/LANGUAGE_SPEC.md`
- `README.md`

## Current Fixture Boundary

Core certification currently has 16 manifest entries:

- 9 positive run fixtures:
  - `basic_return.vais`
  - `call_and_block.vais`
  - `int_bool_string.vais`
  - `if_else_while.vais`
  - `point.vais`
  - `color_match.vais`
  - `option_match.vais`
  - `result_match.vais`
  - `vec_i64.vais`
- 7 negative check fixtures:
  - missing brace
  - bool returned as `i64`
  - undefined variable
  - implicit `str` to `i64`
  - non-predicate condition
  - mismatched `if` branch types
  - unknown struct field

Every positive fixture is currently listed in `tests/core/mir_strict.tsv`.
`tests/core/mir_deferred.tsv` is header-only, so there is no hidden deferred
Core fixture at this date.

## Drift Found And Corrected

| Area | Finding | Action |
|---|---|---|
| README self-hosting claim | Historical `21/21 clang 100%` wording could be read as current correctness proof. | Reworded as historical self-hosting workbench context; Core gate is the proof boundary. |
| README type/memory claim | "full constraint solving" and broad memory-safety wording exceeded the certified Core scope. | Reworded to mention Core-certified inference and non-Core destructor/FFI safety. |
| README status checklist | Broad `[x]` checklist implied product-complete surfaces. | Replaced with an implementation inventory explicitly outside Core proof unless promoted. |
| LANGUAGE_SPEC status legend | `stable` could be read as Core-certified. | Reworded to `implemented` and explicitly separated full-compiler status from Core certification. |
| LANGUAGE_SPEC `?` operator | Matrix listed `?` as fully run-stable while `EXCLUDED_FEATURES.md` defers `?` from Core. | Marked as non-Core partial in the matrix and error-handling section. |
| LANGUAGE_SPEC `Vec<Struct>[i].field =` | Historical B.4 wording implied write-through stability despite non-Core gap notes. | Reworded as non-Core pending a current write-through fixture. |
| Phase 182 milestone wording | Historical milestones read like current certification claims. | Reworded as historical implementation notes. |

## No Change

These broad surfaces remain documented but outside Core unless promoted:

- async/await runtime semantics
- macros
- trait objects and full vtables
- unsafe/FFI-heavy patterns
- advanced generics
- destructor/drop semantics
- networking and server/runtime packages

## Next Audit Slice

The ignored/deferred surface audit is complete and recorded in
`IGNORED_SURFACE_AUDIT.md`. The next certification audit should focus on
whether new AI-native language principles are tied to executable gates:

1. compact syntax must remain unambiguous for new Core fixtures,
2. stale phase notes must not override current gate evidence,
3. every promoted safety or clarity claim needs positive and negative fixtures,
4. broad surfaces must stay outside Core until their promotion gate exists.
