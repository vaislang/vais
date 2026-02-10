# Regex

## 개요

Regex 모듈은 정규 표현식 패턴 매칭을 제공합니다. 리터럴 문자, 메타문자(`.`, `*`, `+`, `?`), 앵커(`^`, `$`), 문자 클래스(`[abc]`, `[^abc]`)를 지원하며, 재귀 하강 매칭 엔진으로 구현됩니다.

## Quick Start

```vais
U std/regex

F main() -> i64 {
    pattern := regex_new("hello.*world")
    I regex_match(pattern, "hello beautiful world") {
        print_str("매칭 성공!")
    }
    R 0
}
```

## API 요약

| 함수 | 설명 |
|------|------|
| `regex_new(pattern)` | 패턴 컴파일 → Regex |
| `regex_match(regex, text)` | 전체 문자열 매칭 여부 |
| `regex_find(regex, text)` | 부분 매칭 찾기 |
| `regex_find_all(regex, text)` | 모든 매칭 위치 배열 |
| `regex_replace(regex, text, replacement)` | 매칭 부분 교체 |
| `regex_split(regex, text)` | 매칭 부분으로 분할 |
| `regex_capture(regex, text)` | 캡처 그룹 추출 |

## 지원 문법

| 패턴 | 설명 | 예제 |
|------|------|------|
| `abc` | 리터럴 문자 | `abc` → "abc" 매칭 |
| `.` | 임의 문자 (개행 제외) | `a.c` → "abc", "a1c" 매칭 |
| `*` | 0회 이상 반복 | `ab*c` → "ac", "abc", "abbc" 매칭 |
| `+` | 1회 이상 반복 | `ab+c` → "abc", "abbc" 매칭 |
| `?` | 0 또는 1회 | `ab?c` → "ac", "abc" 매칭 |
| `^` | 문자열 시작 앵커 | `^hello` → "hello world" 매칭 |
| `$` | 문자열 끝 앵커 | `world$` → "hello world" 매칭 |
| `[abc]` | 문자 클래스 (a or b or c) | `[aeiou]` → 모음 매칭 |
| `[^abc]` | 부정 문자 클래스 | `[^0-9]` → 숫자 아님 |
| `[a-z]` | 범위 | `[a-z]` → 소문자 |

## 실용 예제

### 예제 1: 이메일 검증

```vais
U std/regex

F is_valid_email(email: i64) -> i64 {
    # 단순화된 이메일 패턴: xxx@xxx.xxx
    pattern := regex_new("^[a-zA-Z0-9]+@[a-zA-Z0-9]+\\.[a-zA-Z]+$")
    R regex_match(pattern, email)
}

F main() -> i64 {
    I is_valid_email("user@example.com") {
        print_str("유효한 이메일")
    } E {
        print_str("무효한 이메일")
    }
    R 0
}
```

### 예제 2: 텍스트 검색 및 하이라이팅

```vais
U std/regex

F highlight_matches(text: i64, pattern: i64) -> i64 {
    regex := regex_new(pattern)
    matches := regex_find_all(regex, text)

    I matches.len() == 0 {
        print_str("매칭 없음")
        R 0
    }

    i := 0
    L i < matches.len() {
        pos := matches.get(i)
        print_str("매칭 위치: ~{pos}")
        i = i + 1
    }
    R matches.len()
}

F main() -> i64 {
    text := "The quick brown fox jumps over the lazy dog"
    count := highlight_matches(text, "\\b[a-z]{4}\\b")  # 4글자 단어
    print_str("총 ~{count}개 매칭")
    R 0
}
```

### 예제 3: 문자열 치환

```vais
U std/regex

F sanitize_input(input: i64) -> i64 {
    # 숫자 아닌 문자 제거
    pattern := regex_new("[^0-9]")
    R regex_replace(pattern, input, "")
}

F main() -> i64 {
    phone := "1-800-555-1234"
    clean := sanitize_input(phone)
    print_str(clean)  # "18005551234"
    R 0
}
```

### 예제 4: CSV 파싱 (Split)

```vais
U std/regex

F parse_csv_line(line: i64) -> Vec<i64> {
    # 쉼표로 분할
    pattern := regex_new(",")
    fields := regex_split(pattern, line)
    R fields
}

F main() -> i64 {
    csv := "Alice,30,Engineer"
    fields := parse_csv_line(csv)

    i := 0
    L i < fields.len() {
        print_str(fields.get(i))
        i = i + 1
    }
    R 0
}
```

### 예제 5: 캡처 그룹 추출

```vais
U std/regex

F extract_version(text: i64) -> i64 {
    # 버전 번호 추출: v1.2.3
    pattern := regex_new("v([0-9]+)\\.([0-9]+)\\.([0-9]+)")
    captures := regex_capture(pattern, text)

    I captures.len() >= 3 {
        major := captures.get(0)
        minor := captures.get(1)
        patch := captures.get(2)

        print_str("Major: ~{major}")
        print_str("Minor: ~{minor}")
        print_str("Patch: ~{patch}")
    }
    R 0
}

F main() -> i64 {
    extract_version("Version v2.5.10 released")
    R 0
}
```

## 주의사항

### 1. 탐욕적(Greedy) 매칭
`*`와 `+`는 기본적으로 탐욕적으로 매칭합니다. 최소 매칭이 필요하면 `*?`, `+?`를 사용하세요 (구현 시 지원 확인).

```vais
# 탐욕적: "a...z" 전체 매칭
pattern := regex_new("a.*z")
# "abc xyz" → "abc xyz" 전체 매칭

# 비탐욕적 (최소 매칭)
pattern := regex_new("a.*?z")
# "abc xyz" → "abc xz" 매칭
```

### 2. 이스케이프 문자
메타문자를 리터럴로 매칭하려면 `\\`로 이스케이프하세요.

```vais
# 점(.) 리터럴 매칭
pattern := regex_new("\\.")  # "." 자체 매칭
# 이스케이프 없으면 임의 문자 매칭

# 괄호 리터럴
pattern := regex_new("\\(\\)")  # "()" 매칭
```

### 3. 성능 고려
재귀 하강 파서는 백트래킹으로 인해 일부 패턴에서 지수 시간 복잡도를 가질 수 있습니다.

```vais
# 나쁜 패턴: (a+)+b는 백트래킹 폭발
pattern := regex_new("(a+)+b")
# "aaaaaaaaaaaaaaa" (b 없음) → 매우 느림

# 좋은 패턴: a+b
pattern := regex_new("a+b")
```

### 4. 개행 문자 매칭
`.`는 개행(`\n`)을 매칭하지 않습니다. 멀티라인 패턴은 명시적으로 처리하세요.

```vais
# 개행 포함 매칭
pattern := regex_new("[\\s\\S]*")  # 모든 공백 + 비공백 (개행 포함)
```

### 5. 앵커 조합
`^`와 `$`를 함께 사용하면 전체 문자열 정확 매칭을 의미합니다.

```vais
# 부분 매칭
pattern := regex_new("hello")
# "hello world" → 매칭

# 전체 매칭
pattern := regex_new("^hello$")
# "hello world" → 실패
# "hello" → 성공
```

### 6. 문자 클래스 범위
`[a-z]`는 ASCII 순서를 사용합니다. 유니코드는 지원하지 않습니다.

```vais
# ASCII 범위
[a-z]   # a ~ z
[A-Z]   # A ~ Z
[0-9]   # 0 ~ 9

# 조합
[a-zA-Z0-9_]  # 영숫자 + 언더스코어
```

### 7. 캡처 그룹 인덱스
캡처 그룹은 0부터 시작합니다. 전체 매칭은 별도 반환되거나, 그룹 0으로 제공될 수 있습니다.

```vais
# 패턴: (\\d+)-(\\d+)-(\\d+)
# 입력: "2024-02-10"
captures := regex_capture(pattern, text)
# captures[0] = "2024"
# captures[1] = "02"
# captures[2] = "10"
```

### 8. 반복 패턴 컴파일
같은 패턴을 반복 사용하면 한 번만 컴파일하세요.

```vais
# 나쁜 예
L i < 1000 {
    pattern := regex_new("[0-9]+")
    regex_match(pattern, inputs.get(i))
}

# 좋은 예
pattern := regex_new("[0-9]+")
L i < 1000 {
    regex_match(pattern, inputs.get(i))
}
```

### 9. 메모리 관리
컴파일된 Regex는 동적 할당된 패턴 트리입니다. 사용 후 `regex_free()`를 호출하세요.

```vais
pattern := regex_new("test")
D regex_free(pattern)

# 패턴 사용
```

### 10. 한계 인식
현재 구현은 기본 기능만 지원합니다. 고급 기능은 PCRE/Rust regex 라이브러리 바인딩을 고려하세요.

**미지원:**
- Lookahead/Lookbehind (`(?=...)`, `(?!...)`)
- 명명된 캡처 그룹 (`(?P<name>...)`)
- 유니코드 클래스 (`\p{L}`)
- 재귀 패턴

## See Also

- [Regex API Reference](../api/regex.md)
- [String Manipulation](../api/string.md)
- [Pattern Matching](../language/pattern-matching.md)
- [Text Processing](../guide/text-processing.md)
