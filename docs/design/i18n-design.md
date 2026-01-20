# i18n (국제화) 에러 메시지 설계

> **작성일**: 2026-01-20
> **상태**: 설계 완료
> **구현 대상**: vais-types, vais-parser, vaisc

---

## 1. 개요

### 목표
- 컴파일러 에러 메시지를 다국어로 제공
- 현재 영어(en) 기본, 한국어(ko), 일본어(ja) 지원
- 런타임 오버헤드 최소화 (컴파일 타임 메시지 로딩)

### 핵심 원칙
1. **Rust 생태계 표준 활용**: `rust-i18n` 또는 `fluent` 크레이트 사용
2. **토큰 효율성**: JSON 기반 단순 포맷 (Vais 철학과 일치)
3. **점진적 적용**: 기존 API 호환성 유지

---

## 2. 아키텍처

### 2.1 메시지 포맷 선택: JSON

**선택 이유**:
- Fluent(Mozilla)는 복잡한 문법 규칙에 강하지만 오버헤드 있음
- TOML은 Rust와 잘 맞지만 복잡한 변수 치환 어려움
- **JSON**: 단순, 파싱 빠름, Vais 철학(토큰 효율)과 일치

**파일 구조**:
```
locales/
├── en.json    # 영어 (기본/폴백)
├── ko.json    # 한국어
└── ja.json    # 일본어
```

### 2.2 메시지 키 규칙

```
{category}.{error_code}.{field}
```

예시:
```json
{
  "type.E001.title": "Type mismatch",
  "type.E001.message": "expected {expected}, found {found}",
  "type.E001.help": "try using a type cast or conversion function",

  "parse.P001.title": "Unexpected token",
  "parse.P001.message": "found {found}, expected {expected}"
}
```

### 2.3 모듈 구조

```
crates/
├── vais-i18n/              # 새 크레이트
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs          # 메인 API
│   │   ├── loader.rs       # 메시지 로더
│   │   ├── locale.rs       # 로케일 탐지
│   │   └── message.rs      # 메시지 포맷터
│   └── locales/
│       ├── en.json
│       ├── ko.json
│       └── ja.json
│
├── vais-types/
│   └── src/
│       └── types.rs        # TypeError에 i18n 적용
│
└── vais-parser/
    └── src/
        └── lib.rs          # ParseError에 i18n 적용
```

---

## 3. API 설계

### 3.1 핵심 타입

```rust
// crates/vais-i18n/src/lib.rs

/// 지원하는 로케일
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    En,  // English (기본값)
    Ko,  // 한국어
    Ja,  // 日本語
}

/// i18n 메시지 시스템
pub struct I18n {
    locale: Locale,
    messages: HashMap<String, String>,
}

impl I18n {
    /// 시스템 로케일 자동 탐지하여 생성
    pub fn new() -> Self;

    /// 특정 로케일로 생성
    pub fn with_locale(locale: Locale) -> Self;

    /// 로케일 변경
    pub fn set_locale(&mut self, locale: Locale);

    /// 메시지 가져오기 (변수 치환 포함)
    pub fn get(&self, key: &str, args: &[(&str, &str)]) -> String;

    /// 메시지 가져오기 (변수 없음)
    pub fn get_simple(&self, key: &str) -> String;
}
```

### 3.2 전역 인스턴스

```rust
use std::sync::OnceLock;

static I18N: OnceLock<I18n> = OnceLock::new();

/// 전역 i18n 인스턴스 초기화
pub fn init(locale: Option<Locale>) {
    let i18n = match locale {
        Some(l) => I18n::with_locale(l),
        None => I18n::new(),  // 시스템 로케일 자동 탐지
    };
    let _ = I18N.set(i18n);
}

/// 메시지 가져오기 매크로
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::get_simple($key)
    };
    ($key:expr, $($arg:tt)*) => {
        $crate::get($key, &[$(stringify!($arg), $arg),*])
    };
}
```

### 3.3 로케일 탐지

```rust
// crates/vais-i18n/src/locale.rs

impl Locale {
    /// 시스템 환경에서 로케일 탐지
    pub fn detect() -> Self {
        // 1. VAIS_LANG 환경변수 확인
        if let Ok(lang) = std::env::var("VAIS_LANG") {
            if let Some(locale) = Self::from_str(&lang) {
                return locale;
            }
        }

        // 2. LANG 환경변수 확인
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("ko") { return Self::Ko; }
            if lang.starts_with("ja") { return Self::Ja; }
        }

        // 3. 기본값: 영어
        Self::En
    }

    /// 문자열에서 로케일 파싱
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Self::En),
            "ko" | "korean" | "한국어" => Some(Self::Ko),
            "ja" | "japanese" | "日本語" => Some(Self::Ja),
            _ => None,
        }
    }
}
```

---

## 4. 메시지 파일 구조

### 4.1 en.json (영어 - 기본)

```json
{
  "_meta": {
    "locale": "en",
    "version": "0.0.1"
  },

  "type": {
    "E001": {
      "title": "Type mismatch",
      "message": "expected {expected}, found {found}",
      "help": "try using a type cast or conversion function"
    },
    "E002": {
      "title": "Undefined variable",
      "message": "variable '{name}' is not defined",
      "help": "variable '{name}' not found in this scope"
    },
    "E003": {
      "title": "Undefined type",
      "message": "type '{name}' is not defined"
    },
    "E004": {
      "title": "Undefined function",
      "message": "function '{name}' is not defined",
      "help": "function '{name}' not found in this scope"
    },
    "E005": {
      "title": "Not callable",
      "message": "cannot call non-function type: {type}"
    },
    "E006": {
      "title": "Wrong argument count",
      "message": "expected {expected} arguments, got {got}"
    },
    "E007": {
      "title": "Cannot infer type",
      "message": "type inference failed"
    },
    "E008": {
      "title": "Duplicate definition",
      "message": "'{name}' is already defined"
    },
    "E009": {
      "title": "Immutable assignment",
      "message": "cannot assign to immutable variable '{name}'",
      "help": "consider declaring '{name}' as mutable: '{name}: mut Type'"
    },
    "E010": {
      "title": "Non-exhaustive match",
      "message": "missing patterns: {patterns}"
    },
    "E011": {
      "title": "Unreachable pattern",
      "message": "unreachable pattern at arm {arm}"
    }
  },

  "parse": {
    "P001": {
      "title": "Unexpected token",
      "message": "found {found}, expected {expected}"
    },
    "P002": {
      "title": "Unexpected end of file",
      "message": "unexpected end of file"
    },
    "P003": {
      "title": "Invalid expression",
      "message": "invalid or malformed expression"
    }
  }
}
```

### 4.2 ko.json (한국어)

```json
{
  "_meta": {
    "locale": "ko",
    "version": "0.0.1"
  },

  "type": {
    "E001": {
      "title": "타입 불일치",
      "message": "{expected} 타입을 예상했으나, {found} 타입을 발견",
      "help": "타입 변환 함수를 사용해 보세요"
    },
    "E002": {
      "title": "정의되지 않은 변수",
      "message": "변수 '{name}'이(가) 정의되지 않았습니다",
      "help": "현재 스코프에서 '{name}' 변수를 찾을 수 없습니다"
    },
    "E003": {
      "title": "정의되지 않은 타입",
      "message": "타입 '{name}'이(가) 정의되지 않았습니다"
    },
    "E004": {
      "title": "정의되지 않은 함수",
      "message": "함수 '{name}'이(가) 정의되지 않았습니다",
      "help": "현재 스코프에서 '{name}' 함수를 찾을 수 없습니다"
    },
    "E005": {
      "title": "호출 불가",
      "message": "함수가 아닌 타입을 호출할 수 없습니다: {type}"
    },
    "E006": {
      "title": "인자 개수 불일치",
      "message": "{expected}개의 인자가 필요하지만, {got}개가 전달됨"
    },
    "E007": {
      "title": "타입 추론 불가",
      "message": "타입을 추론할 수 없습니다"
    },
    "E008": {
      "title": "중복 정의",
      "message": "'{name}'이(가) 이미 정의되어 있습니다"
    },
    "E009": {
      "title": "불변 변수 할당",
      "message": "불변 변수 '{name}'에 값을 할당할 수 없습니다",
      "help": "'{name}'을(를) 가변으로 선언하세요: '{name}: mut Type'"
    },
    "E010": {
      "title": "완전하지 않은 매치",
      "message": "누락된 패턴: {patterns}"
    },
    "E011": {
      "title": "도달 불가능한 패턴",
      "message": "{arm}번째 arm의 패턴에 도달할 수 없습니다"
    }
  },

  "parse": {
    "P001": {
      "title": "예상치 못한 토큰",
      "message": "{expected}을(를) 예상했으나, {found}을(를) 발견"
    },
    "P002": {
      "title": "예상치 못한 파일 끝",
      "message": "파일이 예상치 않게 끝났습니다"
    },
    "P003": {
      "title": "잘못된 표현식",
      "message": "올바르지 않거나 형식이 잘못된 표현식"
    }
  }
}
```

### 4.3 ja.json (일본어)

```json
{
  "_meta": {
    "locale": "ja",
    "version": "0.0.1"
  },

  "type": {
    "E001": {
      "title": "型の不一致",
      "message": "{expected}型を期待しましたが、{found}型が見つかりました",
      "help": "型変換関数を使用してください"
    },
    "E002": {
      "title": "未定義の変数",
      "message": "変数'{name}'は定義されていません",
      "help": "このスコープで'{name}'変数が見つかりません"
    },
    "E003": {
      "title": "未定義の型",
      "message": "型'{name}'は定義されていません"
    },
    "E004": {
      "title": "未定義の関数",
      "message": "関数'{name}'は定義されていません",
      "help": "このスコープで'{name}'関数が見つかりません"
    },
    "E005": {
      "title": "呼び出し不可",
      "message": "関数ではない型を呼び出すことはできません: {type}"
    },
    "E006": {
      "title": "引数の数が不一致",
      "message": "{expected}個の引数が必要ですが、{got}個が渡されました"
    },
    "E007": {
      "title": "型推論不可",
      "message": "型を推論できません"
    },
    "E008": {
      "title": "重複定義",
      "message": "'{name}'はすでに定義されています"
    },
    "E009": {
      "title": "不変変数への代入",
      "message": "不変変数'{name}'に値を代入できません",
      "help": "'{name}'をmutableとして宣言してください: '{name}: mut Type'"
    },
    "E010": {
      "title": "網羅性のないmatch",
      "message": "パターンが不足しています: {patterns}"
    },
    "E011": {
      "title": "到達不能パターン",
      "message": "arm {arm}のパターンには到達できません"
    }
  },

  "parse": {
    "P001": {
      "title": "予期しないトークン",
      "message": "{expected}を期待しましたが、{found}が見つかりました"
    },
    "P002": {
      "title": "予期しないファイル終端",
      "message": "ファイルが予期せず終了しました"
    },
    "P003": {
      "title": "無効な式",
      "message": "無効または不正な形式の式です"
    }
  }
}
```

---

## 5. 통합 방법

### 5.1 TypeError 수정

```rust
// crates/vais-types/src/types.rs

impl TypeError {
    /// i18n 적용된 에러 메시지 반환
    pub fn localized_title(&self) -> String {
        use vais_i18n::t;

        match self {
            TypeError::Mismatch { .. } => t!("type.E001.title"),
            TypeError::UndefinedVar(..) => t!("type.E002.title"),
            // ... 나머지 variant
        }
    }

    /// i18n 적용된 상세 메시지 반환
    pub fn localized_message(&self) -> String {
        use vais_i18n::get;

        match self {
            TypeError::Mismatch { expected, found, .. } => {
                get("type.E001.message", &[
                    ("expected", expected),
                    ("found", found),
                ])
            },
            TypeError::UndefinedVar(name, _) => {
                get("type.E002.message", &[("name", name)])
            },
            // ... 나머지 variant
        }
    }

    /// i18n 적용된 help 메시지 반환
    pub fn localized_help(&self) -> Option<String> {
        use vais_i18n::{get, has_key};

        let key = format!("type.{}.help", self.error_code());
        if has_key(&key) {
            Some(match self {
                TypeError::UndefinedVar(name, _) => {
                    get(&key, &[("name", name)])
                },
                TypeError::ImmutableAssign(name, _) => {
                    get(&key, &[("name", name)])
                },
                _ => get(&key, &[]),
            })
        } else {
            None
        }
    }
}
```

### 5.2 ErrorReporter 수정

```rust
// crates/vais-types/src/error_report.rs

impl<'a> ErrorReporter<'a> {
    /// i18n 지원 에러 포맷팅
    pub fn format_localized<E: DiagnosticError>(&self, error: &E) -> String {
        self.format_error(
            error.error_code(),
            &error.localized_title(),
            error.span(),
            &error.localized_message(),
            error.localized_help().as_deref(),
        )
    }
}
```

### 5.3 CLI 옵션 추가

```rust
// crates/vaisc/src/main.rs

#[derive(Parser)]
struct Args {
    /// Set the locale for error messages (en, ko, ja)
    #[arg(long, value_name = "LOCALE")]
    locale: Option<String>,

    // ... 기존 옵션
}

fn main() {
    let args = Args::parse();

    // i18n 초기화
    let locale = args.locale
        .as_ref()
        .and_then(|s| vais_i18n::Locale::from_str(s));
    vais_i18n::init(locale);

    // ... 기존 로직
}
```

---

## 6. 구현 계획

### 6.1 단계별 작업

| 단계 | 작업 | 추천 모델 | 예상 라인 |
|------|------|----------|----------|
| 1 | vais-i18n 크레이트 생성 | Sonnet | ~300 |
| 2 | 번역 파일 생성 (en, ko, ja) | Haiku | ~150 |
| 3 | TypeError/ParseError 통합 | Sonnet | ~200 |
| 4 | CLI --locale 옵션 | Haiku | ~30 |
| 5 | 테스트 및 검증 | Opus | ~100 |

### 6.2 의존성

```toml
# crates/vais-i18n/Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
once_cell = "1.18"  # 전역 인스턴스용
```

---

## 7. 테스트 전략

### 7.1 단위 테스트

```rust
#[test]
fn test_locale_detection() {
    std::env::set_var("VAIS_LANG", "ko");
    assert_eq!(Locale::detect(), Locale::Ko);
}

#[test]
fn test_message_substitution() {
    let i18n = I18n::with_locale(Locale::En);
    let msg = i18n.get("type.E001.message", &[
        ("expected", "i64"),
        ("found", "Str"),
    ]);
    assert_eq!(msg, "expected i64, found Str");
}

#[test]
fn test_fallback_to_english() {
    let i18n = I18n::with_locale(Locale::Ko);
    // 한국어에 없는 키는 영어로 폴백
    let msg = i18n.get_simple("unknown.key");
    // 영어 기본값 또는 키 자체 반환
}
```

### 7.2 통합 테스트

```rust
#[test]
fn test_localized_type_error() {
    vais_i18n::init(Some(Locale::Ko));

    let error = TypeError::Mismatch {
        expected: "i64".to_string(),
        found: "Str".to_string(),
        span: None,
    };

    assert_eq!(error.localized_title(), "타입 불일치");
    assert!(error.localized_message().contains("i64"));
}
```

---

## 8. 확장성

### 8.1 새 언어 추가

1. `locales/` 디렉토리에 `{locale}.json` 추가
2. `Locale` enum에 새 variant 추가
3. `Locale::from_str()`에 매핑 추가

### 8.2 새 에러 추가

1. 각 `.json` 파일에 에러 코드 추가
2. 해당 Error 타입의 `localized_*` 메서드 업데이트

---

## 9. 결론

이 설계는 Vais의 토큰 효율성 철학과 일치하면서도 확장 가능한 i18n 시스템을 제공합니다. JSON 기반의 단순한 구조로 유지보수가 쉽고, 점진적 적용이 가능합니다.
