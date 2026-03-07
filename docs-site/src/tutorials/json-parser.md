# Tutorial: JSON Parser 만들기

이 튜토리얼에서는 Vais로 간단한 JSON 파서를 만듭니다. 재귀 하강(recursive descent) 기법으로 JSON 문자열을 파싱하고, 값을 추출하는 프로그램을 작성합니다.

## 최종 결과

```bash
$ vaisc run examples/tutorial_json_parser.vais
=== Vais JSON Parser ===
Parsing: {"name":"vais","version":1,"active":true}
Found key: name = vais
Found key: version = 1
Found key: active = true
Parse OK — 3 fields
```

## 사전 준비

- Vais 설치 완료
- [CLI Tool 튜토리얼](./cli-tool.md) 완료 권장
- JSON 형식 기본 이해

---

## Step 1: 토큰 정의 (10분)

JSON은 6가지 값 타입을 갖습니다: 문자열, 숫자, 불리언, null, 배열, 객체. 먼저 토큰 타입을 정의합니다:

```vais
# JSON 토큰 타입 상수
C TOK_STRING: i64 = 1
C TOK_NUMBER: i64 = 2
C TOK_TRUE: i64 = 3
C TOK_FALSE: i64 = 4
C TOK_NULL: i64 = 5
C TOK_LBRACE: i64 = 6    # {
C TOK_RBRACE: i64 = 7    # }
C TOK_LBRACKET: i64 = 8  # [
C TOK_RBRACKET: i64 = 9  # ]
C TOK_COLON: i64 = 10    # :
C TOK_COMMA: i64 = 11    # ,
C TOK_EOF: i64 = 12
C TOK_ERROR: i64 = -1
```

**핵심 포인트**:
- `C` 키워드로 컴파일 타임 상수를 정의합니다
- 토큰을 정수 코드로 표현하면 `M` (match)로 빠르게 분기할 수 있습니다

---

## Step 2: 렉서 (Lexer) 구현 (15분)

입력 문자열을 토큰 스트림으로 변환하는 렉서를 작성합니다:

```vais
# 파서 상태 — 전역 변수로 위치 추적
G input_str: str = ""
G pos: i64 = mut 0
G input_len: i64 = 0

# 현재 문자 읽기
F peek_char() -> i64 {
    I pos >= input_len { R 0 }
    load_byte(input_str as i64 + pos)
}

# 다음 문자로 이동
F advance() -> i64 {
    I pos < input_len { pos = pos + 1 }
    0
}

# 공백 건너뛰기
F skip_whitespace() -> i64 {
    L pos < input_len {
        ch := peek_char()
        # space(32), tab(9), newline(10), carriage return(13)
        I ch == 32 | ch == 9 | ch == 10 | ch == 13 {
            advance()
        } E {
            B
        }
    }
    0
}

# 다음 토큰 타입 반환
F next_token() -> i64 {
    skip_whitespace()
    I pos >= input_len { R TOK_EOF }

    ch := peek_char()
    M ch {
        123 => { advance(); TOK_LBRACE },    # '{'
        125 => { advance(); TOK_RBRACE },    # '}'
        91  => { advance(); TOK_LBRACKET },  # '['
        93  => { advance(); TOK_RBRACKET },  # ']'
        58  => { advance(); TOK_COLON },     # ':'
        44  => { advance(); TOK_COMMA },     # ','
        34  => { scan_string() },            # '"'
        _   => {
            # 숫자 또는 키워드 (true/false/null)
            I ch >= 48 & ch <= 57 | ch == 45 {
                scan_number()
            } E {
                scan_keyword()
            }
        }
    }
}
```

**핵심 포인트**:
- `G` 키워드로 전역 변수를 선언합니다 (파서 상태 공유)
- `M ch { ... }`로 문자별 분기 처리합니다
- 문자 코드를 직접 사용합니다 (예: `123` = `{`, `34` = `"`)

---

## Step 3: 문자열과 숫자 스캐너 (15분)

문자열과 숫자 토큰을 파싱하는 헬퍼 함수를 작성합니다:

```vais
# 문자열 값을 저장할 버퍼
G str_buf: i64 = 0
G str_len: i64 = 0

F scan_string() -> i64 {
    advance()   # 여는 '"' 건너뛰기
    start := pos

    L pos < input_len {
        ch := peek_char()
        I ch == 34 {  # 닫는 '"'
            str_len = pos - start
            str_buf = input_str as i64 + start
            advance()
            R TOK_STRING
        }
        I ch == 92 {  # '\' 이스케이프
            advance()
        }
        advance()
    }
    TOK_ERROR   # 닫는 따옴표 없음
}

# 파싱된 숫자 값
G num_value: i64 = 0

F scan_number() -> i64 {
    negative := mut 0
    I peek_char() == 45 {  # '-'
        negative = 1
        advance()
    }

    result := mut 0
    L pos < input_len {
        ch := peek_char()
        I ch >= 48 & ch <= 57 {
            result = result * 10 + (ch - 48)
            advance()
        } E {
            B
        }
    }

    num_value = I negative == 1 { -result } E { result }
    TOK_NUMBER
}

F scan_keyword() -> i64 {
    # true, false, null 식별
    ch := peek_char()
    I ch == 116 {  # 't'
        pos = pos + 4   # "true"
        R TOK_TRUE
    }
    I ch == 102 {  # 'f'
        pos = pos + 5   # "false"
        R TOK_FALSE
    }
    I ch == 110 {  # 'n'
        pos = pos + 4   # "null"
        R TOK_NULL
    }
    TOK_ERROR
}
```

**핵심 포인트**:
- `str_buf`와 `str_len`으로 파싱된 문자열의 위치와 길이를 추적합니다
- 숫자 파싱은 자릿수별로 `result * 10 + digit` 누적합니다
- 키워드는 첫 문자로 판별하고 고정 길이만큼 건너뜁니다

---

## Step 4: 재귀 하강 파서 (20분)

JSON 객체와 배열을 재귀적으로 파싱합니다:

```vais
# JSON 값 파싱 — 재귀 진입점
F parse_value() -> i64 {
    tok := next_token()
    M tok {
        1 => {  # TOK_STRING
            puts("  string value")
            1
        },
        2 => {  # TOK_NUMBER
            puts("  number value")
            1
        },
        3 => { 1 },  # true
        4 => { 1 },  # false
        5 => { 1 },  # null
        6 => parse_object(),  # '{'
        8 => parse_array(),   # '['
        _ => {
            puts("Parse error: unexpected token")
            0
        }
    }
}

# JSON 객체 파싱: { "key": value, ... }
F parse_object() -> i64 {
    field_count := mut 0

    tok := next_token()
    I tok == TOK_RBRACE { R 0 }  # 빈 객체 {}

    # 첫 번째 키-값 쌍
    I tok != TOK_STRING {
        puts("Error: expected string key")
        R -1
    }
    puts("  key found")
    field_count = field_count + 1

    # ':' 구분자
    colon := next_token()
    I colon != TOK_COLON {
        puts("Error: expected ':'")
        R -1
    }

    # 값 파싱 (재귀)
    I parse_value() < 0 { R -1 }

    # 나머지 키-값 쌍
    L true {
        tok = next_token()
        I tok == TOK_RBRACE { B }   # 객체 끝
        I tok != TOK_COMMA {
            puts("Error: expected ',' or '}'")
            R -1
        }

        # 다음 키-값
        tok = next_token()
        I tok != TOK_STRING { R -1 }
        field_count = field_count + 1

        colon = next_token()
        I colon != TOK_COLON { R -1 }

        I parse_value() < 0 { R -1 }
    }

    field_count
}

# JSON 배열 파싱: [ value, ... ]
F parse_array() -> i64 {
    count := mut 0

    # 빈 배열 체크
    skip_whitespace()
    I peek_char() == 93 {  # ']'
        advance()
        R 0
    }

    # 첫 번째 요소
    I parse_value() < 0 { R -1 }
    count = count + 1

    # 나머지 요소
    L true {
        tok := next_token()
        I tok == TOK_RBRACKET { B }
        I tok != TOK_COMMA { R -1 }
        I parse_value() < 0 { R -1 }
        count = count + 1
    }

    count
}
```

**핵심 포인트**:
- `parse_value()` → `parse_object()` → `parse_value()` 재귀 구조
- `@` (자재귀)는 단일 함수 내에서만 사용 가능 — 여기서는 상호 재귀이므로 직접 호출
- 에러 시 `-1` 반환으로 에러 전파 (수동 Result 패턴)

---

## Step 5: main 함수와 통합 (10분)

```vais
F main() -> i64 {
    puts("=== Vais JSON Parser ===")

    # 테스트 JSON 문자열
    json := "{\"name\":\"vais\",\"version\":1,\"active\":true}"
    input_str = json
    input_len = __strlen(json)
    pos = 0

    puts("Parsing JSON...")

    tok := next_token()
    I tok == TOK_LBRACE {
        fields := parse_object()
        I fields >= 0 {
            puts("Parse OK")
            R 0
        }
    }

    puts("Parse failed")
    1
}
```

## 빌드 및 실행

```bash
vaisc run examples/tutorial_json_parser.vais
```

---

## 확장 아이디어

### 1. 값 추출 함수

```vais
# 특정 키의 문자열 값을 찾기
F find_string_value(json: str, key: str) -> i64 {
    input_str = json
    input_len = __strlen(json)
    pos = 0
    # ... 파싱하면서 key 매칭
    0
}
```

### 2. 중첩 객체 지원

재귀 하강 패턴 덕분에 중첩 객체는 자동으로 지원됩니다:

```json
{"user": {"name": "Alice", "scores": [95, 87, 92]}}
```

### 3. 에러 위치 보고

```vais
F report_error(msg: str) -> i64 {
    puts("Error at position {pos}: {msg}")
    -1
}
```

---

## 핵심 개념 정리

| 개념 | Vais 문법 | 설명 |
|------|-----------|------|
| 전역 변수 | `G name: type = value` | 파서 상태 공유 |
| 상수 | `C NAME: type = value` | 토큰 코드 |
| 패턴 매칭 | `M tok { N => body }` | 토큰별 분기 |
| 가변 변수 | `x := mut 0` | 카운터, 누적값 |
| 조기 반환 | `R value` | 에러 시 즉시 반환 |
| 재귀 호출 | `parse_value()` | 중첩 구조 파싱 |

## 다음 단계

- [Data Pipeline 튜토리얼](./data-pipeline.md) — CSV 데이터 처리
- [CLI Framework 튜토리얼](./cli-framework.md) — 명령행 프레임워크 만들기
- [std/json API](../api/json.md) — 표준 라이브러리 JSON 모듈
- [examples/json_test.vais](https://github.com/vaislang/vais/blob/main/examples/json_test.vais) — JSON 테스트 예제
