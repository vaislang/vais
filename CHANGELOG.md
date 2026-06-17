# Vais Changelog

## Unreleased

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

## v0.2.2 - 2026-06-15

Current Vais source release.

### Changed

- Added `scripts/test-release-gates.sh` and
  `docs/release/RELEASE_CHECKLIST.md` as the pre-tag release contract for
  future source releases.
- Added a GitHub Actions release archive workflow for tag builds.
- `scripts/vaisc --engine direct` now stays on the native driver instead of the
  internal Python fallback.
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
- Development-only fallback paths remain internal.
- Added standalone install, uninstall, package, and native install/package test
  scripts.

### Requirements

- `clang`

### Verification

The release baseline is protected by:

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

- Python 3
- `clang`

### Verification

The release baseline is protected by:

```bash
python3 -m py_compile tools/vaisc.py tools/vais-check.py tools/embed_self_source.py tests/vais_check_test.py
bash -n scripts/*.sh
python3 tests/vais_check_test.py
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh
bash scripts/test-fixpoint-full-self.sh
bash scripts/test-fixpoint-full.sh
```
