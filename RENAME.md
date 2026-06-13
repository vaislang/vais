# 이름/전환 결정

**2026-06-13 결정**: 이 프로젝트의 사용자-facing 언어명은 **New Vais / Vais** 로 확정한다.
**2026-06-14 migration**: New Vais repo는 `/Users/sswoo/study/projects/vais`로 물리 이동했고,
기존 Legacy Vais repo는 `/Users/sswoo/study/projects/vais-legacy`로 백업 이동했다.
`nl`은 과거 구현 코드명으로만 남는다.
트랜스파일러의 canonical 이름은 `legacy_vais_bootstrap.py`로 바꿨고, 기존 `nl2vais.py`는 호환 래퍼로만 남긴다.
checked-in New Vais 소스 확장자는 `.vais`로 전환했고, `.nl` transitional 호환성은
`scripts/test-vais-extension-migration.sh`로 검증한다.
lint 도구의 canonical 이름은 `tools/vais-check.py`이며, 기존 `tools/nl-check.py`는 호환 래퍼다.

기존 `/Users/sswoo/study/projects/vais-legacy/compiler`는 **Legacy Vais bootstrap backend** 이다.
새 Vais의 의미론과 예제 코퍼스는 이 repo가 진실의 원천이다.

## 이름이 박힌 지점 (전부 여기 명시 — 분산 금지 원칙)

| 지점 | 현재 값 | 변경 방법 |
|------|---------|-----------|
| 1. 프로젝트 폴더 | `projects/vais/` | 완료. 기존 Legacy repo는 `projects/vais-legacy/` |
| 2. 파일 확장자 | `.vais` checked-in, `.nl` transitional compat | 완료. 호환 제거는 별도 gate |
| 3. New Vais compiler command | `scripts/vaisc` | 최종 설치/배포 때 PATH의 `vaisc`로 승격 |
| 4. legacy bootstrap adapter | `compiler/transpiler/legacy_vais_bootstrap.py` | 완료. `nl2vais.py`는 호환 래퍼 |
| 5. lint command | `tools/vais-check.py` | 완료. `tools/nl-check.py`는 호환 래퍼 |
| 6. 빌드 스크립트 | `scripts/build.sh`의 확장자/경로 | Legacy bootstrap oracle로 유지 |
| 7. 코드/문서 내 산문상 "nl" | 호환 wrapper/역사 기록 | 새 현재 문서는 New Vais/Vais 우선 |

## 지금 하지 않는 것

- `.nl` transitional compatibility 제거
- `nl2vais.py` / `nl-check.py` compatibility wrapper 제거
- repo-local `scripts/vaisc`를 PATH의 전역 `vaisc`로 설치/배포

이미 진행한 것:
- 기존 `/Users/sswoo/study/projects/vais`를 `/Users/sswoo/study/projects/vais-legacy`로 백업 이동했다.
- `/Users/sswoo/study/projects/nl`을 `/Users/sswoo/study/projects/vais`로 물리 이동했다.
- New Vais의 Legacy bootstrap/oracle 기본 경로를 `/Users/sswoo/study/projects/vais-legacy/compiler`로 바꿨다.
- New Vais compiler 명령 계약은 `vaisc`로 고정했다. 전환기에는 repo-local `scripts/vaisc`를 사용한다.
- `.vais` 입력 파일은 `scripts/vaisc` 경로에서 smoke 검증한다.
- `compiler/transpiler/legacy_vais_bootstrap.py`를 canonical legacy adapter로 승격했다.
- `compiler/transpiler/nl2vais.py`는 기존 외부 호출을 깨지 않기 위한 compatibility wrapper로 유지한다.
- `tools/vais-check.py`를 canonical lint command로 승격했다.
- `tools/nl-check.py`는 기존 외부 호출을 깨지 않기 위한 compatibility wrapper로 유지한다.
- checked-in source를 `.vais`로 물리 rename했다.
- `.vais` corpus를 `.nl` mirror로 복사한 compatibility gate를 추가했다.

## 미래 정리 절차
```bash
cd /Users/sswoo/study/projects/vais
# 1. migration gate 재실행
bash scripts/test-vais-extension-migration.sh
# 2. 호환 래퍼 제거 여부 결정
git rm compiler/transpiler/nl2vais.py              # 외부 호출이 모두 사라진 뒤에만
git rm tools/nl-check.py                           # 외부 호출이 모두 사라진 뒤에만
# 3. 산문 치환 (검토 후)
grep -rl '\bnl\b' . --include='*.md' --include='*.py'   # 먼저 확인
# 그다음 신중히 sed 치환
```

## 주의
- 확장자 `.vais`가 현재 checked-in 파일 경로다. `.nl`은 transitional compatibility 입력으로만 유지한다.
- 이름 확정, legacy adapter rename, source 확장자 rename, repo 폴더 물리 rename은 완료됐다.
- 변경 후 최소 `bash scripts/test.sh`, `bash scripts/test-fixpoint-full.sh`,
  `bash scripts/test-fixpoint-full-self.sh`, `bash scripts/test-vaisc.sh`를 모두 통과해야 한다.
