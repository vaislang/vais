# Vais Worklog

## 2026-06-18

- Added the first `vais.toml` package manifest slice for `name`, `version`, and
  `source` source-root resolution.
- Added local dependency package paths under `vais.toml` `[dependencies]`, with
  dependency imports resolving through native public driver and Python fallback
  paths.
- Added native and Python fallback gates for package manifest success,
  dependency imports, and manifest diagnostics.

## 2026-06-17

- Replaced the stale Map example with the verified local `Map<Int,Int>` API:
  `{}`, `insert`, `get(key, default)`, `contains`, and `len`.
- Added a release-corpus List method example for `is_empty()`, `last()`, and
  `pop()`.
- Promoted both examples in `tools/vaisc-parity.tsv` and synced the roadmap,
  examples README, and changelog with the value corpus.
- Specified the Phase 2 module/package/import model in `docs/design/MODULES.md`.
- Added front and `vais-check` diagnostics for reserved `module` and `package`
  declarations.
- Implemented the first full-engine local import slice in the native public
  driver and Python fallback.
- Added gates for multi-file import success, missing imports, duplicate
  symbols, and import cycles.

## 2026-06-16

- Added `List<T>.is_empty()` to the full self-host compiler for local and
  parameter lists.
- Regenerated `compiler/self/vaisc_core.ll` from
  `compiler/self/fixpoint_full.vais`.
- Added native direct `List<Int>` and `List<Struct>` `is_empty()` lowering and
  diagnostics.
- Added full, front, direct, and error gate coverage for the promoted API.
- Synced `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`, roadmap, changelog,
  and website copy with the current gate-backed list surface.
- Ran `bash scripts/test-release-gates.sh`; it passed and produced
  `dist/vais-0.2.2-darwin-arm64.tar.gz`.
- Added `List<T>.last()` for non-empty lists to the full self-host compiler and
  native direct engine.
- Added struct-list `last()` binding coverage with `let item = xs.last()`.
- Updated front, direct, full, and diagnostic gates plus public docs for the
  promoted `last()` API.
- Added `List<T>.pop()` for non-empty lists to the full self-host compiler and
  native direct engine.
- Added scalar and struct-list `pop()` gate coverage, including parameter-list
  length mutation.
- Updated front, direct, full, and diagnostic gates plus public docs for the
  promoted `pop()` API.
- Added runtime trap behavior for invalid `List` access in the full self-host
  compiler and native direct engine.
- Added full and direct gate coverage for out-of-range index, empty `last()`,
  and empty `pop()` behavior.
- Updated `std/PRELUDE.md`, language reference, roadmap, changelog, and website
  copy for the list bounds trap contract.
- Promoted the first `Str` tool-helper slice through the public front contract,
  native direct engine, and parity manifest.
- Added direct lowering for `Str` literals, locals, parameters, returns,
  `s.len()`, `s[i]`, and `Str` equality/inequality, plus `Bool` helper
  signatures.
- Promoted string index, parse_uint, and identifier-scan examples in
  `tools/vaisc-parity.tsv`.
- Promoted named `parse_uint(s)` and `parse_int(s)` prelude helpers through the
  full self-host compiler, native direct engine, front gate, parity manifest,
  and value corpus.
- Regenerated `compiler/self/vaisc_core.ll` with the named parsing helpers.
- Added native direct local `Map<Int,Int>` lowering for `{}`, `insert`,
  `get(key, default)`, `contains`, and `len`, with direct gate coverage.
- Added full self-host local `Map<Int,Int>` lowering for the same surface and
  regenerated `compiler/self/vaisc_core.ll`.
- Updated front diagnostics so local `Map<Int,Int>` values are accepted while
  Map parameters, returns, assignment, and generic key/value forms stay gated.

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
- Added direct-engine element assignment for `List<Int>` and `List<Struct>`
  locals and parameters, including list-index element type inference.
- Added direct-engine returned-list argument lowering for `if` and `else if`
  conditions with both `List<Int>` and `List<Struct>`.
- Synced language reference, roadmap, changelog, design notes, and site copy
  with the promoted direct list slices.
- Added `scripts/test-release-gates.sh` as the pre-tag release gate covering
  native, install/package, front, direct, errors, parity, value, self-host,
  archive, website, and diff checks.
- Added `docs/release/RELEASE_CHECKLIST.md` with the `v0.2.2` next-release
  line, tag policy, manual archive workflow command, and post-tag checks.
- Prepared the `v0.2.2` source release metadata across the native compiler
  version, changelog, release checklist, roadmap, and website package.
- Ran `bash scripts/test-release-gates.sh` for `v0.2.2`; it passed and produced
  `dist/vais-0.2.2-darwin-arm64.tar.gz`.
- Pushed the annotated `v0.2.2` source tag and verified the GitHub Release
  assets for Linux x64, macOS arm64, and macOS x64.
- Verified the `Deploy Website` workflow for commit `5dfb49e3` and confirmed
  live `vaislang.dev` still exposes `scripts/vaisc` and
  `bash scripts/test-release-gates.sh`.

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
