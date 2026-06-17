# Vais Host I/O Model

Status: specified for Phase 3; not implemented as a verified public API yet.

This document fixes the first file, path, and process APIs Vais should grow so
repository checks can move from Python into Vais without mixing host I/O into
the pure self-host compiler core.

## Boundary

- Host I/O is effectful standard-library surface, not core language syntax.
- The public native driver may provide host-backed intrinsics.
- `compiler/self/fixpoint_full.vais` and `compiler/self/vaisc_core.ll` should
  stay focused on source-to-IR compilation. They may call declared host
  intrinsics when compiling user programs, but compiler-core correctness must
  not depend on reading the repository filesystem.
- The first gates must use temporary directories and explicit paths.
- No registry, network access, environment mutation, shell expansion, globbing,
  or implicit current-directory search is part of the first host I/O slice.

## File API

The first file API is text-only and UTF-8 oriented:

| API | Status | Behavior |
| --- | --- | --- |
| `fs_exists(path: Str) -> Bool` | Specified | Return whether a file or directory exists. |
| `fs_read_text(path: Str) -> Str` | Specified | Read a whole text file. Missing or unreadable files trap with a host diagnostic until `Result` exists. |
| `fs_write_text(path: Str, text: Str) -> Int` | Specified | Write text, replacing the file. Return `0` on success and a non-zero host status on failure. |
| `fs_mkdirs(path: Str) -> Int` | Specified | Create a directory and missing parents. Return `0` on success. |

The first implementation should reject NUL bytes in paths. It should not attempt
cross-platform path normalization beyond passing the explicit path to the host.

## Path API

Paths are `Str` values. The first path helpers are deliberately small:

| API | Status | Behavior |
| --- | --- | --- |
| `fs_cwd() -> Str` | Specified | Return the current working directory used by `vaisc run` or the built binary. |
| `fs_temp_dir() -> Str` | Specified | Return the host temporary directory. |
| `path_join(base: Str, child: Str) -> Str` | Specified | Join two path segments using the host separator. If `child` is absolute, return `child`. |
| `path_basename(path: Str) -> Str` | Specified | Return the final path segment. |
| `path_dirname(path: Str) -> Str` | Specified | Return the parent path, or `.` when there is no parent segment. |

The first slice does not include canonicalization, symlink resolution, path
permissions, recursive directory walking, or path objects.

## Process API

Process execution must be argv-based rather than shell-string based:

| API | Status | Behavior |
| --- | --- | --- |
| `proc_run(argv: List<Str>) -> Int` | Specified | Run `argv[0]` with the remaining arguments, inherit stdio, and return the process exit code. |
| `proc_capture(argv: List<Str>) -> ProcessResult` | Specified | Run the process, capture stdout and stderr, and return a result struct. |

`ProcessResult` is specified as:

```vais
struct ProcessResult {
    code: Int,
    stdout: Str,
    stderr: Str,
}
```

The first implementation may gate `proc_run` before `proc_capture` because
capturing output requires verified `List<Str>` and struct fields containing
`Str`. Shell execution, environment overrides, stdin piping, timeouts, and
working-directory overrides are later slices.

## stdout, stderr, And Exit Codes

- `print(EXPR)` and `putchar(Int)` remain the verified stdout primitives.
- A future `eprint(EXPR)` can mirror `print` for stderr, but it is not part of
  the first implementation slice.
- `fn main() -> Int` remains the process exit status source.
- `proc_run` and `proc_capture` report child exit codes exactly when the host
  can provide them. Launch failures return a non-zero status and, for
  `proc_capture`, put the host message in `stderr`.

## First Internal Port Target

The first Python-replacement target should be a small checker, not the compiler
driver:

1. Implement the minimum host intrinsics in the native public driver.
2. Add `scripts/test-vaisc-host.sh` with temp-directory file and process smoke
   tests.
3. Add a Vais source checker that reads one `.vais` file and reports a narrow
   diagnostic subset already covered by `tools/vais-check.py`.
4. Keep the Python checker as an oracle until the Vais checker matches the
   selected cases.

The first checker port is successful only when both the Vais checker and the
Python checker are run by release gates for the same cases.
