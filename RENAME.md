# 언어 이름 변경 가이드

이 프로젝트의 언어 이름은 **가칭 `nl`** (new-language)이다. 의미 없는 중립 코드네임이므로
검색/상표 충돌이 없고, 진짜 이름이 정해지면 아래 지점만 바꾸면 된다.

## ⚠️ 이름 확정 전 반드시 사용자가 검증할 것 (AI가 대신 못 함)
- 웹 검색: `<이름> programming language`, `<이름> lang`
- GitHub / crates.io / npm 동명 프로젝트
- 상표 검색 (USPTO, KIPRIS 등) — 진지하게 갈 거면 변호사

## 이름이 박힌 지점 (전부 여기 명시 — 분산 금지 원칙)

| 지점 | 현재 값 | 변경 방법 |
|------|---------|-----------|
| 1. 프로젝트 폴더 | `projects/nl/` | `git mv nl <newname>` |
| 2. 파일 확장자 | `.nl` | 예제 일괄 `rename` + 트랜스파일러 UNSUPPORTED/usage 텍스트 |
| 3. 트랜스파일러 파일명 | `compiler/transpiler/nl2vais.py` | `git mv` + 내부 usage 문자열 |
| 4. 빌드 스크립트 | `scripts/build.sh`의 확장자/경로 | 스크립트 내 변수 |
| 5. 코드/문서 내 산문상 "nl" | docstring, README, 설계문서 | 일괄 치환 (단어 경계 주의: 영어 "nl"은 흔치 않아 안전) |

## 변경 절차 (예: nl → Foo)
```bash
cd projects
git mv nl foo                                  # 1. 폴더
cd foo
# 2. 확장자: 예제 .nl → .foo
for f in examples/*.nl; do git mv "$f" "${f%.nl}.foo"; done
# 3. 트랜스파일러 파일명 + 내부 문자열
git mv compiler/transpiler/nl2vais.py compiler/transpiler/foo2vais.py
sed -i '' 's/\.nl\b/.foo/g; s/nl2vais/foo2vais/g' compiler/transpiler/foo2vais.py
# 4~5. 산문 치환 (검토 후)
grep -rl '\bnl\b' . --include='*.md' --include='*.py'   # 먼저 확인
# 그다음 신중히 sed 치환
```

## 주의
- 확장자 `.nl`은 단순 코드네임 — `.foo` 등으로 바꿔도 트랜스파일러 로직과 무관(텍스트만).
- **로직에는 이름이 하드코딩돼 있지 않다** (NAME 상수/확장자만 텍스트). 이름 변경이 로직을 안 깬다.
- 변경 후 `bash scripts/build.sh examples/c4.nl` 로 동작 재확인.
