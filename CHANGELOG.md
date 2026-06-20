# Vais Changelog

## Unreleased

### Changed

- Promoted explicit `Bool` locals, helper parameters, helper returns, and unary
  `not` through full self-host, native direct, parity, and value gates, with
  `examples/e88_bool_type.vais` added to the release corpus.
- Promoted explicit `Str` locals, helper parameters, helper returns,
  reassignment, length, index, and equality through full self-host, native
  direct, parity, and value gates, with `examples/e89_str_type.vais` added to
  the release corpus.
- Promoted simple enum expression-arm `match` lowering for multi-field `Int`
  payload variants, with `examples/e02_enum_payload.vais` added to the release
  corpus.

## v0.3.1 - 2026-06-20

Current Vais source release.

### Fixed

- Fixed self-host `print`/`puts` lowering for string-expression arguments so
  generated LLVM IR calls `i32 @puts(i8*)` consistently across release archive
  builders.

## v0.3.0 - 2026-06-20

Previous Vais source release.

### Changed

- Added verified `List<T>.is_empty()` support in the full self-host compiler and
  native direct engine, with front, direct, error, full, and release-gate
  coverage.
- Added verified `List<T>.last()` support for non-empty lists in the full
  self-host compiler and native direct engine, including struct-list local
  binding coverage.
- Added verified `List<T>.pop()` support for non-empty lists in the full
  self-host compiler and native direct engine, including caller-visible length
  mutation for list parameters.
- Defined verified runtime trap behavior for invalid `List` access: negative or
  out-of-range index operations, `last()` on an empty list, and `pop()` on an
  empty list.
- Promoted the first `Str` tool-helper slice: public front contracts now accept
  `Bool` and `Str` helper signatures, native direct mode lowers string
  literals, `s.len()`, `s[i]`, and `Str` equality/inequality, and parity now
  covers string indexing, user-defined integer parsing, and identifier scanning.
- Promoted single-byte `Char` literal equality plus explicit `Char` locals,
  helper parameters, and helper returns through public front, native direct,
  full self-host, and parity gates as Int-compatible scalar values, with
  `examples/e85_char_type.vais` added to the release corpus.
- Promoted range `for` loops with exclusive `..` and inclusive `..=` bounds
  through public front, native direct, full self-host, and parity gates, with
  `examples/e86_for_loop.vais` added to the release corpus.
- Promoted `break` and `continue` inside `while` and range `for` loops through
  public front, native direct, full self-host, and parity gates, with
  `examples/e87_break_continue.vais` added to the release corpus.
- Promoted named integer parsing prelude helpers: `parse_uint(s)` and
  `parse_int(s)` now lower through the full self-host compiler and native direct
  engine, with front, direct, parity, value, and self-host gate coverage.
- Added verified local `Map<Int,Int>` support across the full self-host compiler
  and native direct engine with `{}`, `insert`, `get(key, default)`, `contains`,
  and `len`; front diagnostics still reject Map parameters, return values,
  assignment, and generic key/value forms.
- Added release-corpus examples for local `Map<Int,Int>` and `List<T>`
  `is_empty()`, `last()`, and `pop()` so promoted prelude APIs have value-test
  coverage.
- Specified the Phase 2 module/package/import model and added public front
  diagnostics for reserved `module` and `package` declarations.
- Added the first full-engine local import implementation for single-package
  multi-file builds, including missing-import, duplicate-symbol, and
  import-cycle diagnostics.
- Added source-root `vais.toml` package manifest support for the full engine,
  with required `name`, `version`, and `source` keys plus manifest diagnostics.
- Added local dependency package paths in `vais.toml` `[dependencies]` for the
  full engine, including native gates for dependency imports and manifest
  diagnostics.
- Specified the Phase 3 host file/path/process API plan for future
  checker/tool port work, with the APIs marked as specified rather than
  verified.
- Added the first verified Phase 3 host file intrinsics, `fs_exists(path: Str)
  -> Bool`, `fs_read_text(path: Str) -> Str`,
  `fs_write_text(path: Str, text: Str) -> Int`, and `fs_mkdirs(path: Str) ->
  Int`, for full-engine `scripts/vaisc build` and `scripts/vaisc run`, with a
  native-driver runtime and `scripts/test-vaisc-host.sh` coverage.
- Added verified Phase 3 path helpers, `fs_cwd() -> Str`,
  `fs_temp_dir() -> Str`, `path_join(base: Str, child: Str) -> Str`,
  `path_basename(path: Str) -> Str`, and `path_dirname(path: Str) -> Str`, with
  native-driver runtime support and host gate coverage.
- Added the first verified Phase 3 process intrinsic,
  `proc_run(argv: List<Str>) -> Int`, plus full-engine `List<Str>` local
  `push` support for argv construction, with native-driver runtime support and
  host gate coverage.
- Added verified `proc_capture_stdout(argv: List<Str>) -> Str` for full-engine
  builds and runs, giving Vais-authored repository tools access to child
  process stdout without shell-string APIs.
- Added verified `proc_capture_stderr(argv: List<Str>) -> Str` for full-engine
  builds and runs, giving Vais-authored diagnostics tools access to child
  process stderr without shell-string APIs.
- Added verified `proc_run_env(argv: List<Str>, env: List<Str>) -> Int` for
  child-process environment overrides, and moved the direct-engine no-Python
  PATH check into a Vais-authored harness.
- Added verified `proc_capture_to(argv: List<Str>, stdout_path: Str,
  stderr_path: Str) -> Int` for status-sensitive process checks that need
  captured output files without a struct-returning host ABI.
- Added verified `fs_remove(path: Str) -> Int` and moved standalone uninstall
  option parsing plus binary removal into `tools/uninstall_vaisc.vais`.
- Added `tools/vaisc_install_check.vais` and moved standalone install/package
  verification assertions out of `scripts/test-vaisc-install.sh`.
- Moved the direct-engine arithmetic/build/run smoke checks into
  `tools/vaisc_direct_smoke_check.vais`, further reducing the NV-C2 shell
  fixture.
- Moved the direct-engine import reject and List bounds trap checks into
  `tools/vaisc_direct_error_check.vais`, using `proc_capture_to` to keep status
  and stderr/trap output handling in Vais code.
- Moved the direct helper/control-flow, range `for`, struct-local, and struct
  ABI success fixtures into `tools/vaisc_direct_feature_check.vais`.
- Expanded `tools/vaisc_direct_feature_check.vais` with the direct local
  `List<Int>`, `Str`, `Char`, `parse_uint`/`parse_int`, local `Map<Int,Int>`,
  and local `List<Struct>` success fixtures, further shrinking the NV-C2 shell
  wrapper.
- Moved the remaining direct List ABI, list assignment, and returned-list
  argument hoist fixtures into `tools/vaisc_direct_feature_check.vais`, leaving
  `scripts/test-vaisc-direct.sh` as a thin bootstrap wrapper around
  Vais-authored direct validators.
- Added `tools/vaisc_direct_gate.vais`, so the NV-C2 direct-emitter gate
  orchestration now runs from Vais and the shell entrypoint only supplies the
  temp-dir bootstrap boundary.
- Reduced the remaining single-tool focused shell wrappers to call their
  Vais-authored gates through `scripts/vaisc run`, keeping shell only for
  temporary directories and bootstrap arguments.
- Added `tools/normalize_stage_ir_check.vais` and moved the stage IR
  normalizer sample/expected fixture plus replacement-shape checks out of
  `scripts/test-normalize-stage-ir-vais.sh`.
- Added `tools/embed_self_source_check.vais` and moved the self-source embed
  focused gate fixture generation, generated compiler build/run checks, and
  result assertions out of `scripts/test-embed-self-source-vais.sh`.
- Added `tools/vais_check_contract_check.vais` and moved checker output-count,
  diagnostic-pattern, path, help, and public-wrapper assertions out of
  `scripts/test-vais-check-vais.sh`.
- Added `tools/fixpoint_tier_check.vais` and moved the short `fixpoint.vais`
  and `fixpoint2.vais` tier fixture lists, raw-call embedding, generated
  compiler builds, clang IR checks, and result assertions out of their shell
  gates.
- Added `tools/fixpoint_full_self_check.vais` and moved the long full-source
  self-host gate orchestration out of `scripts/test-fixpoint-full-self.sh`,
  including compiler retargeting, generated compiler build/run checks, final
  binary assertions, and normalized stage comparison.
- Added `tools/fixpoint_full_codegen_check.vais` and moved the long
  full-codegen regression runner out of `scripts/test-fixpoint-full.sh`,
  including compact fixture embedding, trap/stdout cases, source-file checks,
  and emitted-IR shape assertions.
- Audited the remaining shell/host boundary after the full-codegen port:
  native C bootstrap, public command cache wrappers, release/CI orchestration,
  website build tooling, tar/install/clang system tools, and temp-dir wrappers
  remain explicit.
- Fixed native front-contract scanning so unsupported-syntax probes ignore text
  inside string, raw-string, character literals, and comments instead of
  reporting diagnostics for fixture-generator text.
- Added the first Vais-authored checker slice in `tools/vais_check_core.vais`,
  with fixture-based contract checks through `scripts/test-vais-check-vais.sh`
  and the release gate.
- Expanded the Vais checker slice to cover the main non-Vais spelling
  diagnostics in the public fixture catalog, with fixture issue counts checked
  in the checker contract gate.
- Added path, line, column, and help output to the Vais checker slice, with the
  checker contract gate checking diagnostic shape.
- Added `proc_argc()` and `proc_arg(index)` for arguments passed through
  `scripts/vaisc run -- ...`, plus an argv-backed
  `tools/vais_check_cli.vais` checker entrypoint gated by fixture contracts.
- Extended `proc_argc()` and `proc_arg(index)` to binaries produced by
  `scripts/vaisc build`, so standalone Vais tools receive normal OS argv.
- Promoted the Vais-authored checker to the public `scripts/vais-check`
  command, installed and packaged as standalone `bin/vais-check` alongside
  `bin/vaisc`.
- Moved the checker clean/false-positive catalog into fixture-backed Vais
  checker gates and removed the separate checker unit test from the release
  gate.
- Removed checker oracle use from the checker release gate; `scripts/vais-check`
  is now verified by Vais-authored fixture contract checks.
- Moved the NV-C4 parity manifest gate into `tools/vais_parity_check.vais`, so
  release-corpus manifest parsing and native result comparison run through a
  Vais-authored harness.
- Moved the value-corpus gate behind `scripts/test.sh` into
  `tools/vais_value_check.vais`, so release example build/run/exit-code checks
  are driven by a Vais-authored harness.
- Moved the host file/path/string/process smoke gate into
  `tools/vais_host_check.vais`, so IR-shape checks, build/run checks, argv
  checks, and file-output assertions are driven by a Vais-authored harness.
- Moved the NV-C0 public compiler smoke gate into
  `tools/vaisc_smoke_check.vais`, so `emit-ir`, direct `clang`, `build`, and
  `run` contract checks are driven by a Vais-authored harness.
- Moved the NV-C1 front contract gate into `tools/vaisc_front_check.vais`, so
  accepted/rejected source fixture generation, multi-file package setup, stdout
  checks, and diagnostic-shape checks are driven by a Vais-authored harness.
- Moved the native driver smoke gate into `tools/vaisc_native_check.vais`, so
  native-driver build, version, doctor, emit, build, and run checks are driven
  by a Vais-authored harness after the C build bootstrap.
- Moved the NV-C3 diagnostics gate into `tools/vaisc_errors_check.vais`, so
  compiler diagnostic fixture generation and stderr shape checks are driven by
  a Vais-authored harness.
- Moved the legacy self-host compiler smoke gate into
  `tools/compiler_smoke_check.vais`, so program retargeting, generated compiler
  execution, IR staging, and final binary checks are driven by a Vais-authored
  harness and run from the pre-tag release gate.
- Added verified host-backed string construction helpers `str_concat`,
  `str_slice`, and `str_byte`, with native-driver runtime support plus
  `scripts/test-vaisc-host.sh` coverage.
- Extended full-engine `Str` lowering for reassignment and user-defined
  `-> Str` helper returns, covered by the host smoke gate.
- Extended full self-host lowering for runtime `Str` equality/inequality and
  regenerated the reusable compiler core, allowing Vais-authored tools to use
  idiomatic string comparisons.
- Moved release archive packaging orchestration into
  `tools/package_vaisc_release.vais`; `scripts/package-vaisc-release.sh` now
  delegates option parsing, version/platform detection, binary staging, docs
  staging, and archive creation to a Vais-authored tool.
- Moved standalone install orchestration into `tools/install_vaisc.vais`;
  `scripts/install-vaisc.sh` now delegates option parsing, compiler/checker
  staging, and installation to a Vais-authored tool while preserving existing
  CLI and environment inputs.
- Moved internal self-host helper builds onto the native `scripts/vaisc`
  trust-root path, removed the compiler escape hatch, and verified the embed
  helper, stage normalizer, fixpoint, full codegen, and full self-host gates
  through the native path.

## v0.2.2 - 2026-06-15

Current Vais source release.

### Changed

- Added `scripts/test-release-gates.sh` and
  `docs/release/RELEASE_CHECKLIST.md` as the pre-tag release contract for
  future source releases.
- Added a GitHub Actions release archive workflow for tag builds.
- `scripts/vaisc --engine direct` now stays on the native driver.
- The native direct engine now covers Int helper calls, locals, assignment,
  `if`, `while`, return expressions, and simple Int-field struct local
  literal/read/write plus struct parameter/return helper ABI.
- The native direct engine now covers local `List<Int>` initialization with
  `[]`, `list()`, and small integer list literals, plus `push`, `len`, index,
  and `sum`.
- The native direct engine now accepts `List<Int>` function signatures and
  return values through the direct ABI.
- `List<Int>` direct-engine parameters are now native references for local list
  arguments, so callee `push` operations mutate the caller's list.
- Inline `List<Int>` literals and `list()` now lower in direct-engine call
  arguments and return expressions.
- `List<Int>`-returning helper calls now hoist into direct-engine temporaries
  when passed directly to `List<Int>` parameters in statement contexts.
- Direct-engine `while` conditions now hoist returned-list arguments per
  iteration instead of requiring a local list binding.
- Local `List<Struct>` values now lower through the direct engine for typed
  `[]`, `list()`, list literals, `push`, `len`, index, and field reads.
- `List<Struct>` direct-engine function parameters, return values, inline list
  arguments, and returned-list argument hoisting now use the native list ABI.
- `List<Int>` and `List<Struct>` direct-engine assignment now supports
  context-typed `[]`, `list()`, list literals, local lists, and returned lists.
- `List<Struct>` direct-engine indexed field assignment now supports local and
  parameter writes such as `xs[0].value = 42`.
- `List<Int>` and `List<Struct>` direct-engine element assignment now supports
  local and parameter writes such as `xs[0] = value`.
- `List<Int>` and `List<Struct>` returned-list arguments now lower inside
  direct-engine `if` and `else if` conditions.

### Requirements

- `clang`

### Verification

The release baseline is protected by:

```bash
bash scripts/test-release-gates.sh
```

## v0.2.1 - 2026-06-14

Previous Vais source release.

### Changed

- `scripts/vaisc` now defaults to a native public driver that links the checked-in
  self-host compiler core.
- Normal user `emit-ir`, `build`, `run`, and `doctor` use the native driver.
- Added standalone install, uninstall, package, and native install/package test
  scripts.

### Requirements

- `clang`

### Verification

The release baseline is protected by:

```bash
bash scripts/test-release-gates.sh
```

## v0.2.0 - 2026-06-14

Previous Vais source release.

### Included

- `.vais` is the checked-in source extension.
- `scripts/vaisc` is the public compiler command.
- `scripts/vaisc emit-ir`, `scripts/vaisc build`, and `scripts/vaisc run` compile
  `.vais` files through the self-host compiler core and link with `clang`.
- `compiler/self/fixpoint_full.vais` is the trusted full compiler source.
- `compiler/self/vaisc_core.ll` is the reusable self-host compiler core used by
  `scripts/vaisc`.
- `docs/reference/LANGUAGE.md` is the current gate-backed language guide.
- `website/` is the official `vaislang.dev` source and deploys through GitHub
  Pages Actions.

### Requirements

- `clang`

### Verification

The release baseline is protected by:

```bash
bash scripts/test-release-gates.sh
```
