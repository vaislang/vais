# Vais 문서 사이트

이 디렉토리에는 [mdBook](https://rust-lang.github.io/mdBook/)을 사용하여 생성되는 Vais 프로그래밍 언어의 공식 문서 사이트가 포함되어 있습니다.

## 빠른 시작

### 필요 사항

mdBook 설치 방법은 [INSTALLATION.md](INSTALLATION.md) 참고

### 로컬에서 문서 빌드

```bash
# 빌드 스크립트 실행 (자동으로 mdBook 설치)
./build.sh

# 또는 수동으로 빌드
mdbook build
```

### 로컬에서 문서 미리보기

```bash
# 개발 서버 시작 (자동 새로고침 포함)
mdbook serve

# 브라우저에서 http://localhost:3000 열기
```

### 문서 테스트

```bash
# 링크 확인 및 코드 예제 테스트
mdbook test
```

## 구조

```
docs-site/
├── book.toml           # mdBook 설정
├── src/                # 문서 소스 파일
│   ├── SUMMARY.md      # 목차
│   ├── introduction.md # 소개 페이지
│   ├── getting-started/
│   ├── language/
│   ├── stdlib/
│   ├── compiler/
│   ├── tools/
│   ├── advanced/
│   ├── security/
│   ├── contributing/
│   └── appendix.md
├── book/               # 빌드된 HTML 출력 (git에서 무시됨)
└── build.sh            # 빌드 스크립트
```

## 문서 작성

### 새 페이지 추가

1. `src/` 디렉토리에 새 마크다운 파일 생성
2. `src/SUMMARY.md`에 페이지 링크 추가
3. 기존 문서를 참조하려면 `{{#include}}` 구문 사용:

```markdown
{{#include ../../path/to/existing/doc.md}}
```

### 스타일 가이드

- 헤딩은 `#`으로 시작 (h1은 페이지 제목용)
- 코드 블록에는 언어 태그 사용: \`\`\`vais, \`\`\`rust, \`\`\`bash
- 내부 링크는 상대 경로 사용: `[링크 텍스트](./other-page.md)`
- 이미지는 `src/` 디렉토리에 저장

### 검색 기능

mdBook의 내장 검색 기능이 자동으로 활성화됩니다. 추가 설정은 `book.toml`의 `[output.html.search]` 섹션을 참조하세요.

## 배포

문서는 GitHub Actions를 통해 자동으로 배포됩니다:

1. `main` 브랜치에 푸시하면 자동으로 빌드 및 배포
2. GitHub Pages에 배포됨
3. URL: `https://sswoo.github.io/vais/`

수동 배포:

```bash
# 문서 빌드
./build.sh

# book/ 디렉토리의 내용을 웹 서버에 배포
```

## 설정

`book.toml` 파일에서 다음을 설정할 수 있습니다:

- 책 제목 및 설명
- 테마 설정
- 검색 옵션
- GitHub 저장소 링크
- 편집 URL 템플릿

자세한 내용은 [mdBook 문서](https://rust-lang.github.io/mdBook/)를 참조하세요.

## 문제 해결

### mdBook이 설치되지 않음

자세한 설치 방법은 [INSTALLATION.md](INSTALLATION.md) 참고

### 빌드 오류

1. 모든 참조된 파일이 존재하는지 확인
2. `SUMMARY.md`의 링크가 올바른지 확인
3. 마크다운 구문이 유효한지 확인

### 검색이 작동하지 않음

검색 인덱스는 빌드 시 자동으로 생성됩니다. `mdbook clean && mdbook build`를 실행하여 재빌드하세요.

## 기여

문서 개선을 환영합니다! 다음을 참조하세요:

1. [기여 가이드](../CONTRIBUTING.md)
2. 이슈 생성: [GitHub Issues](https://github.com/vaislang/vais/issues)
3. Pull Request 제출

## 라이센스

이 문서는 Vais 프로젝트와 동일한 MIT 라이센스를 따릅니다.
