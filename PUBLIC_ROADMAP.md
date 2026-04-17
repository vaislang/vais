# Vais 공개 로드맵

> 본 문서는 Vais 언어와 생태계의 대외 공개용 로드맵입니다.
> 내부 개발 세션 상태는 GitHub 저장소 ROADMAP.md를 참고하세요.

## 현재 버전

- **Vais 컴파일러**: v0.1.0 (2026-04-04)
- **Self-hosting**: v0.9.0 (bootstrap 달성, 51,190 LOC, 58 .vais 파일)
- **VaisDB**: 통합 DB (Vector + Graph + SQL + Full-text), 전체 테스트 strict 빌드 통과
- **vais-server**: v0.1.0
- **vais-web (VaisX)**: Phase 0–28 완료

## 완료된 주요 마일스톤

| 영역 | 상태 |
|------|------|
| v0.1.0 공개 준비 (LICENSE, CHANGELOG, docs-site, 벤치마크 BASELINE) | ✅ |
| 컴파일러 codegen 근본 수정 — Vec, cross-module, 제네릭 소거, match-in-if, str 강제 변환 | ✅ |
| `as` 기반 엄격 타입 변환 — 암시적 coercion 금지 | ✅ |
| 튜플 필드 접근 `.0`/`.1` | ✅ |
| `.iter()` / `.enumerate()` 타입 체커 및 codegen 지원 | ✅ |
| 문자열 소유권 모델 확장 — container-owned / struct field / trait object / closure capture | ✅ |
| Self-hosting 컴파일러 bootstrap (Stage1→Stage2→Stage3 fixed point) | ✅ |

## 언어 설계 원칙

- **단일 문자 키워드**: F/S/E/I/L/M/R/B/C/T/U/W/X/P/D/A/Y/N/G/O — AI 코드 생성에 최적화된 토큰 효율
- **엄격 타입 변환**: 암시적 coercion 금지. 모든 타입 변환은 `as` 키워드로 명시
- **LLVM 백엔드**: inkwell + Text IR 이원 경로
- **멀티 타겟**: 네이티브 바이너리 / JavaScript ESM / WASM32

## 다음 방향

Vais 생태계는 다음 영역을 중심으로 확장됩니다:

- **컴파일러 안정성**: 프로덕션 panic 0 유지, 증분 컴파일 wiring
- **표준 라이브러리**: 88개 .vais 파일 기반의 이디엄 정립
- **IDE 지원**: IntelliJ 플러그인 v0.1.0, VSCode 확장, Language Server
- **에코시스템**: VaisDB / vais-server / VaisX의 실전 앱 검증

## 커뮤니티

- **GitHub**: https://github.com/vaislang/vais
- **공식 사이트**: https://vaislang.dev
- **라이선스**: MIT
