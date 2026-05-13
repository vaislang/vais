# Vais Standard Library (`std/`)

Vais 표준 라이브러리는 시스템 프로그래밍에 필요한 핵심 모듈을 제공합니다.
모든 모듈은 `U std/<module>` 구문으로 임포트합니다.

## 모듈 목록

| 모듈 | 파일 | 설명 |
|------|------|------|
| **vec** | `vec.vais` | 동적 배열 (`Vec<T>`): push/pop/insert/remove, 자동 용량 확장 |
| **hashmap** | `hashmap.vais` | 해시 테이블 (`HashMap<K,V>`): separate chaining, 제네릭 키/값 |
| **string** | `string.vais` | 동적 힙 문자열 (`String`): UTF-8 바이트 버퍼, 슬라이스/변환 |
| **option** | `option.vais` | 선택적 값 (`Option<T>`): Some/None, map/and_then/unwrap_or |
| **result** | `result.vais` | 성공/실패 (`Result<T,E>`): Ok/Err, map/and_then/? 연산자 |
| **error** | `error.vais` | 에러 유틸리티: ErrorChain, 카테고리 분류, 컨텍스트 추가 |
| **file** | `file.vais` | 파일 I/O: open/create/read/write/seek/close |
| **io** | `io.vais` | 표준 입출력: println/read_line/prompt_line/read_int |
| **net** | `net.vais` | 네트워크: TCP/UDP 소켓, connect/listen/accept/send/recv |
| **time** | `time.vais` | 시간: Duration/Instant, sleep/unix_timestamp |
| **math** | `math.vais` | 수학: abs/sqrt/pow/sin/cos/log + pi/EULER/TAU 상수 |
| **random** | `random.vais` | 난수: LCG PRNG, random_range/random_f64/shuffle |
| **json** | `json.vais` | JSON 파서/생성기: null/bool/number/string/array/object |
| **fmt** | `fmt.vais` | 포맷팅: fmt_i64/fmt_f64/fmt_pad/FormatBuilder |
| **test** | `test.vais` | 테스트 프레임워크: assert_eq/assert_str_eq/test_run_all |
| **async** | `async.vais` | 비동기 유틸: timeout/retry/race/AsyncMutex/AsyncChannel |
| **channel** | `channel.vais` | CSP 채널: UnboundedChannel, channel_select |
| **future** | `future.vais` | Future/async 런타임: Poll/Waker/spawn/block_on |
| **memory** | `memory.vais` | 저수준 메모리: memset/memmove/memcmp/mem_alloc/mem_free |
| **process** | `process.vais` | 프로세스 실행: process_run/process_open/process_getenv/process_exit |

## 추가 모듈

| 모듈 | 파일 | 설명 |
|------|------|------|
| allocator | `allocator.vais` | 커스텀 메모리 할당자 인터페이스 |
| arena | `arena.vais` | Arena 할당자 (일괄 해제) |
| args | `args.vais` | 명령줄 인수 파싱 |
| async_http | `async_http.vais` | 비동기 HTTP 클라이언트 |
| async_io | `async_io.vais` | 비동기 I/O 연산 |
| async_net | `async_net.vais` | 비동기 네트워크 소켓 |
| async_reactor | `async_reactor.vais` | I/O 이벤트 반응기 (epoll/kqueue/IOCP) |
| base64 | `base64.vais` | Base64 인코딩/디코딩 |
| box | `box.vais` | 힙 박싱 (`Box<T>`) |
| btreemap | `btreemap.vais` | B-트리 기반 정렬 맵 |
| bytebuffer | `bytebuffer.vais` | 바이트 버퍼 |
| bytes | `bytes.vais` | 바이트 슬라이스 유틸리티 |
| collections | `collections.vais` | 컬렉션 유틸리티 |
| compress | `compress.vais` | 데이터 압축 (zlib/deflate) |
| contract | `contract.vais` | Design-by-Contract 어서션 |
| crc32 | `crc32.vais` | CRC-32 체크섬 |
| crypto | `crypto.vais` | 암호화 유틸리티 |
| datetime | `datetime.vais` | 날짜·시간 파싱 및 포맷 |
| deque | `deque.vais` | 덱(양방향 큐) |
| dynload | `dynload.vais` | 동적 라이브러리 로딩 |
| env | `env.vais` | 환경 변수 접근 |
| filesystem | `filesystem.vais` | 파일시스템 경로·디렉토리 연산 |
| future | `future.vais` | Future 타입 및 런타임 |
| gc | `gc.vais` | 선택적 가비지 컬렉터 |
| gpu | `gpu.vais` | GPU 연산 (CUDA/Metal/OpenCL/WebGPU) |
| hash | `hash.vais` | 해시 트레이트 및 기본 구현 |
| hot | `hot.vais` | 핫 리로딩 지원 |
| http | `http.vais` | HTTP 타입 및 유틸리티 |
| http_client | `http_client.vais` | HTTP 클라이언트 |
| http_server | `http_server.vais` | HTTP 서버 |
| iter | `iter.vais` | 이터레이터 트레이트 및 어댑터 |
| log | `log.vais` | 로깅 (debug/info/warn/error 레벨) |
| math | `math.vais` | 수학 함수 및 상수 |
| memory | `memory.vais` | 저수준 메모리 연산 |
| msgpack | `msgpack.vais` | MessagePack 직렬화 |
| orm | `orm.vais` | 간단한 ORM 레이어 |
| owned_string | `owned_string.vais` | 소유권 있는 문자열 래퍼 |
| path | `path.vais` | 파일 경로 처리 |
| postgres | `postgres.vais` | PostgreSQL 클라이언트 |
| priority_queue | `priority_queue.vais` | 우선순위 큐 (힙) |
| profiler | `profiler.vais` | 코드 프로파일링 |
| proptest | `proptest.vais` | 속성 기반 테스팅 |
| protobuf | `protobuf.vais` | Protocol Buffers 직렬화 |
| rc | `rc.vais` | 참조 카운팅 (`Rc<T>`) |
| regex | `regex.vais` | 정규 표현식 |
| runtime | `runtime.vais` | 비동기 런타임 코어 |
| set | `set.vais` | 집합 (`Set<T>`) |
| signal | `signal.vais` | Unix 시그널 처리 |
| simd | `simd.vais` | SIMD 벡터 연산 |
| sqlite | `sqlite.vais` | SQLite 클라이언트 |
| stringmap | `stringmap.vais` | 문자열 키 특화 맵 |
| sync | `sync.vais` | 동기화 원시타입 (Mutex/RwLock/Channel) |
| template | `template.vais` | 텍스트 템플릿 엔진 |
| thread | `thread.vais` | 스레드 생성 및 관리 |
| tls | `tls.vais` | TLS/SSL 소켓 |
| toml | `toml.vais` | TOML 파서 |
| url | `url.vais` | URL 파싱 및 구성 |
| uuid | `uuid.vais` | UUID v4 생성 |
| wasi_p2 | `wasi_p2.vais` | WASI Preview 2 인터페이스 |
| wasm | `wasm.vais` | WebAssembly 유틸리티 |
| web | `web.vais` | 웹 플랫폼 API |
| websocket | `websocket.vais` | WebSocket 프로토콜 |
| yaml | `yaml.vais` | YAML 파서 |

## 빠른 시작

```vais
U std/vec
U std/hashmap
U std/io

F main() -> i64 {
    # Vec 사용
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)
    println("len = {v.len()}")

    # HashMap 사용
    m := HashMap::new()
    m.insert("hello", 42)
    val := m.get("hello")
    M val {
        Some(v) => println("hello = {v}"),
        None    => println("not found")
    }

    0
}
```

## 설계 원칙

- **명시적 에러 처리**: null 대신 `Option<T>`, 예외 대신 `Result<T, E>` 사용
- **엄격한 타입 변환**: 암시적 coercion 없음, `as` 키워드로 명시 변환
- **메모리 안전**: 할당/해제를 명시적으로 관리, `free()` 메서드 제공
- **단일 문자 키워드**: `F`(function), `S`(struct), `U`(use), `M`(match) 등 간결한 문법
