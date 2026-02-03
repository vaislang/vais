# 빠른 참조 가이드

## 일반 작업

### 문서 빌드 및 미리보기

```bash
# 개발 서버 시작 (자동 새로고침, 브라우저 자동 열기)
cd docs-site
./serve.sh

# 또는 수동으로
mdbook serve --open
```

### 프로덕션 빌드

```bash
# 빌드 스크립트 실행
cd docs-site
./build.sh

# 또는 수동으로
mdbook build
```

### 정리

```bash
cd docs-site
mdbook clean
```

## 문서 편집

### 기존 문서 수정

1. 원본 파일 수정 (예: `/docs/TUTORIAL.md`)
2. 자동으로 반영됨 (include 패턴 사용)
3. 개발 서버가 자동 새로고침

### 새 페이지 추가

1. **파일 생성**
   ```bash
   # 예: 새 튜토리얼 추가
   touch docs-site/src/getting-started/new-tutorial.md
   ```

2. **내용 작성**
   ```markdown
   # 새 튜토리얼

   {{#include ../../../docs/new-tutorial.md}}
   ```

3. **목차에 추가** (`src/SUMMARY.md`)
   ```markdown
   - [새 튜토리얼](./getting-started/new-tutorial.md)
   ```

### 이미지 추가

```markdown
![설명](./images/screenshot.png)
```

이미지 파일은 `src/images/` 디렉토리에 저장

## 마크다운 구문

### 코드 블록

\`\`\`vais
F main() {
    print("Hello!")
}
\`\`\`

### 링크

```markdown
[내부 링크](./other-page.md)
[외부 링크](https://example.com)
```

### 경고/노트

```markdown
> **Note:** 이것은 중요한 노트입니다.

> **Warning:** 주의가 필요합니다.
```

### 표

```markdown
| 헤더 1 | 헤더 2 |
|--------|--------|
| 셀 1   | 셀 2   |
```

## 배포

### GitHub에 푸시

```bash
git add docs-site/
git commit -m "docs: update documentation"
git push origin main
```

자동으로 빌드 및 배포됨 (몇 분 소요)

### 배포 상태 확인

1. GitHub Actions 탭 방문
2. 최신 "Deploy Documentation" 워크플로우 확인
3. https://sswoo.github.io/vais/ 에서 결과 확인

## 문제 해결

### 빌드 실패

```bash
# 캐시 정리 및 재빌드
mdbook clean
mdbook build
```

### 링크 깨짐

```bash
# 링크 확인
mdbook test
```

### 검색 안 됨

```bash
# 인덱스 재생성
mdbook clean
mdbook build
```

### mdBook 설치

설치 방법은 [INSTALLATION.md](INSTALLATION.md) 참고

## 유용한 명령어

```bash
# 버전 확인
mdbook --version

# 도움말
mdbook --help

# 특정 포트로 서버 시작
mdbook serve --port 8080

# 빌드만 (서버 없이)
mdbook build

# 정리
mdbook clean

# 초기화 (새 프로젝트)
mdbook init
```

## 디렉토리 구조

```
docs-site/
├── book.toml           # 설정
├── src/
│   ├── SUMMARY.md      # 목차 (여기를 수정!)
│   ├── *.md            # 페이지들
│   └── images/         # 이미지
├── book/               # 빌드 출력
├── build.sh            # 빌드 스크립트
└── serve.sh            # 개발 서버 스크립트
```

## 설정 파일 (book.toml)

```toml
[book]
title = "Vais Programming Language"
language = "ko"

[output.html.search]
enable = true           # 검색 활성화
limit-results = 30      # 검색 결과 제한

[output.html]
default-theme = "dark"  # 기본 테마
```

## Git 워크플로우

```bash
# 1. 브랜치 생성
git checkout -b docs/my-change

# 2. 문서 수정
# ... 편집 ...

# 3. 로컬 테스트
cd docs-site
./serve.sh

# 4. 커밋
git add docs-site/
git commit -m "docs: describe your change"

# 5. 푸시
git push origin docs/my-change

# 6. GitHub에서 Pull Request 생성
```

## 편집기 설정

### VSCode

추천 확장:
- Markdown All in One
- Markdown Preview Enhanced
- markdownlint

### Vim

`.vimrc`에 추가:
```vim
autocmd FileType markdown setlocal spell spelllang=ko,en
```

## 단축키 (mdbook serve)

브라우저에서:
- `s` - 검색 포커스
- `/` - 검색 포커스
- `Esc` - 검색 닫기

## 체크리스트

### 새 문서 추가 시

- [ ] 파일 생성됨
- [ ] `SUMMARY.md` 업데이트됨
- [ ] 로컬에서 테스트됨
- [ ] 링크가 작동함
- [ ] 이미지가 표시됨
- [ ] 맞춤법 확인됨

### PR 제출 전

- [ ] `mdbook build` 성공
- [ ] `mdbook test` 통과
- [ ] 모든 링크 작동
- [ ] 커밋 메시지 명확함
- [ ] 관련 이슈 링크됨

## 리소스

- [mdBook 문서](https://rust-lang.github.io/mdBook/)
- [마크다운 가이드](https://www.markdownguide.org/)
- [GitHub Pages 문서](https://docs.github.com/en/pages)

## 도움말

문제가 있나요?
- [이슈 생성](https://github.com/vaislang/vais/issues)
- [토론 포럼](https://github.com/vaislang/vais/discussions)
- README.md 참조
- CONTRIBUTING.md 참조
