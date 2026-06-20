# Vais Examples

This directory contains `.vais` examples. The release value-correctness corpus is
the subset listed as `native-supported` in `tools/vaisc-parity.tsv`.

Each release-corpus file starts with `# expect: N`. `scripts/test.sh` compiles and
runs those files and compares the process exit code with `N % 256`.

Run the release corpus:

```bash
bash scripts/test.sh
```

Run a single example by basename:

```bash
bash scripts/test.sh c4
```

Run the multi-file import example:

```bash
scripts/vaisc run examples/module_basic/main.vais
```

Run the package-manifest example:

```bash
scripts/vaisc run examples/package_basic/src/main.vais
```

Run the local dependency package example:

```bash
scripts/vaisc run examples/dependency_basic/app/src/main.vais
```

Compiler parity coverage is tracked in `tools/vaisc-parity.tsv`:

```bash
bash scripts/test-vaisc-parity.sh
```

Representative promoted examples include `examples/e83_parse_helpers.vais` for
the named `parse_uint(s)` and `parse_int(s)` prelude helpers,
`examples/e74_map_basic.vais` for the verified local `Map<Int,Int>` slice, and
`examples/e84_list_methods.vais` for `List<T>.is_empty()`, `last()`, and
`pop()`, and `examples/e85_char_type.vais` for the promoted Int-compatible
`Char` scalar slice, and `examples/e86_for_loop.vais` for exclusive and
inclusive range `for` loops, and `examples/e87_break_continue.vais` for loop
control flow, and `examples/e88_bool_type.vais` for explicit `Bool` locals,
helper parameters/returns, and unary `not`, and `examples/e89_str_type.vais`
for explicit `Str` locals, helper parameters/returns, reassignment, length,
index, and equality.

Files not listed as `native-supported` are retained as examples or future
coverage candidates, but they are not public release claims until promoted into
the parity manifest.
