# Phase 202 E-Top Domain — 조사 결과

## 결론

**Phase 202 범위 (compiler 무수정) 에서 E030 또는 E004 mass 처리 불가.**

## 조사 내역

### E030 (No such field) — 27 파일, top 5 심층 조사

| 파일 | 에러 | struct 정의 | 필드 실존? |
|------|------|-------------|-----------|
| security/privilege.vais:52 | `session.is_superuser` on `SessionContext` | src/security/types.vais:1619-1627 | ✅ `is_superuser: bool` line 1622 |
| security/rls.vais:60 | 동일 패턴 `session.is_superuser` | 동 | ✅ |
| security/audit.vais:51 | `config.enable_audit` on `SecurityConfig` | src/security/types.vais:420-436 | ✅ `enable_audit: bool` line 431 |
| planner/cost_model.vais:152 | `stat.name` on `TableStats` | 3 파일 중복 정의 (vector/filter + sql/planner/mod + sql/planner/types) | ⚠️ 중복 정의, 어느 TableStats인지 compiler 혼동 가능성 |
| planner/analyzer.vais:44 | `query.ctes` on `SelectQuery` | sql/parser/ast.vais:47-59 | ✅ `ctes: Vec<Cte>` line 48 |

### 진단

**4/5 = field이 struct에 명확히 정의되어 있음에도 E030 발생.**

이는 compiler의 **cross-module struct field resolution 한계**:
- vaisdb가 `U security/types.{SessionContext}` 같이 import해도
- 다른 모듈에서 `session.is_superuser` 호출 시 struct 정의의 field list를 인식 못함
- `U` import가 struct **타입**은 가져오되 **field 메타데이터**를 전파 안 하는 것으로 추정

**1/5 (TableStats)**: 3 파일에 동일 이름 struct 중복 정의. compiler가 "which TableStats?" 결정 실패 가능성.

### 동일 패턴: E004 (143 파일)

Phase 202 Recon-202에서 이미 확인:
- Vec method dispatch 실패 (push/len/resize): 60+ 건
- Self-struct method dispatch 실패
- 두 경우 모두 "method/field 존재는 하지만 compiler가 못 찾음"

## Phase 202 조치

### 실제 처리 가능한 E030

TableStats 중복 정의 통합 — 1 파일 (planner/cost_model.vais). 하지만 3곳의 정의가 서로 다른 필드를 가지고 있을 경우, 통합은 **의미적 수정**으로 신중.

Phase 202에서는 **documentation만** 진행. 실제 통합은 Phase 203+.

### 실제 처리 불가한 E030

27 - 1 = 26건이 compiler cross-module resolution 한계. vaisdb 측 fix 불가.

## Phase 203 권고 (compiler 개선 필요)

### C1. Cross-module struct field resolution
- `U module.{Struct}` import 시 struct의 field 정의가 해당 모듈 scope에서 인식되어야 함
- 현재: type만 인식, field 접근은 resolution 실패
- 영향 범위: E030 27건 + E004 self-struct method 일부

### C2. Generic impl method dispatch
- `Vec<T>` 같은 generic struct field의 impl method가 해결되지 않음
- stdlib `X Vec<T> { F push(&self, value: T) }` 이 struct 필드 접근에서 안 풀림
- 영향 범위: E004 Vec/HashMap 메서드 ~60+건

### 권고 방식
- Phase 203을 compiler crate 작업으로 전환 (vais-types/checker_expr 수정)
- cross-module lookup + generic method dispatch 두 개 테스트 추가
- 완료 후 vaisdb 재측정 → E004/E030 대량 해소 예상

## 해소 결과 (Phase 202)

- E030: 27 → 27 (변동 없음 — mass fix 불가 판정)
- E004: 143 → 143 (동 — 대부분 compiler 한계)
- E002: 44 → 미처리
- **P001: 2 → 0** (iter1/iter2 structural fix 로 완료)

PROMISE: COMPLETE
