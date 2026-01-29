# 부록

## 용어집

### 컴파일러 용어

- **AST (Abstract Syntax Tree)**: 추상 구문 트리. 소스 코드의 구조를 나타내는 트리 형태의 자료구조
- **LLVM**: Low Level Virtual Machine. 컴파일러 백엔드 프레임워크
- **IR (Intermediate Representation)**: 중간 표현. 소스 코드와 기계어 사이의 중간 단계 표현
- **Inkwell**: Rust용 LLVM 바인딩 라이브러리
- **Monomorphization**: 단형화. 제네릭 코드를 구체적인 타입으로 특수화하는 과정

### 언어 기능

- **제네릭 (Generics)**: 타입을 매개변수화하여 코드를 재사용하는 기능
- **트레잇 (Trait)**: 타입이 구현해야 하는 동작을 정의하는 인터페이스
- **타입 추론 (Type Inference)**: 명시적 타입 표기 없이 컴파일러가 타입을 자동으로 결정하는 기능
- **패턴 매칭 (Pattern Matching)**: 값의 구조를 검사하고 분해하는 기능
- **컴파일 타임 평가 (Compile-time Evaluation)**: 컴파일 시점에 코드를 실행하는 기능

### 도구

- **LSP (Language Server Protocol)**: 에디터와 언어 서버 간의 표준 프로토콜
- **DAP (Debug Adapter Protocol)**: 디버거와 IDE 간의 표준 프로토콜
- **REPL (Read-Eval-Print Loop)**: 대화형 프로그래밍 환경
- **JIT (Just-In-Time)**: 실행 시점에 코드를 컴파일하는 방식

## 추가 리소스

### 공식 리소스

- [GitHub 저장소](https://github.com/sswoo88/vais)
- [이슈 트래커](https://github.com/sswoo88/vais/issues)
- [토론 포럼](https://github.com/sswoo88/vais/discussions)

### 관련 프로젝트

- [LLVM](https://llvm.org/) - 컴파일러 백엔드
- [Inkwell](https://github.com/TheDan64/inkwell) - Rust LLVM 바인딩
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)

### 학습 자료

- [Rust 프로그래밍 언어](https://doc.rust-lang.org/book/) - Vais 컴파일러가 작성된 언어
- [LLVM Tutorial](https://llvm.org/docs/tutorial/) - LLVM 사용법
- [Writing An Interpreter In Go](https://interpreterbook.com/) - 인터프리터 구현 가이드

## 라이센스

Vais는 MIT 라이센스 하에 배포됩니다.

```
MIT License

Copyright (c) 2025 Vais Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
