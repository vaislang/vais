# Vais Roadmap

This file tracks current work only.

## Done

- Project path is `/Users/sswoo/study/projects/vais`.
- Checked-in language sources use `.vais`.
- `scripts/vaisc` is the canonical compiler command.
- `scripts/vais-check` is the canonical lint/error-help command, built from
  Vais source and protected by fixture contract gates.
- The workspace now exposes only Vais source and Vais commands.
- The compiler gates cover CLI smoke, front-contract diagnostics, direct LLVM emission, parity, and the value corpus.
- The trusted self-host tier is `compiler/self/fixpoint.vais`, `fixpoint2.vais`, `fixpoint3.vais`, and `fixpoint_full.vais`.
- `compiler/self/vaisc_core.ll` is the reusable self-host compiler core used by `scripts/vaisc`.
- The full compiler path reads `.vais` source files directly through the self-host core.
- Pure regeneration of `compiler/self/vaisc_core.ll` from `compiler/self/fixpoint_full.vais` is green.
- The native compiler and checker can be installed as standalone `vaisc` and
  `vais-check` binaries outside the checkout and packaged as a release archive.
- Source tag builds have a release archive workflow for standalone compiler and
  checker assets.
- The `v0.2.2` source tag produced a GitHub Release with Linux x64, macOS
  arm64, and macOS x64 standalone compiler archives.
- The native direct engine covers Int helper calls, locals, assignment, `if`,
  `while`, returns, simple Int-field struct locals, and struct parameter/return
  helpers through the native direct path.
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
- `List<T>.is_empty()` is promoted for the full self-host path and native
  direct engine, with gates for Int and declared-struct lists.
- `List<T>.last()` is promoted for non-empty lists in the full self-host path
  and native direct engine, with Int and declared-struct list gates.
- `List<T>.pop()` is promoted for non-empty lists in the full self-host path
  and native direct engine, with Int and declared-struct list gates.
- Indexed `List` reads/writes plus `last()` and `pop()` now trap at runtime on
  negative indexes, out-of-range indexes, or empty-list access.
- `Str` length, byte index, equality/inequality, `Bool` byte-classification
  helpers, and user-defined integer parsing patterns are promoted through the
  full self-host compiler, public front, parity, and native direct gates.
- Single-byte `Char` literals, equality, explicit annotations, helper
  parameters, and helper returns are promoted through public front, full
  self-host, parity, and native direct gates as Int-compatible scalar values.
- Named integer parsing helpers `parse_uint(s)` and `parse_int(s)` are promoted
  through the full self-host compiler, native direct engine, front gate, parity
  manifest, value corpus, and regenerated reusable core.
- The first `Map` slices are verified in the full self-host compiler and native
  direct engine for local `Map<Int,Int>` values with `{}`, assignment copy,
  `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and
  `len`, and local `Map<Int,Bool>` and `Map<Int,Char>` values with `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
  `contains`, and `len`, plus local `Map<Str,Int>` values with `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`,
  `get_opt(key)`, `contains`, and `len`, and local `Map<Str,Bool>` values
  with the same local string-key method surface, plus local `Map<Str,Char>`
  values with the same local string-key method surface.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>`
  parameters are verified by reference, so callees can mutate caller-visible
  maps. `Map<Str,Char>` parameters are also verified by reference.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>` return values can initialize explicitly annotated locals
  through caller-owned storage.
- Promoted prelude APIs have value-corpus examples, including local
  `Map<Int,Int>`, local `Map<Int,Bool>`, local `Map<Int,Char>`,
  local `Map<Str,Int>`, local `Map<Str,Bool>`, local `Map<Str,Char>`,
  `Map<Int,Bool>` parameter mutation, `Map<Int,Char>` parameter mutation,
  `Map<Str,Int>` parameter mutation, `Map<Str,Bool>` parameter mutation,
  `Map<Str,Char>` parameter mutation,
  `Map<Int,Int>` return-value local initialization, `Map<Int,Bool>`
  return-value local initialization, `Map<Int,Char>` return-value local
  initialization, `Map<Str,Int>` return-value local initialization, local
  `Map<Str,Bool>` string-key operations, `Map<Str,Bool>` return-value local
  initialization, local `Map<Str,Char>` string-key operations, concrete Map key removal, concrete Map scalar get_opt
  payloads, concrete Map clear and reuse, and
  `List<T>.is_empty()`, `last()`, and `pop()`.
- The full compiler path supports single-package local dotted imports such as
  `import math.add`, with gates for multi-file success, missing imports,
  duplicate symbols, and import cycles.
- The full compiler path supports the first `vais.toml` package manifest slice:
  required `name`, `version`, and `source` keys, source-root import resolution,
  and manifest diagnostics.
- The full compiler path supports local dependency package paths in `vais.toml`
  `[dependencies]`, with native gates for dependency imports and dependency
  manifest diagnostics.
- Phase 3 host file/path/process APIs are specified in `docs/design/HOST_IO.md`;
  `fs_exists`, `fs_read_text`, `fs_write_text`, `fs_mkdirs`, `fs_remove`,
  `fs_cwd`, `fs_temp_dir`, `path_join`, `path_basename`, and `path_dirname`
  are the first verified full-engine host file/path intrinsics, and
  `proc_argc() -> Int`, `proc_arg(index: Int) -> Str`,
  `proc_run(argv: List<Str>) -> Int`,
  `proc_run_env(argv: List<Str>, env: List<Str>) -> Int`,
  `proc_capture_stdout(argv: List<Str>) -> Str`,
  `proc_capture_stderr(argv: List<Str>) -> Str`, and
  `proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int`
  are the first verified process intrinsics. Program argv, child environment
  overrides, captured stdout/stderr, and status-plus-file capture are verified
  for full-engine `vaisc run` and binaries produced by `vaisc build`.
- Host-backed `Str` construction helpers `str_concat`, `str_slice`, and
  `str_byte` are verified through the same host gate so Vais-authored text
  transformation tools can be ported incrementally.
- Host-backed `Str` builder helpers `str_builder_new`, `str_builder_push`,
  `str_builder_append`, and `str_builder_finish` are verified through the host
  gate for large Vais-authored text transformation tools.
- Full-engine `Str` reassignment and user-defined `-> Str` helper returns are
  verified through the host gate.
- `tools/vais_check_core.vais` and `tools/vais_check_cli.vais` are the
  Vais-authored public checker sources. `scripts/vais-check` builds and runs
  them as the canonical lint/error-help command, release archives include the
  standalone `bin/vais-check` binary. `tools/vais_check_contract_check.vais`
  drives the focused checker fixture, CLI, path, help, and public wrapper
  contract gate; the shell entrypoint is only a bootstrap wrapper.
- `tools/embed_self_source.vais` is the Vais-authored self-source embedding
  helper. Its focused gate is driven by `tools/embed_self_source_check.vais`,
  which writes the fixtures, runs normalized and raw embedding, builds the
  generated compilers through the trust-root path, and verifies their emitted
  IR/binary results; the shell entrypoint is only a bootstrap wrapper.
- `tools/normalize_stage_ir.vais` is the Vais-authored stage IR normalizer.
  Its focused gate checks the expected normalized IR shape directly through the
  Vais helper, and the long full-source self-host gate uses it for stage1/stage2
  compiler IR comparison.
- Internal self-host helper builds now use the native `scripts/vaisc`
  trust-root path.
- `docs/design/MAP_ABI.md` specifies the future Map parameter, return, and
  generic expansion contract without promoting broader Map behavior.

## Current Reality

- The full compiler path emits LLVM IR through the self-host compiler source in `compiler/self/fixpoint_full.vais`.
- The direct engine is intentionally narrow and currently supports Int helpers,
  Bool/Str scalar helpers, locals, assignment, calls, `if`, inline
  `if { return ... }`, `while`, range `for`, `break`, `continue`, returns, `Str` literals, `Str.len()`, `Str`
  byte index, `Str` equality/inequality, `Char` literal equality, annotations,
  helper parameters, helper returns, named `parse_uint`/`parse_int`
  helpers, simple Int-field struct local literal/read/write, struct
  parameter/return helper ABI, and local
  `List<Int>` initialization plus `push`, `len`, `is_empty`, `last`, `pop`, index, `sum`, and
  `List<Int>` parameter reference, return value ABI, and inline list
  literal/constructor call and return values. Statement contexts, `if`,
  `else if`, and `while` conditions also lower `List<Int>`-returning helper
  calls before passing them to `List<Int>` parameters. Range `for` supports
  exclusive `..` and inclusive `..=` bounds through both full self-host and
  native direct paths, with `break` and `continue` lowered inside `while` and
  range `for` loops. Local `List<Struct>`
  values support typed `[]`, `list()`, list literal initialization, `push`,
  `len`, `is_empty`, `last`, `pop`, index, field reads/writes, parameter reference, return value ABI,
  inline list arguments, and returned-list argument lowering in statements plus
  `if`, `else if`, and `while` conditions. Context-typed list assignment is supported
  for `List<Int>` and `List<Struct>` locals and list parameters. Element
  assignment is supported for `List<Int>` and `List<Struct>`, including through
  list parameters. Local `Map<Int,Int>` values support `{}`, assignment copy,
  `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key) -> Option<Int>`,
  `contains`, and `len`; local `Map<Int,Bool>` values support `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
  `contains`, and `len`; local `Map<Int,Char>` values support `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
  `contains`, and `len`; local `Map<Str,Int>` values support `{}`,
  assignment copy, `insert`, `remove`, `clear`, `get(key, default)`,
  `get_opt(key)`, `contains`, and `len`; local `Map<Str,Bool>` values support
  the same local string-key method surface; local `Map<Str,Char>` values
  support the same local string-key method surface;
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>` parameters support
  reference mutation in both the full self-host compiler path and native direct
  engine. `Map<Str,Char>` parameters also support reference mutation.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  and `Map<Str,Bool>` return values can initialize explicitly annotated locals.
  Generic key/value forms are not claimed yet.
  The future Map ABI and generic expansion contract is specified
  in `docs/design/MAP_ABI.md`.
- The release compiler command uses a native host driver for `emit-ir`,
  `build`, and `run`; internal self-host helper gates use the same native
  compiler path.
- Standalone install, uninstall, package, and install/package verification
  scripts exist for the native compiler and checker binaries.
- Internal compiler gates no longer depend on a source pass-through helper.
- Full self-host lowering for runtime `Str` equality/inequality is gate-backed,
  and Vais-authored tools can use idiomatic `a == b` / `a != b` string
  comparisons.
- The long full-source self-host gate retargets compiler sources with the
  Vais-native embed helper.
- The long full-source self-host gate normalizes stage comparison IR with the
  Vais-native normalizer.
- Public documentation now starts at `README.md` and `docs/README.md`.
- `docs/reference/LANGUAGE.md` describes only the current gate-backed language surface.
- Local official website source was refreshed and rebuilt from the canonical Vais docs.
- Official site source now lives in `website/` in this repository.
- GitHub Pages workflow was added for `website/` build and artifact deployment.
- `vaislang.dev` deploys from the `website/` GitHub Pages workflow on `main`.
- `CHANGELOG.md` records the current `v0.3.1` source release baseline.
- GitHub `main` now points to the current Vais-only history; old remote `main`
  is preserved at `archive/old-main-2026-06-14`.

## Next Work

1. Expand the standard library only through gate-backed APIs.
2. Specify and implement file and process primitives needed for Vais-authored
   repository tools.
3. Replace host-only internal checks incrementally with Vais-backed tools where
   the language is strong enough.
4. Broaden types, collections, and control syntax without publishing ungated
   claims.
5. Move more compiler development and verification into the self-host tier while
   keeping native host responsibilities explicit.
6. Keep GitHub Releases, GitHub Pages, self-host regeneration, direct/full parity,
   and value gates green at each milestone.

## Vais v1 Completion Roadmap

This is the durable completion plan for turning the current Vais baseline into a
language/toolchain that can reasonably be called complete for a first stable
release. "Complete" means documented, implemented, tested, packaged, and
published from this repository without compatibility notes for older names or
alternate source extensions.

### Phase 0: Release Discipline

Goal: make every future capability land behind a repeatable release process.

- [x] 0.1 Define the next release line and tag policy in `CHANGELOG.md`,
  `README.md`, and release docs.
- [x] 0.2 Add a release checklist that runs native, install/package, direct,
  front, parity, value, and self-host regeneration gates before tagging.
- [x] 0.3 Prove one source tag produces a GitHub Release with standalone
  archives and a smoke-tested packaged `vaisc`.
- [x] 0.4 Keep `vaislang.dev` synced from repository docs for every release.

Done: a clean checkout can produce and verify a tagged release archive, and the
public site describes exactly that release.

### Phase 1: Standard Library Core

Goal: grow a small, reliable prelude instead of a large speculative API list.

- [x] 1.1a Promote verified `List<T>.is_empty()` across the full self-host path
  and native direct engine.
- [x] 1.1b Promote verified `List<T>.last()` across the full self-host path and
  native direct engine.
- [x] 1.1c Promote verified `List<T>.pop()` across the full self-host path and
  native direct engine.
- [x] 1.1d Define bounds-safe diagnostics or documented trap behavior for
  indexed list operations.
- [x] 1.2a Promote `Str` operations needed by real tools: `len`, index,
  equality, byte classification helpers, and user-defined integer parsing
  patterns.
- [x] 1.2b Decide and promote a named integer parsing prelude API, if it should
  be part of the public standard library instead of a user helper pattern.
- [x] 1.3a Specify the first `Map` slice and gate unsupported `Map` use with a
  clear front diagnostic.
- [x] 1.3b Promote native direct local `Map<Int,Int>` for construction,
  insert/replace, `get(key, default)`, `get_opt(key)`, `contains`, and `len`.
- [x] 1.3c Promote full self-host local `Map<Int,Int>` for the same surface.
- [x] 1.3d Specify `Map<K,V>` generic key/value lowering and ABI behavior before
  broadening.
- [x] 1.3e Promote local `Map<Int,Int>` assignment copy through full and direct
  gates.
- [x] 1.3f Promote local `Map<Int,Bool>` through concrete full/direct/front
  gates while keeping `get_opt` behind `Option<Bool>`.
- [x] 1.3g Promote local `Map<Int,Char>` through concrete full/direct/front
  gates while keeping `get_opt` behind `Option<Char>`.
- [x] 1.3h Promote `Map<Int,Int>` function parameters by reference through
  concrete full/direct/front gates while keeping Map returns gated until a
  concrete return slice is promoted.
- [x] 1.3i Promote `Map<Int,Bool>` function parameters by reference through
  concrete full/direct/front gates while keeping non-`Map<Int,Int>` returns
  and broader Map parameters gated.
- [x] 1.3j Promote `Map<Int,Char>` function parameters by reference through
  concrete full/direct/front gates while keeping Map returns gated until the
  next concrete slice.
- [x] 1.3k Broaden `Map<K,V>` only through concrete gate-backed slices:
  promote `Map<Int,Int>` return values for local initialization while keeping
  `Map<Int,Bool>`, `Map<Int,Char>`, and generic Map returns gated.
- [x] 1.3l Broaden `Map<K,V>` only through the next concrete gate-backed slice:
  promote `Map<Int,Bool>` return values for local initialization while keeping
  `Map<Int,Char>` and generic Map returns gated.
- [x] 1.3m Broaden `Map<K,V>` only through the next concrete gate-backed slice:
  promote `Map<Int,Char>` return values for local initialization while keeping
  generic Map returns gated.
- [x] 1.3n Broaden `Map<K,V>` only through the next concrete gate-backed slice:
  promote `remove(key)` for concrete `Map<Int,Int>`, `Map<Int,Bool>`, and
  `Map<Int,Char>` values while keeping generic Map behavior gated.
- [x] 1.3o Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `get_opt(key)` for `Map<Int,Bool>` and
  `Map<Int,Char>` match payloads while keeping generic Map behavior gated.
- [x] 1.3p Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `clear()` for concrete `Map<Int,Int>`,
  `Map<Int,Bool>`, and `Map<Int,Char>` values while keeping generic Map
  behavior gated.
- [x] 1.3q Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote local `Map<Str,Int>` string-key operations before
  parameter, return, and broader generic Map behavior.
- [x] 1.3r Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Int>` function parameters by reference
  while keeping `Map<Str,Int>` returns and broader generic Map behavior gated.
- [x] 1.3s Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Int>` return values for explicitly
  annotated local initialization while keeping broader `Map<Str,V>` and
  generic Map returns gated.
- [x] 1.3t Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote local `Map<Str,Bool>` string-key operations.
- [x] 1.3u Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Bool>` function parameters by reference
  while keeping broader generic Map behavior gated.
- [x] 1.3v Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Bool>` return values for explicitly
  annotated local initialization while keeping broader generic Map behavior
  gated.
- [x] 1.3w Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote local `Map<Str,Char>` string-key operations while
  keeping `Map<Str,Char>` parameters, returns, and broader generic Map behavior
  gated.
- [x] 1.3x Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice: promote `Map<Str,Char>` function parameters by reference
  while keeping `Map<Str,Char>` returns and broader generic Map behavior gated.
- [ ] 1.3y Continue `Map<K,V>` expansion only through the next concrete
  gate-backed slice.
- [x] 1.4 Add examples and value tests for every promoted prelude API.
- [x] 1.5 Update `std/PRELUDE.md` so "Verified" means compiler-gate protected.

Done: `std/PRELUDE.md` has no public "Verified" entry without a matching gate.

### Phase 2: Modules, Packages, And Imports

Goal: allow real projects to split code across files without inventing a large
package manager too early.

- [x] 2.1 Specify a minimal module model: file module names, import paths, symbol
  visibility, duplicate-name diagnostics, and cycle behavior.
- [x] 2.2 Implement single-package multi-file compilation for `scripts/vaisc`.
- [x] 2.3 Add `import` support for local package paths with deterministic
  ordering and stable diagnostics.
- [x] 2.4 Add package manifest support for name/version/source roots.
- [x] 2.5 Add local dependency package paths.
- [x] 2.6 Add package manifest examples, docs, gates, and source-root package
  smoke builds.
- [x] 2.7 Add local dependency examples, docs, gates, and package smoke builds.

Done: small multi-file and local dependency Vais projects build with
`scripts/vaisc build` and are covered by CI gates.

### Phase 3: File And Process Support

Goal: give Vais enough host interaction for repository tools and release
validation.

- [x] 3.1 Specify file read/write, path, temp directory, stdout/stderr, exit code,
  and process execution APIs.
- [x] 3.2 Implement the first host-backed intrinsic in the native driver without
  mixing it into pure compiler-core logic.
- [x] 3.3 Extend host-backed file intrinsics to text writes and directory
  creation.
- [x] 3.4 Add the first `Str`-returning host intrinsic for text file reads.
- [x] 3.5 Add `Str`-returning path helper intrinsics.
- [x] 3.6 Add argv-based process intrinsics.
- [x] 3.6a Add the first captured stdout process intrinsic.
- [x] 3.6b Add captured stderr process support for Vais-authored diagnostics
  harnesses.
- [x] 3.6c Add child-process environment override support for Vais-authored
  process checks.
- [x] 3.6d Add status-plus-stdout/stderr file capture for Vais-authored
  process checks without requiring a struct-returning host ABI.
- [x] 3.7 Port the simplest checker to Vais first.
- [x] 3.7a Add minimal `Str` construction helpers needed by Vais-authored
  repository tools.
- [x] 3.8 Port release archive packaging orchestration to Vais once the
  file/process APIs can read paths, capture platform commands, stage text docs,
  and run argv-based child processes.
- [x] 3.8b Port standalone install orchestration to Vais while keeping the
  initial uninstall path shell-native so removal does not require a compiler.
- [x] 3.8c Move parity, value-corpus, and host smoke gate logic into
  Vais-authored harnesses while preserving thin bootstrap wrappers.
- [x] 3.8d Move the NV-C0 public compiler smoke gate into a Vais-authored
  harness while preserving a thin bootstrap wrapper.
- [x] 3.8e Move the native driver smoke gate into a Vais-authored harness while
  preserving native C build script bootstrap.
- [x] 3.8f Move the NV-C3 diagnostics gate into a Vais-authored harness after
  adding captured stderr process support.
- [x] 3.8g Move the legacy self-host compiler smoke gate into a Vais-authored
  harness while preserving the shell bootstrap boundary.
- [x] 3.8h Move the NV-C1 front contract gate into a Vais-authored harness while
  preserving a thin bootstrap wrapper.
- [x] 3.8i Move the direct-engine no-Python PATH check into a Vais-authored
  harness after adding child environment process support.
- [x] 3.8j Move the direct-engine arithmetic/build/run smoke checks into a
  Vais-authored harness.
- [x] 3.8k Move the direct-engine import reject and List bounds trap checks into
  a Vais-authored harness using status-plus-file process capture.
- [x] 3.8l Move the direct helper/control-flow, range `for`, struct-local, and
  struct ABI success fixtures into a Vais-authored harness.
- [x] 3.8m Move direct local `List<Int>`, `Str`, `Char`,
  `parse_uint`/`parse_int`, local `Map<Int,Int>`, local `Map<Int,Bool>`, local
  `Map<Int,Char>`, and local `List<Struct>` success fixtures into the
  Vais-authored feature harness.
- [x] 3.8n Move the remaining direct List ABI, assignment, and returned-list
  hoist shell fixtures into the Vais-authored feature harness.
- [x] 3.8o Audit remaining shell wrappers and keep only bootstrap, process
  supervision, or platform-specific CI glue.
- [x] 3.8p Move the stage IR normalizer focused gate sample/expected fixture and
  shape checks into a Vais-authored check harness.
- [x] 3.8q Move the self-source embedding focused gate fixture generation,
  trust-root build/run checks, and generated compiler result assertions into a
  Vais-authored check harness.
- [x] 3.8r Move the checker focused gate output-count, diagnostic-pattern,
  path, help, and public-wrapper assertions into a Vais-authored contract
  harness.
- [x] 3.8s Move the short `fixpoint.vais` and `fixpoint2.vais` tier fixture
  lists, raw-call embedding, trust-root compiler builds, emitted-IR clang
  checks, and result assertions into a Vais-authored harness.
- [x] 3.8t Add verified `fs_remove(path)` and port standalone uninstall
  orchestration to `tools/uninstall_vaisc.vais`.
- [x] 3.8u Move standalone install/package verification assertions into
  `tools/vaisc_install_check.vais`.
- [x] 3.8v Move the NV-C2 direct-emitter gate orchestration into
  `tools/vaisc_direct_gate.vais`, leaving the shell file as only the temp-dir
  bootstrap wrapper.
- [x] 3.8w Reduce single-tool focused shell wrappers to temp-dir bootstrap
  wrappers that invoke their Vais-authored gates through `scripts/vaisc run`.
- [x] 3.8x Move the long full-source self-host compiler orchestration into
  `tools/fixpoint_full_self_check.vais`, leaving the shell file as a
  temp-directory bootstrap wrapper.
- [x] 3.8y Move the long full-codegen regression runner into
  `tools/fixpoint_full_codegen_check.vais`, leaving the shell file as a
  temp-directory bootstrap wrapper.
- [x] 3.8z Audit the remaining host boundaries and leave native C build,
  public command cache wrappers, release-gate/CI orchestration, website build,
  tar/install/clang system tools, and temp-dir bootstrap wrappers explicit.
- [x] 3.9 Keep public checker release gates on the Vais-authored checker.

Done: the public checker, release archive packager, standalone installer,
parity manifest validator, value-corpus validator, host smoke validator, NV-C0
public compiler smoke validator, front contract validator, native driver smoke
validator, NV-C3 diagnostics validator, legacy self-host compiler smoke
validator, direct no-Python environment validator, direct arithmetic/build
smoke validator, direct reject/trap validator, direct feature validator, and
direct-emitter gate runner run from Vais source. The direct feature validator
now covers the scalar,
collection, struct, helper, list ABI, assignment, and returned-list hoist
success fixture groups. The checker contract, stage IR normalizer, and
self-source embed focused gates now use Vais-authored check harnesses. The
short fixpoint tier gates also use a shared Vais-authored harness, the
full-codegen regression runner executes its 200 fixture cases plus source-file
and IR shape checks from a Vais-authored harness, and the full-source self-host
gate retargets compiler sources, builds generated compilers, validates emitted
IR, and compares normalized stage output from a Vais-authored harness.
Standalone uninstall plus install/package verification are backed by Vais
tools. The focused, full-codegen, and self-host shell entrypoints now use
`scripts/vaisc run` directly and remain only as temp-directory bootstrap
boundaries. The remaining host boundaries are audited and intentionally limited
to native C bootstrap/driver code, public command cache wrappers,
release-gate/CI orchestration, website build tooling, tar/install/clang system
tools, and temporary directory setup.

### Phase 4: Broader Language Surface

Goal: expand the language deliberately while avoiding unsupported public claims.

- [x] 4.1 Stabilize `Bool`, `Str`, and `Char` as first-class surface types across
  full and direct gates where feasible.
- [x] 4.1a Promote single-byte `Char` literal equality plus explicit `Char`
  local annotations, helper parameters, and helper returns through public
  front, native direct, full self-host, and parity gates.
- [x] 4.1b Promote explicit `Bool` local annotations, helper parameters, helper
  returns, and unary `not` through public front, native direct, full self-host,
  and parity gates.
- [x] 4.1c Promote explicit `Str` local annotations, helper parameters, helper
  returns, reassignment, length, index, and equality through public front,
  native direct, full self-host, and parity gates.
- [ ] 4.2 Add broader enum payloads and pattern/match forms after the current
  simple return-arm shape is fully gated.
- [x] 4.2a Promote simple expression-arm `match` lowering for multi-field `Int`
  payload enum variants through public front, full self-host, parity, and value
  gates.
- [x] 4.2b Promote payload-free enum values stored in simple struct fields and
  matched through field access through public front, full self-host, parity, and
  value gates.
- [x] 4.2c Promote single-field struct payload enum lowering for constructor
  literals and payload field access through public front, parity, and value
  gates.
- [x] 4.2d Promote Int `match` literal arms with `_` catch-all lowering through
  public front, parity, and value gates.
- [x] 4.2e Promote payload-free enum `match` with `_` catch-all through public
  front, parity, and value gates.
- [x] 4.3a Promote exclusive `..` and inclusive `..=` range `for` loops through
  public front, native direct, full self-host, and parity gates.
- [x] 4.3b Decide `break` and `continue` semantics and lower them through both
  full and direct paths where claimed.
- [ ] 4.4 Expand collections with `Map`, `Option`, and `Result` only after syntax,
  ABI, and diagnostics are specified.
  - [x] Promote the first `Option<Int>` `Some`/`None` helper-return and
    statement-match slice.
  - [x] Promote the first `Result<Int,Int>` `Ok`/`Err` helper-return and
    statement-match slice.
  - [x] Promote `Option<Int>` expression-form match binding.
  - [x] Promote `Result<Int,Int>` expression-form match binding.
  - [x] Promote `Option<Int>` local-binding `?` propagation for both success
    and `None` paths.
  - [x] Promote `Result<Int,Int>` local-binding `?` propagation for both
    success and error paths.
  - [x] Promote local `Map<Int,Int>.get_opt(key) -> Option<Int>` on the full
    compiler path and native direct engine.
  - [x] Promote local `Map<Int,Int>` assignment copy on the full compiler path
    and native direct engine.
  - [x] Promote local `Map<Int,Bool>` construction, assignment copy, `insert`,
    `get(key, default)`, `contains`, and `len` on the full compiler path and
    native direct engine.
  - [x] Promote local `Map<Int,Char>` construction, assignment copy, `insert`,
    `get(key, default)`, `contains`, and `len` on the full compiler path and
    native direct engine.
  - [x] Promote `Map<Int,Int>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Int,Bool>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Int,Char>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Int,Int>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote `Map<Int,Bool>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote `Map<Int,Char>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote `remove(key)` for concrete `Map<Int,Int>`, `Map<Int,Bool>`, and
    `Map<Int,Char>` values on the full compiler path and native direct engine.
  - [x] Promote `get_opt(key)` for `Map<Int,Bool>` and `Map<Int,Char>` match
    payloads on the full compiler path and native direct engine.
  - [x] Promote local `Map<Str,Int>` construction, assignment copy,
    `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
    `contains`, and `len` on the full compiler path and native direct engine.
  - [x] Promote `Map<Str,Int>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Str,Int>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote local `Map<Str,Bool>` construction, assignment copy,
    `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
    `contains`, and `len` on the full compiler path and native direct engine.
  - [x] Promote `Map<Str,Bool>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Promote `Map<Str,Bool>` return values for explicitly annotated local
    initialization on the full compiler path and native direct engine.
  - [x] Promote local `Map<Str,Char>` construction, assignment copy,
    `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
    `contains`, and `len` on the full compiler path and native direct engine.
  - [x] Promote `Map<Str,Char>` function parameters by reference on the full
    compiler path and native direct engine.
  - [x] Gate unsupported `Option`/`Result` generic forms with front diagnostics.
- [ ] 4.5 Keep unsupported syntax behind `scripts/vais-check` and front-contract
  diagnostics until promoted.
  - [x] Add checker guidance for Rust-style top-level `use` and `pub` forms.
  - [x] Add front diagnostics for unverified Map function parameters and return
    values.
  - [x] Add front and direct diagnostics for unverified `Map<Int,Int>` value
    assignment.
  - [x] Add front and direct diagnostics for unverified non-`Map<Int,Int>`
    `get_opt` until matching `Option` payload slices are verified.

Done: `docs/reference/LANGUAGE.md` describes a coherent v1 surface, and every
listed feature has examples plus compiler gates.

### Phase 5: Self-Host Expansion

Goal: make the self-host compiler own more of the actual compiler behavior over
time.

- [ ] 5.1 Keep `compiler/self/fixpoint_full.vais` and `vaisc_core.ll`
  regeneration green after each language expansion.
- [ ] 5.2 Move front-contract validation that belongs to the compiler into
  self-host Vais code once the language can express it cleanly.
- [ ] 5.3 Move more diagnostics and source preparation out of the host driver while
  keeping OS-facing file/process work behind explicit host APIs.
- [ ] 5.4 Add stage comparison gates for self-host output where deterministic IR
  is practical.

Done: the compiler can rebuild its checked-in core from Vais source, and the
native host driver is limited to CLI, OS integration, and linking duties.

### Phase 6: Stable v1 Release

Goal: publish a coherent first stable Vais release.

- [ ] 6.1 Freeze the v1 language reference and prelude reference.
- [ ] 6.2 Cut a release candidate tag and attach verified standalone archives.
- [ ] 6.3 Run all release, direct/full, install/package, website, and self-host
  gates from a clean checkout.
- [ ] 6.4 Publish final docs/site copy from repository canonical docs.
- [ ] 6.5 Cut the final v1 tag and verify the GitHub Release assets and
  `vaislang.dev` content.

Done: users can install `vaisc`, read the v1 docs, compile the gate-backed
examples, and reproduce the release archive from source.

### Execution Rules

- Work phase order is dependency order. Do not jump to later public claims unless
  their gates and docs are also updated.
- Each milestone must update `ROADMAP.md`, `CHANGELOG.md`, canonical docs, and
  website copy when public behavior changes.
- Direct engine growth is valuable, but the full self-host path remains the
  language authority unless a direct slice is explicitly promoted.
- Host-tool reduction is not an isolated cleanup task; it depends on
  file/process support and Vais-backed replacement tools.
- Release tags are public state. Create or move tags only as a deliberate
  release milestone.

### Current First Executable Milestone

The current concrete slice moves the Vais checker from a ported rule slice to a
public command protected by its own fixture contract:

- [x] Add a release checklist document and wire it to the current gate commands.
- [x] Confirm the release archive workflow publishes archives for a chosen tag.
- [x] Decide the next release version before creating any public tag.
- [x] Promote the first small standard-library `List<T>` API slice with gates.
- [x] Promote the next `List<T>` API slice, `pop()`, with full/direct/docs
  coverage.
- [x] Define the next `List<T>` behavior slice: empty-list and out-of-range
  runtime trap behavior.
- [x] Promote the next Phase 1 slice: `Str` length/index/equality helpers and
  byte-classification utilities needed by real tools.
- [x] Decide and promote the named integer parsing prelude API.
- [x] Specify the minimal `Map<Int,Int>` design and gate unsupported `Map` use.
- [x] Promote native direct local `Map<Int,Int>` construction and local
  operations.
- [x] Promote the next Phase 1 slice: full self-host local `Map<Int,Int>`.
- [x] Promote the next concrete local Map slice: `Map<Int,Bool>`.
- [x] Promote the next concrete local Map slice: `Map<Int,Char>`.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Int>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Bool>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Char>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Int>` return values.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Bool>` return values.
- [x] Promote the next concrete Map ABI slice: `Map<Int,Char>` return values.
- [x] Promote the next concrete Map method slice: `remove(key)` for concrete
  `Map<Int,V>` values.
- [x] Promote the next concrete Map Option slice: `get_opt(key)` for
  `Map<Int,Bool>` and `Map<Int,Char>` match payloads.
- [x] Promote the next concrete Map method slice: `clear()` for concrete
  `Map<Int,V>` values.
- [x] Promote the next concrete local Map key slice: `Map<Str,Int>`.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Int>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Int>` return values.
- [x] Promote the next concrete local Map value slice: `Map<Str,Bool>`.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Bool>` parameters by
  reference.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Bool>` return values.
- [x] Promote the next concrete local Map value slice: `Map<Str,Char>`.
- [x] Promote the next concrete Map ABI slice: `Map<Str,Char>` parameters by
  reference.
- [x] Add release-corpus examples for the promoted prelude API surface.
- [x] Specify the next Phase 1 slice: Map ABI/generic expansion or defer to the
  Phase 2 module model.
- [x] Specify the minimal Phase 2 module/import/package model and reject
  unimplemented module syntax with public front diagnostics.
- [x] Implement single-package multi-file compilation for `scripts/vaisc`.
- [x] Add local import support with missing-import, duplicate-symbol, and
  import-cycle diagnostics.
- [x] Add the minimal package manifest slice.
- [x] Add local package dependency paths.
- [x] Specify the minimal Phase 3 file/process API needed for repository
  validation tools.
- [x] Implement the first native-driver host I/O intrinsic smoke gate.
- [x] Extend the host runtime beyond `fs_exists` to text writes and directory
  creation.
- [x] Extend host support to text reads.
- [x] Extend host support to path helpers.
- [x] Extend host support to argv-based process execution.
- [x] Port the smallest checker slice to Vais.
- [x] Expand the Vais checker slice to the current public checker fixture
  catalog.
- [x] Add line/column-aware Vais checker diagnostics.
- [x] Add a Vais-backed checker CLI path that can receive a target file path,
  return a normal issue/no-issue status, and remain gated by fixture contracts.
- [x] Promote the Vais checker CLI to the public `scripts/vais-check` command
  and package it as a standalone `bin/vais-check` binary.
- [x] Keep public-facing docs and release gates on the Vais-authored checker.
- [x] Add minimal host-backed `Str` construction helpers for future Vais tool
  ports.
- [x] Add full-engine `Str` reassignment and user-defined `-> Str` returns.
- [x] Build the parity manifest and value-corpus validators in Vais so release
  gates depend on Vais-native harnesses.

## Completed Milestone: Phase 3 Host API Specification

Mode: sequential

- [x] 1. Define the boundary between host-backed standard library intrinsics and
  pure compiler-core logic.
- [x] 2. Specify text file APIs for existence checks, whole-file reads,
  whole-file writes, and directory creation.
- [x] 3. Specify path helpers for current directory, temporary directory, joins,
  basenames, and dirnames.
- [x] 4. Specify argv-based process execution and captured process output without
  shell expansion.
- [x] 5. Mark the broad APIs as specified in canonical docs and identify the
  first checker port target.

## Completed Milestone: Local Dependency Package Paths

Mode: sequential

- [x] 1. Parse optional `vais.toml` `[dependencies]` string entries.
- [x] 2. Resolve dependency aliases to local package source roots with their own
  `vais.toml` manifests.
- [x] 3. Resolve dependency-internal plain imports under the dependency package
  source root.
- [x] 4. Reject missing dependency manifests, unsafe dependency paths, and
  dependency cycles with P4 diagnostics.
- [x] 5. Add dependency examples, canonical docs, website copy, and
  front-contract gates for native paths.

## Completed Milestone: Package Manifest Source Roots

Mode: sequential

- [x] 1. Search for nearest `vais.toml` from the entry file directory upward.
- [x] 2. Parse required `name`, `version`, and `source` string keys.
- [x] 3. Resolve static dotted imports under the manifest source root.
- [x] 4. Reject missing keys, unsafe source paths, missing source directories,
  and entries outside the source root with P4 diagnostics.
- [x] 5. Add package examples, canonical docs, website copy, and front-contract
  gates for native paths.

## Completed Milestone: Single-Package Local Imports

Mode: sequential

- [x] 1. Resolve static dotted `import` paths under the entry file directory.
- [x] 2. Merge imported modules before the entry source for full-engine builds.
- [x] 3. Keep direct-engine builds single-file.
- [x] 4. Reject missing imports, duplicate top-level symbols, and import cycles
  with P4 diagnostics.
- [x] 5. Add a multi-file example and front-contract gates for native paths.

## Completed Milestone: Minimal Module Model Specification

Mode: sequential

- [x] 1. Specify file-derived module names, local dotted import paths, symbol
  visibility, duplicate-name diagnostics, and cycle behavior.
- [x] 2. Keep `Map<K,V>` generic/ABI expansion deferred until its lowering and
  ABI are specified separately.
- [x] 3. Add front diagnostics for reserved `module` and `package` syntax and
  use the spec as the import implementation contract.
- [x] 4. Sync canonical docs, website copy, roadmap, worklog, and changelog.

## Completed Milestone: Prelude API Value Examples

Mode: sequential

- [x] 1. Replace stale Map example syntax with the verified local
  `Map<Int,Int>` API.
- [x] 2. Add a release-corpus List example for `is_empty()`, `last()`, and
  `pop()`.
- [x] 3. Promote both examples in `tools/vaisc-parity.tsv`.
- [x] 4. Keep the examples README and roadmap aligned with the value corpus.

## Completed Milestone: Local Map Slices

Mode: sequential

- [x] 1. Parse `Map<Int,Int>` local annotations in the direct engine.
- [x] 2. Lower `let m: Map<Int,Int> = {}` to a native local map value.
- [x] 3. Lower `m.insert(key, value)` statements with replace-on-existing-key
  behavior.
- [x] 4. Lower `m.get(key, default)`, `m.get_opt(key)`, `m.contains(key)`, and
  `m.len()` expressions.
- [x] 5. Gate direct emitted helper symbols and runtime value behavior.
- [x] 6. Lower the same local surface in the full self-host compiler and
  regenerate the reusable compiler core.
- [x] 7. Keep front diagnostics explicit about verified concrete Map slices;
  non-`Map<Int,Int>` returns and generic key/value forms stay
  rejected.
- [x] 8. Promote local `Map<Int,Int>` assignment copy while keeping Map
  returns and generic key/value forms rejected.

### Task Briefs

#### 1. Full self-host Map<Int,Int> lowering

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: match the direct local Map surface without adding generic or
  ABI claims; regenerate the reusable compiler core after the source change.
- Done: full self-host gates pass a local `Map<Int,Int>` example returning
  the same deterministic value as the direct gate.

#### 2. Map ABI and generic expansion

- Target files: `tools/vaisc_native.c`, `compiler/self/fixpoint_full.vais`,
  `docs/reference/LANGUAGE.md`, `std/PRELUDE.md`.
- Requirements: specify and gate Map parameters, return values, generic
  key/value support, and any broader `Option`/`Result` integration before
  publishing broader claims.
- Status: `docs/design/MAP_ABI.md` now specifies ownership, assignment,
  parameter, return, monomorphic helper, and expansion-order rules. Local
  `Map<Int,Int>` assignment copy and the local `Map<Int,Bool>` and
  `Map<Int,Char>` scalar-value slices are verified. `Map<Str,Int>` is
  verified for string-key local operations, parameter reference mutation, and
  return-value local initialization. Local `Map<Str,Bool>` string-key
  operations, parameter reference mutation, and return-value local
  initialization are verified. Local `Map<Str,Char>` string-key operations and
  parameter reference mutation are verified while `Map<Str,Char>` return ABI
  support remains gated; broader `Map<Str,V>` and generic Map behavior still
  require direct and full gates before publication.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, and `Map<Str,Int>`
  parameter reference mutation is verified.

## Completed Milestone: Map ABI and Generic Expansion Specification

Mode: sequential

- [x] 1. Keep Map expansion limited to explicitly verified concrete local
  slices.
- [x] 2. Specify Map assignment as value-copy instead of aliasing.
- [x] 3. Specify Map parameter mutation as reference-based, matching collection
  parameter behavior.
- [x] 4. Specify Map returns through caller-owned output storage or equivalent
  direct-engine lowering.
- [x] 5. Define monomorphic concrete helper families as the path for future
  `Map<K,V>` slices.
- [x] 6. Keep broader Map forms behind front/direct diagnostics until each slice
  has full gates.

## Completed Milestone: Map design and front gate contract

Mode: sequential

- [x] 1. Keep `Map<K,V>` out of the verified surface until compiler gates cover
  it.
- [x] 2. Define the first implementation target as `Map<Int,Int>` only.
- [x] 3. Choose explicit-empty construction with `let m: Map<Int,Int> = {}`.
- [x] 4. Choose `insert(key, value)` for insert/replace, `get(key, default)` for
  lookup without `Option`, `contains(key)` for presence, and `len()` for
  cardinality.
- [x] 5. Add front-gate diagnostics so unsupported public `Map` use fails
  clearly outside the verified local `Map<Int,Int>` slice.

### Task Briefs

#### 1. Concrete local Map implementation slices

- Target files: `tools/vaisc_native.c`.
- Requirements: local `Map<Int,Int>` values support `{}`, assignment copy,
  `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and
  `len`; local `Map<Int,Bool>` values support `{}`, assignment copy, `insert`,
  `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and
  `len`; local `Map<Int,Char>` values support the same surface without
  publishing broader generic Map return-value ABI claims. Local `Map<Str,Int>`
  values support the same local method surface with string keys and
  return-value local initialization. Local `Map<Str,Bool>` values support the
  same local method surface with string keys, parameter reference mutation, and
  return-value local initialization. Local `Map<Str,Char>` values support the
  same local method surface with string keys and parameter reference mutation
  while keeping `Map<Str,Char>` returns, broader `Map<Str,V>`, and generic Map
  returns gated.
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`,
  `Map<Str,Int>`, `Map<Str,Bool>`, and `Map<Str,Char>` parameters are passed by
  reference and may be mutated by callees.
- Done: native direct gates pass a local map example returning a deterministic
  value, and full self-host gates pass the same local map behavior.

#### 2. Map docs and release claims

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `scripts/test-vaisc-front.sh`, `website/index.html`.
- Requirements: docs distinguish verified local concrete Map slices from
  unsupported generic and ABI Map behavior.
- Done: `scripts/test-vaisc-front.sh` accepts local `Map<Int,Int>`,
  `Map<Int,Bool>`, `Map<Int,Char>`, local `Map<Str,Int>`, local
  `Map<Str,Bool>`, local `Map<Str,Char>`,
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>`, and `Map<Str,Char>`
  parameters while rejecting unsupported generic `Map<K,V>` forms;
  `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>` return values are accepted only for the concrete
  gate-backed slices; docs/site do not imply a verified generic `Map<K,V>` or
  `Map<Str,Char>` ABI.

## Completed Milestone: Named integer parsing prelude helpers

Mode: sequential

- [x] 1. Define `parse_uint(s: Str) -> Int` as leading unsigned decimal parsing
  that stops at the first non-decimal byte and returns `0` for empty/no-digit
  input.
- [x] 2. Define `parse_int(s: Str) -> Int` as optional leading `-` plus the same
  decimal parsing behavior.
- [x] 3. Lower both helpers through the full self-host compiler and regenerate
  `compiler/self/vaisc_core.ll`.
- [x] 4. Lower both helpers through the native direct engine.
- [x] 5. Add front, direct, full self-host, parity, and value gates with
  `examples/e83_parse_helpers.vais`.
- [x] 6. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, examples index, and website copy.

### Task Briefs

#### 1. Full and direct compiler support

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `tools/vaisc_native.c`.
- Requirements: `parse_uint` and `parse_int` are named prelude helpers, not
  user-defined example helpers; the full path must emit reusable helper IR and
  the direct path must stay native-only.
- Done: full codegen emits `@__vais_parse_uint` and `@__vais_parse_int`; direct
  mode rewrites calls to native helpers and verifies `Str` arguments.

#### 2. Gates and public docs

- Target files: `scripts/test-fixpoint-full.sh`,
  `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-direct.sh`,
  `tools/vaisc-parity.tsv`, `examples/e83_parse_helpers.vais`,
  `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`, `website/index.html`.
- Requirements: the API is public only when examples and release gates protect
  both full and direct behavior.
- Done: the named helpers are covered by full, front, direct, parity, and value
  tests.

## Completed Milestone: Str tool-helper slice

Mode: sequential

- [x] 1. Allow public front-contract scalar helper signatures with `Int`,
  `Bool`, and `Str`.
- [x] 2. Lower native direct `Str` literals, locals, parameters, return values,
  `s.len()`, `s[i]`, `a == b`, and `a != b`.
- [x] 3. Gate `Bool` byte-classification helpers and user-defined integer
  parsing over `Str`.
- [x] 4. Promote `examples/e48_string_index.vais`,
  `examples/e70_parse_uint.vais`, and `examples/e72_identifier_scan.vais` in
  the parity manifest.
- [x] 5. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, and website copy.

### Task Briefs

#### 1. Front and direct scalar surface

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-front.sh`,
  `scripts/test-vaisc-direct.sh`.
- Requirements: keep `fn main() -> Int`, but allow helper signatures and locals
  for `Int`, `Bool`, and `Str`; direct mode must stay native-only.
- Done: front and direct gates cover `Str` params/locals, `Bool` classifier
  helpers, and native direct lowering.

#### 2. String operations and tool patterns

- Target files: `tools/vaisc_native.c`, `tools/vaisc-parity.tsv`,
  `examples/e48_string_index.vais`, `examples/e70_parse_uint.vais`,
  `examples/e72_identifier_scan.vais`.
- Requirements: protect `s.len()`, `s[i]`, string equality/inequality,
  byte-classification helpers, and parse/identifier-scan tool shapes.
- Done: direct and parity gates cover string index, string equality, parse_uint,
  and identifier scanning.

#### 3. Documentation and roadmap sync

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: document the promoted `Str` helper surface as gate-backed while
  leaving any named integer parsing prelude API as a follow-up decision.

## Completed Milestone: List bounds trap behavior

Mode: sequential

- [x] 1. Add full self-host runtime trap lowering for invalid `List` index
  reads/writes, `last()` on an empty list, and `pop()` on an empty list.
- [x] 2. Add native direct checked-index helpers for `List<Int>` and
  `List<Struct>` reads/writes plus checked `last()` and `pop()`.
- [x] 3. Gate trap behavior with full self-host and native direct invalid-list
  access tests.
- [x] 4. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, and website copy.

### Task Briefs

#### 1. Full compiler bounds traps

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: emit `llvm.trap` before out-of-range list GEPs and before
  empty-list `last()`/`pop()` length mutation.
- Done: full gates cover invalid scalar list index, empty scalar `last()`,
  empty scalar `pop()`, and empty struct-list `last()`.

#### 2. Native direct bounds traps

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`.
- Requirements: keep direct mode native-only, avoid double-evaluating index
  expressions, and check `pop()` before length mutation.
- Done: direct gates cover invalid `List<Int>` index, empty `last()`, and empty
  `pop()`.

#### 3. Documentation and gate sync

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: document trap behavior as the current release-surface contract,
  not as future work.

## Completed Milestone: List pop API

Mode: sequential

- [x] 1. Add `List<T>.pop()` lowering to the full self-host compiler for
  non-empty scalar lists and struct-list local binding.
- [x] 2. Add native direct `List<Int>` and `List<Struct>` `pop()` expression
  support with type inference and deterministic prelude temporaries.
- [x] 3. Gate local and parameter `List<Int>.pop()` plus struct-list
  `let item = xs.pop()` usage, including caller-visible length mutation.
- [x] 4. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, and website copy.

### Task Briefs

#### 1. Full compiler pop API

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: compile `xs.pop()` by reading `len - 1`, returning that element,
  and storing the decremented length for local and parameter lists.
- Done: full gates cover `List<Int>.pop()` through a `List<Int>` parameter and
  `List<Tok>.pop()` through local and parameter struct-list bindings.

#### 2. Native direct pop API

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`,
  `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: keep direct mode native-only, infer `xs.pop()` as the list
  element type, and sequence mutation through generated temporaries.
- Done: direct gates cover `List<Int>.pop()` locals and parameters plus
  `List<Box>.pop()` binding.

#### 3. Documentation and gate sync

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements at that milestone: document only the non-empty-list API.
  Bounds behavior is now covered by the completed List bounds trap behavior
  milestone above.

## Completed Milestone: List last API

Mode: sequential

- [x] 1. Add `List<T>.last()` lowering to the full self-host compiler for
  non-empty scalar lists and struct-list local binding.
- [x] 2. Add native direct `List<Int>` and `List<Struct>` `last()` expression
  support with type inference.
- [x] 3. Gate local and parameter `List<Int>.last()` plus struct-list
  `let item = xs.last()` usage.
- [x] 4. Sync `std/PRELUDE.md`, the language reference, changelog, roadmap,
  worklog, and website copy.

### Task Briefs

#### 1. Full compiler last API

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: compile `xs.last()` by reading `len - 1` and reusing existing
  list buffer/index lowering; support struct-list values by binding the result
  to a local before field reads.
- Done: full gates cover `List<Int>.last()` through a `List<Int>` parameter and
  `List<Tok>.last()` through local and parameter struct-list bindings.

#### 2. Native direct last API

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`,
  `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: keep direct mode native-only, infer `xs.last()` as the list
  element type, and reject malformed calls in the rewrite path.
- Done: direct gates cover `List<Int>.last()` locals and parameters plus
  `List<Box>.last()` binding.

#### 3. Documentation and gate sync

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements at that milestone: document only the non-empty-list API. `pop()`
  is now covered by the completed List pop API milestone above, and bounds
  behavior is covered by the completed List bounds trap behavior milestone.

## Completed Milestone: List is_empty API

Mode: sequential

- [x] 1. Add `List<T>.is_empty()` lowering to the full self-host compiler.
- [x] 2. Regenerate `compiler/self/vaisc_core.ll` from
  `compiler/self/fixpoint_full.vais`.
- [x] 3. Add native direct `List<Int>` and `List<Struct>` `is_empty()` support.
- [x] 4. Gate the API in full, front, direct, and diagnostic test suites.
- [x] 5. Sync `std/PRELUDE.md`, the language reference, and website copy.

### Task Briefs

#### 1. Full compiler list API

- Target files: `compiler/self/fixpoint_full.vais`,
  `compiler/self/vaisc_core.ll`, `scripts/test-fixpoint-full.sh`.
- Requirements: compile `xs.is_empty()` for local and parameter lists without
  relying on a broad method fallback.
- Done: full gates cover `List<Int>.is_empty()` and declared-struct
  `List<T>.is_empty()` returning the expected boolean-as-Int values.

#### 2. Native direct list API

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`,
  `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: keep public direct mode native-only and reject malformed
  `is_empty` calls with explicit diagnostics.
- Done: direct gates cover local Int and struct lists, and front/error gates
  document the promoted method surface.

#### 3. Documentation and release gate

- Target files: `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`,
  `website/index.html`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs name only the gate-backed API and leave remaining
  list work as roadmap items.
- Done: `bash scripts/test-release-gates.sh` passed after the compiler/core
  changes.

## Completed Milestone: v0.2.2 Source Release

Mode: sequential

- [x] 1. Promote release metadata to a dated `v0.2.2` changelog entry.
- [x] 2. Run the full pre-tag release gate and produce the local standalone
  archive.
- [x] 3. Push the annotated `v0.2.2` source tag and verify the release archive
  workflow.
- [x] 4. Verify the GitHub Pages deploy and live `vaislang.dev` release copy.

### Task Briefs

#### 1. Release metadata

- Target files: `tools/vaisc_native.c`, `CHANGELOG.md`,
  `docs/release/RELEASE_CHECKLIST.md`, `website/package.json`,
  `website/package-lock.json`.
- Requirements: make the native compiler, changelog, release checklist, and
  website package agree on the `v0.2.2` source release line.
- Done: `scripts/vaisc --version` reports `0.2.2` through the native driver, and
  the changelog records `v0.2.2 - 2026-06-15`.

#### 2. Release verification

- Target files: `.github/workflows/release-archives.yml`,
  `scripts/test-release-gates.sh`, `website/`.
- Requirements: prove the tag path publishes standalone archives and the live
  website remains synced with the repository release docs.
- Done: `bash scripts/test-release-gates.sh` passed, `v0.2.2` published
  `vais-0.2.2-linux-x64.tar.gz`, `vais-0.2.2-darwin-arm64.tar.gz`, and
  `vais-0.2.2-darwin-x64.tar.gz`, and the `Deploy Website` workflow succeeded
  for commit `5dfb49e3`.

## Completed Milestone: Release Discipline Checklist

Mode: sequential

- [x] 1. Add a full pre-tag release gate script.
- [x] 2. Add a release checklist with version/tag policy and post-tag verification.
- [x] 3. Link release discipline from the first-read docs and changelog.

### Task Briefs

#### 1. Release gate command

- Target files: `scripts/test-release-gates.sh`.
- Requirements: provide one command that runs the release-level gates before a
  public source tag is created.
- Done: `bash scripts/test-release-gates.sh` runs shell syntax checks,
  front/direct/error/parity/value/native/install gates,
  self-host regeneration gates, release archive packaging, website build, and
  `git diff --check`.

#### 2. Release checklist

- Target files: `docs/release/RELEASE_CHECKLIST.md`, `README.md`,
  `docs/README.md`, `CHANGELOG.md`, `ROADMAP.md`.
- Requirements: document the next planned release line, tag policy, pre-tag
  checks, manual archive workflow trigger, and post-tag verification.
- Done: the current source release is `v0.3.1`, the next planned source
  release is `v0.3.2`, and tag creation is explicitly deferred until release
  gates are green.

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
- Requirements: direct mode accepts local `List<DeclaredStruct>` values through
  the native direct engine; `List<Struct>` function parameter/return ABI is
  handled by the following milestone.
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
- [x] 2. Remove the public direct-engine non-native fallback.
- [x] 3. Expand the native direct engine through Int helper calls, locals, assignment, `if`, `while`, simple struct locals, and struct parameter/return helpers.
- [x] 4. Sync README, language docs, website copy, changelog, and gates.

### Task Briefs

#### 1. Release archive workflow

- Target files: `.github/workflows/release-archives.yml`, `scripts/package-vaisc-release.sh`.
- Requirements: tag builds package standalone compiler/checker archives and
  upload them to the matching GitHub Release.
- Done: workflow builds Linux/macOS archive jobs, smokes packaged `vaisc`, creates the release when needed, and uploads archives.

#### 2. Native direct path

- Target files: `scripts/vaisc`, `tools/vaisc_native.c`.
- Requirements: `--engine direct` must stay on the native driver.
- Done: `scripts/test-vaisc-direct.sh` proves direct mode stays native even
  when an unrelated `python3` shim is first in `PATH`.

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
- Requirements: direct mode accepts local `List<Int>` values through the native
  direct engine; function parameter/return list ABI stays out of this slice.
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
  `List<Int>` signatures through the native direct engine.
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

- [x] 1. Add install and uninstall scripts for standalone `vaisc` and
  `vais-check`.
- [x] 2. Add release archive packaging for the native binaries and first-read
  docs.
- [x] 3. Add an install/package gate that proves installed and packaged
  binaries run.
- [x] 4. Sync docs/site/changelog and run release gates.

### Task Briefs

#### 1. Standalone install and uninstall

- Target files: `scripts/install-vaisc.sh`, `scripts/uninstall-vaisc.sh`.
- Requirements: build the native compiler from the checked-in self-host core and
  install `PREFIX/bin/vaisc` plus the Vais-built checker as
  `PREFIX/bin/vais-check`; uninstall removes those binaries.
- Done: installing into a temporary prefix creates executable `vaisc` and
  `vais-check`, and uninstall removes them.

#### 2. Release archive packaging

- Target files: `scripts/package-vaisc-release.sh`, `.gitignore`.
- Requirements: build a standalone archive containing `bin/vaisc`,
  `bin/vais-check`, and the current first-read docs; keep generated archives out
  of git.
- Done: the package script creates `dist/vais-VERSION-OS-ARCH.tar.gz`.

#### 3. Install/package gate

- Target files: `scripts/test-vaisc-install.sh`, `AGENTS.md`, `README.md`.
- Requirements: verify installed and packaged compiler binaries can report
  version, run `doctor`, and compile/run a `.vais` smoke source, and verify the
  installed and packaged checker accepts/flags fixture sources.
- Done: `bash scripts/test-vaisc-install.sh` passes without writing outside a temporary directory.

#### 4. Documentation, site, and gates

- Target files: `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs describe checkout use, standalone install, uninstall, package, and the gate protecting them.
- Done: docs and site are synced, website builds, stale public-claim scan is clean, and release gates pass.

## Completed Milestone: Native Public `vaisc`

Mode: sequential

- [x] 1. Native driver skeleton and build script.
- [x] 2. Release source-preparation parity with the retired prototype path.
- [x] 3. `scripts/vaisc` default switch and install/doctor UX.
- [x] 4. Documentation/site/changelog sync and release gates.

### Task Briefs

#### 1. Native driver skeleton and build script

- Target files: `tools/`, `scripts/`, `README.md`, `ROADMAP.md`.
- Requirements: compile a native `vaisc` binary from a small host driver and `compiler/self/vaisc_core.ll`; support `emit-ir`, `build`, `run`, `--help`, `--version`, and `doctor` for the full engine path.
- Done: a local native binary can compile and run `examples/c4.vais`.

#### 2. Release source-preparation parity

- Target files: native driver/source-prep implementation and existing `scripts/test-vaisc*.sh` gates.
- Requirements: keep the native release source-preparation behavior for
  enum/match, payload enum, closure-return, typed `Int`, `print`, comments, and
  semicolon normalization.
- Done: `bash scripts/test-vaisc.sh`, `bash scripts/test-vaisc-front.sh`, `bash scripts/test-vaisc-errors.sh`, `bash scripts/test-vaisc-parity.sh`, and `bash scripts/test.sh` pass through the native public command.

#### 3. Public command switch and install UX

- Target files: `scripts/vaisc`, packaging/install scripts, README docs.
- Requirements: `scripts/vaisc` uses the native driver by default for normal user `emit-ir`, `build`, and `run`; `doctor` reports missing `clang` or missing native driver clearly.
- Done: a fresh checkout can build the native driver and run `scripts/vaisc doctor`, `scripts/vaisc run examples/c4.vais`, and `scripts/vaisc build examples/c4.vais -o /tmp/c4`.

#### 4. Documentation, release, and gates

- Target files: `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, `compiler/self/SELF_HOST.md`, `website/`, `CHANGELOG.md`, `WORKLOG.md`.
- Requirements: public docs describe the native command path only after verification.
- Done: verification baseline plus self-host gates pass, the website builds, stale public-claim scan is clean, and GitHub/site release notes are synced.

## Completed Milestone: Vais-Native Self-Host Gate Helpers

Mode: sequential

- [x] 1. Port self-source embedding to `tools/embed_self_source.vais`.
- [x] 2. Port stage IR normalization to `tools/normalize_stage_ir.vais`.
- [x] 3. Move long self-host gates onto the Vais helpers.
- [x] 4. Use Vais-only helper gates in the release baseline.

### Task Briefs

#### 1. Self-source embedding

- Target files: `tools/embed_self_source.vais`,
  `scripts/test-embed-self-source-vais.sh`, `scripts/test-fixpoint-full.sh`.
- Requirements: support normalized `.vais` source-file retargeting, raw
  compact-program embedding, and raw string-call retargeting in the fixpoint
  gates.
- Done: `scripts/test-fixpoint.sh`, `scripts/test-fixpoint2.sh`, and
  `scripts/test-fixpoint-full.sh` build the Vais helper once and use it for all
  harness inputs; `scripts/test-embed-self-source-vais.sh` exercises
  normalized source-file retargeting, raw compile embedding, and raw
  string-call retargeting through the Vais helper.

#### 2. Stage IR normalization

- Target files: `tools/normalize_stage_ir.vais`,
  `scripts/test-normalize-stage-ir-vais.sh`,
  `scripts/test-fixpoint-full-self.sh`.
- Requirements: normalize generated stage IR names through a Vais-built helper
  before comparing stage1/stage2 self-host output.
- Done: the long self-host comparison uses the Vais normalizer; its focused
  gate checks the expected normalized IR shape directly through the Vais
  helper.

#### 3. Gate integration

- Target files: `scripts/test-release-gates.sh`, `AGENTS.md`,
  `compiler/self/SELF_HOST.md`, `WORKLOG.md`.
- Requirements: release gates build and exercise the Vais-native helpers before
  self-host and archive checks.
- Done: focused helper gates, full-codegen, full self-host, archive packaging,
  and website build all run from `bash scripts/test-release-gates.sh`.

## Verification Baseline

Run before closing compiler changes:

```bash
bash -n scripts/*.sh
bash scripts/test-vais-check-vais.sh
bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc-install.sh
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test-vaisc-host.sh
bash scripts/test-embed-self-source-vais.sh
bash scripts/test-normalize-stage-ir-vais.sh
bash scripts/test-fixpoint.sh
bash scripts/test-fixpoint2.sh
bash scripts/test.sh
bash scripts/test-fixpoint-full.sh
bash scripts/test-fixpoint-full-self.sh
```
