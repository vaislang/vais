# Vais Async/Await 고급 튜토리얼

이 튜토리얼은 Vais의 비동기 프로그래밍을 깊이 있게 다룹니다. 기본 튜토리얼을 먼저 학습하신 후 이 문서를 읽으시길 권장합니다.

## 목차

1. [비동기 프로그래밍 개념](#비동기-프로그래밍-개념)
2. [Async 함수 정의](#async-함수-정의)
3. [Await 키워드](#await-키워드)
4. [Future Trait와 Poll](#future-trait와-poll)
5. [Spawn과 동시성](#spawn과-동시성)
6. [비동기 에러 처리](#비동기-에러-처리)
7. [실전 예제](#실전-예제)
8. [성능 최적화](#성능-최적화)

---

## 비동기 프로그래밍 개념

### 동기 vs 비동기

**동기 프로그래밍:**
```vais
F fetch_data(url: str) -> str {
    # 네트워크 요청이 완료될 때까지 블로킹
    # 다른 작업을 수행할 수 없음
    "data from server"
}

F main() -> i64 {
    data1 := fetch_data("url1")  # 대기...
    data2 := fetch_data("url2")  # 또 대기...
    # 순차적으로 실행, 느림
    0
}
```

**비동기 프로그래밍:**
```vais
A F fetch_data(url: str) -> str {
    # 비블로킹 - 다른 작업 수행 가능
    "data from server"
}

F main() -> i64 {
    # 두 요청을 동시에 처리 가능
    data1 := fetch_data("url1").await
    data2 := fetch_data("url2").await
    0
}
```

### Vais의 비동기 모델

Vais는 **stackless coroutine** 기반의 async/await 패턴을 사용합니다:

- **State Machine**: 각 async 함수는 상태 머신으로 컴파일됨
- **Zero-cost Abstraction**: 런타임 오버헤드 최소화
- **Cooperative Scheduling**: 명시적 await 포인트에서만 제어 양보

---

## Async 함수 정의

### 기본 Async 함수

`A` 키워드로 함수를 비동기로 선언합니다:

```vais
# 단순 비동기 함수
A F compute(x: i64) -> i64 {
    x * 2
}

# 비동기 함수는 Future를 반환
A F add_values(a: i64, b: i64) -> i64 {
    a + b
}
```

### 표현식 형태

간단한 비동기 함수는 표현식으로 작성 가능:

```vais
A F double(x: i64) -> i64 = x * 2

A F max(a: i64, b: i64) -> i64 = a > b ? a : b

A F square(x: i64) -> i64 = x * x
```

### Async 함수 시그니처

```vais
# 매개변수 없음
A F get_value() -> i64 {
    42
}

# 여러 매개변수
A F calculate(x: i64, y: i64, multiplier: i64) -> i64 {
    (x + y) * multiplier
}

# 구조체 반환
S Point { x: f64, y: f64 }

A F create_point(x: f64, y: f64) -> Point {
    Point { x: x, y: y }
}
```

### 중요 사항

- Async 함수는 **즉시 실행되지 않음**
- 호출 시 **Future 객체** 반환
- Future는 `.await`로 폴링해야 실행됨

```vais
F main() -> i64 {
    # 이 줄은 compute를 실행하지 않음, Future만 생성
    future := compute(21)

    # .await를 해야 실제로 실행됨
    result := future.await

    print_i64(result)  # 42
    0
}
```

---

## Await 키워드

### 기본 사용법

`.await`는 Future가 완료될 때까지 기다립니다:

```vais
A F fetch_user(id: i64) -> str {
    "User data"
}

F main() -> i64 {
    # fetch_user를 호출하고 결과를 기다림
    user := fetch_user(123).await

    puts(user)
    0
}
```

### Await 체이닝

여러 비동기 작업을 순차적으로 실행:

```vais
A F step1(x: i64) -> i64 {
    x + 10
}

A F step2(x: i64) -> i64 {
    x * 2
}

A F step3(x: i64) -> i64 {
    x - 5
}

F main() -> i64 {
    # 순차적으로 실행
    result1 := step1(5).await      # 15
    result2 := step2(result1).await  # 30
    result3 := step3(result2).await  # 25

    print_i64(result3)  # 25
    0
}
```

### Async 함수 내부의 Await

Async 함수 안에서 다른 async 함수를 호출:

```vais
A F fetch_data(id: i64) -> i64 {
    # 시뮬레이션: 데이터 가져오기
    id * 100
}

A F process_data(data: i64) -> i64 {
    data + 42
}

A F fetch_and_process(id: i64) -> i64 {
    # 비동기 함수 내부에서 await 사용
    raw_data := fetch_data(id).await
    processed := process_data(raw_data).await
    processed
}

F main() -> i64 {
    result := fetch_and_process(5).await
    print_i64(result)  # 542 (5*100 + 42)
    0
}
```

---

## Future Trait와 Poll

### Future Trait 이해하기

Vais의 Future는 `std/future` 모듈에 정의되어 있습니다:

```vais
U std/future

# Poll 결과: Ready 또는 Pending
E Poll {
    Pending,      # 아직 준비 안 됨
    Ready(i64)    # 값 준비 완료
}

# Future trait - 비동기 값의 인터페이스
W Future {
    F poll(&self, ctx: i64) -> Poll
}
```

### Poll의 동작 방식

Future는 **상태 머신**으로 구현됩니다:

```vais
A F simple_async(x: i64) -> i64 {
    x * 2
}

# 컴파일러가 생성하는 state machine (개념적 표현):
S SimpleFuture {
    x: i64,
    state: i64  # 0 = 시작, 1 = 완료
}

X SimpleFuture: Future {
    F poll(&self, ctx: i64) -> Poll {
        I self.state == 0 {
            # 계산 수행
            self.state = 1
            result := self.x * 2
            Ready(result)
        } E {
            # 이미 완료됨
            Ready(0)
        }
    }
}
```

### Context와 Waker

Context는 런타임과의 통신을 위한 객체:

```vais
# Context - async 런타임 컨텍스트
S Context {
    waker_ptr: i64,
    runtime_ptr: i64
}

X Context {
    F new() -> Context {
        Context { waker_ptr: 0, runtime_ptr: 0 }
    }

    F wake(&self) -> i64 {
        # Task를 깨우기 (런타임에 알림)
        1
    }
}
```

### 커스텀 Future 구현 예제

직접 Future를 구현하는 방법:

```vais
U std/future

# 카운트다운 Future
S CountdownFuture {
    count: i64,
    current: i64
}

X CountdownFuture {
    F new(count: i64) -> CountdownFuture {
        CountdownFuture { count: count, current: 0 }
    }
}

X CountdownFuture: Future {
    F poll(&self, ctx: i64) -> Poll {
        I self.current >= self.count {
            # 완료
            Ready(self.count)
        } E {
            # 아직 진행 중
            self.current = self.current + 1
            Pending
        }
    }
}

F main() -> i64 {
    countdown := CountdownFuture::new(5)

    # await하면 poll이 Ready를 반환할 때까지 반복
    result := countdown.await

    print_i64(result)  # 5
    0
}
```

---

## Spawn과 동시성

### Spawn으로 태스크 생성

`spawn`은 새로운 비동기 태스크를 생성하여 동시 실행을 가능하게 합니다:

```vais
A F task1(x: i64) -> i64 {
    puts("Task 1 running")
    x * 2
}

A F task2(x: i64) -> i64 {
    puts("Task 2 running")
    x + 10
}

F main() -> i64 {
    # 두 태스크를 동시에 실행
    future1 := spawn task1(5)
    future2 := spawn task2(3)

    # 결과 기다리기
    result1 := future1.await  # 10
    result2 := future2.await  # 13

    total := result1 + result2
    print_i64(total)  # 23
    0
}
```

### Spawn vs 직접 Await

**직접 await (순차 실행):**
```vais
F main() -> i64 {
    # 순차적으로 실행됨
    r1 := slow_task(1).await   # 먼저 완료 대기
    r2 := slow_task(2).await   # 그 다음 실행

    r1 + r2
}
```

**Spawn 사용 (병렬 실행):**
```vais
F main() -> i64 {
    # 동시에 시작
    f1 := spawn slow_task(1)
    f2 := spawn slow_task(2)

    # 둘 다 완료 대기
    r1 := f1.await
    r2 := f2.await

    r1 + r2
}
```

### 여러 태스크 동시 실행

```vais
A F compute_value(id: i64, multiplier: i64) -> i64 {
    id * multiplier
}

F main() -> i64 {
    puts("Spawning multiple tasks...")

    # 5개 태스크 동시 실행
    t1 := spawn compute_value(1, 10)
    t2 := spawn compute_value(2, 20)
    t3 := spawn compute_value(3, 30)
    t4 := spawn compute_value(4, 40)
    t5 := spawn compute_value(5, 50)

    # 모든 결과 수집
    r1 := t1.await  # 10
    r2 := t2.await  # 40
    r3 := t3.await  # 90
    r4 := t4.await  # 160
    r5 := t5.await  # 250

    total := r1 + r2 + r3 + r4 + r5

    puts("Total:")
    print_i64(total)  # 550
    0
}
```

---

## 비동기 에러 처리

### Option을 사용한 에러 처리

```vais
U std/option

A F safe_divide(a: i64, b: i64) -> Option {
    I b == 0 {
        None
    } E {
        Some(a / b)
    }
}

F main() -> i64 {
    result := safe_divide(10, 2).await

    M result {
        Some(value) => {
            puts("Result:")
            print_i64(value)  # 5
        },
        None => {
            puts("Error: division by zero")
        }
    }

    0
}
```

### Result를 사용한 에러 처리

```vais
E Result {
    Ok(i64),
    Err(str)
}

A F validate_and_compute(x: i64) -> Result {
    I x < 0 {
        Err("Negative value not allowed")
    } E I x == 0 {
        Err("Zero value not allowed")
    } E {
        Ok(x * 2)
    }
}

F main() -> i64 {
    result := validate_and_compute(5).await

    M result {
        Ok(value) => {
            puts("Success:")
            print_i64(value)  # 10
        },
        Err(msg) => {
            puts("Error:")
            puts(msg)
        }
    }

    0
}
```

### 에러 전파 패턴

```vais
A F step_a(x: i64) -> Result {
    I x > 100 {
        Err("Value too large in step A")
    } E {
        Ok(x + 10)
    }
}

A F step_b(x: i64) -> Result {
    I x < 5 {
        Err("Value too small in step B")
    } E {
        Ok(x * 2)
    }
}

A F process_pipeline(x: i64) -> Result {
    # Step A 실행
    result_a := step_a(x).await

    M result_a {
        Err(msg) => Err(msg),  # 에러 전파
        Ok(val_a) => {
            # Step B 실행
            result_b := step_b(val_a).await
            result_b  # 결과 반환
        }
    }
}

F main() -> i64 {
    result := process_pipeline(10).await

    M result {
        Ok(value) => {
            puts("Pipeline result:")
            print_i64(value)  # 40 (10+10)*2
        },
        Err(msg) => {
            puts("Pipeline error:")
            puts(msg)
        }
    }

    0
}
```

---

## 실전 예제

### 예제 1: 비동기 데이터 처리 파이프라인

```vais
U std/option

# 데이터 가져오기
A F fetch_raw_data(id: i64) -> i64 {
    puts("Fetching data...")
    id * 100
}

# 데이터 검증
A F validate_data(data: i64) -> Option {
    I data < 0 {
        None
    } E {
        Some(data)
    }
}

# 데이터 변환
A F transform_data(data: i64) -> i64 {
    puts("Transforming data...")
    data + 42
}

# 데이터 저장
A F save_data(data: i64) -> i64 {
    puts("Saving data...")
    data
}

# 전체 파이프라인
A F data_pipeline(id: i64) -> Option {
    # 1. 데이터 가져오기
    raw := fetch_raw_data(id).await

    # 2. 검증
    validated := validate_data(raw).await

    M validated {
        None => None,
        Some(valid_data) => {
            # 3. 변환
            transformed := transform_data(valid_data).await

            # 4. 저장
            saved := save_data(transformed).await

            Some(saved)
        }
    }
}

F main() -> i64 {
    puts("=== Data Pipeline ===")
    putchar(10)

    result := data_pipeline(5).await

    M result {
        Some(value) => {
            puts("Pipeline success! Final value:")
            print_i64(value)  # 542
        },
        None => {
            puts("Pipeline failed!")
        }
    }

    0
}
```

### 예제 2: 동시 다운로드 시뮬레이션

```vais
A F download_file(file_id: i64, size: i64) -> i64 {
    puts("Downloading file")
    print_i64(file_id)
    putchar(10)

    # 다운로드 시간 시뮬레이션
    # 실제로는 네트워크 작업
    size * 10
}

A F process_file(file_id: i64, data: i64) -> i64 {
    puts("Processing file")
    print_i64(file_id)
    putchar(10)

    data + file_id
}

F main() -> i64 {
    puts("=== Concurrent Downloads ===")
    putchar(10)

    # 3개 파일 동시 다운로드
    d1 := spawn download_file(1, 100)
    d2 := spawn download_file(2, 200)
    d3 := spawn download_file(3, 150)

    # 다운로드 완료 대기
    data1 := d1.await  # 1000
    data2 := d2.await  # 2000
    data3 := d3.await  # 1500

    puts("All downloads complete!")
    putchar(10)

    # 각 파일 처리
    p1 := spawn process_file(1, data1)
    p2 := spawn process_file(2, data2)
    p3 := spawn process_file(3, data3)

    result1 := p1.await  # 1001
    result2 := p2.await  # 2002
    result3 := p3.await  # 1503

    total := result1 + result2 + result3

    puts("Total processed bytes:")
    print_i64(total)  # 4506
    putchar(10)

    0
}
```

### 예제 3: Async 재귀

```vais
# 비동기 팩토리얼
A F async_factorial(n: i64) -> i64 {
    I n <= 1 {
        1
    } E {
        prev := async_factorial(n - 1).await
        n * prev
    }
}

# 비동기 피보나치
A F async_fibonacci(n: i64) -> i64 {
    I n <= 1 {
        n
    } E {
        # 두 재귀 호출을 동시에 실행
        f1 := spawn async_fibonacci(n - 1)
        f2 := spawn async_fibonacci(n - 2)

        v1 := f1.await
        v2 := f2.await

        v1 + v2
    }
}

F main() -> i64 {
    puts("Async factorial(5):")
    fact := async_factorial(5).await
    print_i64(fact)  # 120
    putchar(10)

    puts("Async fibonacci(7):")
    fib := async_fibonacci(7).await
    print_i64(fib)  # 13
    putchar(10)

    0
}
```

---

## 성능 최적화

### 1. 불필요한 Await 제거

**나쁜 예:**
```vais
F main() -> i64 {
    # 각 작업을 순차적으로 기다림
    r1 := task1().await
    r2 := task2().await
    r3 := task3().await

    r1 + r2 + r3
}
```

**좋은 예:**
```vais
F main() -> i64 {
    # 모든 작업을 동시에 시작
    f1 := spawn task1()
    f2 := spawn task2()
    f3 := spawn task3()

    # 결과만 기다림
    r1 := f1.await
    r2 := f2.await
    r3 := f3.await

    r1 + r2 + r3
}
```

### 2. 작업 단위 최적화

작업을 너무 작게 나누면 오버헤드 증가:

```vais
# 너무 세분화 (비효율)
A F add_one(x: i64) -> i64 = x + 1

F bad_example() -> i64 {
    r := add_one(1).await
    r = add_one(r).await
    r = add_one(r).await
    r  # 3
}

# 적절한 크기
A F add_three(x: i64) -> i64 = x + 3

F good_example() -> i64 {
    add_three(0).await  # 3
}
```

### 3. 상태 머신 크기 최소화

Async 함수의 상태는 메모리에 저장됩니다:

```vais
# 큰 상태 (비효율)
A F large_state() -> i64 {
    x1 := compute1().await
    x2 := compute2().await
    x3 := compute3().await
    # 모든 변수가 상태에 저장됨
    x1 + x2 + x3
}

# 작은 상태 (효율적)
A F small_state() -> i64 {
    sum := 0
    sum = sum + compute1().await
    sum = sum + compute2().await
    sum = sum + compute3().await
    # 하나의 변수만 상태에 저장
    sum
}
```

### 4. Future 재사용

```vais
# Future를 여러 번 await하지 말 것
F main() -> i64 {
    future := expensive_task()

    # 나쁜 예: 여러 번 await
    # r1 := future.await  # 첫 실행
    # r2 := future.await  # 에러 또는 잘못된 동작

    # 좋은 예: 한 번만 await하고 결과 저장
    result := future.await
    use_result(result)
    use_result(result)

    0
}
```

---

## 요약

### 핵심 개념

1. **Async 함수**: `A F` 키워드로 정의, Future 반환
2. **Await**: `.await`로 Future 완료 대기
3. **Poll**: Future는 상태 머신으로 구현됨
4. **Spawn**: 동시 태스크 실행
5. **에러 처리**: Option/Result와 패턴 매칭

### 베스트 프랙티스

- ✅ 독립적인 작업은 `spawn`으로 병렬화
- ✅ 에러는 Option/Result로 명시적 처리
- ✅ 상태 머신 크기 최소화
- ✅ Future는 한 번만 await
- ❌ 너무 작은 단위로 async 분할하지 말 것
- ❌ 순차 실행이 필요한 경우에만 순차 await

### 다음 단계

- **std/future** 모듈 살펴보기
- **네트워크 프로그래밍** (std/net 사용)
- **타이머와 스케줄링**
- **동시성 패턴** (Fan-out, Pipeline 등)

---

## 참고 자료

- **기본 튜토리얼**: `TUTORIAL.md`
- **언어 스펙**: `LANGUAGE_SPEC.md`
- **표준 라이브러리**: `STDLIB.md`
- **예제 코드**: `examples/async_test.vais`, `examples/spawn_test.vais`

---

Happy async coding with Vais!
