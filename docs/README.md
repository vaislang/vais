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

### Internal Documents
- [Architecture](internal/architecture.md) - System architecture
- [Core Design](internal/core-design.md) - Core language design
- [FFI Design](internal/ffi-design.md) - Foreign Function Interface
- [Standard Library](internal/stdlib.md) - Standard library design
- [Package System](internal/package-system.md) - Package manager design

## About Vais

**Vais (Vibe AI Script)** is a programming language designed for AI to generate, modify, and execute code most efficiently.

Key features:
- **Token-Efficient Syntax** - 30-60% fewer tokens than Python
- **Functional-First** - First-class functions, closures, collection operations
- **Self-Recursion** - `$` operator for elegant recursive definitions
- **Multiple Backends** - Interpreter, JIT (50-75x faster), C, WASM, LLVM

## License

MIT License - See [LICENSE](../LICENSE) for details.
