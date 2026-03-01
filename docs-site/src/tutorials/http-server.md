# Tutorial: HTTP Server 만들기

이 튜토리얼에서는 Vais의 표준 라이브러리를 사용하여 간단한 REST API 서버를 만듭니다. JSON 요청/응답 처리와 간단한 메모리 내 데이터 저장을 구현합니다.

## 최종 결과

```bash
$ vaisc run examples/tutorial_http_server.vais
Server listening on port 8080
# 다른 터미널에서:
$ curl http://localhost:8080/api/hello
{"message":"Hello from Vais!"}
```

## 사전 준비

- Vais 설치 완료
- [CLI Tool 튜토리얼](./cli-tool.md) 완료 권장
- `curl` 또는 브라우저 (API 테스트용)

---

## Step 1: TCP 서버 기반 (10분)

HTTP 서버는 TCP 소켓 위에 동작합니다. Vais에서는 C 런타임 함수를 통해 네트워킹을 지원합니다:

```vais
# 외부 함수 선언
N "C" {
    F __tcp_listen(port: i64) -> i64
    F __tcp_accept(listener_fd: i64) -> i64
    F __tcp_send(fd: i64, data: i64, len: i64) -> i64
    F __tcp_recv(fd: i64, buffer: i64, len: i64) -> i64
    F __tcp_close(fd: i64) -> i64
    F strlen(s: str) -> i64
    F malloc(size: i64) -> i64
    F free(ptr: i64) -> i64
}
```

**핵심 개념**:
- `N "C"`는 C FFI 블록입니다
- TCP 함수들은 파일 디스크립터(fd) 기반으로 동작합니다
- `__tcp_listen`은 포트를 열고, `__tcp_accept`는 연결을 수락합니다

---

## Step 2: HTTP 요청 파싱 (15분)

HTTP 요청의 첫 줄에서 메서드와 경로를 추출합니다:

```vais
# HTTP 요청 정보
S HttpRequest {
    method: i64    # 0=GET, 1=POST, 2=PUT, 3=DELETE
    path_start: i64
    path_len: i64
    buffer: i64
}

F parse_request(buf: i64, len: i64) -> HttpRequest {
    # 메서드 판별 (첫 바이트)
    first := load_byte(buf, 0)
    method := mut 0
    I first == 71 { method = 0 }   # G -> GET
    I first == 80 { method = 1 }   # P -> POST

    # 경로 시작점 찾기 (첫 번째 공백 이후)
    path_s := mut 0
    L i:0..len {
        I load_byte(buf, i) == 32 {
            path_s = i + 1
            B
        }
    }

    # 경로 끝점 찾기 (두 번째 공백)
    path_e := mut path_s
    L i:path_s..len {
        I load_byte(buf, i) == 32 {
            path_e = i
            B
        }
    }

    R HttpRequest {
        method: method,
        path_start: path_s,
        path_len: path_e - path_s,
        buffer: buf
    }
}
```

**핵심 개념**:
- `B`는 break (루프 탈출)
- `32`는 공백 문자의 ASCII 코드
- HTTP 요청 형식: `GET /path HTTP/1.1\r\n...`
- 구조체 필드에 `i64`를 사용하는 이유: Vais의 기본 정수 타입

---

## Step 3: HTTP 응답 생성 (10분)

HTTP 응답을 구성하는 헬퍼 함수를 만듭니다:

```vais
F send_response(fd: i64, status: i64, body: str) {
    # 응답 헤더 구성
    header := mut "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n"
    I status == 404 {
        header = "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n"
    }

    # 헤더 전송
    hdr_len := strlen(header)
    __tcp_send(fd, header, hdr_len)

    # 본문 전송
    body_len := strlen(body)
    __tcp_send(fd, body, body_len)
}
```

**핵심 개념**:
- HTTP 응답은 상태 줄 + 헤더 + 빈 줄 + 본문으로 구성
- `\r\n`은 HTTP 줄바꿈 (CRLF)
- `Content-Type: application/json`으로 JSON 응답 명시

---

## Step 4: 라우터 구현 (15분)

경로에 따라 다른 핸들러를 실행하는 라우터를 만듭니다:

```vais
F path_equals(buf: i64, start: i64, len: i64, target: str) -> i64 {
    target_len := strlen(target)
    I len != target_len { R 0 }

    L i:0..len {
        I load_byte(buf, start + i) != load_byte(target, i) {
            R 0
        }
    }
    R 1
}

F handle_request(fd: i64, req: HttpRequest) {
    # 라우트 매칭
    I path_equals(req.buffer, req.path_start, req.path_len, "/") == 1 {
        send_response(fd, 200, "{\"service\":\"Vais API\",\"version\":\"1.0\"}")
        R 0
    }

    I path_equals(req.buffer, req.path_start, req.path_len, "/api/hello") == 1 {
        send_response(fd, 200, "{\"message\":\"Hello from Vais!\"}")
        R 0
    }

    I path_equals(req.buffer, req.path_start, req.path_len, "/health") == 1 {
        send_response(fd, 200, "{\"status\":\"ok\"}")
        R 0
    }

    # 404 Not Found
    send_response(fd, 404, "{\"error\":\"Not Found\"}")
}
```

**핵심 개념**:
- 바이트 단위 문자열 비교 (길이 비교 + 바이트별 비교)
- 라우팅은 경로 매칭의 연속입니다
- `R 0`으로 함수 조기 반환 (early return)

---

## Step 5: 서버 메인 루프 (10분)

요청을 수신하고 처리하는 메인 루프를 작성합니다:

```vais
F main() -> i64 {
    port := 8080
    puts("Starting Vais HTTP Server on port ~{port}...")

    listener := __tcp_listen(port)
    I listener < 0 {
        puts("ERROR: Failed to listen on port ~{port}")
        R 1
    }

    puts("Server listening on port ~{port}")
    puts("Try: curl http://localhost:~{port}/api/hello")

    # 요청 수신 버퍼
    buf := malloc(4096)

    # 서버 루프 (데모: 10개 요청 처리 후 종료)
    L i:0..10 {
        client := __tcp_accept(listener)
        I client < 0 { C }   # 에러 시 스킵

        # 요청 읽기
        n := __tcp_recv(client, buf, 4095)
        I n > 0 {
            store_byte(buf, n, 0)   # null-terminate
            req := parse_request(buf, n)
            handle_request(client, req)
        }

        __tcp_close(client)
    }

    free(buf)
    __tcp_close(listener)
    puts("Server shut down.")
    0
}
```

**핵심 개념**:
- `C`는 continue (현재 반복 스킵)
- `malloc`/`free`로 버퍼 메모리 관리
- `store_byte`로 null-terminator 추가
- 데모용으로 10개 요청 후 종료 (실제 서버는 무한 루프 사용)

---

## Step 6: JSON 응답 빌더 (15분)

동적 JSON 응답을 생성하는 헬퍼를 추가합니다:

```vais
# 간단한 JSON key:value 응답 생성
F json_kv(key: str, value: str) -> i64 {
    buf := malloc(512)
    pos := mut 0

    # {"key":"value"}
    store_byte(buf, pos, 123)   # {
    pos = pos + 1
    store_byte(buf, pos, 34)    # "
    pos = pos + 1

    # key 복사
    key_len := strlen(key)
    L i:0..key_len {
        store_byte(buf, pos, load_byte(key, i))
        pos = pos + 1
    }

    store_byte(buf, pos, 34)    # "
    pos = pos + 1
    store_byte(buf, pos, 58)    # :
    pos = pos + 1
    store_byte(buf, pos, 34)    # "
    pos = pos + 1

    # value 복사
    val_len := strlen(value)
    L i:0..val_len {
        store_byte(buf, pos, load_byte(value, i))
        pos = pos + 1
    }

    store_byte(buf, pos, 34)    # "
    pos = pos + 1
    store_byte(buf, pos, 125)   # }
    pos = pos + 1
    store_byte(buf, pos, 0)     # null

    buf
}
```

**핵심 개념**:
- 바이트 단위로 JSON 문자열을 수동 조립합니다
- ASCII 코드: `{`=123, `}`=125, `"`=34, `:`=58
- 반환된 포인터는 호출자가 `free()`해야 합니다

---

## Step 7: 카운터 엔드포인트 (10분)

요청 횟수를 추적하는 상태 관리 기능을 추가합니다:

```vais
# 전역 카운터
G request_count := 0

F handle_request_with_counter(fd: i64, req: HttpRequest) {
    request_count = request_count + 1

    I path_equals(req.buffer, req.path_start, req.path_len, "/stats") == 1 {
        puts("Request #~{request_count}: /stats")
        # 응답 생성
        body := json_kv("requests", "~{request_count}")
        send_response(fd, 200, body)
        free(body)
        R 0
    }

    # 기존 라우트...
    handle_request(fd, req)
}
```

**핵심 개념**:
- `G`는 전역 변수 선언 키워드
- 전역 변수로 요청 간 상태를 유지합니다
- 참고: 실전에서는 구조체 기반 상태 관리를 권장합니다

---

## 전체 코드

`examples/tutorial_http_server.vais`에서 전체 코드를 확인할 수 있습니다.

---

## API 테스트

```bash
# 서버 실행
vaisc run examples/tutorial_http_server.vais

# 다른 터미널에서 테스트
curl http://localhost:8080/
curl http://localhost:8080/api/hello
curl http://localhost:8080/health
curl http://localhost:8080/stats
curl http://localhost:8080/unknown    # 404
```

---

## 확장 아이디어

1. **요청 본문 파싱**: POST 요청의 JSON 본문을 파싱하여 데이터 저장
2. **라우트 패턴**: `/users/:id` 같은 동적 경로 파라미터
3. **CORS 헤더**: 브라우저 호환을 위한 Access-Control 헤더 추가
4. **파일 서빙**: 정적 파일을 HTTP로 서빙
5. **로깅**: 요청/응답 로그를 파일에 기록

---

## 배운 것 요약

| 개념 | Vais 문법 | 설명 |
|------|-----------|------|
| FFI | `N "C" { F func() }` | C 네트워킹 함수 호출 |
| 전역 변수 | `G name := value` | 상태 유지 |
| 메모리 | `malloc`/`free` | 수동 메모리 관리 |
| 바이트 조작 | `store_byte`/`load_byte` | 프로토콜 파싱 |
| 조기 반환 | `R value` | 함수 탈출 |
| 루프 제어 | `B` (break), `C` (continue) | 루프 흐름 제어 |
| 구조체 | `S Name { fields }` | 요청 데이터 구조화 |

이전 튜토리얼: [CLI Tool 만들기](./cli-tool.md)
다음 튜토리얼: [Data Pipeline 만들기](./data-pipeline.md)
