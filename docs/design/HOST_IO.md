# Vais Host I/O Model

Status: `fs_exists(path: Str) -> Bool`,
`fs_read_text(path: Str) -> Str`,
`fs_write_text(path: Str, text: Str) -> Int`,
`fs_mkdirs(path: Str) -> Int`, and `fs_remove(path: Str) -> Int`, plus the first `Str` path helpers,
string-construction helpers, argv helpers, `proc_run(argv: List<Str>) -> Int`,
`proc_run_env(argv: List<Str>, env: List<Str>) -> Int`,
`proc_capture_stdout(argv: List<Str>) -> Str`, and
`proc_capture_stderr(argv: List<Str>) -> Str`, plus
`proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int`,
are verified for full-engine `scripts/vaisc build` and `scripts/vaisc run`;
full in-memory `ProcessResult` capture remains specified for a later Phase 3
gate.

This document fixes the first file, path, and process APIs Vais should grow so
repository checks can run as Vais-authored tools without mixing host I/O into
the pure self-host compiler core.

## Boundary

- Host I/O is effectful standard-library surface, not core language syntax.
- The public native driver may provide host-backed intrinsics.
- `compiler/self/fixpoint_full.vais` and `compiler/self/vaisc_core.ll` should
  stay focused on source-to-IR compilation. They may call declared host
  intrinsics when compiling user programs, but compiler-core correctness must
  not depend on reading the repository filesystem.
- The first gates must use temporary directories and explicit paths.
- No registry, network access, shell expansion, globbing, or implicit
  current-directory search is part of the first host I/O slice. Environment
  overrides are limited to child-process `KEY=value` entries passed through
  `proc_run_env`.

## File API

The first file API is text-only and UTF-8 oriented:

| API | Status | Behavior |
| --- | --- | --- |
| `fs_exists(path: Str) -> Bool` | Verified | Return whether a file or directory exists. |
| `fs_read_text(path: Str) -> Str` | Verified | Read a whole text file. Missing or unreadable files trap with a host diagnostic until `Result` exists. |
| `fs_write_text(path: Str, text: Str) -> Int` | Verified | Write text, replacing the file. Return `0` on success and a non-zero host status on failure. |
| `fs_mkdirs(path: Str) -> Int` | Verified | Create a directory and missing parents. Return `0` on success. |
| `fs_remove(path: Str) -> Int` | Verified | Remove a file path. Missing paths return `0`; recursive directory removal is not part of this slice. |

The verified file/path slice passes explicit Vais `Str` paths to the host
without canonicalization, symlink resolution, recursive directory walking, or
path objects.

## Path API

Paths are `Str` values. The first path helpers are deliberately small:

| API | Status | Behavior |
| --- | --- | --- |
| `fs_cwd() -> Str` | Verified | Return the current working directory used by `vaisc run` or the built binary. |
| `fs_temp_dir() -> Str` | Verified | Return the host temporary directory. |
| `path_join(base: Str, child: Str) -> Str` | Verified | Join two path segments using the host separator. If `child` is absolute, return `child`. |
| `path_basename(path: Str) -> Str` | Verified | Return the final path segment. |
| `path_dirname(path: Str) -> Str` | Verified | Return the parent path, or `.` when there is no parent segment. |

The first slice uses POSIX-style path behavior in the native host runtime. It
does not include path permissions, recursive directory walking, or path objects.

## String Helper API

These helpers are host-backed allocation primitives used by Vais-authored tools
that transform file text:

| API | Status | Behavior |
| --- | --- | --- |
| `str_concat(left: Str, right: Str) -> Str` | Verified | Return a newly allocated string containing `left` followed by `right`. |
| `str_slice(text: Str, start: Int, len: Int) -> Str` | Verified | Return a byte slice; negative or out-of-range ranges trap. |
| `str_byte(value: Int) -> Str` | Verified | Return a one-byte string; values outside `0..255` trap. |

These helpers are intentionally byte-oriented until Vais has a fuller Unicode
string model.

## Process API

Process execution must be argv-based rather than shell-string based:

| API | Status | Behavior |
| --- | --- | --- |
| `proc_argc() -> Int` | Verified | Return the number of arguments passed after `scripts/vaisc run file.vais --`. |
| `proc_arg(index: Int) -> Str` | Verified | Return one argument passed after `scripts/vaisc run file.vais --`; invalid indexes trap. |
| `proc_run(argv: List<Str>) -> Int` | Verified | Run `argv[0]` with the remaining arguments, inherit stdio, and return the process exit code. |
| `proc_run_env(argv: List<Str>, env: List<Str>) -> Int` | Verified | Run `argv` like `proc_run`, applying `KEY=value` entries from `env` in the child process before exec. |
| `proc_capture_stdout(argv: List<Str>) -> Str` | Verified | Run `argv[0]` with the remaining arguments and return captured stdout as `Str`; stderr is inherited. |
| `proc_capture_stderr(argv: List<Str>) -> Str` | Verified | Run `argv[0]` with the remaining arguments and return captured stderr as `Str`; stdout is inherited. |
| `proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int` | Verified | Run `argv`, redirect stdout/stderr to explicit files when paths are non-empty, and return the process exit code. |
| `proc_capture(argv: List<Str>) -> ProcessResult` | Specified | Run the process, capture stdout and stderr, and return a result struct. |

`ProcessResult` is specified as:

```vais
struct ProcessResult {
    code: Int,
    stdout: Str,
    stderr: Str,
}
```

`proc_argc`, `proc_arg`, `proc_run`, `proc_run_env`, `proc_capture_stdout`,
`proc_capture_stderr`, and `proc_capture_to` are the first verified process
slices. Argv access is guaranteed through
`scripts/vaisc run file.vais -- ...` and through directly built binaries
produced by `scripts/vaisc build`. `proc_capture_to` covers status-sensitive
repository tools without requiring a struct-returning host ABI. Full
`proc_capture` is deferred because it requires a result struct with status,
stdout, and stderr fields.
Shell execution, stdin piping, timeouts, and working-directory overrides are
later slices.

## stdout, stderr, And Exit Codes

- A future `eprint(EXPR)` can mirror `print` for stderr, but it is not part of
  the first implementation slice.
- `fn main() -> Int` remains the process exit status source.
- `proc_run` reports child exit codes exactly when the host can provide them.
  `proc_capture_stdout` and `proc_capture_stderr` return one captured stream.
  `proc_capture_to` returns the child exit code and writes both output streams
  to caller-provided files for status-sensitive workflows.

## First Internal Port Target

The first port target is a small checker, not the compiler driver:

1. Implement the minimum host intrinsics in the native public driver.
2. Add `scripts/test-vaisc-host.sh` with temp-directory host API smoke tests.
3. Extend the host gate from file existence to text files, directories, paths,
   and argv-based process execution for both `vaisc run` and built binaries.
4. Add a Vais source checker that reads one `.vais` file and reports the main
   spelling diagnostics covered by the checker fixture contract.
5. Promote the Vais checker command once the selected cases are covered by
   fixture count, coordinate, help, and clean-file gates.

`tools/vais_check_core.vais` is the completed checker rule slice. It reads
fixture files with `fs_read_text`, reports the main non-Vais spelling
diagnostics with path, line, column, and help output. It is gated by
`tools/vais_check_contract_check.vais`, with `scripts/test-vais-check-vais.sh`
kept as the bootstrap wrapper.
`tools/vais_check_cli.vais` is built into the public `scripts/vais-check`
command and into release archives as `bin/vais-check`.

`tools/package_vaisc_release.vais` is the next completed internal port. The
shell entrypoint still provides the repository-root bootstrap boundary, but
option parsing, version/platform detection, binary staging, documentation
staging, and archive creation now run as Vais code using the verified file,
path, string-builder, and process APIs.

`tools/install_vaisc.vais` is also a completed internal port. The install
entrypoint delegates option parsing, temporary compiler/checker staging, and
file installation to Vais. `tools/uninstall_vaisc.vais` now delegates option
parsing and binary removal to Vais using the verified `fs_remove` API.
`tools/vaisc_install_check.vais` drives the standalone install/package gate,
including installed binary smoke checks, package archive extraction, packaged
binary smoke checks, checker fixture checks, and uninstall assertions.

`tools/vais_parity_check.vais` is the first Vais-authored release-corpus parity
harness. `scripts/test-vaisc-parity.sh` now builds that tool and delegates
manifest parsing, expected-exit extraction, compiler invocation, and native
result comparison to Vais code.

`tools/vais_value_check.vais` is the Vais-authored value-correctness harness
behind `scripts/test.sh`. It reads the same parity manifest for the release
subset, extracts each `# expect:` value, builds each program through
`scripts/vaisc`, runs the generated binary, and compares the exit code in Vais
code.

`tools/vais_host_check.vais` is the Vais-authored host smoke validator behind
`scripts/test-vaisc-host.sh`. It writes the temporary host API probe program,
checks emitted IR for file/path/string/process intrinsic declarations and
calls, builds and runs the probe through `scripts/vaisc`, verifies OS argv
delivery and child-process environment overrides, and checks file outputs in
Vais code.

`tools/vaisc_smoke_check.vais` is the Vais-authored NV-C0 public compiler smoke
validator behind `scripts/test-vaisc.sh`. It writes the canonical tiny program,
checks emitted IR shape, invokes `clang` through argv-based `proc_run`, and
verifies `scripts/vaisc emit-ir`, `build`, and `run` from Vais code.

`tools/vaisc_front_check.vais` is the Vais-authored NV-C1 front contract
validator behind `scripts/test-vaisc-front.sh`. It writes accepted and rejected
front-contract fixture programs, creates temporary package/import trees, uses
captured stdout/stderr for output and diagnostic checks, and keeps the shell
entrypoint as a bootstrap boundary.

`tools/vaisc_native_check.vais` is the Vais-authored native driver smoke
validator behind `scripts/test-vaisc-native.sh`. It invokes the native C build
script as the bootstrap boundary, then verifies `--version`, `doctor`,
`emit-ir`, `build`, and `run` behavior from Vais code.

`tools/vaisc_errors_check.vais` is the Vais-authored NV-C3 diagnostics
validator behind `scripts/test-vaisc-errors.sh`. It writes the diagnostic
fixture sources, captures compiler stderr with `proc_capture_stderr`, and
checks coordinate, caret, `help:`, and `fix:` output from Vais code.

`tools/vaisc_direct_env_check.vais` is the first Vais-authored slice inside the
NV-C2 direct-emitter gate. It uses `proc_run_env` to prepend a fake `python3`
to child `PATH` and verifies that the native direct engine still runs without
invoking Python.

`tools/vaisc_direct_smoke_check.vais` is the next NV-C2 slice. It writes the
canonical direct arithmetic program, checks emitted IR shape, invokes `clang`
through argv-based `proc_run`, verifies direct `build` and `run`, and compares
the full engine result from Vais code.

`tools/vaisc_direct_error_check.vais` moves the direct import reject check and
List bounds trap checks into the NV-C2 Vais-authored gate. It uses
`proc_capture_to` so the harness can assert process status while inspecting
stderr or keeping trap output out of the shell wrapper.

`tools/vaisc_direct_feature_check.vais` continues the NV-C2 port by moving the
direct helper/control-flow, range `for`, struct-local, struct ABI, local
`List<Int>`, `Str`, `Char`, `parse_uint`/`parse_int`, local `Map<Int,Int>`,
local `Map<Int,Bool>`, local `Map<Int,Char>`, `Map<Int,Int>`,
`Map<Int,Bool>`, and `Map<Int,Char>` parameters, local
`List<Struct>`, list ABI, list assignment, and returned-list argument hoisting
success fixtures into Vais code with the same emitted-IR shape checks
and exit-code assertions. `tools/vaisc_direct_gate.vais` orchestrates those
NV-C2 sub-checks from Vais, so `scripts/test-vaisc-direct.sh` is now only the
temp-directory bootstrap wrapper around the Vais-authored direct gate.

The remaining single-tool focused gate wrappers follow the same boundary:
shell creates the temporary directory and passes bootstrap arguments, while
`scripts/vaisc run <tool>.vais -- ...` executes the Vais-authored checker
logic. The long full-source self-host gate now follows that boundary too:
`tools/fixpoint_full_self_check.vais` drives self-source retargeting,
trust-root generated compiler builds, emitted-IR validation, final binary
assertions, and normalized stage comparison. The long full-codegen regression
runner follows it as well: `tools/fixpoint_full_codegen_check.vais` drives the
compact fixture catalog, trap/stdout cases, source-file checks, and emitted-IR
shape assertions. Native C compiler bootstrap, tar/package system tools, and
platform CI glue remain explicit host boundaries.

After this port, the audited host boundary is:

- `tools/vaisc_native.c`: native public driver, direct engine, host runtime,
  and linker/process integration.
- `scripts/build-vaisc-native.sh`: native C driver bootstrap and
  `vaisc_core.ll` entrypoint rewrite before linking.
- `scripts/vaisc` and `scripts/vais-check`: public command cache/build-lock
  wrappers.
- `scripts/test-release-gates.sh` and `.github/workflows/*`: release and CI
  orchestration.
- `website/` build commands: npm/Vite site build tooling.
- Thin shell wrappers under `scripts/test-*.sh`: temporary directory setup and
  invocation of the corresponding Vais-authored gate.
- System tools invoked from Vais or bootstrap scripts: `clang`, `install`,
  `tar`, `grep`, `awk`, `wc`, and `cmp` where they are host environment tools
  rather than language/runtime semantics.
