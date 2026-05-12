# Vais Public Status

Last verified: 2026-05-13

This document is the public wording boundary for the Vais repositories and
website. It separates the gates that are currently certified from broader
language, compiler, package, and ecosystem claims that are still experimental or
unverified.

## Certified Baseline

The current certified baseline is a core compiler and promoted-runtime evidence
baseline, with reproducibility scope split as follows.

The current public baseline has two evidence tiers:

1. Main-reproducible gates that run from the current `origin/main` tree.
2. Integration evidence gathered on the long-running
   `codex/ssr-json-grammar-gate` branch and the local multi-repository
   workspace.

It is not a product-complete v1.0 release.

The current `origin/main` tree directly enforces:

- Public claim guard: `node scripts/check-public-claims.mjs`
- Website, docs, and playground GitHub Pages build/deploy workflow
- Playground web mode/build gate: passed; Server-WASM remains API-compiled,
  and Preview remains a syntax/demo fallback only
- Browser-JS playground smoke gate: passed for parser + JavaScript codegen
  compile/execute in the browser; this is not a complete browser-only language
  implementation claim

The following counts are integration evidence from
`codex/ssr-json-grammar-gate` and the local workspace as of 2026-05-12. They
are public evidence, but are not yet reproducible from `origin/main` until the
aggregate integrity runner, product fixtures, and ecosystem runtime gates are
ported to main:

- Core compiler freeze bundle: passed
- Downstream re-entry criteria: passed
- Aggregate integrity runner: pending main port
- Standard library package tests: `82/82`
- VaisDB package tests: `261/261`
- Backend package tests: `18/18`
- `std/http_client` runtime smoke: `15/15`
- TLS runtime smoke: `2/2`
- VaisDB runtime smoke: `34/34`
- Vais Server runtime smoke: `20/20`
- Vais Web runtime smoke: `61/77` with credential/network-gated cases skipped
- Vais Web unit tests: `390/390`
- Ecosystem package tests: `3272/3272`
- Vais Web full-build gate: `24/24`
- Package full-build gate: `2/2`
- Cross-package schema gate: `15/15`
- Multi-domain product schema gate: `9/9`

## Public Non-Claims

Do not present the project as product-complete, fully production-ready, or v1.0
released unless a later release gate explicitly certifies that scope.

The current baseline does not certify:

- Every language feature as complete
- Complete enum/generic/lifetime semantics beyond the current tested surface
- Complete JavaScript or WASM target execution
- Complete browser-only playground compilation and execution
- Complete JSON grammar validation across every parser path
- Arbitrary server middleware dispatch and response transforms
- Broad external network reliability outside the promoted local smoke gates
- Product-complete VaisDB, Vais Server, or Vais Web behavior
- Complete API documentation for every standard-library module
- Main-branch reproducibility for the full aggregate integrity gate until the
  pending main-port work lands

## Public Wording Policy

Use evidence-scoped wording:

- "certified core compiler"
- "promoted runtime smoke gate"
- "integration evidence"
- "pending main port"
- "source-build baseline"
- "experimental target"
- "design target"
- "implementation surface"

Avoid broad wording unless a named gate backs it:

- "production-ready"
- "complete"
- "full-featured"
- "full toolchain"
- "v1.0 release"
- "all targets supported"

If a page makes a capability claim, it should name the gate, test count, or
scope that supports the claim.

The repository enforces the highest-risk public wording boundaries with
`scripts/check-public-claims.mjs`; update that guard when promoting a non-claim
to a certified gate.
