# Removed Keywords

Vais has removed several experimental keywords during its stabilization
work. This page is the single source of truth for what was removed,
when, why, and how to migrate existing code. New code should not use
any of these keywords; the lexer no longer recognizes them.

## `lazy` / `force`

**Removed in**: commit `8c60c075` — `refactor(compiler): remove
lazy/force keywords across all layers (ROADMAP #16+#17)`

**Was**:

```vais
x := lazy 42          // build a thunk
y := force x          // evaluate it
```

**Rationale**: lazy evaluation added pervasive complexity to the lexer,
parser, type checker, effect system, and code generators (text IR,
inkwell, JS). Real-world usage in `vais-apps` was zero before removal.
The cost of maintaining the feature outweighed the benefit, so the
whole layer came out.

**Migration**:
- Plain values that were wrapped in `lazy` can be used directly.
- Deferred side effects should go through a closure
  (`() -> T` function) that the caller invokes when needed.

```vais
// Before
x := lazy expensive_compute(input)
y := force x

// After
x := |_| expensive_compute(input)   // closure
y := x(())
```

## `spawn`

**Removed in**: commit `12592076` — `refactor(compiler): remove spawn
keyword across all layers (ROADMAP #15)`

**Was**:

```vais
handle := spawn compute(x)       // fire-and-forget async
value  := (spawn compute(x)).await
```

**Rationale**: `spawn` introduced a distinct runtime concept — a
future that runs independently of the enclosing `A F` (async function)
— without a clear executor model. Every codegen target (text IR,
inkwell, JS) had to carry its own scheduler shim. The design was
always described as "experimental", and no production code had adopted
it; removing it simplified the async story to just `A F` + `.await`.

**Migration**:
- Every `spawn expr` that was immediately `.await`-ed can drop the
  `spawn`: `(spawn f(x)).await` → `f(x).await`.
- Fire-and-forget patterns must be reworked against a real executor
  (none ships today). Track such rewrites as Phase 197+ work if the
  executor story matures.

```vais
// Before
result := (spawn slow_compute(10)).await

// After
result := slow_compute(10).await
```

## How a future removal should be documented

If another keyword ever needs to come out:

1. Land the removal in one commit that touches **every** layer (lexer,
   AST, parser, type checker, effects, codegen text, codegen inkwell,
   codegen JS, LSP, macros). Include a ROADMAP entry linking to the
   removal commit.
2. Add a row to this file with the same **removed in / was /
   rationale / migration** shape. Migration examples are load-bearing
   — this file is the first place an AI or a human will look when a
   legacy example stops compiling.
3. Move any examples that still use the keyword into
   `examples/archive/` with a line in `examples/archive/README.md`
   tying them back to this document. Do **not** add them to
   `crates/vaisc/tests/examples_fresh_rebuild.rs::SKIP_LIST` — the
   archive directory is auto-excluded and carries its own contract.
