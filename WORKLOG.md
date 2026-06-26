# Vais Worklog

## 2026-06-26

- Prepared and published the `v0.3.2` release-candidate tag from clean mainline:
  updated compiler/site version metadata, cut the annotated tag, verified the
  GitHub Release archives for Linux x64, macOS arm64, and macOS x64, verified
  the GitHub Pages deploy, and reran `bash scripts/test-release-gates.sh` from
  the clean tagged checkout.

## 2026-06-25

- Extended the Vais-authored local import graph checker to follow the first
  package manifest local dependency alias and dependency-internal plain imports.
- Extended the Vais-authored local import graph checker to follow all declared
  entry-package local dependency aliases.
- Wired `scripts/vaisc` to run cached Vais-authored package manifest and import
  graph preflight tools before native `emit-ir`, `build`, and `run`.
- Closed the Phase 5 self-host expansion checklist after release gates confirmed
  regenerated core, preflight, import graph, and self-host paths remain green.
- Froze the v1-candidate language and prelude reference docs around the current
  gate-backed surface.
- Added a Vais-authored local import graph contract checker and release gate for
  manifest-free missing import, duplicate top-level symbol, and import cycle
  diagnostics.
- Reconciled the Phase 4.2 parent roadmap checkbox now that the listed enum
  payload and pattern/match slices are all gate-backed.
- Added optional entry-path source-root containment checking to the
  Vais-authored package manifest checker contract.
- Added local dependency cycle detection to the Vais-authored package manifest
  checker contract using normalized local manifest paths.
- Moved the package manifest missing-source-directory diagnostic into the
  Vais-authored manifest checker contract, matching the native driver error
  while preserving the product driver's OS-facing package discovery boundary.
- Added `tools/vais_manifest_check.vais` and
  `tools/vais_manifest_contract_check.vais` as a Vais-authored package manifest
  contract gate, then wired it into the release gate through
  `scripts/test-vais-manifest-check-vais.sh`.
- Closed Phase 5.2 for the current compiler-owned static source diagnostics:
  remaining native-front-only closure/enum/match rejects are not public checker
  rejects because the full language already verifies those features, and
  manifest/import graph/source-path diagnostics remain explicit 5.3 host-boundary
  work.
- Added invalid `main` entrypoint signature detection to the Vais-authored
  checker contract while preserving function-type and closure examples.
- Added missing helper return-type detection to the Vais-authored checker
  contract and kept function-type values out of that diagnostic path.
- Added unsupported generic `Map<K,V>` detection to the Vais-authored checker
  contract, kept verified concrete Map shapes clean, and updated the standalone
  checker issue count.
- Added unsupported generic `Result<T,E>` detection to the Vais-authored checker
  contract, kept the standalone checker issue count aligned, and preserved
  verified `Result<Int,Int>` as the only claimed Result shape.
- Added unsupported generic `Option<T>` detection to the Vais-authored checker
  contract, kept verified `Option<Int>` examples clean, and updated the
  standalone install/package checker issue count.
- Moved invalid static import path checking into the Vais-authored checker
  contract, added the matching public front reject fixture, and updated the
  checker fixture count to keep `scripts/vais-check` and `scripts/vaisc`
  diagnostic shapes aligned.
- Reconciled Phase 5 roadmap status for the existing stage comparison gate:
  `tools/normalize_stage_ir.vais` is already covered by a focused gate and by
  the full-source self-host stage1/stage2 comparison.
- Promoted `examples/e120_enum_payload_wildcard.vais` into the release corpus
  as the first payload enum `match` with `_` catch-all slice, with matching
  public front fixture, parity entry, docs, site count, changelog, and roadmap
  updates.
- Added `examples/e119_map_param_target_assignment.vais` to cover
  parameter-target assignment copies for every verified concrete Map type,
  updated front and full self-host codegen coverage, docs, site count,
  changelog, and roadmap while keeping generic Map behavior gated.
- Reconciled Phase 4 parent roadmap checkboxes for the completed
  Map/Option/Result and unsupported-syntax diagnostic slices.
- Promoted `examples/e25_for_filter_sum.vais`, `examples/e27_list_max.vais`,
  and `examples/fr2.vais` into the release corpus as gate-backed collection
  for-each examples, covering full self-host array iteration, scalar
  `List<Int>` local/parameter iteration, typed non-empty local `List<Int>`
  literals, inline `List<Int>` literal call arguments, native direct
  `List<Int>` iteration, parity, value, docs, and site count updates.
- Promoted `examples/e82_list_literal_direct_arg.vais` into the release corpus
  as the direct public smoke for inline `List<Int>` literal call arguments.
- Promoted `examples/e63_generic_struct_def.vais` into the release corpus as a
  front/parity/value-gated generic marker struct example used with `Int` values.
- Promoted six struct helper examples into the release corpus, covering
  struct-returning helpers, struct parameter helpers, assignment from
  struct-returning calls, recursive struct accumulators, and multi-value struct
  returns through full self-host and native direct gates.
- Promoted the local module, package source-root, and local dependency package
  examples into the release corpus as value-gated import/package smokes.
- Fixed and promoted single-field nested struct literal/read/write lowering in
  the full self-host compiler, including regenerated reusable core coverage.
- Promoted additional already-correct Result propagation, inline `List<Int>`
  parameter iteration, and direct `Option<Int>` match examples into the release
  corpus.
- Promoted borrowed `&List<Int>` helper parameters through the public front and
  release corpus with recursive traversal and binary-search examples.
- Promoted public struct/function modifiers and `Str` fields in struct literals
  through the checker, public front, full self-host compiler, regenerated core,
  and release corpus with `examples/d5run.vais`.
- Promoted already-supported `examples/t2.vais`, `examples/t3.vais`, and
  `examples/t5.vais` into the release corpus as enum, bitwise, and Option smoke
  coverage.
- Promoted `examples/d2.vais` into the release corpus by lowering multiline
  `Option<Int>` expression-match bindings through the public compiler driver
  before the self-host core receives the source.
- Promoted `examples/e73_int_to_string.vais` into the release corpus by adding
  full self-host and native direct lowering for `Str(Int)` decimal conversion,
  with regenerated reusable compiler core and front/direct/full gate fixtures.
- Promoted `examples/e46_generic_struct.vais` into the release corpus by
  lowering generic identity helpers applied directly to struct literals before
  the self-host core receives the source, with front/parity/value gate coverage.
- Promoted `examples/e51_index_ast.vais` into the release corpus by extending
  self-host `StructDef` field metadata to 20 stored fields and regenerating the
  reusable compiler core.
- Promoted `examples/e59_tuple.vais` into the release corpus by lowering `Int`
  tuple function returns and local destructuring to generated struct storage in
  the public compiler driver.
- Promoted `examples/e81_closure_return_apply.vais` into the release corpus by
  lowering a returned single-`Int` closure passed to an `Int` higher-order
  helper into the existing environment/apply helper representation.
- Promoted `examples/e09_struct_method.vais` into the release corpus by
  lowering simple `impl` struct methods and return-expression method chains to
  ordinary helper calls with intermediate struct locals.
- Promoted `examples/e49_closure_arg.vais` into the release corpus by lowering
  non-capturing inline closure literals passed to a single-closure `Int`
  higher-order helper into generated apply helpers.
- Promoted `examples/c5.vais` into the release corpus by lowering a local
  closure with one `Int` capture to an apply helper and captured environment
  value.
- Promoted `examples/e78_trait_impl_for.vais` into the release corpus by
  treating a simple `trait` declaration as metadata and lowering
  `impl Trait for Struct` methods to ordinary struct helper calls.
- Promoted `examples/e76_list_map.vais` and `examples/d6run.vais` into the
  release corpus by lowering non-capturing `List<Int>` map and filter-sum
  method slices to explicit `for` loops.
- Promoted `examples/e77_nested_list.vais` into the release corpus by lowering
  a local `List<List<Int>>` literal to row `List<Int>` locals and rewriting the
  verified double-index read.
- Promoted `examples/e79_nested_match.vais` into the release corpus by allowing
  a single enum `Option<Int>` payload and lowering its nested Option match arm
  to Int-coded branches.

## 2026-06-24

- Promoted `examples/t4.vais` and `examples/t6.vais` into the release corpus as
  simple struct literal/field-read smoke examples, raising the release corpus
  to 100 native-supported examples.
- Promoted `examples/fr1.vais` into the release corpus as an inclusive range
  for-loop summation smoke, raising the release corpus to 98 native-supported
  examples.
- Promoted `examples/e19_interpolation_print.vais` into the release corpus,
  adding native direct lowering for `print("...{name}...")` interpolation and
  `putchar(Int)` output calls.
- Promoted `examples/e71_string_index_of.vais` into the release corpus as a
  `Str` substring-search pattern with computed byte indexes, covering public
  front, native direct, full self-host codegen, parity, value, docs, and site
  count updates.
- Promoted `examples/e69_palindrome_string.vais` into the release corpus as a
  two-pointer `Str` scan with computed byte indexes from both ends, covering the
  same front, direct, full self-host, parity, value, docs, and site gates.
- Promoted 12 smaller control-flow, Bool predicate, integer-list, and `Str`
  scanner examples into the parity manifest and value corpus, raising the
  release corpus to 96 native-supported examples.

## 2026-06-20

- Prepared the `v0.3.0` source release metadata across the native compiler
  version, website package metadata, changelog, release checklist, and roadmap.
- Fixed self-host `print`/`puts` lowering for string-expression arguments,
  regenerated `compiler/self/vaisc_core.ll`, and promoted the fix as the
  `v0.3.1` patch release line because `v0.3.0` release assets had already been
  published before the darwin-arm64 archive failure was diagnosed.

## 2026-06-19

- Added `tools/vais_check_core.vais` as the first Vais-authored checker slice,
  covering the public non-Vais spelling fixture catalog while reading fixture
  files through verified host file APIs.
- Added `tools/vais_check_smoke.vais`, checker fixtures, and
  `scripts/test-vais-check-vais.sh` as the Vais checker contract gate.
- Expanded the Vais checker slice to cover the main spelling catalog, added a
  fixture count check, and made `.vais` source files visible to git by removing
  the stale ignore rule.
- Added line/column/help output to the Vais-authored checker slice and extended
  the checker contract gate to require error, coordinate, and help counts.
- Added `proc_argc()` and `proc_arg(index)` for `vaisc run -- ...` programs,
  then added `tools/vais_check_cli.vais` as an argv-backed checker entrypoint
  with bad/clean fixture gates.
- Extended `proc_argc()` and `proc_arg(index)` to `vaisc build` binaries by
  linking generated programs through a host runtime `main(argc, argv)` wrapper.
- Added `proc_capture_stdout(argv: List<Str>) -> Str` as the first captured
  process-output slice for Vais-authored repository tools.
- Added `proc_capture_stderr(argv: List<Str>) -> Str` as the captured
  diagnostics stream slice and regenerated `compiler/self/vaisc_core.ll`.
- Promoted the Vais checker to `scripts/vais-check`, then installed and
  packaged it as standalone `bin/vais-check` with install/package gate coverage.
- Expanded the clean checker fixture to cover the former unit-test
  false-positive catalog and removed that separate unit test from the release
  gate.
- Removed checker oracle use from the checker release gate; the public
  `scripts/vais-check` command is now checked by Vais fixture contracts.
- Added verified host-backed `Str` construction helpers `str_concat`,
  `str_slice`, and `str_byte`, regenerated `compiler/self/vaisc_core.ll`, and
  extended the host smoke gate to cover native build/run runtime support.
- Added full-engine lowering for `Str` reassignment and user-defined
  `-> Str` returns, then covered both through the host smoke gate.
- Added full self-host runtime lowering for `Str` equality/inequality,
  regenerated `compiler/self/vaisc_core.ll`, and restored the Vais checker CLI
  to idiomatic `path == "--help" or path == "-h"` syntax.
- Added `tools/package_vaisc_release.vais` as the Vais-authored release archive
  packager and reduced `scripts/package-vaisc-release.sh` to a thin wrapper
  that passes repo root, environment defaults, and CLI options into Vais.
- Added `tools/install_vaisc.vais` as the Vais-authored standalone installer
  and reduced `scripts/install-vaisc.sh` to a wrapper that passes repo root,
  environment defaults, and CLI options into Vais.
- Added verified `fs_remove(path)` and `tools/uninstall_vaisc.vais`, reducing
  `scripts/uninstall-vaisc.sh` to the same Vais-tool bootstrap wrapper shape.
- Added `tools/vaisc_install_check.vais` and reduced
  `scripts/test-vaisc-install.sh` to a bootstrap wrapper; installed binary
  smoke checks, checker fixture checks, archive extraction, packaged binary
  checks, and uninstall assertions now run in Vais.
- Added verified host-backed `Str` builder helpers for large text tools and
  regenerated `compiler/self/vaisc_core.ll`.
- Added `tools/embed_self_source.vais` as the Vais-authored self-source
  embedding helper, with byte-for-byte parity against the previous helper for
  checker fixtures and all self-host compiler tiers.
- Switched `scripts/test-fixpoint-full-self.sh` to build and use the Vais
  embed helper directly, and wired `scripts/test-embed-self-source-vais.sh`
  into the release gate.
- Added `tools/vaisc_errors_check.vais` as the Vais-authored NV-C3 diagnostics
  validator behind `scripts/test-vaisc-errors.sh`, using captured stderr to
  check coordinate, caret, help, and fix output.
- Added `tools/vaisc_front_check.vais` as the Vais-authored NV-C1 front
  contract validator behind `scripts/test-vaisc-front.sh`, including accepted
  source fixtures, rejected diagnostics, and package/import temp trees.
- Added `proc_run_env(argv, env)` for child-process environment overrides,
  extended the host smoke gate, and moved the direct-engine no-Python PATH
  check into `tools/vaisc_direct_env_check.vais`.
- Added `tools/vaisc_direct_smoke_check.vais` and moved the NV-C2 direct
  arithmetic/build/run smoke checks out of `scripts/test-vaisc-direct.sh`.
- Added `proc_capture_to(argv, stdout_path, stderr_path)` for status-plus-file
  process capture, extended the host smoke gate, and documented it as the
  pragmatic step before in-memory `ProcessResult`.
- Added `tools/vaisc_direct_error_check.vais` and moved the direct import
  reject plus List bounds trap checks out of `scripts/test-vaisc-direct.sh`.
- Added `tools/vaisc_direct_feature_check.vais` and moved the direct
  helper/control-flow, range `for`, struct-local, and struct ABI success
  fixtures out of `scripts/test-vaisc-direct.sh`.
- Expanded `tools/vaisc_direct_feature_check.vais` to cover direct local
  `List<Int>`, `Str`, `Char`, `parse_uint`/`parse_int`, local `Map<Int,Int>`,
  and local `List<Struct>` success fixtures, and removed those cases from the
  direct shell wrapper.
- Moved the remaining direct List ABI, assignment, and returned-list hoist
  fixtures into `tools/vaisc_direct_feature_check.vais`, reducing
  `scripts/test-vaisc-direct.sh` to a bootstrap wrapper around Vais-authored
  direct validators.
- Added `tools/vaisc_direct_gate.vais` and reduced
  `scripts/test-vaisc-direct.sh` again so the NV-C2 direct-emitter gate
  orchestration itself runs from Vais; shell now only provides the temp-dir
  bootstrap boundary.
- Reduced the single-tool focused shell wrappers for checker contract, NV-C0,
  NV-C1, NV-C3, host, native smoke, legacy compiler smoke, fixpoint tiers,
  parity, value corpus, embed, and normalizer checks to invoke their
  Vais-authored gates with `scripts/vaisc run`; the wrappers now only provide
  temp directories and environment-specific bootstrap arguments.
- Added `tools/normalize_stage_ir_check.vais` and reduced
  `scripts/test-normalize-stage-ir-vais.sh` to a bootstrap wrapper; sample IR,
  expected-output comparison, and replacement-shape assertions now run in Vais.
- Added `tools/embed_self_source_check.vais` and reduced
  `scripts/test-embed-self-source-vais.sh` to a bootstrap wrapper; fixture
  writing, helper invocation, trust-root generated-compiler builds, clang IR
  validation, and binary result assertions now run in Vais.
- Added `tools/vais_check_contract_check.vais` and reduced
  `scripts/test-vais-check-vais.sh` to a bootstrap wrapper; checker output
  counts, diagnostic pattern assertions, real-path checks, help checks, and
  public `scripts/vais-check` wrapper checks now run in Vais.
- Added `tools/fixpoint_tier_check.vais` and reduced
  `scripts/test-fixpoint.sh` plus `scripts/test-fixpoint2.sh` to bootstrap
  wrappers; their compact-program fixtures, raw-call embedding, trust-root
  compiler builds, emitted-IR clang validation, and result assertions now run
  in Vais.
- Added `tools/fixpoint_full_self_check.vais` and reduced
  `scripts/test-fixpoint-full-self.sh` to a bootstrap wrapper; full-source
  self-host retargeting, generated compiler builds/runs, emitted IR checks,
  final binary assertions, and normalized stage comparison now run in Vais.
- Added `tools/fixpoint_full_codegen_check.vais` and reduced
  `scripts/test-fixpoint-full.sh` to a bootstrap wrapper; the long full-codegen
  fixture catalog, stdout/trap cases, source-file checks, and IR shape
  assertions now run in Vais.
- Audited the remaining host boundaries after the full-codegen port; the
  remaining shell is limited to native C bootstrap, public command cache
  wrappers, release/CI orchestration, website build tooling, system tools, and
  temp-dir bootstrap wrappers.
- Fixed native front-contract probes to ignore unsupported-syntax markers inside
  string, raw-string, character literal, and comment text.
- Added `tools/compiler_smoke_check.vais` as the Vais-authored legacy
  self-host compiler smoke validator behind `scripts/test-compiler.sh`,
  replacing shell `sed` retargeting with Vais string rewriting and wiring the
  smoke into `scripts/test-release-gates.sh`.
- Added full-engine local `List<Str>` index reads, regenerated
  `compiler/self/vaisc_core.ll`, and covered the path through a Vais-authored
  stage IR normalizer.
- Added `tools/normalize_stage_ir.vais`, parity-gated it against the previous
  helper, and switched `scripts/test-fixpoint-full-self.sh` to use the Vais
  normalizer for stage1/stage2 IR comparison.
- Switched the focused self-source embedding and stage IR normalizer gates from
  external parity oracles to Vais-only behavioral and expected-output checks,
  and removed those helper checks from the release gate.
- Added native self-host trust-root handling to `scripts/vaisc`.
- Fixed native source-prep parity for one-line struct fields and multi-field
  struct lines, removed the internal self-host compiler escape hatch, removed
  the fallback branch from `scripts/vaisc`, and verified the embed, normalizer,
  fixpoint, full-codegen, and full self-host gates through the native path.
- Promoted single-byte `Char` literal equality plus explicit `Char` locals,
  helper parameters, and helper returns through the native direct engine and
  front contract as Int-compatible scalar values, and added
  `examples/e85_char_type.vais` to the release corpus.
- Promoted exclusive `..` and inclusive `..=` range `for` loops through the
  native direct engine and front contract, and added
  `examples/e86_for_loop.vais` to the release corpus.
- Promoted `break` and `continue` inside `while` and range `for` loops through
  the native direct engine, full self-host compiler, front contract, and parity
  gates, and added `examples/e87_break_continue.vais` to the release corpus.
- Promoted explicit `Bool` locals, helper parameters, helper returns, and unary
  `not` through the native direct engine, full self-host compiler, front
  contract, and parity gates, and added `examples/e88_bool_type.vais` to the
  release corpus.
- Promoted explicit `Str` locals, helper parameters, helper returns,
  reassignment, length, index, and equality through the native direct engine,
  full self-host compiler, front contract, and parity gates, and added
  `examples/e89_str_type.vais` to the release corpus.
- Promoted simple expression-arm `match` lowering for multi-field `Int` payload
  enum variants through the public front contract, full self-host compiler, and
  parity gates, and added `examples/e02_enum_payload.vais` to the release
  corpus.
- Fixed enum type-token rewriting so struct literal values such as
  `c: Color.Green` are not mistaken for type annotations, then promoted
  payload-free enum struct-field matching with `examples/e24_struct_enum_field.vais`.
- Promoted single-field struct payload enum lowering through the public front
  contract and parity gates, with `examples/e64_enum_struct_payload.vais`
  covering constructor literal extraction and payload field access.
- Promoted Int `match` literal arms with `_` catch-all lowering through the
  public front contract and parity gates, with
  `examples/e55_match_wildcard.vais` added to the release corpus.
- Promoted payload-free enum `match` with `_` catch-all through the public front
  contract and parity gates, with `examples/e90_enum_wildcard.vais` added to
  the release corpus.
- Promoted payload enum `match` with `_` catch-all through the public front
  contract and parity gates, with `examples/e120_enum_payload_wildcard.vais`
  added to the release corpus.
- Added `tools/vais_parity_check.vais` as the Vais-authored NV-C4 parity
  manifest harness and reduced `scripts/test-vaisc-parity.sh` to a bootstrap
  wrapper.
- Added `tools/vais_value_check.vais` as the Vais-authored value-corpus
  build/run/exit-code harness and reduced `scripts/test.sh` to a bootstrap
  wrapper.
- Added `tools/vais_host_check.vais` as the Vais-authored host
  file/path/string/process smoke harness and reduced `scripts/test-vaisc-host.sh`
  to a bootstrap wrapper.
- Added `tools/vaisc_smoke_check.vais` as the Vais-authored NV-C0 public
  compiler smoke harness and reduced `scripts/test-vaisc.sh` to a bootstrap
  wrapper.
- Added `tools/vaisc_native_check.vais` as the Vais-authored native-driver
  smoke harness and reduced `scripts/test-vaisc-native.sh` to a bootstrap
  wrapper.
- Strengthened the Vais checker contract gate to assert real file paths in
  diagnostics and clean output, then fixed the checker CLI path output to use
  explicit `Str` concatenation.

## 2026-06-18

- Added the first `vais.toml` package manifest slice for `name`, `version`, and
  `source` source-root resolution.
- Added local dependency package paths under `vais.toml` `[dependencies]`, with
  dependency imports resolving through native public driver paths.
- Added native gates for package manifest success,
  dependency imports, and manifest diagnostics.
- Specified the Phase 3 host file/path/process API boundary in
  `docs/design/HOST_IO.md` and listed the APIs as specified in
  `std/PRELUDE.md`.
- Implemented `fs_exists(path: Str) -> Bool`, `fs_write_text(path: Str, text:
  Str) -> Int`, and `fs_mkdirs(path: Str) -> Int` as the first host-backed file
  intrinsics for full-engine builds, with the native public driver injecting
  the LLVM declarations and linking a small host runtime.
- Added `scripts/test-vaisc-host.sh` for native
  temp-directory existence, directory creation, and text write checks, and wired
  it into the release gate.
- Added `fs_read_text(path: Str) -> Str` as the first `Str`-returning
  host-backed intrinsic, regenerated `compiler/self/vaisc_core.ll`, and
  extended `scripts/test-vaisc-host.sh` to verify text reads through full-engine
  runs.
- Added verified path helpers `fs_cwd()`, `fs_temp_dir()`, `path_join(...)`,
  `path_basename(...)`, and `path_dirname(...)` as `Str`-returning host-backed
  intrinsics, regenerated `compiler/self/vaisc_core.ll`, and extended the host
  smoke gate to validate native path behavior.
- Added verified `proc_run(argv: List<Str>) -> Int` as the first process
  intrinsic, including full-engine `List<Str>` local `push` support for argv
  construction, native-driver host runtime support, and host smoke coverage for
  `emit-ir`, `build`, and `run`.

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
  driver.
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
- Added `.vais` suffix validation to compiler and self-host helper paths.
- Added `tools/embed_self_source.vais` raw compact-program/call embedding and
  moved `scripts/test-fixpoint.sh`, `scripts/test-fixpoint2.sh`, and
  `scripts/test-fixpoint-full.sh` input generation onto the Vais helper.
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
  `emit-ir`, `build`, and `run` to the native public path.
- Added standalone native install, uninstall, package, and install/package gates.
- Added release archive workflow for source tags.
- Moved `--engine direct` onto the native driver and expanded it through Int
  helper calls, locals, assignment, `if`, and `while`.
- Expanded native direct mode with simple Int-field struct local literal,
  field read, and field write support.
- Expanded native direct mode with struct parameter and return helper ABI.
