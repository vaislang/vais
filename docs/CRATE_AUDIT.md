# Vais Compiler Crate Audit (Phase 0.A)

> Audit date: 2026-04-25 (Phase 0 kickoff)
> Goal: classify all 28+ crates into tiers — **core** (must work for v1.0),
> **auxiliary** (best-effort, not v1.0-blocking), **experimental** (may
> not even compile).

## Summary

| Tier | Count | Crates |
|------|-------|--------|
| **Core** | 7 | vais-lexer, vais-parser, vais-ast, vais-types, vais-codegen, vais-mir, vaisc |
| **Auxiliary** | 6 | vais-lsp, vais-dap, vais-i18n, vais-plugin, vais-macro, vais-bindgen |
| **Experimental** | 16 | vais-jit, vais-gpu, vais-gc, vais-codegen-js, vais-hotreload, vais-dynload, vais-profiler, vais-registry-server, vais-playground-server, vais-tutorial, vais-python, vais-node, vais-query, vais-testgen, vais-supply-chain, vais-security |

## Methodology

For each crate, the audit asked:
1. Is it on the **path from `.vais` source → executable LLVM IR**? → core
2. Is it consumed by `vaisc` driver but optional/diagnostic? → auxiliary
3. Is it a side feature (extra backend, extra tooling, extra runtime)? → experimental

## Core (production-track)

These produce LLVM IR. v1.0 requires all green.

| Crate | LOC | Role |
|-------|----:|------|
| `vais-lexer` | 2,039 | tokens |
| `vais-parser` | 8,367 | AST construction |
| `vais-ast` | 3,375 | AST types |
| `vais-types` | 27,769 | type checker + ResolvedType |
| `vais-codegen` | 69,264 | LLVM IR emission (largest crate) |
| `vais-mir` | 10,141 | optional IR (in codegen path) |
| `vaisc` | 27,706 | driver / CLI |
| **Total** | **148,661** | |

**Trust target**: `tests/lang/` ≥ 95% green, `examples/hello_world_v2/` 100%.

## Auxiliary (best-effort)

| Crate | Reason |
|-------|--------|
| `vais-lsp` | IDE integration; broken LSP doesn't break compilation |
| `vais-dap` | debugger; same |
| `vais-i18n` | error message localization; English fallback works |
| `vais-plugin` | plugin host; current plugins are external |
| `vais-macro` | macro system; basic features only used; no macros in `tests/lang/` |
| `vais-bindgen` | C/Rust bindings generator; not exercised by hello world |

**v1.0 stance**: must `cargo build`. Tests not required green.

## Experimental (may not compile)

These are at risk of bit-rot during Phase 0:

| Crate | Status |
|-------|--------|
| `vais-jit` | optional in `vaisc` Cargo.toml, marked experimental |
| `vais-gpu` | GPU codegen, niche use case |
| `vais-gc` | tracing GC; current vais uses ARC/manual |
| `vais-codegen-js` | alternative JS/WASM backend |
| `vais-hotreload` | live reload during dev |
| `vais-dynload` | dynamic loading |
| `vais-profiler` | runtime profiling |
| `vais-registry-server` | package registry server |
| `vais-playground-server` | online playground server |
| `vais-tutorial` | interactive tutorial |
| `vais-python` | Python bindings |
| `vais-node` | Node.js bindings |
| `vais-query` | runtime query (LINQ-like?) |
| `vais-testgen` | property test generator |
| `vais-supply-chain` | supply-chain verification |
| `vais-security` | additional security passes |

**v1.0 stance**: documented as experimental in README, may not be in CI.
Post-v1.0 each can be promoted to auxiliary or core after individual review.

## Recommended actions

### Short-term (this week)

1. **Add `[features]` gates** to `vaisc/Cargo.toml`:
   - `default = ["core"]` — only core + auxiliary deps
   - `experimental = [...]` — opt-in for experimental crates
2. **Workspace-level CI**: only build `cargo build --workspace --exclude vais-jit --exclude vais-gpu --exclude vais-gc --exclude vais-codegen-js --exclude vais-hotreload --exclude vais-dynload --exclude vais-profiler --exclude vais-registry-server --exclude vais-playground-server --exclude vais-tutorial --exclude vais-python --exclude vais-node --exclude vais-query --exclude vais-testgen --exclude vais-supply-chain --exclude vais-security` — or equivalent feature gate.
3. **Document tier 1 in README** — link to this file.

### Medium-term (Phase 0.A → 0.B → 0.C)

1. Promote auxiliary crates only if they have ≥ 30 own tests passing.
2. Demote experimental crates that haven't compiled in 30 days to a separate `crates/experimental/` subdirectory.

### Stdlib audit (parallel)

`compiler/std/` has 41,772 lines across many domains:

**Core stdlib** (used by `tests/lang/` + `examples/hello_world_v2/`):
- `option.vais`, `result.vais`, `vec.vais`, `hashmap.vais`,
- `string.vais`, `bytes.vais`, `bytebuffer.vais`,
- `file.vais`, `net.vais`, `sync.vais`,
- `panic.vais`, `core.vais` (if exists)

**Experimental stdlib** (defer to v2.0+):
- websocket, yaml, http_server, oauth2, opencl,
- async_iocp, async_kqueue, async_epoll,
- web, wasm, gpu_compute, ML/RL helpers, etc.

Recommendation: move experimental modules to `compiler/std-experimental/`
in a future PR. Until then, `tests/lang/` carefully avoids importing them.

## Open questions

1. **vais-mir** — is MIR actually used in the codegen path, or is it
   future work? Currently has 10k LOC. If unused, demote to experimental.
2. **vais-types 27k LOC** — type checker is huge. Could be split into
   smaller subcrates (parser-frontend, inference, monomorphization)
   for clarity.
3. **vais-codegen 69k LOC** — same. Could split into IR-emit (24k) +
   ABI/specialization (45k). Phase 0 doesn't require this; mention for
   v1.1 cleanup.

## Honesty acknowledgement

This audit is based on dependency graph + grep. Each "experimental"
crate may have hidden dependents. Run `cargo tree` to verify before
removing any from the workspace.
