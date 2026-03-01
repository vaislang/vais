# Tutorial: CLI Tool 만들기

이 튜토리얼에서는 Vais로 텍스트 파일의 줄 수, 단어 수, 바이트 수를 세는 CLI 도구 `vwc` (Vais Word Count)를 만듭니다. Unix의 `wc` 명령어의 간단한 버전입니다.

## 최종 결과

```bash
$ vaisc run examples/tutorial_wc.vais
=== Vais Word Count (vwc) ===
Lines: 15
Words: 42
Bytes: 256
```

## 사전 준비

- Vais 설치 완료 (`vaisc --version`으로 확인)
- 텍스트 에디터 (VSCode + Vais 확장 추천)

---

## Step 1: 프로젝트 뼈대 (5분)

`tutorial_wc.vais` 파일을 만들고 기본 구조를 작성합니다:

```vais
# vwc — Vais Word Count
# 텍스트 파일의 줄/단어/바이트를 센다

F main() -> i64 {
    puts("=== Vais Word Count (vwc) ===")
    0
}
```

컴파일 및 실행:

```bash
vaisc run tutorial_wc.vais
```

**핵심 개념**: Vais에서 `F`는 함수 선언 키워드입니다. `main()` 함수는 프로그램 진입점이며, 반환 타입을 생략하면 암시적으로 `i64`를 반환합니다.

---

## Step 2: 결과를 담을 구조체 (10분)

카운트 결과를 저장할 구조체를 정의합니다:

```vais
# 카운트 결과를 저장하는 구조체
S WcResult {
    lines: i64
    words: i64
    bytes: i64
}

# 메서드 정의
X WcResult {
    F new() -> WcResult {
        R WcResult { lines: 0, words: 0, bytes: 0 }
    }

    F print(&self) {
        puts("Lines: ~{self.lines}")
        puts("Words: ~{self.words}")
        puts("Bytes: ~{self.bytes}")
    }
}
```

**핵심 개념**:
- `S`는 구조체(struct) 선언 키워드
- `X`는 impl 블록 (메서드 정의)
- `&self`는 해당 구조체의 불변 참조
- `~{expr}`는 문자열 보간 (string interpolation)
- `R`은 return 키워드

---

## Step 3: 바이트 카운팅 함수 (10분)

입력 텍스트의 바이트 수를 세는 함수를 작성합니다:

```vais
N "C" {
    F strlen(s: str) -> i64
}

F count_bytes(text: str) -> i64 {
    R strlen(text)
}
```

**핵심 개념**:
- `N "C"`는 extern 블록으로, C 표준 라이브러리 함수를 호출합니다
- Vais는 LLVM 백엔드를 사용하므로 C 함수를 직접 호출할 수 있습니다

---

## Step 4: 줄 수 세기 (10분)

문자열을 순회하며 줄바꿈 문자(`\n`, ASCII 10)를 세는 함수입니다:

```vais
F count_lines(text: str, len: i64) -> i64 {
    count := mut 0
    L i:0..len {
        byte := load_byte(text, i)
        I byte == 10 {
            count = count + 1
        }
    }
    # 마지막 줄이 줄바꿈으로 끝나지 않는 경우
    I len > 0 {
        last := load_byte(text, len - 1)
        I last != 10 {
            count = count + 1
        }
    }
    count
}
```

**핵심 개념**:
- `L i:0..len`은 범위 루프 (0부터 len-1까지)
- `load_byte(ptr, offset)`는 메모리에서 바이트를 읽는 빌트인 함수
- `I`는 if, 10은 줄바꿈 문자의 ASCII 코드
- `mut`는 가변 변수 선언

---

## Step 5: 단어 수 세기 (15분)

공백, 탭, 줄바꿈을 구분자로 사용하여 단어를 셉니다:

```vais
F is_whitespace(b: i64) -> i64 {
    # 공백(32), 탭(9), 줄바꿈(10), 캐리지 리턴(13)
    I b == 32 { R 1 }
    I b == 9 { R 1 }
    I b == 10 { R 1 }
    I b == 13 { R 1 }
    0
}

F count_words(text: str, len: i64) -> i64 {
    count := mut 0
    in_word := mut 0

    L i:0..len {
        byte := load_byte(text, i)
        I is_whitespace(byte) == 1 {
            I in_word == 1 {
                count = count + 1
                in_word = 0
            }
        } E {
            in_word = 1
        }
    }

    # 마지막 단어 처리
    I in_word == 1 {
        count = count + 1
    }

    count
}
```

**핵심 개념**:
- `E`는 else 키워드
- 상태 머신 패턴: `in_word` 플래그로 단어 경계를 추적
- Vais의 `bool`은 `0`/`1` 정수로 처리됩니다 (i64 기반)

---

## Step 6: 전체 조합 (10분)

모든 함수를 조합하여 완성합니다:

```vais
F analyze(text: str) -> WcResult {
    len := strlen(text)
    R WcResult {
        lines: count_lines(text, len),
        words: count_words(text, len),
        bytes: len
    }
}

F main() -> i64 {
    puts("=== Vais Word Count (vwc) ===")

    # 테스트 텍스트
    text := "Hello Vais World\nThis is line two\nAnd line three\n"
    result := analyze(text)
    result.print()

    0
}
```

---

## Step 7: 복수 텍스트 처리 (10분)

여러 텍스트를 처리하고 합계를 구하는 기능을 추가합니다:

```vais
F add_results(a: WcResult, b: WcResult) -> WcResult {
    R WcResult {
        lines: a.lines + b.lines,
        words: a.words + b.words,
        bytes: a.bytes + b.bytes
    }
}

F main() -> i64 {
    puts("=== Vais Word Count (vwc) ===")

    text1 := "Hello Vais World\nThis is a test\n"
    text2 := "Another paragraph\nWith more words here\n"

    r1 := analyze(text1)
    r2 := analyze(text2)
    total := add_results(r1, r2)

    puts("\n--- File 1 ---")
    r1.print()

    puts("\n--- File 2 ---")
    r2.print()

    puts("\n--- Total ---")
    total.print()

    0
}
```

---

## 전체 코드

`examples/tutorial_wc.vais`에서 전체 코드를 확인할 수 있습니다.

```bash
vaisc run examples/tutorial_wc.vais
```

---

## 확장 아이디어

이 프로젝트를 더 발전시킬 수 있는 아이디어:

1. **파일 읽기**: `std/io.vais`의 `read_file()` 함수로 실제 파일 처리
2. **명령행 인수**: `std/args.vais`로 파일 경로를 인수로 받기
3. **포맷 출력**: 칼럼 정렬로 `wc`와 유사한 출력 형식
4. **성능 측정**: `std/time.vais`로 처리 시간 측정

---

## 배운 것 요약

| 개념 | Vais 문법 | 설명 |
|------|-----------|------|
| 함수 | `F name(params) -> T { body }` | 단일 문자 키워드 |
| 구조체 | `S Name { fields }` | 데이터 타입 정의 |
| 메서드 | `X Name { F method(&self) { } }` | impl 블록 |
| 변수 | `x := 42`, `x := mut 0` | 추론 + 가변성 |
| 루프 | `L i:0..n { }` | 범위 루프 |
| 조건 | `I cond { } E { }` | if/else |
| FFI | `N "C" { F func() }` | C 함수 호출 |
| 보간 | `"~{expr}"` | 문자열 내 표현식 |

다음 튜토리얼: [HTTP Server 만들기](./http-server.md)
