# Vais Worklog

## 2026-06-14

- Vais-only source surface enforced.
- Public compiler input is `.vais`.
- Removed wrapper tools and non-Vais gates.
- Updated README, ROADMAP, AGENTS, language reference, examples README, prelude notes, and self-host notes to current Vais status.
- Renamed temporary test sources to `.vais`.
- Added `.vais` suffix validation to `scripts/vaisc`, `scripts/build.sh`, and `tools/embed_self_source.py`.
- `scripts/vaisc --engine` now exposes `full` and `direct`.
- `scripts/vaisc` full mode now uses `compiler/self/vaisc_core.ll` and reads `.vais` inputs directly.
- Pure core regeneration from `compiler/self/fixpoint_full.vais` into `compiler/self/vaisc_core.ll` is green.
- Documentation is being consolidated around `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, and `compiler/self/SELF_HOST.md`.
- Official website source at `/Users/sswoo/study/projects/vais-public-claim-guard-compiler/website` was reduced to the current `.vais` language, `scripts/vaisc`, self-host status, and verification gates.
- Website `dist/` was rebuilt and checked for stale public syntax, install, and ecosystem claims.
- Official website source was copied into this repository at `website/` so future docs and site changes share one Vais baseline.
- Added `.github/workflows/deploy-website.yml` for GitHub Pages deployment from `website/dist`.
- Pushed `codex/website-docs-deploy` to `vaislang/vais`.
- Deployed the built site to `gh-pages` and switched `vaislang.dev` Pages settings to `gh-pages` with HTTPS enforced.
- Preserved the old remote `main` at `archive/old-main-2026-06-14`.
- Force-updated remote `main` to the current Vais-only history.
- Switched `vaislang.dev` from the temporary `gh-pages` deployment to the `main`
  GitHub Pages workflow.
