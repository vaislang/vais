# VAIS Language Support for Helix Editor

이 디렉토리에는 Helix 에디터에서 VAIS 언어를 지원하기 위한 설정 파일들이 포함되어 있습니다.

## 파일 구조

```
editors/helix/
├── languages.toml           # Helix 언어 설정 파일
├── queries/
│   └── vais/
│       └── highlights.scm   # Tree-sitter 문법 강조 쿼리
└── README.md                # 이 파일
```

## 설치 방법

### 1. 설정 파일 복사

Helix의 설정 디렉토리에 파일들을 복사합니다.

**macOS/Linux:**
```bash
# languages.toml을 Helix 설정 디렉토리에 복사
cp editors/helix/languages.toml ~/.config/helix/languages.toml

# 쿼리 파일들을 복사
mkdir -p ~/.config/helix/queries/vais
cp editors/helix/queries/vais/highlights.scm ~/.config/helix/queries/vais/
```

**Windows:**
```bash
# AppData 경로에 복사
cp editors/helix/languages.toml %APPDATA%\helix\languages.toml
mkdir %APPDATA%\helix\queries\vais
cp editors/helix/queries/vais/highlights.scm %APPDATA%\helix\queries\vais\
```

### 2. LSP 서버 설정

VAIS LSP 서버가 설치되어 있어야 합니다.

```bash
# VAIS 프로젝트에서 LSP 서버 빌드
cd /path/to/vais
cargo build --release -p vais-lsp

# 바이너리를 PATH에 추가하거나 절대 경로로 지정
# languages.toml에서 command 설정 확인
```

### 3. Helix에서 확인

Helix를 열고 `.vais` 파일을 편집하면 자동으로 문법 강조와 LSP 기능이 활성화됩니다.

## 주요 기능

### 문법 강조

다음 요소들에 대한 문법 강조가 지원됩니다:

- **키워드**: F, S, E, I, L, M, W, T, X, V, C, R, B, N, A
- **타입**: 내장 타입 (i32, i64, u32, u64, f32, f64, bool, string, void, u8, i8)
- **연산자**: 산술, 비교, 논리, 비트 연산자
- **자재귀 호출 연산자**: @
- **주석**: # (한줄 주석)
- **식별자**: 함수, 변수, 타입 이름
- **문자열과 숫자**: 문자열 리터럴과 숫자 상수

### LSP 기능

다음 기능들이 LSP 서버를 통해 제공됩니다 (설정된 경우):

- 자동 완성
- 정의로 이동 (Goto Definition)
- 참조 찾기 (Find References)
- 타입 정보 표시 (Hover)
- 진단 (문법/타입 오류)
- 문서 포맷팅 (가능한 경우)

## 설정 커스터마이징

### languages.toml 수정

`languages.toml` 파일에서 다음을 커스터마이징할 수 있습니다:

```toml
[[language]]
name = "vais"
scope = "source.vais"
file-extensions = ["vais"]
comment-tokens = "#"
indent = { tab-width = 4, unit = "    " }

[language.lsp]
command = "vais-lsp"  # LSP 서버 바이너리 경로 또는 이름
args = []             # LSP 서버 명령줄 인수

roots = ["Cargo.toml", "vais.toml"]  # 프로젝트 루트 파일들
```

### LSP 서버 절대 경로 설정

LSP 서버가 PATH에 없는 경우 절대 경로로 지정할 수 있습니다:

```toml
[language.lsp]
command = "/path/to/vais-lsp"
```

## 트러블슈팅

### LSP 서버가 시작되지 않음

1. `vais-lsp` 바이너리가 PATH에 있는지 확인
2. `languages.toml`에서 command 경로가 올바른지 확인
3. 다음 명령으로 LSP 서버 직접 실행 테스트:
   ```bash
   which vais-lsp
   # 또는
   /path/to/vais-lsp --version
   ```

### 문법 강조가 작동하지 않음

1. Tree-sitter 문법이 설치되어 있는지 확인:
   ```bash
   hx --version
   # Helix의 Tree-sitter 지원 확인
   ```

2. `highlights.scm` 파일이 올바른 위치에 있는지 확인:
   ```bash
   ls ~/.config/helix/queries/vais/highlights.scm
   ```

3. Helix를 재시작하고 파일을 다시 열어보기

## VAIS 언어 특징

### 단일 문자 키워드

| 키워드 | 의미 | 확장형 |
|--------|------|--------|
| F | 함수 | fn |
| S | 구조체 | struct |
| E | Enum | enum |
| I | If | if |
| L | Loop | loop |
| M | Match | match |
| W | While | while |
| T | Trait | trait |
| X | Impl | impl |
| V | Let | let |
| C | Const | const |
| R | Return | return |
| B | Break | break |
| N | Continue | continue |
| A | Async | async |

### 특수 연산자

- **@**: 자재귀(Self-recursion) 호출 연산자

### 주석

- **#**: 한줄 주석

## 추가 리소스

- [VAIS 언어 레포지토리](https://github.com/vais-lang/vais)
- [Helix 에디터 문서](https://docs.helix-editor.com/)
- [Tree-sitter 쿼리 문서](https://tree-sitter.github.io/tree-sitter/syntax-highlighting)

## 기여하기

VAIS Helix 통합에 개선사항을 제안하거나 버그를 보고하려면:

1. 문제를 [VAIS 레포지토리](https://github.com/vais-lang/vais/issues)에 보고
2. Pull Request로 개선사항 제출

## 라이선스

이 설정 파일들은 VAIS 언어와 동일한 라이선스를 따릅니다.
