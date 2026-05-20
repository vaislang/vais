# Public Claim Evidence Audit

Last audited: 2026-05-13

This document records whether high-risk public claims are reproducible from the
current `origin/main` tree or are integration evidence from the long-running
gate branch.

## Main-Reproducible Evidence

| Claim area | Evidence in `origin/main` |
|---|---|
| Public wording boundary | `node scripts/check-public-claims.mjs` |
| Main-scoped integrity runner | `bash scripts/check-integrity.sh` |
| Website/docs/playground deployment | `.github/workflows/website.yml` |
| Playground mode boundary | `scripts/check-playground-mode-contract.mjs` |
| Browser-JS playground smoke | `scripts/check-browser-compiler-gate.mjs` |
| `vaisc emit-ts` schema declaration surface | `cargo test --locked -p vaisc --test emit_ts_skeleton --test emit_ts_exhaustiveness` |
| VaisDB aggregate main full build | Clean-cache `vaisc build ../lang/packages/vaisdb/src/main.vais`; `36/36` LLVM/object cache artifacts; build log contained no `[IR verify]`, compiler warnings, undefined symbols, or duplicate symbols |

## Main Fixtures With Local Workspace Requirements

These gates are present in the compiler tree and passed locally on
2026-05-13. They require a built `vaisc` binary and the sibling
`lang/packages/vais-web` TypeScript toolchain.

| Claim area | Evidence | Scope |
|---|---:|---|
| Cross-package schema | `15/15` | `tests/empirical/cross_package_schema/tests/gate.sh positive` and `negative`; `.vais` consumers check/run natively after schema concatenation, TS consumer type-checks generated `.d.ts` |
| Multi-domain product schema | `9/9` | `tests/product/multi_domain_schema/tests/gate.sh`; DB/server consumers type-check through real package APIs, web consumer type-checks generated `.d.ts` plus real `@vaisx/db` sources |

## Scoped Integration Evidence

The following counts are public evidence from `codex/ssr-json-grammar-gate`,
the local multi-repository workspace, or scoped package gates. They may be cited
only with their scope. The compiler main tree now has a clean-cache VaisDB
aggregate full-build smoke, but the full DB/server/web runtime aggregate runner
is not yet a single `origin/main` gate.

| Claim area | Integration evidence | Main status |
|---|---:|---|
| Full ecosystem runtime aggregate runner | `scripts/check-integrity.sh` on the gate branch | Still pending a single DB/server/web runtime main gate |
| Std package codegen | `82/82` | Scoped integration evidence |
| VaisDB package codegen | `261/261` | Scoped package evidence; aggregate main full-build smoke now separately passes |
| Backend smoke | `18/18` | Scoped integration evidence |
| HTTP client runtime | `15/15` | Scoped integration evidence |
| TLS runtime | `2/2` | Scoped integration evidence |
| VaisDB runtime | `34/34` | Scoped runtime evidence; not a product-complete SQL/vector/FTS claim |
| vais-server runtime | `20/20` | Scoped runtime evidence |
| Vais Web runtime | `61/77` | Scoped runtime evidence |
| Vais Web unit | `390/390` | Scoped package evidence |
| Vais Web packages | `3277/3277` | Scoped package evidence |
| Vais Web full-build | `24/24` | Scoped full-build evidence |
| Package full-build | `2/2` | Scoped package evidence |

## Required Resolution

Public pages may cite the VaisDB `36/36` aggregate full-build smoke as
main-reproducible evidence. All other counts above remain evidence-scoped and
must not imply product completeness or a single full ecosystem runtime gate
until the DB/server/web runtime and ecosystem package gates are ported and
passing on `origin/main`.
