# AOEL Language Roadmap

## Current Status (v0.1.0)

### Completed Features

#### Core Language
- [x] Lexer - 토큰화, 유니코드 지원
- [x] Parser - AST 생성, 연산자 우선순위
- [x] Type Checker - Hindley-Milner 타입 추론
- [x] IR Lowering - AST → IR 변환
- [x] VM - 스택 기반 인터프리터

#### JIT Compiler (Cranelift)
- [x] Integer 연산 JIT 컴파일
- [x] Float 연산 JIT 컴파일
- [x] 재귀 함수 지원 (TCO)
- [x] 조건문/비교 연산
- [x] 50-75x 성능 향상

#### Code Generation
- [x] C 코드 생성
- [x] WASM/WAT 생성
- [x] LLVM IR 생성

#### Tools
- [x] CLI (aoel) - 실행, 빌드, REPL
- [x] LSP Server - 자동완성, 진단
- [x] Playground - 웹 기반 실행 환경
- [x] Package Manager - init, add, publish

#### Standard Library (100+ functions)
- [x] 컬렉션 (len, first, last, reverse, concat, etc.)
- [x] 수학 (abs, sqrt, pow, sin, cos, log, etc.)
- [x] 문자열 (upper, lower, trim, split, join, etc.)
- [x] 타입 변환 (int, float, str, bool)
- [x] 파일 I/O (read_file, write_file, etc.)
- [x] JSON (parse, stringify, get, set, etc.)
- [x] HTTP (get, post, put, delete)
- [x] 시간 (time_now, time_format, sleep)
- [x] 랜덤 (random, random_int, shuffle, sample)

#### Testing
- [x] 440+ 단위 테스트
- [x] 통합 테스트
- [x] 벤치마크

---

## Phase 0: Core Optimization (현재 진행 중)

> **목표:** Python보다 빠르고, 바이브 코딩에 최적화된 코어 완성

### 0.1 VM 네이티브 최적화 (우선순위: 긴급)

#### 완료
- [x] HashMap → Vec 변환 (locals 최적화)
- [x] Map 네이티브 연산 (MapMulConst, MapAddConst, MapSubConst, MapDivConst)
- [x] Filter 네이티브 연산 (FilterGtConst, FilterLtConst, FilterEven, FilterOdd 등)
- [x] 패턴 감지 및 최적화 opcode 자동 생성

#### 완료 (버그 수정)
- [x] UNIQUE `format!("{:?}")` → `hash_key()` 변경 (10-50x 성능 개선)
- [x] GROUP_BY `format!("{:?}")` → 효율적 키 생성
- [x] INDEX_OF `format!("{:?}")` → `PartialEq` 직접 비교

#### 완료 (성능 최적화)
- [x] Reduce 연산 네이티브화 (Sum, Product, Min, Max, Avg 타입별 최적화)
- [x] 문자열 연산 최적화 (SPLIT 용량 예약, JOIN 직접 추출, CHARS 예약)
- [x] 병렬 연산 구현 (ParallelMap, ParallelFilter, ParallelReduce with Rayon)
- [x] 체인 연산 융합 (MapReduce, FilterReduce, MapFilter, MapFilterReduce - 중간 배열 없이 단일 패스)

### 0.2 성능 목표

| 연산 | Python | AOEL VM | AOEL JIT |
|------|--------|---------|----------|
| Map (1000) | 27.4µs | 24.7µs ✅ | - |
| Filter (1000) | 28.0µs | 24.0µs ✅ | - |
| Factorial(20) | 1030ns | - | 48ns ✅ (21x) |
| Fibonacci(20) | 922µs | - | 60µs ✅ (15x) |

### 0.3 JIT 안정성 ✅
- [x] 더 많은 opcode JIT 지원 (LoadLocal, StoreLocal 추가)
- [x] JIT 컴파일 에러 처리 개선 (인터프리터 폴백)
- [x] Hot path 자동 JIT 컴파일 (프로파일러 기반, 임계값 100회)

---

## Phase 1: Language Features ✅

### 1.1 Pattern Matching ✅
```aoel
// match 표현식
result = match value {
  0 => "zero",
  1..10 => "small",
  n if n > 100 => "large",
  _ => "other"
}

// destructuring
(a, b) = get_point()
{name, age} = get_person()
[first, ...rest] = get_list()
```

**구현 완료:**
- [x] Match 표현식 파서
- [x] 패턴 AST 노드 (Pattern enum)
- [x] 패턴 타입 체크
- [x] IR/VM 구현
- [x] 테스트 추가

### 1.2 Module System ✅
```aoel
// math.aoel
export add(a, b) = a + b
export sub(a, b) = a - b
private helper(x) = x * 2

// main.aoel
import { add, sub } from "./math"
import * as math from "./math"
```

**구현 완료:**
- [x] import/export 구문 파서
- [x] 모듈 해석기 (ModuleResolver)
- [x] 순환 의존성 감지
- [x] 네임스페이스 관리
- [x] 테스트 추가

### 1.3 Error Handling ✅
```aoel
// Result 타입
read_file(path) -> Result<String, Error>

// try/catch
try {
  data = read_file("config.json")
  config = json_parse(data)
} catch e {
  println("Error: " + e.message)
}

// ? 연산자 (early return)
config = read_file("config.json")?
```

**구현 완료:**
- [x] Result/Option 타입
- [x] try/catch 구문 (TryCatch AST, SetCatch/ClearCatch IR)
- [x] ? 연산자 (Try opcode)
- [x] ?? 연산자 (Coalesce opcode)
- [x] 에러 전파 (catch_stack)
- [x] 테스트 추가

### 1.4 Generic Types ✅
```aoel
// 제너릭 함수
map<T, U>(arr: [T], f: T -> U) -> [U] = arr.@(f)

// 제너릭 타입
type Stack<T> = {
  items: [T],
  push: T -> Stack<T>,
  pop: () -> (T, Stack<T>)
}
```

**구현 완료:**
- [x] 타입 파라미터 파서 (`<T, U>` 구문)
- [x] TypeParam, TypeVar, Generic AST 노드
- [x] Type::Var, substitute, contains_var, free_vars
- [x] 제너릭 타입 추론 (Hindley-Milner)
- [x] 테스트 추가

### 1.5 Macro System (우선순위: 낮음)
```aoel
// 컴파일 타임 코드 생성
macro debug!(expr) {
  println!("DEBUG: {} = {}", stringify!(expr), expr)
}

debug!(x + y)  // => println("DEBUG: x + y = ", x + y)
```

### 1.6 Async/Await (우선순위: 낮음)
```aoel
async fetch_data(url) = {
  response = await http_get(url)
  await json_parse(response.body)
}

// 병렬 실행
[a, b, c] = await all([
  fetch_data(url1),
  fetch_data(url2),
  fetch_data(url3)
])
```

---

## Phase 2: Tools & Infrastructure ✅

### 2.1 REPL Enhancement ✅
- [x] 명령어 히스토리 (rustyline)
- [x] 자동완성 (Tab) - rustyline 기본 지원
- [x] 멀티라인 입력 (\ 줄 끝에서 계속)
- [x] `:help`, `:clear`, `:save`, `:load`, `:history` 명령어
- [x] `:type`, `:ast` 검사 명령어
- [ ] 구문 강조 (향후 개선)

### 2.2 Debugger ✅
- [x] 브레이크포인트 설정 (set_breakpoint, conditional)
- [x] 스텝 실행 (step, step_into, step_out, continue)
- [x] 변수 검사 (get_variable, locals)
- [x] 콜스택 표시 (call_stack)
- [x] 감시 표현식 (add_watch, remove_watch)
- [x] CLI 통합 (`aoel debug`)
- [ ] DAP (Debug Adapter Protocol) 지원 (향후 추가)

### 2.3 Profiler ✅
- [x] 함수별 실행 시간 (FunctionProfile)
- [x] 호출 횟수, min/max/avg 시간
- [x] CLI 통합 (`aoel profile`)
- [x] JSON 출력 (to_json)
- [x] 요약 출력 (summary)
- [ ] 메모리 사용량 (향후 추가)
- [ ] flame graph 생성 (향후 추가)

### 2.4 Documentation Generator ✅
- [x] AST에서 문서 추출 (FunctionDoc, TypeDoc, ConstDoc)
- [x] Markdown 출력
- [x] HTML 출력 (스타일 포함)
- [x] JSON 출력
- [x] 타입 시그니처 자동 생성
- [x] CLI 통합 (`aoel doc`)
- [ ] 주석에서 doc comment 추출 (향후 추가)
- [ ] 예제 코드 실행 검증 (향후 추가)

---

## Phase 3: Ecosystem

### 3.1 Package Registry
- [ ] 온라인 패키지 저장소
- [ ] 버전 관리 (semver)
- [ ] 의존성 해결
- [ ] 보안 검사
- [ ] 검색/탐색 UI

### 3.2 VS Code Extension
- [ ] LSP 클라이언트
- [ ] 구문 강조 (TextMate)
- [ ] 스니펫
- [ ] 디버거 통합
- [ ] 테스트 러너

### 3.3 Playground Enhancement
- [ ] 더 많은 예제
- [ ] 튜토리얼 모드
- [ ] 코드 공유 (permalink)
- [ ] 실행 시간 표시
- [ ] 모바일 지원

### 3.4 Community
- [ ] 공식 문서 사이트
- [ ] 튜토리얼/가이드
- [ ] 예제 프로젝트
- [ ] Discord/Forum

---

## Version Milestones

| Version | Target | Key Features |
|---------|--------|--------------|
| v0.1.0 | Done ✅ | Core language, JIT, Basic tools |
| v0.2.0 | Done ✅ | Pattern matching, Module system |
| v0.3.0 | Done ✅ | Error handling, Generics |
| v0.4.0 | Done ✅ | REPL enhancement, Debugger, Profiler, DocGen |
| v0.5.0 | - | Package registry, VS Code extension |
| v1.0.0 | - | Stable API, Full ecosystem |

---

## Contributing

기여를 환영합니다! 다음 영역에서 도움이 필요합니다:
- 버그 리포트 및 수정
- 문서화
- 테스트 추가
- 새로운 stdlib 함수
- 플랫폼별 테스트

## License

MIT
