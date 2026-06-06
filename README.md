# nl — AI-native 프로그래밍 언어 (가칭)

> **이름은 가칭 `nl`(new-language)이다.** 의미 없는 중립 코드네임 — 진짜 이름은 미정.
> 변경 방법: `RENAME.md`. (이름 확정 전 상표/충돌 검증은 사용자 몫 — AI가 대신 못 함.)

---

## 무엇인가

**"AI(바이브코딩)가 가장 정확하게 쓰는 언어"**를 백지에서 설계·구현하는 프로젝트.
기존 언어 **Vais를 실패 사례 데이터**로 삼아, 그 함정을 설계로 원천 차단한다.

설계·검증은 전부 **실측 데이터** 기반 (`docs/design/` 참조). 핵심 결과:
- ✅ "AI가 잘 쓰는 언어"는 만들 수 있다 — cold-start 정확도 예제유무 1/5→5/5, nl 문법 5/5~6/6 (실측).
- ❌ "범용 Rust 대체"는 별개의 훨씬 어려운 문제 — 생태계 해자는 설계로 못 푼다.
  → 이 둘을 분리하는 것이 핵심. (`docs/design/ai-native-competitiveness-assessment-*.md`)

---

## 설계 원칙 (P1~P9, 전부 실측 근거)
`docs/design/next-language-from-failures-*.md` 참조. 요약:
- **모호성 0** (P1 키워드≠식별자/영어단어, P2 한토큰한의미, P3 한작업한문법)
- **중앙화** (P7 단일 coercion, P8 클로저 `{code,env}` day-1)
- **AI 자동수정 에러** (P4 `help:`+수정코드)
- **값-정확성 게이트** (P7b 컴파일≠정답)
- **예제 코퍼스 1급 인프라** (P9 — 최강 레버, cold-start 약점 해소)
- 메타규칙: *직관 수용 ≠ 두 길 허용.*

---

## 폴더 구조

```
nl/
├── README.md            이 파일
├── RENAME.md            이름 변경 가이드
├── docs/design/         설계 문서 (경쟁력/원칙 P1~P9/문법 v0.1·v0.2/모호성 감사)
├── compiler/
│   └── transpiler/      nl → Vais 트랜스파일러 (현재 백엔드 = Vais 재활용)
│       └── nl2vais.py
├── examples/            .nl 예제 = cold-start 코퍼스 (P9 인프라)
├── tests/               값-정확성 e2e (P7b) — TODO
└── scripts/
    └── build.sh         .nl 빌드 (트랜스파일 + vaisc)
```

---

## 현재 상태 (트랜스파일 프로토타입)

**전략**: Vais 백엔드 재활용. nl 표면 문법을 Vais로 트랜스파일해 vaisc로 컴파일.
- AI가 쓴 nl 코드 **12/13 실제 컴파일+실행** (트랜스파일러 미지원 0).
- 유일한 실패: Vais 백엔드의 filter 버그 (트랜스파일 문제 아님).

```bash
scripts/build.sh examples/c4.nl -o /tmp/c4 && /tmp/c4   # struct 예제 → 42
```

### 천장 (정직한 한계 — docs README 상세)
트랜스파일은 **nl 표면이 Vais 표면과 1:1일 때만** 작동한다. nl의 핵심 차별점(P7 단일 coercion,
P8 클로저 day-1, P4 에러)은 **Vais로 표현 불가** — Vais 천장(버그·산재coercion·bare클로저)에 갇힌다.
→ **진짜 nl의 가치를 보려면 자체 컴파일러(L3)**: lexer/parser/types/codegen. 다세션 작업.

---

## 로드맵 (수준별)

| 수준 | 내용 | 상태 |
|------|------|------|
| L1 | nl 표면 동작 + AI 정확 생성 | ✅ 완료 (cold-start 5/5, 6/6) |
| L2 | 트랜스파일러가 v0.2 표현 | ✅ 사실상 완료 (12/13) |
| **L3** | **자체 컴파일러 — P7/P8 day-1 구현** | 🔲 **다음 (이 폴더의 목표)** |
| L4 | 프로덕션 (생태계) | 🔲 별개 거대 문제 |

### L3 시작 시 결정할 것
- **컴파일러 작성 언어**: Rust(vais처럼) / 또는 self-host 부트스트랩 / 다른 언어.
- **백엔드**: Vais 재활용 → 점진적 자체 codegen(LLVM 직접) / 또는 LLVM 바로.
- **에러 인프라(P4)**: `help:`+수정코드를 day-1에.
- (이 결정들은 사용자 의도 필요 — 추측 금지.)

---

## 출처
이 프로젝트는 2026-06-06 Vais 세션에서 분기됨. 원본 설계·실측은 vais repo의
`docs/design/`에도 보존(Vais 실패 분석이라 vais에도 의미 있음). nl 폴더는 그 사본 + 독립 진행.
