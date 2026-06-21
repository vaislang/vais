# Vais Map ABI And Generic Expansion

Status: design contract for future gates. The verified Map surface today is
local `Map<Int,Int>` with `{}`, assignment copy, `insert`,
`get(key, default)`, `get_opt(key)`, `contains`, `len`, and parameter
reference/mutation, plus local `Map<Int,Bool>` and `Map<Int,Char>` with `{}`,
assignment copy, `insert`, `get(key, default)`, `contains`, and `len`, in the
full self-host compiler path and native direct engine.

This document fixes the implementation contract required before `Map<K,V>` can
be broadened. It does not publish new verified syntax by itself.

## Goals

- Keep current local `Map<Int,Int>`, `Map<Int,Bool>`, and `Map<Int,Char>`
  behavior stable.
- Add Map parameters and return values without hidden aliasing.
- Broaden key and value types only as concrete, gate-backed instantiations.
- Keep direct-engine and full self-host lowering behavior aligned.
- Reject unsupported Map forms with P4 diagnostics until they pass gates.

## Current Verified Slice

The current slice is deliberately local:

```vais
fn main() -> Int {
    let scores: Map<Int,Int> = {}
    let flags: Map<Int,Bool> = {}
    let letters: Map<Int,Char> = {}
    scores.insert(4, 40)
    flags.insert(4, true)
    letters.insert(4, 'A')
    if flags.get(4, false) and letters.get(4, 'Z') == 'A' {
        return scores.get(4, 0) + scores.contains(4) + scores.len()
    }
    return 0
}
```

```vais
fn put(scores: Map<Int,Int>, key: Int, value: Int) -> Int {
    scores.insert(key, value)
    return scores.len()
}
```

Verified behavior:

- A Map local must be explicitly annotated as `Map<Int,Int>`, `Map<Int,Bool>`,
  or `Map<Int,Char>`.
- `{}` constructs an empty local map.
- `target = source` copies one local Map into another local with the same
  concrete Map type without aliasing.
- `insert(key, value)` inserts or replaces the key.
- `get(key, default)` returns the stored value or the default.
- `get_opt(key)` returns `Some(value)` or `None` for the local
  `Map<Int,Int>` and `Option<Int>` slice.
- `contains(key)` returns a `Bool`.
- `len()` returns the number of present keys.
- `Map<Int,Int>` parameters are passed by reference; a callee can mutate the
  caller-visible map.

Not verified yet: Map function returns, non-`Map<Int,Int>` Map parameters,
generic key/value pairs, entry literals, deletion, iteration, custom hashing,
and Map APIs that require broader `Option<T>` or `Result<T,E>` support.
`Map<Int,Bool>.get_opt` and `Map<Int,Char>.get_opt` are intentionally excluded
until their Option payload slices are verified.

## Ownership And Mutation Semantics

Future Map ABI gates must use these semantics:

- A local Map variable owns its storage.
- `insert` mutates the receiver Map.
- `a = b` copies the contents of `b` into `a`; it does not make `a` and `b`
  aliases.
- Passing a Map to a function passes a mutable collection reference, matching
  current collection-parameter behavior for `List`. If a function calls
  `insert` on its Map parameter, the caller-visible Map is mutated.
- Returning a Map copies the returned contents into caller-owned storage.
- `len`, `contains`, `get`, and `get_opt` do not mutate the Map.

These rules avoid accidental aliasing for assignment and return values while
preserving efficient mutation through function parameters.

## ABI Shape

The compiler should lower each verified concrete Map type through a monomorphic
runtime helper family. There is no unconstrained generic Map runtime in the next
slice.

For a concrete `Map<K,V>`:

- Local storage is a concrete runtime storage object owned by the variable.
- Parameter ABI is a pointer/reference to the concrete storage object.
- Return ABI uses caller-owned output storage, either as an explicit hidden
  out-parameter in LLVM lowering or an equivalent direct-engine strategy.
- Assignment uses a concrete copy helper.
- `insert`, `get`, `get_opt`, `contains`, and `len` call helpers specialized for
  the concrete key/value pair.

The current full self-host path uses a fixed-capacity integer buffer for local
`Map<Int,Int>`, `Map<Int,Bool>`, and `Map<Int,Char>`. Future gates may keep
fixed-capacity storage
for early concrete slices, but capacity and trap behavior must be documented and
tested before being advertised.

## Generic Expansion Order

Broaden Map support in this order:

1. `Map<Int,Int>` ABI: parameters first, then returns.
2. More `Map<Int,V>` local slices for already verified scalar values where `V`
   has a stable copy ABI; `Map<Int,Bool>` and `Map<Int,Char>` local values are
   the first completed slices in this step.
3. `Map<Int,V>` ABI: parameters and returns after the local concrete slices are
   stable.
4. `Map<Str,V>` only after string equality, hashing, copy, and lifetime rules
   are specified for Map keys.
5. Struct values only after struct copy and return ABI behavior are already
   gate-backed for the chosen Map storage.
6. Generic functions over `Map<K,V>` only after generic type checking can
   constrain key equality and storage helpers deterministically.

Do not publish a broad `Map<K,V>` claim until every accepted `K,V` pair has
front, full, direct, parity or value, and release-gate coverage.

## Method Contract

The next promoted method set should remain the current small API:

| Method | Future concrete signature |
| --- | --- |
| `m.insert(key, value)` | `Map<K,V>, K, V -> Unit` |
| `m.get(key, default)` | `Map<K,V>, K, V -> V` |
| `m.get_opt(key)` | `Map<K,V>, K -> Option<V>` |
| `m.contains(key)` | `Map<K,V>, K -> Bool` |
| `m.len()` | `Map<K,V> -> Int` |

`get_opt` can only be promoted for `V` once `Option<V>` is verified. Until then,
use `get(key, default)` for new value types.

Deletion, iteration, entry literals, capacity configuration, custom hashing,
and ordered maps are later APIs.

## Diagnostics

Until each slice is implemented, the public front must reject unsupported forms:

- Map assignment from anything other than another local with the same verified
  concrete Map type.
- Map function parameters beyond `Map<Int,Int>`.
- Map function returns.
- Generic key/value forms outside verified concrete pairs.
- Map literals with entries.
- Unsupported Map methods.

Diagnostics must include a concrete rewrite or a short explanation that only
local `Map<Int,Int>`, `Map<Int,Bool>`, and `Map<Int,Char>` values are verified
in the current slice.

## Required Gates

Each Map expansion must update or add:

- `scripts/test-vaisc-front.sh` coverage for accepted and rejected forms.
- `scripts/test-vaisc-direct.sh` coverage for direct-engine behavior when the
  direct engine claims that slice.
- `scripts/test-fixpoint-full.sh` coverage for the full self-host compiler.
- `scripts/test-vaisc-parity.sh` or `scripts/test.sh` value coverage for public
  examples.
- `scripts/test-vaisc-errors.sh` coverage for any new diagnostic surface.
- `std/PRELUDE.md`, `docs/reference/LANGUAGE.md`, `CHANGELOG.md`, and
  `ROADMAP.md` updates.
- Website copy only when the public site advertises the new verified behavior.
