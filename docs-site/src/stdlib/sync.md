# Sync

## 개요

Sync 모듈은 동시성 프로그래밍을 위한 동기화 프리미티브를 제공합니다. Mutex(상호 배제), RwLock(읽기-쓰기 락), Condvar(조건 변수), Barrier(장벽), Semaphore(세마포어)를 포함하며, OS 네이티브 구현을 래핑합니다.

## Quick Start

```vais
U std/sync

F main() -> i64 {
    counter := Mutex::new(0)
    guard := counter.lock()
    val := guard.get_inner()
    store_i64(val, 42)
    R 0
}
```

## API 요약

### Mutex

| 함수 | 설명 |
|------|------|
| `Mutex::new(value)` | 새 Mutex 생성 |
| `lock()` | 락 획득 (블로킹) |
| `try_lock()` | 논블로킹 락 시도 |
| `is_locked()` | 락 상태 확인 |

### RwLock

| 함수 | 설명 |
|------|------|
| `RwLock::new(value)` | 새 RwLock 생성 |
| `read()` | 읽기 락 획득 (여러 reader 허용) |
| `write()` | 쓰기 락 획득 (독점) |
| `try_read()` | 논블로킹 읽기 락 시도 |
| `try_write()` | 논블로킹 쓰기 락 시도 |

### Condvar

| 함수 | 설명 |
|------|------|
| `Condvar::new()` | 새 Condvar 생성 |
| `wait(mutex)` | 신호 대기 (mutex 자동 unlock) |
| `notify_one()` | 하나의 대기 스레드 깨우기 |
| `notify_all()` | 모든 대기 스레드 깨우기 |

### Barrier

| 함수 | 설명 |
|------|------|
| `Barrier::new(count)` | N개 스레드용 Barrier 생성 |
| `wait()` | 모든 스레드 도착 대기 |

### Semaphore

| 함수 | 설명 |
|------|------|
| `Semaphore::new(permits)` | 허가 개수 지정 생성 |
| `acquire()` | 허가 획득 (블로킹) |
| `release()` | 허가 반환 |
| `try_acquire()` | 논블로킹 획득 시도 |

## 실용 예제

### 예제 1: Mutex로 공유 상태 보호

```vais
U std/sync
U std/thread

global counter := Mutex::new(0)

F increment() -> i64 {
    i := 0
    L i < 1000 {
        guard := counter.lock()
        val := guard.get_inner()
        store_i64(val, load_i64(val) + 1)
        i = i + 1
    }
    R 0
}

F main() -> i64 {
    threads := Vec::new()
    i := 0
    L i < 10 {
        handle := thread_spawn(increment, 0)
        threads.push(handle)
        i = i + 1
    }

    i = 0
    L i < threads.len() {
        thread_join(threads.get(i))
        i = i + 1
    }

    guard := counter.lock()
    print_i64(load_i64(guard.get_inner()))  # 10000
    R 0
}
```

### 예제 2: RwLock으로 읽기 병렬화

```vais
U std/sync
U std/thread

global cache := RwLock::new(HashMap::new())

F reader(id: i64) -> i64 {
    L i < 100 {
        guard := cache.read()  # 여러 reader 동시 접근 가능
        data := guard.get_inner()
        value := data.get(42)
        print_str("Reader ~{id}: ~{value}")
        thread_sleep(10)
        i = i + 1
    }
    R 0
}

F writer() -> i64 {
    L i < 10 {
        guard := cache.write()  # 독점 쓰기
        data := guard.get_inner()
        data.set(42, i)
        thread_sleep(100)
        i = i + 1
    }
    R 0
}

F main() -> i64 {
    # 5개 reader + 1개 writer
    L i < 5 {
        thread_spawn(reader, i)
        i = i + 1
    }
    thread_spawn(writer, 0)

    thread_sleep(2000)
    R 0
}
```

### 예제 3: Condvar로 이벤트 대기

```vais
U std/sync
U std/thread

S EventQueue {
    mutex: Mutex<Vec<i64>>,
    condvar: Condvar
}

F producer(queue_ptr: i64) -> i64 {
    queue := load_typed(queue_ptr)
    i := 0
    L i < 10 {
        guard := queue.mutex.lock()
        events := guard.get_inner()
        events.push(i)

        # 대기 중인 consumer 깨우기
        queue.condvar.notify_one()

        thread_sleep(100)
        i = i + 1
    }
    R 0
}

F consumer(queue_ptr: i64) -> i64 {
    queue := load_typed(queue_ptr)

    L 1 {
        guard := queue.mutex.lock()
        events := guard.get_inner()

        # 이벤트 없으면 대기
        L events.is_empty() {
            queue.condvar.wait(&queue.mutex)
        }

        event := events.remove(0)
        print_str("소비: ~{event}")

        I event >= 9 { B }  # 종료 조건
    }
    R 0
}

F main() -> i64 {
    queue := EventQueue {
        mutex: Mutex::new(Vec::new()),
        condvar: Condvar::new()
    }

    thread_spawn(producer, &queue)
    thread_spawn(consumer, &queue)
    thread_sleep(2000)
    R 0
}
```

### 예제 4: Barrier로 스레드 동기화

```vais
U std/sync
U std/thread

global barrier := Barrier::new(3)

F phase_worker(id: i64) -> i64 {
    # Phase 1
    print_str("스레드 ~{id}: Phase 1 시작")
    thread_sleep(id * 100)
    print_str("스레드 ~{id}: Phase 1 완료")

    barrier.wait()  # 모든 스레드 대기

    # Phase 2 (모두 동시 시작)
    print_str("스레드 ~{id}: Phase 2 시작")
    thread_sleep(id * 100)
    print_str("스레드 ~{id}: Phase 2 완료")

    barrier.wait()

    # Phase 3
    print_str("스레드 ~{id}: Phase 3 시작")
    R 0
}

F main() -> i64 {
    thread_spawn(phase_worker, 0)
    thread_spawn(phase_worker, 1)
    thread_spawn(phase_worker, 2)

    thread_sleep(2000)
    R 0
}
```

### 예제 5: Semaphore로 리소스 제한

```vais
U std/sync
U std/thread

# 동시 접근 최대 3개로 제한
global db_pool := Semaphore::new(3)

F database_query(id: i64) -> i64 {
    print_str("~{id}: 연결 대기 중...")
    db_pool.acquire()  # 허가 획득

    print_str("~{id}: 쿼리 실행 중")
    thread_sleep(500)  # DB 작업 시뮬레이션

    db_pool.release()  # 허가 반환
    print_str("~{id}: 연결 해제")
    R 0
}

F main() -> i64 {
    # 10개 스레드가 3개 연결 공유
    i := 0
    L i < 10 {
        thread_spawn(database_query, i)
        i = i + 1
    }

    thread_sleep(5000)
    R 0
}
```

## 주의사항

### 1. Lock 순서 (Deadlock 방지)
여러 Mutex를 동시에 잠글 때는 일관된 순서를 유지하세요.

```vais
# Deadlock 발생 가능
F thread1() {
    lock_a := mutex_a.lock()
    lock_b := mutex_b.lock()  # A → B
}

F thread2() {
    lock_b := mutex_b.lock()
    lock_a := mutex_a.lock()  # B → A (Deadlock!)
}

# 안전한 방법
F safe_order() {
    lock_a := mutex_a.lock()  # 항상 A → B 순서
    lock_b := mutex_b.lock()
}
```

### 2. Guard 생명주기
`MutexGuard`는 소멸 시 자동으로 unlock합니다. Guard를 너무 오래 유지하면 다른 스레드가 블로킹됩니다.

```vais
# 나쁜 예: 락을 오래 유지
guard := mutex.lock()
expensive_computation()  # 다른 스레드 블로킹!
val := guard.get_inner()

# 좋은 예: 최소 임계 영역
temp := 0
{
    guard := mutex.lock()
    temp = load_i64(guard.get_inner())
}  # guard 소멸 → unlock

expensive_computation()
```

### 3. RwLock Writer Starvation
많은 reader가 계속 들어오면 writer가 영원히 대기할 수 있습니다. Writer 우선 정책을 고려하세요.

```vais
# Writer starvation 발생 가능
L 1 {
    guard := rwlock.read()  # Reader 계속 진입
    # Writer는 영원히 대기
}
```

### 4. Condvar Spurious Wakeup
`wait()`는 신호 없이 깰 수 있습니다(Spurious Wakeup). 항상 루프로 조건을 재확인하세요.

```vais
# 나쁜 예
I queue.is_empty() {
    condvar.wait(&mutex)
}

# 좋은 예: 루프로 재확인
L queue.is_empty() {
    condvar.wait(&mutex)
}
```

### 5. Barrier 재사용
Barrier는 한 번만 사용 가능한 구현도 있습니다. 재사용 시 새로 생성하거나, 명시적 reset을 호출하세요.

```vais
barrier := Barrier::new(3)

# Round 1
barrier.wait()

# Round 2 - 새 Barrier 필요할 수 있음
barrier = Barrier::new(3)
barrier.wait()
```

### 6. Semaphore 카운팅
`acquire()`와 `release()` 호출 횟수가 일치해야 합니다. 누락 시 리소스 고갈 또는 과다 허가가 발생합니다.

```vais
# 나쁜 예: release 누락
semaphore.acquire()
I error_occurred {
    R 1  # release 없이 리턴!
}
semaphore.release()

# 좋은 예: defer 사용
semaphore.acquire()
D semaphore.release()
I error_occurred { R 1 }
```

### 7. try_lock 패턴
`try_lock()`은 실패 시 None을 반환합니다. Polling loop에서 유용합니다.

```vais
L 1 {
    guard := mutex.try_lock()
    M guard {
        Some(g) => {
            # 락 획득 성공
            process(g.get_inner())
            B
        },
        None => {
            # 다른 작업 수행
            thread_yield()
        }
    }
}
```

### 8. 플랫폼 차이
- POSIX: `pthread_mutex_t`, `pthread_rwlock_t`, `pthread_cond_t`
- Windows: `CRITICAL_SECTION`, `SRWLOCK`, `CONDITION_VARIABLE`

대부분의 API는 동일하지만, 성능 특성이 다를 수 있습니다(예: Windows SRW는 fair lock).

### 9. Poison 처리
Rust와 달리 Vais Mutex는 panic poison을 지원하지 않습니다. 스레드 패닉 시 락이 영구 잠길 수 있으므로, 에러 처리를 철저히 하세요.

### 10. Atomic vs Mutex
간단한 카운터는 Atomic 연산이 더 효율적입니다.

```vais
# Mutex: 무겁지만 범용적
counter := Mutex::new(0)
guard := counter.lock()
store_i64(guard.get_inner(), load_i64(guard.get_inner()) + 1)

# Atomic: 가볍지만 제한적 (향후 지원)
counter := AtomicI64::new(0)
counter.fetch_add(1)
```

## See Also

- [Sync API Reference](../api/sync.md)
- [Thread Documentation](./thread.md)
- [Channel Communication](./channel.md)
- [Mutex API Reference](../api/sync.md#mutex)
- [RwLock API Reference](../api/sync.md#rwlock)
- [Concurrency Guide](../guide/concurrency.md)
