# Vais Token Density Plan

Status: measured baseline landed 2026-07-29; Stage 1a (string and char
escapes) promoted 2026-07-30 — `\n`/`\t`/`\r`/`\"`/`\\` (strings), plus
`'\''`/`'\0'` (chars) decode on both engines, unknown escapes are loud front
errors, locked by examples/e374_string_escapes.vais, front accept/reject
fixtures, and direct feature shape 234. Stage 1b (interpolation) promoted
same day as explicit f-strings — `f"n={n}: {title}"` desugars in one shared
driver pass to `str_concat`/`Str(...)` chains, `Str(x)` became total
(identity on Str), locked by examples/e375_string_interpolation.vais and
direct feature shape 235. Implicit interpolation of plain `"..."` literals
was measured impossible for this corpus: 2,500+ existing literals embed
source-as-string fixtures with literal braces, so the `f` prefix is the
collision-free (and Python-familiar) spelling. Stage 1c (compound
assignment) promoted same day — `place op= expr` desugars to
`place = place op (expr)` for +=/-=/*=//=/%= over plain places (locals,
list indexes, struct fields; call-bearing left sides stay loud front
errors), locked by examples/e376_compound_assign.vais and direct feature
shape 236; this also closed a full-engine silent trap where `x += 2`
compiled and dropped the increment. Stage 1d verified same day — bare
predicate conditions (str_*/fs_* builtins and container predicates in
if/while/not/and/or, bare predicate locals) and range-for/for-each
accumulation idioms already ran on both engines and are now locked by
examples/e377_bool_predicate_conditions.vais and direct feature shape 237,
retiring the `== 1` idiom. STAGE 1 COMPLETE: every measured hotspot
(escapes, interpolation, compound assignment, predicate spelling, loop
idioms) is verified surface. Stage 2a promoted 2026-07-30 — curated Str/fs
receiver methods (has/starts/ends/trim/lower/upper/replace/slice and
read/write/exists/is_dir) lower to the existing builtins in the shared
pass with postfix-chain receivers spliced once, locked by
examples/e378_str_receiver_methods.vais and direct feature shape 238.
Stage 2b promoted same day — return-value collection spellings for the
`*_into` out-param family (`let xs = body.lines()`, `csv.split(sep)`,
`dir.files()`, `dir.dirs()`, plus `for x in recv.lines() {` heads) lower
to the existing out-param calls with injected temp lists, locked by
examples/e379_collect_methods.vais and direct feature shape 239.
Stage 2c promoted 2026-07-31 — expression bodies (implicit return on
bare-expression tails), `args()` argv reads with the `return match args()`
CLI-dispatch tail lowered to first-match if-chains, and `.join(sep)` over
List<Str> pipelines, locked by examples/e380_expr_body_argv_pipeline.vais
and direct feature shape 240. STAGE 2 COMPLETE. Stage 3a promoted
same day — enumerate for-head destructuring
(`for (i, x) in expr.enumerate() {`) lowers to an indexed range-for over
plain places, collect-method results, and filter-chain temps, locked by
examples/e382_enumerate_for_destructuring.vais and direct feature shape
241; this closes the last idiom from the Stage 2 sketch. Stage 3b promoted
same day — closure tuple parameters in enumerate-map pipeline statements
(`coll.enumerate().map(|(i, x)| expr)[.join(sep)]`, let and return forms)
lower to indexed accumulation loops, locked by
examples/e383_enum_map_pipeline.vais and direct feature shape 242; the
execution-verified e332 rewrite with the one-line pipeline renderer
measures 897 tokens (below TypeScript's 914). EVERY idiom from the
original Stage 2 design sketch is now verified surface; general tuple
types and `.take(k)` remain the only deliberately deferred candidates.
Next: adopt the idioms in new product code and re-measure the corpus. Goal: cut LLM development token cost by half against
performance-equivalent mainstream languages while keeping runtime behavior and
performance unchanged.

## Method

Three real corpus programs were translated into idiomatic Python, TypeScript,
and Rust, every translation was verified equivalent by running it (all nine
exit 42 like the originals), and all sources were tokenized with two BPE
encodings (`o200k_base`, `cl100k_base`; the encodings agree within 1%, so
ranking conclusions are tokenizer-robust):

- `examples/e332_vaisdb_topk_ranking_report.vais` (hand-written ranking sort)
- `examples/e336_list_struct_sort_by.vais` (built-in key sorts)
- `examples/e341_vaisgrep_package` (real CLI tool, `main.vais` + `grep/scan.vais`)

Comments were kept identical across languages; user-code identifiers were kept
identical across variants so measurements attribute to language surface, not
naming style.

## Baseline (o200k tokens, 2026-07-29)

| Program | Vais | Rust | TypeScript | Python |
| --- | --- | --- | --- | --- |
| e332 top-k ranking | 1,013 | 1,072 | 914 | 823 |
| e336 sort_by | 573 | 567 | 512 | 479 |
| vaisgrep CLI | 2,156 | 1,829 | 1,705 | 1,511 |
| Total | 3,742 | 3,468 | 3,131 | 2,813 |

Current Vais costs +33% vs Python, +20% vs TypeScript, +8% vs Rust for the
same programs. The e332 translations even had to implement prelude helpers
Vais gets for free (`doc_term_counts_into`, `doc_term_weighted_score`) and
still came out ahead, so the finding is conservative.

Token hotspots measured inside the 367 Vais lines:

| Hotspot | Count | Cause |
| --- | --- | --- |
| nested `str_concat` | 24 | no string interpolation in general expressions |
| `== 1` / `!= 0` / `== 0` | 31 | predicate builtins return Int 1/0 |
| manual `i = i + 1` + `while` | 13 + 11 | no `+=`; range `for` not yet idiomatic |
| `str_byte(10)` | 4 | no string escapes |

Keyword shape is not the problem: the `fn`/`let` surface shares BPE vocabulary
with Rust, so the penalty comes from the hotspots above, not from keywords.

## Improvement Ladder (measured on rewritten sketches)

| Version | Total | vs now | vs Python | Content |
| --- | --- | --- | --- | --- |
| Vais today | 3,742 | 1.00x | 1.33x | |
| Stage 1 | 3,389 | 0.91x | 1.20x | escapes, interpolation, Bool predicates, `+=`, range `for` |
| Stage 2 | 2,802 | 0.75x | 1.00x | + receiver methods, return-value collections, expression bodies, argv slice match |
| Python | 2,813 | 0.75x | 1.00x | reference |
| Rust | 3,468 | 0.93x | 1.23x | reference |

Stage 2 reaches Python parity overall (and beats it on the real tool:
vaisgrep 1,481 vs 1,511; code-only totals are 3% below Python). The sketches
are design fixtures, not verified surface; numbers may shift a few percent
during gate promotion.

## Dead Ends (measured, do not revisit)

| Idea | Measured effect | Verdict |
| --- | --- | --- |
| single-letter keywords (`fn`->`F`, `if`->`I`, ...) | exactly 0.0% on all three files | BPE already encodes `fn`/`if`/`return` as one token; diverging from mainstream syntax only raises model retry cost |
| `check x == y else N` gate DSL | +1.0~1.8% | new surface not worth the coverage cost |
| whitespace minification | not pursued | indent runs already merge into single BPE tokens; breaks formatter round-trip |

## Why Performance Stays Unchanged

Every Stage 1/2 item is a driver-lowering desugar or a prelude mapping onto
existing C-host builtins, following the established precedents (`sort_by`
desugar shared by both engines, `@(` self-recursion rewrite, driver
pre-passes). Lowered IR patterns are identical to what the verbose forms emit
today, so runtime behavior and performance are structurally unchanged;
`scripts/vaisbench-gate.sh` enforces the baseline.

## Roadmap

Promotion follows the standing rule: full, direct, front, parity,
documentation, and example coverage together.

1. Stage 1a - string escapes (`\n`, `\t`, `\r`; `\"` and `\\` already work).
   Also removes a silent trap: today `"a\nb"` silently decodes to `anb`
   (backslash dropped, no diagnostic, no newline). Unknown escapes become a
   loud front error; the corpus sweep found only two real `\n` uses (e300
   family report headers, `str_contains`-checked, safe under new semantics).
2. Stage 1b - general string interpolation `"{expr}"` desugared to
   `str_concat` chains (the `print`/printf path already interpolates plain
   `{ident}`; this generalizes the surface to all string expressions).
3. Stage 1c - `+=` statement desugar; supersedes the "Compound assignment is
   not Vais syntax" line in `docs/reference/LANGUAGE.md`.
4. Stage 1d - Bool-returning predicate surface for host builtins; range `for`
   promoted through remaining verified positions.
5. Stage 2 - prelude v2: receiver-method spellings mapped to existing
   builtins, `*_into` out-params gaining return-value spellings (lowered to
   the same out-param calls, 4095-slot contract intact), expression bodies,
   argv slice match.
6. Stage 3 - LLM spec card (3-5k tokens) as the model-facing language
   reference; `LANGUAGE.md` (28.8k tokens) and `std/PRELUDE.md` (14.2k)
   remain the gate-backed canon. Session fixed cost today is ~43k tokens.
7. Stage 4 - domain prelude absorbed from product dogfooding (VaisDB, CLI
   harness patterns), demand-driven.

## Target Arithmetic

Source density alone cannot halve token cost against Python (identifiers,
literals, and structure are the information floor; Stage 2 parity is the
practical ceiling). The half target is the product of:

- source density: 0.75x vs today (measured), 0.81x vs Rust (measured);
- domain prelude on product code: estimated additional 20-30%, to be measured
  in the next dogfooding cycle;
- session layer: spec card cuts the ~43k fixed reference cost to 3-5k, and a
  mainstream-adjacent surface lowers retry loops.

Against the performance-equivalent baseline (Rust-class native), the combined
path lands in the 0.5-0.6x band. Against Python the honest offer is token
parity plus 10-100x runtime performance, not half tokens.

## Re-measurement With The Landed Surface (2026-07-30)

After Stages 1a-1d and 2a-2b landed, the two heaviest benchmark programs
were rewritten in the promoted surface only and verified by execution
(package self-test / example both exit 42 on both engines) before
re-tokenizing (o200k):

| Program | Vais before | Vais landed | Python | TypeScript | Rust |
| --- | --- | --- | --- | --- | --- |
| vaisgrep (main+scan) | 2,156 | 1,736 | 1,511 | 1,705 | 1,829 |
| e332 top-k ranking | 1,013 | 902 | 823 | 914 | 1,072 |
| Total | 3,169 | 2,638 (0.83x) | 2,334 | 2,619 | 2,901 |

Verified-execution result: the landed surface recovers 17% on real
programs, reaching TypeScript parity (2,638 vs 2,619) and 9% below Rust —
the performance-equivalent baseline — with Python 12% ahead. Unlike the
earlier sketches, these numbers carry no design risk — every construct in
the rewrite is gate-locked surface.

With Stage 2c (expression bodies, `return match args()` dispatch,
`.join(sep)`) the vaisgrep rewrite drops further: 2,156 -> 1,670 (-23%),
now below TypeScript (1,705) and 9% under Rust, with Python (1,511) 11%
ahead — the remaining gap is stdlib expressivity (enumerate-style
pipelines, comprehensions), tracked as Stage 3 candidates.

## Fleet Migration (2026-08-01)

The eleven landed CLI tools plus the vaisdb report/dispatch paths were
migrated in place to the promoted surface — behavior-identical (every
package self-test stays 42/42 on both engines, the vaisdb workflow and
vaisfmt hygiene gates stay green) and measured with comments included,
so these are conservative whole-file numbers, not code-only:

| Tool | before | after | delta |
| --- | --- | --- | --- |
| vaisgrep | 2,156 | 1,747 | -19.0% |
| vaismake | 2,821 | 2,474 | -12.3% |
| vaisfmt | 1,855 | 1,600 | -13.7% |
| vaisbench | 1,479 | 1,225 | -17.2% |
| vaisdiff | 1,687 | 1,421 | -15.8% |
| vaiswc | 1,272 | 997 | -21.6% |
| vaisbox | 1,089 | 855 | -21.5% |
| vaissort | 1,616 | 1,409 | -12.8% |
| vaisenv | 566 | 497 | -12.2% |
| vaistee | 962 | 892 | -7.3% |
| vaiscut | 1,147 | 1,000 | -12.8% |
| vaisdb (report+main) | 6,692 | 6,430 | -3.9% |
| Total | 23,342 | 20,547 | **-12.0%** |

The vaisdb posting-index internals (`vaisdb/index.vais`) stay in the
explicit byte-level style on purpose — the sharded posting machinery is
correctness-critical and its loops are not idiom-shaped. Migration also
field-tested the new diagnostics: the literal-in-receiver front error
fired twice (bind-first fixes), the direct bare-builtin-statement bound
surfaced once, and the full-engine eager `or` divergence was caught by a
self-test trap — and then root-fixed: `and`/`or` now short-circuit on
both engines (`examples/e387_short_circuit_logic.vais`), so guarded
index spellings like `j == 0 or xs[j - 1] > 0` are part of the surface.

## Macro Data Points

- `compiler/self/fixpoint_full.vais` = 247.6k o200k tokens (23.1k lines,
  ~10.7 tokens/line): the whole self-host compiler fits one 1M context.
- Reference docs: `LANGUAGE.md` 28.8k, `std/PRELUDE.md` 14.2k, `CLAUDE.md` 0.3k.
