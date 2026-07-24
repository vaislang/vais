# Vais Language Reference

This page describes the current gate-backed Vais source surface. Together with
`std/PRELUDE.md`, it is the v1-candidate reference freeze for verified language
claims. Vais files use the `.vais` extension and are compiled with
`scripts/vaisc`.

## Status Model

Vais documentation uses these terms:

| Term | Meaning |
| --- | --- |
| Verified | Covered by `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-parity.sh`, `scripts/test-vaisc-host.sh`, `scripts/test.sh`, or a self-host gate |
| Full engine | Compiled by the native public driver linked with the reusable self-host compiler core |
| Direct engine | Native promoted-slice LLVM path selected with `--engine direct` |
| Specified | Intended surface, not yet protected as a release claim |

Public examples should use verified syntax unless they explicitly document a
planned or experimental area.

## Program Shape

```vais
fn main() -> Int {
    return 42
}
```

- Entry point: `fn main() -> Int`.
- Source files must end in `.vais`.
- Statements do not require semicolons in ordinary source files.
- Line comments start with `#` and continue to the end of the line outside
  string literals.

## Modules, Packages, And Imports

The Phase 2 module model is specified in
[../design/MODULES.md](../design/MODULES.md). The implemented slice supports
local dotted imports in both the full/default and direct engines:

```vais
import math.add

fn main() -> Int {
    return add(20, 22)
}
```

Without a package manifest, `import math.add` resolves to `math/add.vais` under
the entry file's directory. With a nearest `vais.toml`, imports resolve under
the manifest source root:

```toml
name = "demo"
version = "0.1.0"
source = "src"
binary = "demo-cli"
assets = "assets"

[dependencies]
mathlib = "../mathlib"
```

For `source = "src"`, explicit file compilation can target `src/main.vais`,
and package-directory compilation can target the directory containing
`vais.toml`:

```bash
scripts/vaisc run examples/e323_cli_package
scripts/vaisc build examples/e323_cli_package -o /tmp/e323
scripts/vaisc emit-ir examples/e323_cli_package --engine direct -o /tmp/e323.ll
scripts/vaisc package examples/e323_cli_package -o /tmp/e323-dist
scripts/vaisc package examples/e326_cli_binary_target -o /tmp/e326-dist
scripts/vaisc package examples/e326_cli_binary_target -o /tmp/e326-dist --archive
scripts/vaisc package examples/e328_cli_package_assets -o /tmp/e328-dist --archive
```

The package directory form resolves to `source/main.vais`, so
`import math.add` resolves to `src/math/add.vais`. The source root and
`main.vais` entry must exist before compiling. The package command writes an
installable binary to `dist/bin/<package-name>` and copies `vais.toml` to
`dist/vais.toml`; packaged binaries preserve ordinary `proc_argc`/`proc_arg`
argv behavior. Optional `binary = "cmd-name"` overrides only the output command
name, so the binary is written to `dist/bin/<cmd-name>` while package entry
resolution still uses `source/main.vais`. Manifest package names or `binary`
values used as output binary filenames must be non-empty, must not start with
`.` or `-`, and may contain only letters, digits, `_`, `-`, and `.`. A
package manifest may also set optional `assets = "assets"` metadata. That path
must be one local relative directory under the package root; absolute paths,
`..`, globs, multiple asset roots, and non-directory asset roots are rejected
or unsupported. `scripts/vaisc package` copies it to `dist/assets`, and
`--archive` includes it as `<binary-or-name>-<version>/assets/`. A package
command with `--archive` also writes
`dist/<binary-or-name>-<version>.tar.gz`, containing
`<binary-or-name>-<version>/bin/<binary-or-name>`, the copied `vais.toml`, and
optional package assets; manifest versions used in archive filenames follow
the same safe filename-component rules.
`[dependencies]` entry maps an import prefix to another local package directory
with its own `vais.toml`; for example,
`import mathlib.public` resolves to `public.vais` under the `mathlib` package's
source root when no local `mathlib/public.vais` file exists. Files loaded from
a dependency resolve plain imports under their own package source root.

Imported files are merged in deterministic import-path order before
compilation. The direct native engine uses the same merged source before direct
lowering, so local/package/dependency imports are no longer limited to the full
engine. Missing imports, duplicate top-level symbols, import cycles, missing
dependency manifests, unsafe dependency paths, and invalid manifests are
front-contract errors. `scripts/vaisc` runs Vais-authored manifest and import
graph preflight tools before native `emit-ir`, `build`, and `run`.

Representative gate-backed examples are `examples/module_basic/main.vais`,
`examples/package_basic/src/main.vais`,
`examples/dependency_basic/app/src/main.vais`, and
`examples/e322_vaisdb_module_boundary/main.vais`. `examples/e323_cli_package`
verifies package-directory entry resolution plus CLI argv forwarding, and the
workflow gate verifies installable package output through `scripts/vaisc
package`. `examples/e326_cli_binary_target` verifies optional package
`binary` metadata for a distinct installed command name.

Explicit `module` and `package` declarations are reserved for later Phase 2
gates and are rejected for now.

## Functions

```vais
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn main() -> Int {
    return add(20, 22)
}
```

Verified today:

- `Int` parameters and `Int` return values.
- Multiple helper functions.
- Recursive and mutually recursive `Int` functions.
- Generic marker syntax for simple `Int` helper cases and generic identity
  helpers applied to struct literals, as tracked in the parity manifest.

The direct engine gate covers Int, Bool, Char, and Str helper calls in addition to the
full engine.

## Variables

```vais
let x = 5
let mut total = 0
total = total + x
let typed: Int = 42
```

- `let` binds an immutable value.
- `let mut` binds a mutable value.
- `let name: Int = value`, `let name: Bool = value`,
  `let name: Char = value`, and `let name: Str = value` are verified for
  scalar locals.
- Compound assignment such as `+=` is not Vais syntax.

## Types

Verified release surface:

| Type | Notes |
| --- | --- |
| `Int` | Primary scalar type |
| `Bool` | Produced by comparisons and boolean expressions; verified for local annotations, helper parameters, and helper returns |
| `Str` | String literals, local annotations, struct fields, reassignment, helper parameters/returns, length, index, equality, `str_contains` substring search, `str_index_of` first-index search, `str_starts_with` prefix checks, `str_ends_with` suffix checks, `str_replace` string rewriting, `str_trim` edge cleanup, `str_lower`/`str_upper` ASCII case normalization, `str_split_ws_into` whitespace tokenization, `str_split_into` delimiter tokenization, `str_split_lines_into` LF/CRLF line tokenization, `str_join` list reconstruction, `doc_term_counts_into` term-frequency indexing, `doc_term_overlap_score` query/document overlap scoring, and `doc_term_weighted_score` repeated-term scoring |
| `Char` | Single-byte character literals, equality, annotations, helper parameters, and helper returns as Int-compatible scalar values |
| `List<Int>` | Empty/list literal, typed non-empty local literal, inline call argument, borrowed helper parameter, list/element assignment including `List<Struct>` field projection reassignment, `push`, `clear`, `contains`, `index_of`, `count`, `remove_at`, `insert_at`, `extend` from named lists, inline list literals, list-returning helper calls, and `List<Struct>` field projections, `len`, `is_empty`, `first`, `last`, `pop`, index, local/parameter `sum`, `max`, and `min`, filtered `sum`/`len`/`max`/`min` lowering, same-item map and filter-map transformed `sum`/`max`/`min` lowering in direct scalar, broader `Int`, and broader condition expressions, and local `List<List<Int>>` literal double-index reads, and in-place ascending `sort()` statements on local and parameter lists |
| `List<Str>` | Full/direct typed local literal, helper parameter/return, assignment slices including `List<Struct>` field projection reassignment, `push`, `clear`, `contains`, `index_of`, `count`, `remove_at`, `insert_at`, `extend` from named lists, inline list literals, list-returning helper calls, `List<Struct>` field projections, direct `List<Str>.filter(...).map(...)` / `List<Str>.map(...).filter(...)` result sources, and direct `List<Str>.map(...).filter(...).len/contains/index_of/count` / `List<Str>.filter(...).map(...).len/contains/index_of/count` scalar contexts including same-family or mixed-family multiple scalar calls in one expression, arithmetic-tail reassignments, negated Bool expressions, Bool if-expression locals/reassignments/helper-call arguments/Bool returns, Int if-expression locals/reassignments/helper-call arguments/returns, nested helper-call arguments inside reassignments, and composite Bool local inference for pipeline scalar conditions, index, `first`, `last`, `pop`, `len`, `is_empty`, local/parameter for-each, direct `.len()` chains and string equality on string element results, element assignment (`words[i] = value`), in-place ascending `sort()` statements on local and parameter lists, and argv-based host process arguments |
| `List<Struct>` | Full/direct `[]`, `list()`, typed local literals including multiline trailing-comma literals, multiline inline literal call arguments including standalone call statements, local/parameter assignment from inline struct list literals, list/element assignment including multiline struct literals, `push` from struct values including multiline trailing-comma and single-field nested struct literals, list element values, list method return values, filtered first/last whole-record selections, and struct-returning helper calls, `insert_at` including multiline struct literals, list element values, list method return values, filtered first/last whole-record selections, and struct-returning helper calls, `clear`, `remove_at`, `extend` from named lists, inline struct list literals including multiline struct literals, and list-returning helper calls, `filter(|x| predicate)` result lists for known receivers, `filter(|x| predicate).len()` record counts in returns and typed/inferred `Int` locals, `filter(|x| predicate).first().field`/`.last().field` record field projection in `Int`/`Str` returns, typed or inferred locals, direct helper-call arguments including helpers declared later, simple arithmetic suffixes on `Int` helper calls, and helper calls starting `if`, `else if`, or `while` conditions, and direct scalar `push`/`insert_at` arguments including `Str` field `.len()` reads into inferred `Int` locals, `Int` helper-call arguments, and `List<Int>` mutations, `filter(|x| predicate).first()`/`.last()` whole-record selection in same-struct returns, typed/inferred same-struct locals, direct same-struct helper-call arguments including helpers declared later, simple arithmetic suffixes on `Int` helper calls, and helper calls starting `if`, `else if`, or `while` conditions, and direct same-struct `push`/`insert_at` arguments, `filter(|x| predicate).map(|x| x.field)` projected scalar result lists including reusable locals, existing `List<Int>`/`List<Str>` variable reassignments, direct helper returns, direct `List<Int>`/`List<Str>` helper-call arguments, helper calls starting `if`, `else if`, or `while` conditions, and direct `List<Int>`/`List<Str>.extend(...)` arguments, `filter(|x| predicate).map(|x| x.int_field).sum()` same-item score aggregation including direct `Int` helper-call arguments, broader `Int` expressions, and broader `if`/`while`/`else if` condition expressions, `filter(|x| predicate).map(|x| x.int_field).max()`/`.min()` same-item score ranking including direct `Int` helper-call arguments, broader `Int` expressions, and broader `if`/`while`/`else if` condition expressions, `map(|x| x.field)` field projection into `List<Int>` or typed `List<Str>` locals plus direct `List<Int>`/`List<Str>` helper returns, helper-call arguments including helper calls starting `if`, `else if`, or `while` conditions, `extend(...)` sources, and existing-list reassignments, direct `map(|x| x.int_field).sum()`/`.max()`/`.min()` aggregation in `Int` returns, typed/inferred locals, helper-call arguments including simple arithmetic suffixes, standalone simple arithmetic suffixes, direct `List<Int>` mutation arguments, known `Int` reassignments, broader `Int` expressions, and broader `if`/`while`/`else if` condition expressions, `len`, `is_empty`, `first`, `last`, `pop`, index, field read/write including `Str` field reads with equality/helpers/`.len()` and indexed single-field nested field-chain reads/writes and multi-field nested read/write/copy/push/parameter slices, method-result field chains including `Str` fields, single-field nested chains, and verified multi-field nested chains, full/direct indexed field assignment, local/parameter for-each, parameter reference, return value, and in-place `sort_by(|x| x.int_field)`/`sort_by_desc(|x| x.int_field)` key-sort statements on local and parameter lists |
| `Map<Int,Int>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, `key_at(index)`, and `value_at(index)` |
| `Map<Int,Bool>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, `key_at(index)`, and `value_at(index)` |
| `Map<Int,Char>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, `key_at(index)`, and `value_at(index)` |
| `Map<Str,Int>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, `key_at(index)`, and `value_at(index)` |
| `Map<Str,Bool>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, `key_at(index)`, and `value_at(index)` |
| `Map<Str,Char>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, `key_at(index)`, and `value_at(index)` |
| `Map<Str,Str>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization including inferred locals, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)` match binding and string match expression contexts including return-inferred locals, Result-returning helper flows over Map parameters, `contains`, `len`, `key_at(index)`, and `value_at(index)` |
| `Option<Int>` | `Some(Int)`/`None`, helper returns, struct/local storage, statement-form `match`, single-line and multiline expression-match binding, local-binding `?` propagation, and nested match through an enum payload |
| `Result<Int,Int>` | `Ok(Int)`/`Err(Int)`, helper returns, statement-form `match`, expression-match binding, local-binding `?` propagation, helper flows over `Map<Str,Str>` parameters with `get_opt` matches, and `fs_exists` guarded file-read error flows |
| `Result<Str,Int>` | `Ok(Str)`/`Err(Int)`, helper returns for file-read flows, helper parameters/forwarding, direct call-argument use of Result-returning helpers, local-binding `?` propagation that binds the `Str` payload, and inline match recovery to `Int` or `Str` values |
| `Result<Str,Str>` | First non-Int error payload slice: `Ok(Str)`/`Err(Str)` so failures carry human-readable messages, helper returns, local-binding `?` propagation, and inline match recovery of either the `Str` value or the `Str` error message to a `Str` or its `Int` length. Verified in the native direct engine and the full self-host compiler; used by the VaisDB ingest workflow in `examples/e330_vaisdb_ingest_error_message_flow.vais` |
| `Result<Metric,Int>` | First structured payload slice: `Ok(Metric)`/`Err(Int)`, helper returns, helper parameters/forwarding, and inline match recovery of `Metric` fields to `Int` |
| `Result<DeclaredStruct,Int>` | Verified for declared struct payloads such as Int-field `Record`, multiline `Entry`, Str-field `DocSummary`, and VaisDB `DocArtifact`: helper returns, helper parameters/forwarding, direct call-argument use of Result-returning helpers, explicit wrapper payload local copies, local-binding `?`, `List<Struct>` output storage, persisted store reload parsing, inline match recovery of multiple struct fields or string field lengths to `Int`, direct `Str` field recovery to string locals, nested `str_concat(...)` composition of `Str` fields, `str_replace`/`str_trim`/`str_upper`/`str_lower` normalization in match arms, transformed string `.len()` scoring in `Int` arms, `Ok` payload handoff to reusable `Int` scoring helpers, helper-call terms composed with normal payload fields, and Bool returns from payload helper terms plus `Err(Int)` comparisons |
| `(Int, Int)` tuple | Function return and local destructuring slice lowered through generated structs |
| Simple `struct` | Literal construction including multiline local initialization, same-type local assignment, call arguments, `Str` fields, and direct helper returns for nested literals, field access including `Str` field equality/helper/`.len()` use and struct-returning helper field-chain reads, local field write, single-field nested struct read/write including direct flattening for previously declared single-Int-field nested structs, scalar multi-field nested struct local literals/direct returns/field-chain reads, helper parameters, helper returns, helper-return assignment, generic marker syntax used with `Int` values, generic identity helpers applied to struct literals, simple `impl` method return chains, and simple `trait` plus `impl Trait for Struct` method calls |
| Small `enum` | Payload-free enum/match, payload enum wildcard match, small recursive `Int` payload enum/match, single-field struct payload enum/match, and single `Option<Int>` payload enum with a nested Option match arm |

Specified or partial areas are tracked in [../../std/PRELUDE.md](../../std/PRELUDE.md)
and `tools/vaisc-parity.tsv`.

### Rejected `Option`/`Result` shapes

Only the concrete `Option`/`Result` forms in the table above are verified. The
checker and both engines reject everything else with a `help:` message that
lists the verified shapes, so an unverified type fails at check time rather than
miscompiling:

- `Option<T>` for any `T` other than `Int`.
- `Result<T,E>` outside `Result<Int,Int>`, `Result<Str,Int>`, `Result<Str,Str>`,
  and `Result<DeclaredStruct,Int>` where the payload struct is declared in the
  same file. In particular a non-`Int`, non-`Str` error payload
  (e.g. `Result<Int,Str>`) and an undeclared struct payload
  (e.g. `Result<Unknown,Int>`) are rejected.
- Nested `Option`/`Result` payloads such as `Result<Result<Int,Int>,Int>`,
  `Result<Option<Int>,Int>`, `Option<Result<Int,Int>>`, and
  `Option<Option<Int>>`.

These reject cases are pinned by the `tests/fixtures/vais_check/bad.vais`
diagnostic-count gate and the `result_generic_not_verified`,
`option_generic_not_verified`, `result_nested_not_verified`, and
`option_nested_not_verified` front-contract cases.

## Expressions And Operators

```vais
return (a + b) * 2
return n % 10
return a == b
return a < b and b < c
```

Verified operators:

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Boolean words: `and`, `or`, `not`
- Bool expression locals and Bool-returning helper predicates are covered by
  `examples/e10_bool_logic.vais` and `examples/e36_bool_predicate.vais`.
- Bit helpers: `bitnot(x)`, `bitand(a, b)`, `bitor(a, b)`, `bitxor(a, b)`,
  `shl(x, n)`, `shr(x, n)`
- Conversion calls: `Int(x)` and `Str(x)` for Int-compatible scalar values in
  verified examples
- Parsing helpers: `parse_uint(s)` and `parse_int(s)` for `Str`

Vais does not use Rust spellings such as `&&`, `||`, `!x`, `x as Int`, or
`Enum::Variant`.

## Control Flow

```vais
if n <= 1 {
    return 1
} else if n == 2 {
    return 2
} else {
    return n * fact(n - 1)
}

let mut i = 0
let mut sum = 0
while i < n {
    sum = sum + i
    i = i + 1
}

for j in 0..n {
    sum = sum + j
}

for k in 0..=n {
    sum = sum + k
}

for k in 0..10 {
    if k == 3 { continue }
    if k == 6 { break }
    sum = sum + k
}
```

The direct engine gate covers `if`, `while`, range `for`, named `List<Int>`
for-each loops, `break`/`continue` inside loops, local `let`, assignment, helper
calls, `return`, simple inline `if { return ... }`, `Bool`/`Char`/`Str` scalar helper
signatures, `Str` literals/length/index/equality, `Char` literal equality and
annotations,
simple Int-field struct
locals, struct parameter/return helpers, and `List<Int>` local operations plus
parameter reference and return value ABI, local `Map<Int,Int>`,
`Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>` construction and lookup/update helpers,
`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>` return-value local initialization,
`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>` parameter reference/mutation, plus `List<Struct>` construction with
`[]`, `list()`, list literals, list/element assignment, `push`, `insert_at`
from same-type list element values, `len`, index, field read/write, indexed
field assignment, parameter reference, and return value ABI.

Verified today:

- `if`, `else if`, `else`
- `while`
- `for name in start..end` and `for name in start..=end`
- `for name in xs` over gate-backed integer collections
- `break` and `continue` inside `while` and range `for` loops
- Early `return`
- Scalar `if ... then ... else ...` value expressions in local assignments,
  reassignments, helper-call arguments, and returns for `Int`, `Bool`, `Str`,
  and `Char` values, as covered by
  `examples/e276_scalar_value_if_expr_embedded_call_args.vais`,
  `examples/e277_scalar_bool_value_if_expr.vais`,
  `examples/e278_scalar_str_value_if_expr.vais`, and
  `examples/e279_scalar_char_value_if_expr.vais`

Representative gate-backed examples include `examples/e06_for_sum.vais`,
`examples/e12_exclusive_range.vais`, `examples/e13_nested_for.vais`,
`examples/e25_for_filter_sum.vais`, `examples/e57_break.vais`,
`examples/e58_continue.vais`, and `examples/e65_loop_break_acc.vais`.

## Structs

```vais
struct Box {
    value: Int,
}

fn main() -> Int {
    let b = Box { value: 42 }
    return b.value
}
```

Struct helper parameters and return values are also gate-backed:

```vais
fn make_box(value: Int) -> Box {
    return Box { value: value }
}

fn read_box(b: Box) -> Int {
    return b.value
}
```

Struct-returning helper results can be read directly with field chains:

```vais
fn main() -> Int {
    return make_box(40).value + 2
}
```

Generic marker syntax on simple structs is gate-backed when used with `Int`
values:

```vais
struct Box<T> { val: T }

fn main() -> Int {
    let b = Box { val: 7 }
    return b.val
}
```

Generic identity helpers are also gate-backed when they pass through a struct
literal directly:

```vais
struct Pair { a: Int, b: Int }
fn first_field(p: Pair) -> Int { return p.a }
fn apply<T>(x: T) -> T { return x }

fn main() -> Int {
    let p = apply(Pair { a: 7, b: 2 })
    return first_field(p)
}
```

Verified today:

- Simple struct declarations with `Int` fields.
- Struct literals.
- Field access.
- Field write for direct-engine local struct values.
- Single-field nested struct literals, reads, and writes in the full self-host
  path and native direct engine when the nested struct has one `Int` field and
  is declared before the outer struct, as covered by
  `examples/e01_nested_struct.vais`,
  `examples/e32_nested_field_mut.vais`, and
  `examples/e190_direct_nested_struct_multiline.vais`.
- Indexed `List<Struct>` element field-chain reads/writes for the same
  single-field nested struct shape, as covered by
  `examples/e191_list_nested_struct_field_chain.vais` and
  `examples/e192_list_nested_struct_field_chain_write.vais`.
- `List<Struct>` method-result field-chain reads for the same single-field
  nested struct shape, as covered by
  `examples/e193_list_nested_struct_method_field_chain.vais`.
- Struct-returning helper field-chain reads for top-level fields and the same
  single-field nested struct shape, as covered by
  `examples/e194_struct_return_field_chain.vais`.
- Direct returns of single-field nested struct literals, as covered by
  `examples/e195_nested_struct_literal_return.vais`.
- Scalar multi-field nested struct literals and field-chain reads for local
  values and direct struct-returning helpers, as covered by
  `examples/e196_multi_field_nested_struct.vais`.
- `List<Struct>` elements whose nested struct contains multiple `Int` fields
  can be pushed from struct locals, copied as whole elements, mutated through
  indexed nested field chains on local and parameter lists, and read through
  method-result nested field chains, as covered by
  `examples/e197_list_multi_field_nested_struct.vais`.
- Structs can contain `Str` fields used in equality, string helper calls, and
  `.len()` chains, including through `List<Struct>` index and method-result
  reads from `first`, `last`, `pop`, and `remove_at`. Indexed local and
  parameter `List<Struct>` records can also reassign `Str` fields such as
  `docs[i].title = title`, as covered by
`examples/e198_struct_str_fields.vais`,
`examples/e199_list_struct_str_fields.vais`, and
`examples/e200_list_struct_str_field_write.vais` plus
`examples/e201_list_struct_str_method_fields.vais`; the same `Str` field
storage is also used by the standard process-capture result in
`examples/e202_proc_capture_result.vais`.
- Twenty-field flat struct literals and field reads on the full self-host path,
  as covered by `examples/e51_index_ast.vais`.
- Struct parameters and return values in full self-host and direct-engine helper
  functions, including assignment from struct-returning calls.
- `pub` modifiers on struct declarations, struct fields, and helper functions
  are accepted as source-level visibility markers. The current compiler treats
  them as metadata rather than a separate ABI/export mechanism, as covered by
  `examples/d5run.vais`.
- Generic marker syntax on simple structs used with `Int` values and generic
  identity helpers applied directly to struct literals, as covered by
  `examples/e63_generic_struct_def.vais` and
  `examples/e46_generic_struct.vais`.
- Simple `impl Struct { fn method(self, ...) ... }` methods used in a return
  expression chain, as covered by `examples/e09_struct_method.vais`.
- Simple `trait` declarations paired with `impl Trait for Struct` methods, when
  the method call is used in a return expression, as covered by
  `examples/e78_trait_impl_for.vais`.
- Selected struct/list combinations through self-host gates.

Representative struct helper examples include `examples/e17_struct_return.vais`,
`examples/e28_struct_rebuild.vais`, `examples/e37_struct_area.vais`,
`examples/e41_recursion_struct.vais`, `examples/e54_inventory.vais`, and
`examples/e62_struct_multi_return.vais`. Promoted struct-method examples are
`examples/e09_struct_method.vais` and `examples/e78_trait_impl_for.vais`.

## Enums And Match

Payload-free enum:

```vais
enum Color { Red, Green, Blue }

fn number(c: Color) -> Int {
    match c {
        Color.Red => return 1,
        Color.Green => return 2,
        Color.Blue => return 3,
    }
}
```

Small payload enum:

```vais
enum Node { Lit(Int), Add(Node, Node), Mul(Node, Node) }

fn eval(n: Node) -> Int {
    match n {
        Lit(v) => return v,
        Add(a, b) => return eval(a) + eval(b),
        Mul(a, b) => return eval(a) * eval(b),
    }
}
```

Verified today:

- Payload-free enum tags with `Enum.Tag` spelling.
- Simple return-arm and expression-arm `match`.
- Int `match` with literal patterns and `_` catch-all, as covered by
  `examples/e55_match_wildcard.vais`.
- Payload-free enum `match` with `_` catch-all, as covered by
  `examples/e90_enum_wildcard.vais`.
- Payload enum `match` with `_` catch-all, as covered by
  `examples/e120_enum_payload_wildcard.vais`.
- Small recursive `Int` payload enum/match lowering used by the parity corpus.
- Multi-field `Int` payload enum expression-arm lowering, as covered by
  `examples/e02_enum_payload.vais`.
- Payload-free enum values stored in simple struct fields and matched through
  field access, as covered by `examples/e24_struct_enum_field.vais`.
- Single-field struct payload enum values constructed from a struct literal and
  matched through payload field access, as covered by
  `examples/e64_enum_struct_payload.vais`.
- A single `Option<Int>` enum payload matched through a nested Option match arm,
  as covered by `examples/e79_nested_match.vais`.

Broader payload shapes, including multi-field struct payloads and arbitrary
nested enum payloads, are not a release claim yet.

## Lists

```vais
fn main() -> Int {
    let xs: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    let tail = xs.pop()
    return xs.len() + xs[1] + xs.is_empty() + xs.last() + tail - 30
}
```

Function parameters can also receive a local `List<Int>`:

```vais
fn fill(out: List<Int>, n: Int) -> Int {
    out.push(n)
    out.push(n + 2)
    return out.len()
}

fn main() -> Int {
    let xs: List<Int> = []
    let count = fill(xs, 20)
    return count * 10 + xs[1]
}
```

Inline list values are valid return values and call arguments in the direct
engine:

```vais
fn make(n: Int) -> List<Int> {
    return [n, n + 2]
}

fn score(xs: List<Int>) -> Int {
    return xs.sum()
}

fn main() -> Int {
    return score([20, 22])
}
```

List-returning helper calls can be passed directly to `List<Int>` parameters in
ordinary statements and `if`/`else if` conditions:

```vais
fn make(n: Int) -> List<Int> {
    return [n, n + 1]
}

fn score(xs: List<Int>) -> Int {
    return xs.sum()
}

fn main() -> Int {
    if score([1, 1]) == 99 {
        return 1
    } else if score(make(20)) == 41 {
        return 42
    }
    return 0
}
```

Declared struct values can also be stored in local direct-engine lists:

```vais
struct Box {
    value: Int,
}

fn make(v: Int) -> List<Box> {
    return [Box { value: v }, Box { value: v + 1 }]
}

fn score(xs: List<Box>) -> Int {
    let first = xs[0]
    let tail = xs.pop()
    return first.value + xs.len() + tail.value
}

fn main() -> Int {
    let xs: List<Box> = []
    xs.push(Box { value: 20 })
    let ys: List<Box> = make(21)
    xs = [Box { value: 19 }]
    xs[0] = Box { value: 20 }
    xs[0].value = xs[0].value + 1
    return xs[0].value + ys[1].value + score([Box { value: 0 }, Box { value: 0 }]) - 1
}
```

Verified today:

- Empty `List<Int>` with an explicit type, typed non-empty local `List<Int>`
  literals such as `let xs: List<Int> = [10, 20, 30]`, and inline `List<Int>`
  literals passed to `List<Int>` parameters, as covered by
  `examples/e82_list_literal_direct_arg.vais` and `examples/d4b.vais`.
- Borrowed `&List<Int>` helper parameters in the full self-host path, as
  covered by `examples/e15_list_recursion.vais` and
  `examples/e68_binary_search.vais`.
- Integer list literals such as `[10, 20, 30]`.
- Computed index reads over integer list literals, as covered by
  `examples/e61_array_index_expr.vais`, and simple array-backed state machines,
  as covered by `examples/e52_state_machine.vais`.
- `for x in xs { ... }` over integer collections, as covered by
  `examples/e25_for_filter_sum.vais`. The full self-host path covers fixed
  integer arrays, typed non-empty local `List<Int>` literals, inline
  `List<Int>` literal call arguments, and scalar `List<Int>` locals/parameters;
  the native direct engine covers named local or parameter `List<Int>` values.
- `for word in words { ... }` over local or parameter `List<Str>` values in the
  full self-host path and native direct engine, as covered by
  `examples/e144_doc_score_for_each.vais`.
- `for box in boxes { ... }` over local or parameter `List<Struct>` values in the
  full self-host path and native direct engine, copying each element into a
  declared-struct loop variable, as covered by
  `examples/e166_list_struct_for_each.vais`.
- Inline `[]`, `list()`, and integer list literals as `List<Int>` return values
  in the direct engine, and inline integer list literals as `List<Int>` call
  arguments in both the full self-host path and direct engine.
- Local `List<List<Int>>` literals with constant row selection and `Int` column
  indexing, as covered by `examples/e77_nested_list.vais`.
- `List<Int>`-returning helper calls used directly as `List<Int>` call arguments
  in statement contexts plus `if`, `else if`, and `while` conditions.
- `xs.push(value)`, including same-type `List<Struct>` element values.
- `xs.clear()`.
- `xs.contains(value)`.
- `xs.len()`.
- `xs.is_empty()`.
- `xs.first()`.
- `xs.last()`.
- `xs.pop()`.
- `xs.remove_at(index)`.
- `xs.insert_at(index, value)`.
- `xs.extend(other)`.
- `xs[index]`.
- `xs.sum()`.
- `xs.max()`.
- `xs.min()`.
- `List<Str>` element/method results can feed directly into `.len()` and string
  equality, as covered by `examples/e121_list_str_methods.vais` and
  `examples/e123_list_str_return.vais`.
- Assigning `[]`, `list()`, list literals, local list values, and returned-list
  values to a matching direct-engine list local or list parameter.
- Assigning values to matching direct-engine list elements with `xs[index] = value`.
- Runtime trap behavior for out-of-range `xs[index]`, empty `xs.last()`,
  empty `xs.pop()`, and full-engine `xs.push(value)` past the fixed backing
  capacity.
- `xs.clear()` resets the list length to zero and reuses the same backing
  storage for later `push` calls.
- `xs.contains(value)` returns whether a local or parameter `List<Int>` contains
  the integer value, as covered by `examples/e153_list_contains.vais`.
- `xs.index_of(value)` returns the first matching index in a local or parameter
  `List<Int>`, or `-1` when missing, and `xs.count(value)` returns the number
  of matching integers, as covered by
  `examples/e157_list_int_index_count.vais`.
- `xs.remove_at(index)` removes and returns the indexed integer from a local or
  parameter `List<Int>`, shifts following elements left, and traps when `index`
  is out of range, as covered by `examples/e158_list_remove_at.vais`.
- `xs.insert_at(index, value)` inserts an integer into a local or parameter
  `List<Int>`, shifts elements at and after `index` right, permits appending at
  `index == len`, and traps when `index` is out of range or the backing buffer
  is full, as covered by `examples/e159_list_insert_at.vais`.
- `xs.extend(other)` appends all elements from a same-type named local or
  parameter `List<Int>`, inline `List<Int>` literal, or same-type
  `List<Int>`-returning helper call to a local or parameter `List<Int>`,
  supports self-extension by snapshotting the source length, and traps before
  mutation when the combined length exceeds the backing buffer, as covered by
  `examples/e160_list_extend.vais`,
  `examples/e178_list_scalar_str_extend_return_call.vais`, and
  `examples/e179_list_extend_inline_literal_source.vais`.
- `xs.first()` returns the first integer from a local or parameter `List<Int>`
  and traps on an empty list, as covered by `examples/e161_list_first.vais`.
- `words.contains(value)` returns whether a local or parameter `List<Str>`
  contains the string value, as covered by `examples/e154_list_str_contains.vais`.
- `words.index_of(value)` returns the first matching index in a local or
  parameter `List<Str>`, or `-1` when missing, as covered by
  `examples/e155_list_str_index_of.vais`.
- `words.count(value)` returns the number of matching strings in a local or
  parameter `List<Str>`, or `0` when missing, as covered by
  `examples/e156_list_str_count.vais`.
- `words.remove_at(index)` removes and returns the indexed string from a local
  or parameter `List<Str>`, shifts following elements left, and supports direct
  `.len()` on the returned string, as covered by
  `examples/e158_list_remove_at.vais`.
- `words.insert_at(index, value)` inserts a string into a local or parameter
  `List<Str>`, shifts elements at and after `index` right, permits appending at
  `index == len`, and traps when `index` is out of range or the backing buffer
  is full, as covered by `examples/e159_list_insert_at.vais`.
- `words.extend(other)` appends all elements from a same-type named local or
  parameter `List<Str>`, inline `List<Str>` literal, or same-type
  `List<Str>`-returning helper call to a local or parameter `List<Str>`,
  supports self-extension by snapshotting the source length, and traps before
  mutation when the combined length exceeds the backing buffer, as covered by
  `examples/e160_list_extend.vais`,
  `examples/e178_list_scalar_str_extend_return_call.vais`, and
  `examples/e179_list_extend_inline_literal_source.vais`.
- `words.first()` returns the first string from a local or parameter
  `List<Str>`, supports direct `.len()` on the returned string, and traps on an
  empty list, as covered by `examples/e161_list_first.vais`.
- Passing a local `List<Int>` to a `List<Int>` parameter.
- Returning `List<Int>` from helper functions.
- `List<Str>` typed local literals, inline literal call arguments, local and
  parameter-target assignment copy, literal assignment, return-call assignment,
  locals, helper parameters, and helper returns with `push`, index reads,
  `first()`, `last()`, `pop()`, `remove_at(index)`, `insert_at(index, value)`,
  `extend(other)`, `len()`, `is_empty()`, `contains(value)`, `index_of(value)`,
  and `count(value)` in the full self-host path and native direct engine.
  String-returning `words[i]`,
  `words.first()`, `words.last()`, and
  `words.pop()` and `words.remove_at(index)` results can feed directly into
  `.len()`, including returned list locals such as `let words = make_words()`.
- `List<Struct>` values with an explicit type, `[]`, `list()`, typed local
  literals, multiline inline literal call arguments including standalone call
  statements, local/parameter assignment from inline struct list literals,
  list/element assignment, `push` from struct values, multiline trailing-comma
  struct literals, and struct-returning helper calls, `insert_at` including
  struct-returning helper calls, `clear`,
  `remove_at(index)`,
  `insert_at(index, value)`, `extend(other)`, `len`, `is_empty`, `first`,
  `last`, `pop`, index, local/parameter `for` iteration, indexed field
  assignment, field reads/writes, parameter references, return values, inline
  call arguments, and returned-list call arguments in the direct engine.

Invalid list access is a runtime trap. This covers negative indexes,
out-of-range indexes, `first()` on an empty list, `last()` on an empty list, and
`pop()`/`max()`/`min()` on an empty list, plus filtered `max()`/`min()` when no
element matches.
`pop()` checks before mutating the list length. In the full self-host path,
`push()` also traps before writing when a fixed backing list buffer is full.
`insert_at(index, value)` traps before mutating when `index < 0`, `index > len`,
or the fixed backing list buffer is full.
`extend(other)` traps before mutating when the target length plus source length
would exceed the fixed backing list buffer.

The direct engine gate covers `List<Int>` values created with `[]`, `list()`, or
small integer list literals, plus `push`, `len`/`len()`, `is_empty()`,
`clear()`, `contains(value)`, `index_of(value)`, `count(value)`,
`remove_at(index)`, `insert_at(index, value)`, `extend(other)`, `first()`,
`last()`, `pop()`, index, `sum()`, `max()`, `min()`, `List<Int>.map(|x| expr)`,
`List<Int>.filter(|x| predicate)` result lists with known `Int` parameter or
local captures,
`List<Str>.map(|s| str_lower(str_trim(s)))`-style result lists,
`str_concat` map bodies, identity maps when the receiver type is known, and
`List<Str>.filter(|s| predicate)` result lists when the receiver type is known
from a local or `List<Str>` function parameter. Declared `List<Struct>` values
also support `filter(|item| predicate)` result lists when the receiver type is
known from a local or function parameter, including field predicates and
returning the filtered list from helpers, plus `map(|item| item.field)` field
projection into reusable `List<Int>` or typed `List<Str>` results. `List<Str>`
map/filter closure bodies may also capture known `Str` parameters or locals.
`List<Str>.filter(|s| predicate).map(|s| str_expr)` direct result lists are
covered for typed or inferred locals, helper returns, helper-call arguments
including `if`, `while`, and `else if` conditions, direct `extend(...)` sources,
and existing-list reassignments.
`List<Str>.map(|s| str_expr).filter(|s| predicate)` direct result lists are
covered for typed or inferred locals, helper returns, helper-call arguments
including `if`, `while`, and `else if` conditions, direct `extend(...)` sources,
and existing-list reassignments.
`List<Str>.map(|s| str_expr).filter(|s| predicate).len()`,
`.contains(value)`, `.index_of(value)`, and `.count(value)` direct scalar
contexts are covered for typed or inferred locals, helper returns, helper-call
arguments, direct `List<Int>` mutation arguments, reassignments, and `if`,
`while`, and `else if` conditions. They lower directly over the source list
without requiring a user-written mapped-list temporary.
`List<Str>.filter(|s| predicate).map(|s| str_expr).len()`,
`.contains(value)`, `.index_of(value)`, and `.count(value)` direct scalar
contexts are covered for the same local, return, helper-call, `List<Int>`
mutation, reassignment, and condition contexts without requiring a user-written
filtered-list temporary.
Within a single expression, multiple scalar calls from the same pipeline family
are also covered, such as two `map(...).filter(...).len/count` calls or two
`filter(...).map(...).contains/index_of` calls in the same arithmetic or
condition expression. Mixed map-filter and filter-map scalar calls are also
covered in the same expression. Composite local assignments that combine
pipeline scalar conditions with `and`, `or`, or comparisons infer `Bool`, so
later exact pipeline scalar Bool reassignments stay in the verified direct
lowering path. Existing `Int` locals can also be updated with arithmetic-tail
pipeline scalar expressions, such as `total = total + words.map(...).filter(...).len()`.
Negated pipeline scalar `contains(...)` expressions are covered in Bool locals,
Bool reassignments, `if` conditions, and `while` conditions.
Bool `if ... then ... else ...` expressions built from pipeline scalar
conditions infer `Bool` and support later exact Bool pipeline scalar
reassignments, helper-call arguments, Bool returns, and helper-call arguments
nested inside reassignment expressions.
Int `if ... then ... else ...` expressions built from pipeline scalar
conditions are covered in locals, reassignments, helper-call arguments, and
returns for scoring-style code.
`List<Int>` map bodies may capture known `Int` parameters or locals.
`map(|x| int_expr).sum()`, `.max()`, and `.min()` are covered for same-item
`List<Int>` transformed aggregation/ranking in `return` expressions, typed or
inferred `Int` local assignments, direct `Int` helper-call arguments, direct
`List<Int>` mutation arguments, known `Int` reassignments, broader `Int`
expressions, and broader `if`, `while`, and `else if` condition expressions.
They lower directly over the source list without requiring a user-written
temporary result list.
`filter(|x| predicate).sum()` is covered for `List<Int>` in `return`
expressions and typed or inferred `Int` local assignments, and may capture
known locals or parameters in the lowered predicate.
`filter(|x| predicate).len()` is covered for `List<Int>`, `List<Str>`, and
declared `List<Struct>` values in `return` expressions and typed or inferred
`Int` local assignments, and may capture known locals or parameters plus struct
field predicates in the lowered predicate.
`filter(|x| predicate).max()` and `.min()` are covered for `List<Int>` values
in `return` expressions and typed or inferred `Int` local assignments. They
lower directly to a guarded selection loop without building an intermediate
list, and trap when the predicate selects no elements.
`filter(|x| predicate).first().field` and `.last().field` are covered for
declared `List<Struct>` values in `return` expressions with `Int`/`Str`
function return types, typed or inferred `Int`/`Str` local assignments, and
direct `Int`/`Str` helper-call arguments, including helpers declared later.
Simple arithmetic suffixes on `Int` helper calls are preserved after lowering.
Helper calls with filtered first/last arguments can also start `if`, `else if`,
or `while` condition expressions. In `while` conditions, the lowered selections
are recomputed on each iteration.
`Str` field `.len()` chains are
covered for `Int` returns, typed or inferred `Int` local assignments, and
direct `Int` helper-call arguments. They lower directly to a guarded
field-selection loop without building an intermediate record list, and trap
when the predicate selects no elements.
`filter(|x| predicate).first()` and `.last()` are covered for declared
`List<Struct>` values, including multiline struct declarations, in same-struct
`return` expressions, typed or inferred same-struct local assignments, and
direct same-struct helper-call arguments, including helpers declared later.
Simple arithmetic suffixes on `Int` helper calls are preserved after lowering.
Helper calls with filtered first/last arguments can also start `if`, `else if`,
or `while` condition expressions. In `while` conditions, the lowered selections
are recomputed on each iteration.
They lower directly to a guarded
record-selection loop without building an intermediate record list, copying the
matched record fields into the selected value, and trap when the predicate
selects no elements.
`filter(|x| predicate).map(|x| int_expr).sum()` is covered for same-item
`List<Int>` transformed aggregation and declared `List<Struct>` values in
`return` expressions, typed or inferred `Int` local assignments, and direct
`Int` helper-call arguments including helper calls starting `if`, `else if`,
or `while` condition expressions, and lowers directly to an accumulator over the
source list. For same-item `List<Int>` and declared `List<Struct>` receivers,
these aggregates can also appear inside broader `Int` expressions used by
locals, helper-call arguments, direct `List<Int>` mutation arguments,
reassignments, and returns. For same-item `List<Int>` and declared
`List<Struct>` receivers, they also cover broader `if`, `while`, and `else if`
condition expressions.
`filter(|x| predicate).map(|x| int_expr).max()` and `.min()` are covered
for same-item `List<Int>` transformed ranking and declared `List<Struct>`
values in `return` expressions, typed or inferred `Int` local assignments, and
direct `Int` helper-call arguments including helper calls starting `if`,
`else if`, or `while` condition expressions. For same-item `List<Int>` and
declared `List<Struct>` receivers, they can also appear inside broader `Int`
expressions used by locals, helper-call arguments, direct `List<Int>` mutation
arguments, reassignments, and returns. For same-item `List<Int>` and declared
`List<Struct>` receivers, they also cover broader `if`, `while`, and `else if`
condition expressions, and lower directly to a guarded selection loop over the
source list.
`List<Struct>.filter(|x| predicate).map(|x| x.field)` projected result lists
are covered for reusable `List<Int>` locals, annotated `List<Str>` locals,
existing `List<Int>`/`List<Str>` variable reassignments, direct
`List<Int>`/`List<Str>` helper returns, direct helper-call arguments,
helper-call conditions, and direct `List<Int>`/`List<Str>.extend(...)`
arguments without a user-written intermediate record list.
`List<Struct>.map(|x| x.field)` projected result lists are covered for
reusable locals, direct `List<Int>`/`List<Str>` helper returns, direct
helper-call arguments including helper-call conditions, direct `extend(...)`
sources, and existing-list reassignment without a user-written intermediate
list. `Int` field projections also support direct `.sum()`, `.max()`, and
`.min()` aggregation in helper returns, typed or inferred `Int` locals,
helper-call arguments including simple arithmetic suffixes, standalone simple
arithmetic suffixes, direct `List<Int>` mutation arguments, known `Int`
reassignments, broader `Int` expressions, and broader `if`/`while`/`else if`
condition expressions.
The direct gate also covers named `for x in xs` iteration and function calls where `List<Int>` parameters are
local list names or inline list values. It also covers `List<Int>`-returning helper calls passed
directly to `List<Int>` parameters in `return`, `let`, list-literal item, `push`, assignment
statements, `if`, `else if`, and `while` conditions. In the direct engine
native ABI, `List<Int>`
parameters are passed by reference, so `push`, `clear`, `pop`, `remove_at`,
`insert_at`, and `extend` on a local-list parameter mutate the caller's local list, and
assigning a new list to a list parameter replaces the caller's local list.
Local and parameter `List<Int>`, `List<Str>`, and `List<Struct>` targets also
accept inline list literals and same-type list-returning helper calls as
`extend(...)` sources.
Local and parameter `List<Struct>` assignment also accepts inline same-type
struct list literals, replacing the target length and caller-visible parameter
contents.
Typed local `List<Struct>` literals may be split across lines and may use a
trailing comma before the closing bracket. Inline `List<Struct>` literal
arguments to `List<Struct>` parameters may use the same multiline trailing-comma
form in expression contexts and standalone call statements.
`List<Struct>.push(Box { ... })` also accepts multiline struct literals with a
trailing comma before the closing brace in the full self-host path and native
direct engine.
`List<Struct>` indexed element assignment and struct-returning `return`
statements also accept multiline struct literals in the full self-host path and
native direct engine.
`List<Struct>.insert_at(index, Box { ... })` and
`List<Struct>.extend([Box { ... }])` also accept multiline struct literal
sources in the full self-host path and native direct engine.
For elements whose declared struct field is a previously declared nested struct,
indexed reads and writes can use a nested field chain such as `xs[0].inner.v` or
`xs[0].inner.v = value` in local and parameter `List<Struct>` values. The
verified multi-field nested slice covers struct-local `push`, whole-element
copy/assignment, local and parameter indexed nested reads/writes, and
method-result nested reads.
Plain struct locals and struct call arguments also accept multiline struct
literals in the full self-host path and native direct engine, including
inferred local initialization, typed local initialization, and same-type local
assignment.
`push`, `clear`, `pop`, `remove_at`, `insert_at`, `extend`, or assignment on an inline or
returned-list temporary mutates only that temporary value. `List<Int>` return values are
returned by value. The
same parameter-reference and return-by-value ABI applies to `List<Struct>` for
declared structs, including inline list arguments and `List<Struct>`-returning
helper calls passed directly to `List<Struct>` parameters in statement contexts
plus `if`, `else if`, and `while` conditions. Local `List<Int>` elements in the
full self-host path, and direct-engine `List<Int>`/`List<Struct>` elements, can
be assigned with `xs[index] = value`. The full self-host path also covers
`List<Struct>` indexed whole-element assignment for local and parameter lists
from same-type struct literals, struct locals, list elements, and
struct-returning helper calls. The full self-host path and direct engine cover
local and parameter `List<Struct>.push(value)` from same-type struct local or
parameter values, local/parameter `List<Struct>.push(make_struct(...))` from
same-type struct-returning helper calls, and local/parameter
`List<Struct>.insert_at(index, make_struct(...))` from same-type
struct-returning helper calls. They also cover local/parameter
`List<Struct>.push(xs[i])` and `List<Struct>.insert_at(index, xs[i])` from
same-type list element values, including same-list insertion, plus
`List<Struct>.push(xs.pop()/xs.remove_at(i))` and
`List<Struct>.insert_at(index, xs.pop()/xs.remove_at(i))` from same-type list
method return values, plus `List<Struct>.push(xs.first()/xs.last())` and
`List<Struct>.insert_at(index, xs.first()/xs.last())` from non-mutating
same-type list method return values, including same-list insertion
materialization. The full self-host path and direct engine cover
indexed `List<Struct>` element field assignment with `xs[index].field = value`,
including list parameters.
`List<Struct>.first()`, `List<Struct>.last()`, `List<Struct>.pop()`, and
`List<Struct>.remove_at(index)` can be bound to a struct local before reading
fields, or read directly with method-result field chains such as
`xs.first().tag`, `xs.last().value`, `xs.pop().tag`, and
`xs.remove_at(i).value`. Verified nested method-result field chains can also
read through nested fields such as `xs.last().inner.v`; the multi-field nested
slice is covered for non-mutating method results. `Str` fields on struct list
records can be compared, passed to string helpers, used with direct `.len()`
chains through local fields, indexes, non-mutating method results such as
`docs.last().body.len()`, and mutating method results such as
`docs.pop().title` or `docs.remove_at(i).body.len()`. They can also be reassigned
through indexed local or parameter lists with `docs[index].title = value`. `List<Struct>.remove_at(index)` shifts following
struct elements left, and `List<Struct>.insert_at(index, value)` shifts following struct elements
right and accepts declared-struct literal/local/parameter/list-element/list-method
values.
`List<Struct>.extend(other)` appends a same-type named source list, an inline
same-type struct list literal including multiline struct literal elements, or a
same-type `List<Struct>`-returning helper call, copying struct fields while
mutating local/parameter list length.
`for item in xs` over `List<Struct>` copies each struct element field-by-field
into the loop variable for local and parameter lists.
The direct engine also covers local and parameter `List<Str>.contains(value)`,
`List<Str>.index_of(value)`, `List<Str>.count(value)`, and
`List<Str>.remove_at(index)`/`List<Str>.insert_at(index, value)`/
`List<Str>.extend(other)` including inline list literal and list-returning
helper call sources, and `List<Str>.first()` with string equality.
`sum()` on `List<Struct>` is not a direct-engine release claim.

List method forms outside the verified `List<Int>.contains(...)`,
`List<Int>.index_of(...)`,
`List<Int>.count(...)`,
`List<Int>.remove_at(...)`,
`List<Int>.insert_at(...)`,
`List<Int>.extend(...)`,
`List<Int>.first(...)`,
`List<Str>.contains(...)`,
`List<Str>.index_of(...)`,
`List<Str>.count(...)`,
`List<Str>.remove_at(...)`,
`List<Str>.insert_at(...)`,
`List<Str>.extend(...)`,
`List<Str>.first(...)`,
`List<Struct>.first()`,
`List<Struct>.remove_at(...)`,
`List<Struct>.insert_at(...)`,
`List<Struct>.extend(...)`,
multi-value or escaping list closure captures, broader list method result types
beyond the verified `List<Int>` and `List<Str>` map/filter slices,
and method forms outside the verified
simple struct `impl`/`impl Trait for Struct` return-expression slices, are not
release-surface claims yet.
Broader `List<List<T>>` mutation, parameter passing, returns, and dynamic row
selection are not release-surface claims yet.

## Maps

`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
`Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>` have verified slices in
the full self-host compiler path and native direct engine. All seven support
`{}`, assignment copy, `insert`, `get(key, default)`, `remove`, `clear`,
`get_opt(key)` match binding, `contains`, `len`, `key_at(index)`, and
`value_at(index)`.
`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
`Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>` support function return values that
initialize a local; the explicit annotation is optional when the initializer is
a same-type Map-returning call. They also support function parameters
by reference; a callee can mutate the caller-visible Map with `insert`,
`remove`, `clear`, or same-type assignment. A Map parameter can be the source or
target of `target = source`, and assignment copies contents instead of creating
an alias.

Verified example:

```vais
fn main() -> Int {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 38)
    scores.insert(4, 40)
    scores.insert(9, 2)
    scores.remove(5)
    let copy: Map<Int,Int> = {}
    copy = scores
    scores.insert(4, 1)
    let found = copy.get(4, 0)
    let missing = scores.get(5, 0 - 1)
    let maybe = match scores.get_opt(9) { Some(v) => v, None => 0 }
    return found + missing + maybe + scores.contains(4) + scores.contains(5) + scores.len()
}
```

```vais
fn put(scores: Map<Int,Int>, key: Int, value: Int) -> Int {
    scores.insert(key, value)
    return scores.len()
}

fn main() -> Int {
    let scores: Map<Int,Int> = {}
    let n = put(scores, 4, 40)
    return scores.get(4, 0) + n + scores.contains(4)
}
```

```vais
fn make_scores() -> Map<Int,Int> {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 40)
    scores.insert(9, 2)
    return scores
}

fn main() -> Int {
    let scores: Map<Int,Int> = make_scores()
    if scores.contains(4) and scores.contains(9) {
        return scores.get(4, 0) + scores.get(9, 0)
    }
    return 0
}
```

```vais
fn make_names() -> Map<Str,Int> {
    let scores: Map<Str,Int> = {}
    scores.insert("red", 40)
    scores.insert("blue", 2)
    return scores
}

fn main() -> Int {
    let scores: Map<Str,Int> = make_names()
    let copy: Map<Str,Int> = {}
    copy = scores
    scores.remove("blue")
    if copy.contains("blue") and scores.contains("red") and not scores.contains("blue") {
        return scores.get("red", 0) + copy.get("blue", 0)
    }
    return 0
}
```

```vais
fn make_flags() -> Map<Int,Bool> {
    let flags: Map<Int,Bool> = {}
    flags.insert(4, true)
    flags.insert(9, false)
    return flags
}

fn main() -> Int {
    let flags: Map<Int,Bool> = make_flags()
    if flags.get(4, false) and not flags.get(9, true) and flags.contains(4) and flags.len() == 2 {
        return 42
    }
    return 0
}
```

```vais
fn mark(flags: Map<Int,Bool>, key: Int) -> Int {
    flags.insert(key, true)
    if flags.get(key, false) and flags.len() == 1 {
        return 40
    }
    return 0
}

fn main() -> Int {
    let flags: Map<Int,Bool> = {}
    let n = mark(flags, 4)
    if flags.get(4, false) and flags.contains(4) {
        return n + flags.len() + flags.contains(4)
    }
    return 0
}
```

```vais
fn main() -> Int {
    let flags: Map<Int,Bool> = {}
    let copy: Map<Int,Bool> = {}
    flags.insert(4, true)
    flags.insert(5, false)
    copy = flags
    if copy.get(4, false) and not copy.get(5, true) and copy.contains(4) {
        return 42
    }
    return 0
}
```

```vais
fn main() -> Int {
    let flags: Map<Int,Bool> = {}
    flags.insert(4, true)
    let yes_value = match flags.get_opt(4) { Some(v) => v, None => 0 }

    let letters: Map<Int,Char> = {}
    letters.insert(4, 'A')
    let letter_value = match letters.get_opt(4) { Some(v) => v, None => 58 }

    return yes_value * 20 + letter_value - 43
}
```

```vais
fn main() -> Int {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 1)
    scores.clear()
    scores.insert(9, 40)
    if not scores.contains(4) and scores.get(9, 0) == 40 and scores.len() == 1 {
        return 42
    }
    return 0
}
```

```vais
fn make_letters() -> Map<Int,Char> {
    let letters: Map<Int,Char> = {}
    letters.insert(4, 'A')
    letters.insert(9, 'B')
    return letters
}

fn main() -> Int {
    let letters: Map<Int,Char> = make_letters()
    if letters.get(4, 'Z') == 'A' and letters.get(9, 'Z') == 'B' and letters.contains(4) and letters.len() == 2 {
        return 42
    }
    return 0
}
```

```vais
fn stamp(letters: Map<Int,Char>, key: Int) -> Int {
    letters.insert(key, 'A')
    if letters.get(key, 'Z') == 'A' and letters.len() == 1 {
        return 40
    }
    return 0
}

fn main() -> Int {
    let letters: Map<Int,Char> = {}
    let n = stamp(letters, 4)
    if letters.get(4, 'Z') == 'A' and letters.contains(4) {
        return n + letters.len() + letters.contains(4)
    }
    return 0
}
```

```vais
fn main() -> Int {
    let letters: Map<Int,Char> = {}
    let copy: Map<Int,Char> = {}
    letters.insert(4, 'A')
    letters.insert(5, 'B')
    copy = letters
    letters.insert(4, 'Z')
    if copy.get(4, 'Z') == 'A' and copy.get(5, 'Z') == 'B' and copy.contains(4) {
        return 42
    }
    return 0
}
```

```vais
fn main() -> Int {
    let flags: Map<Str,Bool> = {}
    let red: Str = "red"
    flags.insert("red", true)
    flags.insert("blue", false)
    flags.insert(red, true)

    let copy: Map<Str,Bool> = {}
    copy = flags

    flags.remove("blue")
    flags.clear()
    flags.insert("green", true)

    let yes_value = match copy.get_opt("red") { Some(v) => v, None => 0 }
    let no_value = match copy.get_opt("blue") { Some(v) => v, None => 1 }
    let missing = match flags.get_opt("red") { Some(v) => 0, None => 5 }

    if copy.contains("blue") and not flags.contains("red") and flags.get("green", false) and flags.len() == 1 {
        return yes_value * 20 + (1 - no_value) * 10 + missing + flags.contains("green") + copy.len() * 3
    }
    return 0
}
```

```vais
fn mark(flags: Map<Str,Bool>, key: Str, value: Bool) -> Int {
    flags.insert(key, value)
    flags.insert("blue", false)
    flags.remove("blue")
    flags.insert("green", true)
    return flags.len()
}

fn main() -> Int {
    let flags: Map<Str,Bool> = {}
    let red: Str = "red"
    let n = mark(flags, red, true)
    let red_value = match flags.get_opt(red) { Some(v) => v, None => 0 }
    if flags.contains("green") and not flags.contains("blue") and flags.get("green", false) {
        return n * 20 + red_value + flags.contains("green")
    }
    return 0
}
```

```vais
fn make_flags() -> Map<Str,Bool> {
    let flags: Map<Str,Bool> = {}
    flags.insert("red", true)
    flags.insert("blue", false)
    return flags
}

fn main() -> Int {
    let flags: Map<Str,Bool> = make_flags()
    let copy: Map<Str,Bool> = {}
    copy = flags
    flags.remove("blue")
    let red_value = match flags.get_opt("red") { Some(v) => v, None => 0 }
    if copy.contains("blue") and flags.contains("red") and not flags.contains("blue") {
        return red_value * 40 + copy.len()
    }
    return 0
}
```

```vais
fn put_name(scores: Map<Str,Int>, key: Str, value: Int) -> Int {
    scores.insert(key, value)
    scores.insert("blue", 2)
    scores.remove("blue")
    return scores.len()
}

fn main() -> Int {
    let scores: Map<Str,Int> = {}
    let red: Str = "red"
    let n = put_name(scores, red, 40)
    if scores.contains("red") and not scores.contains("blue") {
        return scores.get("red", 0) + n + scores.len()
    }
    return 0
}
```

```vais
fn main() -> Int {
    let scores: Map<Str,Int> = {}
    let red: Str = "red"
    scores.insert("red", 10)
    scores.insert("blue", 20)
    scores.insert(red, 40)

    let copy: Map<Str,Int> = {}
    copy = scores

    scores.remove("blue")
    scores.clear()
    scores.insert("green", 1)

    let maybe = match copy.get_opt("red") { Some(v) => v, None => 0 }
    if copy.contains("blue") and not scores.contains("red") and scores.get("green", 0) == 1 and scores.len() == 1 {
        return maybe + copy.get("blue", 0) - 18
    }
    return 0
}
```

```vais
fn main() -> Int {
    let letters: Map<Str,Char> = {}
    let red: Str = "red"
    letters.insert("red", 'A')
    letters.insert("blue", 'B')
    letters.insert(red, 'A')

    let copy: Map<Str,Char> = {}
    copy = letters

    letters.remove("blue")
    letters.clear()
    letters.insert("green", 'C')

    let red_value = match copy.get_opt("red") { Some(v) => v, None => 0 }
    let blue_value = match copy.get_opt("blue") { Some(v) => v, None => 0 }
    let missing = match letters.get_opt("red") { Some(v) => 0, None => 5 }

    if copy.contains("blue") and not letters.contains("red") and letters.get("green", 'Z') == 'C' and letters.len() == 1 {
        return red_value - 65 + blue_value - 66 + missing + letters.contains("green") + copy.len() * 18
    }
    return 0
}
```

```vais
fn stamp(letters: Map<Str,Char>, key: Str, value: Char) -> Int {
    letters.insert(key, value)
    letters.insert("green", 'C')
    return letters.len()
}

fn main() -> Int {
    let letters: Map<Str,Char> = {}
    let red: Str = "red"
    let n = stamp(letters, red, 'A')
    if letters.get(red, 'Z') == 'A' and letters.get("green", 'Z') == 'C' {
        return n * 20 + letters.contains("green") + letters.len() - 1
    }
    return 0
}
```

```vais
fn make_letters() -> Map<Str,Char> {
    let letters: Map<Str,Char> = {}
    letters.insert("red", 'A')
    letters.insert("blue", 'B')
    return letters
}

fn main() -> Int {
    let letters: Map<Str,Char> = make_letters()
    let copy: Map<Str,Char> = {}
    copy = letters
    letters.remove("blue")
    if copy.contains("blue") and letters.contains("red") and not letters.contains("blue") {
        return letters.get("red", 'Z') - 65 + copy.get("blue", 'Z') - 66 + copy.len() * 20 + letters.len() + copy.contains("blue")
    }
    return 0
}
```

```vais
fn put_doc(docs: Map<Str,Str>, key: Str, value: Str) -> Int {
    docs.insert(key, value)
    docs.insert("tmp", "drop")
    docs.remove("tmp")
    return docs.len()
}

fn make_docs() -> Map<Str,Str> {
    let docs: Map<Str,Str> = {}
    docs.insert("title", "VaisDB")
    docs.insert("body", "cache")
    return docs
}

fn main() -> Int {
    let docs: Map<Str,Str> = make_docs()
    let n = put_doc(docs, "tail", "learned")
    let copy: Map<Str,Str> = {}
    copy = docs
    docs.remove("body")
    if copy.get("title", "") == "VaisDB" and copy.get("tail", "") == "learned" and docs.get("body", "missing") == "missing" {
        return n * 10 + copy.len() * 4
    }
    return 0
}
```

Verified behavior:

- Local `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  `Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>` values are supported.
- `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  `Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>` can also be passed as function
  parameters by reference.
- `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  `Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>` return values can initialize an explicitly
  annotated local, copying returned contents into caller-owned storage.
- Map-returning calls can also initialize unannotated locals, with the local Map
  type inferred from the helper return type. See
  `examples/e139_map_return_infer.vais`.
- Str-returning Map methods can feed directly into `.len()`, for example
  `docs.get("title", "").len()` or `docs.value_at(1).len()`, across the verified
  full and direct paths.
- `{}` constructs an empty map when the local type is explicitly one of the
  verified concrete Map types.
- `target = source` copies a same-type Map local, same-type Map parameter, or
  same-type Map-returning call into the target Map; later mutation of either
  map does not alias the other. Parameter-source/target copies are covered by
  `examples/e116_map_param_assignment.vais` and
  `examples/e119_map_param_target_assignment.vais`, and Map-returning call
  assignment copies are covered by `examples/e117_map_return_assignment.vais`
  and `examples/e118_map_return_assignment_args.vais`.
- `insert(key, value)` inserts or replaces a value.
- `remove(key)` removes a present key if it exists; removing a missing key is a
  no-op.
- `clear()` removes all keys and allows the map to be reused, as covered by
  `examples/e106_map_clear.vais`.
- `get(key, default)` returns `default` when the key is absent.
- `get_opt(key)` returns `Some(value)` when the key is present and `None` when
  absent for `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`,
  `Map<Str,Int>`, `Map<Str,Bool>`, `Map<Str,Char>`, and `Map<Str,Str>`, as covered by `examples/e94_map_get_opt.vais`,
  `examples/e105_map_scalar_get_opt.vais`, `examples/e107_map_str_int.vais`,
  `examples/e108_map_str_int_param.vais`, and
  `examples/e109_map_str_int_return.vais`, `examples/e110_map_str_bool.vais`,
  `examples/e111_map_str_bool_param.vais`, and
  `examples/e112_map_str_bool_return.vais`, plus
  `examples/e113_map_str_char.vais`, `examples/e114_map_str_char_param.vais`,
  `examples/e115_map_str_char_return.vais`,
  `examples/e116_map_param_assignment.vais`, and
  `examples/e117_map_return_assignment.vais`, plus
  `examples/e118_map_return_assignment_args.vais` and
  `examples/e135_map_str_str_get_opt.vais`. `Map<Str,Str>.get_opt` supports
  direct `match` binding to a `Str` payload or to an Int expression derived from
  that payload, such as `Some(v) => v.len()`. It also supports string payload
  match expressions in returns, reassignments, helper-call arguments, and
  embedded Int return expressions, as covered by
  `examples/e280_map_str_str_get_opt_match_contexts.vais`. The same contexts
  are verified when the local map type is inferred from a `Map<Str,Str>` helper
  return, as covered by
  `examples/e281_map_str_str_return_infer_get_opt_match_contexts.vais`. String
  payload match expressions also support `str_concat`, `str_trim`, `str_lower`,
  `str_upper`, and `str_replace` transforms in verified `Str` contexts, as
  covered by `examples/e282_map_str_str_get_opt_match_str_transforms.vais` and
  `examples/e289_str_replace.vais`. Reassigned
  `Str` locals read their current runtime string when `.len()` is applied after
  those match-transform results, as covered by
  `examples/e283_str_len_reassigned_match_transform.vais`. Match arms can also
  compute `str_trim`/`str_lower` transform lengths directly with `.len()`, as
  covered by `examples/e284_map_str_str_get_opt_match_transform_len.vais`.
  String payload match lowering uses a map presence check and value load instead
  of pointer-tagged string payload integers, so saved `Str` payload locals stay
  stable across later embedded match/string helper expressions, including
  helper calls used inside `if`/`while` conditions, as covered by
  `examples/e285_map_str_str_get_opt_str_payload_stability.vais`. Embedded
  string payload matches are also verified in `while` and `else if` condition
  chains, preserving per-iteration loop reevaluation and else-chain structure,
  as covered by `examples/e286_map_str_str_get_opt_condition_chains.vais`.
- `contains(key)` returns whether a key is present.
- `len()` returns the number of present keys.
- `key_at(index)` and `value_at(index)` return the key or value stored at a
  compact entry index in `0..m.len()`. These methods are intended for
  serialization/debugging loops over concrete Maps, not sorted ordering; remove
  compacts storage by moving the last entry into the removed slot. See
  `examples/e136_map_entries.vais`.
- `examples/e137_map_str_str_snapshot.vais` combines `Map<Str,Str>` entry reads
  with `str_builder`, `fs_write_text`, and `fs_read_text` to write/read a simple
  text snapshot. This is the current minimal storage-oriented slice for document
  metadata or small knowledge-entry probes.
- `examples/e138_map_str_str_snapshot_load.vais` extends that storage slice by
  parsing the loaded text snapshot back into a `Map<Str,Str>` and verifying
  lookups from the rebuilt map.
- `examples/e293_map_str_str_snapshot_builtin.vais` promotes the same
  `Map<Str,Str>` metadata storage pattern to builtins:
  `map_str_str_snapshot(docs) -> Str` emits one `key=value` record per LF line
  in compact map entry order, and `map_str_str_load_snapshot(text, docs) -> Int`
  clears `docs`, loads LF/CRLF `key=value` lines, skips blank/malformed/no-key
  lines, preserves additional `=` bytes in values, allows empty values, and
  returns the number of loaded entries.
- `examples/e139_map_return_infer.vais` protects local Map type inference from
  Map-returning calls such as `let docs = make_docs()` and direct `.len()` chains
  on Str-valued Map reads.

Not included in the current Map slice: broader generic key/value lowering,
iteration, entry literals, broader Map APIs that return `Option`, `Result`,
custom hashing, broader `Map<Str,V>` return values, full JSON parsing, escaped
structured-text fields, or public ABI claims for generic Map return values.
Unverified generic Map parameters, unverified return values, and non-promoted
assignment sources are rejected by front diagnostics instead of being treated as
part of the release surface.
The future Map ABI and generic expansion contract is specified in
`docs/design/MAP_ABI.md`, but no broader Map behavior is verified until it has
compiler gates.

## Option And Result

`Option<Int>` has a first verified slice across the full and direct compiler
paths:

```vais
fn find(x: Int) -> Option<Int> {
    if x > 0 { return Some(x * 2) }
    return None
}

fn main() -> Int {
    match find(21) {
        Some(v) => return v,
        None => return 0,
    }
}
```

Verified behavior:

- `Some(Int)` and `None` constructors.
- `Option<Int>` helper return values.
- `Option<Int>` stored in a simple struct field and matched through field
  access, as covered by `examples/e40_option_in_struct.vais`.
- Statement-form `match` arms that return from the current function.
- Expression-form `let x = match ...` bindings for `Option<Int>`, including
  multiline arm blocks, as covered by `examples/e23_option_flow.vais` and
  `examples/d2.vais`.
- Local-binding `?` propagation for `Option<Int>` helper calls: `None` returns
  from the current `Option<Int>` function, and `Some` binds its payload, as
  covered by `examples/e93_option_question.vais`.
- Direct `Option<Int>` helper-return matching from `main`, as covered by
  `examples/e08_option_chain.vais`.
- Native direct concrete `Option<Int>` value lowering for helper
  return/parameter/local types, `Some`/`None` constructors, expression-match
  bindings, and local-binding `?` propagation.
- `Option<Int>` carried as a single enum payload and matched in a nested
  statement arm, as covered by `examples/e79_nested_match.vais`.

`Result<Int,Int>` has the same first statement-match shape:

```vais
fn div(a: Int, b: Int) -> Result<Int, Int> {
    if b == 0 { return Err(0) }
    return Ok(a / b)
}

fn main() -> Int {
    match div(21, 3) {
        Ok(v) => return v,
        Err(e) => return e,
    }
}
```

It also supports expression-form `let x = match ...` bindings for
`Result<Int,Int>`, as covered by `examples/e91_result_flow.vais`.
The `?` operator is verified for `Result<Int,Int>` helper calls assigned to a
local: an `Err` value returns from the current `Result<Int,Int>` function, and an
`Ok` value binds its payload, as covered by `examples/e39_error_propagate.vais`
and `examples/e92_result_question_success.vais`.
Helper-chain `Result<Int,Int>` propagation is covered by `examples/d3run.vais`.
`examples/e294_result_try_parse_error_flow.vais` promotes the same concrete
`Result<Int,Int>` parse/error pattern through native direct and full paths:
helpers can return `Ok(Int)`/`Err(Int)`, callers can use local-binding `?` to
return `Err` early, and `main` can recover values through inline
expression-form `match`.
`examples/e296_result_map_param_flow.vais` verifies the follow-on helper shape
needed by document metadata code: `Result<Int,Int>` helpers can take
`Map<Str,Str>` parameters, match `Map<Str,Str>.get_opt` inside the helper,
propagate another Result helper with local-binding `?`, and recover the payload
through inline `Ok`/`Err` match in `main`. The full self-host codegen gate
protects the same surface through `case_080g7_result_map_param_flow`.
`examples/e298_vaisdb_file_ingest_result_flow.vais` extends the product file
workflow with `fs_exists` guarded reads: helpers return `Err(10)` for a missing
path, `Err(11)` or `Err(12)` for malformed text, propagate those errors with
local-binding `?`, and keep the current `fs_read_text` trap path out of normal
missing-file control flow. For now, bind a `Result<Int,Int>` helper call to a
local before matching it when the call also contains nested host-helper
arguments; that is the gate-backed shape used by e298 across direct and full
paths.
`examples/e301_result_str_int_file_read.vais` adds the next concrete Result
payload slice for file reads. A helper can return `Result<Str,Int>`, construct
`Ok(text)` or `Err(code)`, use local-binding `?` inside another
`Result<Str,Int>` helper to bind the `Str` payload, and recover through inline
matches that produce either `Int` values such as lengths/error codes or `Str`
values such as normalized text. This is still a concrete slice, not generic
`Result<T,E>` support.
`examples/e302_result_str_int_param_flow.vais` extends the same concrete shape
to helper parameters: a `Result<Str,Int>` local can be passed into another
helper, forwarded again, and matched there to recover string payloads or integer
error values.
`examples/e303_result_metric_int_struct_payload.vais` opens the first
structured Result payload slice. A helper can return `Result<Metric,Int>` with
`Ok(metric)` or `Err(code)`, callers can pass and forward that value through
helper parameters, and inline matches can recover `metric.docs` and
`metric.terms` or an integer error code. This is a concrete `Metric` slice, not
generic `Result<Struct,Int>` support.
`examples/e304_result_record_int_struct_payload.vais` broadens that verified
surface to declared Int-field struct payloads beyond `Metric`. The gate-backed
`Record` example returns, passes, forwards, and matches `Result<Record,Int>`,
including a three-field payload sum. This is still not full generic
`Result<T,E>` support.
`examples/e305_result_multiline_struct_payload.vais` removes the direct
source-lowering one-line declaration restriction for that concrete surface. The
gate-backed `Entry` example declares the payload struct over multiple lines,
returns, passes, forwards, and matches `Result<Entry,Int>`, and recovers four
Int fields.
`examples/e306_result_struct_str_fields.vais` extends the same declared
structured Result surface to document-like records with `Str` fields. The
gate-backed `DocSummary` example returns, passes, forwards, and matches
`Result<DocSummary,Int>`, and recovers `title.len()`/`summary.len()` together
with an Int field inside the Ok arm.
`examples/e307_result_struct_try_payload.vais` adds local-binding `?` to that
declared-struct Result surface. A Result-returning helper can bind a
`DocSummary` payload from `build_doc_summary(...)?`, reuse the extracted
`Str` and `Int` fields, and return the original integer error code early.
`examples/e308_vaisdb_artifact_record_workflow.vais` applies that surface to a
small VaisDB artifact record workflow. `DocArtifact` values flow through
`Result<DocArtifact,Int>` helpers, are extracted with local-binding `?`,
pushed into `List<DocArtifact>` output parameters, and paired with
`Map<Str,Str>` metadata snapshots while integer errors still propagate.
`examples/e309_vaisdb_artifact_store_snapshot.vais` persists those artifact
records as a tab-delimited text store, writes and reads the snapshot through
host file helpers, parses records back through `Result<DocArtifact,Int>`
helpers, and queries the best loaded record while malformed and missing store
paths remain integer `Result` errors.
`examples/e310_vaisdb_artifact_query_report.vais` adds the query/report layer:
persisted artifact records are loaded into `List<DocArtifact>`, ranked with
`Map<Str,Int>` term scoring, returned as a `Result<Str,Int>` report payload,
and checked against missing-store and empty-query integer errors.
`examples/e311_result_call_argument_flow.vais` removes the next call-site
ergonomics gap: `Result<Str,Int>` and `Result<DocArtifact,Int>` returning
helpers can be passed directly into other helpers without binding explicit
temporary locals first.
`examples/e312_result_struct_local_wrapper_flow.vais` removes the next
self-host wrapper gap: explicit `VaisResult<DocArtifact>Int`-style wrappers can
bind `flow.value` to a local `DocArtifact`, read all payload fields, and return
that local inside another wrapper literal without dropping nested fields.
`examples/e313_result_struct_str_match_flow.vais` removes the next
report-building gap for structured Results: matches over
`Result<DocArtifact,Int>` can recover `Str` fields such as `artifact.title`
or `artifact.id` directly into string locals while `Err(Int)` arms use
`Str(code)`.
`examples/e314_result_struct_str_concat_match_flow.vais` removes the follow-on
report-label gap: `Result<DocArtifact,Int>` matches can compose payload string
fields with nested `str_concat(...)` in the `Ok` arm while `Err(Int)` arms keep
using `Str(code)`.
`examples/e315_result_struct_str_transform_match_flow.vais` removes the next
normalization gap: `Result<DocArtifact,Int>` matches can apply `str_replace`,
`str_trim`, `str_upper`, `str_lower`, and local-prefix `str_concat(...)` to
payload string fields in the `Ok` arm while `Err(Int)` arms keep using
`Str(code)`.
`examples/e316_result_struct_str_transform_len_match_flow.vais` removes the
follow-on scoring gap: `Result<DocArtifact,Int>` matches can compute integer
scores from transformed payload string fields with chained `.len()` calls while
combining those terms with ordinary integer payload fields.
`examples/e317_result_struct_payload_helper_call_score.vais` removes the
reusable-helper gap: `Result<DocArtifact,Int>` matches can pass the `Ok`
payload directly to an `Int` helper such as `score_artifact(artifact)` while
`Err(Int)` arms keep returning error codes.
`examples/e318_result_struct_payload_helper_call_arithmetic.vais` removes the
helper-composition gap: `Result<DocArtifact,Int>` matches can use a reusable
`Ok` payload helper-call result as one `Int` term and add ordinary payload
fields such as `artifact.terms + artifact.score` in the same arm.
`examples/e319_result_struct_payload_field_helper_call_arithmetic.vais` removes
the field-helper composition gap: `Result<DocArtifact,Int>` matches can pass
`Ok` payload `Str` fields such as `artifact.title` and `artifact.body` to
reusable `Int` helpers, then add ordinary payload fields in the same arm.
`examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais`
removes the numeric field-helper composition gap: `Result<DocArtifact,Int>`
matches can pass `Ok` payload `Int` fields such as `artifact.terms` and
`artifact.score` to reusable `Int` helpers, then compose those terms with
string-field helper terms in the same arm.
`examples/e321_result_struct_payload_bool_match_condition.vais` removes the
Bool-return gap: `Result<DocArtifact,Int>` matches can return conditions
directly from `Ok` payload helper terms and `Err(Int)` code comparisons.
`examples/e322_vaisdb_module_boundary/main.vais` removes the first reusable
module-boundary gap for VaisDB-style programs: imported scoring and artifact
modules can share `DocArtifact`, structured Result helpers, List outputs, and
Map-backed term scoring in direct/default runs.
`examples/e323_cli_package` removes the first package-directory CLI gap:
manifest-backed package directories can be passed directly to `scripts/vaisc`
commands, which resolve `source/main.vais`, preserve imports, and forward argv
to the compiled program.
`scripts/vaisc package <package-dir> -o <dist-dir>` removes the next packaging
gap by building an argv-capable `dist/bin/<package-name>` binary and copying
`dist/vais.toml`.
`examples/e326_cli_binary_target` removes the first package target-metadata
gap: `binary = "veriqel-demo"` lets the packaged command name differ from the
package name while preserving `source/main.vais` entry resolution.
e327 adds user-package archive output: `--archive` creates an extractable
`<binary-or-name>-<version>.tar.gz` release payload and gates it in
native/direct/workflow paths.
`examples/e328_cli_package_assets` adds optional package static assets:
`assets = "assets"` copies regular files/directories to `dist/assets` and into
the archive payload as `assets/` while preserving argv-capable packaged
binaries.
`examples/e299_vaisdb_benchmark_report.vais` adds the next product workflow
fixture: Vais code times a document scoring pass with `time_millis()`, writes a
small report file, reads it back, and validates the recorded metrics.
`examples/e300_vaisdb_benchmark_cli_report.vais` extends that into the first
CLI-style benchmark/report fixture written in Vais: it finds the repository
root from `fs_cwd`, `path_dirname`, and `path_basename`, runs the indexer with
`proc_capture`, times direct/default runs, and persists status metrics.
`tools/vaisdb_benchmark_report.vais` turns the fixture into a reusable
Vais-authored developer command: it writes the raw report, parses the saved
metric lines with `str_split_lines_into`, `str_starts_with`, `str_slice`, and
`parse_int`, then writes a direct/default summary report.

Not included yet: generic `Option<T>`, `Result<T,E>`, or `Map<K,V>` beyond the
verified concrete Map shapes, broader expression-form `match` beyond the
gate-backed `Option<Int>`, `Result<Int,Int>`, and `Result<Str,Int>` binding
shapes and the declared `Result<Struct,Int>` structured-payload match
shape including multiline struct declarations, Str-field length recovery,
direct Str-field recovery, nested `str_concat(...)` field composition,
string-helper normalization transforms, transformed string length scoring,
Ok-payload helper-call scoring, helper-call term arithmetic with payload fields,
local-binding `?`, and the e308 artifact-record
`List<Struct>` storage slice,
`?` beyond the gate-backed `Option<Int>`, `Result<Int,Int>`, and
`Result<Str,Int>` local-binding shapes plus declared `Result<Struct,Int>`
payload bindings, broader Map APIs that return `Option`,
Result helper flows beyond the gate-backed `Map<Str,Str>` parameter/get_opt and
`fs_exists` guarded file-read plus `Result<Str,Int>` parameter-forwarding and
`Result<DeclaredStruct,Int>` structured-payload forwarding, including
multiline declarations, Str-field length recovery, direct Str-field recovery,
nested `str_concat(...)` field composition, string-helper normalization
transforms, and local-binding `?`,
slices, generic direct-engine Option/Result forms
beyond the concrete `Option<Int>`, `Result<Int,Int>`, `Result<Str,Int>`, and
declared `Result<Struct,Int>`
slices, and nested option/result payloads. Unsupported
generic `Option`, `Result`, and `Map` forms are rejected by front diagnostics
instead of being treated as verified language.

## Strings, Characters, And Output

```vais
fn main() -> Int {
    let x = 42
    print("the answer is {x}")
    putchar(33)
    return 0
}
```

Verified today:

- String literals with `"` or backtick delimiters.
- `Str` helper parameters, local values, reassignment, and helper return values,
  as covered by `examples/e89_str_type.vais`.
- `s.len()` and `s[i]` in the full self-host path and native direct engine.
- `a == b` and `a != b` for `Str` in the full self-host path, native direct
  engine, and parity examples.
- `str_concat(left, right)` and `str_byte(value)` in the full self-host runtime
  and native direct engine.
- `str_slice(text, start, len)` in the full self-host runtime and native direct
  engine.
- `str_contains(text, needle)` in the full self-host path and native direct
  engine.
- `str_index_of(text, needle)` in the full self-host path and native direct
  engine.
- `str_starts_with(text, prefix)` in the full self-host path and native direct
  engine.
- `str_ends_with(text, suffix)` in the full self-host path and native direct
  engine.
- `str_replace(text, needle, replacement)` in the full self-host path and native
  direct engine.
- `str_trim(text)` in the full self-host path and native direct engine.
- `str_lower(text)` in the full self-host path and native direct engine.
- `str_upper(text)` in the full self-host path and native direct engine.
- `str_split_into(text, sep, out)` in the full self-host path and native direct
  engine.
- `str_split_lines_into(text, out)` in the full self-host path and native
  direct engine.
- `str_join(parts, sep)` in the full self-host path and native direct engine.
- `Bool` byte-classification helpers such as `is_digit(c: Int) -> Bool`.
- Explicit `Bool` locals, helper parameters, helper returns, and unary `not`,
  as covered by `examples/e88_bool_type.vais`.
- Named integer parsing helpers `parse_uint(s)` and `parse_int(s)`, as covered
  by `examples/e83_parse_helpers.vais`.
- Literal string length, as covered by `examples/e44_string_len.vais`.
- Word-count state-machine scans over `Str`, as covered by
  `examples/e53_word_count.vais`.
- Two-pointer `Str` scans with computed byte indexes from both ends, as covered
  by `examples/e69_palindrome_string.vais`.
- User-defined integer parsing over `Str`, as covered by
  `examples/e70_parse_uint.vais`.
- Substring search patterns over `Str` with computed byte indexes, as covered by
  `examples/e71_string_index_of.vais`.
- Built-in substring checks with `str_contains(text, needle)`, including
  document metadata strings from `Map<Str,Str>` and token-list strings from
  `List<Str>`, as covered by
  `examples/e140_str_contains.vais`.
- Built-in first substring indexing with `str_index_of(text, needle)`, which
  returns the first byte index, `-1` when absent, and `0` for an empty needle,
  as covered by `examples/e149_str_index_of_builtin.vais`.
- Built-in prefix checks with `str_starts_with(text, prefix)`, which returns
  `1` when `text` begins with `prefix`, including an empty prefix, otherwise
  `0`, as covered by `examples/e150_str_starts_with_builtin.vais`.
- Built-in suffix checks with `str_ends_with(text, suffix)`, which returns
  `1` when `text` ends with `suffix`, including an empty suffix, otherwise
  `0`, and accepts normalized strings, `Map<Str,Str>` reads, `List<Str>` reads,
  and `Map<Str,Str>.get_opt` match values, as covered by
  `examples/e288_str_ends_with.vais`.
- Built-in string rewriting with `str_replace(text, needle, replacement)`,
  which replaces all non-overlapping occurrences, leaves the string unchanged
  when `needle` is empty, and accepts normalized strings, `Map<Str,Str>` reads,
  `List<Str>` reads, and `Map<Str,Str>.get_opt` match values, as covered by
  `examples/e289_str_replace.vais`.
- Built-in edge cleanup with `str_trim(text)`, including document metadata
  strings from `Map<Str,Str>` and token-list strings from `List<Str>`, as
  covered by `examples/e141_str_trim.vais`.
- Built-in ASCII lowercase normalization with `str_lower(text)`, including
  trimmed document metadata strings from `Map<Str,Str>` and token-list strings
  from `List<Str>`, as covered by `examples/e142_str_lower.vais`.
- Built-in ASCII uppercase normalization with `str_upper(text)`, including
  trimmed document metadata strings, `Map<Str,Str>` reads, `List<Str>` element
  reads, and `Map<Str,Str>.get_opt` match payload transforms, as covered by
  `examples/e287_str_upper.vais`.
- Document tokenization with `str_slice`, `str_trim`, `str_lower`,
  `str_contains`, and `List<Str>` return values, as covered by
  `examples/e143_doc_tokenize.vais`.
- Document token scoring with `List<Str>` for-each over normalized tokens, as
  covered by `examples/e144_doc_score_for_each.vais`.
- Built-in whitespace tokenization with `str_split_ws_into(text, out)`, which
  clears and fills a `List<Str>` out-param and returns the token count, as
  covered by `examples/e145_str_split_ws_into.vais`.
- Built-in delimiter tokenization with `str_split_into(text, sep, out)`, which
  clears and fills a `List<Str>` out-param, preserves empty fields, treats an
  empty separator as one whole-text field, and returns the field count, as
  covered by `examples/e290_str_split_into.vais`.
- Built-in line tokenization with `str_split_lines_into(text, out)`, which
  clears and fills a `List<Str>` out-param, splits on LF, trims a trailing CR
  from CRLF lines, preserves interior blank lines, omits the final empty line
  for a trailing line break, and returns the line count, as covered by
  `examples/e292_str_split_lines_into.vais`.
- Built-in string reconstruction with `str_join(parts, sep)`, which joins a
  local or parameter `List<Str>` with `sep` between elements, returns an empty
  string for an empty list, and preserves delimiter round trips when paired with
  `str_split_into`, as covered by `examples/e291_str_join.vais`.
- Built-in document term-frequency indexing with
  `doc_term_counts_into(text, out)`, which lowercases ASCII whitespace tokens,
  clears/fills a `Map<Str,Int>` out-param, and returns the total token count, as
  covered by `examples/e146_doc_term_counts_into.vais`.
- Built-in term-frequency overlap scoring with
  `doc_term_overlap_score(query, doc)`, which sums `min(query_count, doc_count)`
  for each query term across two `Map<Str,Int>` maps, as covered by
  `examples/e147_doc_term_overlap_score.vais`.
- Built-in weighted term-frequency scoring with
  `doc_term_weighted_score(query, doc)`, which sums
  `query_count * doc_count` for each query term across two `Map<Str,Int>` maps,
  as covered by `examples/e148_doc_term_weighted_score.vais`.
- Identifier scanning over `Str`, as covered by
  `examples/e72_identifier_scan.vais`.
- Single-byte character literal equality plus `Char` locals, helper
  parameters, and helper returns as Int-compatible scalar values in the front,
  native direct, full, and parity gates, as covered by
  `examples/e85_char_type.vais`.
- `print("...{name}...")` interpolation for simple identifiers in the full
  self-host path and native direct engine.
- `putchar(Int)` in the full self-host path and native direct engine.

`parse_uint` parses a leading unsigned decimal run and stops at the first
non-decimal byte. `parse_int` accepts a leading `-` and then parses the same
decimal run. Empty input and input with no leading decimal digit return `0`.
`Str(x)` converts an Int-compatible scalar value into a decimal `Str`, as
covered by `examples/e73_int_to_string.vais`.

## Host Files, Paths, And Processes

Phase 3 host APIs start with verified full-engine file, path, and argv-based
process intrinsics. The current and planned broader surface is:

```vais
fs_exists(path: Str) -> Bool
fs_is_dir(path: Str) -> Int
stdin_read_all() -> Str
stdout_write(text: Str) -> Int
stderr_write(text: Str) -> Int
proc_self() -> Str
fs_list_dirs(dir: Str, out: List<Str>) -> Int
fs_read_text(path: Str) -> Str
fs_write_text(path: Str, text: Str) -> Int
fs_mkdirs(path: Str) -> Int
fs_remove(path: Str) -> Int
fs_cwd() -> Str
fs_temp_dir() -> Str
path_join(base: Str, child: Str) -> Str
path_basename(path: Str) -> Str
path_dirname(path: Str) -> Str
str_concat(left: Str, right: Str) -> Str
str_cmp(left: Str, right: Str) -> Int
str_slice(text: Str, start: Int, len: Int) -> Str
str_contains(text: Str, needle: Str) -> Int
str_index_of(text: Str, needle: Str) -> Int
str_starts_with(text: Str, prefix: Str) -> Int
str_ends_with(text: Str, suffix: Str) -> Int
str_replace(text: Str, needle: Str, replacement: Str) -> Str
str_trim(text: Str) -> Str
str_lower(text: Str) -> Str
str_upper(text: Str) -> Str
str_split_ws_into(text: Str, out: List<Str>) -> Int
str_split_into(text: Str, sep: Str, out: List<Str>) -> Int
str_split_lines_into(text: Str, out: List<Str>) -> Int
str_join(parts: List<Str>, sep: Str) -> Str
doc_term_counts_into(text: Str, out: Map<Str,Int>) -> Int
doc_term_overlap_score(query: Map<Str,Int>, doc: Map<Str,Int>) -> Int
doc_term_weighted_score(query: Map<Str,Int>, doc: Map<Str,Int>) -> Int
str_byte(value: Int) -> Str
proc_run(argv: List<Str>) -> Int
proc_run_env(argv: List<Str>, env: List<Str>) -> Int
proc_capture_stdout(argv: List<Str>) -> Str
proc_capture_stderr(argv: List<Str>) -> Str
proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int
proc_capture(argv: List<Str>) -> ProcessResult
```

`fs_exists(path: Str) -> Bool`, `fs_read_text(path: Str) -> Str`,
`fs_write_text(path: Str, text: Str) -> Int`, and
`fs_mkdirs(path: Str) -> Int`, and `fs_remove(path: Str) -> Int`, plus `fs_cwd() -> Str`,
`fs_temp_dir() -> Str`, `path_join(base: Str, child: Str) -> Str`,
`path_basename(path: Str) -> Str`, `path_dirname(path: Str) -> Str`,
`str_concat(left: Str, right: Str) -> Str`, `str_slice(text: Str, start: Int,
len: Int) -> Str`, `str_byte(value: Int) -> Str`, and
`time_millis() -> Int` are gate-backed by
`scripts/test-vaisc-host.sh` for `scripts/vaisc build` and `scripts/vaisc run`;
`str_concat`, `str_slice`, and `str_byte` are checked there as self-contained
helper calls in generated IR rather than external host calls.
`fs_exists(path)` is also covered in the native direct engine by
`examples/e298_vaisdb_file_ingest_result_flow.vais`.
`examples/e301_result_str_int_file_read.vais` covers the same guarded file-read
path with a `Result<Str,Int>` success payload and missing-file error recovery.
`fs_cwd()`, `path_basename(path)`, and `path_dirname(path)` are also covered in
the native direct engine by `examples/e300_vaisdb_benchmark_cli_report.vais`.
`str_concat(left, right)` is also covered in the native direct engine by the
`List<Str>.map` concat parity example and direct feature gate.
`str_slice(text, start, len)` is also covered in the native direct engine by
the document tokenization parity example and direct feature gate.
`str_index_of(text, needle)` is covered in the full self-host and native direct
engines by `examples/e149_str_index_of_builtin.vais`.
`str_starts_with(text, prefix)` is covered in the full self-host and native
direct engines by `examples/e150_str_starts_with_builtin.vais`.
`str_ends_with(text, suffix)` is covered in the full self-host and native
direct engines by `examples/e288_str_ends_with.vais`.
`str_upper(text)` is covered in the full self-host and native direct engines by
`examples/e287_str_upper.vais`; it uppercases ASCII `a-z` only and accepts the
same verified `Str`-returning Map/List and get_opt match payload contexts as
the direct feature gate.
`str_split_ws_into(text, out)` is covered in the full self-host and native
direct gates by `examples/e145_str_split_ws_into.vais`.
`str_split_into(text, sep, out)` is covered in the full self-host and native
direct gates by `examples/e290_str_split_into.vais`; it preserves empty
delimiter fields and treats an empty separator as one whole-text field.
`str_split_lines_into(text, out)` is covered in the full self-host and native
direct gates by `examples/e292_str_split_lines_into.vais`; it splits LF/CRLF
line text into `List<Str>`, preserves interior blank lines, and omits the final
empty line for a trailing line break.
`str_join(parts, sep)` is covered in the full self-host and native direct gates
by `examples/e291_str_join.vais`; it joins `List<Str>` values with separators,
returns `""` for empty lists, and preserves split/join delimiter round trips.
`doc_term_counts_into(text, out)` is covered by
`examples/e146_doc_term_counts_into.vais` and provides the current verified
document term-frequency primitive for VaisDB experiments.
`doc_term_overlap_score(query, doc)` is covered by
`examples/e147_doc_term_overlap_score.vais` and provides the first verified
ranking primitive for VaisDB query/document term-frequency maps.
`doc_term_weighted_score(query, doc)` is covered by
`examples/e148_doc_term_weighted_score.vais` and rewards repeated term hits in
VaisDB query/document term-frequency maps.
`examples/e295_vaisdb_indexer_prototype.vais` combines those document helpers
with `Map<Str,Str>` metadata snapshots in a single Vais-authored index/query
prototype. It is a language dogfooding fixture, not a database engine API.
`examples/e296_result_map_param_flow.vais` verifies `Result<Int,Int>` helper
chains over `Map<Str,Str>` metadata parameters with `get_opt` matching and
local-binding `?` propagation.
`examples/e297_vaisdb_file_ingest_workflow.vais` extends the same dogfooding
path to file-backed ingest: it reads document/query files with `fs_read_text`,
creates reproducible temp inputs with `fs_temp_dir`, `path_join`, and
`fs_write_text`, accepts external paths through `proc_argc`/`proc_arg`, splits
lines, snapshots metadata, indexes terms, and scores the query in one Vais
workflow.
`examples/e298_vaisdb_file_ingest_result_flow.vais` adds the missing-file
error recipe for that path: file ingest helpers check `fs_exists`, return
`Result<Int,Int>` error codes instead of calling `fs_read_text` on absent
paths, and preserve the same generated-file plus argv-file workflow shape in
direct and full paths.
`examples/e301_result_str_int_file_read.vais` adds the string-payload Result
follow-up: guarded file-read helpers return `Result<Str,Int>`, `?` binds the
read text into another helper, and inline matches recover both normalized
string payloads and missing-file error codes. `examples/e302_result_str_int_param_flow.vais`
then verifies passing those `Result<Str,Int>` values through helper parameters
and forwarding them to other helpers before matching.
`examples/e303_result_metric_int_struct_payload.vais` then verifies the first
structured Result payload: `Result<Metric,Int>` helpers can return, accept, and
forward a struct payload, and inline matches can read `Metric` fields.
`examples/e304_result_record_int_struct_payload.vais` extends that structured
payload path beyond `Metric`: declared Int-field structs such as `Record` can
flow through `Result<DeclaredStruct,Int>` helpers and recover multiple fields
through inline matches.
`examples/e305_result_multiline_struct_payload.vais` verifies the same
declared-struct Result path when the payload record is written as a multiline
Int-field struct declaration.
`examples/e306_result_struct_str_fields.vais` verifies declared-struct Result
payloads with `Str` fields, recovering string field lengths and Int fields in
the Ok arm.
`examples/e307_result_struct_try_payload.vais` verifies local-binding `?` for
declared-struct Result payloads, including field reuse after extraction and
early integer error propagation.
`examples/e308_vaisdb_artifact_record_workflow.vais` verifies the first
VaisDB-style artifact record composition: declared struct Result payloads are
extracted with `?`, pushed into a caller-visible `List<Struct>` output, and
checked alongside `Map<Str,Str>` metadata snapshot round trips.
`examples/e309_vaisdb_artifact_store_snapshot.vais` verifies the persistable
artifact-store follow-up: `List<DocArtifact>` records are serialized to text,
written/read with file helpers, parsed back through declared-struct Result
helpers, and queried after reload.
`examples/e310_vaisdb_artifact_query_report.vais` verifies the persisted
artifact-store query/report follow-up: loaded `DocArtifact` records are ranked
with term scoring, rendered into `Result<Str,Int>` report payloads, persisted,
and checked for missing-store/empty-query error propagation.
`examples/e311_result_call_argument_flow.vais` verifies the direct
call-argument follow-up: `Result<Str,Int>` and declared-struct Result helpers
can be passed straight into recovery helpers without explicit temporary locals.
`examples/e312_result_struct_local_wrapper_flow.vais` verifies the self-host
wrapper local-copy follow-up: declared-struct Result wrapper payloads can be
copied through local struct variables and returned without losing fields.
`examples/e313_result_struct_str_match_flow.vais` verifies the string recovery
follow-up: declared-struct Result matches can select title/ID string fields
directly for report-style code while integer error codes are stringified.
`examples/e314_result_struct_str_concat_match_flow.vais` verifies the string
composition follow-up: declared-struct Result matches can build report labels
with nested `str_concat(...)` over payload fields while integer error codes are
stringified.
`examples/e315_result_struct_str_transform_match_flow.vais` verifies the string
normalization follow-up: declared-struct Result matches can transform payload
strings with `str_replace`, `str_trim`, `str_upper`, `str_lower`, and local-prefix
`str_concat(...)` while integer error codes are stringified.
`examples/e316_result_struct_str_transform_len_match_flow.vais` verifies the
string scoring follow-up: declared-struct Result matches can transform payload
strings and feed chained `.len()` results into integer score expressions while
integer error codes are preserved.
`examples/e317_result_struct_payload_helper_call_score.vais` verifies the
reusable-helper follow-up: declared-struct Result matches can pass `Ok`
payload structs to scoring helpers and preserve integer error codes.
`examples/e318_result_struct_payload_helper_call_arithmetic.vais` verifies the
helper-composition follow-up: declared-struct Result matches can combine
reusable `Ok` payload helper-call terms with ordinary payload fields in integer
score expressions.
`examples/e319_result_struct_payload_field_helper_call_arithmetic.vais` verifies
the field-helper follow-up: declared-struct Result matches can pass payload
string fields into reusable integer helpers and compose those helper-call terms
with ordinary payload fields.
`examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais`
verifies the numeric field-helper follow-up: declared-struct Result matches can
pass payload integer fields into reusable integer helpers and compose those
helper-call terms with string-field helper terms.
`examples/e321_result_struct_payload_bool_match_condition.vais` verifies the
Bool-return follow-up: declared-struct Result matches can return validation
conditions from payload helper terms and error-code comparisons.
`examples/e322_vaisdb_module_boundary/main.vais` verifies imported VaisDB
module boundaries that share declared structs, structured Result helpers,
List outputs, and Map-backed scoring helpers across files.
`examples/e323_cli_package` verifies package-directory entry resolution with
imports and CLI argv forwarding in direct/default workflow gates. The same
workflow now verifies installable package output by running the generated
`bin/e323_cli_package` binary with and without CLI args, checking the copied
manifest, and rejecting unsafe package names before they can become output
paths. `examples/e326_cli_binary_target` verifies optional `binary` metadata
with direct/default package output and file-entry parity gates.
`examples/e299_vaisdb_benchmark_report.vais` adds the first Vais-authored
benchmark/report fixture for that path: it calls `time_millis()` before and
after term counting/scoring, writes a text report with `fs_write_text`, reads it
back with `fs_read_text`, and validates the expected metrics in direct, full,
workflow, and parity gates.
`examples/e300_vaisdb_benchmark_cli_report.vais` adds the CLI-style follow-up:
it uses `fs_cwd`, `path_dirname`, and `path_basename` to locate the repo,
invokes `scripts/vaisc run examples/e295_vaisdb_indexer_prototype.vais`
through `proc_capture`, measures elapsed milliseconds for direct and default
engines, writes the combined report, and validates the saved status fields.
`tools/vaisdb_benchmark_report.vais` is the reusable tool form of that path:
it runs the same indexer benchmark, reads the raw report back, parses required
metrics from line-oriented text, computes the direct/default delta, and writes
a summary file.
`bash scripts/test-vaisdb-workflow.sh` is the focused reproducibility gate for
the current document workflow, including generated temp-file and argv-file
ingest modes plus the e301/e302 string-payload Result fixtures, the e303
structured Result payload fixture, the e304/e305/e306/e307/e308
declared-struct Result payload fixtures, the e300 CLI
report fixture, and reusable benchmark report tool, and
`bash scripts/bench-vaisdb-indexer.sh` records the local direct/default
compile+run timing protocol.
`proc_run(argv: List<Str>) -> Int` is covered by the same gate and returns the
child process exit code while inheriting stdio.
`proc_run_env(argv: List<Str>, env: List<Str>) -> Int` is also covered there
and applies child-only `KEY=value` environment overrides before exec.
`proc_capture_stdout(argv: List<Str>) -> Str` and
`proc_capture_stderr(argv: List<Str>) -> Str` are covered by the same gate and
return one captured child stream while inheriting the other.
`proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int`
is covered by the same gate and redirects stdout/stderr to explicit files while
returning the child exit code.
`proc_capture(argv: List<Str>) -> ProcessResult` is covered by the same
full-engine host gate and by the native direct feature gate when the source
declares the standard result shape:

```vais
struct ProcessResult {
    code: Int,
    stdout: Str,
    stderr: Str,
}
```

`examples/e202_proc_capture_result.vais` verifies reading all three fields from
that result.
`fs_remove(path: Str) -> Int` removes an existing file path and also succeeds
when the path is already missing; it is covered by the same host gate.

```vais
fn main() -> Int {
    let argv: List<Str> = []
    argv.push("/bin/sh")
    argv.push("-c")
    argv.push("exit 7")
    return proc_run(argv)
}
```

The host API contract is maintained in
[../design/HOST_IO.md](../design/HOST_IO.md). Except for those verified file,
path, and process intrinsics, these names should not be treated as
release-surface verified until their gates are added.

## Tuples

The verified tuple slice covers function return and immediate local
destructuring for `Int` tuples:

```vais
fn pair() -> (Int, Int) {
    return (3, 4)
}

fn main() -> Int {
    let (a, b) = pair()
    return a + b
}
```

This slice is covered by `examples/e59_tuple.vais`. The public driver lowers it
to generated struct storage before the self-host core receives the source.
Nested tuples, tuple parameters, tuple mutation, and non-`Int` tuple fields are
not release-surface claims yet.

## Closures

The verified closure slice is intentionally narrow:

```vais
fn adder(n: Int) -> fn(Int) -> Int {
    return |x| x + n
}

fn main() -> Int {
    let add3 = adder(3)
    return add3(4)
}
```

Returned single-`Int` closures can also be passed to a single-closure
higher-order helper that applies the closure to an `Int`, as covered by
`examples/e81_closure_return_apply.vais`.

Non-capturing inline closure literals can be passed directly to the same
single-closure `Int` higher-order helper shape, as covered by
`examples/e49_closure_arg.vais`.

Local closures can capture one `Int` local and be called inside the same
function, as covered by `examples/c5.vais`.

Captured inline closure arguments, multiple captured values, multiple closure
parameters, escaping local closure values, and broader higher-order function
patterns are not release-surface claims yet.

## Diagnostics

`scripts/vais-check` and `scripts/vaisc` front diagnostics catch common
non-Vais spellings and print source coordinates and `help:`.

```bash
scripts/vais-check examples/c4.vais
```

The public checker is built from `tools/vais_check_cli.vais` and
`tools/vais_check_core.vais`. Release gates check its fixture issue counts,
coordinate/help output shape, clean-file behavior, invalid static import path
diagnostics, invalid `main` signature diagnostics, missing helper return-type
diagnostics, unsupported generic `Option<T>`, `Result<T,E>`, and non-verified
`Map<K,V>` diagnostics, and packaged command path.

Common corrections:

| Do not write | Write |
| --- | --- |
| `a && b` | `a and b` |
| `a || b` | `a or b` |
| `!x` | `not x` |
| `x as Int` | `Int(x)` |
| `use math.add` | `import math.add` |
| `import math::add` | `import math.add` |
| `Color::Red` | `Color.Red` |
| `Vec<T>` | `List<T>` |
| `String` | `Str` |
| `fn main(argc: Int) -> Int { ... }` | `fn main() -> Int { ... }` |
| `fn add(a: Int, b: Int) { ... }` | `fn add(a: Int, b: Int) -> Int { ... }` |
| `x += 1` | `x = x + 1` |

## Build And Test

```bash
scripts/vaisc doctor
scripts/vaisc emit-ir examples/c4.vais -o /tmp/c4.ll
scripts/vaisc build examples/c4.vais -o /tmp/c4
scripts/vaisc run examples/c4.vais

bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc-install.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test-vaisc-host.sh
bash scripts/test.sh
```

The exact release subset is tracked in `tools/vaisc-parity.tsv`.


## Self-recursion

`@(args)` inside a function body calls the enclosing function itself. The
driver rewrites `@(` to the enclosing function's name before either engine
runs, so `@` works anywhere a named call works: tail returns
(`return @(n - 1)`), compound arithmetic (`n * @(n - 1)`), and nested call
arguments (`@(str_slice(s, 1, s.len() - 1), d + 1)`), as covered by
`examples/e343_self_recursion_at.vais`. Named self-calls remain equivalent.
