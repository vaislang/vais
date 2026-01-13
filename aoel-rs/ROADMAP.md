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

## Phase 1: Language Features

### 1.1 Pattern Matching (우선순위: 높음)
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

**구현 항목:**
- [ ] Match 표현식 파서
- [ ] 패턴 AST 노드
- [ ] 패턴 타입 체크
- [ ] IR/VM 구현
- [ ] 테스트 추가

### 1.2 Module System (우선순위: 높음)
```aoel
// math.aoel
export add(a, b) = a + b
export sub(a, b) = a - b
private helper(x) = x * 2

// main.aoel
import { add, sub } from "./math"
import * as math from "./math"
```

**구현 항목:**
- [ ] import/export 구문 파서
- [ ] 모듈 해석기
- [ ] 순환 의존성 감지
- [ ] 네임스페이스 관리
- [ ] 테스트 추가

### 1.3 Error Handling (우선순위: 중간)
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

**구현 항목:**
- [ ] Result/Option 타입
- [ ] try/catch 구문
- [ ] ? 연산자
- [ ] 에러 전파
- [ ] 테스트 추가

### 1.4 Generic Types (우선순위: 중간)
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

**구현 항목:**
- [ ] 타입 파라미터 파서
- [ ] 제너릭 타입 추론
- [ ] 단형화 (monomorphization)
- [ ] 타입 제약조건
- [ ] 테스트 추가

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

## Phase 2: Tools & Infrastructure

### 2.1 REPL Enhancement
- [ ] 명령어 히스토리 (readline)
- [ ] 자동완성 (Tab)
- [ ] 구문 강조
- [ ] 멀티라인 입력
- [ ] `.help`, `.clear`, `.save` 명령어

### 2.2 Debugger
- [ ] 브레이크포인트 설정
- [ ] 스텝 실행 (step in/over/out)
- [ ] 변수 검사
- [ ] 콜스택 표시
- [ ] DAP (Debug Adapter Protocol) 지원

### 2.3 Profiler
- [ ] 함수별 실행 시간
- [ ] 메모리 사용량
- [ ] 핫스팟 감지
- [ ] flame graph 생성
- [ ] CLI 통합 (`aoel profile`)

### 2.4 Documentation Generator
- [ ] 주석에서 문서 추출
- [ ] Markdown/HTML 출력
- [ ] 타입 시그니처 자동 생성
- [ ] 예제 코드 실행 검증
- [ ] 검색 기능

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
| v0.1.0 | Done | Core language, JIT, Basic tools |
| v0.2.0 | - | Pattern matching, Module system |
| v0.3.0 | - | Error handling, Generics |
| v0.4.0 | - | REPL enhancement, Debugger |
| v0.5.0 | - | Package registry, VS Code |
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
