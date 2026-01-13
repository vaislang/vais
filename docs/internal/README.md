# Vais Internal Design Documents

This folder contains internal design documents for the Vais language.
These documents serve as references for language designers, compiler developers, and core contributors.

## Language / 언어 선택

| Language | Description |
|----------|-------------|
| [English](en/README.md) | English documentation |
| [한국어](ko/README.md) | 한국어 문서 |

---

## Document Overview

| Document | English | 한국어 |
|----------|---------|--------|
| Architecture | [en/architecture.md](en/architecture.md) | [ko/architecture.md](ko/architecture.md) |
| Core Design | [en/core-design.md](en/core-design.md) | [ko/core-design.md](ko/core-design.md) |
| FFI Design | [en/ffi-design.md](en/ffi-design.md) | [ko/ffi-design.md](ko/ffi-design.md) |
| Standard Library | [en/stdlib.md](en/stdlib.md) | [ko/stdlib.md](ko/stdlib.md) |
| Package System | [en/package-system.md](en/package-system.md) | [ko/package-system.md](ko/package-system.md) |
| Extension Guide | [en/extension-guide.md](en/extension-guide.md) | [ko/extension-guide.md](ko/extension-guide.md) |

---

## Design Philosophy

Vais follows these principles:

1. **Small Core + Big Ecosystem** - Keep core minimal, implement most features as libraries
2. **AI-First Design** - Structure that AI can efficiently generate and understand
3. **Token Efficiency** - 30-60% token reduction compared to Python
4. **Functional-First** - Functional programming first approach

---

## 설계 철학

Vais는 다음 원칙을 따릅니다:

1. **Small Core + Big Ecosystem** - 코어는 최소화, 대부분은 라이브러리로
2. **AI-First Design** - AI가 효율적으로 생성하고 이해할 수 있는 구조
3. **Token Efficiency** - Python 대비 30-60% 토큰 절감
4. **Functional-First** - 함수형 프로그래밍 우선

---

## Directory Structure

```
docs/internal/
├── README.md           # This file (index)
├── en/                 # English documentation
│   ├── README.md
│   ├── architecture.md
│   ├── core-design.md
│   ├── ffi-design.md
│   ├── stdlib.md
│   ├── package-system.md
│   └── extension-guide.md
├── ko/                 # Korean documentation (한국어 문서)
│   ├── README.md
│   ├── architecture.md
│   ├── core-design.md
│   ├── ffi-design.md
│   ├── stdlib.md
│   ├── package-system.md
│   └── extension-guide.md
└── ../archive/         # Historical documents (과거 버전)
```

## Historical Documents

For design documents from previous versions, see the `../archive/` folder.
과거 버전별 설계 문서는 `../archive/` 폴더를 참조하세요.
