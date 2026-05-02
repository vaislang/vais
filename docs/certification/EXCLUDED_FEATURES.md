# Vais Deferred and Experimental Features

## Purpose

This document removes unstable surfaces from the first correctness target. Exclusion does not mean deletion. It means the feature cannot be used to claim Core language correctness until it passes the promotion rule in `VAIS_CORE_V0.md`.

## Deferred Language Features

| Feature | Reason for exclusion | Promotion gate |
|---|---|---|
| broad implicit coercion | Repeated source of type/codegen mismatches. | Explicit invariant, negative fixtures, no new fallback path. |
| complex type inference | Current failures include unresolved `Var`/generic leaks. | Type-check completion invariant passes for fixtures. |
| integer truthy control-flow predicates | Accepted by the current full compiler for legacy/downstream compatibility, but excluded from Core style. | Dedicated core-mode or lint gate that rejects non-bool predicates without breaking ecosystem builds. |
| `?` error propagation | Cross-module and wrong receiver interactions are still unstable in downstream code. | Result fixture suite plus stage-specific negative tests. |
| trait objects / full vtables | Broad backend and ABI surface. | Dedicated design doc and call dispatch invariant. |
| advanced generics / HKT / ImplTrait | High risk for monomorphization and codegen erasure. | Separate generic certification gate. |
| macros | Expands the language before Core semantics are stable. | Macro expansion stage contract and hygiene tests. |
| async / await runtime semantics | Runtime and lowering complexity beyond Core. | MIR/runtime contract and deterministic fixtures. |
| closures beyond simple certified cases | Existing closure inference work is still active. | Closure parameter/return type invariant and fixtures. |
| first-class function pointers | Marked unsupported in existing safe subset. | New RFC and parser/type/codegen tests. |
| `drop` / auto-free semantics | Existing docs say drop calls are disabled. | Ownership/destructor design and run-time tests. |
| unsafe blocks and FFI-heavy patterns | Safety story is not Core-ready. | FFI safety contract and negative tests. |

## Deferred Syntax Forms

| Syntax | Reason | Replacement in Core |
|---|---|---|
| legacy context-dependent `E` for new enum/else fixtures | Ambiguous for agents and docs. | Use `EN` for enum and `EL` for else. |
| tuple structs and tuple variant multi-field binding | Existing docs mention parser/codegen gaps. | Use named struct fields or single-field variants. |
| match returning `str` in complex arms | Existing safe subset warns about phi type mismatch. | Use if/else or bind strings before match until certified. |
| `Vec<struct>[i].field` direct access | Existing safe subset lists erasure/field access hazards. | Bind indexed value to a temporary in non-Core code. |

## Deferred Stdlib and Runtime Domains

| Area | Reason | Promotion gate |
|---|---|---|
| networking and HTTP server stdlib | Broad platform/runtime surface. | Separate integration gate after Core. |
| websocket, oauth, yaml, GPU/OpenCL helpers | Not needed for compiler Core. | Package-specific certification. |
| async platform backends | Runtime-specific and difficult to prove early. | MIR/runtime design plus platform tests. |
| package registry and playground server | Product tooling, not compiler correctness. | Auxiliary service gate. |

## Experimental Compiler Crates

The `compiler/docs/CRATE_AUDIT.md` experimental tier remains outside Core v0: JIT, GPU, GC, JS codegen, hot reload, dynamic loading, profiler, registry server, playground server, tutorial, Python/Node bindings, query, testgen, supply-chain, and security crates.

These crates may continue to exist and build opportunistically, but they do not block Core certification unless explicitly promoted.

## Current Certification Exclusion Audit

`tests/core/certification_exclusions.tsv` is the machine-readable source of truth for ignored tests, plus any future partial markers, that still appear inside the canonical certification gate. `core_certification_exclusion_manifest_is_current` fails if:

- a new `#[ignore]` appears in the audited gate files without a manifest entry,
- a known ignore or partial marker is removed without updating the manifest,
- an ignore reason changes silently,
- `tests/core/mir_deferred.tsv` gains a deferred Core fixture.

The manifest may be empty when the audited quarantine surface is empty. This keeps temporary quarantine visible while preserving the narrower Core v0 proof boundary.

For the current dated pass/fail evidence, use `CURRENT_STATUS.md`. This file
defines what remains outside Core; it is not a substitute for a fresh gate run.

## Downstream Ecosystem

`lang/packages/vaisdb`, `lang/packages/vais-server`, and `lang/packages/vais-web` are promotion gates after Core. They are not the first proof of language correctness.

The order is:

1. Core v0 fixtures pass.
2. Type/codegen boundary invariants pass.
3. MIR structural validation passes for the lowered Core subset.
4. Reference interpreter and LLVM execution agree for Core run fixtures.
5. Downstream packages are reintroduced one at a time.

## Reintroduction Rule

To reintroduce an excluded feature, create a task with:

- a short design document,
- exact Core or non-Core status,
- positive and negative fixtures,
- required compiler stage,
- same-class audit command,
- rollback trigger if any existing Core fixture regresses.
