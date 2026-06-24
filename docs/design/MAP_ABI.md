# Vais Map ABI And Generic Expansion

Status: design contract for future gates. The verified Map surface today is
local `Map<Int,Int>` with `{}`, assignment copy, `insert`,
`remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, and parameter
reference/mutation, parameter assignment copy, return-call assignment copy, and return-value local initialization, plus
`Map<Int,Bool>` and `Map<Int,Char>` with local
values, assignment copy, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`,
`contains`, `len`,
parameter reference/mutation, parameter assignment copy, return-call assignment copy, and return-value local initialization.
`Map<Str,Int>` values support `{}`, assignment copy, `insert`, `remove`,
`clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, function
parameter reference/mutation, parameter assignment copy, return-call assignment copy, and return-value local initialization.
`Map<Str,Bool>` values support local `{}`, assignment copy, `insert`, `remove`,
`clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, and function
parameter reference/mutation, parameter assignment copy, return-call assignment copy, and return-value local initialization.
`Map<Str,Char>` values support local `{}`, assignment copy, `insert`,
`remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, `len`, and
function parameter reference/mutation, parameter assignment copy, return-call assignment copy, and return-value local initialization.
These slices are verified in the full self-host compiler path and native direct
engine.

This document fixes the implementation contract required before `Map<K,V>` can
be broadened. It does not publish new verified syntax by itself.

## Goals

- Keep current local `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`,
  `Map<Str,Int>`, `Map<Str,Bool>`, and `Map<Str,Char>` behavior stable.
- Add Map parameters and return values without hidden aliasing.
- Broaden key and value types only as concrete, gate-backed instantiations.
- Keep direct-engine and full self-host lowering behavior aligned.
- Reject unsupported Map forms with P4 diagnostics until they pass gates.

## Current Verified Slice

The current slice is deliberately concrete:

```vais
fn main() -> Int {
    let scores: Map<Int,Int> = {}
    let flags: Map<Int,Bool> = {}
    let letters: Map<Int,Char> = {}
    let names: Map<Str,Int> = {}
    let flags_by_name: Map<Str,Bool> = {}
    let letters_by_name: Map<Str,Char> = {}
    scores.insert(4, 40)
    scores.insert(9, 2)
    scores.remove(9)
    scores.clear()
    scores.insert(4, 40)
    flags.insert(4, true)
    letters.insert(4, 'A')
    names.insert("red", 1)
    names.insert("blue", 2)
    flags_by_name.insert("red", true)
    letters_by_name.insert("red", 'A')
    let yes_value = match flags.get_opt(4) { Some(v) => v, None => 0 }
    let letter_value = match letters.get_opt(4) { Some(v) => v, None => 58 }
    if flags.get(4, false) and letters.get(4, 'Z') == 'A' {
        return scores.get(4, 0) + scores.len() + scores.contains(9) + yes_value * 20 + letter_value + names.get("red", 0) + names.contains("blue") - 86
    }
    return 0
}
```

```vais
fn put(scores: Map<Int,Int>, key: Int, value: Int) -> Int {
    scores.insert(key, value)
    return scores.len()
}

fn mark(flags: Map<Int,Bool>, key: Int) -> Int {
    flags.insert(key, true)
    if flags.get(key, false) and flags.len() == 1 {
        return 40
    }
    return 0
}

fn stamp(letters: Map<Int,Char>, key: Int) -> Int {
    letters.insert(key, 'A')
    if letters.get(key, 'Z') == 'A' and letters.len() == 1 {
        return 40
    }
    return 0
}

fn put_name(scores: Map<Str,Int>, key: Str, value: Int) -> Int {
    scores.insert(key, value)
    scores.insert("blue", 2)
    scores.remove("blue")
    return scores.len()
}

fn mark_name(flags: Map<Str,Bool>, key: Str, value: Bool) -> Int {
    flags.insert(key, value)
    flags.insert("blue", false)
    flags.remove("blue")
    flags.insert("green", true)
    return flags.len()
}
```

```vais
fn make_scores() -> Map<Int,Int> {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 40)
    scores.insert(9, 2)
    return scores
}

fn main() -> Int {
    let scores: Map<Int,Int> = make_scores()
    return scores.get(4, 0) + scores.get(9, 0)
}
```

```vais
fn make_names() -> Map<Str,Int> {
    let scores: Map<Str,Int> = {}
    scores.insert("red", 40)
    scores.insert("blue", 2)
    return scores
}

fn main() -> Int {
    let scores: Map<Str,Int> = make_names()
    return scores.get("red", 0) + scores.get("blue", 0)
}
```

```vais
fn make_flags() -> Map<Int,Bool> {
    let flags: Map<Int,Bool> = {}
    flags.insert(4, true)
    flags.insert(9, false)
    return flags
}

fn main() -> Int {
    let flags: Map<Int,Bool> = make_flags()
    if flags.get(4, false) and not flags.get(9, true) {
        return 42
    }
    return 0
}
```

```vais
fn make_letters() -> Map<Int,Char> {
    let letters: Map<Int,Char> = {}
    letters.insert(4, 'A')
    letters.insert(9, 'B')
    return letters
}

fn main() -> Int {
    let letters: Map<Int,Char> = make_letters()
    if letters.get(4, 'Z') == 'A' and letters.get(9, 'Z') == 'B' {
        return 42
    }
    return 0
}
```

Verified behavior:

- A Map local must be explicitly annotated as `Map<Int,Int>`, `Map<Int,Bool>`,
  `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, or `Map<Str,Char>`.
- `{}` constructs an empty local map.
- `target = source` copies a same-type local Map, same-type Map parameter, or
  same-type Map-returning call into the target Map without aliasing.
- `insert(key, value)` inserts or replaces the key.
- `remove(key)` removes the key if present and leaves the map unchanged if the
  key is missing.
- `clear()` removes all keys and allows the map to be reused.
- `get(key, default)` returns the stored value or the default.
- `get_opt(key)` returns `Some(value)` or `None` for local `Map<Int,Int>`,
  `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, and
  `Map<Str,Char>` values.
- `contains(key)` returns a `Bool`.
- `len()` returns the number of present keys.
- `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>`, and `Map<Str,Char>` parameters are passed by reference; a
  callee can mutate the caller-visible map.
- `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
  `Map<Str,Bool>`, and `Map<Str,Char>` return values copy returned contents
  into caller-owned local storage when initializing an explicitly annotated
  local.

Not verified yet: generic key/value pairs, entry literals, iteration, custom
hashing, and broader Map APIs that require `Option<T>` or `Result<T,E>` support
beyond the current concrete `Map<Int,V>.get_opt` slices and local
`Map<Str,Int>.get_opt`, `Map<Str,Bool>.get_opt`, and
`Map<Str,Char>.get_opt`.

## Ownership And Mutation Semantics

Future Map ABI gates must use these semantics:

- A local Map variable owns its storage.
- `insert`, `remove`, and `clear` mutate the receiver Map.
- `a = b` copies the contents of `b` into `a`; it does not make `a` and `b`
  aliases.
- Passing a Map to a function passes a mutable collection reference, matching
  current collection-parameter behavior for `List`. If a function calls
  `insert`, `remove`, `clear`, or same-type assignment on its Map parameter, the
  caller-visible Map is mutated.
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
- Assignment uses a concrete copy helper. The source and target can be locals or
  Map parameters when both sides have the same verified concrete Map type.
- `insert`, `remove`, `clear`, `get`, `get_opt`, `contains`, and `len` call helpers specialized for
  the concrete key/value pair.

The current full self-host path uses a fixed-capacity integer buffer for local
`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
`Map<Str,Bool>`, and `Map<Str,Char>`. Future gates may keep
fixed-capacity storage
for early concrete slices, but capacity and trap behavior must be documented and
tested before being advertised.

## Generic Expansion Order

Broaden Map support in this order:

1. `Map<Int,Int>` ABI: parameters first, then returns. Both are now complete
   for the concrete `Map<Int,Int>` slice.
2. More `Map<Int,V>` local slices for already verified scalar values where `V`
   has a stable copy ABI; `Map<Int,Bool>` and `Map<Int,Char>` local values are
   the first completed slices in this step, including `get_opt` match payloads.
3. `Map<Int,V>` ABI: parameters and returns after the local concrete slices are
   stable. Parameters for `Map<Int,Int>`, `Map<Int,Bool>`, and
   `Map<Int,Char>` are complete; returns for all three concrete slices are now
   complete for explicitly annotated local initialization.
4. `Map<Str,Int>` local values as the first string-key slice. This is complete
   for local construction, assignment copy, lookup/update helpers, `remove`,
   `clear`, `get_opt`, parameter reference/mutation, and return-value local
   initialization.
5. Broader `Map<Str,V>` local values only as concrete gates. `Map<Str,Bool>` is
   complete for local construction, assignment copy, lookup/update helpers,
   `remove`, `clear`, `get_opt`, parameter reference/mutation,
   parameter-source, parameter-target, and return-call assignment copy, and
   return-value local initialization. `Map<Str,Char>` is complete for local
   construction, assignment copy, lookup/update helpers, `remove`, `clear`, and
   `get_opt`, plus parameter reference/mutation, parameter-source,
   parameter-target, and return-call assignment copy, and return-value local
   initialization.
6. Broader `Map<Str,V>` only after string equality, hashing, copy, and lifetime
   rules are specified for each value type and ABI boundary.
7. Struct values only after struct copy and return ABI behavior are already
   gate-backed for the chosen Map storage.
8. Generic functions over `Map<K,V>` only after generic type checking can
   constrain key equality and storage helpers deterministically.

Do not publish a broad `Map<K,V>` claim until every accepted `K,V` pair has
front, full, direct, parity or value, and release-gate coverage.

## Method Contract

The next promoted method set should remain the current small API:

| Method | Future concrete signature |
| --- | --- |
| `m.insert(key, value)` | `Map<K,V>, K, V -> Unit` |
| `m.remove(key)` | `Map<K,V>, K -> Unit` |
| `m.clear()` | `Map<K,V> -> Unit` |
| `m.get(key, default)` | `Map<K,V>, K, V -> V` |
| `m.get_opt(key)` | `Map<K,V>, K -> Option<V>` |
| `m.contains(key)` | `Map<K,V>, K -> Bool` |
| `m.len()` | `Map<K,V> -> Int` |

`get_opt` is promoted for the current concrete `Map<Int,Int>`,
`Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, and
`Map<Str,Char>` slices. For future value types, promote
`get_opt` only when the corresponding `Option<V>` payload behavior has a gate.

Iteration, entry literals, capacity configuration, custom hashing, and ordered
maps are later APIs.

## Diagnostics

Until each slice is implemented, the public front must reject unsupported forms:

- Map assignment from anything other than a same-type local Map, same-type Map
  parameter, or same-type Map-returning call with a verified concrete Map type.
- Map function parameters beyond the verified `Map<Int,Int>`,
  `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, and
  `Map<Str,Char>` slices.
- Map function returns beyond the verified `Map<Int,Int>`, `Map<Int,Bool>`,
  `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, and `Map<Str,Char>`
  slices.
- Generic key/value forms outside verified concrete pairs.
- Map literals with entries.
- Unsupported Map methods.

Diagnostics must include a concrete rewrite or a short explanation that only
local `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and
`Map<Str,Bool>`, and `Map<Str,Char>` values are verified in the current local
slice; all six have parameter ABI support and return ABI support.

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
