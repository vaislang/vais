# New Vais 설계 — 인덱스

**최종 갱신**: 2026-06-13
**상태**: 종이 설계 + 데이터 검증 + 프로토타입/self-host baseline 완료. 사용자-facing 이름은 **New Vais / Vais** 로 확정.

---

## 이게 무엇인가

기존 Legacy Vais를 **실패 사례 데이터**로 삼아, "AI(바이브코딩)가 가장 정확하게 쓰는 언어"를 백지에서
설계하고 구현하는 작업. 2026-06-06 세션에서 방향·원칙·문법·검증을 **실측 데이터로** 정립했고,
2026-06-13에 새 Vais mainline으로 확정했다.

---

## 한 줄 결론 (데이터 기반)

> **"AI가 잘 쓰는 언어"는 만들 수 있다** (예제 코퍼스 + 모호성 0; cold-start 1/5→6/6 실측).
> **그러나 "범용 Rust 대체"는 별개의 훨씬 어려운 문제다** (생태계 해자는 설계로 못 푼다).
> 이 둘을 분리하는 것이 가장 중요하다.

---

## 문서 읽는 순서

1. **`ai-native-competitiveness-assessment-2026-06-06.md`** — 왜/경쟁력/시장.
   경쟁 현실(생태계 해자), 실측 실험(생성·속도·에러). **기대치 조정용.**
2. **`next-language-from-failures-2026-06-06.md`** — 설계 원칙 P1~P9.
   Vais 8버그+7함정 → 근본원인 → 원칙. **설계 철학.**
3. **`new-language-grammar-v0.2-2026-06-06.md`** — 최신 문법 명세 (v0.1은 전신).
   모든 구문이 한 가지로 확정. **프로토타입의 입력.**
4. **`new-language-ambiguity-audit-2026-06-06.md`** — 모호성 감사.
   v0.1의 결함 발견 → v0.2가 해결. **감사 방법론.**
5. **`new-vais-compiler-mainline-2026-06-13.md`** — New Vais 확정 후 자체 컴파일러 mainline 계약.

---

## 핵심 설계 원칙 (P1~P9, 전부 실측 근거)

| 원칙 | 내용 | Vais 실패 차단 |
|------|------|---------------|
| P1 | 키워드≠식별자, 영어 단어, 단일문자 금지 | `A`=async 충돌 |
| P2 | 한 토큰=한 의미 | `~`=mut/NOT 모호 |
| P3 | 한 작업=한 문법 | `Color::Red` vs unqualified |
| P4 | 에러에 `help:`+수정코드 | turbofish cryptic 에러 |
| P5 | prelude+toolchain-global std | `use std/vec`+환경의존 |
| P6 | 흔한 패턴 ceremony 없이 | `=> return` 거부 |
| P7 | 단일 지점 coercion | f32 10-site 산재 |
| P7b | 컴파일≠정답, 값-정확성 게이트 | vaisdb 정렬 무음오류 |
| P8 | 클로저=`{code,env}` 1급, day-1 확정 | 캡처 유실(vaisdb 정렬) |
| **P9** | **검증된 예제 코퍼스 1급 인프라** | **cold-start 1/5→5/5 (최강 레버)** |

**메타 규칙**: *직관 수용 ≠ 두 길 허용.* 둘 다 열면 모호성 = Vais 반복.

---

## 검증 데이터 (실측, 2026-06-06)

| 측정 | 결과 |
|------|------|
| 예제 코퍼스 효과 (신규 AI, 5과제) | 예제 없음 1/5 → 예제 있음 5/5 |
| 새 언어 v0.1 cold-start (5과제) | 5/5 (Vais 동일과제 1/5) |
| 새 언어 v0.2 cold-start (새 구문 6과제) | 6/6 |
| 컴파일 속도 (vaisc vs rustc) | 전체 동급, 프론트엔드 2배 빠름 |
| 런타임 성능 | 동급 (같은 LLVM, 압도 불가) |

---

## 현재 진입점 (자체 컴파일러 mainline)

**목표**: Legacy Vais bootstrap/oracle을 유지하면서 New Vais 자체 컴파일러를 mainline으로 전진.

**전략** (실패 줄이는 순서):
1. **현재 baseline 보존** — `scripts/test.sh` 112/112, self-host e2e, full-source stage compare green 유지.
2. **제품 경계 유지** — `scripts/vaisc emit-ir/build/run`이 `.vais`/`.nl` 입력을 받고 Legacy oracle smoke와 비교된다.
3. **front 계약 유지** — native subset은 Int 함수/let/return/if/while/plain call, print/putchar, simple struct, List push/len/index slice이며 unsupported 문법은 `help:` 진단으로 실패한다.
4. **직접 LLVM emitter 분리** — `scripts/vaisc --engine direct`가 최소 arithmetic `main`을 Legacy Vais 없이 emit/build/run하며, 이후 `compiler/self/fixpoint_full.nl` seed에서 Legacy Vais 트랜스파일 의존을 단계적으로 줄임.
5. **P4 에러 UX 유지** — native `vaisc` day-1 오류는 source 좌표, 원문 line, caret, `help:`, `fix:`를 포함한다.
6. **parity manifest 유지** — `tools/vaisc-parity.tsv`가 native-supported/bootstrap-only/tracked 예제와 self-host tier를 기록하고 `scripts/test-vaisc-parity.sh`가 값 비교한다.
7. **구현이 노출하는 새 함정을 문서화** → 설계 갱신.

**재활용 가능 자산** (Legacy Vais):
- bootstrap/oracle 경로, LLVM codegen 경험, 타입체커/추론 노하우, self-host 노하우, 빠른 프론트엔드, 게이트 패턴.
**폐기**: 단일문자 키워드, 토큰절약, 산재 coercion, bare-fn-ptr 클로저.

**경고** (Vais 교훈):
- broad 작업은 incremental + 규칙4(회귀 시 revert).
- 종이 설계는 깔끔해도 구현은 함정 노출 — 작게 시작.
- 값-정확성을 처음부터 게이트로 (컴파일 성공에 속지 말 것).

---

## 정직한 한계 (잊지 말 것)

- 종이 설계 + 작은 표본(5~6 과제) 검증. 구현 시 새 함정 확실.
- "AI가 잘 쓴다"는 검증됨 ≠ "시장에서 성공". 생태계·채택은 코드로 못 푼다.
- 이 작업의 가치는 **학습/연구로서 최고** (self-host 언어를 직접 설계·구현). 시장 성공과 분리해 평가.

---

## 병행: Legacy Vais 안정성 (bootstrap/oracle)

New Vais 자체 컴파일러가 parity를 얻을 때까지 기존 Vais 컴파일러는 Legacy bootstrap/oracle로 유지한다.
이미 캡처 클로저 ABI, Vec/Map/string 등 주요 갭은 값-정확성 회귀로 잠겨 있다.
