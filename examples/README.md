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

Representative promoted examples include `examples/e02_enum_payload.vais` for
multi-field `Int` payload enum expression-arm match lowering,
`examples/e24_struct_enum_field.vais` for payload-free enum values stored in
struct fields and matched through field access,
`examples/e64_enum_struct_payload.vais` for a single-field struct payload enum
matched through payload field access,
`examples/e55_match_wildcard.vais` for Int match literal arms with a `_`
catch-all,
`examples/e90_enum_wildcard.vais` for payload-free enum match with a `_`
catch-all,
`examples/e16_option_match.vais` for the first `Option<Int>` helper-return and
statement-match slice,
`examples/e21_result_match.vais` for the first `Result<Int,Int>`
helper-return and statement-match slice,
`examples/e23_option_flow.vais` for `Option<Int>` expression-match binding,
`examples/e93_option_question.vais` for `Option<Int>` `?` propagation,
`examples/e39_error_propagate.vais` for `Result<Int,Int>` `?` propagation,
`examples/e91_result_flow.vais` for `Result<Int,Int>` expression-match binding,
`examples/e92_result_question_success.vais` for the `Result<Int,Int>` `?`
success path,
`examples/e40_option_in_struct.vais` for `Option<Int>` stored in a struct field,
`examples/e94_map_get_opt.vais` for `Map<Int,Int>.get_opt(key)` returning
`Option<Int>`, `examples/e95_map_assignment.vais` for local `Map<Int,Int>`
assignment copy semantics, `examples/e96_map_bool.vais` for local
`Map<Int,Bool>` insert/get/contains/len and assignment-copy semantics,
`examples/e97_map_char.vais` for local `Map<Int,Char>`
insert/get/contains/len and assignment-copy semantics,
`examples/e98_map_param.vais` for `Map<Int,Int>` parameter mutation by
reference, `examples/e99_map_bool_param.vais` for `Map<Int,Bool>` parameter
mutation by reference, `examples/e100_map_char_param.vais` for
`Map<Int,Char>` parameter mutation by reference,
`examples/e101_map_return.vais` for a `Map<Int,Int>` return value initializing
a local, `examples/e102_map_bool_return.vais` for a `Map<Int,Bool>` return
value initializing a local, `examples/e103_map_char_return.vais` for a
`Map<Int,Char>` return value initializing a local, `examples/e104_map_remove.vais`
for concrete Map key removal, `examples/e105_map_scalar_get_opt.vais` for
`Map<Int,Bool>` and `Map<Int,Char>` get_opt match payloads,
`examples/e106_map_clear.vais` for concrete Map clear and reuse,
`examples/e107_map_str_int.vais` for local-only `Map<Str,Int>` string-key
operations and assignment copy,
`examples/e83_parse_helpers.vais` for the named `parse_uint(s)` and
`parse_int(s)` prelude helpers,
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
