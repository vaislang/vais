# API Reference Index

> Complete API reference for the Vais standard library

## Core Types

| Module | Description |
|--------|-------------|
| [Option](option.md) | Optional value type (`Some(T)` / `None`) |
| [Result](result.md) | Error handling type (`Ok(i64)` / `Err(i64)`) |
| [String](string.md) | Heap-allocated dynamic string |
| [OwnedString](owned_string.md) | Owned string with lifecycle management |
| [ByteBuffer](bytebuffer.md) | Growable byte buffer for binary I/O |
| [Box](box.md) | Heap-allocated single-ownership pointer |
| [Rc](rc.md) | Reference-counted smart pointer |

## Collections

| Module | Description |
|--------|-------------|
| [Vec](vec.md) | Dynamic growable array |
| [HashMap](hashmap.md) | Hash table with generic keys and values |
| [StringMap](stringmap.md) | Hash table with string keys |
| [BTreeMap](btreemap.md) | Self-balancing ordered map (B-tree) |
| [Set](set.md) | Hash-based set collection |
| [Deque](deque.md) | Double-ended queue (circular buffer) |
| [PriorityQueue](priority_queue.md) | Min-heap priority queue |
| [Collections](collections.md) | Unified re-export of all collections |

## I/O and Filesystem

| Module | Description |
|--------|-------------|
| [IO](io.md) | Standard input/output operations |
| [File](file.md) | File I/O with memory mapping and locks |
| [Filesystem](filesystem.md) | POSIX filesystem operations |
| [Fmt](fmt.md) | String formatting and number conversion |

## Networking and Web

| Module | Description |
|--------|-------------|
| [Net](net.md) | TCP/UDP socket networking (IPv4/IPv6) |
| [HTTP](http.md) | HTTP protocol constants and types |
| [HTTP Client](http_client.md) | HTTP client with request/response |
| [HTTP Server](http_server.md) | HTTP server framework with routing |
| [WebSocket](websocket.md) | WebSocket protocol (RFC 6455) |
| [TLS](tls.md) | TLS/SSL via OpenSSL/LibreSSL |
| [URL](url.md) | URL parsing and manipulation |

## Concurrency

| Module | Description |
|--------|-------------|
| [Thread](thread.md) | OS-level threading and thread pools |
| [Sync](sync.md) | Mutex, RwLock, Channel, atomics |
| [Future](future.md) | Future trait and combinators |
| [Async](async.md) | Async utilities (timeout, retry, race) |
| [Runtime](runtime.md) | Async task executor |
| [AsyncReactor](async_reactor.md) | Platform event loop (kqueue/epoll) |

## Data Processing

| Module | Description |
|--------|-------------|
| [JSON](json.md) | JSON parser and generator |
| [Regex](regex.md) | Regular expression matching |
| [Base64](base64.md) | Base64 encoding/decoding |
| [Template](template.md) | Template engine with variable interpolation |

## Databases

| Module | Description |
|--------|-------------|
| [SQLite](sqlite.md) | SQLite3 database bindings |
| [Postgres](postgres.md) | PostgreSQL client (libpq) |
| [ORM](orm.md) | Lightweight object-relational mapping |

## Math and Algorithms

| Module | Description |
|--------|-------------|
| [Math](math.md) | Mathematical functions and constants |
| [Hash](hash.md) | Hash functions for collections |
| [Random](random.md) | Pseudo-random number generation |
| [UUID](uuid.md) | UUID v4 generation and parsing |
| [CRC32](crc32.md) | CRC32 checksum computation |

## Security and Crypto

| Module | Description |
|--------|-------------|
| [Crypto](crypto.md) | SHA-256, HMAC, AES-256 |
| [Compress](compress.md) | Gzip/deflate compression (zlib) |
| [Log](log.md) | Structured logging with JSON output |

## Memory Management

| Module | Description |
|--------|-------------|
| [Memory](memory.md) | Low-level memory operations |
| [Allocator](allocator.md) | Custom allocator traits |
| [Arena](arena.md) | Arena (region) allocator |
| [GC](gc.md) | Optional garbage collector |

## System and Runtime

| Module | Description |
|--------|-------------|
| [Time](time.md) | Time measurement, sleep, Duration |
| [Profiler](profiler.md) | Runtime performance profiling |
| [Test](test.md) | Built-in test framework |
| [PropTest](proptest.md) | Property-based testing |
| [Contract](contract.md) | Design-by-contract support |
| [GPU](gpu.md) | GPU compute (CUDA/Metal) |
| [HotReload](hot.md) | Hot reloading support |
| [DynLoad](dynload.md) | Dynamic module loading |
