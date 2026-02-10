# 표준 라이브러리 레퍼런스

## 개요

Vais 표준 라이브러리는 **73개의 모듈**로 구성되어 있으며, 시스템 프로그래밍부터 웹 개발까지 다양한 용도를 지원합니다. 모든 모듈은 `std/` 디렉토리에 위치하며, `U std::{module}` 또는 `U {module}` 구문으로 임포트할 수 있습니다.

## Quick Start

```vais
U std::vec      # 동적 배열
U std::hashmap  # 해시맵
U std::file     # 파일 I/O

F main() {
    v := Vec::new()
    v.push(42)
    println("Hello, Vais!")
}
```

---

## 카테고리별 모듈 목록

### Core Types

기본 타입 및 에러 처리를 위한 핵심 모듈.

| 모듈 | 설명 | API Reference |
|------|------|---------------|
| `option` | `Option<T>` — 값의 유무를 나타내는 타입 | [API](../api/option.md) |
| `result` | `Result<T,E>` — 성공/실패를 나타내는 타입 | [API](../api/result.md) |
| `string` | 불변 문자열 (`&str`) 유틸리티 | [API](../api/string.md) |
| `owned_string` | 소유 문자열 (`String`) 구현 | [API](../api/owned_string.md) |
| `bytebuffer` | 동적 바이트 버퍼 | [API](../api/bytebuffer.md) |
| `box` | 힙 할당 스마트 포인터 `Box<T>` | [API](../api/box.md) |
| `rc` | 참조 카운팅 스마트 포인터 `Rc<T>` | [API](../api/rc.md) |
| `fmt` | 포매팅 유틸리티 | [API](../api/fmt.md) |
| `error` | 에러 처리 트레이트 및 유틸리티 | [API](../api/error.md) |

### Collections

데이터 컬렉션 및 반복자를 위한 모듈.

| 모듈 | 설명 | 가이드 | API Reference |
|------|------|--------|---------------|
| `vec` | 동적 배열 `Vec<T>` | [가이드](./vec.md) | [API](../api/vec.md) |
| `hashmap` | 해시맵 `HashMap<K,V>` | [가이드](./hashmap.md) | [API](../api/hashmap.md) |
| `stringmap` | 문자열 키 전용 해시맵 | | [API](../api/stringmap.md) |
| `btreemap` | B-트리 기반 정렬된 맵 | | [API](../api/btreemap.md) |
| `set` | 집합 자료구조 `Set<T>` | | [API](../api/set.md) |
| `deque` | 양방향 큐 `Deque<T>` | | [API](../api/deque.md) |
| `priority_queue` | 우선순위 큐 | | [API](../api/priority_queue.md) |
| `collections` | 컬렉션 트레이트 및 공통 인터페이스 | | [API](../api/collections.md) |
| `iter` | 반복자 트레이트 및 어댑터 | | [API](../api/iter.md) |

### I/O & Filesystem

파일 시스템 및 입출력 작업을 위한 모듈.

| 모듈 | 설명 | 가이드 | API Reference |
|------|------|--------|---------------|
| `io` | 입출력 트레이트 (`Read`, `Write`) | | [API](../api/io.md) |
| `file` | 파일 읽기/쓰기 | [가이드](./file_io.md) | [API](../api/file.md) |
| `filesystem` | 파일 시스템 조작 (생성/삭제/이동) | | [API](../api/filesystem.md) |
| `path` | 경로 조작 유틸리티 | | [API](../api/path.md) |

### Networking & Web

네트워크 통신 및 웹 서비스를 위한 모듈.

| 모듈 | 설명 | 가이드 | API Reference |
|------|------|--------|---------------|
| `net` | TCP/UDP 소켓 | [가이드](./net.md) | [API](../api/net.md) |
| `http` | HTTP 프로토콜 공통 타입 | | [API](../api/http.md) |
| `http_client` | HTTP 클라이언트 | | [API](../api/http_client.md) |
| `http_server` | HTTP 서버 | | [API](../api/http_server.md) |
| `websocket` | WebSocket 프로토콜 | | [API](../api/websocket.md) |
| `tls` | TLS/SSL 암호화 통신 | | [API](../api/tls.md) |
| `url` | URL 파싱 및 생성 | | [API](../api/url.md) |

### Concurrency

멀티스레딩 및 비동기 프로그래밍을 위한 모듈.

| 모듈 | 설명 | 가이드 | API Reference |
|------|------|--------|---------------|
| `thread` | 스레드 생성 및 관리 | [가이드](./thread.md) | [API](../api/thread.md) |
| `sync` | 동기화 프리미티브 (Mutex, RwLock, Atomic) | [가이드](./sync.md) | [API](../api/sync.md) |
| `channel` | 스레드 간 메시지 패싱 | [가이드](./channel.md) | [API](../api/channel.md) |
| `future` | Future 타입 및 combinators | | [API](../api/future.md) |
| `async` | Async 런타임 프리미티브 (Barrier, Semaphore, WaitGroup) | | [API](../api/async.md) |
| `runtime` | Async 런타임 스케줄러 | | [API](../api/runtime.md) |
| `async_reactor` | 비동기 이벤트 reactor | | [API](../api/async_reactor.md) |
| `async_io` | 비동기 I/O 추상화 | | [API](../api/async_io.md) |
| `async_net` | 비동기 네트워크 (TCP/UDP) | | [API](../api/async_net.md) |
| `async_http` | 비동기 HTTP 서버 | | [API](../api/async_http.md) |

### Data Processing

데이터 직렬화, 정규표현식, 압축 등을 위한 모듈.

| 모듈 | 설명 | 가이드 | API Reference |
|------|------|--------|---------------|
| `json` | JSON 파싱 및 직렬화 | [가이드](./json.md) | [API](../api/json.md) |
| `regex` | 정규표현식 엔진 | [가이드](./regex.md) | [API](../api/regex.md) |
| `base64` | Base64 인코딩/디코딩 | | [API](../api/base64.md) |
| `template` | 텍스트 템플릿 엔진 | | [API](../api/template.md) |
| `compress` | 압축/해제 (gzip, deflate) | | [API](../api/compress.md) |
| `hash` | 해시 함수 (SHA-256, SHA-512) | | [API](../api/hash.md) |
| `crc32` | CRC32 체크섬 | | [API](../api/crc32.md) |

### Databases

데이터베이스 연동을 위한 모듈.

| 모듈 | 설명 | API Reference |
|------|------|---------------|
| `sqlite` | SQLite 바인딩 | [API](../api/sqlite.md) |
| `postgres` | PostgreSQL 클라이언트 | [API](../api/postgres.md) |
| `orm` | ORM (Object-Relational Mapping) | [API](../api/orm.md) |

### Security & Crypto

암호화 및 로깅을 위한 모듈.

| 모듈 | 설명 | 가이드 | API Reference |
|------|------|--------|---------------|
| `crypto` | 암호화 알고리즘 (AES, RSA, ECDSA) | [가이드](./crypto.md) | [API](../api/crypto.md) |
| `log` | 구조화된 로깅 | | [API](../api/log.md) |

### Memory Management

메모리 할당 및 관리를 위한 모듈.

| 모듈 | 설명 | API Reference |
|------|------|---------------|
| `memory` | 메모리 할당자 인터페이스 | [API](../api/memory.md) |
| `allocator` | 커스텀 할당자 구현 | [API](../api/allocator.md) |
| `arena` | Arena 할당자 | [API](../api/arena.md) |
| `gc` | 가비지 컬렉터 (선택적) | [API](../api/gc.md) |

> **참고**: `box`와 `rc`는 Core Types 섹션을 참조하세요.

### System & Runtime

시스템 인터페이스, 시간, 테스트 등을 위한 모듈.

| 모듈 | 설명 | API Reference |
|------|------|---------------|
| `time` | 시간 측정 및 타이머 | [API](../api/time.md) |
| `datetime` | 날짜/시간 파싱 및 포매팅 | [API](../api/datetime.md) |
| `process` | 프로세스 생성 및 관리 | [API](../api/process.md) |
| `signal` | Unix 시그널 처리 | [API](../api/signal.md) |
| `env` | 환경 변수 접근 | [API](../api/env.md) |
| `args` | 커맨드라인 인자 파싱 | [API](../api/args.md) |
| `random` | 난수 생성기 | [API](../api/random.md) |
| `uuid` | UUID 생성 | [API](../api/uuid.md) |
| `math` | 수학 함수 (sin, cos, sqrt 등) | [API](../api/math.md) |
| `profiler` | 성능 프로파일러 | [API](../api/profiler.md) |
| `test` | 유닛 테스트 프레임워크 | [API](../api/test.md) |
| `proptest` | Property-based 테스트 | [API](../api/proptest.md) |
| `contract` | Design-by-Contract (사전조건/사후조건) | [API](../api/contract.md) |

### GPU & WASM

GPU 컴퓨팅 및 WASM 런타임을 위한 모듈.

| 모듈 | 설명 | API Reference |
|------|------|---------------|
| `gpu` | GPU 연산 (CUDA/Metal/OpenCL/WebGPU) | [API](../api/gpu.md) |
| `wasm` | WASM 런타임 인터페이스 (WASI) | [API](../api/wasm.md) |
| `web` | Web API (Console/Timer/DOM/Fetch/Storage) | [API](../api/web.md) |
| `hot` | Hot reloading 지원 | [API](../api/hot.md) |
| `dynload` | 동적 모듈 로딩 | [API](../api/dynload.md) |

---

## 사용 예제

### Core Types
```vais
U std::option
U std::result

F divide(a: i64, b: i64) -> Result<i64, String> {
    I b == 0 {
        R Err("Division by zero")
    }
    R Ok(a / b)
}

F main() {
    M divide(10, 2) {
        Ok(v) => println(~"Result: {v}"),
        Err(e) => println(~"Error: {e}")
    }
}
```

### Collections
```vais
U std::vec
U std::hashmap

F main() {
    # Vec 예제
    nums := Vec::new()
    nums.push(1)
    nums.push(2)
    nums.push(3)

    # HashMap 예제
    map := HashMap::new()
    map.insert("key", 42)
    println(~"Value: {map.get("key")!}")
}
```

### Concurrency
```vais
U std::thread
U std::channel

F main() {
    ch := channel::new()

    thread::spawn(|| {
        ch.send(42)
    })

    value := ch.recv()
    println(~"Received: {value}")
}
```

---

## 추가 리소스

- [표준 라이브러리 가이드 목록](./index.md)
- [API Reference 전체 목록](../api/index.md)
- [Vais 언어 레퍼런스](../reference/index.md)
