# Vais Public Status

Last verified: 2026-05-10

This document is the public wording boundary for the Vais repositories and
website. It separates the gates that are currently certified from broader
language, compiler, package, and ecosystem claims that are still experimental or
unverified.

## Certified Baseline

The current certified baseline is a core compiler and promoted-runtime
baseline. It is not a product-complete v1.0 release.

The latest local integrity pass covers:

- Core compiler freeze bundle: passed
- Downstream re-entry criteria: passed
- Final integrity gate: passed via `scripts/check-integrity.sh`
- Standard library package tests: `82/82`
- VaisDB package tests: `261/261`
- Backend package tests: `18/18`
- `std/http_client` runtime smoke: `15/15`
- TLS runtime smoke: `2/2`
- VaisDB runtime smoke: `34/34`
- Vais Server runtime smoke: `15/15`
- Vais Web runtime smoke: `61/77` with credential/network-gated cases skipped
- Vais Web unit tests: `390/390`
- Ecosystem package tests: `3272/3272`
- Vais Web full-build gate: `24/24`
- Package full-build gate: `2/2`
- Playground web mode/build gate: passed; Server-WASM remains API-compiled,
  and Preview remains a syntax/demo fallback only

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

## Public Wording Policy

Use gate-backed wording:

- "certified core compiler"
- "promoted runtime smoke gate"
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
