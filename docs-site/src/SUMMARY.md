# Summary

[소개](./introduction.md)

# 시작하기

- [설치](./getting-started/installation.md)
- [빠른 시작](./getting-started/quick-start.md)
- [튜토리얼](./getting-started/tutorial.md)
- [Getting Started (상세)](./guide/getting-started.md)

# 가이드

- [에러 처리 패턴](./guide/error-handling.md)
- [성능 최적화](./guides/performance.md)
- [코딩 스타일](./guide/style-guide.md)
- [Rust에서 Vais로 전환하기](./guides/migration-from-rust.md)
- [C/C++에서 Vais로 전환하기](./guides/migration-from-c.md)
- [Cookbook — 실전 레시피](./guides/cookbook.md)
- [테스트 가이드](./guides/testing.md)
- [에러 처리 Best Practices](./guides/error-handling.md)
- [Ecosystem Packages](./guide/ecosystem-packages.md)
- [트러블슈팅 & FAQ](./troubleshooting.md)

# 언어 레퍼런스

- [언어 사양](./language/language-spec.md)
- [제네릭](./language/generics.md)
- [타입 추론](./language/type-inference.md)
- [Slice Types](./language/slices.md)
- [Lifetimes & Borrow Checking](./language/lifetimes.md)
- [비동기 프로그래밍](./language/async-tutorial.md)
- [컴파일 타임 기능](./language/comptime-feature.md)

# 표준 라이브러리

- [표준 라이브러리 레퍼런스](./stdlib/stdlib.md)
- [Vec](./stdlib/vec.md)
- [HashMap](./stdlib/hashmap.md)
- [File I/O](./stdlib/file_io.md)
- [Networking](./stdlib/net.md)
- [Thread](./stdlib/thread.md)
- [Channel](./stdlib/channel.md)
- [Sync](./stdlib/sync.md)
- [JSON](./stdlib/json.md)
- [Regex](./stdlib/regex.md)
- [Crypto](./stdlib/crypto.md)

# 컴파일러

- [아키텍처](./compiler/architecture.md)
- [기술 사양](./compiler/tech-spec.md)
- [JIT 컴파일](./compiler/jit-compilation.md)
- [GPU 코드 생성](./compiler/gpu-codegen.md)
- [JavaScript 코드 생성](./compiler/js-codegen.md)
- [컴파일러 내부 구조](./compiler/internals.md)

# 개발자 도구

- [에디터 통합](./tools/editors.md)
- [LSP 서버](./tools/lsp-features.md)
- [플레이그라운드](./tools/playground/README.md)
  - [기능](./tools/playground/features.md)
  - [빠른 시작](./tools/playground/quickstart.md)
  - [튜토리얼](./tools/playground/tutorial.md)
  - [통합](./tools/playground/integration.md)
- [인터랙티브 튜토리얼](./tools/vais-tutorial/README.md)
  - [사용법](./tools/vais-tutorial/usage.md)
  - [빠른 시작](./tools/vais-tutorial/quickstart.md)
- [핫 리로드](./tools/hot-reload.md)
- [코드 커버리지](./tools/coverage.md)
- [패키지 매니저](./tools/package-manager.md)

# 고급 주제

- [FFI (Foreign Function Interface)](./advanced/ffi/README.md)
  - [FFI 가이드](./advanced/ffi/guide.md)
  - [FFI 기능](./advanced/ffi/features.md)
- [언어 바인딩](./advanced/language-bindings.md)
- [Bindgen](./advanced/bindgen/README.md)
  - [C++ 지원](./advanced/bindgen/cpp-support.md)
  - [C++ 빠른 시작](./advanced/bindgen/cpp-quickstart.md)
- [WASM 컴포넌트](./advanced/wasm/README.md)
  - [Getting Started](./advanced/wasm/getting-started.md)
  - [컴포넌트 모델](./advanced/wasm/component-model.md)
  - [JS 인터롭](./advanced/wasm/js-interop.md)
  - [WASI](./advanced/wasm/wasi.md)
- [Async 런타임](./advanced/async-runtime.md)

# 보안

- [임포트 경로 보안](./security/import-path-security.md)

---

# API Reference

- [Index](./api/index.md)

## Core Types

- [Option](./api/option.md)
- [Result](./api/result.md)
- [String](./api/string.md)
- [OwnedString](./api/owned_string.md)
- [ByteBuffer](./api/bytebuffer.md)
- [Box](./api/box.md)
- [Rc](./api/rc.md)
- [Fmt](./api/fmt.md)

## Collections

- [Vec](./api/vec.md)
- [HashMap](./api/hashmap.md)
- [StringMap](./api/stringmap.md)
- [BTreeMap](./api/btreemap.md)
- [Set](./api/set.md)
- [Deque](./api/deque.md)
- [PriorityQueue](./api/priority_queue.md)
- [Collections](./api/collections.md)

## Utility

- [Path](./api/path.md)
- [DateTime](./api/datetime.md)
- [Channel](./api/channel.md)
- [Args](./api/args.md)

## I/O and Filesystem

- [IO](./api/io.md)
- [File](./api/file.md)
- [Filesystem](./api/filesystem.md)

## Networking and Web

- [Net](./api/net.md)
- [HTTP](./api/http.md)
- [HTTP Client](./api/http_client.md)
- [HTTP Server](./api/http_server.md)
- [WebSocket](./api/websocket.md)
- [TLS](./api/tls.md)
- [URL](./api/url.md)

## Concurrency

- [Thread](./api/thread.md)
- [Sync](./api/sync.md)
- [Future](./api/future.md)
- [Async](./api/async.md)
- [Runtime](./api/runtime.md)
- [Async Reactor](./api/async_reactor.md)

## Data Processing

- [JSON](./api/json.md)
- [Regex](./api/regex.md)
- [Base64](./api/base64.md)
- [Template](./api/template.md)
- [Compress](./api/compress.md)

## Databases

- [SQLite](./api/sqlite.md)
- [PostgreSQL](./api/postgres.md)
- [ORM](./api/orm.md)

## Math and Algorithms

- [Math](./api/math.md)
- [Hash](./api/hash.md)
- [Random](./api/random.md)
- [UUID](./api/uuid.md)
- [CRC32](./api/crc32.md)

## Security and Crypto

- [Crypto](./api/crypto.md)
- [Log](./api/log.md)

## Memory Management

- [Memory](./api/memory.md)
- [Allocator](./api/allocator.md)
- [Arena](./api/arena.md)
- [GC](./api/gc.md)

## System and Runtime

- [Time](./api/time.md)
- [Profiler](./api/profiler.md)
- [Test](./api/test.md)
- [PropTest](./api/proptest.md)
- [Contract](./api/contract.md)
- [GPU](./api/gpu.md)
- [Hot Reload](./api/hot.md)
- [DynLoad](./api/dynload.md)

---

# Contributing

- [팀 온보딩 가이드](./onboarding.md)
- [기여 가이드](./contributing/contributing.md)
- [구현 요약](./contributing/implementation-summaries.md)
- [리팩토링 요약](./contributing/refactoring-summary.md)
- [로드맵](./contributing/roadmap.md)
- [Production Readiness Checklist](./production-checklist.md)

---

[부록](./appendix.md)
