# JSON

## 개요

JSON 모듈은 가볍고 빠른 JSON 파서 및 생성기를 제공합니다. null, bool, number(i64), string, array, object를 지원하며, 구조체 ↔ JSON 매핑을 통해 직렬화/역직렬화를 구현합니다.

## Quick Start

```vais
U std/json

F main() -> i64 {
    json_str := ~{"name": "Alice", "age": 30}
    obj := json_parse(json_str)

    name := json_get_string(obj, "name")
    age := json_get_number(obj, "age")

    print_str(name)  # "Alice"
    print_i64(age)   # 30
    R 0
}
```

## API 요약

### 파싱

| 함수 | 설명 |
|------|------|
| `json_parse(str)` | JSON 문자열 파싱 → JsonValue |
| `json_get_type(val)` | 타입 반환 (0=null, 1=bool, 2=number, 4=string, 5=array, 6=object) |

### 접근자

| 함수 | 설명 |
|------|------|
| `json_get_bool(val)` | bool 값 추출 |
| `json_get_number(val)` | i64 숫자 추출 |
| `json_get_string(val, key)` | object에서 string 추출 |
| `json_get_object(val, key)` | object에서 중첩 object 추출 |
| `json_get_array_item(val, idx)` | array 인덱스 접근 |
| `json_array_len(val)` | array 길이 |

### 생성

| 함수 | 설명 |
|------|------|
| `json_new_null()` | null 값 생성 |
| `json_new_bool(val)` | bool 값 생성 |
| `json_new_number(val)` | number 값 생성 |
| `json_new_string(str)` | string 값 생성 |
| `json_new_array()` | 빈 array 생성 |
| `json_new_object()` | 빈 object 생성 |
| `json_array_push(arr, val)` | array에 추가 |
| `json_object_set(obj, key, val)` | object에 키-값 추가 |
| `json_stringify(val)` | JsonValue → JSON 문자열 |

## 실용 예제

### 예제 1: JSON 파싱 및 접근

```vais
U std/json

F main() -> i64 {
    json_str := ~{
        "user": {
            "id": 123,
            "name": "Bob",
            "active": true
        }
    }

    root := json_parse(json_str)
    user := json_get_object(root, "user")

    id := json_get_number(user, "id")
    name := json_get_string(user, "name")
    active := json_get_bool(user, "active")

    print_i64(id)       # 123
    print_str(name)     # "Bob"
    print_i64(active)   # 1
    R 0
}
```

### 예제 2: JSON 배열 순회

```vais
U std/json

F main() -> i64 {
    json_str := ~{"scores": [85, 92, 78, 90]}

    root := json_parse(json_str)
    scores := json_get_array(root, "scores")

    len := json_array_len(scores)
    i := 0
    L i < len {
        score := json_get_array_item(scores, i)
        num := json_get_number(score)
        print_i64(num)
        i = i + 1
    }
    R 0
}
```

### 예제 3: 구조체 → JSON 직렬화

```vais
U std/json

S User {
    id: i64,
    name: i64,
    age: i64
}

F user_to_json(user: User) -> i64 {
    obj := json_new_object()
    json_object_set(obj, "id", json_new_number(user.id))
    json_object_set(obj, "name", json_new_string(user.name))
    json_object_set(obj, "age", json_new_number(user.age))
    R json_stringify(obj)
}

F main() -> i64 {
    user := User { id: 456, name: "Charlie", age: 28 }
    json_str := user_to_json(user)
    print_str(json_str)  # {"id":456,"name":"Charlie","age":28}
    R 0
}
```

### 예제 4: JSON → 구조체 역직렬화

```vais
U std/json

S Config {
    host: i64,
    port: i64,
    debug: i64
}

F parse_config(json_str: i64) -> Config {
    obj := json_parse(json_str)

    Config {
        host: json_get_string(obj, "host"),
        port: json_get_number(obj, "port"),
        debug: json_get_bool(obj, "debug")
    }
}

F main() -> i64 {
    json_str := ~{"host": "127.0.0.1", "port": 8080, "debug": true}
    config := parse_config(json_str)

    print_str(config.host)    # "127.0.0.1"
    print_i64(config.port)    # 8080
    print_i64(config.debug)   # 1
    R 0
}
```

### 예제 5: 중첩 JSON 생성

```vais
U std/json

F main() -> i64 {
    # {"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}

    alice := json_new_object()
    json_object_set(alice, "name", json_new_string("Alice"))
    json_object_set(alice, "age", json_new_number(30))

    bob := json_new_object()
    json_object_set(bob, "name", json_new_string("Bob"))
    json_object_set(bob, "age", json_new_number(25))

    users := json_new_array()
    json_array_push(users, alice)
    json_array_push(users, bob)

    root := json_new_object()
    json_object_set(root, "users", users)

    json_str := json_stringify(root)
    print_str(json_str)
    R 0
}
```

## 주의사항

### 1. 타입 안전
`json_get_*` 함수는 타입이 맞지 않으면 0 또는 null을 반환합니다. 사용 전 `json_get_type()`으로 확인하세요.

```vais
val := json_get_object(root, "key")
I json_get_type(val) == 6 {  # 6 = object
    # 안전하게 접근
    nested := json_get_string(val, "field")
} E {
    print_str("타입 불일치")
}
```

### 2. 메모리 관리
`json_parse()`는 동적 할당된 JsonValue를 반환합니다. 사용 후 `json_free()`를 호출하여 메모리를 해제하세요.

```vais
obj := json_parse(json_str)
D json_free(obj)  # defer로 자동 정리

# obj 사용
```

### 3. 부동소수점 미지원
현재 구현은 정수(i64)만 지원합니다. 소수는 1,000,000을 곱한 스케일된 정수로 저장됩니다.

```vais
# 3.14 → 3140000 (3.14 * 1000000)
val := json_new_number_scaled(3140000)
json_str := json_stringify(val)  # "3.14"
```

### 4. 대용량 JSON
재귀 파싱은 스택 오버플로 위험이 있습니다. 깊이 제한을 구현하거나, 이터레이티브 파서를 사용하세요.

```vais
# 최대 깊이 32
C MAX_JSON_DEPTH: i64 = 32

F safe_parse(json_str: i64, depth: i64) -> i64 {
    I depth > MAX_JSON_DEPTH {
        R 0  # 에러
    }
    # 파싱 로직...
}
```

### 5. 키 순서 미보장
JSON object는 해시맵 기반이므로, 키 순서가 보존되지 않습니다. 순서가 중요하면 array를 사용하세요.

```vais
# 키 순서 미보장
obj := json_parse(~{"a": 1, "b": 2, "c": 3})
# stringify 시 순서 변경 가능: {"c":3,"a":1,"b":2}
```

### 6. 이스케이프 문자
문자열 내 `"`, `\`, `\n` 등은 자동으로 이스케이프됩니다. 수동 이스케이프 불필요합니다.

```vais
str := ~{Hello\nWorld}
obj := json_new_string(str)
json_str := json_stringify(obj)  # "Hello\\nWorld"
```

### 7. 에러 처리
`json_parse()`는 실패 시 null 또는 부분 파싱 결과를 반환합니다. 항상 `json_get_type()`으로 유효성을 확인하세요.

```vais
obj := json_parse(invalid_json)
I json_get_type(obj) == 0 {  # 0 = null (파싱 실패)
    print_str("JSON 파싱 에러")
    R 1
}
```

### 8. UTF-8 지원
현재 구현은 ASCII만 완전 지원합니다. UTF-8 문자열은 바이트 단위로 처리되므로, 유니코드 이스케이프(`\uXXXX`)가 필요할 수 있습니다.

### 9. JSON Streaming
현재는 전체 문서를 메모리에 로드합니다. 대용량 스트림은 청크 단위 파싱을 구현하세요.

```vais
# 스트림 파싱 패턴
F parse_stream(file: File) -> Vec<i64> {
    results := Vec::new()
    L 1 {
        chunk := file.read_str(1024)
        I chunk == 0 { B }

        obj := json_parse(chunk)
        results.push(obj)
    }
    R results
}
```

### 10. 성능 최적화
- 반복 파싱 시 `json_parse()` 결과를 캐싱하세요.
- 대량 생성 시 `json_stringify()`를 한 번만 호출하세요.
- 중첩 접근은 변수에 저장하여 반복 조회를 피하세요.

```vais
# 나쁜 예: 반복 조회
L i < 100 {
    val := json_get_object(json_get_object(root, "data"), "field")
}

# 좋은 예: 한 번만 조회
data := json_get_object(root, "data")
L i < 100 {
    val := json_get_object(data, "field")
}
```

## See Also

- [JSON API Reference](../api/json.md)
- [Template API Reference](../api/template.md)
- [String Manipulation](../api/string.md)
- [File I/O](./file_io.md)
- [HTTP Client](../api/http_client.md)
