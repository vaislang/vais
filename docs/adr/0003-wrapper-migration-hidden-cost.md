# ADR 0003 — Wrapper Migration의 Hidden Cost 평가 의무

**Status**: Accepted (Phase Ω Task #35, iter 124, 2026-04-28)
**Decision driver**: P1.4 (Type-Tagged IR Builder) 17 iter 작업에서 카테고리 B 사이트의 wrapper migration이 결정적 negative (3회 재현)로 확증됨. ADR 0001/0002 R1+R2+R3 게이트를 모두 통과해도 wrapper migration 자체가 production impact를 악화시킬 수 있음을 발견.
**Depends on**: [ADR 0001](0001-root-cause-definition.md), [ADR 0002](0002-codegen-invariants.md)

---

## 배경 (Context)

ADR 0002는 codegen 4 클래스 invariant + AI multi-session protocol을 명세화했다. P1.4는 그 위에서 "TypedEmitter API로 763 산발 사이트를 단일 API로 수렴"을 목표로 진행. 17 iter (104~120) 후 close 결정 — **카테고리 A만 안전 확정**.

**관찰된 wrapper migration의 hidden cost** (P1.4 iter 110/116/119, 3회 동일 패턴):

ret-load 사이트 (`stmt_visitor.rs:729`)를 `emit_load_with_prefix("ret.", ...)` wrapper로 마이그레이션 시도:
- 시도 1 (iter 110): -0.6 file → REVERTED (variance 의심)
- 시도 2 (iter 116): -0.4 file, ±2 (vs ±0.5 baseline), threshold 도달 → REVERTED (CLAUDE 규칙 4)
- 시도 3 (iter 119): RegisterPolicy::Skip으로 자동 register 회피 → 평균 -1.4 file, min 218 → REVERTED

3회 모두 R1+R2+R3 게이트 통과:
- R1 (invariant): "ret-load 결과의 LLVM 타입이 emit 후 register됨"
- R2 (차단 테스트): TypedEmitter unit test 824/0 PASS
- R3 (audit): 동일 사이트 패턴 grep, 카테고리 분류 명문화

그럼에도 production impact가 negative. **ADR 게이트 외 영역에 hidden cost가 존재**.

### Hidden cost의 가능 메커니즘

1. **Rust borrow split 변화** — wrapper 호출 시 `counter mut + fn_ctx mut + ir mut` 패턴이 release 빌드 inlining 결정을 바꿈
2. **추가 alloc** — `loaded_temp.name().to_string()` 같은 헬퍼 호출에서 String 신규 할당
3. **TypedTemp 생성/destruct** — wrapper 반환값의 lifecycle 자체가 cost
4. **후속 self method 호출과의 borrow scope 차이** — defer_cleanup 같은 호출이 wrapper context에서 다른 codepath 선택

## 결정 (Decision)

**wrapper migration 시도 시 R1+R2+R3 외에 추가로 R4 (Hidden Cost Audit)를 충족 의무**한다.

### R4 — Hidden Cost Audit

다음 4 항목을 wrapper migration commit 전 측정한다:

#### R4.1 — Per-site 5-run baseline 측정
- migration 적용 전 5-run check-integrity 실행
- 각 run의 vaisdb_files count + spread 기록
- 평균 + min + max + variance 4 통계 모두 명시

#### R4.2 — Per-site 5-run post-migration 측정
- migration 적용 후 동일 5-run 실행
- pre/post 비교: 평균 delta, variance delta, min delta
- **threshold**: 평균 -0.5 file 이상 OR variance 2배 이상 증가 OR min count -1 이상 하락 → revert 의무

#### R4.3 — Migration 자체의 Rust 변경 영향 분석
- borrow pattern: 추가 mut 참조 카운트 (before/after)
- alloc 카운트: 추가 String / Box / Vec 할당 식별
- inlining 변화 가능성: `cargo rustc --release -- --emit asm` diff (선택)

#### R4.4 — 카테고리 분류 명시
- 본 사이트가 P1.4 카테고리 A/B/C/D 중 어디인지 명시
- LLVM 타입 단일성 (단일/다양/조건적/일반화)
- positive expected (카테고리 A) / negative risk (카테고리 B/C/D) 사전 평가

### Migration 결정 게이트

```
R1+R2+R3 통과
  ↓
R4.1+R4.2 측정
  ↓
R4.4 카테고리 분류:
  - 카테고리 A → R4.2 통과 시 LANDED
  - 카테고리 B/C/D → R4.2 통과 + R4.3 hidden cost 분석 필수
  ↓
R4.2 threshold 위반 시 → REVERT 의무 (CLAUDE 규칙 4 강화)
```

### 적용 범위

- **의무 적용**: TypedEmitter wrapper / coerce_ret_value wrapper / resolve_index_access wrapper / 향후 신설되는 모든 codegen wrapper API의 사이트 마이그레이션
- **면제**: 신규 사이트 추가 (마이그레이션 아님), wrapper 자체의 API 변경 (사이트 무영향)

---

## 결과 (Consequences)

### 긍정
- wrapper migration의 production impact를 사전에 측정 → 카테고리 A 외 사이트의 negative impact 차단
- ADR 0001/0002 게이트가 보장하지 못하는 영역 (Rust 컴파일러 영향) 명시
- P1.4 iter 110/116/119 같은 3회 반복 시도의 cost 절감

### 부정
- per-site 5-run 측정이 8~10분 / 사이트 (deterministic protocol 적용 시)
- 모든 wrapper migration commit 의무화로 throughput 감소
- R4.3 (Rust 변경 영향 분석)이 정성적 — automation 어려움

### 위험 완화
- R4.1/R4.2 5-run은 이미 P1.4 deterministic protocol (iter 114) 인프라로 자동화 가능
- R4.3은 권장 사항으로 격하 가능 (사이트 위험도가 낮을 때)
- R4.4 카테고리 분류는 ADR 0002 §"AI multi-session protocol" 와 통합

---

## 참고

- P1.4 17-iter 학습 출처: `lang/packages/vaisdb/ROADMAP.md` iter 104~120 entries
- 관련 memory: phase_omega_p14_close_2026-04-28.md, phase_omega_iter116_117_session_2026-04-27.md
- ADR 0002 §"수정된 invariant 3" amendment와 함께 LANDED (P1.4 763 사이트 단일 API 수렴 한계)

---

## 적용 시점

- **2026-04-28 (이 문서 채택일)부터 모든 신규 wrapper migration commit에 적용**
- 기존 LANDED 마이그레이션 (P1.3 resolve_index_access Path 1+2+3, P1.4 stmt_visitor.rs:708 ret-cast)은 retro 적용 면제
- 향후 wrapper migration 시도는 R4 충족 commit message에 명시 의무
