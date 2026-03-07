# Tutorial: CLI Framework 만들기

이 튜토리얼에서는 Vais로 재사용 가능한 CLI (Command-Line Interface) 프레임워크를 만듭니다. 서브커맨드, 옵션 파싱, 헬프 출력을 지원하는 미니 프레임워크를 구현합니다.

## 최종 결과

```bash
$ ./mycli help
mycli v1.0 — Vais CLI Framework Demo

Commands:
  greet <name>     Say hello
  count <n>        Count from 1 to n
  fib <n>          Compute Fibonacci
  help             Show this help

$ ./mycli greet Vais
Hello, Vais!

$ ./mycli count 5
1 2 3 4 5

$ ./mycli fib 10
fib(10) = 55
```

## 사전 준비

- Vais 설치 완료
- [CLI Tool 튜토리얼](./cli-tool.md) 완료 권장

---

## Step 1: 인자 파싱 기반 (10분)

CLI 프로그램의 핵심은 명령행 인자를 읽는 것입니다. Vais에서는 C 런타임의 `argc`/`argv`를 통해 접근합니다:

```vais
# C 런타임 함수
N "C" {
    F __get_argc() -> i64
    F __get_argv(index: i64) -> str
    F __strlen(s: str) -> i64
    F __strcmp(a: str, b: str) -> i64
    F __atoi(s: str) -> i64
}

F main() -> i64 {
    argc := __get_argc()

    I argc < 2 {
        puts("Usage: mycli <command> [args...]")
        puts("Try: mycli help")
        R 1
    }

    cmd := __get_argv(1)
    puts("Command: {cmd}")
    0
}
```

**핵심 포인트**:
- `__get_argc()`는 인자 개수, `__get_argv(i)`는 i번째 인자를 반환합니다
- `__strcmp`로 문자열 비교 (0이면 동일)
- `__atoi`로 문자열→정수 변환

---

## Step 2: 커맨드 디스패처 (15분)

서브커맨드별로 핸들러 함수를 분기하는 디스패처를 만듭니다:

```vais
# 서브커맨드 핸들러들
F cmd_help() -> i64 {
    puts("mycli v1.0 — Vais CLI Framework Demo")
    puts("")
    puts("Commands:")
    puts("  greet <name>     Say hello")
    puts("  count <n>        Count from 1 to n")
    puts("  fib <n>          Compute Fibonacci")
    puts("  help             Show this help")
    0
}

F cmd_greet() -> i64 {
    argc := __get_argc()
    I argc < 3 {
        puts("Error: greet requires a name")
        puts("Usage: mycli greet <name>")
        R 1
    }
    name := __get_argv(2)
    puts("Hello, {name}!")
    0
}

F cmd_count() -> i64 {
    argc := __get_argc()
    I argc < 3 {
        puts("Error: count requires a number")
        R 1
    }
    n := __atoi(__get_argv(2))
    I n <= 0 {
        puts("Error: n must be positive")
        R 1
    }

    L i:1..n+1 {
        putchar(i / 10 + 48)
        putchar(i % 10 + 48)
        putchar(32)   # space
    }
    putchar(10)  # newline
    0
}

F cmd_fib() -> i64 {
    argc := __get_argc()
    I argc < 3 {
        puts("Error: fib requires a number")
        R 1
    }
    n := __atoi(__get_argv(2))
    result := fib(n)
    puts("fib({n}) = {result}")
    0
}

# 피보나치 (자재귀)
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)

# 디스패처
F dispatch(cmd: str) -> i64 {
    I __strcmp(cmd, "help") == 0 { R cmd_help() }
    I __strcmp(cmd, "greet") == 0 { R cmd_greet() }
    I __strcmp(cmd, "count") == 0 { R cmd_count() }
    I __strcmp(cmd, "fib") == 0 { R cmd_fib() }

    puts("Unknown command: {cmd}")
    puts("Try: mycli help")
    1
}

F main() -> i64 {
    argc := __get_argc()
    I argc < 2 {
        R cmd_help()
    }

    cmd := __get_argv(1)
    dispatch(cmd)
}
```

**핵심 포인트**:
- 각 커맨드는 독립 함수로 분리합니다
- `dispatch`가 문자열 비교로 적절한 핸들러를 호출합니다
- 인자 검증을 각 핸들러에서 수행합니다 (argc 체크)
- `R` (return)으로 조기 반환하여 에러 처리합니다

---

## Step 3: 옵션 파싱 (15분)

`--verbose`, `--output=file` 같은 옵션을 파싱하는 기능을 추가합니다:

```vais
# 전역 옵션 플래그
G verbose: i64 = mut 0
G output_file: str = ""

# 문자열 접두사 비교
F starts_with(s: str, prefix: str) -> i64 {
    s_len := __strlen(s)
    p_len := __strlen(prefix)
    I p_len > s_len { R 0 }

    i := mut 0
    L i < p_len {
        sc := load_byte(s as i64 + i)
        pc := load_byte(prefix as i64 + i)
        I sc != pc { R 0 }
        i = i + 1
    }
    1
}

# 옵션 파싱 — 옵션이 아닌 첫 인자의 인덱스 반환
F parse_options() -> i64 {
    argc := __get_argc()
    i := mut 1    # argv[0]은 프로그램 이름

    L i < argc {
        arg := __get_argv(i)

        I __strcmp(arg, "--verbose") == 0 | __strcmp(arg, "-v") == 0 {
            verbose = 1
            i = i + 1
            C
        }

        I starts_with(arg, "--output=") == 1 {
            # "=" 뒤의 값 추출
            output_file = arg as i64 + 9 as str
            i = i + 1
            C
        }

        I starts_with(arg, "-") == 1 {
            puts("Unknown option: {arg}")
            i = i + 1
            C
        }

        # 옵션이 아닌 인자 발견 → 커맨드 시작
        B
    }

    i   # 커맨드 인자의 시작 인덱스
}
```

**핵심 포인트**:
- 전역 플래그 `G verbose: i64 = mut 0`로 옵션 상태를 저장합니다
- `starts_with` 헬퍼로 접두사 매칭합니다
- `C` (continue)로 다음 인자를 처리합니다
- `B` (break)로 옵션 파싱을 종료하고 커맨드 처리로 넘어갑니다

---

## Step 4: 에러 처리와 종료 코드 (10분)

Unix 관례에 따라 적절한 종료 코드를 반환합니다:

```vais
C EXIT_OK: i64 = 0
C EXIT_USAGE: i64 = 1
C EXIT_ERROR: i64 = 2

F error(msg: str) -> i64 {
    puts("Error: {msg}")
    EXIT_ERROR
}

F usage_error(msg: str) -> i64 {
    puts("Error: {msg}")
    puts("")
    cmd_help()
    EXIT_USAGE
}

# 개선된 디스패처
F dispatch_v2(cmd: str) -> i64 {
    I verbose == 1 {
        puts("[verbose] Dispatching command: {cmd}")
    }

    I __strcmp(cmd, "help") == 0 { R cmd_help() }
    I __strcmp(cmd, "greet") == 0 { R cmd_greet() }
    I __strcmp(cmd, "count") == 0 { R cmd_count() }
    I __strcmp(cmd, "fib") == 0 { R cmd_fib() }

    usage_error("unknown command")
}
```

---

## Step 5: 전체 프로그램 (10분)

```vais
F main() -> i64 {
    # 1) 옵션 파싱
    cmd_start := parse_options()

    argc := __get_argc()
    I cmd_start >= argc {
        R cmd_help()
    }

    # 2) 커맨드 실행
    cmd := __get_argv(cmd_start)

    I verbose == 1 {
        puts("[verbose] Options parsed")
        I __strlen(output_file) > 0 {
            puts("[verbose] Output: {output_file}")
        }
    }

    dispatch_v2(cmd)
}
```

## 빌드 및 실행

```bash
# 빌드
vaisc --emit-ir examples/tutorial_cli_framework.vais
clang -o mycli tutorial_cli_framework.ll

# 테스트
./mycli help
./mycli greet World
./mycli --verbose fib 10
./mycli --output=result.txt count 5
```

---

## 확장 아이디어

### 1. 플러그인 커맨드 등록

```vais
# 함수 포인터 배열로 커맨드 등록
S Command {
    name: str,
    description: str,
    handler: i64    # 함수 포인터
}
```

### 2. 자동 완성 힌트

```vais
F suggest_command(partial: str) -> i64 {
    # "co" → "count"
    # "fi" → "fib"
    # 접두사 매칭으로 후보 출력
    0
}
```

### 3. 환경 변수 지원

```vais
N "C" {
    F getenv(name: str) -> str
}

F get_config_dir() -> str {
    dir := getenv("MYCLI_CONFIG")
    I __strlen(dir) == 0 {
        R "/etc/mycli"
    }
    dir
}
```

---

## 핵심 개념 정리

| 개념 | Vais 문법 | 설명 |
|------|-----------|------|
| 외부 함수 | `N "C" { F name(...) }` | argc/argv 접근 |
| 전역 변수 | `G verbose: i64 = mut 0` | 옵션 플래그 |
| 상수 | `C EXIT_OK: i64 = 0` | 종료 코드 |
| 문자열 보간 | `"Error: {msg}"` | 에러 메시지 |
| 자재귀 | `F fib(n) = n<2 ? n : @(n-1)+@(n-2)` | 재귀 함수 |
| 조기 반환 | `R value` | 에러 시 즉시 반환 |
| 루프 제어 | `B` (break), `C` (continue) | 인자 파싱 루프 |

## 다음 단계

- [WebSocket Chat 튜토리얼](./websocket-chat.md) — 실시간 서버 만들기
- [Data Pipeline 튜토리얼](./data-pipeline.md) — 데이터 처리 파이프라인
- [Cookbook](../guides/cookbook.md) — 실전 레시피 모음
- [Testing Guide](../guides/testing.md) — 테스트 작성법
