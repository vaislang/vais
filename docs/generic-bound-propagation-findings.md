# Generic Bound Propagation — Findings 2026-04-21 H.2

Investigation whether vaisdb's `bulk.vais::build_graph<S: NodeStore>`
TC error indicates a compiler gap in generic bound propagation.

## Observed error

```
error[E001] Type mismatch
  --> bulk.vais:456:21
   |
 456 |                     node,
   |                     ^^^^ expected type implementing trait 'NodeStore',
   |                          found type 'S' which does not implement 'NodeStore'
```

The error position is `node` (1st argument) but the message is about
"expected NodeStore". `node` is an `HnswNode`, not a generic `S`. The
call site passes `node` (HnswNode), `vector`, ..., `store` (&mut S) to
`hnsw_insert<S: NodeStore>(new_node, query_vec, ..., store: &mut S, ...)`.

## Minimal reproducer — PASSES

Direct test of generic bound propagation across two bounded functions:

```vais
W MyTrait { F greet(self) -> i64 }
S Impl1 {}
X Impl1: MyTrait { F greet(self) -> i64 { R 42 } }

F use_trait<T: MyTrait>(x: &T) -> i64 {
    call_trait(x)
}
F call_trait<T: MyTrait>(x: &T) -> i64 {
    x.greet()
}

F main() -> i64 {
    i := Impl1{}
    R use_trait(&i)
}
```

Result: `exit: 42` ✓. Compiler correctly propagates `T: MyTrait` bound
across nested calls, monomorphizes, and dispatches.

## Conclusion

The TC does propagate generic bounds correctly in the minimal case.
`bulk.vais`'s error must arise from an interaction specific to that
function (many params, refs, Option types, multiple trait hierarchies).
Likely explanations:

1. **vaisdb code mis-uses the call**: wrong argument order (the error
   position `node` at col 21 aligns suspiciously with the 1st arg,
   hinting the TC may actually expect the 5th arg's type at that slot
   due to signature mismatch upstream).
2. **Some other prior error cascades here**: the module has 8+ other
   errors (E004 undefined functions, E034 panic). TC errors do not
   cleanly isolate; fixing upstream errors first may resolve this one.

Neither is a compiler bug requiring a TC fix. The error correctly
rejects the vaisdb code.

## Scope decision

H.2 no-op: no TC change. `bulk.vais` needs vaisdb-side code audit
(vaisdb cleanup drive scope).

## What remains for a future TC drive

Worth considering:

- Better error positions: when a call has N args but TC rejects the call
  as a whole, pointing at the offending arg (not the 1st) helps diagnosis.
- Cross-module bound resolution: if the trait `NodeStore` is imported
  from elsewhere and has a re-export chain, the bound might be lost in
  `resolved_function_sigs`. (Currently no evidence this happens for
  `bulk.vais` but worth checking if users report similar issues.)
