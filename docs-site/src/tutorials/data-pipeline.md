# Tutorial: Data Pipeline 만들기

이 튜토리얼에서는 Vais로 데이터를 읽고, 변환하고, 출력하는 파이프라인을 만듭니다. CSV 형식의 데이터를 파싱하여 통계를 계산하고, 결과를 출력하는 프로그램입니다.

## 최종 결과

```bash
$ vaisc run examples/tutorial_pipeline.vais
=== Data Pipeline ===
Records: 5
Total score: 435
Average: 87
Max: 98
Min: 72
```

## 사전 준비

- Vais 설치 완료
- [CLI Tool 튜토리얼](./cli-tool.md) 완료 권장

---

## Step 1: 데이터 모델 (10분)

파이프라인에서 처리할 데이터 구조를 정의합니다:

```vais
# 학생 성적 레코드
S Record {
    id: i64
    score: i64
}

# 통계 결과
S Stats {
    count: i64
    total: i64
    max_val: i64
    min_val: i64
}

X Stats {
    F average(&self) -> i64 {
        I self.count == 0 { R 0 }
        R self.total / self.count
    }

    F print(&self) {
        puts("Records: {self.count}")
        puts("Total score: {self.total}")
        puts("Average: {self.average()}")
        puts("Max: {self.max_val}")
        puts("Min: {self.min_val}")
    }
}
```

**핵심 개념**:
- `S`로 데이터 모델 정의
- `X`로 메서드 추가 (self를 통한 필드 접근)
- `I`/`R`로 0 나눗셈 방어

---

## Step 2: 데이터 소스 (10분)

메모리 내 배열로 데이터를 구성합니다. 실전에서는 파일이나 네트워크에서 읽어옵니다:

```vais
N "C" {
    F malloc(size: i64) -> i64
    F free(ptr: i64) -> i64
    F strlen(s: str) -> i64
}

# 고정 배열로 데이터 생성
F create_dataset(buf: i64) -> i64 {
    # 5개 레코드: id, score 쌍
    # Record 0: id=1, score=85
    store_i64(buf, 0, 1)
    store_i64(buf, 8, 85)
    # Record 1: id=2, score=92
    store_i64(buf, 16, 2)
    store_i64(buf, 24, 92)
    # Record 2: id=3, score=78
    store_i64(buf, 32, 3)
    store_i64(buf, 40, 78)
    # Record 3: id=4, score=98
    store_i64(buf, 48, 4)
    store_i64(buf, 56, 98)
    # Record 4: id=5, score=72
    store_i64(buf, 64, 5)
    store_i64(buf, 72, 72)

    R 5   # 레코드 수 반환
}

F store_i64(buf: i64, offset: i64, value: i64) {
    # 8바이트 정수 저장 (little-endian)
    L i:0..8 {
        byte := (value >> (i * 8)) & 255
        store_byte(buf + offset, i, byte)
    }
}

F load_i64_at(buf: i64, offset: i64) -> i64 {
    result := mut 0
    L i:0..8 {
        byte := load_byte(buf + offset, i)
        result = result | (byte << (i * 8))
    }
    result
}
```

**핵심 개념**:
- 바이트 레벨 메모리 조작으로 데이터 저장/로드
- `store_byte`/`load_byte`는 빌트인 함수
- 비트 연산 (`>>`, `<<`, `&`, `|`)으로 바이트 추출/조합

---

## Step 3: 변환 단계 (Transform) (15분)

데이터를 읽고 변환하는 파이프라인 단계를 구현합니다:

```vais
# 레코드 하나 읽기
F read_record(buf: i64, index: i64) -> Record {
    offset := index * 16   # 각 레코드 16바이트 (id 8 + score 8)
    R Record {
        id: load_i64_at(buf, offset),
        score: load_i64_at(buf, offset + 8)
    }
}

# 점수 보정: 커브 적용 (10% 보너스, 최대 100)
F apply_curve(score: i64) -> i64 {
    curved := score + score / 10
    I curved > 100 { R 100 }
    curved
}

# 필터: 최소 점수 이상만 통과
F passes_filter(score: i64, min_score: i64) -> i64 {
    I score >= min_score { R 1 }
    0
}
```

**핵심 개념**:
- 순수 함수로 각 변환 단계를 구현
- 구조체를 값으로 반환
- 조건부 반환으로 범위 제한

---

## Step 4: 집계 단계 (Aggregate) (15분)

전체 데이터를 순회하며 통계를 계산합니다:

```vais
F compute_stats(buf: i64, count: i64) -> Stats {
    total := mut 0
    max_v := mut 0
    min_v := mut 999999

    L i:0..count {
        rec := read_record(buf, i)
        score := apply_curve(rec.score)

        total = total + score

        I score > max_v {
            max_v = score
        }
        I score < min_v {
            min_v = score
        }
    }

    R Stats {
        count: count,
        total: total,
        max_val: max_v,
        min_val: min_v
    }
}
```

**핵심 개념**:
- 루프 내에서 min/max 추적 패턴
- `mut` 변수로 누적 계산
- 파이프라인 단계: read -> transform (curve) -> aggregate

---

## Step 5: 필터링 파이프라인 (15분)

조건에 맞는 레코드만 처리하는 필터를 추가합니다:

```vais
F compute_filtered_stats(buf: i64, count: i64, min_score: i64) -> Stats {
    total := mut 0
    max_v := mut 0
    min_v := mut 999999
    passed := mut 0

    L i:0..count {
        rec := read_record(buf, i)
        score := apply_curve(rec.score)

        I passes_filter(score, min_score) == 1 {
            total = total + score
            passed = passed + 1

            I score > max_v { max_v = score }
            I score < min_v { min_v = score }
        }
    }

    I passed == 0 {
        min_v = 0
    }

    R Stats {
        count: passed,
        total: total,
        max_val: max_v,
        min_val: min_v
    }
}
```

**핵심 개념**:
- 필터와 변환을 조합하는 패턴
- 빈 결과 처리 (passed == 0일 때 min 초기화)

---

## Step 6: 전체 파이프라인 조합 (10분)

```vais
F main() -> i64 {
    puts("=== Data Pipeline ===")

    # 1. 데이터 생성
    buf := malloc(256)
    count := create_dataset(buf)

    # 2. 전체 통계
    puts("\n--- All Records (with curve) ---")
    all_stats := compute_stats(buf, count)
    all_stats.print()

    # 3. 필터링 통계 (85점 이상)
    puts("\n--- Filtered (min 85, with curve) ---")
    filtered := compute_filtered_stats(buf, count, 85)
    filtered.print()

    # 4. 원본 데이터 출력
    puts("\n--- Raw Data ---")
    L i:0..count {
        rec := read_record(buf, i)
        curved := apply_curve(rec.score)
        puts("  ID {rec.id}: {rec.score} -> {curved}")
    }

    # 5. 정리
    free(buf)

    puts("\nPipeline complete.")
    0
}
```

---

## 파이프라인 아키텍처

```
[데이터 소스]     [변환]        [집계]       [출력]
 create_dataset -> read_record -> compute_  -> print()
                   apply_curve    stats
                   passes_filter
```

이 구조는 일반적인 ETL (Extract-Transform-Load) 패턴을 따릅니다:
- **Extract**: `create_dataset` + `read_record`
- **Transform**: `apply_curve` + `passes_filter`
- **Load**: `compute_stats` + `print`

---

## Step 7: 파이프 연산자 활용 (보너스)

Vais의 `|>` 연산자로 변환 체인을 표현적으로 작성할 수 있습니다:

```vais
# 단일 레코드 처리
F process_score(score: i64) -> i64 {
    score |> apply_curve |> |s| I s > 100 { 100 } E { s }
}
```

**핵심 개념**:
- `|>`는 파이프 연산자: 왼쪽 결과를 오른쪽 함수의 첫 인수로 전달
- 클로저 `|s| expr`와 조합하여 인라인 변환 가능

---

## 전체 코드

`examples/tutorial_pipeline.vais`에서 전체 코드를 확인할 수 있습니다.

```bash
vaisc run examples/tutorial_pipeline.vais
```

---

## 확장 아이디어

1. **CSV 파싱**: `std/string.vais`의 `split_char()`로 실제 CSV 파일 파싱
2. **JSON 출력**: `std/json.vais`로 결과를 JSON 형식으로 출력
3. **TOML 설정**: `std/toml.vais`로 파이프라인 설정 파일 읽기
4. **정렬**: 버블 정렬이나 `swap()` 빌트인으로 결과 정렬
5. **다단계 파이프라인**: 변환 함수를 배열로 구성하여 동적 파이프라인

---

## 배운 것 요약

| 개념 | Vais 문법 | 설명 |
|------|-----------|------|
| 데이터 모델 | `S Name { fields }` | 구조체로 레코드 정의 |
| 변환 함수 | `F transform(x) -> T` | 순수 함수 변환 |
| 필터링 | `I cond { }` | 조건부 처리 |
| 집계 | `mut` + `L` | 누적 계산 |
| 파이프 | `expr \|> func` | 체이닝 변환 |
| 메모리 | `malloc`/`free` | 버퍼 관리 |
| 비트 연산 | `>>`, `<<`, `&`, `\|` | 바이트 조작 |

이전 튜토리얼: [HTTP Server 만들기](./http-server.md)

---

## 다음 단계

- [학습 경로](../learning-path.md) - 체계적 학습 가이드
- [표준 라이브러리](../stdlib/stdlib.md) - 사용 가능한 모든 모듈
- [Cookbook](../guides/cookbook.md) - 실전 레시피 모음
