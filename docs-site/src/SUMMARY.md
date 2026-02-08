# Summary

[소개](./introduction.md)

# 시작하기

- [팀 온보딩 가이드](./onboarding.md)
- [설치](./getting-started/installation.md)
- [튜토리얼](./getting-started/tutorial.md)
- [빠른 시작](./getting-started/quick-start.md)

# 가이드

- [Getting Started](./guide/getting-started.md)
- [에러 처리 패턴](./guide/error-handling.md)
- [성능 튜닝](./guide/performance.md)
- [코딩 스타일](./guide/style-guide.md)
- [FAQ](./guide/faq.md)
- [Rust에서 Vais로 전환하기](./guides/migration-from-rust.md)
- [C/C++에서 Vais로 전환하기](./guides/migration-from-c.md)
- [Cookbook — 실전 레시피](./guides/cookbook.md)
- [성능 최적화 가이드](./guides/performance.md)
- [트러블슈팅 & FAQ](./troubleshooting.md)

# 언어 레퍼런스

- [언어 사양](./language/language-spec.md)
- [제네릭](./language/generics.md)
- [비동기 프로그래밍](./language/async-tutorial.md)
- [컴파일 타임 기능](./language/comptime-feature.md)
- [타입 추론 개선](./language/iterator-type-inference.md)

# 표준 라이브러리

- [표준 라이브러리 레퍼런스](./stdlib/stdlib.md)
- [GC 구현](./stdlib/gc-implementation.md)
- [GC 빠른 레퍼런스](./stdlib/gc-quick-reference.md)

# 컴파일러

- [아키텍처](./compiler/architecture.md)
- [기술 사양](./compiler/tech-spec.md)
- [JIT 컴파일](./compiler/jit-compilation.md)
- [GPU 코드 생성](./compiler/gpu-codegen.md)

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

# 고급 주제

- [FFI (Foreign Function Interface)](./advanced/ffi/README.md)
  - [FFI 가이드](./advanced/ffi/guide.md)
  - [FFI 기능](./advanced/ffi/features.md)
- [언어 바인딩](./advanced/language-bindings.md)
- [Bindgen](./advanced/bindgen/README.md)
  - [C++ 지원](./advanced/bindgen/cpp-support.md)
  - [C++ 빠른 시작](./advanced/bindgen/cpp-quickstart.md)
- [WASM 컴포넌트](./advanced/wasm/README.md)
  - [컴포넌트 모델](./advanced/wasm/component-model.md)

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

# 프로덕션 배포

- [Production Readiness Checklist](./production-checklist.md)

---

[부록](./appendix.md)
