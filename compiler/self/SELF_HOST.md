# nl self-host compiler — status & architecture

The `compiler/self/*.nl` modules are the nl compiler **written in nl** (transpiled
to Vais → vaisc → native). They take an nl-like source string, parse it, and
either evaluate it (interpreter track) or emit real LLVM IR (codegen track).

This is the "fixpoint" effort: build a compiler in nl capable of the kind of
work a compiler does. The arithmetic/function subset is a **complete compiler**
(real tokenizer → recursive parser → control-flow-aware code generation).

## Two tracks

### Interpreter track — evaluates a program to a value
| Module | What it compiles | Verify |
|--------|------------------|--------|
| `lexer.nl` / `parser.nl` / `codegen.nl` | early lexer/parser/codegen fragments | value-correctness |
| `compiler.nl` (CX1–4) | `let`/`return`/arithmetic + `if/then/else`, single-letter vars | `test-compiler.sh` |
| `cx5_compiler.nl` (CX5–9) | functions, nested calls, **recursion**, 2-arg, locals, 26-slot vars | `test-cx5.sh` |
| `fixpoint.nl` (FP1) | List<Token> tokenizer + recursive eval (precedence, left-assoc) | `test-fixpoint.sh` |
| `fixpoint2.nl` (FP2/FP4) | **multi-char** variable names (List<Var> symbol table) | `test-fixpoint2.sh` |
| `fixpoint3.nl` (FP3/FP3b) | multi-char **functions** + nested calls + **recursion** (if bodies) | `test-fixpoint3.sh` |

### Codegen track — emits real LLVM IR that runs natively
| Module | What it generates | Verify |
|--------|-------------------|--------|
| `fixpoint_codegen.nl` (FP5) | arithmetic → `mul`/`add`/`sub` + SSA temps | `test-fixpoint-codegen.sh` |
| `fixpoint_codegen2.nl` (FP6) | + **variables** (SSA operand model, multi-char) | `test-fixpoint-codegen2.sh` |
| `fixpoint_codegen3.nl` (FP7) | + **functions** (`define @name` / `call`) | `test-fixpoint-codegen3.sh` |
| `fixpoint_codegen4.nl` (FP8) | + **recursion** via `icmp`/`br`/labeled blocks/`phi` | `test-fixpoint-codegen4.sh` |

Example of what `fixpoint_codegen4.nl` emits for
`fn factorial(n) {{ return if n < 2 then 1 else n * factorial(n - 1) }}; return factorial(5);`:

```llvm
define i64 @factorial(i64 %n) {
  %t1 = icmp slt i64 %n, 2
  br i1 %t1, label %then1, label %else1
then1:
  br label %merge1
else1:
  %t2 = sub i64 %n, 1
  %t3 = call i64 @factorial(i64 %t2)
  %t4 = mul i64 %n, %t3
  br label %merge1
merge1:
  %t5 = phi i64 [ 1, %then1 ], [ %t4, %else1 ]
  ret i64 %t5
}
define i64 @main() {
  %t1 = call i64 @factorial(i64 5)
  ret i64 %t1
}
```

That IR runs to 120.

## Key design notes

- **struct-not-Vec for recursion env** (CX5–9): Vais `Vec` is move-by-value and
  cannot be threaded through recursive functions by value (E022). The
  interpreter env (`Env`) and function table (`Defs`) are fixed-field **structs**,
  which copy cleanly through recursion. (For the List-based fixpoint compilers, a
  `&List<...>` borrow is threaded instead — see below.)
- **`&List` borrow recursion** (FP1+): the List-based pipeline threads
  `&List<Token>` / `&List<Var>` / `&List<Fn>` through recursive codegen. This
  required fixing a Vais `&Vec` borrow-codegen bug (compiler 214c97cf).
- **multi-char identifiers**: tokens carry the name as a source range
  `(nstart, nlen)`; the symbol table compares source bytes (`name_eq`), with a
  length check first so `foo`/`food` don't collide.
- **`{{`/`}}` brace escaping**: an embedded program string (code-as-data) uses
  `{{`/`}}` for literal braces; the transpiler emits Vais `\{`/`\}` (Vais
  interpolates `{ }` in every string literal).
- **emitting LLVM identifiers**: codegen prints `@name`/`%param` by copying the
  source bytes (`emit_name` via `putchar`). This exposed a Vais bug where a
  literal `%` in an interpolated string was consumed as a printf specifier
  (compiler e711dac1).

## Current full-source gate

As of 2026-06-09, the source-file harness can feed the actual
`compiler/self/fixpoint_full.nl` source back into `fixpoint_full.nl` and produce
a working first-generation compiler. The long gate is automated as
`scripts/test-fixpoint-full-self.sh`:

1. normalize/embed the full source with `tools/embed_self_source.py`
2. transpile with `compiler/transpiler/nl2vais.py`
3. build the generated Vais compiler with `vaisc`
4. run that compiler to emit LLVM IR
5. clang the emitted IR and run the resulting program
6. retarget that first-generation compiler to the real
   `compiler/self/fixpoint.nl`, `fixpoint2.nl`, and `fixpoint3.nl`, then
   clang/run each final IR

The latest run emitted a 980174-byte compiler IR module for the full
`fixpoint_full.nl` self probe with exactly one `@main`, zero negative GEPs,
passed clang, ran the generated compiler successfully, then clang'd and ran the
second-stage emitted IR with exit code 42. The same gate then retargeted the
first-generation compiler to the real `fixpoint.nl`, `fixpoint2.nl`, and
`fixpoint3.nl`, emitted 984794/990906/1002636-byte compiler IR modules,
verified the final IR contains `ret i64 24` / `ret i64 50` / `ret i64 120`,
clang'd each final IR, and ran the final binaries with exit code 24/50/120.

The FP12pp blocker was **struct-valued function parameters/returns**:
`fixpoint_full.nl` uses helpers such as `emit_op(o: Op)`,
`emit_binop(..., l: Op, r: Op)`, and `gen_factor(...) -> Op`. FP12qq supports
that shape with the same hidden out-param strategy already used for `-> List`:
struct-returning functions lower to `define void ... (regular args..., i64*
out)`, struct params are passed by pointer and copied into local aggregate slots,
and returns of struct locals, literals, and struct-returning calls all copy or
forward through the hidden out-param.

## Honest limits

- The compilers handle an **arithmetic + function + recursion subset** of nl
  (multi-digit ints, `+ - *`, `let`, `fn`, calls, `if/then/else`, `< > ==`).
- A **repeatable self-compilation fixpoint** still needs stage comparison or
  another stable oracle. The current milestone is stronger than a snippet probe:
  the full `fixpoint_full.nl` source builds a working compiler, and a
  first-generation compiler can consume the real `fixpoint.nl`, `fixpoint2.nl`,
  and `fixpoint3.nl` files again and produce/run final IR. Retargeting that
  first-generation compiler to the full `fixpoint_full.nl` source is the next
  scale/storage gate; the current stack-backed fixed List buffers need a heap,
  segmented, or stage-specific storage strategy before that should become a
  required gate.
- Upstream: two Vais compiler bugs were root-fixed to enable this work
  (`&Vec` borrow recursion, literal-`%` escaping), both gate-verified
  (`check-integrity.sh INTEGRITY OK`, zero regression).

## Codegen track — complete construct coverage (2026-06-06)

The code generators below collectively cover **every core construct the nl
compiler itself is written in**. Each emits real LLVM IR (verified by compiling
the emitted IR with clang and checking the runtime value).

| Module | Constructs codegen'd | Verify |
|--------|---------------------|--------|
| `fixpoint_codegen.nl` (FP5) | arithmetic (mul/add/sub, SSA temps) | `test-fixpoint-codegen.sh` |
| `fixpoint_codegen2.nl` (FP6) | + variables (SSA operands) | `test-fixpoint-codegen2.sh` |
| `fixpoint_codegen3.nl` (FP7) | + functions (`define`/`call`) | `test-fixpoint-codegen3.sh` |
| `fixpoint_codegen4.nl` (FP8) | + recursion (`icmp`/`br`/`phi`) | `test-fixpoint-codegen4.sh` |
| `fixpoint_imperative.nl` (FP10a-c) | mutable vars (alloca), `while`, `if/else` | `test-fixpoint-imperative.sh` |
| `fixpoint_array.nl` (FP10d) | fixed arrays (`alloca [N x i64]` + GEP) | `test-fixpoint-array.sh` |
| `fixpoint_full.nl` (FP10f–FP12qq) | **the unified compiler**: functions (**0-10 params**, recursion, nested calls, **bare call statements**) + mutable vars + while + if/else + arrays + Lists + structs + **strings** (`s[i]`/`s.len()`) + putchar + **comparison-as-value** + **`(...)` grouping** + **`>=`/`<=`** + **`and`/`or` as values** + **compound conditions** + **string parameters** + **List parameters** (read + **push/write-through** + **post-call length-sync**) + **List-of-structs** local + **as a by-pointer parameter** (`toks.push(Token{...})` whole-struct elements → `[64*nf+1 x i64]` contiguous buffer, `toks[i].field` = `buf[i*stride + field_index]`, length at `buf[64*nf]`) — runs the REAL self-host helpers `name_eq`/`kw3`, and **the complete self-host tokenize→parse/eval shape**: `fn tokenize(s: Str, out: List<Token>)` fills a token list, then a SEPARATE consumer `fn eval(toks: List<Token>)` dispatches on `toks[i].kind` (multiple consumers can share one token list); element type of a read-only `List<Struct>` param is inferred from its own annotation; **typed let bindings** (`let mut toks: List<Token> = []` — the let parser skips the `: Type` annotation to find the RHS, recognizes `[]` empty lists, and reads the element type from the annotation) + **boolean literals** `true`/`false` (the `let mut go = true; ... go = false` loop-flag pattern) — runs fixpoint.nl's real multi-digit digit-run tokenizer (nested `while go` accumulating `v = v*10 + (d-48)`) + **`else if` chains inside a loop** (the `if is_space {} else if is_digit {} else if c==43 {} ...` dispatch) — **compiles and runs fixpoint.nl's complete real tokenize function** (4 token kinds, helper calls, 6-way else-if chain, out-param `List<Token>`) + **`let t = toks[i]` binding a List-of-structs element to a local** (copies the struct's fields so `t.kind`/`t.value` work) — **compiles and runs fixpoint.nl's complete recursive evaluator** (eval_term/eval_expr/eval_expr_fold over `List<Token>`: `2 + 3 * 4` = 14 with precedence + left-fold) — and **the whole tokenize+eval compiler as one integrated program**: `run(src: Str)` tokenizes an expression string into a `List<Token>` then evaluates it (`2+3*4`→14, `10 - 2 * 3`→4, `7 + 8 + 9`→24), the smallest complete self-host compiler running end to end; **`!=` operator** + **`name_eq` source-byte comparison** + **a `List<Var>` symbol-table lookup** + **variable evaluation** (eval_factor resolves an identifier token to its value via `lookup`, with the `List<Var>` table threaded through the whole recursive eval chain) — runs fixpoint2.nl's arithmetic+variables tier: `x + y * 4` with `x=2, y=3` → 14 (variable operands + precedence) — and **the whole arithmetic+variables compiler as one integrated program**: `run_program(src)` tokenizes a multi-`let` program (keyword recognition), builds the symbol table from the bindings, and evaluates (`let x = 2; let y = x + 1; return x + y * 4;` → 14, with `y` referencing `x`) — the second self-host tier (a language with variables) running end to end; **functions + recursion** (fixpoint3.nl's tier: a `List<Fn>` function table via `find_fn`, `eval_call` binding args into a fresh per-call `List<Var>` scope, and recursion where a body calls itself — `factorial(5)` via a fresh scope per call → 120) — confirming fixpoint_full can codegen the architecture of all three self-host tiers — and **the whole function-language compiler as one integrated program**: `run_program(src)` tokenizes a function-language program, scans `fn` definitions into a `List<Fn>` table, and evaluates the top-level call (`fn double(x) { return x * 2 } return double(21);` → 42; 2-function table dispatch). **All three self-host tiers (arithmetic / arithmetic+variables / functions) now compile end to end** — including a **recursive function language with if-expressions**: `fn fac(n) { return if n <= 1 then 1 else n * fac(n - 1) } return fac(5);` → 120 (eval_value evaluates the if-expression; eval_call recurses with a fresh scope). **`-> List` direct return** (`fn build() -> List<T> { ...; return xs }` compiled via a hidden out-param: the caller allocates the buffer, the callee's `return xs` copies its local list into it — restoring fixpoint.nl's verbatim `fn tokenize(src) -> List<Token>` shape, no out-param rewrite). **`for v in lo..hi { body }`** (exclusive) / **`..=`** (inclusive) desugared to an induction-variable while-loop (tokenized for=34/in=35/..=36/..==37; `icmp slt`/`sle` against the bound; `v = v + 1` increment) — exclusive/inclusive bounds, body arithmetic, function-param bound, push into a List, if-in-body, and nested for where the inner range depends on the outer var all run. **fixpoint_full now codegens every general-nl construct.** **`print(...)` with `{ident}` interpolation** — the self-host *codegen* emission mechanism (`print("define i64 @main() {")` / `print("  ret i64 {value}")` / `print("%t{counter} = {op_s} ...")`): the transpiler rewrites print→puts, and a brace-bearing literal routes to a `printf` call against a `@.fmt<nstart>` format global (`{ident}` → `%d` for Int or `%s` for Str, `%` → `%%`, trailing `\n` to match puts). Disambiguates a **lone literal `{`** (the trailing brace of `define ... {`, kept as a plain `puts`) from a **valid `{ident}` interpolation** via Vais's lexer rule (`{` + identifier + `}`), shared by one `interp_end` helper across length/emit/arg-load; Str interpolation uses function metadata/local string lets to emit `%s` and `i8*` varargs. This **compiles and runs fixpoint.nl's complete tokenize→eval→emit-IR pipeline**: given `2 + 3 * 4`, the self-host compiler emits `define i64 @main() {\n  ret i64 14\n}` as real LLVM IR text on stdout — the full self-host arc (source → tokens → value → emitted IR), front end **and** codegen, compiled by fixpoint_full. FP12mm also supports a user-defined `fn main()` without emitting a duplicate synthetic wrapper, and the file harness now compiles the actual `compiler/self/fixpoint.nl` source file after normalization, then runs the generated compiler to emit and execute `ret i64 24` IR. FP12nn extends function metadata/calls/List length sync to 10 params for `fixpoint2.nl`'s real `word_is` helper and late List out-param calls, and adds the actual `compiler/self/fixpoint2.nl` source-file smoke, which emits and executes `ret i64 50` IR after normalization. FP12oo adds the actual `compiler/self/fixpoint3.nl` source-file smoke: normalization now handles multi-line struct field types, multi-line call semicolons, and nested string brace escapes; codegen supports the 8-field `Fn` table shape, Str-param retlist calls, correct `List[index].field` scalar assignments, and void `-> List` functions. The normalized real file emits and executes `ret i64 120` IR. FP12qq adds struct-valued function params/returns, List param alias return-copying, 10-arg `-> List` call support, fixed internal `Fn`/`StructDef` metadata field lookup, and `/` codegen; the actual `fixpoint_full.nl` full-source probe now builds a working first-generation compiler whose emitted program runs with exit 42. | `test-fixpoint-full.sh` |
| `fixpoint_struct.nl` (FP10e) | structs/records (Token/Op/Fn/Slot shape) | `test-fixpoint-struct.sh` |
| `fixpoint_list.nl` (FP10g) | dynamic `List` (push/len/index — List<Token>/List<Fn> shape) | `test-fixpoint-list.sh` |
| `fixpoint_str.nl` (FP12c) | **string literals + `s[i]` byte load + `s.len()`** (the source-tokenization primitive) | `test-fixpoint-str.sh` |

Example — `fixpoint_list.nl` compiling a build-then-consume loop (the exact
pattern the tokenizer/evaluator use):

```
let xs = list(); let mut i = 0; while i < 5 { xs.push(i * 10); i = i + 1 };
let mut s = 0; let mut j = 0; while j < xs.len { s = s + xs[j]; j = j + 1 };
return s;   // -> 100, via alloca [64 x i64] buffer + length counter + GEP
```

Example — `fixpoint_str.nl` compiling the source-tokenization primitive (a
string literal indexed byte by byte, plus its length):

```
let s = "ABC"; return s[1] + s.len();   // -> 69  ('B'=66 + len 3)
```

emits

```llvm
@.s0 = private constant [4 x i8] c"ABC\00"
define i64 @main() {
  %v0 = alloca i8*
  %g0 = getelementptr [4 x i8], [4 x i8]* @.s0, i64 0, i64 0
  store i8* %g0, i8** %v0
  %t1 = load i8*, i8** %v0
  %t2 = getelementptr i8, i8* %t1, i64 1   ; s[1]
  %t3 = load i8, i8* %t2
  %t4 = zext i8 %t3 to i64                  ; 'B' = 66
  %t5 = add i64 %t4, 3                      ; + s.len()
  ret i64 %t5
}
```

Combined with `while` + assignment, this scans a source string byte by byte —
the exact inner loop of `fixpoint.nl`'s own tokenizer (`while i < src.len() { c =
src[i]; ... }`). String handling is what lets the compiler read its *own source
text*, so this is the construct that crosses from "compiles a subset" toward
"could read nl source": a compiler that codegens string indexing + length can, in
principle, tokenize the very source it is written in.

### What "complete codegen coverage" means — and the honest remaining gap

These modules prove the nl language can express, and the nl compiler can codegen,
**each** construct the compiler is made of — including the string indexing +
length that source tokenization is built on. As of FP12d the per-construct code
generators are **unified into one compiler** (`fixpoint_full.nl`): a single
program now codegens functions + mutable vars + while + if/else + arrays + Lists
+ structs + strings + putchar together, both at top level and inside function
bodies. The unified compiler can codegen its *own tokenizer's shape* — a function
that scans a string byte by byte into a `List` (verified e2e: `fn tok() { let s =
"..."; let xs = list(); while i < s.len() { xs.push(s[i]); i = i + 1 }; ... }`).

The remaining gap to a *literal* repeatable self-compilation fixpoint is now a
stage-comparison problem, not a known codegen blocker: the actual
`fixpoint_full.nl` source builds a first-generation compiler, that compiler emits
LLVM IR for its embedded sample, and the emitted program runs. The automated long
gate also proves a first-generation compiler can consume a real file-sized
`fixpoint.nl` input again and produce/run final IR (`ret i64 24`). The next step
is to define a stable stage oracle, then compare generated compiler output across
repeated stages or retarget the generated compiler to progressively larger real
sources.
