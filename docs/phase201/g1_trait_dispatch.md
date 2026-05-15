# Phase 201 G1-Trait — MetaUpdater + TableSqlProvider trait dispatch

## 결정

**Phase 201 현재 범위에서는 "stub + 문서화" 선택**, Phase 202+에서 실제 trait 도입.

### 근거
1. 대상 함수 3개 (`update_meta` x 2, `dump_to_string`/`restore`/`verify_restore`) 모두 **callee 0건** (dead code)
2. 현재 호출이 없으므로 trait 설계 + caller 업데이트 작업이 premature
3. Phase 201 목표 (P001 ≤5) 달성을 위해 stub으로 P001 제거 후 진행

## 처리 결과

### A. fulltext/concurrency.vais & vector/concurrency.vais — `update_meta`

**원본**:
```vais
F update_meta(self, update_fn: Fn(FullTextMeta)) -> Result<(), VaisError> {
    meta_guard := mut M self.meta.write() { ... }
    update_fn(mut *meta_guard)
    Ok(())
}
```

**변경**: 함수 전체 제거 (호출자 없음 — grep 전수 확인).
주석 placeholder 추가:
```vais
# update_meta (Fn(T) callback) removed — dead code, callers absent.
# Phase 201 decision (docs/phase200/p1_decisions.md): Fn(T) 미지원.
# 나중에 trait dispatch가 필요하면 MetaUpdater trait 도입.
```

### B. ops/dump.vais — `dump_to_string`, `restore`, `verify_restore`

**원본** (3 callback 파라미터 포함):
```vais
F dump_to_string(
    self,
    table_names: &Vec<str>,
    get_create_table_sql: F(str) -> Result<str, VaisError>,
    get_index_sql: F(str) -> Result<Vec<str>, VaisError>,
    get_row_sql: F(str) -> Result<Vec<str>, VaisError>,
) -> Result<(str, DumpResult), VaisError> { ... }
```

**변경**: 3 함수 모두 stub 버전으로 교체 (callable 인자 제거, `Err(DumpProvider trait 필요)` 반환).

## Phase 202+ 권장 Trait 설계

### MetaUpdater trait (concurrency용)

```vais
W MetaUpdater<T> {
    F apply(self, meta: mut T);
}

# fulltext 사용
S MarkRebuilt {}
X MarkRebuilt: MetaUpdater<FullTextMeta> {
    F apply(self, meta: mut FullTextMeta) {
        meta.needs_rebuild = false
    }
}

# ConcurrentFullTextIndex 측
F update_meta<U: MetaUpdater<FullTextMeta>>(self, updater: U) -> Result<(), VaisError> {
    meta_guard := mut M self.meta.write() { Ok(g) => g, Err(_) => { R Err(...) }, }
    updater.apply(mut *meta_guard)
    Ok(())
}
```

### DumpProvider trait (dump.vais용)

```vais
W DumpProvider {
    F get_create_table_sql(self, table_name: str) -> Result<str, VaisError>;
    F get_index_sql(self, table_name: str) -> Result<Vec<str>, VaisError>;
    F get_row_sql(self, table_name: str) -> Result<Vec<str>, VaisError>;
    F execute_sql(self, stmt: str) -> Result<u64, VaisError>;
    F get_row_count(self, table_name: str) -> Result<u64, VaisError>;
}

# SqlDumper 측
F dump_to_string<P: DumpProvider>(
    self,
    table_names: &Vec<str>,
    provider: &P,
) -> Result<(str, DumpResult), VaisError> { ... }
```

## 호환성 영향

- **현재 vaisdb 빌드**: dead code 제거 + stub 추가로 영향 없음
- **vaisdb 외부 소비자**: 해당 함수 미사용 (grep 전수 0건)
- **Phase 202+ 마이그레이션**: 본 문서의 trait 설계로 진행. 사용처 발견 시 구체 struct 정의 (예: `MarkRebuilt`) 추가

## 해소

- fulltext/concurrency.vais P001 ✅
- vector/concurrency.vais P001 ✅
- ops/dump.vais P001 ✅ (3 instance 모두)

총 4 P001 해소 (dump.vais 내부 cascade 포함).

PROMISE: COMPLETE
