# Networking

## 개요

Net 모듈은 TCP/UDP 소켓을 위한 low-level 네트워킹 API를 제공합니다. BSD socket API를 래핑하며, IPv4/IPv6, 논블로킹 소켓, 소켓 옵션 설정을 지원합니다.

## Quick Start

```vais
U std/net

F main() -> i64 {
    server := TcpListener::bind("127.0.0.1", 8080)
    client := server.accept()
    client.send("Hello, client!")
    client.close()
    R 0
}
```

## API 요약

### TCP

| 함수 | 설명 |
|------|------|
| `TcpListener::bind(host, port)` | TCP 서버 소켓 생성 및 바인드 |
| `accept()` | 클라이언트 연결 수락 |
| `TcpStream::connect(host, port)` | TCP 클라이언트 연결 |
| `send(data, len)` | 데이터 전송 |
| `recv(buf, max_len)` | 데이터 수신 |
| `close()` | 소켓 닫기 |

### UDP

| 함수 | 설명 |
|------|------|
| `UdpSocket::bind(host, port)` | UDP 소켓 생성 및 바인드 |
| `send_to(data, len, host, port)` | 특정 주소로 전송 |
| `recv_from(buf, max_len)` | 데이터 수신 (sender 주소 반환) |

### Socket Options

| 함수 | 설명 |
|------|------|
| `set_nonblocking(enabled)` | 논블로킹 모드 설정 |
| `set_reuseaddr(enabled)` | SO_REUSEADDR 설정 |
| `set_timeout(ms)` | 타임아웃 설정 |

## 실용 예제

### 예제 1: Echo 서버

```vais
U std/net

F handle_client(client: TcpStream) -> i64 {
    buffer := malloc(1024)

    L 1 {
        bytes := client.recv(buffer, 1024)
        I bytes <= 0 { B }  # 연결 종료

        # Echo back
        client.send(buffer, bytes)
    }

    client.close()
    R 0
}

F main() -> i64 {
    server := TcpListener::bind("127.0.0.1", 8080)
    I !server.is_valid() {
        print_str("포트 바인드 실패")
        R 1
    }

    print_str("서버 시작: 8080")

    L 1 {
        client := server.accept()
        I client.is_valid() {
            handle_client(client)
        }
    }
    R 0
}
```

### 예제 2: HTTP 클라이언트 (간단한 GET 요청)

```vais
U std/net

F http_get(host: i64, path: i64) -> i64 {
    client := TcpStream::connect(host, 80)
    I !client.is_valid() {
        R 0
    }

    # HTTP 요청 전송
    client.send("GET ")
    client.send(path)
    client.send(" HTTP/1.1\r\n")
    client.send("Host: ")
    client.send(host)
    client.send("\r\n\r\n")

    # 응답 수신
    buffer := malloc(4096)
    bytes := client.recv(buffer, 4096)
    print_str(buffer)

    client.close()
    R 1
}

F main() -> i64 {
    http_get("example.com", "/")
    R 0
}
```

### 예제 3: 멀티스레드 서버

```vais
U std/net
U std/thread

F worker_thread(client_ptr: i64) -> i64 {
    client := load_typed(client_ptr)  # TcpStream 복원
    buffer := malloc(1024)

    L 1 {
        bytes := client.recv(buffer, 1024)
        I bytes <= 0 { B }
        client.send("Received: ")
        client.send(buffer, bytes)
    }

    client.close()
    R 0
}

F main() -> i64 {
    server := TcpListener::bind("0.0.0.0", 8080)
    server.set_reuseaddr(1)

    L 1 {
        client := server.accept()
        I client.is_valid() {
            # 클라이언트마다 스레드 생성
            thread_spawn(worker_thread, &client)
        }
    }
    R 0
}
```

### 예제 4: UDP 메시지 전송

```vais
U std/net

F udp_client() -> i64 {
    sock := UdpSocket::bind("0.0.0.0", 0)  # 임의 포트
    message := "Hello, UDP!"

    # 서버로 전송
    sock.send_to(message, strlen(message), "127.0.0.1", 9000)

    # 응답 수신
    buffer := malloc(512)
    sender_addr := malloc(64)
    sender_port := 0

    bytes := sock.recv_from(buffer, 512, sender_addr, &sender_port)
    print_str(buffer)

    sock.close()
    R 0
}

F udp_server() -> i64 {
    sock := UdpSocket::bind("0.0.0.0", 9000)
    buffer := malloc(512)
    sender_addr := malloc(64)

    L 1 {
        sender_port := 0
        bytes := sock.recv_from(buffer, 512, sender_addr, &sender_port)

        I bytes > 0 {
            # Echo back to sender
            sock.send_to(buffer, bytes, sender_addr, sender_port)
        }
    }
    R 0
}
```

### 예제 5: 논블로킹 소켓과 타임아웃

```vais
U std/net

F nonblocking_client(host: i64, port: i64) -> i64 {
    client := TcpStream::connect(host, port)
    I !client.is_valid() { R 1 }

    # 논블로킹 모드 + 5초 타임아웃
    client.set_nonblocking(1)
    client.set_timeout(5000)

    buffer := malloc(1024)
    bytes := client.recv(buffer, 1024)

    I bytes > 0 {
        print_str(buffer)
    } E I bytes == 0 {
        print_str("연결 종료")
    } E {
        print_str("타임아웃 또는 에러")
    }

    client.close()
    R 0
}
```

## 주의사항

### 1. 소켓 닫기 필수
네트워크 소켓은 파일 디스크립터를 소비합니다. 항상 `close()`를 호출하여 리소스를 반환하세요.

```vais
client := TcpStream::connect(host, port)
D client.close()  # defer로 자동 정리
# 작업 수행
```

### 2. 바이너리 데이터 처리
`send()`와 `recv()`는 바이너리 데이터를 처리합니다. 문자열 전송 시 길이를 명시하세요.

```vais
# 나쁜 예
client.send("Hello")  # strlen("Hello")이 자동으로 계산되지 않음!

# 좋은 예
msg := "Hello"
client.send(msg, strlen(msg))
```

### 3. 부분 전송/수신
`send()`는 버퍼 전체를 한 번에 보내지 못할 수 있습니다. 반환값을 확인하고 반복하세요.

```vais
F send_all(sock: TcpStream, data: i64, len: i64) -> i64 {
    sent := 0
    L sent < len {
        n := sock.send(data + sent, len - sent)
        I n <= 0 { R 0 }  # 실패
        sent = sent + n
    }
    R 1
}
```

### 4. IPv6 지원
`AF_INET6` 상수는 플랫폼마다 다릅니다. IPv6 주소는 `"::"` 형식으로 전달하세요.

```vais
# IPv6 서버
server := TcpListener::bind("::", 8080)  # 모든 IPv6 인터페이스

# IPv6 클라이언트
client := TcpStream::connect("::1", 8080)  # localhost IPv6
```

### 5. 타임아웃 에러 처리
`set_timeout()` 설정 후 `recv()`가 타임아웃되면 -1을 반환합니다. errno를 확인하여 구분하세요.

```vais
bytes := client.recv(buffer, 1024)
I bytes < 0 {
    # 타임아웃인지 다른 에러인지 확인
    I errno == EAGAIN || errno == EWOULDBLOCK {
        print_str("타임아웃")
    } E {
        print_str("네트워크 에러")
    }
}
```

### 6. SO_REUSEADDR 필수 (서버)
서버 프로그램을 재시작할 때 "Address already in use" 에러를 방지하려면 반드시 설정하세요.

```vais
server := TcpListener::bind(host, port)
server.set_reuseaddr(1)  # 필수!
```

### 7. 논블로킹 모드 주의
논블로킹 소켓에서 `recv()`는 데이터가 없으면 즉시 -1을 반환합니다. 이벤트 루프(epoll/kqueue)와 함께 사용하세요.

```vais
# 논블로킹 + 이벤트 루프 패턴
client.set_nonblocking(1)

L 1 {
    bytes := client.recv(buffer, 1024)
    I bytes > 0 {
        process_data(buffer, bytes)
    } E I bytes == 0 {
        B  # 연결 종료
    } E {
        # EAGAIN → 데이터 대기, 다른 에러 → 종료
        I errno != EAGAIN { B }
    }
}
```

### 8. 플랫폼별 상수 차이
`SOL_SOCKET`, `AF_INET6` 등의 값은 OS마다 다릅니다. `#[cfg(target_os)]` 속성으로 분기된 정의를 사용하세요.

## See Also

- Net API Reference
- HTTP API Reference
- WebSocket API Reference
- TLS API Reference
- Async Networking
