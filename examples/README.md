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

These module, package, and dependency examples are part of the release
value-correctness corpus.

Compiler parity coverage is tracked in `tools/vaisc-parity.tsv`:

```bash
bash scripts/test-vaisc-parity.sh
```

Representative promoted examples include `examples/e02_enum_payload.vais` for
multi-field `Int` payload enum expression-arm match lowering,
`examples/e24_struct_enum_field.vais` for payload-free enum values stored in
struct fields and matched through field access,
`examples/e17_struct_return.vais`, `examples/e28_struct_rebuild.vais`,
`examples/e37_struct_area.vais`, `examples/e41_recursion_struct.vais`,
`examples/e54_inventory.vais`, and `examples/e62_struct_multi_return.vais` for
struct helper parameters, returns, reassignment, recursion, and aggregation,
`examples/d5run.vais` for public struct/function modifiers with a `Str` field,
`examples/e63_generic_struct_def.vais` for generic marker syntax on a simple
struct used with `Int` values,
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
`examples/e08_option_chain.vais` for direct `Option<Int>` helper-return
matching from `main`,
`examples/e39_error_propagate.vais` for `Result<Int,Int>` `?` propagation,
`examples/e91_result_flow.vais` for `Result<Int,Int>` expression-match binding,
`examples/e92_result_question_success.vais` for the `Result<Int,Int>` `?`
success path,
`examples/d3run.vais` for a `Result<Int,Int>` helper propagation chain,
`examples/e40_option_in_struct.vais` for `Option<Int>` stored in a struct field,
`examples/e19_interpolation_print.vais` for print interpolation and `putchar`
output calls,
`examples/e25_for_filter_sum.vais` for collection for-each over integer values,
`examples/e27_list_max.vais` for `List<Int>` parameter for-each with a running
max,
`examples/e82_list_literal_direct_arg.vais` for an inline `List<Int>` literal
passed directly to a `List<Int>` parameter,
`examples/d4b.vais` for an inline `List<Int>` literal iterated through a
`List<Int>` parameter,
`examples/e15_list_recursion.vais` and `examples/e68_binary_search.vais` for
borrowed `&List<Int>` helper parameters,
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
`examples/e107_map_str_int.vais` for `Map<Str,Int>` string-key operations and
assignment copy, `examples/e108_map_str_int_param.vais` for `Map<Str,Int>`
parameter mutation by reference, `examples/e109_map_str_int_return.vais` for a
`Map<Str,Int>` return value initializing a local,
`examples/e110_map_str_bool.vais` for `Map<Str,Bool>` string-key operations and
assignment copy, `examples/e111_map_str_bool_param.vais` for `Map<Str,Bool>`
parameter mutation by reference, `examples/e112_map_str_bool_return.vais` for a
`Map<Str,Bool>` return value initializing a local,
`examples/e113_map_str_char.vais` for local `Map<Str,Char>` string-key
operations and assignment copy, `examples/e114_map_str_char_param.vais` for
`Map<Str,Char>` parameter mutation by reference,
`examples/e115_map_str_char_return.vais` for a `Map<Str,Char>` return value
initializing a local, `examples/e116_map_param_assignment.vais` for concrete
Map parameter-source and parameter-target assignment copies,
`examples/e117_map_return_assignment.vais` for concrete Map-returning call
assignment copies, `examples/e118_map_return_assignment_args.vais` for
argument-bearing Map-returning call assignment copies,
`examples/e83_parse_helpers.vais` for the named `parse_uint(s)` and
`parse_int(s)` prelude helpers,
`examples/e69_palindrome_string.vais` for two-pointer `Str` scans with
computed byte indexes,
`examples/e71_string_index_of.vais` for `Str` substring search with computed
byte indexes,
`examples/e74_map_basic.vais` for the verified local `Map<Int,Int>` slice, and
`examples/e84_list_methods.vais` for `List<T>.is_empty()`, `last()`, and
`pop()`, and `examples/e85_char_type.vais` for the promoted Int-compatible
`Char` scalar slice, and `examples/e86_for_loop.vais` for exclusive and
inclusive range `for` loops, and `examples/e87_break_continue.vais` for loop
control flow, and `examples/e88_bool_type.vais` for explicit `Bool` locals,
helper parameters/returns, and unary `not`, and `examples/e89_str_type.vais`
for explicit `Str` locals, helper parameters/returns, reassignment, length,
index, and equality.
The release corpus also includes smaller control-flow and scanner examples:
`examples/e06_for_sum.vais`, `examples/e10_bool_logic.vais`,
`examples/e12_exclusive_range.vais`, `examples/e13_nested_for.vais`,
`examples/e36_bool_predicate.vais`, `examples/e44_string_len.vais`,
`examples/e52_state_machine.vais`, `examples/e53_word_count.vais`,
`examples/e57_break.vais`, `examples/e58_continue.vais`,
`examples/e61_array_index_expr.vais`, and
`examples/e65_loop_break_acc.vais`, `examples/fr1.vais`,
`examples/fr2.vais`, `examples/t2.vais`, `examples/t3.vais`,
`examples/t4.vais`, `examples/t5.vais`, and `examples/t6.vais`.

Files not listed as `native-supported` are retained as examples or future
coverage candidates, but they are not public release claims until promoted into
the parity manifest.
