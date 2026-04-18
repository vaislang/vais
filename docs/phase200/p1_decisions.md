# Phase 200 P1-Decision — vaisdb grammar 보류건 결정

## 컨텍스트

Phase 199 Recon-H에서 4종 sub-pattern이 Vais grammar 미지원으로 P001 발생. 본 문서는 각 사례에 대해 (a) compiler grammar 추가, (b) Vais 의도된 제약 (vaisdb 우회), (c) 우회 패턴 결정을 내림. 결정 근거는 Vais 언어 철학 (CLAUDE.md), 단일-문자 키워드 일관성, 토큰 효율성을 기준.

## 1. C17 — `Fn(T)` callable type annotation

### 사례
```vais
F update_meta(self, update_fn: Fn(FullTextMeta)) -> Result<(), VaisError> {
                              ^ P001: found LParen, expected ','
```

파일: src/{fulltext,vector}/concurrency.vais (각 1건)

### 분석
Vais는 단일 문자 keyword 정책 — `F` (function), `Fn`은 식별자로 인식됨. 토큰 stream에서 `update_fn: Fn` 까지는 `var: TypeIdent` 패턴으로 파싱 시도, 다음 `(` 에서 expected `,` (struct field) 또는 `=` (default value) 와 충돌.

Vais closure 문법은 `|x| x*2` 같은 형태 (CLAUDE.md "Closures: |x| x * 2"). callable type annotation 자체는 정의되지 않음.

### 결정: **(b) Vais 의도된 제약 + (c) 우회 패턴**

**근거**:
- Vais는 generic 함수형 시스템보다 **데이터-우선 언어** 설계로 보임
- callable as 1st-class type 도입 시 단일 문자 키워드 (`Fn`) 추가 필요 → 충돌
- AI 토큰 효율성 측면에서 callback API보다 직접 호출 패턴 선호

**우회 패턴**:
1. **trait 기반 dispatch** (Vais 표준):
   ```vais
   W MetaUpdater {
       F apply(self, meta: mut FullTextMeta);
   }
   F update_meta<U: MetaUpdater>(self, updater: U) -> Result<...> { ... }
   ```
2. **closure 인자 inline** (가능하면 callback 제거):
   ```vais
   # Before: update_meta(|meta| meta.x = 1)
   # After: meta_guard := lock(); meta_guard.x = 1
   ```

**vaisdb 적용**: fulltext/vector concurrency.vais의 `update_meta` 두 곳 — trait 정의 추가 또는 caller에서 직접 lock + mutation. Phase 201 작업.

## 2. C15 — `mut` in pattern (`Some((mut x, mut y))`, `Term(mut tq)`)

### 사례
```vais
Some((mut decoded_label_id, mut node_id)) => { ... }
                                       ^ P001: found Mut, expected pattern
Term(mut tq) => { ... }
     ^ P001: found Mut, expected pattern
```

파일: src/graph/index/label.vais, src/fulltext/mod.vais

### 분석
Pattern 내부 mut binding은 Rust ergonomics 기능. Vais는 패턴 destructuring 시 immutable binding만 허용하는 것으로 보임 (CLAUDE.md `M expr { pattern => result }` 명시).

### 결정: **(c) 우회 패턴**

**근거**:
- 패턴은 pure structural matching, mut은 의미적으로 binding 후 변경
- 둘을 분리하면 코드 의도가 명확

**우회 패턴**:
```vais
# Before
Some((mut x, mut y)) => { x = x + 1; ... }

# After
Some((x_orig, y_orig)) => {
    x := mut x_orig
    y := mut y_orig
    x = x + 1
    ...
}
```

또는 mut가 진짜 필요 없으면 단순 제거 (값 사용만 한다면 mut 불필요).

**vaisdb 적용**:
- label.vais:120 `Some((mut decoded_label_id, mut node_id))` — 본문 검토 결과 mut 변경 없음 → mut 제거
- fulltext/mod.vais:665 `Term(mut tq)` — tq를 메서드 호출용이면 ref or 직접 사용

## 3. C14 — `vec![0u8; self.X]` macro with self in count

### 사례
```vais
page_data := mut vec![0u8; self.page_size as u64]
                          ^ P001: found SelfLower, expected macro token
```

파일: src/graph/edge/storage.vais, src/vector/storage.vais (각 1건)

### 분석
`vec!` macro는 macro token만 허용 (literal, ident, simple expr). `self.field` 같은 access expression은 macro context에서 미지원.

Vais 매크로 시스템 (CLAUDE.md "Declarative macro system, vais-macro/") — 의도적으로 limited expansion으로 추측.

### 결정: **(c) 우회 패턴 — `Vec.repeat` 헬퍼**

**근거**:
- Macro 확장기를 expression 전체로 확장하면 token efficiency 저하
- Method call이 더 명시적

**우회 패턴**:
```vais
# Before
page_data := mut vec![0u8; self.page_size as u64]

# After (Vec.repeat 사용)
page_data := mut Vec.repeat(0u8, self.page_size as u64)

# 또는 explicit loop
page_data := mut Vec.with_capacity(self.page_size as u64)
i := mut 0u64
LW i < self.page_size as u64 {
    page_data.push(0u8)
    i = i + 1
}
```

**stdlib 추가 필요**: `Vec.repeat<T>(value: T, count: u64) -> Vec<T>` — std/vec.vais 또는 std/collections.vais. **이 추가는 별도 phase로 분리** (compiler crate 무수정 원칙).

**vaisdb 적용**: 두 파일에서 explicit loop 사용 (Vec.repeat 없는 경우), 또는 stdlib 추가 후 호출.

## 4. C18 — Struct field with fn-type (`F(str) -> Result<...>`)

### 사례
```vais
S OpsConfig {
    get_create_table_sql: F(str) -> Result<str, VaisError>,
                          ^ P001: found LParen, expected ','
}
```

파일: src/ops/dump.vais (1건)

### 분석
Struct field 타입에 함수 시그니처 직접 명시. C17과 본질적으로 같은 한계 — Vais는 1st-class function type annotation 미지원.

### 결정: **(c) 우회 패턴 — type alias + N (extern)**

**근거**:
- Function pointer는 FFI/extern 인터페이스에서 주로 필요
- Struct에 callback 보관은 trait dispatch가 더 자연스러움

**우회 패턴**:
```vais
# Option A: trait dispatch
W TableSqlProvider {
    F get_create_table_sql(self, table: str) -> Result<str, VaisError>;
}

S OpsConfig<P: TableSqlProvider> {
    provider: P,
}

# Option B: type alias for fn pointer (compiler 지원 시)
T CreateTableFn = F(str) -> Result<str, VaisError>;
S OpsConfig {
    get_create_table_sql: CreateTableFn,
}
```

**현재 vaisdb는 Option A 권장** — Vais의 trait/generic이 잘 정의되어 있어 자연스러움. dump.vais는 trait 도입.

## 종합 결정 표

| Pattern | 결정 | vaisdb 적용 | 추가 작업 |
|---------|------|-------------|-----------|
| C17 Fn(T) | (b) 의도된 제약 + trait 우회 | fulltext/vector concurrency.vais MetaUpdater trait | Phase 201 |
| C15 mut in pattern | (c) 우회 — binding 분리 | label.vais, fulltext/mod.vais | Phase 200 P2 가능 |
| C14 vec! self | (c) 우회 — Vec.repeat 또는 loop | graph/edge/storage, vector/storage | Phase 200 P2 가능. stdlib Vec.repeat 추가는 별도 phase |
| C18 fn-type field | (c) 우회 — trait dispatch | ops/dump.vais TableSqlProvider trait | Phase 201 |

## 메타-결정

**Vais는 의도적으로 first-class function annotation을 제한**하는 것으로 판단. 토큰 효율성 + 단일 문자 keyword 일관성을 위해 trait 기반 polymorphism을 권장. 위 4 사례 모두 vaisdb 측 마이그레이션으로 해결 가능 — compiler grammar 변경 불필요.

**Phase 200 후속 작업** (P2/Phase 201로):
- C15 mut in pattern 2건 → Phase 200 P2 시도 (mechanical)
- C14 vec! self 2건 → Phase 200 P2 (explicit loop)
- C17 Fn(T) 2건 → Phase 201 (trait 도입 필요)
- C18 fn-type field 1건 → Phase 201 (trait + generic)

PROMISE: COMPLETE
