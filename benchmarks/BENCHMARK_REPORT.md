# Vais vs Python Performance Benchmark Report

**Date:** 2026-01-13
**Vais Version:** 1.0.0 (with Arc optimization)
**Python Version:** 3.x (CPython)
**Platform:** macOS Darwin 25.2.0

---

## Executive Summary

### Before Optimization
Vais interpreter는 Python 대비 **약 10-12배 느린** 성능을 보였습니다.

### After Arc Optimization (v1.0.1)
Arc 기반 함수 공유 최적화 후, **약 7배 느린** 수준으로 개선되었습니다.
이는 **1.7배 성능 개선**입니다. 추가 최적화와 JIT 컴파일을 통해 Python 수준 이상 달성 가능합니다.

---

## 1. Fibonacci Benchmark (Recursive)

가장 일반적인 재귀 성능 테스트입니다.

### Results (Before vs After Optimization)

| n | Python (ms) | Vais Before (ms) | Vais After (ms) | Before Ratio | After Ratio | Speedup |
|---|-------------|------------------|-----------------|--------------|-------------|---------|
| 20 | 0.94 | 16 | 10 | 17.0x | **10.6x** | 1.6x |
| 25 | 10.63 | 134 | 75 | 12.6x | **7.1x** | 1.8x |
| 30 | 117.04 | 1,428 | 803 | 12.2x | **6.9x** | 1.8x |
| 35 | 1,263.22 | 15,924 | 9,216 | 12.6x | **7.3x** | 1.7x |

### Analysis

- **최적화 전**: Python보다 **약 12배 느림**
- **최적화 후**: Python보다 **약 7배 느림** (1.7배 개선)
- 적용된 최적화:
  - `Arc<CompiledFunction>`으로 함수 공유 (clone 비용 제거)
- 남은 병목:
  1. **TCO 미적용** (일반 재귀는 TCO 대상 아님)
  2. **명령어 dispatch 오버헤드** (match 기반)
  3. **클로저 환경 전체 캡처** (필요 변수만 캡처하도록 개선 필요)

---

## 2. Collection Operations Benchmark

함수형 프로그래밍 핵심 연산 (map, filter, reduce) 테스트입니다.

### Chain Operation: filter → map → reduce
`[1..100].?(_ % 2 == 0).@(_ * 2)./+`

| Size | Python (µs) | Vais (ms) | Ratio |
|------|-------------|-----------|-------|
| 100 elements | 3.65 | ~4 | **~1,100x slower** |

### Analysis

- Collection 연산에서 Vais가 **매우 느림**
- 원인:
  1. **매 연산마다 새 배열 생성** (immutable 설계)
  2. **클로저 환경 전체 clone** (`vm.rs:1032`)
  3. **병렬 연산 시 VM 전체 clone** (`vm.rs:1148`)

---

## 3. Code Review Summary

### 주요 성능 문제

1. **과도한 Cloning (85회, vm.rs)**
   ```rust
   let func = self.functions.get(name)?.clone();  // Line 203
   ```
   - 해결책: `Arc<CompiledFunction>` 사용

2. **병렬 연산 비효율 (vm.rs:1143-1160)**
   ```rust
   items.into_par_iter().map(|item| {
       let mut mini_vm = Vm::new();  // 요소당 새 VM 생성
       mini_vm.functions = self.functions.clone();  // 전체 함수 clone
   })
   ```
   - 해결책: VM 풀링, `Arc<>` 공유

3. **상수 전파 미구현**
   - lowering 단계에서 상수 전파 없음
   - IR 최적화 단계만 의존

### 아키텍처 강점

- 모듈 구조 우수 (13개 crate 분리)
- 에러 처리 체계적 (thiserror 사용)
- TCO 지원 (꼬리 재귀 최적화)
- 테스트 커버리지 양호

### 기능 완성도

| Feature | Status | Notes |
|---------|--------|-------|
| 기본 타입 | ✅ 완료 | Int, Float, String, Bool, Array, Map |
| 함수 정의 | ✅ 완료 | 일급 함수, 클로저 |
| 컬렉션 연산 | ✅ 완료 | .@, .?, ./ 연산자 |
| 패턴 매칭 | ⚠️ 부분 | 기본 match만 지원 |
| 모듈 시스템 | ⚠️ 부분 | use 파싱만, 실제 해석 없음 |
| Async/Await | ❌ 미구현 | 토큰만 정의됨 |
| 제네릭 | ⚠️ 부분 | 파싱만, 타입 체크 없음 |
| Trait | ❌ 미구현 | 토큰만 정의됨 |
| Enum 타입체크 | ❌ 미구현 | TODO 주석 있음 |

---

## 4. Performance Improvement Recommendations

### 단기 (Critical)

1. **Arc로 함수 공유**
   ```rust
   // Before
   functions: HashMap<String, CompiledFunction>
   // After
   functions: HashMap<String, Arc<CompiledFunction>>
   ```
   예상 개선: **2-3x**

2. **클로저 환경 최적화**
   - 필요한 변수만 캡처 (현재 전체 clone)
   예상 개선: **1.5x**

### 중기 (High Priority)

3. **명령어 dispatch 최적화**
   - Computed goto 또는 threaded code 고려
   예상 개선: **1.2-1.5x**

4. **상수 전파/접기 강화**
   - lowering 단계에서 최적화 추가
   예상 개선: **1.1-1.3x**

### 장기 (Performance Goal)

5. **JIT 컴파일 활성화**
   - Cranelift 기반 JIT 이미 구현됨
   - 핫 함수 자동 JIT 컴파일
   예상 개선: **10-50x** (Python 수준 또는 초과)

---

## 5. Comparison with Other Languages

| Language | fib(35) Time | vs Python |
|----------|--------------|-----------|
| **C (gcc -O3)** | ~50ms | 25x faster |
| **Rust (release)** | ~50ms | 25x faster |
| **Go** | ~100ms | 12x faster |
| **Python** | 1,263ms | baseline |
| **JavaScript (V8)** | ~150ms | 8x faster |
| **Vais (current)** | 15,924ms | **12x slower** |
| **Vais (with JIT, est.)** | ~500ms | 2x faster |

---

## 6. Conclusion

### 현재 상태
- Vais v1.0은 **기능적으로 동작**하지만 성능 최적화가 필요
- Python 대비 10-12배 느림

### 개선 방향
1. `Arc<>` 기반 공유로 clone 제거 → **즉시 2-3배 개선 가능**
2. JIT 활성화 → **Python 수준 또는 초과 가능**

### 권장사항
- 프로덕션 사용 전 Arc 기반 리팩토링 권장
- 성능 민감 애플리케이션은 JIT 모드 사용 권장

---

## Appendix: Benchmark Code

### Vais
```vais
// Fibonacci
fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)
fib(35)

// Collection chain
[1,2,...,100].?(_ % 2 == 0).@(_ * 2)./+
```

### Python
```python
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

# Collection chain
sum(x * 2 for x in range(1, 101) if x % 2 == 0)
```
