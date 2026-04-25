# Vais Language Phase 0 — Compiler Stabilization Roadmap

> **Created**: 2026-04-25 — after Phase 17 Wave 1-4a partial (~258 codegen sites migrated, vaisdb 0/14 tests passing).
> **Mission**: Make `vaisc` produce LLVM IR that `clang` accepts, **always**, for any well-typed Vais program.
> **Honest precondition**: Until this is done, every higher-level project (vaisdb, vais-web, vais-server) is built on quicksand.

---

## Part I — Diagnosis

### What Phase 17 actually proved

65 iterations of compiler invariant work uncovered a consistent pattern:

> **Every layer of fix surfaces the next hidden bug** — not because Phase 17's work was wrong, but because the bugs are not isolated. They are **integration bugs** between language features: enum × generic × match × ABI × narrow integer × ownership.

Concrete evidence (this session alone, just from `test_page_manager.vais`):

| # | Bug | Feature interaction |
|---|-----|---------------------|
| 1 | match default → `null` for any phi type | match × phi |
| 2 | `vec[i] = struct_value` stores ptr instead of value | Vec × index assign × struct value |
| 3 | 4-byte Named struct stored as `i32` | generic specialization × byte sizing |
| 4 | match arm phi width mismatch | match × narrow integer |
| 5 | Vec → slice not auto-coerced | ABI × generic |
| 6 | Specialized enum lookup misses (`%Unknown`) | match × generic × enum |
| 7 | Enum payload of >8B struct decoded as i64 | enum × struct × ABI |
| 8 | `slice.to_vec()` builtin unresolved | stdlib × method dispatch |

Each is a 1-line fix in isolation. **Together, they form a class** — the compiler doesn't have a coherent story for "what type does each SSA register actually hold" across these interaction points. Wave 4 catch-all removal targets exactly this, but isn't enough on its own — coverage of the catch-all is currently ~80%, and the missing 20% is precisely the integration paths.

### What dependencies are stacked atop quicksand

The repository contains 28 compiler-related crates. Most assume the core works:
- **`vais-lsp`, `vais-dap`** — IDE support assuming TC and codegen are reliable
- **`vais-jit`, `vais-codegen-js`** — alternative backends assuming the type system is stable
- **`vais-bindgen`, `vais-python`, `vais-node`** — FFI layers assuming ABI is locked
- **`vais-registry-server`, `vais-supply-chain`, `vais-playground-server`** — distribution infrastructure assuming there's something to distribute
- **`vais-gpu`, `vais-gc`** — runtime additions assuming GC-less core is solid

And the stdlib (`compiler/std/`) is **41,772 lines** including websocket, yaml, base64, http_server. Most stdlib modules have never been individually verified.

This is the inverted pyramid problem. Fixing it is Phase 0's job.

### Trust diagnostics — what would prove the compiler is stable?

1. `cargo test -p vais-codegen --lib` passes — *currently does* (796/796), but tests cover IR-string matching, not "is the IR valid LLVM".
2. **Every example in `compiler/examples/` builds and runs** — *unverified*.
3. **`compiler/std/` self-test** — each module independently linked + executed — *not done*.
4. **Hello world programs in 50 representative shapes** — *no such suite exists*.
5. **Self-hosting: `vaisc` compiled by `vaisc`** — currently we use `cargo build` (Rust). Self-hosting is the gold standard.

Phase 0 builds these one by one.

---

## Part II — Phase 0 Sub-Phases

### 0.A: Reduce surface area (1-2 weeks)

Before stabilizing anything, we need to know **what core means**.

**Action**:
1. Audit all 28 crates. Tag each as `core` (must work) / `auxiliary` (nice-to-have, can be broken) / `experimental` (tag-locked, may not even compile).
2. Set up CI that *only* tests `core` crates. Auxiliary tagged `--allow-broken`.
3. Move `experimental` crates to a `crates/experimental/` subdirectory with a top-level disclaimer.

**Recommended core crates** (everything else is auxiliary or experimental):
- `vais-lexer` — tokens
- `vais-parser` — AST
- `vais-ast` — AST types
- `vais-types` — TC + ResolvedType
- `vais-codegen` — LLVM IR emit
- `vais-mir` — IR (if used in codegen path)
- `vaisc` — driver

That's **7 crates**. Everything else postponed to v1.0+.

**Stdlib reduction**: keep only modules that are **transitive dependencies of hello world**:
- `core` (option, result, panic)
- `vec`, `hashmap`, `string`, `bytes`, `bytebuffer`
- `file`, `net` (minimal)
- `sync` (Mutex, RwLock)

That's ~10 modules. Move the rest (websocket, yaml, http_server, async_*, ...) to `compiler/std-experimental/` until language is stable.

**Exit criterion**: `cargo build --workspace` builds only the 7 core crates by default. README has a "Stability tiers" section.

### 0.B: Fundamental conformance test suite (3-4 weeks)

The single most important deliverable. **Without this, every fix is anecdotal.**

**Structure** — `compiler/tests/lang/`:

```
tests/lang/
├── 01_primitives/
│   ├── int_widths.vais          # i8/i16/i32/i64 literals + arith
│   ├── float_widths.vais        # f32/f64
│   ├── bool_logic.vais
│   ├── char_byte.vais
│   └── ...
├── 02_control_flow/
│   ├── if_else_value.vais
│   ├── while_loop.vais
│   ├── for_range.vais
│   ├── for_collection.vais
│   ├── break_continue.vais
│   ├── return_early.vais
│   └── ...
├── 03_match/
│   ├── match_int_literal.vais
│   ├── match_int_range.vais
│   ├── match_string.vais
│   ├── match_enum_unit_variant.vais
│   ├── match_enum_tuple_variant.vais
│   ├── match_enum_struct_variant.vais
│   ├── match_with_guard.vais
│   ├── match_default_arm.vais
│   ├── match_phi_narrow_int.vais        # ← bug 4
│   ├── match_phi_default_null.vais      # ← bug 1
│   └── ...
├── 04_struct/
│   ├── struct_simple.vais
│   ├── struct_nested.vais
│   ├── struct_field_assign.vais
│   ├── struct_4_bytes.vais             # ← bug 3
│   ├── struct_value_in_vec.vais        # ← bug 2
│   └── ...
├── 05_enum/
│   ├── option.vais
│   ├── result.vais
│   ├── result_with_struct_payload.vais  # ← bug 7
│   ├── result_specialized_match.vais    # ← bug 6
│   └── ...
├── 06_generic/
│   ├── fn_generic.vais
│   ├── struct_generic.vais
│   ├── method_on_generic.vais
│   ├── generic_in_generic.vais
│   ├── specialized_struct_lookup.vais
│   └── ...
├── 07_collections/
│   ├── vec_basic.vais
│   ├── vec_struct_elem.vais
│   ├── vec_index_assign.vais            # ← bug 2
│   ├── slice_from_vec.vais              # ← bug 5
│   ├── hashmap_basic.vais
│   └── ...
├── 08_strings/
│   ├── str_literal.vais
│   ├── str_concat.vais
│   ├── str_interp.vais
│   ├── str_methods.vais  (.len, .contains, .starts_with)
│   └── ...
├── 09_traits/
│   ├── trait_define.vais
│   ├── trait_impl.vais
│   ├── trait_object.vais
│   ├── trait_generic_bound.vais
│   └── ...
├── 10_ffi/
│   ├── extern_c.vais
│   ├── ptr_cast.vais
│   └── ...
├── 11_async/  (optional — defer if too unstable)
└── 99_integration/
    ├── linked_list.vais
    ├── binary_search_tree.vais
    ├── json_parser.vais
    └── word_count.vais
```

**Each test file**:
- Self-contained (no external deps beyond core stdlib)
- Has exactly one or two `assert_eq` / `assert_true` calls — fail loud
- Header comment: `# CHECKS: <one-line summary of what's being verified>`
- Header comment: `# REGRESSION: <iter # or commit hash>` if added in response to a bug

**Test runner** (`compiler/tests/run_lang_tests.sh`):
1. For each `.vais` file, run `vaisc build $file --emit-ir -o /tmp/out.ll`
2. Pipe through `clang -O0 -o /tmp/out` — if clang rejects, FAIL
3. Run `/tmp/out` — if exit code != 0 or `Assertion failed` in stderr, FAIL
4. Aggregate: `M passed / N total`

**Bootstrap target**: 50 tests by end of week 1. 150 by end of week 2. 300+ by end of week 4 (full coverage of language features).

**Exit criterion**: `./run_lang_tests.sh` shows ≥ 95% green. The remaining 5% are documented in `STATUS.md` as "known issues" with reproduction code.

### 0.C: Stdlib self-test (2 weeks, parallel with 0.B)

For each *core* stdlib module (`vec`, `hashmap`, `string`, `bytes`, `bytebuffer`, `file`, `net`, `sync`):

1. Write `compiler/std/tests/test_<module>.vais` exercising **every public method**.
2. Run via `vaisc build` → link → execute.
3. Each assertion failure is a P0 bug — fix in either stdlib or compiler.

**Standard for each module**: ≥ 30 assertions covering happy path + edge cases (empty, single element, max size, ownership transfer, error paths).

**Exit criterion**: `compiler/std/tests/` has 8+ files, each running 30+ assertions, all green.

### 0.D: Hello world conformance (1 week)

A **public-facing** smoke test. `compiler/examples/hello/` has 12 programs:

| # | Program | Demonstrates |
|---|---------|--------------|
| 1 | `01_print.vais` | `print()` works |
| 2 | `02_args.vais` | command-line args |
| 3 | `03_arith.vais` | integer + float arithmetic |
| 4 | `04_strings.vais` | string literals + concat |
| 5 | `05_collections.vais` | Vec + HashMap basics |
| 6 | `06_struct.vais` | struct + methods |
| 7 | `07_enum_match.vais` | Option/Result + match |
| 8 | `08_generic.vais` | generic fn + specialization |
| 9 | `09_traits.vais` | trait definition + dyn dispatch |
| 10 | `10_file_io.vais` | read + write file |
| 11 | `11_concurrency.vais` | spawn + join (or basic Mutex) |
| 12 | `12_error.vais` | `?` operator + Result chains |

Each program is **< 30 lines**. `compiler/examples/hello/Makefile` builds + runs all 12. CI fails if any breaks.

**Exit criterion**: `make -C compiler/examples/hello check` runs all 12 with exit 0. Output is stable (golden file diff).

### 0.E: Compiler self-hosting (4-6 weeks)

The ultimate trust test: **vaisc can compile vaisc**.

This requires:
1. Rewriting `vaisc` driver in Vais (currently Rust).
2. Verifying it produces an `.ll` file identical (or behaviorally equivalent) to the Rust version.
3. CI: Rust-built `vaisc` compiles Vais-version `vaisc.vais` to `vaisc-stage1`. `vaisc-stage1` compiles itself to `vaisc-stage2`. Diff `stage1` vs `stage2` outputs — must be zero diff (fixpoint).

Reality check: this is a multi-month commitment. **0.E is post-v1.0 unless explicitly prioritized**. For v1.0 we accept "Rust-hosted" as the supported configuration.

### 0.F: Stability gate (continuous)

Once 0.A–0.D are done:

**Pre-merge CI policy**:
1. `cargo test -p vais-codegen --lib` 796/796
2. `cargo test -p vais-types --lib` 355/355
3. `./run_lang_tests.sh` ≥ 95% green
4. `compiler/std/tests/` all green
5. `compiler/examples/hello` all green
6. PR description must explain WHY a feature/test was added — no "internal cleanup" commits without a regression test reference

**Public README badge**: live count of "language tests passing" + last commit hash.

**Bug discipline**: every reported bug → minimized test case in `tests/lang/` → fix until green. Never `// TODO`, never `#[ignore]`.

---

## Part III — Concrete Schedule

| Week | Focus | Deliverable |
|------|-------|-------------|
| 1 | 0.A audit | core/auxiliary/experimental tagging + tier 1 CI |
| 2 | 0.B foundation | `tests/lang/01_primitives` + `02_control_flow` (50 tests) |
| 3 | 0.B match + struct | `03_match` + `04_struct` + `05_enum` (100 tests) |
| 4 | 0.B generic + collections | `06_generic` + `07_collections` (200 tests) |
| 5 | 0.B traits + integration | `09_traits` + `99_integration` (300 tests, target 95% green) |
| 6 | 0.C stdlib | `vec`, `hashmap`, `string` self-tests |
| 7 | 0.C stdlib | `bytes`, `bytebuffer`, `file`, `net`, `sync` self-tests |
| 8 | 0.D hello world + 0.F gate | 12 examples + CI badge live |
| 9-10 | Buffer / catch-up | Fix red tests, refactor compiler hot spots |
| 11+ | 0.E self-hosting (optional) or v1.0 release | |

**v1.0 release criteria**:
- 0.A done
- 0.B ≥ 95% green
- 0.C all green (8+ modules)
- 0.D all 12 examples green
- 0.F policy active (CI enforces)
- README has honest STATUS.md + roadmap to vaisdb / vais-web / vais-server
- One external user has tried hello world and reported success (or a real bug we then fixed)

This is **2-3 months of disciplined work**, not 6+ months of ad-hoc.

---

## Part IV — Process Lessons from Phase 17

These are mistakes we will not repeat:

1. **No more "ad-hoc fix per error"** without a regression test entering `tests/lang/`.
2. **No more "wave migration"** that touches 100+ sites at once. Each PR ≤ 5 sites or 1 minimal-test fix.
3. **No more "linked count" as a metric** — it conflates real progress with measurement noise. Use `tests/lang/` pass rate.
4. **No more deferring cascade-trigger sites**. They are the actual bugs. Fix or document permanent limitation in STATUS.md.
5. **No more "code-only" commits**. Every commit must touch either a `tests/lang/*.vais` file or `STATUS.md`.

---

## Part V — Why this leads to trust

A user who finds the project a year from now will see:

```
$ git clone https://github.com/vaislang/vais
$ cd vais
$ make test
running 384 language tests... 384 passed.
running 8 stdlib tests... 8 passed.
running 12 hello-world examples... 12 passed.
$ ./compiler/examples/hello/01_print
Hello, World!
$ cat compiler/STATUS.md
# Vais language status as of 2026-XX-XX
- Language v1.0 stable: ✅
- Self-hosting: 🟡 in progress
- Stdlib coverage: 8 modules production-ready
- Known issues: 3 (see issues #N, #M, #K)
- Contributors: 1
```

That's a project I would trust to dependent on. Phase 0 is exactly the work to make that screenshot real.

---

## Part VI — Immediate first action

Pick ONE bug from this session's nine layers. Write the minimal `.vais` test that reproduces it. Confirm it's red. Fix it. Confirm green. Commit both test + fix together.

Suggested first bug: **#1 — match default arm phi null literal**. It's the smallest fix and we already understand it.

Suggested filename: `compiler/tests/lang/03_match/match_phi_default_zero.vais`

```vais
# CHECKS: match expression default arm uses type-correct zero, not null
# REGRESSION: Phase 17 iter 64 (commit 7c3aed52)

F main() -> i64 {
    x: i32 := mut 5
    result: i64 := mut M x {
        1 => 100i64,
        2 => 200i64,
        _ => 0i64,
    }
    assert_eq(result, 0)
    R 0
}
```

Make this red (revert the fix), confirm red, re-apply fix, confirm green, commit. **That's the new working rhythm.**
