# Tutorial: WebSocket Chat Server 만들기

이 튜토리얼에서는 Vais로 실시간 WebSocket 채팅 서버를 만듭니다. 클라이언트가 메시지를 보내면 서버가 모든 연결된 클라이언트에게 브로드캐스트하는 echo-broadcast 패턴을 구현합니다.

## 최종 결과

```bash
$ vaisc run examples/tutorial_ws_chat.vais
=== Vais WebSocket Chat Server ===
Listening on port 9001
Test with: websocat ws://127.0.0.1:9001
```

## 사전 준비

- Vais 설치 완료
- [CLI Tool 튜토리얼](./cli-tool.md) 완료 권장
- `websocat` 또는 `wscat` (WebSocket 클라이언트)

---

## Step 1: TCP 기반 서버 뼈대 (10분)

WebSocket은 TCP 위에 동작합니다. 먼저 TCP 서버를 설정합니다:

```vais
# 외부 함수 선언 — C 런타임 네트워킹
N "C" {
    F __tcp_listen(port: i64) -> i64
    F __tcp_accept(listener_fd: i64) -> i64
    F __tcp_send(fd: i64, data: i64, len: i64) -> i64
    F __tcp_recv(fd: i64, buffer: i64, len: i64) -> i64
    F __tcp_close(fd: i64) -> i64
    F __malloc(size: i64) -> i64
    F __free(ptr: i64) -> i64
}

F main() -> i64 {
    port := 9001
    puts("=== Vais WebSocket Chat Server ===")

    listener := __tcp_listen(port)
    I listener < 0 {
        puts("Failed to start server")
        R 1
    }

    puts("Listening on port 9001")
    puts("Waiting for connections...")

    # Accept loop
    L true {
        client := __tcp_accept(listener)
        I client < 0 { C }

        puts("New connection!")
        __tcp_close(client)
    }

    __tcp_close(listener)
    0
}
```

**핵심 포인트**:
- `N "C" { ... }` 블록으로 C 런타임 함수를 선언합니다
- `__tcp_listen`은 포트를 바인딩하고 리스너 파일 디스크립터를 반환합니다
- `L true { ... }`는 무한 루프입니다 (클라이언트를 계속 수락)

---

## Step 2: WebSocket 핸드셰이크 (15분)

WebSocket 연결은 HTTP Upgrade 요청으로 시작됩니다. 핸드셰이크를 처리하는 함수를 작성합니다:

```vais
# WebSocket 런타임 함수들
N "C" {
    F __ws_parse_upgrade_request(buffer: i64, len: i64) -> i64
    F __ws_accept_key(client_key: i64) -> i64
    F __ws_build_upgrade_response(accept_key: i64, out_buffer: i64) -> i64
    F __find_header_end(buffer: i64, len: i64) -> i64
    F __strlen(s: str) -> i64
}

F do_handshake(fd: i64) -> i64 {
    buf := __malloc(8192)
    I buf == 0 { R 0 }

    # HTTP 요청 전체를 읽기 (헤더 끝까지)
    total := mut 0
    L true {
        n := __tcp_recv(fd, buf + total, 4096)
        I n <= 0 {
            __free(buf)
            R 0
        }
        total = total + n
        I __find_header_end(buf, total) >= 0 { B }
        I total >= 8192 {
            __free(buf)
            R 0
        }
    }

    # Sec-WebSocket-Key 추출
    ws_key := __ws_parse_upgrade_request(buf, total)
    __free(buf)
    I ws_key == 0 { R 0 }

    # SHA-1 + Base64로 Accept 키 생성
    accept_key := __ws_accept_key(ws_key)
    __free(ws_key)
    I accept_key == 0 { R 0 }

    # 101 Switching Protocols 응답 전송
    resp_buf := __malloc(512)
    I resp_buf == 0 {
        __free(accept_key)
        R 0
    }
    resp_len := __ws_build_upgrade_response(accept_key, resp_buf)
    __free(accept_key)

    sent := __tcp_send(fd, resp_buf, resp_len)
    __free(resp_buf)

    I sent > 0 { 1 } E { 0 }
}
```

**핵심 포인트**:
- WebSocket 핸드셰이크는 HTTP → WebSocket 프로토콜 업그레이드입니다
- `__ws_parse_upgrade_request`가 `Sec-WebSocket-Key` 헤더를 추출합니다
- `__ws_accept_key`가 RFC 6455 규격에 따라 Accept 키를 계산합니다
- `D` (defer)로 메모리 해제를 보장할 수도 있습니다 (Step 4 참고)

---

## Step 3: 메시지 수신과 에코 (15분)

WebSocket 프레임을 디코딩하고 에코하는 루프를 작성합니다:

```vais
# WebSocket opcodes
C WS_TEXT: i64 = 1
C WS_CLOSE: i64 = 8
C WS_PING: i64 = 9
C WS_PONG: i64 = 10

N "C" {
    F __ws_encode_frame(opcode: i64, payload: i64, payload_len: i64,
                        masked: i64, mask_key: i64, out_frame: i64) -> i64
    F __ws_decode_frame(data: i64, data_len: i64, out_frame: i64) -> i64
    F __load_i64(ptr: i64) -> i64
}

F ws_send_text(fd: i64, msg: str) -> i64 {
    msg_len := __strlen(msg)
    buf := __malloc(msg_len + 14)
    I buf == 0 { R -1 }
    frame_len := __ws_encode_frame(WS_TEXT, msg as i64, msg_len, 0, 0, buf)
    sent := __tcp_send(fd, buf, frame_len)
    __free(buf)
    sent
}

F handle_client(fd: i64) -> i64 {
    recv_buf := __malloc(65550)
    frame_out := __malloc(40)   # 5 x i64 필드
    I recv_buf == 0 { R -1 }

    is_open := mut 1
    L is_open == 1 {
        # 최소 2바이트 수신 (프레임 헤더)
        total := mut 0
        L total < 2 {
            n := __tcp_recv(fd, recv_buf + total, 4096)
            I n <= 0 { is_open = 0; B }
            total = total + n
        }
        I is_open == 0 { B }

        # 프레임 디코딩
        consumed := __ws_decode_frame(recv_buf, total, frame_out)

        opcode := __load_i64(frame_out)
        payload := __load_i64(frame_out + 8)
        payload_len := __load_i64(frame_out + 16)

        M opcode {
            1 => {
                # 텍스트 메시지 — 에코
                puts("Received text message")
                ws_send_text(fd, payload as str)
            },
            8 => {
                # Close 프레임
                puts("Client disconnected")
                is_open = 0
            },
            9 => {
                # Ping → Pong 응답
                pong_buf := __malloc(16)
                __ws_encode_frame(WS_PONG, 0, 0, 0, 0, pong_buf)
                __tcp_send(fd, pong_buf, 2)
                __free(pong_buf)
            },
            _ => { }
        }

        I payload != 0 { __free(payload) }
    }

    __free(recv_buf)
    __free(frame_out)
    0
}
```

**핵심 포인트**:
- `M opcode { ... }`로 WebSocket 프레임 타입별 처리를 합니다
- 서버→클라이언트는 마스킹 없이 (`masked=0`) 전송합니다
- `__load_i64(frame_out + offset)`으로 구조체 필드를 읽습니다
- `is_open := mut 1` — 가변 플래그로 루프 제어

---

## Step 4: Defer로 리소스 정리 (5분)

메모리 누수를 방지하기 위해 `D` (defer)를 활용합니다:

```vais
F handle_client_safe(fd: i64) -> i64 {
    recv_buf := __malloc(65550)
    D __free(recv_buf)       # 함수 종료 시 자동 해제

    frame_out := __malloc(40)
    D __free(frame_out)      # LIFO 순서로 해제

    # ... 나머지 로직 동일
    0
}
```

`D` 문은 현재 스코프가 종료될 때 역순(LIFO)으로 실행됩니다. 에러 경로에서도 정리가 보장됩니다.

---

## Step 5: 전체 조합 (10분)

모든 함수를 조합하여 완전한 서버를 만듭니다:

```vais
F main() -> i64 {
    port := 9001
    puts("=== Vais WebSocket Chat Server ===")

    listener := __tcp_listen(port)
    I listener < 0 {
        puts("Failed to start server")
        R 1
    }

    puts("Listening on port 9001")
    puts("Test with: websocat ws://127.0.0.1:9001")

    L true {
        client := __tcp_accept(listener)
        I client < 0 { C }

        puts("New TCP connection")

        ok := do_handshake(client)
        I ok == 1 {
            puts("WebSocket handshake OK")
            handle_client(client)
        } E {
            puts("Handshake failed")
        }

        __tcp_close(client)
    }

    __tcp_close(listener)
    0
}
```

## 빌드 및 테스트

```bash
# IR 생성
vaisc --emit-ir examples/tutorial_ws_chat.vais

# C 런타임과 링크
clang -o ws_chat tutorial_ws_chat.ll std/http_runtime.c std/websocket_runtime.c

# 실행
./ws_chat

# 다른 터미널에서 테스트
websocat ws://127.0.0.1:9001
# "Hello!" 입력 → "Hello!" 에코 확인
```

---

## 핵심 개념 정리

| 개념 | Vais 문법 | 설명 |
|------|-----------|------|
| 외부 함수 | `N "C" { F name(...) }` | C 런타임 바인딩 |
| 상수 | `C NAME: type = value` | 컴파일 타임 상수 |
| 패턴 매칭 | `M expr { pattern => body }` | opcode 분기 처리 |
| Defer | `D expr` | 스코프 종료 시 자동 실행 |
| 가변 변수 | `x := mut value` | 루프 카운터, 플래그 |
| 타입 캐스팅 | `ptr as str` | 포인터↔타입 변환 |

## 다음 단계

- [HTTP Server 튜토리얼](./http-server.md) — REST API 서버 만들기
- [JSON Parser 튜토리얼](./json-parser.md) — 재귀 하강 파서 구현
- [WebSocket API Reference](../api/websocket.md) — 전체 WebSocket API
- [examples/websocket_example.vais](https://github.com/vaislang/vais/blob/main/examples/websocket_example.vais) — 완전한 에코 서버 예제
