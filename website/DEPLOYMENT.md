# Vais Website Deployment

This document describes the current production deployment for `vaislang.dev`.

## Current Production Shape

The production site is a single GitHub Pages deployment assembled by
`.github/workflows/website.yml`.

| Surface | Source | Production path |
|---------|--------|-----------------|
| Website | `website/` | `https://vaislang.dev/` |
| Blog | `website/blog/` | `https://vaislang.dev/blog/` |
| Docs | `docs-site/` | `https://vaislang.dev/docs/` |
| Playground | `playground/` | `https://vaislang.dev/playground/` |
| Install scripts | `install.sh`, `install.ps1` | `https://vaislang.dev/install.sh`, `https://vaislang.dev/install.ps1` |

Subdomains such as `docs.vaislang.dev` and `play.vaislang.dev` are not the
canonical production targets unless DNS and workflow routing are explicitly
changed later.

## Trigger

The deployment workflow runs on:

- push to `main` for website, docs-site, playground, browser compiler, claim
  guard, installer, and workflow files
- manual `workflow_dispatch`

Pull requests validate the affected surfaces, but production only updates after
the PR is merged into `main` and the Pages deployment succeeds.

## Workflow Stages

The `Deploy Website` workflow performs these steps:

1. Check out the repository.
2. Run `node scripts/check-public-claims.mjs`.
3. Build the main website with `npm ci && npm run build` in `website/`.
4. Build multilingual docs with `bash docs-site/build.sh`.
5. Build and validate the playground browser compiler:
   - `npm run browser-compiler:build`
   - `npm run browser-compiler:check`
   - `npm run build`
6. Combine artifacts into `website/dist`:
   - `docs-site/book` -> `website/dist/docs`
   - `playground/dist` -> `website/dist/playground`
   - install scripts -> `website/dist/`
7. Upload `website/dist` with `actions/upload-pages-artifact`.
8. Deploy with `actions/deploy-pages`.

## Local Verification

Run these checks before merging public docs or site changes:

```bash
node scripts/check-public-claims.mjs
bash docs-site/build.sh

cd website
npm ci
npm run build

cd ../playground
npm install
npm run browser-compiler:build
npm run browser-compiler:check
npm run build
```

Use `npm run dev` in `website/` for local website iteration and `npm run dev`
in `playground/` for playground iteration.

## Claim Freshness Rules

Public performance and ecosystem claims must name their evidence boundary.

- Single-file compile-speed claims cite the current benchmark date and command.
- Runtime benchmark numbers remain historical/scoped until the runtime benchmark
  suite is refreshed on the current compiler.
- DB, server, and web claims must point to explicit gates or clearly say they
  are workbench/scoped evidence.
- The deployed website can lag local source until PRs pass review, merge into
  `main`, and the Pages workflow completes.

If a public page needs an emergency correction, update the source in this repo,
run the verification above, merge to `main`, and confirm the deployed content at
`https://vaislang.dev/`.
