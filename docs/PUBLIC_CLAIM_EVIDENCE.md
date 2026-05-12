# Public Claim Evidence Audit

Last audited: 2026-05-13

This document records whether high-risk public claims are reproducible from the
current `origin/main` tree or are integration evidence from the long-running
gate branch.

## Main-Reproducible Evidence

| Claim area | Evidence in `origin/main` |
|---|---|
| Public wording boundary | `node scripts/check-public-claims.mjs` |
| Website/docs/playground deployment | `.github/workflows/website.yml` |
| Playground mode boundary | `scripts/check-playground-mode-contract.mjs` |
| Browser-JS playground smoke | `scripts/check-browser-compiler-gate.mjs` |

## Integration Evidence Pending Main Port

The following counts are public evidence from `codex/ssr-json-grammar-gate`
and the local multi-repository workspace, but the aggregate runner and product
fixtures are not yet present on `origin/main`.

| Claim area | Integration evidence | Main status |
|---|---:|---|
| Aggregate integrity runner | `scripts/check-integrity.sh` | Pending main port |
| Std package codegen | `82/82` | Pending aggregate gate port |
| VaisDB package codegen | `261/261` | Pending aggregate gate port |
| Backend smoke | `18/18` | Pending aggregate gate port |
| HTTP client runtime | `15/15` | Pending aggregate gate port |
| TLS runtime | `2/2` | Pending aggregate gate port |
| VaisDB runtime | `34/34` | Pending aggregate gate port |
| vais-server runtime | `20/20` | Pending aggregate gate port |
| Vais Web runtime | `61/77` | Pending ecosystem gate port |
| Vais Web unit | `390/390` | Pending ecosystem gate port |
| Vais Web packages | `3272/3272` | Pending ecosystem gate port |
| Vais Web full-build | `24/24` | Pending ecosystem gate port |
| Cross-package schema | `15/15` | Pending fixture + runner port |
| Multi-domain product schema | `9/9` | Pending fixture + runner port |
| Package full-build | `2/2` | Pending aggregate gate port |

## Required Resolution

Public pages may cite these numbers only as evidence-scoped claims. They must
not imply that the full aggregate gate is reproducible from `origin/main` until
the runner, fixtures, and ecosystem package gates are ported and passing there.
