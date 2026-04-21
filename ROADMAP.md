# Vais — Next Drive Candidates

> **상태**: 신규 드라이브 선택 대기
> **직전 드라이브**: `ROADMAP-fh-drive.md` ("struct-in-array-literal → Phase 4.x SCOPED", 3/3 완료 2026-04-21, 마지막 커밋 `b74376b0`)
> **더 이전 드라이브**: `ROADMAP-cascade-drive.md` (4/4 완료)
> **더 이전**: `ROADMAP-compiler-drive.md` (11/11 완료)
> **아카이브**: `ROADMAP-archive.md` (Phase 0 ~ 6.31)

---

## 현재 baseline (2026-04-21)

| 항목 | 숫자 |
|------|------|
| compiler_syntax | 200/200 |
| compiler_stages | 14/14 (1 #[ignore] B7) |
| std_files | 82/82 |
| living_spec | 117/117 |
| phase158 strict | 18/18 |
| vaisdb_files | 237/261 (90.8%) |
| vaisc e2e | 2625/0/1 |
| vaisc integration | 147/147 |

모든 다음 드라이브는 이 baseline 을 **regression floor** 로 유지.

---

## 직전 드라이브들의 잔여 항목 (다음 드라이브 후보 풀)

### 가장 명확히 정의된 후보 (이 compiler 저장소 내)

#### 1. Vec<Struct> write-through (B.4 후속) ⭐ 작고 명확
**목적**: D.2 + F.1 이후 `v: Vec<Point> := [...]; v[0].x = 99` 로 literal
초기화 된 Vec 의 원소 수정이 실제 버퍼에 반영되지 않음. B.4 write-through
가 Vec<scalar>만 처리하고 Vec<Struct> 는 memcpy-to-temp 경로로 fallback.

**target**: B.4 관련 codegen 경로 (추정: `expr_helpers_data.rs` 또는
`generate_expr` 인덱스-write-back 로직).

**완료 기준**: `v: Vec<Point> := [Point{x:1,y:2}, Point{x:3,y:4}]; v[0].x = 99;
R v[0].x + v[1].x` → 102.

**리스크**: 중간. B.4 는 기존 Vec<scalar> 로직이 섬세함. 전체 memcpy→GEP
변환은 위험. Vec<Struct> 만 분기하는 쪽이 안전.

#### 2. E034 flow-sensitive analysis (`contains_key → insert → get!` 패턴)
**목적**: H.1 에서 발견한 가장 가치 있는 E034 개선. Flow-sensitive 분석으로
`insert` 후 `get!` 가 안전함을 증명. vaisdb 4 E034 중 window.vais 1 개
확실히 unblock 가능.

**target**: `crates/vais-types/src/totality.rs` 의 walk 확장 + 간단한
control-flow tracking.

**리스크**: 높음. Phase 4.x SCOPED, Phase 158 요요 경고. 범위 폭발 위험.
별도 "purity drive" 로 계획 후 진행 권장.

#### 3. 매크로 + dyn trait 완전 vtable (Phase 4.21)
**목적**: LANGUAGE_SPEC Matrix 의 `◐` 항목들 중 일부 (macro, dyn trait full
vtable). 사용자가 실제로 dyn trait 많이 쓰지 않으면 우선순위 낮음.

**리스크**: 매우 높음. Phase 4.21 SCOPED 전체 재설계. 장기 프로젝트.

#### 4. `Vec::new()` 를 `std/vec.vais` 에 추가 (초소형)
**목적**: 현재 `Vec.with_capacity(0)` 만 가능. `Vec::new()` 관용구 지원.

**target**: `std/vec.vais` ~5 줄 추가.

**리스크**: 최소. 단순 stdlib 확장.

### 다른 저장소 작업 후보 (이 세션에서 불가)

#### 5. vaisdb cleanup drive
**위치**: `/Users/sswoo/study/projects/vais/lang/packages/vaisdb/` (별도 저장소).
**목적**: 20 bucket (b) bugs 정리. 237 → 250+/261 목표.

**왜 별도인가**: vaisdb CLAUDE.md 규칙 별도, `check-integrity.sh` 없음,
BufferPool API 재설계 (`write_page` 2-arg → 1-arg + frame.data 흐름) 등
9+ 사이트 개별 판단 필요. compiler 저장소 세션에서는 안전망 밖.

**시작 방법**: `cd /Users/sswoo/study/projects/vais/lang/packages/vaisdb && /harness`.
`docs/vaisdb-cascade-survey.md` (이 저장소) §3.2 가 가이드.

---

## 선택 방식

사용자가 1/2/3/4/5 또는 조합을 지정하면 이 파일을 해당 드라이브의 실행
ROADMAP 으로 재작성.

추천 우선순위 (Opus 판단):
- **1번 (Vec<Struct> write-through)**: 작고 가치 명확. D.2+F.1 자연 마무리.
- **4번 (Vec::new in std)**: 5줄 stdlib 확장, trivial.
- **5번 (vaisdb cleanup)**: 사용자 체감 가장 큰 효과, 저장소 이동 필요.
- **2번 / 3번**: 범위 큰 투자 필요, 별도 세션 + 계획 권장.

---

## 드라이브 외 단발성 과제 (언제든 가능)

- **LANGUAGE_SPEC.md Matrix `◐` 행 재검증** (한 번 훑기).
- **examples/ 디렉토리 정리** (deprecated 예제 제거).
- **`docs/LANGUAGE_SPEC.md` L262 최종 ✓ 선언** (Vec<Struct> write-through 완성 시).

---

## 작업 시작 시

드라이브 선택 후:
1. 이 파일을 새 드라이브 ROADMAP 으로 재작성.
2. 현재 baseline 재측정 (`./scripts/check-integrity.sh`) → 고정.
3. `harness-init` 또는 `harness-plan` 이 나머지 처리.
