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

Compiler parity coverage is tracked in `tools/vaisc-parity.tsv`:

```bash
bash scripts/test-vaisc-parity.sh
```

Representative promoted examples include `examples/e83_parse_helpers.vais` for
the named `parse_uint(s)` and `parse_int(s)` prelude helpers,
`examples/e74_map_basic.vais` for the verified local `Map<Int,Int>` slice, and
`examples/e84_list_methods.vais` for `List<T>.is_empty()`, `last()`, and
`pop()`.

Files not listed as `native-supported` are retained as examples or future
coverage candidates, but they are not public release claims until promoted into
the parity manifest.
