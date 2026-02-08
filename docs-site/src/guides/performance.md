# 성능 최적화 가이드

Vais 컴파일러는 다양한 최적화 기법을 제공하여 실행 속도와 컴파일 시간을 모두 개선할 수 있습니다. 이 가이드는 Vais 프로그램의 성능을 극대화하는 방법을 다룹니다.

---

## 최적화 레벨

Vais 컴파일러는 4가지 최적화 레벨을 제공합니다.

### -O0 (기본값)

최적화 없음. 디버깅에 최적화되어 있습니다.

```bash
vaisc build main.vais
vaisc build main.vais -O0
```

- **장점**: 빠른 컴파일, 디버깅 용이
- **단점**: 느린 실행 속도
- **사용 시점**: 개발 단계, 버그 추적

### -O1

기본 최적화. 컴파일 시간과 실행 속도의 균형.

```bash
vaisc build main.vais -O1
```

- **최적화**: dead code elimination, constant folding, basic inlining
- **사용 시점**: 일반 개발, CI 빌드

### -O2

적극적 최적화. 프로덕션 권장 레벨.

```bash
vaisc build main.vais -O2
```

- **최적화**: loop unrolling, vectorization, function inlining, tail call optimization
- **컴파일 시간**: -O1 대비 20-30% 증가
- **실행 속도**: -O1 대비 30-50% 개선
- **사용 시점**: 프로덕션 릴리스, 벤치마크

### -O3

최대 최적화. 컴파일 시간 희생.

```bash
vaisc build main.vais -O3
```

- **최적화**: aggressive inlining, loop transformations, interprocedural optimization
- **컴파일 시간**: -O2 대비 50-100% 증가
- **실행 속도**: -O2 대비 5-15% 추가 개선
- **사용 시점**: CPU-bound 애플리케이션, 고성능 라이브러리

---

## PGO (Profile-Guided Optimization)

PGO는 실제 실행 프로파일을 기반으로 최적화합니다. **2단계 프로세스**입니다.

### 1단계: 프로파일 수집

```bash
# 프로파일링 빌드 생성
vaisc build main.vais --pgo-gen -O2

# 실행하여 프로파일 데이터 수집 (.profraw 생성)
./main

# 프로파일 병합 (여러 실행 결과 통합)
llvm-profdata merge -o default.profdata default_*.profraw
```

### 2단계: 프로파일 사용

```bash
# 프로파일 기반 최적화 빌드
vaisc build main.vais --pgo-use=default.profdata -O2
```

### PGO 성능 향상

- **브랜치 예측**: 자주 실행되는 경로를 최적화
- **함수 인라이닝**: 호출 빈도 기반 인라이닝 결정
- **코드 레이아웃**: 핫 경로를 연속된 메모리에 배치

**예상 성능 개선**: 5-20% (워크로드 의존적)

### 모범 사례

1. **대표적인 워크로드 사용**: 프로파일링 실행은 실제 사용 패턴을 반영해야 함
2. **여러 시나리오 수집**: 다양한 입력으로 여러 번 실행 후 병합
3. **정기적 갱신**: 코드 변경 시 프로파일 재수집

```bash
# 예: 여러 워크로드 프로파일링
vaisc build server.vais --pgo-gen -O2
./server < workload1.txt
./server < workload2.txt
./server < workload3.txt
llvm-profdata merge -o server.profdata default_*.profraw
vaisc build server.vais --pgo-use=server.profdata -O2
```

---

## 인크리멘탈 컴파일

대규모 프로젝트에서 컴파일 시간을 단축합니다.

### 기본 사용법

```bash
# 모듈별 컴파일 활성화
vaisc build main.vais --per-module
```

### 동작 방식

1. 각 모듈을 개별 LLVM IR로 컴파일
2. 변경되지 않은 모듈은 캐시 재사용
3. 변경된 모듈만 재컴파일
4. 링크 단계에서 통합

### 성능 수치

| 프로젝트 크기 | 초기 빌드 | 1파일 변경 | 개선율 |
|------------|---------|----------|--------|
| 5K lines   | 380ms   | 78ms     | 4.9x   |
| 30K lines  | 571ms   | 96ms     | 5.9x   |
| 50K lines  | 900ms   | 140ms    | 6.4x   |

### 캐시 관리

```bash
# 캐시 크기 제한 (기본값: 512MB)
vaisc build main.vais --per-module --cache-limit=256

# 캐시 위치: ~/.vais-cache/ir_cache/
# 수동 정리:
rm -rf ~/.vais-cache/ir_cache/
```

### 주의사항

- **초기 빌드는 더 느릴 수 있음** (캐시 생성 오버헤드)
- **디스크 공간 사용**: 대규모 프로젝트는 수백 MB 캐시
- **최적화 레벨**: -O2/-O3와 함께 사용 가능

---

## 벤치마크 작성

Vais는 Criterion 기반 벤치마크를 지원합니다.

### 기본 벤치마크

```vais
# benches/my_bench.vais
U std/bench

F fibonacci(n: i64) -> i64 {
  I n <= 1 { R n }
  R fibonacci(n - 1) + fibonacci(n - 2)
}

F bench_fib() {
  benchmark("fibonacci_20", || {
    fibonacci(20)
  })
}

F main() {
  bench_fib()
  bench_report()
}
```

### 실행

```bash
vaisc bench benches/my_bench.vais
```

### 비교 벤치마크

```vais
F bench_sorting() {
  v1 := vec_new<i64>()
  L i := 0; i < 1000; i := i + 1 {
    vec_push(v1, 1000 - i)
  }

  benchmark("quicksort_1000", || {
    vec_sort(v1)
  })

  benchmark("merge_sort_1000", || {
    vec_merge_sort(v1)
  })
}
```

### 벤치마크 모범 사례

1. **워밍업**: 첫 실행은 캐시 미스로 느릴 수 있음
2. **반복 횟수**: 충분히 반복하여 통계적 유의성 확보
3. **일관된 환경**: CPU 거버너 설정, 백그라운드 프로세스 최소화
4. **프로파일링과 병행**: `--profile` 플래그로 상세 분석

---

## 메모리 최적화

### Arena Allocator

빈번한 할당/해제 시 성능 향상.

```vais
U std/alloc

F main() {
  arena := arena_new(1024 * 1024)  # 1MB arena

  L i := 0; i < 10000; i := i + 1 {
    ptr := arena_alloc(arena, 64)
    # 개별 free 불필요
  }

  arena_free(arena)  # 한 번에 전체 해제
}
```

**성능**: 일반 malloc 대비 **10-50배 빠름** (할당 패턴 의존)

### Object Pool

재사용 가능한 객체 풀.

```vais
U std/pool

S Connection { fd: i64 }

F main() {
  pool := pool_new<Connection>(10)  # 10개 미리 할당

  L i := 0; i < 100; i := i + 1 {
    conn := pool_acquire(pool)
    # 연결 사용
    pool_release(pool, conn)
  }
}
```

### Box (힙 할당)

큰 구조체는 Box로 감싸서 스택 오버플로 방지.

```vais
S LargeData {
  buffer: [i64; 10000]
}

F main() {
  data := box(LargeData { buffer: [0; 10000] })
  # 스택 대신 힙에 할당
}
```

---

## SIMD 활용

Vais는 SIMD (Single Instruction Multiple Data)를 지원합니다.

### 기본 SIMD 연산

```vais
U std/simd

F dot_product_simd(a: Vec<f32>, b: Vec<f32>) -> f32 {
  result := 0.0
  len := vec_len(a)

  # SIMD 레지스터 크기 (4 floats for SSE, 8 for AVX)
  simd_width := 8

  L i := 0; i + simd_width < len; i := i + simd_width {
    va := simd_load_f32(vec_ptr(a), i)
    vb := simd_load_f32(vec_ptr(b), i)
    vc := simd_mul_f32(va, vb)
    result := result + simd_sum_f32(vc)
  }

  # 나머지 처리
  L i >= len - simd_width; i < len; i := i + 1 {
    result := result + vec_get(a, i) * vec_get(b, i)
  }

  R result
}
```

### SIMD 성능 향상

- **AVX2 (256-bit)**: 4-8배 속도 향상 (f32 기준)
- **NEON (ARM)**: 3-4배 속도 향상
- **자동 벡터화**: -O2/-O3는 일부 루프를 자동 SIMD화

### 컴파일러 힌트

```bash
# AVX2 활성화
vaisc build main.vais -O3 --target-cpu=native

# 특정 CPU 타겟
vaisc build main.vais -O3 --target-cpu=skylake
```

---

## 프로파일링

### --profile 플래그

컴파일러 내부 프로파일링.

```bash
vaisc build main.vais --profile
```

출력 예:
```
=== Vais Compiler Profile ===
Lexing:        12ms
Parsing:       45ms
Type Checking: 78ms
Codegen:       234ms
Linking:       91ms
Total:         460ms
```

### 런타임 프로파일링

```bash
# Linux: perf
perf record ./main
perf report

# macOS: Instruments
instruments -t "Time Profiler" ./main

# 범용: flamegraph
vaisc build main.vais -O2 --debug-info
perf record -g ./main
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

---

## 컴파일 시간 최적화

### 1. 인크리멘탈 컴파일 사용

```bash
vaisc build main.vais --per-module
```

### 2. 병렬 컴파일

```bash
# 4개 스레드로 병렬 컴파일 (기본값: CPU 코어 수)
vaisc build main.vais --per-module --jobs=4
```

### 3. IR 캐싱

```bash
# IR 파일 보존 (재컴파일 시 재사용)
vaisc build main.vais --emit-ir --per-module
```

### 4. 타입 체커 최적화

- **모듈 분할**: 대형 파일을 여러 모듈로 분리
- **제네릭 사용 최소화**: 과도한 monomorphization 방지
- **타입 명시**: 타입 추론 부담 감소

```vais
# 나쁨: 과도한 타입 추론
F process(data) {
  result := transform(data)
  R finalize(result)
}

# 좋음: 명시적 타입
F process(data: Vec<i64>) -> i64 {
  result: Vec<i64> = transform(data)
  R finalize(result)
}
```

### 5. 의존성 최소화

```vais
# 나쁨: 전체 라이브러리 임포트
U std/collections

# 좋음: 필요한 함수만 선택적 임포트
U std/collections { vec_new, vec_push, vec_len }
```

---

## 성능 체크리스트

### 개발 단계
- [ ] `-O0` 또는 `-O1` 사용
- [ ] `--per-module` 활성화 (빠른 재컴파일)
- [ ] 디버그 심볼 유지

### 프로덕션 릴리스
- [ ] `-O2` 또는 `-O3` 사용
- [ ] PGO 적용 고려 (CPU-bound 앱)
- [ ] `--strip` (디버그 심볼 제거, 바이너리 크기 50% 감소)
- [ ] SIMD 최적화 활성화 (`--target-cpu=native`)

### 성능 크리티컬 애플리케이션
- [ ] PGO 필수 적용
- [ ] 벤치마크 작성 및 정기 실행
- [ ] 프로파일링으로 핫스팟 식별
- [ ] Arena/Pool 할당자 고려
- [ ] SIMD 수동 최적화

---

## 실전 예제

### 고성능 HTTP 서버 빌드

```bash
# 1. PGO 프로파일 수집
vaisc build server.vais --pgo-gen -O2 --per-module
./server &
SERVER_PID=$!
wrk -t4 -c100 -d30s http://localhost:8000/
kill $SERVER_PID
llvm-profdata merge -o server.profdata default_*.profraw

# 2. 최적화 빌드
vaisc build server.vais \
  --pgo-use=server.profdata \
  -O3 \
  --target-cpu=native \
  --strip

# 3. 검증
wrk -t4 -c100 -d60s http://localhost:8000/
```

### 수치 계산 라이브러리 빌드

```bash
vaisc build matrix.vais \
  -O3 \
  --target-cpu=native \
  --emit-ir \
  --per-module

# SIMD 벡터화 확인
grep "vectorize" matrix.ll
```

---

## 추가 리소스

- [Compiler Flags Reference](../compiler/flags.md)
- [Benchmark Suite](../../benches/README.md)
- [Profiler Documentation](../tools/profiler.md)
- [LLVM Optimization Guide](https://llvm.org/docs/Passes.html)

---

성능 최적화는 측정에서 시작합니다. **벤치마크 → 프로파일 → 최적화 → 검증** 사이클을 반복하세요.
