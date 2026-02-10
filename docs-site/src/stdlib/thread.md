# Thread

## 개요

Thread 모듈은 OS 레벨 스레드 생성, 조인, 스레드 로컬 저장소(TLS)를 제공합니다. POSIX pthread 또는 Windows thread API를 래핑하며, 공유 메모리 멀티스레딩을 지원합니다.

## Quick Start

```vais
U std/thread

F worker(arg: i64) -> i64 {
    print_i64(arg)
    R arg * 2
}

F main() -> i64 {
    handle := thread_spawn(worker, 42)
    result := thread_join(handle)
    print_i64(result)  # 84
    R 0
}
```

## API 요약

| 함수 | 설명 |
|------|------|
| `thread_spawn(fn, arg)` | 새 스레드 생성 및 시작 |
| `thread_join(handle)` | 스레드 종료 대기 및 결과 반환 |
| `thread_detach(handle)` | 스레드 분리 (백그라운드 실행) |
| `thread_sleep(ms)` | 현재 스레드를 지정 시간 대기 |
| `thread_yield()` | 스케줄러에 CPU 양보 |
| `thread_current_id()` | 현재 스레드 ID 반환 |

## 실용 예제

### 예제 1: 병렬 계산

```vais
U std/thread
U std/vec

F compute_sum(range_ptr: i64) -> i64 {
    start := load_i64(range_ptr)
    end := load_i64(range_ptr + 8)

    sum := 0
    i := start
    L i < end {
        sum = sum + i
        i = i + 1
    }
    R sum
}

F parallel_sum(n: i64) -> i64 {
    threads := Vec::new()

    # 4개 스레드로 분할
    chunk := n / 4
    i := 0
    L i < 4 {
        range := malloc(16)
        store_i64(range, i * chunk)
        store_i64(range + 8, (i + 1) * chunk)

        handle := thread_spawn(compute_sum, range)
        threads.push(handle)
        i = i + 1
    }

    # 결과 합산
    total := 0
    i = 0
    L i < threads.len() {
        handle := threads.get(i)
        result := thread_join(handle)
        total = total + result
        i = i + 1
    }

    R total
}

F main() -> i64 {
    result := parallel_sum(1000000)
    print_i64(result)
    R 0
}
```

### 예제 2: 백그라운드 작업 (Detached Thread)

```vais
U std/thread

F background_logger(msg: i64) -> i64 {
    L 1 {
        print_str(msg)
        thread_sleep(1000)  # 1초마다
    }
    R 0
}

F main() -> i64 {
    handle := thread_spawn(background_logger, "서버 실행 중...")
    thread_detach(handle)  # 조인 없이 분리

    # 메인 작업 계속
    L i < 10 {
        print_i64(i)
        thread_sleep(500)
        i = i + 1
    }

    R 0
}
```

### 예제 3: 공유 상태 + Mutex

```vais
U std/thread
U std/sync

S SharedCounter {
    mutex: Mutex<i64>
}

F increment_worker(counter_ptr: i64) -> i64 {
    counter := load_typed(counter_ptr)
    i := 0
    L i < 1000 {
        guard := counter.mutex.lock()
        value := guard.get_inner()
        store_i64(value, load_i64(value) + 1)
        # guard 소멸 시 자동 unlock
        i = i + 1
    }
    R 0
}

F main() -> i64 {
    counter := SharedCounter {
        mutex: Mutex::new(0)
    }

    # 10개 스레드 생성
    threads := Vec::new()
    i := 0
    L i < 10 {
        handle := thread_spawn(increment_worker, &counter)
        threads.push(handle)
        i = i + 1
    }

    # 모두 대기
    i = 0
    L i < threads.len() {
        thread_join(threads.get(i))
        i = i + 1
    }

    # 최종 값 확인
    guard := counter.mutex.lock()
    print_i64(load_i64(guard.get_inner()))  # 10000
    R 0
}
```

### 예제 4: 스레드 풀 패턴

```vais
U std/thread
U std/channel
U std/vec

S ThreadPool {
    workers: Vec<i64>,
    sender: Sender<i64>
}

F worker_thread(receiver_ptr: i64) -> i64 {
    receiver := load_typed(receiver_ptr)

    L 1 {
        task := receiver.recv()
        I task == 0 { B }  # 종료 신호

        # 작업 실행
        result := task()
        print_i64(result)
    }
    R 0
}

X ThreadPool {
    F new(size: i64) -> ThreadPool {
        channel := channel_new(100)
        workers := Vec::new()

        i := 0
        L i < size {
            handle := thread_spawn(worker_thread, &channel.receiver)
            workers.push(handle)
            i = i + 1
        }

        ThreadPool {
            workers: workers,
            sender: channel.sender
        }
    }

    F submit(&self, task: i64) {
        self.sender.send(task)
    }

    F shutdown(&self) {
        # 종료 신호 전송
        i := 0
        L i < self.workers.len() {
            self.sender.send(0)
            i = i + 1
        }

        # 모든 스레드 조인
        i = 0
        L i < self.workers.len() {
            thread_join(self.workers.get(i))
            i = i + 1
        }
    }
}
```

### 예제 5: 스레드 로컬 저장소 (TLS)

```vais
U std/thread

# 스레드별 고유 ID 저장
global tls_key := 0

F init_tls() -> i64 {
    tls_key = __tls_create()
    R 0
}

F set_thread_id(id: i64) {
    __tls_set(tls_key, id)
}

F get_thread_id() -> i64 {
    R __tls_get(tls_key)
}

F worker(id: i64) -> i64 {
    set_thread_id(id)
    thread_sleep(100)

    my_id := get_thread_id()
    print_i64(my_id)  # 각 스레드는 고유 ID 출력
    R 0
}

F main() -> i64 {
    init_tls()

    i := 0
    L i < 5 {
        thread_spawn(worker, i)
        i = i + 1
    }

    thread_sleep(1000)  # 모든 스레드 대기
    R 0
}
```

## 주의사항

### 1. 스레드 조인 필수
`thread_spawn()`으로 생성한 스레드는 반드시 `thread_join()` 또는 `thread_detach()`를 호출하세요. 그렇지 않으면 리소스 누수가 발생합니다.

```vais
# 나쁜 예
handle := thread_spawn(worker, arg)
# join/detach 없음!

# 좋은 예 1: 결과 대기
handle := thread_spawn(worker, arg)
result := thread_join(handle)

# 좋은 예 2: 백그라운드 실행
handle := thread_spawn(worker, arg)
thread_detach(handle)
```

### 2. 공유 메모리 경쟁 조건
여러 스레드가 같은 메모리를 동시에 쓰면 데이터 경쟁(Data Race)이 발생합니다. 반드시 `Mutex` 또는 `RwLock`으로 보호하세요.

```vais
# 나쁜 예: 경쟁 조건
global counter := 0

F bad_worker() -> i64 {
    counter = counter + 1  # 원자성 없음!
    R 0
}

# 좋은 예: Mutex 보호
global safe_counter := Mutex::new(0)

F good_worker() -> i64 {
    guard := safe_counter.lock()
    val := guard.get_inner()
    store_i64(val, load_i64(val) + 1)
    R 0
}
```

### 3. Deadlock 방지
여러 Mutex를 동시에 잠글 때는 항상 같은 순서로 잠그세요.

```vais
# Deadlock 발생 가능
F thread1() {
    lock_a := mutex_a.lock()
    lock_b := mutex_b.lock()  # A → B 순서
}

F thread2() {
    lock_b := mutex_b.lock()
    lock_a := mutex_a.lock()  # B → A 순서 (Deadlock!)
}

# 안전한 방법: 일관된 순서
F safe_thread() {
    lock_a := mutex_a.lock()  # 항상 A → B
    lock_b := mutex_b.lock()
}
```

### 4. 스레드 수 제한
OS는 프로세스당 스레드 수를 제한합니다 (Linux 기본 ~1000). 대량 동시 작업은 스레드 풀을 사용하세요.

```vais
# 나쁜 예: 10,000개 스레드 생성
L i < 10000 {
    thread_spawn(worker, i)
}

# 좋은 예: 스레드 풀 재사용
pool := ThreadPool::new(8)  # 8개 워커 스레드
L i < 10000 {
    pool.submit(task)
}
```

### 5. 스레드 간 에러 전파
스레드 함수는 i64를 반환하므로, 에러 코드를 전달해야 합니다.

```vais
F worker(arg: i64) -> i64 {
    I arg < 0 {
        R -1  # 에러 코드
    }
    R compute(arg)
}

F main() -> i64 {
    handle := thread_spawn(worker, -5)
    result := thread_join(handle)

    I result < 0 {
        print_str("스레드 에러 발생")
    }
    R 0
}
```

### 6. 스레드 안전 함수
Vais 표준 라이브러리의 대부분 함수는 thread-safe하지 않습니다. 특히:
- `malloc()`/`free()`: 대부분의 구현은 thread-safe
- `print_str()`: 출력 순서가 섞일 수 있음
- `HashMap`/`Vec`: Mutex로 보호 필요

### 7. 스레드 스케줄링
`thread_yield()`는 현재 스레드의 CPU 타임슬라이스를 포기하고 다른 스레드에 양보합니다. Busy-waiting 루프에서 유용합니다.

```vais
# 나쁜 예: CPU 100% 사용
L !ready {
    # busy-wait
}

# 좋은 예: CPU 양보
L !ready {
    thread_yield()
}
```

### 8. 플랫폼 차이
- POSIX: `pthread_create()`, `pthread_join()`
- Windows: `CreateThread()`, `WaitForSingleObject()`

대부분의 동작은 동일하지만, TLS API와 우선순위 설정은 플랫폼마다 다릅니다.

## See Also

- [Thread API Reference](../api/thread.md)
- [Sync Primitives](./sync.md)
- [Channel Communication](./channel.md)
- [Async Programming](../language/async-tutorial.md)
- [Concurrency Overview](../api/sync.md)
