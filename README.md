# New Vais — AI-native 프로그래밍 언어

> **공식 방향(2026-06-13)**: 이 프로젝트는 **새 Vais(New Vais)** 로 확정한다.
> `nl`은 repo/확장자/스크립트가 안정될 때까지 유지하는 구현 코드명이다.
> 기존 `/Users/sswoo/study/projects/vais/compiler`는 **Legacy Vais bootstrap backend** 로만 유지한다.

---

## 무엇인가

**"AI(바이브코딩)가 가장 정확하게 쓰는 언어"**를 백지에서 설계·구현한 새 Vais 프로젝트.
기존 언어 **Legacy Vais**를 실패 사례 데이터와 bootstrap backend로 삼아, 그 함정을 설계로 원천 차단한다.

설계·검증은 전부 **실측 데이터** 기반 (`docs/design/` 참조). 핵심 결과:
- ✅ "AI가 잘 쓰는 언어"는 만들 수 있다 — cold-start 정확도 예제유무 1/5→5/5, New Vais 문법 5/5~6/6 (실측).
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
nl/                         # 전환기 repo 코드명. 사용자-facing 언어명은 New Vais.
├── README.md            이 파일
├── RENAME.md            이름 변경 가이드
├── docs/design/         설계 문서 (경쟁력/원칙 P1~P9/문법 v0.1·v0.2/모호성 감사)
├── compiler/
│   └── transpiler/      New Vais(.nl) → Legacy Vais 트랜스파일러 (bootstrap backend)
│       └── nl2vais.py
├── examples/            .nl 예제 = New Vais cold-start 코퍼스 (P9 인프라)
├── tools/
│   └── vaisc.py         New Vais compiler CLI 구현
├── tests/               값-정확성 e2e (P7b) — TODO
└── scripts/
    ├── vaisc            New Vais compiler command wrapper
    └── build.sh         Legacy Vais bootstrap 빌드 경로
```

---

## 현재 상태

**전략**: New Vais 표면 문법을 Legacy Vais로 트랜스파일해 `vaisc`로 검증하고,
동시에 자체 컴파일러 라인을 mainline으로 전환한다.
- 현재 value-corpus **112/112 실제 컴파일+실행 통과** (`scripts/test.sh`, 2026-06-13).
- New Vais compiler 명령 계약 **`scripts/vaisc`** 추가: `.vais`/`.nl` 입력 → LLVM IR emit/build/run,
  Legacy bootstrap oracle 비교 smoke 통과(`scripts/test-vaisc.sh`).
- `scripts/vaisc` day-1 native front 계약 추가: Int 함수/let/return/if/while/plain call subset은 통과,
  for/struct/list/string/Rust식 `&&` 등은 source 좌표와 `help:` 진단으로 거절(`scripts/test-vaisc-front.sh`).
- `scripts/vaisc --engine direct` 추가: 단일 `fn main() -> Int { return <Int expr> }` slice를 Legacy Vais 없이
  LLVM IR로 직접 emit/build/run하고 bootstrap engine과 값 일치 검증(`scripts/test-vaisc-direct.sh`).
- native `vaisc` P4 진단 확장: Rust식 `&&`/`||`/`as`/`::`/타입명/turbofish와 direct parse 실패가
  source 좌표, 원문 line, caret, `help:`, `fix:`를 포함(`scripts/test-vaisc-errors.sh`).
- 초기 실패였던 Legacy Vais filter 버그와 캡처 클로저 반환 ABI 갭은 production 경로에서 해결 확인.

```bash
scripts/vaisc emit-ir program.vais -o /tmp/program.ll   # New Vais self compiler가 LLVM IR emit
scripts/vaisc build program.vais -o /tmp/program        # clang까지 연결
scripts/vaisc run program.vais --engine direct          # NV-C2 최소 direct LLVM emitter
scripts/test-vaisc-errors.sh                            # NV-C3 native P4 diagnostics
scripts/build.sh examples/c4.nl -o /tmp/c4              # Legacy bootstrap oracle 경로
```

### 전환 원칙
Legacy Vais 백엔드는 당분간 **oracle/bootstrap** 으로 유지한다. 폴더명 `nl`, 확장자 `.nl`,
`nl2vais.py`는 게이트 안정성을 위해 즉시 rename하지 않는다. 자체 컴파일러가 예제 코퍼스와
self-host 게이트를 대체할 만큼 parity를 얻으면 물리적 rename과 legacy 의존 축소를 진행한다.

---

## 로드맵 (수준별)

| 수준 | 내용 | 상태 |
|------|------|------|
| L1 | New Vais 표면 동작 + AI 정확 생성 | ✅ 완료 (cold-start 5/5, 6/6) |
| L2 | 트랜스파일러가 v0.2 표현 | ✅ 사실상 완료 (12/13) |
| **L3** | **자체 컴파일러 — P7/P8/P4 day-1 구현** | 🚧 **mainline으로 진입** |
| L4 | 프로덕션 (생태계) | 🔲 별개 거대 문제 |

### L3 결정 사항
- **언어명**: 사용자-facing 이름은 New Vais/Vais로 확정. `nl`은 전환기 코드명.
- **컴파일러 작성 언어**: New Vais self-host 라인을 우선한다. 현재 `compiler/self/fixpoint_full.nl`이 seed.
- **백엔드**: Legacy Vais는 bootstrap/oracle로 유지하고, 자체 컴파일러는 직접 LLVM IR emit 경로로 전진한다.
- **명령명**: New Vais compiler의 공식 명령 계약은 `vaisc`. 전환기에는 repo-local `scripts/vaisc`로 실행해
  Legacy `vaisc` 바이너리와 충돌을 피한다.
- **front 계약**: native day-1은 Int 스칼라 subset으로 고정하고, 넓은 언어 표면은 Legacy bootstrap 또는 후속 NV-C2~C4에서 다룬다.
- **에러 인프라(P4)**: `help:`+수정코드와 값-정확성 게이트를 day-1 계약으로 둔다.
- **mainline 계약**: `docs/design/new-vais-compiler-mainline-2026-06-13.md`.

---

## 출처
이 프로젝트는 2026-06-06 Legacy Vais 세션에서 분기됨. 원본 설계·실측은 vais repo의
`docs/design/`에도 보존되어 있다. 이 repo는 이제 New Vais의 mainline이다.

---

## AI 에이전트(Codex 등)로 작업한다면

> **시작 전 [`AGENTS.md`](AGENTS.md)를 끝까지 읽어라.** 검증된 작업 방법론, 두 repo 경계
> (New Vais/nl ↔ Legacy Vais 백엔드), 게이트(안전망) 실행법, 함정 카탈로그(silent miscompile / 8-bit 절단 /
> cache race / line-기반 트랜스파일러 한계 / `{{`-이스케이프)가 모두 거기 있다.
> 진실의 원천은 `ROADMAP.md`(현재-only), 최근 맥락은 `WORKLOG.md`(최신이 맨 위).
