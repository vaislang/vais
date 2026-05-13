# examples/archive/

Historical examples that reference removed or conceptual features. Not
compiled by `examples_fresh_rebuild`. Kept for reference; each file
documents its own status in a header comment.

| File | Status | Blocked on |
|---|---|---|
| `lazy_simple.vais` | Uses removed `lazy`/`force` keywords (commit 8c60c075, ROADMAP #16+#17) | Keyword removal policy |
| `lazy_test.vais` | Same as above | " |
| `lazy_func_test.vais` | Same as above | " |
| `tcp_10k_bench.vais` | Uses stdlib byte-ops (`store_i8`, `store_i16`, `store_i32`, `load_i32`) that were never implemented. File header labels itself "simplified" and says production would use `AsyncTcpListener` | AsyncTcpListener redesign |
| `simd_test.vais` | SIMD intrinsics produce LLVM IR that fails the verifier with "Aggregate extract index out of range". SIMD builtins need a contract/type-checker rework | SIMD feature redesign (Phase 197+) |
| `simd_distance.vais` | Same as simd_test | " |
| `async_reactor_test.vais` | Example misuses `LW` (while) as if it were `I` (if) and overloads `!` as an else clause rather than the Option/Result unwrap it actually is. Not a compiler bug — needs an author rewrite against the real async-reactor API | Rewrite against current API |
| `wasm_todo_app.vais` | Global fixed-size arrays (`G todo_ids: [i64; 100] = [0; 100]`) now type-check after Phase 196 P196-C1, but inkwell's identifier load path pulls the whole array value instead of emitting `getelementptr` for `todo_ids[idx]`. Archive until the inkwell array-access path grows a GEP variant for array-typed globals | Inkwell array GEP codegen |

If one of these becomes buildable again after a compiler/stdlib change,
move it back to `examples/` and verify with
`cargo test -p vaisc --test examples_fresh_rebuild -- --ignored`.
