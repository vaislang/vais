# 에러 처리 모범 사례

## 개요

Vais는 예외(exceptions) 대신 **명시적인 에러 타입**을 사용하는 시스템 프로그래밍 언어입니다. 모든 실패 가능한 연산은 `Result<T, E>` 또는 `Option<T>` 타입을 반환하여 컴파일 타임에 에러 처리를 강제합니다.

### 핵심 원칙
- **명시성**: 함수 시그니처에서 에러 가능성이 드러남
- **타입 안전성**: 컴파일러가 처리되지 않은 에러를 감지
- **제로 비용 추상화**: 런타임 오버헤드 없음

## `Result<T, E>` 패턴

### 기본 구조

`Result` enum은 성공(`Ok`)과 실패(`Err`) 두 가지 상태를 표현합니다:

```vais
E Result<T, E> {
    Ok(T),
    Err(E)
}
```

### 함수에서 Result 반환

```vais
U std.io

F read_config(path: str) -> Result<str, str> {
    file := open(path)?
    content := read_to_string(file)?
    close(file)
    Result.Ok(content)
}
```

### `?` 연산자로 에러 전파

`?` 연산자는 `Result`가 `Err`일 때 즉시 함수에서 반환합니다:

```vais
F parse_and_process(data: str) -> Result<i64, str> {
    # parse가 실패하면 즉시 Err 반환
    num := parse_i64(data)?

    # 검증 실패 시 커스텀 에러
    I num < 0 {
        R Result.Err("음수는 허용되지 않습니다")
    }

    Result.Ok(num * 2)
}
```

### `!` 연산자로 언래핑

`!` 연산자는 `Ok` 값을 추출하고, `Err`일 경우 패닉을 발생시킵니다:

```vais
F main() -> i64 {
    # 확실히 성공하는 경우에만 사용
    config := read_config("app.conf")!

    print_str(config)
    0
}
```

**주의**: `!`는 프로토타입이나 테스트 코드에서만 사용하고, 프로덕션 코드에서는 명시적 처리를 권장합니다.

### M (match)로 Result 처리

```vais
F safe_divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 {
        R Result.Err("0으로 나눌 수 없습니다")
    }
    Result.Ok(a / b)
}

F main() -> i64 {
    result := safe_divide(10, 2)

    M result {
        Result.Ok(value) => {
            print_str("결과: ")
            print_i64(value)
        },
        Result.Err(err) => {
            print_str("에러: ")
            print_str(err)
        }
    }

    0
}
```

## `Option<T>` 패턴

### 기본 구조

`Option`은 값이 있을 수도 없을 수도 있는 상황을 표현합니다:

```vais
E Option<T> {
    Some(T),
    None
}
```

### 널 안전성

```vais
F find_user(id: i64) -> Option<str> {
    I id == 1 {
        R Option.Some("Alice")
    }
    Option.None
}

F main() -> i64 {
    user := find_user(1)

    M user {
        Option.Some(name) => {
            print_str("찾은 사용자: ")
            print_str(name)
        },
        Option.None => {
            print_str("사용자를 찾을 수 없습니다")
        }
    }

    0
}
```

### `?`와 `!` 연산자 사용

```vais
F get_first_char(s: str) -> Option<i64> {
    I strlen(s) == 0 {
        R Option.None
    }
    Option.Some(load_byte(s, 0))
}

F process_string(s: str) -> Option<i64> {
    # Option에서도 ? 연산자 사용 가능
    first := get_first_char(s)?
    Option.Some(first + 1)
}

F main() -> i64 {
    # ! 연산자로 Some 값 추출 (None이면 패닉)
    ch := get_first_char("Hello")!
    print_i64(ch)
    0
}
```

## 커스텀 에러 타입

### 에러 enum 정의

```vais
E FileError {
    NotFound(str),
    PermissionDenied(str),
    IoError(str)
}

F read_file(path: str) -> Result<str, FileError> {
    I !file_exists(path) {
        R Result.Err(FileError.NotFound(path))
    }

    I !can_read(path) {
        R Result.Err(FileError.PermissionDenied(path))
    }

    # 실제 읽기 로직...
    Result.Ok("file content")
}
```

### 에러 변환 패턴

```vais
E ParseError {
    InvalidFormat(str),
    OutOfRange(i64)
}

F parse_positive_number(s: str) -> Result<i64, ParseError> {
    # 기본 파싱
    M parse_i64(s) {
        Result.Ok(num) => {
            I num < 0 {
                R Result.Err(ParseError.OutOfRange(num))
            }
            Result.Ok(num)
        },
        Result.Err(_) => {
            Result.Err(ParseError.InvalidFormat(s))
        }
    }
}
```

## 에러 전파 패턴

### `?` 체이닝

```vais
U std.io

F load_and_parse_config(path: str) -> Result<i64, str> {
    content := read_file(path)?
    trimmed := trim(content)?
    number := parse_i64(trimmed)?
    Result.Ok(number)
}
```

### 파이프라인에서의 에러 처리

```vais
F process_data(input: str) -> Result<i64, str> {
    input
        |> validate_input
        |> parse_number
        |> transform_value
}

F validate_input(s: str) -> Result<str, str> {
    I strlen(s) == 0 {
        R Result.Err("빈 입력")
    }
    Result.Ok(s)
}

F parse_number(s: str) -> Result<i64, str> {
    parse_i64(s)
}

F transform_value(n: i64) -> Result<i64, str> {
    Result.Ok(n * 2)
}
```

## 모범 사례

### 1. 라이브러리 vs 애플리케이션 에러 처리

**라이브러리 코드**: 항상 `Result`를 반환하여 호출자가 결정하도록 합니다.

```vais
# ✅ 좋은 예: 라이브러리 함수
F lib_parse(data: str) -> Result<i64, str> {
    I strlen(data) == 0 {
        R Result.Err("빈 데이터")
    }
    parse_i64(data)
}
```

**애플리케이션 코드**: 최상위에서 에러를 처리하거나 의미 있는 에러 메시지를 제공합니다.

```vais
# ✅ 좋은 예: 애플리케이션 엔트리포인트
F main() -> i64 {
    M run_app() {
        Result.Ok(_) => {
            print_str("성공적으로 완료되었습니다")
            0
        },
        Result.Err(e) => {
            print_str("에러: ")
            print_str(e)
            1
        }
    }
}

F run_app() -> Result<i64, str> {
    config := load_config()?
    data := process_data(config)?
    Result.Ok(data)
}
```

### 2. 에러 메시지 가이드라인

**구체적이고 실행 가능한 정보를 제공하세요**:

```vais
# ❌ 나쁜 예
F open_file(path: str) -> Result<File, str> {
    Result.Err("에러 발생")
}

# ✅ 좋은 예
F open_file(path: str) -> Result<File, str> {
    I !file_exists(path) {
        R Result.Err("파일을 찾을 수 없습니다: " + path)
    }
    # ...
}
```

### 3. 절대 하지 말아야 할 것들

**에러 무시하지 않기**:

```vais
# ❌ 절대 금지
F bad_example() -> i64 {
    _ := might_fail()  # 에러 무시!
    0
}

# ✅ 올바른 방법
F good_example() -> Result<i64, str> {
    might_fail()?
    Result.Ok(0)
}
```

**과도한 `!` 사용 금지**:

```vais
# ❌ 나쁜 예: 프로덕션 코드에서 !
F process() -> i64 {
    data := read_file("data.txt")!
    parse_i64(data)!
}

# ✅ 좋은 예: 명시적 에러 처리
F process() -> Result<i64, str> {
    data := read_file("data.txt")?
    parse_i64(data)
}
```

## 실전 예제

### 파일 읽기 에러 처리

```vais
U std.io

E FileError {
    NotFound(str),
    ReadError(str)
}

F read_config_file(path: str) -> Result<str, FileError> {
    # 파일 존재 확인
    I !file_exists(path) {
        R Result.Err(FileError.NotFound(path))
    }

    # 파일 열기
    file := M open(path) {
        Result.Ok(f) => f,
        Result.Err(e) => {
            R Result.Err(FileError.ReadError(e))
        }
    }

    # 내용 읽기
    content := M read_to_string(file) {
        Result.Ok(c) => c,
        Result.Err(e) => {
            close(file)
            R Result.Err(FileError.ReadError(e))
        }
    }

    close(file)
    Result.Ok(content)
}

F main() -> i64 {
    M read_config_file("config.txt") {
        Result.Ok(content) => {
            print_str("설정 파일 내용: ")
            print_str(content)
            0
        },
        Result.Err(FileError.NotFound(path)) => {
            print_str("파일을 찾을 수 없습니다: ")
            print_str(path)
            1
        },
        Result.Err(FileError.ReadError(msg)) => {
            print_str("읽기 에러: ")
            print_str(msg)
            2
        }
    }
}
```

### 네트워크 요청 에러 처리

```vais
U std.net

E HttpError {
    ConnectionFailed(str),
    Timeout,
    InvalidResponse(i64)
}

F fetch_data(url: str) -> Result<str, HttpError> {
    # 연결 시도
    conn := M connect(url) {
        Result.Ok(c) => c,
        Result.Err(e) => {
            R Result.Err(HttpError.ConnectionFailed(e))
        }
    }

    # 요청 전송 (타임아웃 5초)
    response := M send_request(conn, 5000) {
        Result.Ok(r) => r,
        Result.Err("timeout") => {
            R Result.Err(HttpError.Timeout)
        },
        Result.Err(e) => {
            R Result.Err(HttpError.ConnectionFailed(e))
        }
    }

    # 상태 코드 확인
    status := get_status_code(response)
    I status != 200 {
        R Result.Err(HttpError.InvalidResponse(status))
    }

    Result.Ok(read_response_body(response))
}
```

### 파싱 에러 처리

```vais
E JsonError {
    SyntaxError(i64, str),  # line, message
    TypeError(str)
}

F parse_user_json(json: str) -> Result<User, JsonError> {
    # JSON 파싱
    obj := M parse_json(json) {
        Result.Ok(o) => o,
        Result.Err(msg) => {
            line := find_error_line(json, msg)
            R Result.Err(JsonError.SyntaxError(line, msg))
        }
    }

    # 필드 추출
    name := M get_string_field(obj, "name") {
        Result.Ok(n) => n,
        Result.Err(_) => {
            R Result.Err(JsonError.TypeError("name 필드가 없거나 문자열이 아닙니다"))
        }
    }

    age := M get_i64_field(obj, "age") {
        Result.Ok(a) => a,
        Result.Err(_) => {
            R Result.Err(JsonError.TypeError("age 필드가 없거나 숫자가 아닙니다"))
        }
    }

    Result.Ok(User { name: name, age: age })
}

S User {
    name: str,
    age: i64
}
```

## 요약

Vais의 에러 처리는 다음 원칙을 따릅니다:

1. **명시적 타입**: `Result<T, E>`와 `Option<T>`로 실패 가능성 표현
2. **`?` 연산자**: 에러 전파를 간결하게
3. **`!` 연산자**: 프로토타입/테스트 전용, 프로덕션에서는 신중히
4. **패턴 매칭**: `M`으로 모든 경우를 처리
5. **커스텀 에러 타입**: 도메인 특화 에러로 명확성 향상

이러한 패턴을 통해 Vais는 안전하고 유지보수 가능한 시스템 프로그래밍을 지원합니다.
