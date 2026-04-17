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

If one of these becomes buildable again after a compiler/stdlib change,
move it back to `examples/` and verify with
`cargo test -p vaisc --test examples_fresh_rebuild -- --ignored`.
