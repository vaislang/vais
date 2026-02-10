# Vec

## 개요

`Vec<T>`는 동적으로 크기가 변하는 배열(Dynamic Array)로, 임의 타입 T의 요소를 메모리 연속적으로 저장합니다. 자동 크기 조정(resizing)과 타입 안전 메모리 접근(`load_typed`/`store_typed`)을 지원합니다.

## Quick Start

```vais
U std/vec

F main() -> i64 {
    v := Vec::new()
    v.push(10)
    v.push(20)
    print_i64(v.get(0))  # 10
    R 0
}
```

## API 요약

| 함수 | 설명 | 시간 복잡도 |
|------|------|------------|
| `Vec::new()` | 빈 Vec 생성 (capacity=16) | O(1) |
| `Vec::with_capacity(n)` | 지정 capacity로 생성 | O(n) |
| `push(val)` | 끝에 요소 추가 | O(1) 평균 |
| `pop()` | 끝 요소 제거 반환 | O(1) |
| `get(idx)` | 인덱스로 읽기 | O(1) |
| `set(idx, val)` | 인덱스에 쓰기 | O(1) |
| `insert(idx, val)` | 중간 삽입 | O(n) |
| `remove(idx)` | 중간 제거 | O(n) |
| `len()` | 요소 개수 | O(1) |
| `capacity()` | 할당 용량 | O(1) |
| `is_empty()` | 빈 벡터 여부 | O(1) |
| `clear()` | 모든 요소 제거 | O(1) |

## 실용 예제

### 예제 1: 기본 생성 및 순회

```vais
U std/vec

F main() -> i64 {
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)

    i := 0
    L i < v.len() {
        print_i64(v.get(i))
        i = i + 1
    }
    R 0
}
```

### 예제 2: 사전 할당으로 성능 최적화

```vais
U std/vec

F process_data(count: i64) -> Vec<i64> {
    # 재할당 방지: 미리 용량 확보
    v := Vec::with_capacity(count)
    i := 0
    L i < count {
        v.push(i * 2)
        i = i + 1
    }
    R v
}
```

### 예제 3: 중간 삽입 및 제거

```vais
U std/vec

F main() -> i64 {
    v := Vec::new()
    v.push(10)
    v.push(30)

    # 인덱스 1에 20 삽입 (10, 20, 30)
    v.insert(1, 20)

    # 인덱스 0 제거 (20, 30)
    removed := v.remove(0)
    print_i64(removed)  # 10

    R 0
}
```

### 예제 4: 제네릭 Vec 사용

```vais
U std/vec

S Point { x: i64, y: i64 }

F main() -> i64 {
    points := Vec::new()
    points.push(Point { x: 10, y: 20 })
    points.push(Point { x: 30, y: 40 })

    p := points.get(0)
    print_i64(p.x)  # 10
    R 0
}
```

### 예제 5: 슬라이스로 변환

```vais
U std/vec

F sum_vec(v: Vec<i64>) -> i64 {
    # Vec을 슬라이스로 변환하여 이터레이션
    slice := v.as_slice()
    total := 0
    i := 0
    L i < slice.len() {
        total = total + slice[i]
        i = i + 1
    }
    R total
}

F main() -> i64 {
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)
    print_i64(sum_vec(v))  # 6
    R 0
}
```

## 주의사항

### 1. 범위 검사
`get()`과 `set()`은 경계 검사를 수행합니다. 범위 밖 접근 시 0 또는 기본값을 반환하지만, 프로덕션 코드에서는 `len()` 체크를 먼저 수행하세요.

```vais
# 안전한 패턴
I idx >= 0 && idx < v.len() {
    val := v.get(idx)
    # 사용
}
```

### 2. 용량 관리
빈번한 `push()` 호출 시 자동 재할당이 발생합니다. 최종 크기를 미리 알면 `with_capacity()`를 사용하세요.

```vais
# 나쁜 예: 10,000번 재할당 가능
v := Vec::new()
L i < 10000 { v.push(i); i = i + 1 }

# 좋은 예: 재할당 0회
v := Vec::with_capacity(10000)
L i < 10000 { v.push(i); i = i + 1 }
```

### 3. 메모리 누수 방지
Vec는 GC를 사용하거나, 명시적으로 `free(v.data)`를 호출해야 합니다. 긴 생명주기 Vec는 메모리 누수 위험이 있습니다.

### 4. 타입 크기 제약
현재 구현은 `type_size()` 빌트인에 의존합니다. 복잡한 nested 타입은 `elem_size` 필드를 직접 설정해야 할 수 있습니다.

### 5. 슬라이스 변환
`as_slice()`는 Vec의 내부 버퍼를 참조하는 fat pointer(`&[T]`)를 반환합니다. Vec가 재할당되면 슬라이스가 무효화될 수 있습니다.

```vais
v := Vec::new()
v.push(1)
slice := v.as_slice()  # 버퍼 주소 저장
v.push(2)  # 재할당 발생 가능!
# slice는 이제 dangling pointer
```

## See Also

- [Vec API Reference](../api/vec.md)
- [Slice Types](../language/slices.md)
- [Collections Overview](../api/collections.md)
- [Memory Management](../api/memory.md)
