# 문서 기여 가이드

Vais 문서 개선에 관심을 가져주셔서 감사합니다! 이 가이드는 문서에 기여하는 방법을 설명합니다.

## 시작하기

### 개발 환경 설정

1. 저장소 클론:
```bash
git clone https://github.com/vaislang/vais.git
cd vais/docs-site
```

2. mdBook 설치:
```bash
cargo install mdbook
```

3. 개발 서버 시작:
```bash
./serve.sh
```

## 문서 구조

```
docs-site/
├── book.toml           # mdBook 설정
├── src/                # 문서 소스
│   ├── SUMMARY.md      # 목차 (중요!)
│   ├── getting-started/
│   ├── language/
│   ├── stdlib/
│   ├── compiler/
│   ├── tools/
│   ├── advanced/
│   ├── security/
│   └── contributing/
└── book/               # 빌드 출력 (git에서 무시됨)
```

## 기여 유형

### 1. 오타 수정

작은 오타나 문법 오류는 직접 수정하여 Pull Request를 제출하세요.

### 2. 기존 문서 개선

- 불명확한 설명을 개선
- 예제 코드 추가
- 다이어그램이나 이미지 추가
- 링크 업데이트

### 3. 새 문서 추가

새 페이지를 추가하는 경우:

1. `src/` 디렉토리의 적절한 위치에 `.md` 파일 생성
2. `src/SUMMARY.md`에 페이지 링크 추가
3. 기존 문서 스타일을 따름

## 작성 가이드라인

### 마크다운 스타일

#### 헤딩

```markdown
# H1 - 페이지 제목 (페이지당 하나만)
## H2 - 주요 섹션
### H3 - 하위 섹션
#### H4 - 세부 항목
```

#### 코드 블록

언어를 명시하세요:

```markdown
\`\`\`vais
F main() {
    print("Hello, Vais!")
}
\`\`\`

\`\`\`bash
cargo build --release
\`\`\`

\`\`\`rust
fn main() {
    println!("Hello, Rust!");
}
\`\`\`
```

#### 링크

내부 링크는 상대 경로 사용:

```markdown
[튜토리얼](./getting-started/tutorial.md)
[언어 사양](../language/language-spec.md)
```

외부 링크:

```markdown
[Rust 공식 사이트](https://www.rust-lang.org/)
```

#### 리스트

```markdown
- 순서 없는 리스트
- 항목 2
  - 중첩된 항목

1. 순서 있는 리스트
2. 항목 2
   1. 중첩된 항목
```

#### 표

```markdown
| 헤더 1 | 헤더 2 | 헤더 3 |
|--------|--------|--------|
| 셀 1   | 셀 2   | 셀 3   |
| 셀 4   | 셀 5   | 셀 6   |
```

#### 강조

```markdown
**굵게**
*기울임*
`코드`
```

### 콘텐츠 가이드라인

#### 명확하고 간결하게

- 짧고 명확한 문장 사용
- 전문 용어는 처음 사용 시 설명
- 복잡한 개념은 예제로 설명

#### 예제 코드

- 실행 가능한 완전한 예제 제공
- 주석으로 중요한 부분 설명
- 일반적인 사용 사례 포함

#### 일관성

- 기존 문서의 톤과 스타일 유지
- 용어를 일관되게 사용
- 형식을 통일

### 기존 문서 참조

mdBook의 `{{#include}}` 구문을 사용하여 기존 문서를 참조할 수 있습니다:

```markdown
# 언어 사양

{{#include ../../../docs/LANGUAGE_SPEC.md}}
```

이렇게 하면 기존 문서를 복제하지 않고 재사용할 수 있습니다.

## Pull Request 프로세스

### 1. 브랜치 생성

```bash
git checkout -b docs/my-improvement
```

### 2. 변경 사항 작성

- 문서를 수정하고 로컬에서 테스트
- `mdbook serve`로 변경 사항 확인

### 3. 커밋

```bash
git add docs-site/src/...
git commit -m "docs: improve tutorial section"
```

커밋 메시지 형식:
- `docs: ` 접두사 사용
- 명확하고 설명적인 메시지

### 4. 푸시 및 PR 생성

```bash
git push origin docs/my-improvement
```

GitHub에서 Pull Request를 생성하고:
- 변경 사항을 명확히 설명
- 관련 이슈가 있다면 링크

### 5. 리뷰 대응

- 리뷰어의 피드백에 응답
- 필요한 수정 사항 반영
- CI 확인이 통과하는지 확인

## 자동 배포

`main` 브랜치에 병합되면:
1. GitHub Actions가 자동으로 문서 빌드
2. GitHub Pages에 배포
3. 몇 분 내에 https://sswoo.github.io/vais/ 에서 확인 가능

## 로컬 테스트

### 빌드 테스트

```bash
./build.sh
```

### 링크 확인

```bash
mdbook test
```

### 전체 정리 후 재빌드

```bash
mdbook clean
mdbook build
```

## 문제 보고

문서에서 문제를 발견했지만 직접 수정할 수 없는 경우:

1. [GitHub Issues](https://github.com/vaislang/vais/issues)에 이슈 생성
2. `documentation` 라벨 추가
3. 문제를 명확히 설명:
   - 어떤 페이지인지
   - 무엇이 잘못되었는지
   - 어떻게 개선할 수 있는지

## 도움 받기

질문이 있으신가요?

- [GitHub Discussions](https://github.com/vaislang/vais/discussions)
- [Discord 채널](#) (준비 중)
- 이슈에 질문 태그로 문의

## 스타일 체크리스트

PR을 제출하기 전에 확인하세요:

- [ ] 마크다운이 올바르게 렌더링됨
- [ ] 모든 링크가 작동함
- [ ] 코드 예제가 실행 가능함
- [ ] 맞춤법과 문법이 올바름
- [ ] 기존 문서 스타일을 따름
- [ ] `SUMMARY.md`가 업데이트됨 (새 페이지인 경우)
- [ ] 이미지가 적절한 크기임
- [ ] 커밋 메시지가 명확함

## 감사의 말

문서 기여자 목록은 [Contributors](https://github.com/vaislang/vais/graphs/contributors)에서 확인할 수 있습니다.

모든 기여에 감사드립니다! 🙏
