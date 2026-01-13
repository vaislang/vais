# Vais 내부 설계 문서

이 폴더는 Vais 언어의 내부 설계 문서를 포함합니다.
이 문서들은 언어 설계자, 컴파일러 개발자, 핵심 기여자를 위한 참고 자료입니다.

## 문서 목록

| 문서 | 설명 |
|------|------|
| [architecture.md](architecture.md) | 확장성 중심 전체 아키텍처 설계 |
| [core-design.md](core-design.md) | 코어 언어 설계 원칙 |
| [ffi-design.md](ffi-design.md) | FFI (Foreign Function Interface) 설계 |
| [stdlib.md](stdlib.md) | 표준 라이브러리 설계 |
| [package-system.md](package-system.md) | 패키지 시스템 설계 |
| [extension-guide.md](extension-guide.md) | 생태계 확장 가이드 |

## 설계 철학

Vais는 다음 원칙을 따릅니다:

1. **Small Core + Big Ecosystem** - 코어는 최소화, 대부분은 라이브러리로
2. **AI-First Design** - AI가 효율적으로 생성하고 이해할 수 있는 구조
3. **Token Efficiency** - Python 대비 30-60% 토큰 절감
4. **Functional-First** - 함수형 프로그래밍 우선

## 히스토리 문서

과거 버전별 설계 문서는 `../archive/` 폴더를 참조하세요.

## 다른 언어

- [English](../en/README.md)
