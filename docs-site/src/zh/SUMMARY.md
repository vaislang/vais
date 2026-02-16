# 目录

[简介](./introduction.md)

# 入门指南

- [安装](./getting-started/installation.md)
- [快速开始](./getting-started/quick-start.md)
- [教程](./getting-started/tutorial.md)
- [Getting Started（详细）](../guide/getting-started.md)

# 指南

- [错误处理模式](../guide/error-handling.md)
- [性能优化](../guides/performance.md)
- [编码风格](../guide/style-guide.md)
- [从Rust迁移到Vais](../guides/migration-from-rust.md)
- [从C/C++迁移到Vais](../guides/migration-from-c.md)
- [Cookbook — 实用食谱](../guides/cookbook.md)
- [测试指南](../guides/testing.md)
- [错误处理最佳实践](../guides/error-handling.md)
- [生态系统包](../guide/ecosystem-packages.md)
- [故障排除 & FAQ](../troubleshooting.md)

# 语言参考

- [语言规范](./language/language-spec.md)
- [泛型](../language/generics.md)
- [类型推断](../language/type-inference.md)
- [迭代器类型推断](../language/iterator-type-inference.md)
- [高级类型系统](../language/advanced-types.md)
- [切片类型](../language/slices.md)
- [生命周期 & 借用检查](../language/lifetimes.md)
- [异步编程](../language/async-tutorial.md)
- [闭包 & Lambda](../language/closures.md)
- [惰性求值（Lazy/Force）](../language/lazy-evaluation.md)
- [编译时功能](../language/comptime-feature.md)

# 标准库

- [标准库参考](../stdlib/stdlib.md)
- [Vec](../stdlib/vec.md)
- [HashMap](../stdlib/hashmap.md)
- [File I/O](../stdlib/file_io.md)
- [网络](../stdlib/net.md)
- [线程](../stdlib/thread.md)
- [通道](../stdlib/channel.md)
- [同步](../stdlib/sync.md)
- [JSON](../stdlib/json.md)
- [Regex](../stdlib/regex.md)
- [加密](../stdlib/crypto.md)

# 编译器

- [架构](../compiler/architecture.md)
- [技术规范](../compiler/tech-spec.md)
- [JIT编译](../compiler/jit-compilation.md)
- [GPU代码生成](../compiler/gpu-codegen.md)
- [JavaScript代码生成](../compiler/js-codegen.md)
- [编译器内部结构](../compiler/internals.md)
- [Inkwell集成](../compiler/inkwell-integration.md)
- [单态化设计](../compiler/monomorphization-design.md)
- [基准测试设计](../compiler/benchmark-design.md)

# 开发工具

- [编辑器集成](../tools/editors.md)
- [LSP服务器](../tools/lsp-features.md)
- [Playground](../tools/playground/README.md)
  - [功能](../tools/playground/features.md)
  - [快速开始](../tools/playground/quickstart.md)
  - [教程](../tools/playground/tutorial.md)
  - [集成](../tools/playground/integration.md)
- [交互式教程](../tools/vais-tutorial/README.md)
  - [使用方法](../tools/vais-tutorial/usage.md)
  - [快速开始](../tools/vais-tutorial/quickstart.md)
- [热重载](../tools/hot-reload.md)
- [代码覆盖率](../tools/coverage.md)
- [包管理器](../tools/package-manager.md)

# 高级主题

- [FFI（外部函数接口）](../advanced/ffi/README.md)
  - [FFI指南](../advanced/ffi/guide.md)
  - [FFI功能](../advanced/ffi/features.md)
- [语言绑定](../advanced/language-bindings.md)
- [Bindgen](../advanced/bindgen/README.md)
  - [C++支持](../advanced/bindgen/cpp-support.md)
  - [C++快速开始](../advanced/bindgen/cpp-quickstart.md)
  - [设计](../advanced/bindgen/design.md)
- [WASM组件](../advanced/wasm/README.md)
  - [Getting Started](../advanced/wasm/getting-started.md)
  - [组件模型](../advanced/wasm/component-model.md)
  - [JS互操作](../advanced/wasm/js-interop.md)
  - [WASI](../advanced/wasm/wasi.md)
- [异步运行时](../advanced/async-runtime.md)
- [国际化（i18n）](../advanced/i18n-design.md)
- [IPv6实现](../advanced/ipv6-implementation.md)
- [包管理器](../advanced/package-manager-design.md)
- [插件系统](../advanced/plugin-system-design.md)
- [Range类型实现](../advanced/range-type-implementation.md)
- [自托管](../advanced/self-hosting-design.md)

# 安全

- [导入路径安全](../security/import-path-security.md)
- [安全增强](../security/security-enhancement.md)

---

# API参考

- [索引](../api/index.md)

## 核心类型

- [Option](../api/option.md)
- [Result](../api/result.md)
- [String](../api/string.md)
- [OwnedString](../api/owned_string.md)
- [ByteBuffer](../api/bytebuffer.md)
- [Box](../api/box.md)
- [Rc](../api/rc.md)
- [Fmt](../api/fmt.md)

## 集合

- [Vec](../api/vec.md)
- [HashMap](../api/hashmap.md)
- [StringMap](../api/stringmap.md)
- [BTreeMap](../api/btreemap.md)
- [Set](../api/set.md)
- [Deque](../api/deque.md)
- [PriorityQueue](../api/priority_queue.md)
- [Collections](../api/collections.md)

## 实用工具

- [Path](../api/path.md)
- [DateTime](../api/datetime.md)
- [Channel](../api/channel.md)
- [Args](../api/args.md)

## I/O和文件系统

- [IO](../api/io.md)
- [File](../api/file.md)
- [Filesystem](../api/filesystem.md)

## 网络和Web

- [Net](../api/net.md)
- [HTTP](../api/http.md)
- [HTTP Client](../api/http_client.md)
- [HTTP Server](../api/http_server.md)
- [WebSocket](../api/websocket.md)
- [TLS](../api/tls.md)
- [URL](../api/url.md)

## 并发

- [Thread](../api/thread.md)
- [Sync](../api/sync.md)
- [Future](../api/future.md)
- [Async](../api/async.md)
- [Runtime](../api/runtime.md)
- [Async Reactor](../api/async_reactor.md)

## 数据处理

- [JSON](../api/json.md)
- [Regex](../api/regex.md)
- [Base64](../api/base64.md)
- [Template](../api/template.md)
- [Compress](../api/compress.md)

## 数据库

- [SQLite](../api/sqlite.md)
- [PostgreSQL](../api/postgres.md)
- [ORM](../api/orm.md)

## 数学和算法

- [Math](../api/math.md)
- [Hash](../api/hash.md)
- [Random](../api/random.md)
- [UUID](../api/uuid.md)
- [CRC32](../api/crc32.md)

## 安全和加密

- [Crypto](../api/crypto.md)
- [Log](../api/log.md)

## 内存管理

- [Memory](../api/memory.md)
- [Allocator](../api/allocator.md)
- [Arena](../api/arena.md)
- [GC](../api/gc.md)

## 系统和运行时

- [Time](../api/time.md)
- [Profiler](../api/profiler.md)
- [Test](../api/test.md)
- [PropTest](../api/proptest.md)
- [Contract](../api/contract.md)
- [GPU](../api/gpu.md)
- [Hot Reload](../api/hot.md)
- [DynLoad](../api/dynload.md)

---

# 贡献

- [团队入职指南](../onboarding.md)
- [贡献指南](../contributing/contributing.md)
- [实现摘要](../contributing/implementation-summaries.md)
  - [完整实现摘要](../contributing/summaries/implementation-summary.md)
  - [异步类型检查](../contributing/summaries/async-type-checking.md)
  - [Bindgen实现](../contributing/summaries/bindgen-implementation.md)
  - [C++ Bindgen实现](../contributing/summaries/cpp-bindgen-implementation.md)
  - [FFI实现](../contributing/summaries/ffi-implementation.md)
  - [GC实现](../contributing/summaries/gc-implementation.md)
  - [热重载实现](../contributing/summaries/hot-reload-implementation.md)
  - [Playground实现](../contributing/summaries/playground-implementation.md)
  - [WASM组件实现](../contributing/summaries/wasm-component-implementation.md)
- [重构摘要](../contributing/refactoring-summary.md)
- [路线图](../contributing/roadmap.md)
- [生产就绪检查清单](../production-checklist.md)

---

[附录](../appendix.md)
