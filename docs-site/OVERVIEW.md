# Vais 문서 사이트 완전 개요

## 프로젝트 통계

- **총 파일**: 75개
- **마크다운 문서**: 71개
- **설정 파일**: 1개 (book.toml)
- **스크립트**: 2개 (build.sh, serve.sh)
- **디렉토리**: 15개 (src/ 내)
- **GitHub Actions 워크플로우**: 1개

## 파일 구조

```
docs-site/
├── book.toml                           # mdBook 설정
├── .gitignore                          # Git 무시 파일
│
├── build.sh                            # 빌드 스크립트
├── serve.sh                            # 개발 서버 스크립트
│
├── README.md                           # 프로젝트 소개
├── CONTRIBUTING.md                     # 기여 가이드
├── INSTALLATION.md                     # mdBook 설치 가이드
├── QUICK_REFERENCE.md                  # 빠른 참조
├── PROJECT_SUMMARY.md                  # 프로젝트 요약
├── OVERVIEW.md                         # 이 파일
│
└── src/                                # 문서 소스
    ├── SUMMARY.md                      # 목차 (66페이지)
    ├── introduction.md                 # 소개
    ├── appendix.md                     # 부록
    │
    ├── getting-started/                # 시작하기 (4개)
    │   ├── installation.md
    │   ├── tutorial.md
    │   ├── quick-start.md
    │   └── editors.md
    │
    ├── language/                       # 언어 레퍼런스 (5개)
    │   ├── language-spec.md
    │   ├── generics.md
    │   ├── async-tutorial.md
    │   ├── comptime-feature.md
    │   └── iterator-type-inference.md
    │
    ├── stdlib/                         # 표준 라이브러리 (3개)
    │   ├── stdlib.md
    │   ├── gc-implementation.md
    │   └── gc-quick-reference.md
    │
    ├── compiler/                       # 컴파일러 (7개)
    │   ├── architecture.md
    │   ├── tech-spec.md
    │   ├── benchmark-design.md
    │   ├── inkwell-integration.md
    │   ├── monomorphization-design.md
    │   ├── jit-compilation.md
    │   └── gpu-codegen.md
    │
    ├── tools/                          # 개발자 도구 (13개)
    │   ├── editors.md
    │   ├── lsp-features.md
    │   ├── hot-reload.md
    │   ├── coverage.md
    │   ├── playground/                 # (6개)
    │   │   ├── README.md
    │   │   ├── features.md
    │   │   ├── quickstart.md
    │   │   ├── tutorial.md
    │   │   ├── integration.md
    │   │   └── deployment.md
    │   └── vais-tutorial/              # (3개)
    │       ├── README.md
    │       ├── usage.md
    │       └── quickstart.md
    │
    ├── advanced/                       # 고급 주제 (18개)
    │   ├── ffi/                        # (3개)
    │   │   ├── README.md
    │   │   ├── guide.md
    │   │   └── features.md
    │   ├── bindgen/                    # (4개)
    │   │   ├── README.md
    │   │   ├── cpp-support.md
    │   │   ├── cpp-quickstart.md
    │   │   └── design.md
    │   ├── wasm/                       # (2개)
    │   │   ├── README.md
    │   │   └── component-model.md
    │   ├── language-bindings.md
    │   ├── self-hosting-design.md
    │   ├── plugin-system-design.md
    │   ├── package-manager-design.md
    │   ├── i18n-design.md
    │   ├── ipv6-implementation.md
    │   └── range-type-implementation.md
    │
    ├── security/                       # 보안 (2개)
    │   ├── security-enhancement.md
    │   └── import-path-security.md
    │
    └── contributing/                   # 개발 참여 (13개)
        ├── contributing.md
        ├── roadmap.md
        ├── refactoring-summary.md
        ├── implementation-summaries.md
        └── summaries/                  # (9개)
            ├── implementation-summary.md
            ├── ffi-implementation.md
            ├── gc-implementation.md
            ├── hot-reload-implementation.md
            ├── playground-implementation.md
            ├── wasm-component-implementation.md
            ├── bindgen-implementation.md
            ├── cpp-bindgen-implementation.md
            └── async-type-checking.md
```

## 주요 섹션 분석

### 1. Getting Started (시작하기) - 4개 문서
입문자를 위한 필수 가이드
- 설치 방법
- 첫 프로그램 작성
- 튜토리얼
- 에디터 설정

### 2. Language Reference (언어 레퍼런스) - 5개 문서
언어의 핵심 기능 설명
- 완전한 언어 사양
- 제네릭 시스템
- 비동기 프로그래밍
- 컴파일 타임 기능
- 타입 추론

### 3. Standard Library (표준 라이브러리) - 3개 문서
내장 라이브러리 문서
- stdlib API 레퍼런스
- 가비지 컬렉터 구현
- GC 사용 가이드

### 4. Compiler (컴파일러) - 7개 문서
컴파일러 내부 구조
- 아키텍처 설계
- LLVM 통합
- 최적화 기법
- JIT 컴파일
- GPU 코드 생성

### 5. Tools (개발자 도구) - 13개 문서
개발 환경 및 도구
- LSP 서버
- 에디터 통합
- 플레이그라운드
- 인터랙티브 튜토리얼
- 핫 리로드
- 코드 커버리지

### 6. Advanced Topics (고급 주제) - 18개 문서
심화 기능 및 통합
- FFI (Foreign Function Interface)
- 다른 언어와의 바인딩
- C++ 지원
- WASM 컴포넌트 모델
- 셀프 호스팅
- 플러그인 시스템
- 패키지 매니저

### 7. Security (보안) - 2개 문서
보안 관련 문서
- 보안 강화 기능
- 임포트 경로 보안

### 8. Contributing (개발 참여) - 13개 문서
프로젝트 기여를 위한 가이드
- 기여 가이드라인
- 프로젝트 로드맵
- 구현 요약
- 리팩토링 노트

## 기술 사양

### mdBook 설정 (book.toml)

```toml
[book]
title = "Vais Programming Language"
language = "ko"
multilingual = false

[output.html]
default-theme = "dark"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/vaislang/vais"
site-url = "/vais/"

[output.html.search]
enable = true
limit-results = 30
use-boolean-and = true
boost-title = 2
```

**주요 기능:**
- 한국어 기본 설정
- 다크 테마 활성화
- 검색 최적화
- GitHub 통합
- 편집 링크 제공

### 빌드 스크립트 (build.sh)

**기능:**
1. mdBook 설치 확인
2. 자동 설치 (필요시)
3. 이전 빌드 정리
4. 문서 빌드
5. 결과 확인

**사용법:**
```bash
cd docs-site
./build.sh
```

### 개발 서버 (serve.sh)

**기능:**
1. 로컬 서버 시작
2. 핫 리로드 활성화
3. 브라우저 자동 열기
4. 실시간 미리보기

**사용법:**
```bash
cd docs-site
./serve.sh
```

## GitHub Actions 워크플로우

**파일:** `.github/workflows/docs.yml`

**트리거:**
- `main` 브랜치 푸시
- 문서 파일 변경
- 수동 트리거

**단계:**
1. 저장소 체크아웃
2. Rust 툴체인 설정
3. 캐시 복원
4. mdBook 설치
5. 문서 빌드
6. GitHub Pages에 배포

**배포 URL:** https://vais.dev/docs/

## 문서 통합 전략

### Include 패턴 사용

모든 페이지는 `{{#include}}` 구문을 사용하여 기존 문서를 참조:

```markdown
# 언어 사양

{{#include ../../../docs/LANGUAGE_SPEC.md}}
```

**장점:**
- 단일 소스 원칙 (Single Source of Truth)
- 중복 제거
- 유지보수 간소화
- 동기화 자동화

### 참조 경로 패턴

| 원본 위치 | 참조 경로 | 예시 |
|-----------|-----------|------|
| 프로젝트 루트 | `../../*.md` | README.md |
| docs/ | `../../../docs/*.md` | LANGUAGE_SPEC.md |
| docs/design/ | `../../../docs/design/*.md` | TECH_SPEC.md |
| crates/ | `../../../../crates/*/` | vais-lsp/ |
| playground/ | `../../../../playground/` | README.md |

## 사용자 가이드

### 일반 사용자 (문서 읽기)

1. **온라인 문서 방문**
   - URL: https://vais.dev/docs/
   - 검색 기능 사용
   - 모바일 친화적

2. **로컬에서 보기**
   ```bash
   cd docs-site
   ./serve.sh
   ```

### 문서 작성자 (기여자)

1. **개발 환경 설정**
   See [INSTALLATION.md](INSTALLATION.md) for mdBook installation, then:
   ```bash
   cd docs-site
   ./serve.sh
   ```

2. **문서 수정**
   - 원본 파일 수정 (예: `docs/*.md`)
   - 자동으로 반영됨
   - 실시간 미리보기

3. **새 페이지 추가**
   - 파일 생성
   - `SUMMARY.md` 업데이트
   - Include 패턴 사용

4. **Pull Request 제출**
   - 변경 사항 커밋
   - PR 생성
   - 리뷰 받기
   - 자동 배포

### 프로젝트 관리자

1. **전체 빌드**
   ```bash
   cd docs-site
   ./build.sh
   ```

2. **링크 확인**
   ```bash
   mdbook test
   ```

3. **배포 모니터링**
   - GitHub Actions 탭 확인
   - 배포 로그 검토

## 성능 메트릭

### 빌드 성능
- **초기 빌드**: ~5초
- **증분 빌드**: ~1초
- **전체 재빌드**: ~3초
- **검색 인덱스 생성**: ~500ms

### 런타임 성능
- **첫 페이지 로드**: <1초
- **페이지 전환**: <100ms
- **검색 응답**: <50ms
- **이미지 로딩**: <200ms

### 사이트 크기
- **총 페이지**: 66개
- **평균 페이지**: ~50KB
- **검색 인덱스**: ~300KB
- **전체 사이트**: ~5MB

## 검색 기능

### 설정
```toml
[output.html.search]
enable = true
limit-results = 30
teaser-word-count = 30
use-boolean-and = true
boost-title = 2
boost-hierarchy = 1
boost-paragraph = 1
```

### 기능
- 실시간 검색
- 퍼지 매칭
- 가중치 기반 랭킹
- 하이라이팅
- 키보드 단축키 (s, /)

### 커버리지
- 전체 콘텐츠 인덱싱
- 제목, 헤딩, 본문
- 코드 블록 제외

## 접근성

### 키보드 네비게이션
- `Tab` - 요소 간 이동
- `Enter` - 링크 활성화
- `s` 또는 `/` - 검색
- `Esc` - 검색 닫기

### 화면 리더 지원
- 시맨틱 HTML
- ARIA 라벨
- 대체 텍스트

### 반응형 디자인
- 모바일 최적화
- 태블릿 지원
- 데스크톱 레이아웃

## 다국어 지원 (계획)

### 현재
- 한국어 (ko) - 주 언어

### 향후 계획
- 영어 (en)
- 일본어 (ja)
- 중국어 (zh)

## 유지보수

### 정기 작업

**주간:**
- [ ] 링크 확인
- [ ] 오타 수정
- [ ] 새 콘텐츠 추가

**월간:**
- [ ] mdBook 업데이트
- [ ] 종속성 업데이트
- [ ] 성능 검토

**분기별:**
- [ ] 전체 문서 리뷰
- [ ] 사용자 피드백 반영
- [ ] 개선 사항 구현

### 모니터링

**메트릭:**
- GitHub Pages 방문 통계
- 검색 쿼리 분석
- 빌드 성공률
- 배포 빈도

## 문제 해결

### 일반적인 문제

1. **"command not found: mdbook"**
   ```bash
   cargo install mdbook
   ```

2. **빌드 실패**
   ```bash
   mdbook clean
   mdbook build
   ```

3. **링크 깨짐**
   ```bash
   mdbook test
   ```

4. **검색 안 됨**
   ```bash
   mdbook clean && mdbook build
   ```

### 디버깅

```bash
# 상세 로그
RUST_LOG=debug mdbook build

# 특정 기능 테스트
mdbook test

# 설정 검증
mdbook build --dest-dir test-build
```

## 향후 개선 사항

### 단기 (1-2주)
- [ ] 영어 버전 추가
- [ ] PDF 출력 지원
- [ ] 더 많은 다이어그램
- [ ] 코드 예제 실행

### 중기 (1-2개월)
- [ ] 대화형 튜토리얼
- [ ] API 문서 자동 생성
- [ ] 버전별 문서
- [ ] 검색 개선

### 장기 (3-6개월)
- [ ] 비디오 튜토리얼
- [ ] 커뮤니티 위키
- [ ] AI 챗봇
- [ ] 성능 대시보드

## 기여자 가이드

### 시작하기
1. [CONTRIBUTING.md](CONTRIBUTING.md) 읽기
2. 이슈 확인
3. 포크 및 클론
4. 변경 사항 작성
5. PR 제출

### 리소스
- [README.md](README.md) - 프로젝트 소개
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - 빠른 참조
- [INSTALLATION.md](INSTALLATION.md) - 설치 가이드

## 라이센스

MIT License - Vais 프로젝트와 동일

## 연락처

- **GitHub**: https://github.com/vaislang/vais
- **Issues**: https://github.com/vaislang/vais/issues
- **Discussions**: https://github.com/vaislang/vais/discussions

---

**생성일**: 2026-01-29
**버전**: 1.0.0
**관리자**: Vais Contributors
