# Union 타입 (O)

`O` 키워드는 C 스타일의 비태그(untagged) 유니온을 정의합니다. 모든 필드가 같은 메모리 위치를 공유합니다.

## 기본 문법

```vais
O Name {
    field1: Type1,
    field2: Type2,
    ...
}
```

## Enum과의 차이

| 특성 | `E` (Enum) | `O` (Union) |
|------|-----------|-------------|
| 태그 | 런타임 태그 포함 (tagged) | 태그 없음 (untagged) |
| 안전성 | 패턴 매칭으로 안전한 접근 | 호출자가 활성 필드를 추적해야 함 |
| 메모리 | `tag_size + max(variant_size)` | `max(field_size)` |
| 용도 | 일반적인 합 타입 | C interop, 저수준 메모리 제어 |

## 예제

### 기본 Union

```vais
O Value {
    i: i64,
    f: f64,
    b: bool
}

F main() -> i64 {
    v := Value { i: 42 }
    # v.i, v.f, v.b 모두 같은 메모리를 참조
    # 어떤 필드가 활성인지는 프로그래머가 관리
    v.i
}
```

### C Interop

Union은 C 라이브러리와의 FFI(Foreign Function Interface)에서 주로 사용됩니다:

```vais
# C의 union sockaddr_in과 호환
O SockAddr {
    sa_family: u16,
    sa_data: [u8; 14]
}

# C의 union epoll_data와 호환
O EpollData {
    ptr: i64,
    fd: i32,
    u32_val: u32,
    u64_val: u64
}
```

### 타입 변환 (Type Punning)

Union을 사용하면 같은 비트 패턴을 다른 타입으로 재해석할 수 있습니다:

```vais
O FloatBits {
    f: f64,
    bits: u64
}

F float_to_bits(val: f64) -> u64 {
    fb := FloatBits { f: val }
    fb.bits
}
```

## 메모리 레이아웃

Union의 크기는 가장 큰 필드의 크기와 같습니다. 모든 필드는 오프셋 0에서 시작합니다:

```
O Example {        메모리 레이아웃:
    a: i8,         [        a        ]
    b: i32,        [ b  b  b  b      ]
    c: i64         [ c  c  c  c  c  c  c  c ]
}
# sizeof(Example) = 8 (i64의 크기)
# 모든 필드가 offset 0에서 시작
```

## 제네릭 Union

Union도 제네릭 타입 파라미터를 지원합니다:

```vais
O Either<A, B> {
    left: A,
    right: B
}
```

## 주의사항

- Union 필드 접근은 **안전하지 않습니다**. 활성화되지 않은 필드를 읽으면 정의되지 않은 동작이 발생할 수 있습니다.
- C interop이 아닌 일반적인 경우에는 `E` (enum)을 사용하는 것을 권장합니다. Enum은 태그를 통해 안전한 패턴 매칭을 제공합니다.
- Union은 Drop trait를 자동으로 호출하지 않습니다. 리소스 정리가 필요하면 수동으로 처리해야 합니다.
