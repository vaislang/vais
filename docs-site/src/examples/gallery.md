# Examples Gallery

> 184+ example programs organized by category and difficulty

This gallery provides an overview of the Vais example programs in the `examples/` directory. Examples range from basic "hello world" programs to advanced GPU, async, and self-hosting compiler demos.

---

## Basics

Introductory programs demonstrating core syntax and features.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [hello.vais](https://github.com/vaislang/vais/blob/main/examples/hello.vais) | Minimal program returning exit code | `F main()` |
| [hello_world.vais](https://github.com/vaislang/vais/blob/main/examples/hello_world.vais) | Print "Hello, World!" | `println`, strings |
| [fib.vais](https://github.com/vaislang/vais/blob/main/examples/fib.vais) | Fibonacci with self-recursion | `@` operator, recursion |
| [math.vais](https://github.com/vaislang/vais/blob/main/examples/math.vais) | Arithmetic operations | Operators, expressions |
| [math_test.vais](https://github.com/vaislang/vais/blob/main/examples/math_test.vais) | Math function tests | Function calls |
| [putchar_var.vais](https://github.com/vaislang/vais/blob/main/examples/putchar_var.vais) | Character output | Variables, FFI |
| [printf_test.vais](https://github.com/vaislang/vais/blob/main/examples/printf_test.vais) | Formatted printing | `printf`, format strings |

**Featured: Fibonacci**

```vais
# Self-recursion with @ operator
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

F main()->i64 = fib(10)   # Returns 55
```

---

## Control Flow

If/else, loops, match, and branching patterns.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [control_flow.vais](https://github.com/vaislang/vais/blob/main/examples/control_flow.vais) | Max, countdown, factorial | `I`/`E`, ternary `?:` |
| [loop_break_test.vais](https://github.com/vaislang/vais/blob/main/examples/loop_break_test.vais) | Loop with break | `L`, `B` (break) |
| [loop_opt_test.vais](https://github.com/vaislang/vais/blob/main/examples/loop_opt_test.vais) | Loop optimization | `L`, `C` (continue) |
| [match_test.vais](https://github.com/vaislang/vais/blob/main/examples/match_test.vais) | Pattern matching basics | `M` (match) |
| [match_binding.vais](https://github.com/vaislang/vais/blob/main/examples/match_binding.vais) | Match with variable binding | `M`, bindings |
| [range_test.vais](https://github.com/vaislang/vais/blob/main/examples/range_test.vais) | Range iteration | `..` operator |
| [range_comprehensive_test.vais](https://github.com/vaislang/vais/blob/main/examples/range_comprehensive_test.vais) | Complete range tests | Range types, inclusive |
| [defer_test.vais](https://github.com/vaislang/vais/blob/main/examples/defer_test.vais) | Deferred execution | `D` (defer) |
| [defer_simple.vais](https://github.com/vaislang/vais/blob/main/examples/defer_simple.vais) | Simple defer | `D` |

**Featured: Pattern Matching**

```vais
F describe(x: i64) -> i64 {
    M x {
        0 => 100,
        1 => 200,
        _ => 999
    }
}
```

---

## Functions and Closures

Functions, lambdas, closures, and the pipe operator.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [pipe_operator.vais](https://github.com/vaislang/vais/blob/main/examples/pipe_operator.vais) | Pipe chaining `\|>` | Pipe operator |
| [closure_simple.vais](https://github.com/vaislang/vais/blob/main/examples/closure_simple.vais) | Simple closure | `\|x\| expr` |
| [closure_test.vais](https://github.com/vaislang/vais/blob/main/examples/closure_test.vais) | Closure capturing | Capture, closures |
| [lambda_test.vais](https://github.com/vaislang/vais/blob/main/examples/lambda_test.vais) | Lambda expressions | Lambdas |
| [inline_test.vais](https://github.com/vaislang/vais/blob/main/examples/inline_test.vais) | Inline functions | `#[inline]` |
| [tco_tail_call.vais](https://github.com/vaislang/vais/blob/main/examples/tco_tail_call.vais) | Tail call optimization | TCO, `@` |
| [tco_stress.vais](https://github.com/vaislang/vais/blob/main/examples/tco_stress.vais) | TCO stress test | Deep recursion |

**Featured: Pipe Operator**

```vais
F double(x: i64) -> i64 = x * 2
F add_ten(x: i64) -> i64 = x + 10

F main() -> i64 {
    result := 5 |> double |> add_ten   # 20
    result
}
```

---

## Types and Structs

Struct definitions, methods, enums, and type features.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [enum_test.vais](https://github.com/vaislang/vais/blob/main/examples/enum_test.vais) | Enum variants | `E` (enum), variants |
| [enum_struct_variant_test.vais](https://github.com/vaislang/vais/blob/main/examples/enum_struct_variant_test.vais) | Struct-like enum variants | `E`, struct variants |
| [method_test.vais](https://github.com/vaislang/vais/blob/main/examples/method_test.vais) | Struct methods | `X` (impl), methods |
| [destructuring.vais](https://github.com/vaislang/vais/blob/main/examples/destructuring.vais) | Destructuring | Pattern destructuring |
| [union_test.vais](https://github.com/vaislang/vais/blob/main/examples/union_test.vais) | Union types | `O` (union) |
| [slice_test.vais](https://github.com/vaislang/vais/blob/main/examples/slice_test.vais) | Slice operations | `&[T]`, fat pointers |
| [type_infer_params.vais](https://github.com/vaislang/vais/blob/main/examples/type_infer_params.vais) | Type inference | `:=` inference |
| [linear_types_test.vais](https://github.com/vaislang/vais/blob/main/examples/linear_types_test.vais) | Linear types | Ownership, move |

---

## Generics and Traits

Generic programming and trait-based polymorphism.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [generic_test.vais](https://github.com/vaislang/vais/blob/main/examples/generic_test.vais) | Basic generics | `<T>` |
| [generic_struct_test.vais](https://github.com/vaislang/vais/blob/main/examples/generic_struct_test.vais) | Generic structs | `S Pair<T>` |
| [generic_bounds_test.vais](https://github.com/vaislang/vais/blob/main/examples/generic_bounds_test.vais) | Trait bounds | `<T: Trait>` |
| [generic_vec_test.vais](https://github.com/vaislang/vais/blob/main/examples/generic_vec_test.vais) | Generic Vec usage | `Vec<T>` |
| [const_generic_test.vais](https://github.com/vaislang/vais/blob/main/examples/const_generic_test.vais) | Const generics | `<const N: i64>` |
| [trait_test.vais](https://github.com/vaislang/vais/blob/main/examples/trait_test.vais) | Trait definition | `W` (trait) |
| [trait_advanced_test.vais](https://github.com/vaislang/vais/blob/main/examples/trait_advanced_test.vais) | Advanced traits | Default methods |
| [trait_iter_test.vais](https://github.com/vaislang/vais/blob/main/examples/trait_iter_test.vais) | Iterator trait | `W Iterator` |
| [gat_container.vais](https://github.com/vaislang/vais/blob/main/examples/gat_container.vais) | GAT containers | GATs |
| [gat_functor.vais](https://github.com/vaislang/vais/blob/main/examples/gat_functor.vais) | GAT functors | GATs |
| [gat_iterator.vais](https://github.com/vaislang/vais/blob/main/examples/gat_iterator.vais) | GAT iterators | GATs |

---

## Collections

Standard library collection types.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [simple_vec_test.vais](https://github.com/vaislang/vais/blob/main/examples/simple_vec_test.vais) | Vec basics | `Vec<T>`, push/pop |
| [minimal_vec_test.vais](https://github.com/vaislang/vais/blob/main/examples/minimal_vec_test.vais) | Minimal Vec | Allocation |
| [simple_hashmap_test.vais](https://github.com/vaislang/vais/blob/main/examples/simple_hashmap_test.vais) | HashMap basics | `HashMap<K,V>` |
| [map_literal.vais](https://github.com/vaislang/vais/blob/main/examples/map_literal.vais) | Map literal syntax | Map literals |
| [btreemap_test.vais](https://github.com/vaislang/vais/blob/main/examples/btreemap_test.vais) | BTreeMap ordered map | `BTreeMap` |
| [set_test.vais](https://github.com/vaislang/vais/blob/main/examples/set_test.vais) | Set operations | `Set<T>` |
| [deque_test.vais](https://github.com/vaislang/vais/blob/main/examples/deque_test.vais) | Double-ended queue | `Deque<T>` |
| [priority_queue_test.vais](https://github.com/vaislang/vais/blob/main/examples/priority_queue_test.vais) | Priority queue | `PriorityQueue` |
| [arrays.vais](https://github.com/vaislang/vais/blob/main/examples/arrays.vais) | Array operations | Arrays |
| [iter_test.vais](https://github.com/vaislang/vais/blob/main/examples/iter_test.vais) | Iterator patterns | `Iterator` trait |

---

## Error Handling

Option, Result, and error patterns.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [option_test.vais](https://github.com/vaislang/vais/blob/main/examples/option_test.vais) | Option basics | `Option<T>` |
| [option_test2.vais](https://github.com/vaislang/vais/blob/main/examples/option_test2.vais) | Option advanced | `Some`/`None` |
| [option_test3.vais](https://github.com/vaislang/vais/blob/main/examples/option_test3.vais) | Option chaining | `?` operator |
| [result_test.vais](https://github.com/vaislang/vais/blob/main/examples/result_test.vais) | Result type | `Result<T,E>` |
| [option_result_test.vais](https://github.com/vaislang/vais/blob/main/examples/option_result_test.vais) | Combined patterns | Option + Result |
| [pattern_full_test.vais](https://github.com/vaislang/vais/blob/main/examples/pattern_full_test.vais) | Full pattern matching | Guards, nested |
| [pattern_alias.vais](https://github.com/vaislang/vais/blob/main/examples/pattern_alias.vais) | Pattern alias | `x @ pattern` |

---

## I/O and Networking

File, network, and HTTP operations.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [io_test.vais](https://github.com/vaislang/vais/blob/main/examples/io_test.vais) | I/O operations | `std/io` |
| [file_test.vais](https://github.com/vaislang/vais/blob/main/examples/file_test.vais) | File read/write | `std/file` |
| [http_test.vais](https://github.com/vaislang/vais/blob/main/examples/http_test.vais) | HTTP client | `std/http` |
| [http_server_example.vais](https://github.com/vaislang/vais/blob/main/examples/http_server_example.vais) | HTTP server | `std/http_server` |
| [websocket_example.vais](https://github.com/vaislang/vais/blob/main/examples/websocket_example.vais) | WebSocket | `std/websocket` |
| [ipv6_test.vais](https://github.com/vaislang/vais/blob/main/examples/ipv6_test.vais) | IPv6 networking | IPv6 |
| [ipv6_dual_stack.vais](https://github.com/vaislang/vais/blob/main/examples/ipv6_dual_stack.vais) | Dual-stack networking | IPv4/v6 |

---

## Data Formats

JSON, TOML, templates, and serialization.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [json_test.vais](https://github.com/vaislang/vais/blob/main/examples/json_test.vais) | JSON builder API | `std/json` |
| [template_example.vais](https://github.com/vaislang/vais/blob/main/examples/template_example.vais) | String templates | `std/template` |
| [compress_example.vais](https://github.com/vaislang/vais/blob/main/examples/compress_example.vais) | Data compression | `std/compress` |
| [crc32.vais](https://github.com/vaislang/vais/blob/main/examples/crc32.vais) | CRC32 checksums | `std/crc32` |
| [pilot_json2toml.vais](https://github.com/vaislang/vais/blob/main/examples/pilot_json2toml.vais) | JSON to TOML converter | JSON, TOML |

---

## Async and Concurrency

Async operations, threads, and synchronization.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [async_test.vais](https://github.com/vaislang/vais/blob/main/examples/async_test.vais) | Async basics | `A` (async), `Y` (await) |
| [async_reactor_test.vais](https://github.com/vaislang/vais/blob/main/examples/async_reactor_test.vais) | Async reactor | Event loop |
| [spawn_test.vais](https://github.com/vaislang/vais/blob/main/examples/spawn_test.vais) | Task spawning | `spawn` |
| [thread_test.vais](https://github.com/vaislang/vais/blob/main/examples/thread_test.vais) | Thread creation | `std/thread` |
| [sync_test.vais](https://github.com/vaislang/vais/blob/main/examples/sync_test.vais) | Mutex/lock | `std/sync` |
| [concurrency_stress.vais](https://github.com/vaislang/vais/blob/main/examples/concurrency_stress.vais) | Concurrency stress | Thread safety |
| [lazy_test.vais](https://github.com/vaislang/vais/blob/main/examples/lazy_test.vais) | Lazy evaluation | `lazy`/`force` |
| [lazy_simple.vais](https://github.com/vaislang/vais/blob/main/examples/lazy_simple.vais) | Simple lazy | Thunks |

---

## Memory and System

Memory management, GC, and system operations.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [memory_test.vais](https://github.com/vaislang/vais/blob/main/examples/memory_test.vais) | Memory operations | `malloc`/`free` |
| [malloc_test.vais](https://github.com/vaislang/vais/blob/main/examples/malloc_test.vais) | Manual allocation | Pointers |
| [rc_test.vais](https://github.com/vaislang/vais/blob/main/examples/rc_test.vais) | Reference counting | `Rc<T>` |
| [gc_test.vais](https://github.com/vaislang/vais/blob/main/examples/gc_test.vais) | Garbage collector | `std/gc` |
| [gc_vec_test.vais](https://github.com/vaislang/vais/blob/main/examples/gc_vec_test.vais) | GC with Vec | GC + collections |
| [gc_simple_demo.vais](https://github.com/vaislang/vais/blob/main/examples/gc_simple_demo.vais) | GC demo | GC basics |
| [lifetime_test.vais](https://github.com/vaislang/vais/blob/main/examples/lifetime_test.vais) | Lifetime checking | Borrow checker |
| [runtime_test.vais](https://github.com/vaislang/vais/blob/main/examples/runtime_test.vais) | Runtime system | `std/runtime` |

---

## Databases

Database integration examples.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [sqlite_example.vais](https://github.com/vaislang/vais/blob/main/examples/sqlite_example.vais) | SQLite operations | `std/sqlite` |
| [postgres_example.vais](https://github.com/vaislang/vais/blob/main/examples/postgres_example.vais) | PostgreSQL client | `std/postgres` |
| [orm_example.vais](https://github.com/vaislang/vais/blob/main/examples/orm_example.vais) | ORM usage | `std/orm` |

---

## WebAssembly

WASM target and interop examples.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [wasm_calculator.vais](https://github.com/vaislang/vais/blob/main/examples/wasm_calculator.vais) | WASM calculator | `#[wasm_export]` |
| [wasm_interop.vais](https://github.com/vaislang/vais/blob/main/examples/wasm_interop.vais) | JS/WASM interop | `#[wasm_import]` |
| [wasm_api_client.vais](https://github.com/vaislang/vais/blob/main/examples/wasm_api_client.vais) | WASM API client | Fetch, DOM |
| [wasm_todo_app.vais](https://github.com/vaislang/vais/blob/main/examples/wasm_todo_app.vais) | WASM todo app | Full app |
| [js_target.vais](https://github.com/vaislang/vais/blob/main/examples/js_target.vais) | JavaScript target | `--target js` |
| [js_target_advanced.vais](https://github.com/vaislang/vais/blob/main/examples/js_target_advanced.vais) | Advanced JS output | ESM modules |

---

## GPU

GPU computing examples.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [gpu_vector_add.vais](https://github.com/vaislang/vais/blob/main/examples/gpu_vector_add.vais) | GPU vector addition | `std/gpu`, kernels |
| [simd_test.vais](https://github.com/vaislang/vais/blob/main/examples/simd_test.vais) | SIMD operations | `std/simd` |
| [simd_distance.vais](https://github.com/vaislang/vais/blob/main/examples/simd_distance.vais) | SIMD distance calc | Vectorization |

---

## Macros and Metaprogramming

Macro system and compile-time features.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [macro_test.vais](https://github.com/vaislang/vais/blob/main/examples/macro_test.vais) | Declarative macros | `macro!` |
| [comptime_test.vais](https://github.com/vaislang/vais/blob/main/examples/comptime_test.vais) | Compile-time eval | `comptime` |
| [comptime_simple.vais](https://github.com/vaislang/vais/blob/main/examples/comptime_simple.vais) | Simple comptime | Const evaluation |
| [contract_test.vais](https://github.com/vaislang/vais/blob/main/examples/contract_test.vais) | Design by contract | Pre/postconditions |

---

## Benchmarks

Performance measurement programs.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [bench_fibonacci.vais](https://github.com/vaislang/vais/blob/main/examples/bench_fibonacci.vais) | Fibonacci benchmark | Recursive performance |
| [bench_compute.vais](https://github.com/vaislang/vais/blob/main/examples/bench_compute.vais) | Compute benchmark | Arithmetic performance |
| [bench_sorting.vais](https://github.com/vaislang/vais/blob/main/examples/bench_sorting.vais) | Sorting benchmark | Algorithm performance |
| [bench_matrix.vais](https://github.com/vaislang/vais/blob/main/examples/bench_matrix.vais) | Matrix operations | Dense computation |
| [bench_tree.vais](https://github.com/vaislang/vais/blob/main/examples/bench_tree.vais) | Tree benchmark | Data structure perf |
| [stress_memory.vais](https://github.com/vaislang/vais/blob/main/examples/stress_memory.vais) | Memory stress test | Allocation patterns |
| [stress_fd.vais](https://github.com/vaislang/vais/blob/main/examples/stress_fd.vais) | File descriptor stress | I/O limits |

---

## Self-Hosting

Self-hosting compiler components in Vais.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [selfhost_arith.vais](https://github.com/vaislang/vais/blob/main/examples/selfhost_arith.vais) | Arithmetic codegen | Bootstrap |
| [selfhost_loop.vais](https://github.com/vaislang/vais/blob/main/examples/selfhost_loop.vais) | Loop codegen | Bootstrap |
| [selfhost_cond.vais](https://github.com/vaislang/vais/blob/main/examples/selfhost_cond.vais) | Conditional codegen | Bootstrap |
| [selfhost_nested.vais](https://github.com/vaislang/vais/blob/main/examples/selfhost_nested.vais) | Nested calls codegen | Bootstrap |
| [selfhost_bitwise.vais](https://github.com/vaislang/vais/blob/main/examples/selfhost_bitwise.vais) | Bitwise ops codegen | Bootstrap |

---

## Pilot Projects

Complete application examples.

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [pilot_rest_api.vais](https://github.com/vaislang/vais/blob/main/examples/pilot_rest_api.vais) | REST API server | HTTP, routing |
| [pilot_json2toml.vais](https://github.com/vaislang/vais/blob/main/examples/pilot_json2toml.vais) | JSON-to-TOML converter | Data conversion |
| [tutorial_wc.vais](https://github.com/vaislang/vais/blob/main/examples/tutorial_wc.vais) | Word count tool | CLI tutorial |
| [tutorial_pipeline.vais](https://github.com/vaislang/vais/blob/main/examples/tutorial_pipeline.vais) | Data pipeline | ETL pattern |

---

## Running Examples

```bash
# Compile and run
cargo run --bin vaisc -- examples/fib.vais

# Compile to JavaScript
cargo run --bin vaisc -- --target js examples/js_target.vais

# Compile to WASM
cargo run --bin vaisc -- --target wasm32-unknown-unknown examples/wasm_calculator.vais
```
