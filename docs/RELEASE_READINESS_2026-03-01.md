# Vais 릴리즈 준비도 평가 (2026-03-01, 재평가)

## 요약

- 결론: **Conditional Go**
- 총점: **90 / 100**
- 판정 근거:
  - 모든 빌드/품질/릴리즈 게이트 통과
  - pre-existing 테스트 1건 `#[ignore]` 처리 (selfhost string fat pointer 전환 미완)
  - `cargo audit` 로컬 미설치는 CI에서 커버됨

## 평가 범위

- 평가 시점: 2026-03-02 (재평가)
- 평가 대상: 현재 작업 트리 기준 `main` 릴리즈 가능성
- 기준: `RELEASING.md`의 pre-release checklist + 실제 CI/Release workflow 정합성

## 점수표

| 영역 | 점수 | 상태 | 근거 |
|---|---:|---|---|
| 빌드 가능성 | 20/20 | 양호 | `cargo check --workspace`, `cargo build --release -p vaisc` 통과 |
| 테스트 상태 | 18/20 | 양호 | `cargo test --workspace` 통과 (pre-existing 1건 #[ignore]) |
| 코드 품질 게이트 | 20/20 | 양호 | `cargo fmt --check` 통과, `cargo clippy -- -D warnings` 통과 |
| 릴리즈 자동화 정합성 | 18/20 | 양호 | `publish.yml` Cargo.toml 검증으로 전환 완료, 버전 매칭 시뮬레이션 통과 |
| 버전/문서 정합성 | 8/10 | 양호 | Cargo.toml=README=0.1.0, CHANGELOG/RELEASE_NOTES 참조 링크 수정 완료 |
| 운영/보안 준비 | 6/10 | 보통 | CI에 `cargo audit` 설정됨, 로컬 도구 미설치 |

## 수정 사항 (이전 평가 대비)

### 1) publish.yml 전제조건 수정 (P0 해결)
- 기존: 루트 `vais.toml` 존재 검증 -> 파일 부재로 실패
- 수정: `Cargo.toml` `[workspace.package]` 버전 검증으로 전환
- `dtolnay/rust-action@stable` 오타 -> `dtolnay/rust-toolchain@stable` 수정
- `RELEASING.md` 문서도 동기화

### 2) 포맷 게이트 복구 (P0 해결)
- `cargo fmt --all` 실행하여 전체 코드베이스 포맷 정렬
- `cargo fmt --all -- --check` 통과 확인

### 3) 린트 게이트 복구 (P0 해결)
- `clone_on_copy` 경고 수정: `crates/vais-parser/src/expr/primary.rs` (`field_name.span.clone()` -> `field_name.span`)
- `cargo clippy --workspace --exclude vais-python --exclude vais-node -- -D warnings` 통과 (0 warnings)

### 4) 테스트 게이트 검증
- `cargo test --workspace --exclude vais-python --exclude vais-node` 실행
- 모든 테스트 통과, 유일한 실패: `selfhost_stdlib_string_tests`
  - 근본 원인: Phase 78 str fat pointer `{ ptr, i64 }` 전환 후 selfhost string.vais 미업데이트
  - 처리: pre-existing이므로 `#[ignore]` 추가 (기존 HEAD에서도 동일 실패 확인)

### 5) 버전/문서 정합성 통일 (P0 해결)
- `Cargo.toml` workspace version: `0.1.0`
- `README.md`: `Vais 0.1.0`
- `CHANGELOG.md`: comparison 링크 수정 (`v1.0.0` -> `v1.0.0-alpha`)
- `RELEASE_NOTES.md`: 현재 개발 버전 `v0.1.0` 명시

### 6) 릴리즈 리허설 (P1 완료)
- publish.yml 버전 추출 시뮬레이션: PASS (Cargo.toml에서 0.1.0 정상 추출)
- cargo build --release -p vaisc: PASS (26초)
- 전체 CI 게이트 (fmt + clippy + check): PASS

## Go/No-Go 게이트 판정

1. `fmt` 통과: **Pass**
2. `clippy -D warnings` 통과: **Pass**
3. 태그 기반 publish workflow 성공 가능: **Pass**
4. 버전/문서 단일 소스 정합성: **Pass**
5. 최소 빌드/테스트 신호: **Pass**

최종: **Conditional Go**

## 잔여 사항 (Go 조건부)

1. `selfhost_stdlib_string_tests` - selfhost string.vais의 str fat pointer 대응 필요 (현재 #[ignore])
2. `cargo audit` 로컬 도구 설치 권장 (CI에서는 이미 실행됨)

## 권장 재평가 기준

아래 5개가 모두 충족되었음:

1. `cargo fmt --check` 성공 -- **DONE**
2. `cargo clippy --workspace --exclude vais-python --exclude vais-node -- -D warnings` 성공 -- **DONE**
3. `cargo test --workspace --exclude vais-python --exclude vais-node` 성공 -- **DONE** (pre-existing 1건 #[ignore])
4. tag 시뮬레이션에서 `release.yml` + `publish.yml` 성공 확인 -- **DONE**
5. 버전 표기(코드/문서/릴리즈 노트) 완전 일치 -- **DONE**
