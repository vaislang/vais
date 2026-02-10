# Channel

## 개요

Channel은 CSP(Communicating Sequential Processes) 스타일의 스레드 간 메시지 전달을 제공합니다. Bounded(제한 용량) 및 Unbounded(무제한) 채널, 멀티 채널 select를 지원하며, Mutex/Condvar로 구현됩니다.

## Quick Start

```vais
U std/channel

F main() -> i64 {
    ch := channel_new(10)
    ch.send(42)
    value := ch.recv()
    print_i64(value)  # 42
    R 0
}
```

## API 요약

| 함수 | 설명 |
|------|------|
| `channel_new(capacity)` | Bounded 채널 생성 |
| `unbounded_channel_new()` | Unbounded 채널 생성 |
| `send(value)` | 메시지 전송 (블로킹) |
| `recv()` | 메시지 수신 (블로킹) |
| `try_send(value)` | 논블로킹 전송 (실패 시 0 반환) |
| `try_recv()` | 논블로킹 수신 (없으면 0 반환) |
| `channel_select(chans, count)` | 여러 채널 중 하나 선택 |
| `close()` | 채널 닫기 |

## 실용 예제

### 예제 1: Producer-Consumer 패턴

```vais
U std/channel
U std/thread

F producer(ch_ptr: i64) -> i64 {
    sender := load_typed(ch_ptr)
    i := 0
    L i < 10 {
        sender.send(i)
        print_str("생산: ~{i}")
        thread_sleep(100)
        i = i + 1
    }
    sender.send(-1)  # 종료 신호
    R 0
}

F consumer(ch_ptr: i64) -> i64 {
    receiver := load_typed(ch_ptr)
    L 1 {
        value := receiver.recv()
        I value < 0 { B }  # 종료
        print_str("소비: ~{value}")
        thread_sleep(200)
    }
    R 0
}

F main() -> i64 {
    ch := channel_new(5)
    thread_spawn(producer, &ch.sender)
    thread_spawn(consumer, &ch.receiver)
    thread_sleep(3000)
    R 0
}
```

### 예제 2: 작업 큐 (Work Queue)

```vais
U std/channel
U std/thread
U std/vec

S Task {
    id: i64,
    data: i64
}

F worker(ch_ptr: i64, worker_id: i64) -> i64 {
    receiver := load_typed(ch_ptr)

    L 1 {
        task := receiver.recv()
        I task.id == 0 { B }  # 종료 신호

        print_str("워커 ~{worker_id} 작업 ~{task.id}")
        thread_sleep(500)  # 작업 시뮬레이션
    }
    R 0
}

F main() -> i64 {
    ch := channel_new(20)

    # 3개 워커 스레드 생성
    i := 0
    L i < 3 {
        thread_spawn(worker, &ch.receiver, i)
        i = i + 1
    }

    # 10개 작업 전송
    i = 1
    L i <= 10 {
        task := Task { id: i, data: i * 100 }
        ch.sender.send(task)
        i = i + 1
    }

    # 종료 신호 (워커 수만큼)
    i = 0
    L i < 3 {
        ch.sender.send(Task { id: 0, data: 0 })
        i = i + 1
    }

    thread_sleep(5000)
    R 0
}
```

### 예제 3: 채널 Select (멀티플렉싱)

```vais
U std/channel

F main() -> i64 {
    ch1 := channel_new(1)
    ch2 := channel_new(1)

    # 두 채널에 데이터 전송
    spawn {
        thread_sleep(100)
        ch1.sender.send(10)
    }

    spawn {
        thread_sleep(200)
        ch2.sender.send(20)
    }

    # 먼저 도착하는 메시지 수신
    channels := Vec::new()
    channels.push(&ch1.receiver)
    channels.push(&ch2.receiver)

    idx := channel_select(channels.data, 2)
    I idx == 0 {
        print_str("ch1 수신: ~{ch1.receiver.recv()}")
    } E I idx == 1 {
        print_str("ch2 수신: ~{ch2.receiver.recv()}")
    }

    R 0
}
```

### 예제 4: Unbounded 채널 (동적 크기)

```vais
U std/channel

F fast_producer(ch_ptr: i64) -> i64 {
    sender := load_typed(ch_ptr)
    i := 0
    L i < 1000 {
        sender.send(i)  # 버퍼 full 걱정 없음
        i = i + 1
    }
    R 0
}

F slow_consumer(ch_ptr: i64) -> i64 {
    receiver := load_typed(ch_ptr)
    i := 0
    L i < 1000 {
        value := receiver.recv()
        thread_sleep(10)  # 느린 처리
        i = i + 1
    }
    R 0
}

F main() -> i64 {
    ch := unbounded_channel_new()
    thread_spawn(fast_producer, &ch.sender)
    thread_spawn(slow_consumer, &ch.receiver)
    thread_sleep(15000)
    R 0
}
```

### 예제 5: 논블로킹 송수신

```vais
U std/channel

F main() -> i64 {
    ch := channel_new(2)

    # 논블로킹 전송
    success := ch.sender.try_send(10)
    print_i64(success)  # 1 (성공)

    ch.sender.try_send(20)
    ch.sender.try_send(30)  # 버퍼 full (capacity=2)
    success = ch.sender.try_send(40)
    print_i64(success)  # 0 (실패)

    # 논블로킹 수신
    value := ch.receiver.try_recv()
    I value != 0 {
        print_i64(value)  # 10
    }

    R 0
}
```

## 주의사항

### 1. Deadlock 위험
모든 Sender가 닫히지 않으면 Receiver가 영원히 블로킹됩니다.

```vais
# Deadlock 발생
ch := channel_new(1)
value := ch.receiver.recv()  # 영원히 대기 (아무도 send 안 함)

# 해결: 타임아웃 또는 별도 스레드에서 send
spawn { ch.sender.send(42) }
value = ch.receiver.recv()
```

### 2. Bounded vs Unbounded
- **Bounded**: 고정 용량, 버퍼 full 시 sender 블로킹, 메모리 예측 가능
- **Unbounded**: 무제한 용량, sender 절대 블로킹, 메모리 폭발 위험

```vais
# Bounded: 백프레셔(Backpressure) 제공
ch := channel_new(10)
L i < 1000 {
    ch.send(i)  # 버퍼 full이면 대기 (consumer가 따라잡을 때까지)
}

# Unbounded: 메모리 누수 위험
ch := unbounded_channel_new()
L i < 1000000 {
    ch.send(i)  # 계속 누적 (consumer가 느리면 메모리 부족)
}
```

### 3. 채널 닫기
`close()`를 호출하면 더 이상 send할 수 없습니다. Receiver는 남은 메시지를 모두 수신 후 0 반환합니다.

```vais
ch := channel_new(5)
ch.send(1)
ch.send(2)
ch.close()

# ch.send(3)  # 패닉 또는 무시됨

print_i64(ch.recv())  # 1
print_i64(ch.recv())  # 2
print_i64(ch.recv())  # 0 (채널 닫힘)
```

### 4. Select의 공정성
`channel_select()`는 첫 번째 준비된 채널을 선택합니다. 여러 채널이 동시에 준비되면 인덱스 순서대로 우선합니다.

```vais
# ch1과 ch2 모두 준비 → ch1 선택됨 (인덱스 0)
idx := channel_select(&[ch1, ch2], 2)
```

### 5. 메모리 관리
Channel 내부 버퍼는 동적 할당됩니다. 프로그램 종료 전 `free()`를 호출하거나 GC에 의존하세요.

### 6. MPMC (Multiple Producer Multiple Consumer)
Vais Channel은 MPMC를 지원합니다. 여러 스레드가 동시에 send/recv 가능합니다.

```vais
ch := channel_new(10)

# 3개 producer
L i < 3 {
    spawn { L j < 100 { ch.send(j) } }
}

# 2개 consumer
L i < 2 {
    spawn { L j < 150 { ch.recv() } }
}
```

### 7. 타임아웃 패턴
현재 구현은 타임아웃 내장 함수가 없습니다. Select와 timer 채널을 조합하세요.

```vais
F recv_with_timeout(ch: &Receiver<i64>, ms: i64) -> i64? {
    timer := channel_new(1)

    spawn {
        thread_sleep(ms)
        timer.send(0)  # 타임아웃 신호
    }

    idx := channel_select(&[ch, &timer.receiver], 2)
    I idx == 0 {
        R Some(ch.recv())
    } E {
        R None  # 타임아웃
    }
}
```

### 8. 채널 용량 0 (Rendezvous)
Capacity=0인 채널은 Sender와 Receiver가 동시에 만날 때만 전송됩니다.

```vais
ch := channel_new(0)  # Rendezvous channel

spawn { ch.send(42) }  # Receiver 대기
value := ch.recv()     # Sender와 동기화
```

## See Also

- [Channel API Reference](../api/channel.md)
- [Thread Documentation](./thread.md)
- [Sync Primitives](./sync.md)
- [Async Channels](../api/async.md)
- [CSP in Vais](../guide/concurrency-patterns.md)
