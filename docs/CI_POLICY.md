# Vais Compiler CI Policy (Phase 0.F)

> Effective: when stability gate is wired into CI (target: end of Phase 0).
> Goal: never let a breaking change land without a regression test.

## Required checks (all must pass)

A pull request CANNOT merge unless every one of these passes on the
proposed merge commit:

1. **Core unit tests**:
   - `cargo test -p vais-codegen --lib` → 796/796 (or current count) green
   - `cargo test -p vais-types --lib` → 355/355 green
2. **Conformance suite**:
   - `cd compiler/tests/lang && ./run_lang_tests.sh` ≥ 95% green
   - Any new failure must be added to `STATUS.md` known-issues with
     reproduction code AND a tracking issue link.
3. **Hello world**:
   - `cd compiler/examples/hello_world_v2 && make check` exit 0
4. **Stdlib core**:
   - `cd compiler/std/tests && ./run.sh` (when 0.C lands) all green
5. **Workspace builds**:
   - `cargo build --workspace` (excluding experimental crates) succeeds

## PR description requirements

The PR body MUST include:

- **What changed**: 1-3 sentences describing the diff
- **Why**: link to issue, regression test, or `STATUS.md` line item
- **Verification**: paste of the local `./run_lang_tests.sh` output

If no `tests/lang/` test was added or modified, the PR description must
explain why (e.g., "internal refactor — no behavior change", with the
reviewer judging that claim).

## Bug-driven test discipline

Every reported bug **must** become a `tests/lang/` test before fix lands:

```
1. Open issue:   "match against narrow int produces invalid IR"
2. Reduce:       create tests/lang/03_match/match_narrow_int.vais
                 that reproduces it
3. Confirm red:  ./run_lang_tests.sh → reports FAIL on the new file
4. Fix compiler: smallest possible change
5. Confirm green: ./run_lang_tests.sh → all pass
6. Commit both:  the test AND the fix in the same PR
7. Update STATUS.md if the fix changes pass/fail counts
```

Anti-pattern (rejected at review):
- Fix without a regression test
- Test marked `#[ignore]` or `// TODO`
- "Internal cleanup" PRs that touch >5 production files without a test

## Conformance suite size targets

| Phase 0 milestone | Target |
|-------------------|--------|
| Kickoff | 30 tests |
| 0.B mid | 100 tests |
| 0.B end | 300 tests |
| 0.C end | + 8 stdlib modules × 30 assertions = ~240 stdlib assertions |
| 0.D end | + 12 hello-world examples |
| **v1.0 release** | **~600 total tests/assertions** |

## Public-facing badge

`README.md` displays:

```markdown
![tests](https://img.shields.io/badge/lang_tests-XXX/YYY_passing-color)
![hello](https://img.shields.io/badge/hello_world-N/M-color)
![stdlib](https://img.shields.io/badge/stdlib-K/L_modules-color)
```

These numbers are auto-updated by CI from `STATUS.md`. If they drift
from STATUS.md by more than 24h, CI fails.

## Release criteria (v1.0)

The project tags `v1.0.0` only when ALL of:

- ☐ Conformance suite ≥ 95% green (≥ 285/300)
- ☐ All hello-world examples pass
- ☐ All core stdlib modules' self-tests pass
- ☐ At least 3 external contributors have submitted PRs that landed
- ☐ `STATUS.md` has been accurate for 30 consecutive days
- ☐ One real-world project (vaisdb / vais-web / vais-server) has at
  least 1 test passing on top of the released compiler

## Honesty pledge

If CI is broken, the PR cannot merge. No "we'll fix CI later" workarounds.

If `STATUS.md` is wrong, opening a PR to correct it is **always
welcome and merged with priority**.
