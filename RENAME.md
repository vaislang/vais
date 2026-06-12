# 이름/전환 결정

**2026-06-13 결정**: 이 프로젝트의 사용자-facing 언어명은 **New Vais / Vais** 로 확정한다.
`nl`은 repo 경로, 파일 확장자, 트랜스파일러 이름이 안정된 게이트에 묶여 있기 때문에
전환기 구현 코드명으로 유지한다.

기존 `/Users/sswoo/study/projects/vais/compiler`는 **Legacy Vais bootstrap backend** 이다.
새 Vais의 의미론과 예제 코퍼스는 이 repo가 진실의 원천이다.

## 이름이 박힌 지점 (전부 여기 명시 — 분산 금지 원칙)

| 지점 | 현재 값 | 변경 방법 |
|------|---------|-----------|
| 1. 프로젝트 폴더 | `projects/nl/` | 자체 컴파일러 parity 이후 `git mv nl vais` 검토 |
| 2. 파일 확장자 | `.nl` | parity 이후 `.vais` 또는 유지 여부 결정 |
| 3. 트랜스파일러 파일명 | `compiler/transpiler/nl2vais.py` | `git mv` + 내부 usage 문자열 |
| 4. 빌드 스크립트 | `scripts/build.sh`의 확장자/경로 | 스크립트 내 변수 |
| 5. 코드/문서 내 산문상 "nl" | docstring, README, 설계문서 | 현재는 "repo 코드명" 의미로 보존 |

## 지금 하지 않는 것

- `projects/nl/` 폴더 rename
- `.nl` 확장자 대량 rename
- `nl2vais.py` rename
- 기존 게이트 경로 변경

이 네 가지는 자체 컴파일러가 `scripts/test.sh`, `scripts/test-fixpoint-full.sh`,
`scripts/test-fixpoint-full-self.sh`를 대체하거나 동등하게 통과한 뒤 진행한다.

## 미래 정리 절차 (예: `nl` 물리명 → `vais`)
```bash
cd projects
git mv nl vais                                 # 1. 폴더
cd vais
# 2. 확장자 변경이 결정된 경우에만 예제 .nl → .vais
for f in examples/*.nl; do git mv "$f" "${f%.nl}.vais"; done
# 3. 트랜스파일러 파일명 + 내부 문자열
git mv compiler/transpiler/nl2vais.py compiler/transpiler/vais_bootstrap.py
sed -i '' 's/\.nl\b/.vais/g; s/nl2vais/vais_bootstrap/g' compiler/transpiler/vais_bootstrap.py
# 4~5. 산문 치환 (검토 후)
grep -rl '\bnl\b' . --include='*.md' --include='*.py'   # 먼저 확인
# 그다음 신중히 sed 치환
```

## 주의
- 확장자 `.nl`은 현재 검증 인프라의 일부다. 논리적으로는 코드명이어도, 지금 바꾸면 게이트와 문서 경로가 넓게 흔들린다.
- 이름 확정은 완료됐지만 물리 rename은 별도 migration 작업이다.
- 변경 후 최소 `bash scripts/test.sh`, `bash scripts/test-fixpoint-full.sh`,
  `bash scripts/test-fixpoint-full-self.sh`를 모두 통과해야 한다.
