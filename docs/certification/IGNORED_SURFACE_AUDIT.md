# Ignored And Deferred Surface Audit

Date: 2026-05-01
Re-evaluation cadence: every 6 months. Next due: **2026-11-01**.
Trip-wire threshold: **130 ignored matches** (current baseline 111). Any
single quarter that crosses 130 triggers an out-of-cadence re-audit.

## Scope

This audit classifies ignored tests outside the canonical Core certification
gate. It does not promote or unignore tests. Its purpose is to make sure ignored
workspace tests are not mistaken for hidden Core failures.

## Core Result

Canonical Core certification has no ignored quarantine at this date:

- `tests/core/certification_exclusions.tsv` is header-only.
- `tests/core/mir_deferred.tsv` is header-only.
- `core_certification_exclusion_manifest_is_current` passes.
- `compiler_syntax` integrity reports `218 passed; 0 failed; 0 ignored`.

Therefore, the ignored tests below are outside the current Core proof boundary.
They may still matter for future product readiness, self-hosting, robustness, or
platform coverage, but they do not block the current Core compiler gate.

## Workspace Scan

Raw search:

```bash
rg -n '#\[ignore' crates tests -g '*.rs'
```

The scan found 111 textual matches. That number includes real ignored tests plus
test harness code that parses `#[ignore]`, so it is a triage index rather than a
certification count.

## Classification

| Class | Files | Current classification | Promotion rule |
|---|---|---|---|
| Core audit harness internals | `crates/vaisc/tests/core_certification.rs`, `crates/vaisc/tests/integrity/compiler_stages.rs` | Harness code/comments, not ignored Core fixtures. | Keep covered by `core_certification_exclusion_manifest_is_current`. |
| Self-hosting and bootstrap standalone gaps | `selfhost_clang_tests.rs`, `bootstrap_tests.rs`, `selfhost_mir_tests.rs`, `selfhost_stdlib_tests.rs` | Non-Core self-hosting/product-readiness surface. Some entries need full module context or clang. | Promote only through a dedicated self-hosting certification drive, not Core RC. |
| Cross-verify corpus | `cross_verify_tests.rs` | On-demand broad semantic comparison corpus. | Promote selected fixtures one at a time into Core or strict MIR if they match Core constructs. |
| Scale/stress/endurance | `scale_test.rs`, `stress_tests.rs`, `endurance_tests.rs`, `examples_fresh_rebuild.rs` | Long-running or expensive gates. | Keep on-demand unless a small deterministic case exposes a Core invariant. |
| Parser and recursion robustness | `vais-parser/tests/negative_tests.rs`, `vais-parser/tests/fuzz_tests.rs`, `vais-macro/src/expansion.rs`, `memory_safety_tests.rs` | Robustness/fuzzing risk, not current Core semantic proof. Stack overflow notes are real technical debt. | Promote bounded minimal cases after parser depth/error-recovery policy is defined. |
| Optional runtime/platform dependencies | `phase33_integration_tests.rs`, `lto_tests.rs`, platform-specific e2e ignores | Require native runtime libraries, OS-specific constants, or optional optimization behavior. | Promote under platform/runtime gates after Core RC. |
| Non-Core language/product gap | `e2e/phase134_string.rs` ignored single-file std import gap | Harness/source-layout gap outside Core. | Promote only if the fixture is made deterministic and aligned with Core or a named std gate. |

## Risks To Track

These ignored areas are not Core blockers today, but they should not disappear
from planning:

- Parser stack/depth behavior: robustness issue for hostile or generated input.
- Self-hosting module-context gaps: future compiler self-hosting proof cannot
  depend on standalone ignored tests.
- LTO/platform runtime ignores: product release gates need OS-specific
  accounting.
- Cross-verify corpus: useful source of future semantic fixtures, but too broad
  to import wholesale into Core.

## Stop Rules

Stop and update the Core certification manifest if:

- a `#[ignore]` is added to any file audited by
  `core_certification_exclusion_manifest_is_current`,
- `tests/core/mir_deferred.tsv` gains an entry,
- an ignored workspace test is cited as evidence of a Core guarantee,
- a broad ignored fixture is unignored without adding a narrower invariant gate
  or without running the same-class regression tests.

## Re-audit Triggers

This audit is intentionally a snapshot of one date. Re-audit (rewrite the
classification table and the date stamp at the top) when **any** of the
following hold:

1. The 6-month cadence date passes (next: 2026-11-01).
2. The raw `rg -n '#\[ignore' crates tests -g '*.rs'` count crosses the
   trip-wire threshold of 130 matches (current baseline 111).
3. A class in the table moves between tiers (e.g. a self-hosting ignore
   becomes a Core-blocking failure).
4. A new top-level crate is added that introduces ignored tests not yet
   classified here.
5. The Core certification manifest gains or loses an entry, since the
   "outside Core" classification depends on what `Core` means today.

Re-audit means: re-run the raw scan, re-classify, update the
`Classification` table, and update the date stamp + cadence header.
`core_certification_exclusion_manifest_is_current` does **not** by itself
trigger this audit — it polices the audited gate files only, not workspace
expansion.

## Next Step

The next Core RC slice should be strict MIR promotion. Pick the smallest Core
construct not yet represented beyond the current strict subset, add the MIR
lowering/interpreter contract first, and then run the full certification gate.
