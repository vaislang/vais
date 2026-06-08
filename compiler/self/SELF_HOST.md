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

## Honest limits

- The compilers handle an **arithmetic + function + recursion subset** of nl
  (multi-digit ints, `+ - *`, `let`, `fn`, calls, `if/then/else`, `< > ==`).
- A **true self-compilation fixpoint** (nl compiling its *own full compiler
  source*) requires implementing the entire nl grammar — structs, `while`,
  `List` + methods, `&` borrows, string interpolation — and a code generator for
  all of it. That is a months-scale effort (thousands of lines), not reachable in
  incremental steps. What exists today is a genuine, verified compiler for the
  subset above, demonstrating every core compiler capability end to end.
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
| `fixpoint_full.nl` (FP10f–FP12ll) | **the unified compiler**: functions (**0-8 params**, recursion, nested calls, **bare call statements**) + mutable vars + while + if/else + arrays + Lists + structs + **strings** (`s[i]`/`s.len()`) + putchar + **comparison-as-value** + **`(...)` grouping** + **`>=`/`<=`** + **`and`/`or` as values** + **compound conditions** + **string parameters** + **List parameters** (read + **push/write-through** + **post-call length-sync**) + **List-of-structs** local + **as a by-pointer parameter** (`toks.push(Token{...})` whole-struct elements → `[64*nf+1 x i64]` contiguous buffer, `toks[i].field` = `buf[i*stride + field_index]`, length at `buf[64*nf]`) — runs the REAL self-host helpers `name_eq`/`kw3`, and **the complete self-host tokenize→parse/eval shape**: `fn tokenize(s: Str, out: List<Token>)` fills a token list, then a SEPARATE consumer `fn eval(toks: List<Token>)` dispatches on `toks[i].kind` (multiple consumers can share one token list); element type of a read-only `List<Struct>` param is inferred from its own annotation; **typed let bindings** (`let mut toks: List<Token> = []` — the let parser skips the `: Type` annotation to find the RHS, recognizes `[]` empty lists, and reads the element type from the annotation) + **boolean literals** `true`/`false` (the `let mut go = true; ... go = false` loop-flag pattern) — runs fixpoint.nl's real multi-digit digit-run tokenizer (nested `while go` accumulating `v = v*10 + (d-48)`) + **`else if` chains inside a loop** (the `if is_space {} else if is_digit {} else if c==43 {} ...` dispatch) — **compiles and runs fixpoint.nl's complete real tokenize function** (4 token kinds, helper calls, 6-way else-if chain, out-param `List<Token>`) + **`let t = toks[i]` binding a List-of-structs element to a local** (copies the struct's fields so `t.kind`/`t.value` work) — **compiles and runs fixpoint.nl's complete recursive evaluator** (eval_term/eval_expr/eval_expr_fold over `List<Token>`: `2 + 3 * 4` = 14 with precedence + left-fold) — and **the whole tokenize+eval compiler as one integrated program**: `run(src: Str)` tokenizes an expression string into a `List<Token>` then evaluates it (`2+3*4`→14, `10 - 2 * 3`→4, `7 + 8 + 9`→24), the smallest complete self-host compiler running end to end; **`!=` operator** + **`name_eq` source-byte comparison** + **a `List<Var>` symbol-table lookup** + **variable evaluation** (eval_factor resolves an identifier token to its value via `lookup`, with the `List<Var>` table threaded through the whole recursive eval chain) — runs fixpoint2.nl's arithmetic+variables tier: `x + y * 4` with `x=2, y=3` → 14 (variable operands + precedence) — and **the whole arithmetic+variables compiler as one integrated program**: `run_program(src)` tokenizes a multi-`let` program (keyword recognition), builds the symbol table from the bindings, and evaluates (`let x = 2; let y = x + 1; return x + y * 4;` → 14, with `y` referencing `x`) — the second self-host tier (a language with variables) running end to end; **functions + recursion** (fixpoint3.nl's tier: a `List<Fn>` function table via `find_fn`, `eval_call` binding args into a fresh per-call `List<Var>` scope, and recursion where a body calls itself — `factorial(5)` via a fresh scope per call → 120) — confirming fixpoint_full can codegen the architecture of all three self-host tiers — and **the whole function-language compiler as one integrated program**: `run_program(src)` tokenizes a function-language program, scans `fn` definitions into a `List<Fn>` table, and evaluates the top-level call (`fn double(x) { return x * 2 } return double(21);` → 42; 2-function table dispatch). **All three self-host tiers (arithmetic / arithmetic+variables / functions) now compile end to end** — including a **recursive function language with if-expressions**: `fn fac(n) { return if n <= 1 then 1 else n * fac(n - 1) } return fac(5);` → 120 (eval_value evaluates the if-expression; eval_call recurses with a fresh scope). **`-> List` direct return** (`fn build() -> List<T> { ...; return xs }` compiled via a hidden out-param: the caller allocates the buffer, the callee's `return xs` copies its local list into it — restoring fixpoint.nl's verbatim `fn tokenize(src) -> List<Token>` shape, no out-param rewrite). **`for v in lo..hi { body }`** (exclusive) / **`..=`** (inclusive) desugared to an induction-variable while-loop (tokenized for=34/in=35/..=36/..==37; `icmp slt`/`sle` against the bound; `v = v + 1` increment) — exclusive/inclusive bounds, body arithmetic, function-param bound, push into a List, if-in-body, and nested for where the inner range depends on the outer var all run. **fixpoint_full now codegens every general-nl construct.** **`print(...)` with `{ident}` interpolation** — the self-host *codegen* emission mechanism (`print("define i64 @main() {")` / `print("  ret i64 {value}")` / `print("%t{counter} = {op_s} ...")`): the transpiler rewrites print→puts, and a brace-bearing literal routes to a `printf` call against a `@.fmt<nstart>` format global (`{ident}` → `%d` for Int or `%s` for Str, `%` → `%%`, trailing `\n` to match puts). Disambiguates a **lone literal `{`** (the trailing brace of `define ... {`, kept as a plain `puts`) from a **valid `{ident}` interpolation** via Vais's lexer rule (`{` + identifier + `}`), shared by one `interp_end` helper across length/emit/arg-load; Str interpolation uses function metadata/local string lets to emit `%s` and `i8*` varargs. This **compiles and runs fixpoint.nl's complete tokenize→eval→emit-IR pipeline**: given `2 + 3 * 4`, the self-host compiler emits `define i64 @main() {\n  ret i64 14\n}` as real LLVM IR text on stdout — the full self-host arc (source → tokens → value → emitted IR), front end **and** codegen, compiled by fixpoint_full. | `test-fixpoint-full.sh` |
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

The remaining gap to a *literal* self-compilation fixpoint is **scale**, not a
missing capability: feed `fixpoint_full.nl` the actual multi-thousand-line nl
compiler source (with its full mix of these constructs in one program) and have
the emitted IR reproduce it. That is a months-scale engineering effort — the
compiler source uses these same constructs but in far greater volume and with
deeper nesting than the e2e probes. What exists today is a verified, unified code
generator for every core construct, demonstrated end to end, that contains the
tokenizer's exact shape.
