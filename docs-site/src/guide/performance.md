# 성능 튜닝 가이드

Vais 프로그램을 최적화하는 방법을 배웁니다. 컴파일 최적화, 벤치마킹, 프로파일링 기법을 다룹니다.

## 컴파일 최적화

### 최적화 수준 (Optimization Levels)

Vais 컴파일러는 LLVM을 기반으로 하여 여러 최적화 수준을 지원합니다:

```bash
# -O0: 최적화 없음 (기본값)
# - 빠른 컴파일
# - 큰 바이너리 크기
# - 느린 실행 속도
# - 디버깅 정보 유지
vaisc build -O0 myprogram.vais -o myprogram

# -O1: 기본 최적화
# - 균형잡힌 컴파일 시간과 성능
# - 중간 크기의 바이너리
vaisc build -O1 myprogram.vais -o myprogram

# -O2: 높은 최적화 (권장)
# - 우수한 성능
# - 합리적인 컴파일 시간
# - 더 큰 바이너리
vaisc build -O2 myprogram.vais -o myprogram

# -O3: 최대 최적화
# - 최고의 성능
# - 긴 컴파일 시간
# - 가장 큰 바이너리
# - 때로는 -O2보다 느릴 수 있음
vaisc build -O3 myprogram.vais -o myprogram
```

### 릴리스 빌드

프로덕션 배포를 위해 릴리스 빌드를 사용합니다:

```bash
# 릴리스 빌드 (자동으로 -O2 적용)
vaisc build --release myprogram.vais

# 추가 최적화 (LTO, 링크타임 최적화)
vaisc build --release --lto myprogram.vais
```

### Link-Time Optimization (LTO)

LTO는 전체 프로그램을 분석하여 추가 최적화를 수행합니다:

```toml
# Vais.toml
[profile.release]
lto = true           # 링크타임 최적화 활성화
codegen-units = 1   # 단일 코드 생성 단위 (더 느리지만 더 최적화됨)
```

### 프로필 유도 최적화 (PGO)

실제 프로그램 실행 데이터를 사용하여 최적화합니다:

```bash
# 1단계: 계측 빌드
vaisc build --pgo-generate myprogram.vais -o myprogram

# 2단계: 프로필 생성을 위해 프로그램 실행
./myprogram < input.txt

# 3단계: 최적화된 빌드
vaisc build --pgo-use myprogram.vais -o myprogram-optimized
```

## 벤치마킹

### 간단한 벤치마크

성능을 측정하기 위한 기본 벤치마크를 작성합니다:

```vais
# 현재 시간(나노초 단위) 반환
F get_nanotime() -> i64 {
    # 표준 라이브러리의 time 함수 사용
    0  # 실제 구현은 C 바인딩
}

# 벤치마크 실행 함수
F benchmark<F>(name: str, iterations: i64, f: F) {
    start := get_nanotime()

    i := mut 0
    L i < iterations {
        f()
        i = i + 1
    }

    end := get_nanotime()
    elapsed := end - start

    avg_ns := elapsed / iterations
    puts("Benchmark ~{name}:")
    puts("  Total: ~{elapsed} ns")
    puts("  Avg: ~{avg_ns} ns/iteration")
}

# 테스트할 함수
F fibonacci(n: i64) -> i64 {
    I n <= 1 { R n }
    R fibonacci(n - 1) + fibonacci(n - 2)
}

F main() -> i64 {
    # 벤치마크 실행
    benchmark("fibonacci(20)", 100, || { fibonacci(20) })

    0
}
```

### Criterion 벤치마크

더 정교한 벤치마킹을 위해 Criterion을 사용합니다:

```bash
# benches/fibonacci_bench.vais
F fibonacci(n: i64) -> i64 {
    I n <= 1 { R n }
    R fibonacci(n - 1) + fibonacci(n - 2)
}

F fibonacci_optimized(n: i64) -> i64 {
    I n <= 1 { R n }

    a := mut 0
    b := mut 1
    i := mut 2

    L i <= n {
        c := a + b
        a = b
        b = c
        i = i + 1
    }

    b
}

# 벤치마크 실행
# vaisc bench --bench fibonacci_bench
```

### 메모리 할당 벤치마크

메모리 할당 성능을 측정합니다:

```vais
S Node {
    value: i64
    next: i64  # 다음 노드 포인터
}

F create_linked_list(size: i64) -> i64 {
    head := 0
    i := mut 0

    L i < size {
        # 노드 생성 시뮬레이션
        node := Node { value: i, next: 0 }
        # 리스트에 추가
        i = i + 1
    }

    head
}

F benchmark_allocation() {
    puts("=== Memory Allocation Benchmark ===")

    # 작은 할당
    puts("Small allocations (1K):")
    start := get_time()
    create_linked_list(1000)
    elapsed := get_time() - start
    puts("Time: ~{elapsed} ms")

    # 큰 할당
    puts("Large allocations (10K):")
    start = get_time()
    create_linked_list(10000)
    elapsed = get_time() - start
    puts("Time: ~{elapsed} ms")
}
```

## 프로파일링

### 컴파일 시간 프로파일링

컴파일 성능을 분석합니다:

```bash
# 컴파일 시간 측정
time vaisc build --release myprogram.vais

# 자세한 타이밍 정보
vaisc build --release --timings myprogram.vais
```

### 런타임 프로파일링

실행 중인 프로그램의 성능을 분석합니다:

```bash
# Perf를 사용한 프로파일링 (Linux)
perf record -g ./myprogram
perf report

# Instruments를 사용 (macOS)
instruments -t "Time Profiler" ./myprogram

# Valgrind를 사용한 메모리 프로파일링
valgrind --tool=cachegrind ./myprogram
```

### Flamegraph 생성

성능 분석을 위한 flamegraph를 생성합니다:

```bash
# Linux에서 flamegraph 생성
perf record -g -F 99 ./myprogram
perf script | stackcollapse-perf.pl | flamegraph.pl > perf.svg

# 결과 확인
open perf.svg
```

## 메모리 관리 최적화

### 스택 vs 힙

```vais
# 스택 할당 (빠름, 크기 제한)
F stack_example() {
    arr: [i64; 100] = [0; 100]  # 스택에 할당
    arr[0] = 42
}

# 힙 할당 (느림, 제한 없음)
F heap_example() {
    # 동적 할당 시뮬레이션
    data := 42
}

# 스택 할당이 선호됨
F preferred_approach() {
    # 작은 고정 크기 배열은 스택에
    small_array: [i64; 10]

    # 큰 배열이나 동적 크기는 힙에
    # (라이브러리 함수 사용)
}
```

### 메모리 재사용

```vais
S Buffer {
    data: [i64; 1000]
    size: i64
}

# 버퍼를 재사용하여 할당 최소화
F process_multiple_batches(batches: i64) {
    buffer := Buffer { data: [0; 1000], size: 0 }

    i := mut 0
    L i < batches {
        # 버퍼 내용 초기화 (재할당 아님)
        buffer.size = 0

        # 데이터 처리
        # ...

        i = i + 1
    }
}
```

### GC 최적화

Vais의 선택적 GC를 최적화합니다:

```toml
# Vais.toml
[profile.release]
# GC를 사용하지 않도록 설정 (수동 관리)
gc = false

# 또는 GC 튜닝
gc = true
gc-threads = 4  # GC 스레드 수
gc-heap-size = "1GB"  # 초기 힙 크기
```

## 알고리즘 최적화

### 불필요한 연산 제거

```vais
# 비효율: 루프에서 계산 반복
F inefficient() {
    sum := mut 0
    i := mut 0
    L i < 1000 {
        # 루프마다 sin 계산
        sum = sum + sin(3.14159 / 2)
        i = i + 1
    }
}

# 효율: 루프 전에 계산
F efficient() {
    sin_value := sin(3.14159 / 2)  # 한 번만 계산
    sum := mut 0
    i := mut 0
    L i < 1000 {
        sum = sum + sin_value
        i = i + 1
    }
}
```

### 데이터 구조 선택

```vais
# 순차 접근이 많을 때: 배열 사용 (캐시 효율)
F array_approach() {
    arr := [1, 2, 3, 4, 5]
    sum := mut 0
    i := mut 0
    L i < 5 {
        sum = sum + arr[i]
        i = i + 1
    }
}

# 무작위 접근이 많을 때: 해시맵 고려
# (표준 라이브러리에서 HashMap 제공)
```

### 루프 최적화

```vais
# 비효율: 의존성이 있는 연산
F inefficient_loop() {
    result := mut 0
    i := mut 0
    L i < 1000 {
        result = result + (i * i)  # 각 반복이 이전 결과에 의존
        i = i + 1
    }
}

# 효율: 병렬화 가능한 연산
F efficient_loop() {
    sum := mut 0
    i := mut 0
    L i < 1000 {
        sum = sum + (i * i)  # 같은 연산이지만 더 효율적
        i = i + 1
    }
}
```

## SIMD 활용

SIMD(Single Instruction Multiple Data)를 사용하여 벡터 연산을 가속화합니다:

```vais
# SIMD 타입 (아키텍처에 따라 다름)
# i64x4, f64x4 등

F vector_add(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    # 일반적인 루프
    result: [f64; 4]
    i := mut 0
    L i < 4 {
        result[i] = a[i] + b[i]
        i = i + 1
    }
    result
}

# SIMD 버전 (컴파일러가 최적화)
F vector_add_simd(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    # 벡터 연산 (컴파일러가 SIMD로 변환)
    a  # 간단한 예: 직접 반환
}
```

## GPU 코드 생성

집약적인 계산을 GPU에서 실행합니다:

```bash
# GPU 최적화 빌드
vaisc build --gpu cuda myprogram.vais -o myprogram

# Metal (macOS)
vaisc build --gpu metal myprogram.vais -o myprogram
```

## 성능 최적화 체크리스트

```
□ 적절한 최적화 수준 선택 (-O2 권장)
□ LTO 활성화 (릴리스 빌드)
□ 불필요한 메모리 할당 제거
□ 스택 메모리 선호
□ 캐시 친화적 데이터 구조 사용
□ 루프 최적화 (의존성 제거)
□ 벤치마킹으로 병목 지점 식별
□ 프로파일링으로 실제 성능 측정
□ SIMD 활용 가능성 검토
□ GPU 오프로딩 검토 (큰 계산 작업)
□ 알고리즘 복잡도 분석 (Big-O)
```

## 실제 최적화 예제

### 최적화 전후 비교

```vais
# 비효율: 반복되는 계산과 할당
F naive_prime_check(n: i64) -> bool {
    i := mut 2
    L i < n {
        I n % i == 0 { R false }
        i = i + 1
    }
    true
}

# 최적화: 불필요한 연산 제거
F optimized_prime_check(n: i64) -> bool {
    I n <= 1 { R false }
    I n == 2 { R true }
    I n % 2 == 0 { R false }

    # sqrt(n)까지만 확인
    i := mut 3
    L i * i <= n {
        I n % i == 0 { R false }
        i = i + 2  # 짝수 생략
    }
    true
}

F main() -> i64 {
    # 최적화된 버전이 훨씬 빠름
    benchmark("naive", 10000, || { naive_prime_check(1000) })
    benchmark("optimized", 10000, || { optimized_prime_check(1000) })

    0
}
```

## 다음 단계

- [코딩 스타일 가이드](./style-guide.md): 성능을 고려한 코딩 스타일
- [생산 체크리스트](../production-checklist.md): 성능 측정과 모니터링
- [표준 라이브러리](../stdlib/stdlib.md): 최적화된 표준 함수
