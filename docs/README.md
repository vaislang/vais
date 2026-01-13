# Vais Documentation

Welcome to the Vais programming language documentation.

## Documentation Structure

```
docs/
├── en/           # English documentation
├── ko/           # 한국어 문서
├── internal/     # Internal design documents
└── archive/      # Historical documents
```

## Quick Links

### English Documentation
- [Getting Started](en/getting-started.md) - Installation and first program
- [Syntax Guide](en/syntax.md) - Complete language syntax
- [API Reference](en/api.md) - Built-in functions
- [Examples](en/examples.md) - Code examples
- [Contributing](en/contributing.md) - How to contribute

### 한국어 문서
- [시작 가이드](ko/getting-started.md) - 설치 및 첫 프로그램
- [문법 가이드](ko/syntax.md) - 전체 언어 문법
- [API 레퍼런스](ko/api.md) - 내장 함수
- [예제](ko/examples.md) - 코드 예제
- [기여 가이드](ko/contributing.md) - 기여 방법

### Internal Documents (설계 문서)
- [Internal Docs Index](internal/README.md) - All internal documents
- English: [Architecture](internal/en/architecture.md) | [Core Design](internal/en/core-design.md) | [FFI](internal/en/ffi-design.md) | [Stdlib](internal/en/stdlib.md) | [Package System](internal/en/package-system.md) | [Extension Guide](internal/en/extension-guide.md)
- 한국어: [아키텍처](internal/ko/architecture.md) | [코어 설계](internal/ko/core-design.md) | [FFI](internal/ko/ffi-design.md) | [표준 라이브러리](internal/ko/stdlib.md) | [패키지 시스템](internal/ko/package-system.md) | [확장 가이드](internal/ko/extension-guide.md)

## About Vais

**Vais (Vibe AI Script)** is a programming language designed for AI to generate, modify, and execute code most efficiently.

Key features:
- **Token-Efficient Syntax** - 30-60% fewer tokens than Python
- **Functional-First** - First-class functions, closures, collection operations
- **Self-Recursion** - `$` operator for elegant recursive definitions
- **Multiple Backends** - Interpreter, JIT (50-75x faster), C, WASM, LLVM

## License

MIT License - See [LICENSE](../LICENSE) for details.
