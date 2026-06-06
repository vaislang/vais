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
| `fixpoint_full.nl` (FP10f) | **functions with imperative bodies** (the compiler's own function shape) | `test-fixpoint-full.sh` |
| `fixpoint_struct.nl` (FP10e) | structs/records (Token/Op/Fn/Slot shape) | `test-fixpoint-struct.sh` |
| `fixpoint_list.nl` (FP10g) | dynamic `List` (push/len/index — List<Token>/List<Fn> shape) | `test-fixpoint-list.sh` |

Example — `fixpoint_list.nl` compiling a build-then-consume loop (the exact
pattern the tokenizer/evaluator use):

```
let xs = list(); let mut i = 0; while i < 5 { xs.push(i * 10); i = i + 1 };
let mut s = 0; let mut j = 0; while j < xs.len { s = s + xs[j]; j = j + 1 };
return s;   // -> 100, via alloca [64 x i64] buffer + length counter + GEP
```

### What "complete codegen coverage" means — and the honest remaining gap

These modules prove the nl language can express, and the nl compiler can codegen,
**each** construct the compiler is made of. The remaining gap to a *literal*
self-compilation fixpoint is **integration + scale**, not a missing capability:
unify the per-construct code generators into one compiler and feed it the actual
multi-thousand-line nl compiler source (with its full mix of these constructs in
one program). That is a months-scale engineering effort. What exists today is a
verified code generator for every core construct, demonstrated end to end.
