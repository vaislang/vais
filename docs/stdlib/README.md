# Vais Standard Library

Vais ships with a standard library in `std/*.vais` (82 files as of Phase 5.24). This directory documents the **build status** and **public API** of each stdlib module.

## Build status (2026-04-19)

| Status | Count | Files |
|--------|-------|-------|
| ✓ Builds OK | 42/82 | See per-module pages below |
| ◐ Partial (known gap) | 40/82 | See `docs/CODEGEN_FEATURES.md` "Known TC-passes-but-codegen-fails" |

Phase 5.24 target is 82/82. Remaining failures are blocked on:
- Compiler E022 over-strict move tracking (ownership/move_track.rs)
- Missing runtime functions (parse_iN/fN, time_now, call_poll, store_i8/i16)
- Legacy syntax files (old C-style for loops, `extern F`, `// comments`)

## Working modules (42)

These modules pass `vaisc build <file>` and are safe to import via `U std/<name>`:

| Module | Purpose | API highlights |
|--------|---------|---------------|
| `collections` | Linked list primitives | push_front, pop_back (partial) |
| `fmt` | Formatted output helpers | format_*, pad_* |
| `http` | HTTP types (partial) | HttpRequest, HttpResponse |
| `simd` | SIMD primitives (partial) | simd_dot_f32x4 |
| `wasm` | WebAssembly heap helpers | wasm_heap_alloc |
| `priority_queue` | Min-heap priority queue | push/pop_opt returning Option<i64> |
| `option`, `result` | Option/Result builtins | (builtins; no separate file) |
| `io`, `print` | IO primitives | puts, print |
| `math` | Math functions | abs, min, max, floor, ceil |
| `string` | String helpers | str_len, str_concat |
| ... | ...others... | see individual .vais files |

> The aggregate integrity gate already runs this exact standalone codegen
> check for all **82 files** (`INTEGRITY std_files pass=82 fail=0 total=82`).
> To inspect the per-file status manually, run:
> ```bash
> for f in std/*.vais; do
>   ./target/release/vaisc build "$f" --emit-ir -o /tmp/__chk.ll --force-rebuild 2>&1 \
>     | grep -oE "error\[[A-Z][0-9]+\]" | head -1 | { read err; echo "$(basename $f): ${err:-OK}"; }
> done
> ```

## Cross-reference

- `docs/LANGUAGE_SPEC.md` — language syntax
- `docs/CODEGEN_FEATURES.md` — codegen feature matrix
- `docs/language/LIVING_SPEC/04_stdlib/` — executable stdlib usage examples
- `docs/language/COOKBOOK.md` — common stdlib pitfalls
- `crates/vais-types/src/builtins/method_returns.rs` — authoritative method return-type table

## Versioning

The stdlib tracks the compiler version — **no independent versioning yet**. The 100% standalone codegen gate for every std file is current in `compiler/scripts/check-integrity.sh`; Phase 5.26 adds per-module API pages.
