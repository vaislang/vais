# Vais Worklog

## 2026-06-15

- Expanded native direct mode with the first local `List<Int>` slice:
  `[]`, `list()`, small integer list literals, `push`, `len`/`len()`, index,
  and `sum()`.
- Added direct-engine gate coverage for local `List<Int>` emit/run behavior.
- Expanded native direct mode with `List<Int>` function parameter and return
  ABI.
- Switched direct `List<Int>` parameters to native references for local list
  arguments and gated caller-visible callee `push` mutation.
- Added direct-engine lowering and gate coverage for inline `List<Int>` literal
  and `list()` call/return values.
- Added direct-engine temporary hoisting for `List<Int>`-returning helper calls
  passed directly to `List<Int>` parameters in statement contexts.
- Added per-iteration direct-engine hoisting for returned-list arguments inside
  `while` conditions.
- Added direct-engine local `List<Struct>` lowering for declared structs,
  including typed `[]`, `list()`, list literals, `push`, `len`, index, and
  field reads.
- Added direct-engine gate coverage for local `List<Box>` emit/run behavior.
- Expanded direct-engine `List<Struct>` support through function parameter and
  return ABI, inline call arguments, returned-list argument hoisting, and
  while-condition hoisting.
- Added direct-engine gate coverage for `List<Box>` ABI behavior returning 42.
- Added context-typed direct list assignment for `List<Int>` and `List<Struct>`,
  including list-parameter replacement and gate coverage returning 42.
- Added direct-engine indexed field assignment for `List<Struct>` locals and
  parameters, plus P4 diagnostics for unknown indexed fields.
- Synced language reference, roadmap, changelog, design notes, and site copy
  with the promoted direct list slices.

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
- Added `CHANGELOG.md` for the current source release baseline.
- Added a native `vaisc` host driver and switched `scripts/vaisc` normal
  `emit-ir`, `build`, and `run` to the Python-free public path.
- Added standalone native install, uninstall, package, and install/package gates.
- Added release archive workflow for source tags.
- Moved `--engine direct` onto the native driver and expanded it through Int
  helper calls, locals, assignment, `if`, and `while`.
- Expanded native direct mode with simple Int-field struct local literal,
  field read, and field write support.
- Expanded native direct mode with struct parameter and return helper ABI.
