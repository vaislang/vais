# Vais Changelog

## Unreleased

No changes yet.

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
