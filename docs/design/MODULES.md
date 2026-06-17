# Vais Module Model

Status: first local-import and source-root manifest slices implemented for the
full `scripts/vaisc` engine.

Current verified builds can compile one entry `.vais` file plus local files
reached through static dotted `import` declarations. A nearest `vais.toml`
manifest can set the package source root. Explicit `module` and `package`
declarations are still reserved and rejected.

## Goals

- Let a small Vais project split source across files.
- Keep package loading deterministic and local.
- Avoid a registry or dependency solver in the first module slice.
- Preserve clear diagnostics for duplicate symbols and import cycles.

## Source Files And Module Names

- A module is one `.vais` file under a package source root.
- Without `vais.toml`, the first source root is the directory containing the
  entry file.
- With `vais.toml`, the first source root is the manifest directory plus its
  `source` value.
- Module names are derived from the package-relative path without `.vais`.
- Path separators become dots: `math/add.vais` is module `math.add`.
- A file named `main.vais` has module name `main`.
- Explicit `module name` declarations are reserved for a later gate and are
  rejected for now.

## Import Paths

The first implementation supports only local package imports:

```vais
import math.add
```

`import math.add` resolves to `math/add.vais` under the current package source
root. Import paths must be static dotted identifiers. Absolute paths, `..`,
environment expansion, URLs, wildcard imports, and generated imports are not in
the first slices.

## Visibility And Symbols

- Top-level `fn`, `struct`, and verified `enum` names from imported modules are
  package-visible.
- Imported symbols enter one package-level namespace for the first slice.
- There is no `pub` keyword yet; package boundaries provide the first visibility
  boundary.
- The entry package may define exactly one `fn main() -> Int`.
- Duplicate top-level names across the loaded module graph are compile errors.
  Diagnostics must include the repeated name and both source paths.

## Ordering

Compilation loads the entry file, recursively loads each file's imports in
module-name order, then emits imported modules before the importing file. The
same checkout must produce the same source merge order on every platform.

## Cycles

Import cycles are rejected in the first slice. Diagnostics must show the cycle
path, for example:

```text
main -> math.add -> main
```

## Package Manifest

The first package manifest is `vais.toml` with only:

```toml
name = "demo"
version = "0.1.0"
source = "src"
```

`name`, `version`, and `source` are required top-level string keys. `source`
must be a local relative path such as `src`; absolute paths and `..` are
rejected. The compiled entry file must be under the resolved source root.

No registry, semver solver, build scripts, features, binary targets, or external
dependencies are part of the first module/package implementation. Local package
dependencies are reserved for the next Phase 2 slice.

## Current Gates

- `scripts/vaisc build examples/module_basic/main.vais` builds a multi-file
  local package.
- `scripts/vaisc run examples/module_basic/main.vais` returns the expected value.
- `scripts/vaisc build examples/package_basic/src/main.vais` builds a package
  using `vais.toml`.
- `scripts/vaisc run examples/package_basic/src/main.vais` returns the expected
  value.
- Duplicate top-level symbols produce a P4 diagnostic with both file paths.
- Missing import paths produce a P4 diagnostic with the resolved path.
- Import cycles produce a P4 diagnostic with the cycle path.
- Invalid package manifests produce P4 diagnostics with source coordinates and
  help text.
- Existing single-file examples continue to pass `scripts/test-vaisc-parity.sh`
  and `scripts/test.sh`.
