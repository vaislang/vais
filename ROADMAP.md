# Vais Roadmap

This file tracks current work only.

## Done

- Project path is `/Users/sswoo/study/projects/vais`.
- Checked-in language sources use `.vais`.
- `scripts/vaisc` is the canonical compiler command.
- `tools/vais-check.py` is the canonical lint/error-help command.
- The workspace now exposes only Vais source and Vais commands.
- The compiler gates cover CLI smoke, front-contract diagnostics, direct LLVM emission, parity, and the value corpus.
- The trusted self-host tier is `compiler/self/fixpoint.vais`, `fixpoint2.vais`, `fixpoint3.vais`, and `fixpoint_full.vais`.
- `compiler/self/vaisc_core.ll` is the reusable self-host compiler core used by `scripts/vaisc`.
- The full compiler path reads `.vais` source files directly through the self-host core.
- Pure regeneration of `compiler/self/vaisc_core.ll` from `compiler/self/fixpoint_full.vais` is green.
- The native compiler can be installed as a standalone `vaisc` binary outside
  the checkout and packaged as a release archive.
- Source tag builds have a release archive workflow for standalone compiler
  assets.
- The native direct engine covers Int helper calls, locals, assignment, `if`,
  `while`, returns, simple Int-field struct locals, and struct parameter/return
  helpers without invoking Python.
- The native direct engine covers the first local `List<Int>` slice: `[]`,
  `list()`, small integer list literals, `push`, `len`/`len()`, index, and
  `sum()`.
- The native direct engine covers `List<Int>` function parameter and return
  ABI, including push-to-parameter mutation for local list arguments.
- The native direct engine covers inline `List<Int>` literal and `list()`
  values in call arguments and return expressions.
- The native direct engine hoists `List<Int>`-returning helper calls used as
  `List<Int>` call arguments in statement contexts.
- The native direct engine hoists `List<Int>`-returning helper calls in `while`
  conditions and reevaluates them on each loop iteration.
- The native direct engine lowers `List<Int>` and `List<Struct>` returned-list
  helper calls used as list arguments in `if` and `else if` conditions.
- The native direct engine covers local `List<Struct>` values for declared
  structs: typed `[]`, `list()`, list literal initialization, `push`, `len`,
  index, and field reads.
- The native direct engine covers `List<Struct>` function parameter and return
  ABI, including inline list arguments and returned-list argument hoisting.
- The native direct engine covers context-typed assignment for `List<Int>` and
  `List<Struct>` locals and list parameters.
- The native direct engine covers `List<Int>` and `List<Struct>` element
  assignment, including assignments through list parameters.
- The native direct engine covers indexed `List<Struct>` field assignment,
  including assignments through list parameters.

## Current Reality

- The full compiler path emits LLVM IR through the self-host compiler source in `compiler/self/fixpoint_full.vais`.
- The direct engine is intentionally narrow and currently supports Int helpers,
  locals, assignment, calls, `if`, `while`, returns, simple Int-field struct
  local literal/read/write, struct parameter/return helper ABI, and local
  `List<Int>` initialization plus `push`, `len`, index, `sum`, and
  `List<Int>` parameter reference, return value ABI, and inline list
  literal/constructor call and return values. Statement contexts, `if`,
  `else if`, and `while` conditions also lower `List<Int>`-returning helper
  calls before passing them to `List<Int>` parameters. Local `List<Struct>`
  values support typed `[]`, `list()`, list literal initialization, `push`,
  `len`, index, field reads/writes, parameter reference, return value ABI,
  inline list arguments, and returned-list argument lowering in statements plus
  `if`, `else if`, and `while` conditions. Context-typed list assignment is supported
  for `List<Int>` and `List<Struct>` locals and list parameters. Element
  assignment is supported for `List<Int>` and `List<Struct>`, including through
  list parameters.
- The release compiler command uses a native host driver for normal user
  `emit-ir`, `build`, and `run`; Python remains for internal repository checks
  and diagnostics only.
- Standalone install, uninstall, package, and install/package verification
  scripts exist for the native compiler binary.
- Internal compiler gates no longer depend on a source pass-through helper.
- Public documentation now starts at `README.md` and `docs/README.md`.
- `docs/reference/LANGUAGE.md` describes only the current gate-backed language surface.
- Local official website source was refreshed and rebuilt from the canonical Vais docs.
- Official site source now lives in `website/` in this repository.
- GitHub Pages workflow was added for `website/` build and artifact deployment.
- `vaislang.dev` deploys from the `website/` GitHub Pages workflow on `main`.
- `CHANGELOG.md` records the current `v0.2.1` source release baseline.
- GitHub `main` now points to the current Vais-only history; old remote `main`
  is preserved at `archive/old-main-2026-06-14`.

## Next Work

1. Keep standalone release archives attached to future source tags.
2. Extend the direct engine beyond the current list ABI and assignment coverage
   toward broader list operations and larger composite value coverage.
3. Keep README, language docs, website copy, and `CHANGELOG.md` synced with the
   Python-free public command path.
4. Replace the remaining Python-only internal checks when the language has
   enough file/process support.
5. Keep source release tags, GitHub Releases, GitHub Pages, self-host regeneration, and parity gates green.

## Completed Milestone: Native Direct List Else-If Condition Arguments

Mode: sequential

- [x] 1. Lower returned `List<Int>` and `List<Struct>` helper calls in `else if` conditions.
- [x] 2. Gate direct `else if score(make(...))` behavior for both integer and struct lists.
- [x] 3. Sync docs/site/changelog with the promoted condition-argument slice.

### Task Briefs

#### 1. Direct else-if returned-list argument lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts `else if` conditions such as
  `} else if score(make(20)) == 41 {` when `make` returns a list and `score`
  receives the matching `List<T>` parameter.
- Done: returned-list call arguments can lower as C compound-literal list
  temporaries in expression contexts that cannot receive a statement prelude,
  preserving `else if` evaluation order without rewriting the control-flow
  shape.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `docs/reference/LANGUAGE.md`,
  `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`, `docs/design/`.
- Requirements: prove direct `List<Int>` and `List<Struct>` returned-list
  arguments execute inside `else if` conditions and keep public docs precise
  about the promoted scope.
- Done: direct gate covers `score_int(make_int(...))` and
  `score_box(make_box(...))` inside `else if` conditions returning 42.

## Completed Milestone: Native Direct List If-Condition Hoisting

Mode: sequential

- [x] 1. Hoist returned `List<Int>` and `List<Struct>` helper calls in plain `if` conditions.
- [x] 2. Gate direct `if score(make(...))` behavior for both integer and struct lists.
- [x] 3. Sync docs/site/changelog with the promoted condition-hoisting slice.

### Task Briefs

#### 1. Direct plain-if returned-list argument hoisting

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts plain `if` conditions such as
  `if score(make(20)) == 41 {` when `make` returns a list and `score` receives
  the matching `List<T>` parameter.
- Done: direct `if` lowering now attaches the existing list-argument prelude
  before the generated C `if`, so returned-list temporaries are available to
  the condition expression.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `docs/reference/LANGUAGE.md`,
  `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`, `docs/design/`.
- Requirements: prove direct `List<Int>` and `List<Struct>` returned-list
  arguments execute inside plain `if` conditions and keep public docs precise
  about the promoted scope.
- Done: direct gate covers `score_int(make_int(...))` and
  `score_box(make_box(...))` inside plain `if` conditions returning 42.

## Completed Milestone: Native Direct List Element Assignment

Mode: sequential

- [x] 1. Parse `List` indexed element assignment targets.
- [x] 2. Infer `xs[index]` expression types from the list element type.
- [x] 3. Gate `List<Int>` and `List<Struct>` element assignment locally and through parameters.
- [x] 4. Sync docs/site/changelog with the promoted element-assignment slice.

### Task Briefs

#### 1. Indexed list element assignment

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts assignments such as `xs[0] = 42`,
  `boxes[0] = Box { value: 42 }`, and `boxes[1] = boxes[0]` when the value
  matches the list element type.
- Done: assignment target validation now recognizes `base[index]`, target type
  lookup returns the list element type, and exact list-index expressions infer
  to their element type.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove local and parameter element assignments execute through
  `scripts/vaisc --engine direct` for both `List<Int>` and `List<Struct>`, and
  keep non-list indexed assignment targets behind a P4 diagnostic.
- Done: direct gate covers `List<Box>` element literal assignment, element copy,
  parameter element replacement, and `List<Int>` element assignment returning
  42; error gate covers a non-list indexed assignment target.

## Completed Milestone: Native Direct List Struct Field Assignment

Mode: sequential

- [x] 1. Parse `List<Struct>` indexed field assignment targets.
- [x] 2. Type-check indexed struct-list field assignments as `Int` field writes.
- [x] 3. Gate local and parameter `xs[index].field = value` behavior.
- [x] 4. Sync docs/site/changelog with the promoted field-write slice.

### Task Briefs

#### 1. Indexed List<Struct> field assignment

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts assignments such as `xs[0].value = 42` and
  `xs[i].value = value` when `xs` is a `List<DeclaredStruct>` and `value` is a
  declared `Int` field.
- Done: assignment target validation now recognizes `base[index].field`, checks
  the list element struct field, and rewrites the left-hand side through the
  existing list-index expression lowering.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove local and parameter `List<Struct>` indexed field writes
  execute through `scripts/vaisc --engine direct` and keep unknown element
  fields behind a P4 diagnostic.
- Done: direct gate covers local and parameter `List<Box>` field writes
  returning 42; error gate covers an unknown indexed field target.

## Completed Milestone: Native Direct List Assignment

Mode: sequential

- [x] 1. Make direct list assignment context-typed for `List<Int>` and `List<Struct>`.
- [x] 2. Support assigning `[]`, `list()`, list literals, local lists, and returned lists to matching list locals and list parameters.
- [x] 3. Gate caller-visible replacement through list parameter assignment.
- [x] 4. Sync docs/site/changelog with the promoted assignment slice.

### Task Briefs

#### 1. Context-typed list assignment

- Target files: `tools/vaisc_native.c`.
- Requirements: direct assignment to a list target should validate list
  literals using the target element type instead of inferring bare list
  literals as `List<Int>`.
- Done: assignment lowering now treats list initializer expressions as
  context-typed when the target is `List<T>`, then rewrites the value with the
  target list type.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `docs/reference/LANGUAGE.md`,
  `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`, `docs/design/`.
- Requirements: prove direct `List<Int>` and `List<Struct>` assignment runs,
  including assignment through a `List<Struct>` parameter that replaces the
  caller's list.
- Done: direct gate covers `List<Box>` local assignment from `[]`, `list()`,
  literals, returned lists, parameter replacement, and `List<Int>` literal
  assignment returning 42.

## Completed Milestone: Native Direct List Struct ABI

Mode: sequential

- [x] 1. Accept `List<Struct>` in direct function parameter and return types.
- [x] 2. Lower `List<Struct>` parameters as native references and return values by value.
- [x] 3. Gate inline `List<Struct>` arguments and returned-list argument hoisting.
- [x] 4. Sync docs/site/changelog with the promoted struct-list ABI.

### Task Briefs

#### 1. Direct List<Struct> ABI lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can pass local `List<DeclaredStruct>` values to
  helpers by reference, return `List<DeclaredStruct>` values by value, lower
  inline struct-list literals, and hoist `List<Struct>`-returning helper calls
  before passing them to `List<Struct>` parameters.
- Done: direct lowering now uses `DirectList_<Struct> *` for list parameters,
  `DirectList_<Struct>` for returns and temporaries, and context-typed list
  literals for `List<Struct>`.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove `List<Struct>` parameters, return values, inline
  arguments, returned-list argument hoisting, and while-condition hoisting run
  through `scripts/vaisc --engine direct`.
- Done: direct gate covers `List<Box>` parameter mutation, return-by-value,
  inline arguments, returned-list arguments, and while-condition hoisting
  returning 42.

## Completed Milestone: Native Direct Local List Struct Slice

Mode: sequential

- [x] 1. Parse and validate direct-engine local `List<Struct>` types.
- [x] 2. Lower local `List<Struct>` storage, `[]`, `list()`, literals, `push`, `len`, index, and field reads.
- [x] 3. Gate the promoted slice and leave `List<Struct>` function ABI to the following milestone.
- [x] 4. Sync docs/site/changelog with the promoted local struct-list slice.

### Task Briefs

#### 1. Direct local List<Struct> lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts local `List<DeclaredStruct>` values without
  invoking Python; `List<Struct>` function parameter/return ABI is handled by
  the following milestone.
- Done: direct lowering emits `DirectList_<Struct>` locals for typed `[]`,
  `list()`, and small struct list literals, lowers `push`, `len`/`len()`, index
  reads, and field reads such as `xs[0].value`.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove the local `List<Struct>` slice emits LLVM IR and runs
  through `scripts/vaisc --engine direct`, with function ABI left for the next
  promoted slice.
- Done: direct gate covers local `List<Box>` push, length, index, and field-read
  behavior returning 42.

## Completed Milestone: Release Automation And Native Direct Int Slice

Mode: sequential

- [x] 1. Add release archive workflow for source tags.
- [x] 2. Remove the public direct-engine Python fallback.
- [x] 3. Expand the native direct engine through Int helper calls, locals, assignment, `if`, `while`, simple struct locals, and struct parameter/return helpers.
- [x] 4. Sync README, language docs, website copy, changelog, and gates.

### Task Briefs

#### 1. Release archive workflow

- Target files: `.github/workflows/release-archives.yml`, `scripts/package-vaisc-release.sh`.
- Requirements: tag builds package standalone compiler archives and upload them to the matching GitHub Release.
- Done: workflow builds Linux/macOS archive jobs, smokes packaged `vaisc`, creates the release when needed, and uploads archives.

#### 2. Native direct path

- Target files: `scripts/vaisc`, `tools/vaisc_native.c`.
- Requirements: `--engine direct` must stay on the native driver and must not invoke Python.
- Done: `scripts/test-vaisc-direct.sh` proves direct mode still works with a failing `python3` shim first in `PATH`.

#### 3. Direct Int control-flow and struct slice

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: direct mode accepts Int helper functions, locals, assignment, calls, `if`, `while`, returns, simple Int-field struct local literal/read/write, and struct parameter/return helper ABI; unsupported identifiers keep P4 diagnostics.
- Done: direct tests cover arithmetic, helper calls, locals, control flow, struct locals, struct parameter/return helpers, full-engine parity, and P4 errors.

#### 4. Documentation and gates

- Target files: `README.md`, `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `AGENTS.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs describe current native direct and release archive automation without publishing unsupported direct lists or self-host claims.
- Done: docs/site/changelog are synced and release gates pass.

## Completed Milestone: Native Direct Local List Slice

Mode: sequential

- [x] 1. Add native direct local `List<Int>` storage and helper lowering.
- [x] 2. Add direct tests for `[]`, small integer list literals, `push`, `len`, index, and `sum`.
- [x] 3. Sync docs/site/changelog with the promoted direct list slice.

### Task Briefs

#### 1. Direct local List<Int> lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode accepts local `List<Int>` values without invoking
  Python; function parameter/return list ABI stays out of this slice.
- Done: direct lowering emits `DirectListInt` locals for `[]`, `list()`, and
  small integer list literals, lowers `push`, `len`/`len()`, index reads, and
  `sum()`.

#### 2. Direct list gate

- Target files: `scripts/test-vaisc-direct.sh`.
- Requirements: prove the new list slice emits LLVM IR and runs through
  `scripts/vaisc --engine direct`.
- Done: direct gate covers local list push, length, index, literal, and sum
  behavior returning 42.

#### 3. Documentation sync

- Target files: `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`,
  `ROADMAP.md`, `WORKLOG.md`, `docs/design/`.
- Requirements: public docs describe the promoted direct list slice and leave
  list parameters/returns as future work.
- Done: docs and site copy are synced to the current direct/full engine split.

## Completed Milestone: Native Direct List Int Inline Values

Mode: sequential

- [x] 1. Lower inline `List<Int>` literals and `list()` as direct return values.
- [x] 2. Lower inline `List<Int>` literals and `list()` as direct call arguments.
- [x] 3. Gate inline call/return values and preserve non-local argument diagnostics.
- [x] 4. Sync docs/site/changelog with the promoted inline value slice.

### Task Briefs

#### 1. Inline list value lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can lower `return []`, `return [1, 2]`,
  `return list()`, `score([])`, `score([1, 2])`, and `score(list())` for
  `List<Int>` signatures without invoking Python.
- Done: direct lowering emits `DirectListInt` compound literals for inline list
  return values and passes addresses of inline list compound literals to
  `List<Int>` parameters.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove inline list values execute through direct mode and keep
  list-returning helper calls used directly as list arguments behind a diagnostic.
- Done: direct gates cover inline list call/return values; non-literal
  returned-list arguments were left for the returned-argument hoisting milestone.

## Completed Milestone: Native Direct List Int Returned-Argument Hoisting

Mode: sequential

- [x] 1. Hoist `List<Int>`-returning helper calls used as `List<Int>` arguments.
- [x] 2. Gate nested returned-list arguments across return, let, list literal,
  push, and assignment statements.
- [x] 3. Keep loop-condition returned-list arguments behind a diagnostic.
- [x] 4. Sync docs/site/changelog with the promoted hoisting slice.

### Task Briefs

#### 1. Returned-list argument hoisting

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can lower statement-context calls such as
  `score(make(10))`, `score(pass(make(5)))`, list literal items containing those
  calls, `push(score(make(2)))`, and assignment from those calls.
- Done: direct lowering adds per-function temporary `DirectListInt` locals before
  the current C statement and passes their addresses to `List<Int>` parameters.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove returned-list arguments execute through direct mode and
  document that loop-condition hoisting is still outside the direct claim.
- Done: direct gates cover returned-list argument hoisting in statement contexts;
  while-condition hoisting was left for the following milestone.

## Completed Milestone: Native Direct List Int While Hoisting

Mode: sequential

- [x] 1. Hoist `List<Int>`-returning helper calls inside direct `while`
  conditions.
- [x] 2. Preserve per-iteration condition reevaluation.
- [x] 3. Gate while-condition returned-list argument hoisting.
- [x] 4. Sync docs/site/changelog with the promoted loop-hoisting slice.

### Task Briefs

#### 1. While condition hoisting

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can lower `while score(make(i)) < limit { ... }`
  without evaluating `make(i)` only once before the loop.
- Done: direct lowering emits `while (1)` when condition prelude temporaries are
  required, rebuilds the hoisted `DirectListInt` temporaries each iteration, and
  breaks when the translated condition is false.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove returned-list argument hoisting in direct `while`
  conditions executes through direct mode and keep docs synced to the new claim.
- Done: direct gates cover per-iteration while-condition hoisting returning 42.

## Completed Milestone: Native Direct List Int Out-Param Semantics

Mode: sequential

- [x] 1. Lower `List<Int>` parameters as direct native references.
- [x] 2. Preserve `List<Int>` return values as value returns.
- [x] 3. Gate callee `push` mutation of caller local lists.
- [x] 4. Keep unsupported non-local returned-list arguments covered by diagnostics.
- [x] 5. Sync docs/site/changelog with the promoted out-param slice.

### Task Briefs

#### 1. Direct list parameter references

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode passes named local `List<Int>` arguments to
  `List<Int>` parameters by reference while keeping non-list parameters on their
  existing value ABI.
- Done: direct lowering emits native pointer parameters for `List<Int>`, rewrites
  calls to pass local list addresses, and rewrites parameter `len`, index, `sum`,
  assignment, and `push` operations through the referenced list.

#### 2. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: prove callee `push` mutates the caller local list and keep
  returned list expressions out of direct list argument claims.
- Done: direct gates cover caller-visible mutation and diagnostics require
  non-literal `List<Int>` arguments to be local list names.

## Completed Milestone: Native Direct List Int ABI

Mode: sequential

- [x] 1. Parse `List<Int>` in direct function headers.
- [x] 2. Lower `List<Int>` parameters and return values through the direct ABI.
- [x] 3. Add direct/error gates for list ABI and type mismatch diagnostics.
- [x] 4. Sync docs/site/changelog with the promoted ABI slice.

### Task Briefs

#### 1. Function header parsing

- Target files: `tools/vaisc_native.c`.
- Requirements: direct function parameter and return annotations may use
  `List<Int>` in addition to `Int` and declared structs.
- Done: direct header parsing and validation accept `List<Int>`.

#### 2. List ABI lowering

- Target files: `tools/vaisc_native.c`.
- Requirements: direct mode can pass local `List<Int>` values to helpers, return
  local or helper-produced `List<Int>` values, and bind returned list values to
  locals.
- Done: direct lowering handles `List<Int>` helper parameters and return values
  and checks return, local initializer, assignment, and call-argument types
  before C/LLVM.

#### 3. Gates and documentation

- Target files: `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`,
  `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`,
  `WORKLOG.md`, `docs/design/`.
- Requirements: gate the promoted ABI and keep unsupported direct list call
  expressions out of public direct claims.
- Done: direct gates cover list parameter/return ABI and diagnostics cover list
  type mismatches and non-local list call arguments.

## Completed Milestone: Standalone Install And Release Archive

Mode: sequential

- [x] 1. Add install and uninstall scripts for standalone `vaisc`.
- [x] 2. Add release archive packaging for the native binary and first-read docs.
- [x] 3. Add an install/package gate that proves installed and packaged binaries run.
- [x] 4. Sync docs/site/changelog and run release gates.

### Task Briefs

#### 1. Standalone install and uninstall

- Target files: `scripts/install-vaisc.sh`, `scripts/uninstall-vaisc.sh`.
- Requirements: build the native compiler from the checked-in self-host core and install it as `PREFIX/bin/vaisc`; uninstall removes that binary.
- Done: installing into a temporary prefix creates an executable `vaisc`, and uninstall removes it.

#### 2. Release archive packaging

- Target files: `scripts/package-vaisc-release.sh`, `.gitignore`.
- Requirements: build a standalone archive containing `bin/vaisc` and the current first-read docs; keep generated archives out of git.
- Done: the package script creates `dist/vais-VERSION-OS-ARCH.tar.gz`.

#### 3. Install/package gate

- Target files: `scripts/test-vaisc-install.sh`, `AGENTS.md`, `README.md`.
- Requirements: verify installed and packaged binaries can report version, run `doctor`, and compile/run a `.vais` smoke source.
- Done: `bash scripts/test-vaisc-install.sh` passes without writing outside a temporary directory.

#### 4. Documentation, site, and gates

- Target files: `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs describe checkout use, standalone install, uninstall, package, and the gate protecting them.
- Done: docs and site are synced, website builds, stale public-claim scan is clean, and release gates pass.

## Completed Milestone: Python-Free Public `vaisc`

Mode: sequential

- [x] 1. Native driver skeleton and build script.
- [x] 2. Release source-preparation parity with the current Python path.
- [x] 3. `scripts/vaisc` default switch and install/doctor UX.
- [x] 4. Documentation/site/changelog sync and release gates.

### Task Briefs

#### 1. Native driver skeleton and build script

- Target files: `tools/`, `scripts/`, `README.md`, `ROADMAP.md`.
- Requirements: compile a native `vaisc` binary from a small host driver and `compiler/self/vaisc_core.ll`; support `emit-ir`, `build`, `run`, `--help`, `--version`, and `doctor` for the full engine path.
- Done: a local native binary can compile and run `examples/c4.vais` without invoking Python.

#### 2. Release source-preparation parity

- Target files: native driver/source-prep implementation and existing `scripts/test-vaisc*.sh` gates.
- Requirements: match the release subset behavior currently implemented by `tools/vaisc.py`, including enum/match, payload enum, closure-return, typed `Int`, `print`, comments, and semicolon normalization.
- Done: `bash scripts/test-vaisc.sh`, `bash scripts/test-vaisc-front.sh`, `bash scripts/test-vaisc-errors.sh`, `bash scripts/test-vaisc-parity.sh`, and `bash scripts/test.sh` pass through the native public command.

#### 3. Public command switch and install UX

- Target files: `scripts/vaisc`, packaging/install scripts, README docs.
- Requirements: `scripts/vaisc` uses the native driver by default; Python is not required for normal user `emit-ir`, `build`, or `run`; `doctor` reports missing `clang` or missing native driver clearly.
- Done: a fresh checkout can build the native driver and run `scripts/vaisc doctor`, `scripts/vaisc run examples/c4.vais`, and `scripts/vaisc build examples/c4.vais -o /tmp/c4`.

#### 4. Documentation, release, and gates

- Target files: `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, `compiler/self/SELF_HOST.md`, `website/`, `CHANGELOG.md`, `WORKLOG.md`.
- Requirements: public docs describe the Python-free command path only after verification; internal Python tools remain documented as checks, not as user runtime prerequisites.
- Done: verification baseline plus self-host gates pass, the website builds, stale public-claim scan is clean, and GitHub/site release notes are synced.

## Verification Baseline

Run before closing compiler changes:

```bash
python3 -m py_compile tools/vaisc.py tools/vais-check.py tools/embed_self_source.py tests/vais_check_test.py
bash -n scripts/*.sh
python3 tests/vais_check_test.py
bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc-install.sh
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh
```
