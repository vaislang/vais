# mdBook 설치 및 사용

## mdBook이란?

mdBook은 Rust로 작성된 정적 사이트 생성기로, 마크다운 파일로부터 아름다운 문서 사이트를 생성합니다. Rust 공식 문서(The Rust Book)와 같은 많은 프로젝트에서 사용됩니다.

## 설치

### Option 1: build.sh 스크립트 사용 (권장)

`build.sh` 스크립트가 자동으로 mdBook을 설치합니다:

```bash
cd docs-site
./build.sh
```

### Option 2: 수동 설치

#### Cargo를 통한 설치 (모든 플랫폼)

```bash
cargo install mdbook
```

#### 바이너리 다운로드

[GitHub Releases](https://github.com/rust-lang/mdBook/releases)에서 사전 컴파일된 바이너리를 다운로드할 수 있습니다.

#### 패키지 매니저를 통한 설치

**macOS (Homebrew)**
```bash
brew install mdbook
```

**Linux (Arch)**
```bash
pacman -S mdbook
```

## 사용법

### 문서 빌드

```bash
cd docs-site
mdbook build
```

빌드된 HTML은 `book/` 디렉토리에 생성됩니다.

### 개발 서버 시작

```bash
cd docs-site
mdbook serve
```

또는 편의 스크립트 사용:

```bash
cd docs-site
./serve.sh
```

브라우저가 자동으로 열리고 `http://localhost:3000`에서 문서를 볼 수 있습니다. 파일을 수정하면 자동으로 새로고침됩니다.

### 문서 정리

```bash
cd docs-site
mdbook clean
```

## 고급 옵션

### 특정 포트에서 서버 시작

```bash
mdbook serve --port 8080
```

### 자동으로 브라우저 열기

```bash
mdbook serve --open
```

### 링크 체크

mdbook-linkcheck 플러그인 설치:

```bash
cargo install mdbook-linkcheck
```

`book.toml`에 추가:

```toml
[preprocessor.linkcheck]
```

### 검색 기능

검색은 기본적으로 활성화되어 있습니다. `book.toml`에서 설정을 조정할 수 있습니다:

```toml
[output.html.search]
enable = true
limit-results = 30
```

## 문제 해결

### "command not found: mdbook"

PATH에 `~/.cargo/bin`이 포함되어 있는지 확인하세요:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 빌드 오류

1. 모든 참조된 파일이 존재하는지 확인
2. `SUMMARY.md`의 모든 링크가 유효한지 확인
3. 캐시를 지우고 재빌드: `mdbook clean && mdbook build`

### 포트가 이미 사용 중

다른 포트를 사용하세요:

```bash
mdbook serve --port 3001
```

## 추가 리소스

- [mdBook 공식 가이드](https://rust-lang.github.io/mdBook/)
- [mdBook GitHub 저장소](https://github.com/rust-lang/mdBook)
- [mdBook 플러그인 목록](https://github.com/rust-lang/mdBook/wiki/Third-party-plugins)
