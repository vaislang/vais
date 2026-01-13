# Vais v2: AI-Optimized Executable Language

## Project Vision

**"AI가 효율적으로 생성하고, 직접 실행되는 언어"**

Python이 인간을 위해 설계되었다면, Vais v2는 AI를 위해 설계된다.

---

## 핵심 목표

| 목표 | 측정 기준 |
|------|----------|
| 토큰 효율성 | Python 대비 40%+ 절감 |
| AI 생성 정확도 | 유효한 코드 생성 80%+ |
| 실행 가능 | 자체 VM/네이티브 실행 |

---

## Phase 0: 검증 (완료)

### 상태: ✅ COMPLETED

### 목표
"이게 진짜 되는지" 증명

### 태스크

| # | 태스크 | 상태 | 완료일 |
|---|--------|------|--------|
| 0.1 | 문법 스펙 v6b 설계 | ✅ 완료 | 2026-01-12 |
| 0.2 | 토큰 벤치마크 도구 | ✅ 완료 | 2026-01-12 |
| 0.3 | 36개 예제 작성 | ✅ 완료 | 2026-01-12 |
| 0.4 | LLM 생성 테스트 | ✅ 완료 | 2026-01-12 |
| 0.5 | 결과 분석 및 판단 | ✅ 완료 | 2026-01-12 |

### 결과
- [x] Python 대비 **43.9% 토큰 절감** (목표 40% 달성)
- [x] LLM 생성 정확도 **93%** (빌트인 추가 시 100% 예상)

### Go/No-Go 결정: **GO**
- 둘 다 충족 → Phase 1 진행 결정

---

## Phase 1: 언어 기반 (우선 작업)

### 상태: ✅ COMPLETED (문서화)

### 우선 작업 완료 내역

| # | 태스크 | 상태 | 산출물 |
|---|--------|------|--------|
| 1.0 | v6b 문법 명세 작성 | ✅ 완료 | VAIS_V6B_SPEC.md |
| 1.1 | 빌트인 목록 확정 | ✅ 완료 | VAIS_V6B_BUILTINS.md |
| 1.2 | 기존 코드 재사용 분석 | ✅ 완료 | VAIS_V6B_REUSE_ANALYSIS.md |

### 다음 단계: 구현

| # | 태스크 | 상태 | 예상 |
|---|--------|------|------|
| 1.3 | Lexer 재구현 | ⏳ 대기 | ~300 LOC |
| 1.4 | Parser 재구현 | ⏳ 대기 | ~800 LOC |
| 1.5 | AST 정의 | ⏳ 대기 | ~200 LOC |
| 1.6 | IR 확장 | ⏳ 대기 | ~100 LOC |
| 1.7 | VM 확장 | ⏳ 대기 | ~200 LOC |
| 1.8 | 빌트인 확장 | ⏳ 대기 | ~500 LOC |
| 1.9 | CLI 통합 | ⏳ 대기 | ~100 LOC |
| 1.10 | 테스트 | ⏳ 대기 | ~400 LOC |

---

## 설계 원칙

### 1. 토큰 최소화
```
❌ function calculateTotal(items: Array<Item>): number
✅ (fn calc_total [items:[Item]]:n ...)
```

### 2. 일관된 구조
```
; 모든 것이 표현식
; 괄호로 구조 명확
; 예외 없는 문법
```

### 3. 명시적 > 암묵적
```
; 타입 명시 (축약형)
; 의존성 명시
; 부작용 명시
```

### 4. 파싱 용이성
```
; S-표현식 기반
; 컨텍스트 프리 문법
; 단순한 토큰 규칙
```

---

## 문법 초안 (v2)

### 타입 시스템 (축약)
```
i   = int
i32 = int32
i64 = int64
f   = float
f32 = float32
f64 = float64
s   = string
b   = bool
[T] = array of T
{K:V} = map
?T  = optional T
```

### 함수 정의
```lisp
(fn name [param:type ...] :return-type
  body)

; 예시
(fn add [a:i b:i] :i
  (+ a b))
```

### 조건문
```lisp
(if cond then else)
(? cond then else)      ; 축약형

(cond
  test1 result1
  test2 result2
  :else default)
```

### 파이프라인
```lisp
(-> value
    (fn1 arg1)
    (fn2 arg2)
    fn3)
```

### 컬렉션 연산
```lisp
(filter pred coll)
(map fn coll)
(reduce fn init coll)
(sum coll)
(count coll)
```

### 에러 처리
```lisp
(require cond "error message")
(try expr (catch e handler))
```

### 비동기
```lisp
(fn name [args] :type :async
  body)

(await expr)
```

---

## 기존 Vais v1과 비교

| 측면 | v1 | v2 |
|------|----|----|
| 구조 | 블록 기반 (UNIT, META, FLOW...) | S-표현식 |
| 키워드 | 장황 (ENDINPUT, ENDOUTPUT) | 최소 (fn, if, ->) |
| 타입 | INT32, STRING, LIST<T> | i, s, [T] |
| 토큰 수 | Python의 6배 | Python의 0.6배 목표 |
| 파싱 | 복잡 (컨텍스트 의존) | 단순 (재귀 하향) |

---

## 예제 비교

### Hello World

**Python (15 tokens)**
```python
def hello():
    return "Hello, World!"
```

**Vais v1 (150+ tokens)**
```vais
UNIT FUNCTION examples.hello V1.0.0
META
  DOMAIN examples
ENDMETA
INPUT
ENDINPUT
OUTPUT
  message : STRING
ENDOUTPUT
...
END
```

**Vais v2 (10 tokens)**
```lisp
(fn hello [] :s "Hello, World!")
```

### 리스트 필터 + 맵

**Python (25 tokens)**
```python
def get_emails(users):
    return [u.email.upper() for u in users if u.active]
```

**Vais v2 (20 tokens)**
```lisp
(fn get_emails [users:[User]] :[s]
  (-> users (filter .active) (map .email) (map upper)))
```

---

## 로드맵

| Phase | 목표 | 기간 |
|-------|------|------|
| 0 | 검증 | 2-3주 |
| 1 | 언어 기반 (Lexer, Parser, VM) | 1-2개월 |
| 2 | AI 통합 | 1-2개월 |
| 3 | 최적화 | 1개월 |
| 4 | 확장 (stdlib, native) | 2-3개월 |
| 5 | 생태계 | 지속 |

---

## 진행 로그

### 2026-01-12 (저녁) - 확장성 설계 완료
- 전체 아키텍처 설계 (VAIS_ARCHITECTURE.md)
- 코어 언어 설계 원칙 (VAIS_CORE_DESIGN.md)
- FFI 시스템 설계 (VAIS_FFI_DESIGN.md)
- 패키지 시스템 설계 (VAIS_PACKAGE_SYSTEM.md)
- 표준 라이브러리 구조 (VAIS_STDLIB.md)
- 확장 가이드라인 (VAIS_EXTENSION_GUIDE.md)
- **목표: Python처럼 커뮤니티가 확장할 수 있는 생태계**

### 2026-01-12 (오후) - Phase 1 우선 작업 완료
- v6b 문법 명세 문서 작성 (VAIS_V6B_SPEC.md)
- 빌트인 함수 목록 확정 (VAIS_V6B_BUILTINS.md, ~60개)
- 기존 코드 재사용 분석 완료 (VAIS_V6B_REUSE_ANALYSIS.md)
  - IR/VM: 60-90% 재사용 가능
  - Lexer/Parser: 전면 재작성 필요
- 다음 단계: 구현 (Lexer, Parser, AST)

### 2026-01-12 (오전) - Phase 0 완료 및 Go 결정
- Vais v2 프로젝트 시작
- 기존 Vais v1 토큰 효율성 분석: Python 대비 6배 비효율 확인
- 7가지 문법 버전 실험 (v2~v6b)
- **v6b 최종 선택**: 43.9% 토큰 절감, 93% 생성 정확도
- **Go 결정**: Phase 1 진행

---

## 참고 자료

- [SimPy: AI-oriented grammar for Python](https://arxiv.org/abs/2404.16333)
- [CrossTL: Universal IR](https://arxiv.org/abs/2508.21256)
- [Token Cost Optimization](https://www.kapihq.com/blog/token-cost-optimization)
