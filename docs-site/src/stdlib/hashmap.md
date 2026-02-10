# HashMap

## 개요

`HashMap<K,V>`는 해시 테이블 기반 key-value 저장소로, 평균 O(1) 삽입/조회를 제공합니다. Separate chaining으로 충돌을 처리하며, 제네릭 키 타입을 지원합니다. 문자열 키 전용 최적화 버전으로 `StringMap`도 제공됩니다.

## Quick Start

```vais
U std/hashmap

F main() -> i64 {
    m := HashMap::new()
    m.set(1, 100)
    m.set(2, 200)
    val := m.get(1)  # 100
    R 0
}
```

## API 요약

| 함수 | 설명 | 시간 복잡도 |
|------|------|------------|
| `HashMap::new()` | 빈 HashMap 생성 (capacity=16) | O(1) |
| `HashMap::with_capacity(n)` | 지정 capacity로 생성 | O(n) |
| `set(key, val)` | 키-값 삽입/업데이트 | O(1) 평균 |
| `get(key)` | 키로 값 조회 | O(1) 평균 |
| `contains(key)` | 키 존재 여부 | O(1) 평균 |
| `remove(key)` | 키-값 쌍 제거 | O(1) 평균 |
| `len()` | 요소 개수 | O(1) |
| `is_empty()` | 빈 맵 여부 | O(1) |
| `clear()` | 모든 요소 제거 | O(n) |

## 실용 예제

### 예제 1: 기본 사용법

```vais
U std/hashmap

F main() -> i64 {
    scores := HashMap::new()

    # 삽입
    scores.set(100, 85)  # ID 100 학생의 점수 85
    scores.set(101, 92)
    scores.set(102, 78)

    # 조회
    I scores.contains(101) {
        score := scores.get(101)
        print_i64(score)  # 92
    }

    # 제거
    scores.remove(102)
    R 0
}
```

### 예제 2: 빈도 카운팅

```vais
U std/hashmap
U std/vec

F count_frequencies(numbers: Vec<i64>) -> HashMap<i64,i64> {
    freq := HashMap::new()
    i := 0
    L i < numbers.len() {
        num := numbers.get(i)
        count := I freq.contains(num) { freq.get(num) } E { 0 }
        freq.set(num, count + 1)
        i = i + 1
    }
    R freq
}

F main() -> i64 {
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(1)
    v.push(3)
    v.push(2)
    v.push(1)

    freq := count_frequencies(v)
    print_i64(freq.get(1))  # 3 (1이 3번 등장)
    R 0
}
```

### 예제 3: 캐시 구현

```vais
U std/hashmap

S Cache<K,V> {
    map: HashMap<K,V>,
    max_size: i64
}

X Cache<K,V> {
    F new(max_size: i64) -> Cache<K,V> {
        Cache {
            map: HashMap::with_capacity(max_size),
            max_size: max_size
        }
    }

    F get_or_compute(&self, key: K, compute_fn: F(K) -> V) -> V {
        I self.map.contains(key) {
            R self.map.get(key)
        }

        # 캐시 미스 - 계산 후 저장
        value := compute_fn(key)
        I self.map.len() < self.max_size {
            self.map.set(key, value)
        }
        R value
    }
}
```

### 예제 4: StringMap 사용 (문자열 키 최적화)

```vais
U std/stringmap

F main() -> i64 {
    config := StringMap::new()

    # 문자열 키-값 저장
    config.set("host", "127.0.0.1")
    config.set("port", "8080")
    config.set("debug", "true")

    # 조회
    host := config.get("host")
    I host != 0 {
        print_str(host)  # "127.0.0.1"
    }

    R 0
}
```

### 예제 5: 해시 충돌 처리 확인

```vais
U std/hashmap

F main() -> i64 {
    m := HashMap::with_capacity(4)  # 작은 capacity로 충돌 유도

    # 많은 키 삽입 (자동 rehash)
    i := 0
    L i < 100 {
        m.set(i, i * 2)
        i = i + 1
    }

    # 모두 정상 조회 가능
    print_i64(m.get(50))  # 100
    print_i64(m.len())    # 100
    R 0
}
```

## 주의사항

### 1. 해시 함수 선택
현재 구현은 `mult_hash(key)` 함수를 사용합니다. 커스텀 타입을 키로 사용하려면 해시 함수를 구현해야 합니다.

```vais
# 커스텀 해시 필요
S CustomKey { id: i64, name: i64 }

F hash_custom(k: CustomKey) -> i64 {
    R k.id * 31 + hash_str(k.name)
}
```

### 2. Rehashing 오버헤드
load factor(len/capacity)가 0.75를 초과하면 자동 rehash가 발생합니다. 대량 삽입 전에 `with_capacity()`로 충분한 용량을 확보하세요.

```vais
# 나쁜 예: 여러 번 rehash
m := HashMap::new()  # capacity=16
L i < 10000 { m.set(i, i); i = i + 1 }  # ~log(n) rehash

# 좋은 예: rehash 0~1회
m := HashMap::with_capacity(15000)  # 10000 / 0.75 ≈ 13333
L i < 10000 { m.set(i, i); i = i + 1 }
```

### 3. 메모리 누수
제거된 Entry 노드는 명시적으로 `free()`되어야 합니다. 현재 `clear()`는 bucket 포인터만 초기화하므로, 긴 생명주기 HashMap은 누수 위험이 있습니다.

### 4. StringMap vs HashMap
문자열 키만 사용한다면 `StringMap`이 더 효율적입니다. 내부적으로 문자열 전용 해시 함수를 사용하여 성능이 향상됩니다.

```vais
# 문자열 키 → StringMap 사용
config := StringMap::new()
config.set("key", "value")

# 정수 키 → HashMap 사용
scores := HashMap::new()
scores.set(123, 456)
```

### 5. 반복 순서 미보장
HashMap은 해시 순서로 저장되므로, 삽입 순서와 무관합니다. 순서가 중요하면 `BTreeMap`을 사용하세요.

### 6. 동시성 주의
HashMap은 thread-safe하지 않습니다. 멀티스레드 환경에서는 `Mutex<HashMap<K,V>>`로 래핑하세요.

```vais
U std/sync
U std/hashmap

global_cache := Mutex::new(HashMap::new())

F thread_safe_insert(key: i64, val: i64) -> i64 {
    guard := global_cache.lock()
    cache := guard.get_inner()
    cache.set(key, val)
    R 0
}
```

## See Also

- [HashMap API Reference](../api/hashmap.md)
- [StringMap API Reference](../api/stringmap.md)
- [BTreeMap API Reference](../api/btreemap.md)
- [Collections Overview](../api/collections.md)
- [Hash Functions](../api/hash.md)
