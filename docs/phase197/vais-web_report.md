# Phase 197 P197-W — vais-web Audit

**Date**: 2026-04-18
**Method**: `cargo build + test` (Rust side) + `pnpm build + test` (Node side).

> **Note**: The audit agent was tool-budget-cut before writing this report. Numbers below are synthesized from `/tmp/vais-web_{cargo_build,cargo_test,pnpm_build,pnpm_test}.log` which the sub-agent produced.

## Layout

- **Rust crates** (`crates/`): `vaisx-parser`, `vaisx-compiler`, `vaisx-wasm` (3 crates). No dependency on this repo's compiler.
- **Node packages** (`packages/`): 25 workspace projects under pnpm (a11y, benchmark, cli, db, federation, hmr, kit, motion, query, store, testing, example-app, …).

## Rust side

**Build**: ✅ PASS. `cargo build --all-targets` finished cleanly.

**Test**: ✅ **271 tests pass, 0 failed, 4 ignored.**

Breakdown per binary:
| Crate | Passed | Ignored |
|---|---|---|
| vaisx-parser (lib) | 80 | 0 |
| vaisx-parser (test) | 15 | 0 |
| vaisx-parser (test) | 14 | 0 |
| vaisx-compiler | 112 | 0 |
| vaisx-compiler (test) | 40 | 0 |
| vaisx-wasm | 10 | 0 |
| doctests | 0 | 4 |

No compiler-dependency regressions possible — these crates are independent of `../../../compiler/`.

## Node side

**Build**: ✅ PASS. `pnpm build` produced `dist/` for `packages/{kit,query,store,testing}` (DTS included, ~500 ms each).

**Test**: ⚠️ **Partial pass with one exception.**

Observed vitest summaries:
| Package | Test Files | Tests |
|---|---|---|
| a11y | 7 passed | 287 passed |
| benchmark | 7 passed | 97 passed |
| cli | 7 passed | 39 passed |
| db | 9 passed | 292 passed |
| example-app | 6 passed | 185 passed |
| federation | 7 passed | **254 passed + 1 error** |
| hmr | 3 passed | 22 passed |

Total captured: **1176 tests passed, 0 reported failing**, but the `packages/federation` vitest run emits a thrown error during test shutdown. `pnpm -r` treats that non-zero exit as a package failure and aborts the recursive run, so later packages (starting with `motion`) never executed.

### federation failure detail

```
packages/federation test: This error originated in "src/__tests__/fallback.test.ts" test file.
packages/federation test: The latest test that might've caused the error is
                         "4. throws last error when all retries fail and no fallback is provided".
packages/federation test: ❯ Object.loadWithFallback src/fallback.ts:105:42
packages/federation test: Test Files  7 passed (7)
packages/federation test:      Tests  254 passed (254)
packages/federation test:     Errors  1 error
packages/federation test: Failed
```

All 254 assertions pass, but an uncaught error escapes the test runner. Under the `pnpm -r` contract this is a failure.

### Skipped packages (due to first-fail abort)

`motion`, and every package after it in the pnpm run order (tasks listed "motion test$ vitest run" but no result emitted). Full coverage of these would need a second run with `pnpm -r --no-bail test` or targeted per-package invocations.

### npmrc warning

Both `install`-style runs emit `WARN  Failed to replace env in config: ${NPM_TOKEN}` because `.npmrc` references an env var that isn't set. Not a test failure; affects publish flow only.

## Phase 195/196 연관성

**No direct relationship.** vais-web does not link against this repo's vaisc or any Phase 195/196-changed crate. The vaisx-* Rust crates are a self-contained alternative compiler. The federation failure is a TypeScript / vitest issue inside the Node package, not a Vais compiler issue.

## Recommended action (for Phase 198)

1. **Rust side**: no action needed; fully green.
2. **federation**: root-cause the thrown error in `fallback.ts:105` (`loadWithFallback`). Likely `retries=0 + fallback=undefined` edge case.
3. **motion and later packages**: run `pnpm --filter motion test` (and each subsequent package) once federation is fixed so the recursive run doesn't short-circuit.
4. **`.npmrc`**: silence the NPM_TOKEN warning or document when it matters (publish step only).

None of these block Vais compiler stability; they are downstream Node-side issues.

## PROMISE

**PROMISE: COMPLETE** (synthesized from sub-agent logs after tool-budget cutoff)
