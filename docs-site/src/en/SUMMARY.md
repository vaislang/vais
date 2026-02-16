# Summary

[Introduction](./introduction.md)

# Getting Started

- [Installation](./getting-started/installation.md)
- [Quick Start](./getting-started/quick-start.md)
- [Tutorial](./getting-started/tutorial.md)
- [Getting Started (Detailed)](../guide/getting-started.md)

# Guides

- [Error Handling Patterns](../guide/error-handling.md)
- [Performance Optimization](../guides/performance.md)
- [Coding Style](../guide/style-guide.md)
- [Migrating from Rust](../guides/migration-from-rust.md)
- [Migrating from C/C++](../guides/migration-from-c.md)
- [Cookbook — Practical Recipes](../guides/cookbook.md)
- [Testing Guide](../guides/testing.md)
- [Error Handling Best Practices](../guides/error-handling.md)
- [Ecosystem Packages](../guide/ecosystem-packages.md)
- [Troubleshooting & FAQ](../troubleshooting.md)

# Language Reference

- [Language Specification](./language/language-spec.md)
- [Generics](../language/generics.md)
- [Type Inference](../language/type-inference.md)
- [Iterator Type Inference](../language/iterator-type-inference.md)
- [Advanced Type System](../language/advanced-types.md)
- [Slice Types](../language/slices.md)
- [Lifetimes & Borrow Checking](../language/lifetimes.md)
- [Async Programming](../language/async-tutorial.md)
- [Closures & Lambdas](../language/closures.md)
- [Lazy Evaluation (Lazy/Force)](../language/lazy-evaluation.md)
- [Compile-Time Features](../language/comptime-feature.md)

# Standard Library

- [Standard Library Reference](../stdlib/stdlib.md)
- [Vec](../stdlib/vec.md)
- [HashMap](../stdlib/hashmap.md)
- [File I/O](../stdlib/file_io.md)
- [Networking](../stdlib/net.md)
- [Thread](../stdlib/thread.md)
- [Channel](../stdlib/channel.md)
- [Sync](../stdlib/sync.md)
- [JSON](../stdlib/json.md)
- [Regex](../stdlib/regex.md)
- [Crypto](../stdlib/crypto.md)

# Compiler

- [Architecture](../compiler/architecture.md)
- [Technical Specification](../compiler/tech-spec.md)
- [JIT Compilation](../compiler/jit-compilation.md)
- [GPU Code Generation](../compiler/gpu-codegen.md)
- [JavaScript Code Generation](../compiler/js-codegen.md)
- [Compiler Internals](../compiler/internals.md)
- [Inkwell Integration](../compiler/inkwell-integration.md)
- [Monomorphization Design](../compiler/monomorphization-design.md)
- [Benchmark Design](../compiler/benchmark-design.md)

# Developer Tools

- [Editor Integration](../tools/editors.md)
- [LSP Server](../tools/lsp-features.md)
- [Playground](../tools/playground/README.md)
  - [Features](../tools/playground/features.md)
  - [Quick Start](../tools/playground/quickstart.md)
  - [Tutorial](../tools/playground/tutorial.md)
  - [Integration](../tools/playground/integration.md)
- [Interactive Tutorial](../tools/vais-tutorial/README.md)
  - [Usage](../tools/vais-tutorial/usage.md)
  - [Quick Start](../tools/vais-tutorial/quickstart.md)
- [Hot Reload](../tools/hot-reload.md)
- [Code Coverage](../tools/coverage.md)
- [Package Manager](../tools/package-manager.md)

# Advanced Topics

- [FFI (Foreign Function Interface)](../advanced/ffi/README.md)
  - [FFI Guide](../advanced/ffi/guide.md)
  - [FFI Features](../advanced/ffi/features.md)
- [Language Bindings](../advanced/language-bindings.md)
- [Bindgen](../advanced/bindgen/README.md)
  - [C++ Support](../advanced/bindgen/cpp-support.md)
  - [C++ Quick Start](../advanced/bindgen/cpp-quickstart.md)
  - [Design](../advanced/bindgen/design.md)
- [WASM Component](../advanced/wasm/README.md)
  - [Getting Started](../advanced/wasm/getting-started.md)
  - [Component Model](../advanced/wasm/component-model.md)
  - [JS Interop](../advanced/wasm/js-interop.md)
  - [WASI](../advanced/wasm/wasi.md)
- [Async Runtime](../advanced/async-runtime.md)
- [Internationalization (i18n)](../advanced/i18n-design.md)
- [IPv6 Implementation](../advanced/ipv6-implementation.md)
- [Package Manager](../advanced/package-manager-design.md)
- [Plugin System](../advanced/plugin-system-design.md)
- [Range Type Implementation](../advanced/range-type-implementation.md)
- [Self-Hosting](../advanced/self-hosting-design.md)

# Security

- [Import Path Security](../security/import-path-security.md)
- [Security Enhancement](../security/security-enhancement.md)

---

# API Reference

- [Index](../api/index.md)

## Core Types

- [Option](../api/option.md)
- [Result](../api/result.md)
- [String](../api/string.md)
- [OwnedString](../api/owned_string.md)
- [ByteBuffer](../api/bytebuffer.md)
- [Box](../api/box.md)
- [Rc](../api/rc.md)
- [Fmt](../api/fmt.md)

## Collections

- [Vec](../api/vec.md)
- [HashMap](../api/hashmap.md)
- [StringMap](../api/stringmap.md)
- [BTreeMap](../api/btreemap.md)
- [Set](../api/set.md)
- [Deque](../api/deque.md)
- [PriorityQueue](../api/priority_queue.md)
- [Collections](../api/collections.md)

## Utility

- [Path](../api/path.md)
- [DateTime](../api/datetime.md)
- [Channel](../api/channel.md)
- [Args](../api/args.md)

## I/O and Filesystem

- [IO](../api/io.md)
- [File](../api/file.md)
- [Filesystem](../api/filesystem.md)

## Networking and Web

- [Net](../api/net.md)
- [HTTP](../api/http.md)
- [HTTP Client](../api/http_client.md)
- [HTTP Server](../api/http_server.md)
- [WebSocket](../api/websocket.md)
- [TLS](../api/tls.md)
- [URL](../api/url.md)

## Concurrency

- [Thread](../api/thread.md)
- [Sync](../api/sync.md)
- [Future](../api/future.md)
- [Async](../api/async.md)
- [Runtime](../api/runtime.md)
- [Async Reactor](../api/async_reactor.md)

## Data Processing

- [JSON](../api/json.md)
- [Regex](../api/regex.md)
- [Base64](../api/base64.md)
- [Template](../api/template.md)
- [Compress](../api/compress.md)

## Databases

- [SQLite](../api/sqlite.md)
- [PostgreSQL](../api/postgres.md)
- [ORM](../api/orm.md)

## Math and Algorithms

- [Math](../api/math.md)
- [Hash](../api/hash.md)
- [Random](../api/random.md)
- [UUID](../api/uuid.md)
- [CRC32](../api/crc32.md)

## Security and Crypto

- [Crypto](../api/crypto.md)
- [Log](../api/log.md)

## Memory Management

- [Memory](../api/memory.md)
- [Allocator](../api/allocator.md)
- [Arena](../api/arena.md)
- [GC](../api/gc.md)

## System and Runtime

- [Time](../api/time.md)
- [Profiler](../api/profiler.md)
- [Test](../api/test.md)
- [PropTest](../api/proptest.md)
- [Contract](../api/contract.md)
- [GPU](../api/gpu.md)
- [Hot Reload](../api/hot.md)
- [DynLoad](../api/dynload.md)

---

# Contributing

- [Team Onboarding Guide](../onboarding.md)
- [Contributing Guide](../contributing/contributing.md)
- [Implementation Summaries](../contributing/implementation-summaries.md)
  - [Full Implementation Summary](../contributing/summaries/implementation-summary.md)
  - [Async Type Checking](../contributing/summaries/async-type-checking.md)
  - [Bindgen Implementation](../contributing/summaries/bindgen-implementation.md)
  - [C++ Bindgen Implementation](../contributing/summaries/cpp-bindgen-implementation.md)
  - [FFI Implementation](../contributing/summaries/ffi-implementation.md)
  - [GC Implementation](../contributing/summaries/gc-implementation.md)
  - [Hot Reload Implementation](../contributing/summaries/hot-reload-implementation.md)
  - [Playground Implementation](../contributing/summaries/playground-implementation.md)
  - [WASM Component Implementation](../contributing/summaries/wasm-component-implementation.md)
- [Refactoring Summary](../contributing/refactoring-summary.md)
- [Roadmap](../contributing/roadmap.md)
- [Production Readiness Checklist](../production-checklist.md)

---

[Appendix](../appendix.md)
