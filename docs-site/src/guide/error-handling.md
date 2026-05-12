# 에러 처리 패턴 가이드

Vais에서 에러를 안전하게 처리하는 방법을 배웁니다. Vais는 예외 대신 타입 기반의 에러 처리를 권장합니다.

## 에러 처리 전략

Vais는 여러 에러 처리 패턴을 지원합니다:

1. **에러 코드 반환** - 정수 반환값으로 상태 표시
2. **Option/Result 패턴** - 선택적 값과 에러 정보
3. **에러 전파** - 고수준 함수에 에러 처리 위임
4. **패닉 (panic)** - 복구 불가능한 에러

## 1. 에러 코드 패턴

### 기본 에러 코드

전통적인 C 스타일의 에러 코드를 사용합니다:

```vais
# 에러 코드 상수
G SUCCESS: i64 = 0
G ERROR_FILE_NOT_FOUND: i64 = 1
G ERROR_INVALID_INPUT: i64 = 2
G ERROR_PERMISSION_DENIED: i64 = 3

# 함수에서 에러 코드 반환
fn read_config(path: str) -> i64 {
    I path.len() == 0 {
        return ERROR_INVALID_INPUT
    }

    # 파일 읽기 시뮬레이션
    return SUCCESS
}

fn main() -> i64 {
    status := read_config("")

    I status == SUCCESS {
        puts("Configuration loaded")
    } else I status == ERROR_INVALID_INPUT {
        puts("Error: Invalid file path")
    } else I status == ERROR_FILE_NOT_FOUND {
        puts("Error: File not found")
    } else {
        puts("Error: Unknown error")
    }

    status
}
```

### 에러 코드 모듈화

여러 함수에서 재사용할 수 있도록 에러 코드를 모듈화합니다:

```vais
# errors.vais - 에러 정의 모듈
struct ErrorCode {
    code: i64
    message: str
}

fn error_success() -> ErrorCode = ErrorCode { code: 0, message: "Success" }
fn error_not_found() -> ErrorCode = ErrorCode { code: 1, message: "Not found" }
fn error_invalid_arg() -> ErrorCode = ErrorCode { code: 2, message: "Invalid argument" }
fn error_io() -> ErrorCode = ErrorCode { code: 3, message: "I/O error" }

fn is_error(err: ErrorCode) -> bool = err.code != 0
fn error_message(err: ErrorCode) -> str = err.message
```

## 2. Option 패턴

`Option` 타입으로 값이 있을 수도, 없을 수도 있는 경우를 표현합니다:

```vais
# Option 구조체 정의
enum Option<T> {
    Some(T),
    None
}

# Option을 반환하는 함수
fn find_user(id: i64) -> Option<str> {
    I id == 1 {
        return Option.Some("Alice")
    }
    return Option.None
}

# Option 값 처리
fn main() -> i64 {
    user := find_user(1)

    match user {
        Option.Some(name) => puts("Found user: {name}"),
        Option.None => puts("User not found")
    }

    0
}
```

### Option 체이닝

여러 Option 연산을 연쇄적으로 수행합니다:

```vais
struct User {
    id: i64
    name: str
    email: str
}

# Option 반환 함수들
fn find_user_by_id(id: i64) -> Option<User> {
    I id > 0 {
        return Option.Some(User { id: id, name: "User", email: "user@example.com" })
    }
    return Option.None
}

fn get_email(user: User) -> Option<str> {
    I user.email.len() > 0 {
        return Option.Some(user.email)
    }
    return Option.None
}

# 함수 조합
fn get_user_email(user_id: i64) -> Option<str> {
    user := find_user_by_id(user_id)
    match user {
        Option.Some(u) => get_email(u),
        Option.None => Option.None
    }
}
```

## 3. Result 패턴

에러 정보와 함께 성공/실패를 표현하는 패턴입니다:

```vais
# Result 구조체 정의
enum Result<T> {
    Ok(T),
    Err(str)
}

# Result를 반환하는 함수
fn parse_number(s: str) -> Result<i64> {
    # 간단한 파싱 (실제로는 더 복잡함)
    I s.len() == 0 {
        return Result.Err("Empty string")
    }

    # 성공 케이스
    return Result.Ok(42)
}

fn main() -> i64 {
    result := parse_number("123")

    match result {
        Result.Ok(num) => puts("Parsed: {num}"),
        Result.Err(err) => puts("Error: {err}")
    }

    0
}
```

### Result 에러 전파

에러를 상위 함수로 전파합니다:

```vais
enum FileError {
    NotFound,
    PermissionDenied,
    IoError
}

enum Result<T> {
    Ok(T),
    Err(FileError)
}

# 파일 읽기 함수
fn read_file(path: str) -> Result<str> {
    I path.len() == 0 {
        return Result.Err(FileError.NotFound)
    }

    # 파일 내용 읽기
    content := "file content"
    return Result.Ok(content)
}

# 파일 처리 함수 (에러 전파)
fn process_file(path: str) -> Result<i64> {
    content := read_file(path)

    match content {
        Result.Ok(data) => {
            # 데이터 처리
            line_count := 1
            return Result.Ok(line_count)
        },
        Result.Err(err) => {
            # 에러 전파
            return Result.Err(err)
        }
    }
}
```

## 4. 안전한 에러 처리 패턴

### 기본값 제공

```vais
fn get_config_value(key: str) -> i64 {
    # 기본값 반환
    0
}

fn main() -> i64 {
    # 에러 발생 시 기본값 사용
    timeout := get_config_value("timeout")

    # 0이 기본값 (에러 처리됨)
    actual_timeout := timeout > 0 ? timeout : 30
    puts("Timeout: {actual_timeout}")

    0
}
```

### 조건부 처리

```vais
fn divide(a: i64, b: i64) -> Result<i64> {
    I b == 0 {
        return Result.Err("Division by zero")
    }
    return Result.Ok(a / b)
}

fn main() -> i64 {
    result := divide(10, 2)

    match result {
        Result.Ok(value) => puts("Result: {value}"),
        Result.Err(msg) => puts("Cannot divide: {msg}")
    }

    0
}
```

## 5. 구조화된 로깅 통합

에러와 함께 로깅합니다:

```vais
# log.vais 사용 (표준 라이브러리)

enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal
}

struct Logger {
    level: LogLevel
}

impl Logger {
    fn log(&self, level: LogLevel, msg: str) {
        match level {
            LogLevel.Error => puts("[ERROR] {msg}"),
            LogLevel.Warn => puts("[WARN] {msg}"),
            LogLevel.Info => puts("[INFO] {msg}"),
            LogLevel.Debug => puts("[DEBUG] {msg}"),
            LogLevel.Fatal => {
                puts("[FATAL] {msg}")
                # 프로그램 종료
            }
        }
    }

    fn error(&self, msg: str) {
        self.log(LogLevel.Error, msg)
    }

    fn warn(&self, msg: str) {
        self.log(LogLevel.Warn, msg)
    }
}

fn main() -> i64 {
    logger := Logger { level: LogLevel.Info }

    result := divide(10, 0)

    match result {
        Result.Ok(value) => logger.log(LogLevel.Info, "Division successful"),
        Result.Err(msg) => logger.error("Division failed: {msg}")
    }

    0
}
```

## 6. 리소스 정리 (defer)

에러 발생 후에도 리소스를 정리하는 패턴입니다:

```vais
fn read_and_process(filename: str) -> Result<i64> {
    # 파일 열기
    handle := fopen(filename, "r")

    I handle == 0 {
        return Result.Err("Failed to open file")
    }

    # defer를 사용하여 파일 닫기 보장
    D fclose(handle)

    # 파일 처리
    line_count := 0
    # ... 파일 읽기 로직 ...

    return Result.Ok(line_count)
}
```

## 7. 복합 에러 처리 예제

실제 시나리오에서의 에러 처리:

```vais
struct Config {
    host: str
    port: i64
    timeout: i64
}

enum ConfigError {
    NotFound,
    InvalidFormat,
    MissingField
}

fn load_config(path: str) -> Result<Config> {
    # 1. 파일 존재 확인
    I path.len() == 0 {
        return Result.Err(ConfigError.NotFound)
    }

    # 2. 파일 읽기
    content := "host=localhost\nport=8080\ntimeout=30"

    # 3. 파싱
    I content.len() == 0 {
        return Result.Err(ConfigError.InvalidFormat)
    }

    # 4. 구성 객체 생성
    config := Config {
        host: "localhost",
        port: 8080,
        timeout: 30
    }

    return Result.Ok(config)
}

fn validate_config(config: Config) -> Result<bool> {
    I config.port <= 0 || config.port > 65535 {
        return Result.Err(ConfigError.InvalidFormat)
    }

    I config.timeout <= 0 {
        return Result.Err(ConfigError.InvalidFormat)
    }

    return Result.Ok(true)
}

fn main() -> i64 {
    # 설정 로드
    config_result := load_config("config.txt")

    I config_result {
        match config_result {
            Result.Ok(cfg) => {
                # 설정 검증
                valid := validate_config(cfg)
                match valid {
                    Result.Ok(_) => {
                        puts("Configuration loaded and validated")
                        puts("Host: {cfg.host}:{cfg.port}")
                    },
                    Result.Err(_) => puts("Configuration validation failed")
                }
            },
            Result.Err(err) => {
                match err {
                    ConfigError.NotFound => puts("Config file not found"),
                    ConfigError.InvalidFormat => puts("Invalid config format"),
                    ConfigError.MissingField => puts("Missing required field")
                }
            }
        }
    }

    0
}
```

## 모범 사례 (Best Practices)

### 1. 명확한 에러 타입

```vais
# Good: 명확한 에러 타입
enum DatabaseError {
    ConnectionFailed,
    QueryFailed,
    TimeoutError
}

# Bad: 문자열만 사용
fn bad_db_operation() -> Result<str> {
    return Result.Err("Some error occurred")
}
```

### 2. 에러 전파 명시

```vais
# 에러 전파를 함수 시그니처에 명시
fn fetch_user_data(id: i64) -> Result<str> {
    # ...에러 처리...
}
```

### 3. 컨텍스트 제공

```vais
fn process_records(count: i64) -> Result<bool> {
    I count <= 0 {
        # 좋은 에러 메시지: 무엇이 잘못되었는지 알 수 있음
        return Result.Err("Record count must be positive, got {count}")
    }
    return Result.Ok(true)
}
```

### 4. 조기 반환

```vais
fn validate_user(name: str, age: i64) -> Result<bool> {
    # 각 조건을 빠르게 검사하고 반환
    I name.len() == 0 { return Result.Err("Name is required") }
    I age < 0 { return Result.Err("Age cannot be negative") }
    I age > 150 { return Result.Err("Age seems invalid") }

    return Result.Ok(true)
}
```

## 테스트 에러 처리

```vais
# 에러를 예상하는 테스트
fn test_division_by_zero() {
    result := divide(10, 0)

    match result {
        Result.Ok(_) => puts("Test failed: should have returned error"),
        Result.Err(_) => puts("Test passed: error caught correctly")
    }
}

# 성공을 예상하는 테스트
fn test_valid_division() {
    result := divide(10, 2)

    match result {
        Result.Ok(value) => {
            I value == 5 {
                puts("Test passed: correct result")
            } else {
                puts("Test failed: wrong result")
            }
        },
        Result.Err(_) => puts("Test failed: unexpected error")
    }
}
```

## 다음 단계

- [성능 튜닝 가이드](./performance.md): 에러 처리의 성능 영향
- [코딩 스타일 가이드](./style-guide.md): 에러 처리 스타일
- [표준 라이브러리](../stdlib/stdlib.md): 표준 에러 타입
