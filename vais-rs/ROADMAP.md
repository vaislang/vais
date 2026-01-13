# Vais Language Roadmap

> **Vais** = **V**ibe **AI** **S**cript - AI-assisted "vibe coding"을 위한 프로그래밍 언어

---

## 🎯 Current Version: v0.0.5

### 완료된 기능

#### Core Language
- ✅ **Lexer** - 토큰화, 유니코드 지원
- ✅ **Parser** - Pratt parser, 연산자 우선순위, Coalesce (??) 연산자
- ✅ **Type Checker** - Hindley-Milner 타입 추론, 제네릭 타입 시스템
- ✅ **IR Lowering** - AST → IR 변환, 최적화
- ✅ **VM** - 스택 기반 인터프리터, 100+ 내장 함수

#### JIT Compiler (Cranelift)
- ✅ Integer/Float 연산 JIT 컴파일
- ✅ 재귀 함수 (TCO)
- ✅ 조건문/비교 연산
- ✅ Hot path 자동 JIT (프로파일러 기반)
- ✅ **15-75x Python 대비 성능 향상**

#### Language Features
- ✅ **Pattern Matching** - match 표현식, destructuring
- ✅ **Module System** - import/export, 순환 의존성 감지
- ✅ **Error Handling** - try/catch, ?, ?? 연산자
- ✅ **Generic Types** - TypeScheme 기반 다형성, 타입 추론

#### Code Generation
- ✅ C 코드 생성
- ✅ WASM/WAT 생성
- ✅ LLVM IR 생성

#### Performance Optimizations
- ✅ `hash_key()` 효율적 해싱 (10-50x 개선)
- ✅ Fused operations (MapReduce, FilterReduce 등)
- ✅ Parallel operations (Rayon + ParallelContext 최적화)
- ✅ Native loop optimizations
- ✅ Checked arithmetic (integer overflow 보호)
- ✅ Arc-based function sharing

#### Tools
- ✅ **CLI** - run, build, check, format, repl, debug, profile, doc
- ✅ **LSP Server** - 자동완성, 진단, hover
- ✅ **REPL** - 히스토리, 멀티라인, :commands
- ✅ **Debugger** - 브레이크포인트, 스텝 실행, 변수 검사
- ✅ **Profiler** - 함수 타이밍, JSON 출력
- ✅ **Doc Generator** - Markdown, HTML, JSON

#### Ecosystem
- ✅ **Package Manager** - init, add, remove, publish
- ✅ **VS Code Extension** - LSP, 구문 강조, 스니펫
- ✅ **Web Playground** - 브라우저에서 실행

#### Standard Library (100+ functions)
- ✅ Collections (len, first, last, reverse, sort, unique, etc.)
- ✅ Math (abs, sqrt, pow, sin, cos, log, etc.)
- ✅ Strings (upper, lower, trim, split, join, etc.)
- ✅ File I/O, JSON, HTTP, Time, Random

#### Quality
- ✅ 522+ 단위 테스트
- ✅ 31개 통합 테스트
- ✅ 벤치마크

---

## 📊 Performance

| Operation | Python | Vais VM | Vais JIT |
|-----------|--------|---------|----------|
| Map (1000 elements) | 27.4µs | 24.7µs | - |
| Filter (1000 elements) | 28.0µs | 24.0µs | - |
| Factorial(20) | 1030ns | - | 48ns (21x faster) |
| Fibonacci(20) | 922µs | - | 60µs (15x faster) |

---

## 🚀 Future Plans (v1.x / v2.0)

### 언어 기능
| 기능 | 설명 | 우선순위 |
|------|------|----------|
| Macro System | 컴파일 타임 코드 생성 | 낮음 |
| Async/Await | 비동기 프로그래밍 | 중간 |
| Traits/Interfaces | 타입 추상화 | 중간 |
| Algebraic Effects | 부작용 관리 | 낮음 |

### 도구 개선
| 기능 | 설명 | 우선순위 |
|------|------|----------|
| DAP Support | VS Code 디버거 통합 | 중간 |
| Flame Graph | 프로파일러 시각화 | 낮음 |
| Memory Profiler | 메모리 사용량 분석 | 낮음 |
| Test Runner | 내장 테스트 프레임워크 | 중간 |

### 생태계
| 기능 | 설명 | 우선순위 |
|------|------|----------|
| Online Registry | 패키지 저장소 서버 | 높음 |
| Documentation Site | 공식 문서 웹사이트 | 높음 |
| Code Sharing | Playground 영구 링크 | 중간 |
| Mobile Support | Playground 모바일 UI | 낮음 |

---

## 📝 Version History

| Version | Date | Highlights |
|---------|------|------------|
| **v0.0.5** | 2026-01-13 | Generic type system (TypeScheme), Coalesce operator, ParallelContext 최적화 |
| v0.0.4 | 2026-01-13 | Checked arithmetic, error handling 개선 |
| v0.0.3 | 2026-01-13 | 프로젝트명 AOEL → Vais 변경, 문서 구조화 (en/ko) |
| v0.0.2 | 2026-01-12 | Package registry, VS Code extension, Playground |
| v0.0.1 | 2026-01-11 | Initial release, Core language, JIT |

---

## 🤝 Contributing

기여를 환영합니다!

- 🐛 버그 리포트 및 수정
- 📖 문서화 개선
- ✅ 테스트 추가
- 📦 새로운 stdlib 함수
- 🌐 다국어 지원

자세한 내용은 [CONTRIBUTING.md](docs/CONTRIBUTING.md)를 참조하세요.

## License

MIT
