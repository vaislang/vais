# Vais 전체 지도

> 2026-08-20 기준 · v1.5.0 태그 + 3커밋(main `a2bcab21`) · 처음 읽는 사람을 위한 한 장짜리 안내서

**세 문장 요약.** Vais는 정수·문자열 중심의 정적 타입 언어이고, 그 컴파일러는 Vais 자신으로 작성되어 스스로를 컴파일합니다(자기호스팅). 이 언어로 검색엔진(vaisdb), Lisp 인터프리터(vaislisp), jq 호환 쿼리 도구(vaisjq), 수식 계산기(vaiscalc)를 포함한 설치형 도구 17종을 만들었고, 전부 네이티브 바이너리로 배포됩니다. 모든 기능은 "게이트"라는 자동 검증에 잠겨 있어서, 문서에 적힌 것은 곧 테스트가 보증하는 것입니다.

---

## 1. 숫자로 보는 현재

| 항목 | 값 |
|---|---|
| 최신 릴리스 | v1.5.0 (2026-08-20) — 태그 8개 누적 (v0.3.2 → v1.5.0) |
| 자기호스팅 컴파일러 | `compiler/self/fixpoint_full.vais` **23,539줄**(Vais) |
| 네이티브 드라이버 | `tools/vaisc_native.c` **38,754줄**(C) |
| 예제 코퍼스 | `examples/` 423개 (`# expect: 42` 마커로 값 잠금) |
| 엔진 일치(parity) | 418개 예제가 두 엔진에서 동일 결과 |
| 설치형 도구 | 17종 (vaisbox 디스패처 + 로스터 applet 16) |
| 워크플로 게이트 | 350여 케이스 (도구 동작·에러 exit·회귀) |
| front 계약 픽스처 | 340여 건 (무엇을 받아들이고 무엇을 거부하는지) |

---

## 2. 전체 구조 한 장

```
  hello.vais ──▶ vaisc (C 드라이버) ──▶ front 검사 (vais-check)
                                          │  미검증 문법은 여기서 "크게" 거부
                                          ▼
                          ┌───────────────┴───────────────┐
                          ▼                               ▼
               full 엔진 (기본)                    direct 엔진 (--engine direct)
        self-host core = Vais로 쓴 컴파일러           드라이버 안의 C 직접 방출기
        (vaisc_core.ll, LLVM IR)                   (같은 결과를 내야 함 = parity)
                          │                               │
                          └──────────▶ clang ◀────────────┘
                                          │
                                          ▼
                                   네이티브 바이너리

  패키지: <dir>/vais.toml + src/main.vais ──▶ vaisc package ──▶ dist/bin/<binary>
```

- **왜 엔진이 둘인가** — full 엔진은 "Vais가 Vais를 컴파일한다"는 증명이고, direct 엔진은 그 증명을 검산하는 독립 구현입니다. 418개 예제가 두 엔진에서 같은 값을 내야 합니다.
- **자기호스팅 루프** — `fixpoint_full.vais`를 설치된 core로 컴파일해 새 core를 만들고, 그 core로 다시 자신을 컴파일했을 때 바이트 동일(gen2 == gen3)하면 "수렴"입니다. 컴파일러를 고칠 때마다 이 루프를 돌립니다.
- **front의 역할** — Vais는 "검증된 표면"만 허용합니다. 아직 게이트로 보증되지 않은 문법은 front가 이유와 대안을 적어 거부합니다. 조용히 틀린 코드를 내는 것이 최악이라는 원칙입니다.

---

## 3. 언어 기능

### 3.1 한 파일로 보는 핵심 문법

```vais
# expect: 42
struct Point { x: Int, y: Int }

fn dist1(p: Point) -> Int { return p.x + p.y }        # 한 줄 본문도 됩니다

fn sum_to(n: Int) -> Int {
    if n <= 0 { return 0 }
    return n + @(n - 1)                               # @ = 자기 재귀
}

fn split_pair(v: Int) -> (Int, Int) {                # 튜플 반환
    return (v / 10, v - (v / 10) * 10)
}

fn main() -> Int {
    let p = Point { x: 20, y: 2 }
    let mut acc = dist1(p)                            # 22
    let xs: List<Int> = [-5, 3, 9]                    # 음수 리터럴 OK
    for v in xs {
        acc += v                                      # 29
    }
    let (tens, ones) = split_pair(42)                 # 4, 2
    let m: Map<Str,Int> = {}
    m.insert("k", tens + ones)                        # 6
    let label = f"sum={acc}"                          # f-string (명시적 보간만)
    if label.len() == 6 and m.get("k", 0) == 6 {
        return acc + sum_to(3) + 7                    # 29 + 6 + 7 = 42
    }
    0
}
```

### 3.2 타입

| 타입 | 설명 | 한계 |
|---|---|---|
| `Int` | 64비트 정수 (유일한 수 타입) | **부동소수점 없음** |
| `Bool`, `Char`, `Str` | 불리언 / 바이트 문자 / 바이트 문자열 | `s[i]`는 바이트 값(Int) |
| `List<T>` | 고정 용량 리스트 (`push/pop/sort/index_of/...`) | **4095 슬롯**, 초과 시 LOUD 트랩 |
| `Map<K,V>` | Str/Int 키 맵 (`insert/get/contains/remove/clear`) | **4096 엔트리** |
| `struct` | 값 구조체, 리스트 원소·필드 체인 가능 | 메서드 없음(함수로) |
| `enum` + `match` | 제한된 슬라이스 | 문장형 match 위주 |
| `Option<Int>`, `Result<T,E>` | `Some/None`, `Ok/Err`, `?` 전파 | `Result<Str,Int>`, `Result<Str,Str>` 등 검증된 조합만 |
| `(Int, Int)` 튜플 | 반환·구조분해 | 구조분해 RHS는 명명 호출만 |

### 3.3 꼭 알아야 할 규칙

- **세미콜론 없음**, `#` 주석, `\n \t \r \" \\` 이스케이프, `f"{expr}"` 보간(암묵 보간 없음).
- **함수 파라미터 최대 16개** — 초과는 front가 거부합니다(컴파일러 모델의 명시적 상한).
- `let` 불변, `let mut` 가변. `+= -= *= /= %=`는 단순 장소(로컬·리스트 인덱스·필드)에만.
- 단항 마이너스(`-5`, `-x`, `-f()`)는 두 엔진 모두 지원 — 과거엔 `0 - n` 관용만 안전했습니다.
- 클로저/람다는 **호스트 언어에 없습니다**(vaislisp가 인터프리터 차원에서 제공).
- 제네릭은 identity 헬퍼 슬라이스(`fn apply<T>(x: T) -> T { return x }`, 한 줄 형태 필수)만.

### 3.4 호스트 API (표준 라이브러리에 해당)

| 분류 | 함수 |
|---|---|
| 파일 | `fs_read_text` `fs_write_text` `fs_append_text` `fs_exists` `fs_is_dir` `fs_list_files` `fs_list_dirs` `fs_mkdirs` `fs_remove` `fs_rename` `fs_mtime` `fs_temp_dir` `fs_cwd` |
| 경로 | `path_join` `path_basename` `path_dirname` |
| 프로세스 | `proc_argc` `proc_arg` `proc_self` `proc_run` `proc_run_env` `proc_capture` `proc_capture_stdout/stderr/to` |
| 입출력 | `print` `putchar` `stdout_write` `stderr_write` `stdin_read_line` `stdin_read_all` |
| 환경·시간 | `env_get` `time_millis` `time_sleep_millis` |
| 문자열 | `str_concat` `str_slice(start,len)` `str_index_of` `str_contains` `str_starts_with` `str_ends_with` `str_replace` `str_split_into` `str_split_lines_into` `str_split_ws_into` `str_join` `str_trim` `str_upper` `str_lower` `str_cmp` `str_byte` `parse_int` `parse_uint` + `str_builder_new/push/append/finish` |
| 비트 | `bitand` `bitnot` `shl` |

정본: `std/PRELUDE.md`(상태 표), `docs/reference/LANGUAGE.md`(문법 레퍼런스).

---

## 4. 컴파일러와 도구 체인

### 4.1 `vaisc` 명령

| 명령 | 하는 일 |
|---|---|
| `scripts/vaisc run <file.vais \| pkg-dir> [--engine full\|direct]` | 컴파일 후 즉시 실행 (종료 코드 = `main` 반환값) |
| `scripts/vaisc build <src> -o out` | 네이티브 바이너리 생성 |
| `scripts/vaisc emit-ir <src> -o out.ll` | LLVM IR 방출 (core 재생성에도 사용) |
| `scripts/vaisc package <pkg-dir> -o dist` | `vais.toml` 기반 설치형 배포본 (`dist/bin/<binary>`) |
| `scripts/vaisc doctor` / `--version` | 환경 진단 / 버전(현재 1.5.0) |
| `scripts/vais-check <file>` | front 계약 검사만 |

### 4.2 게이트 래더 (작은 것부터 넓은 것으로)

| 스크립트 | 보증하는 것 |
|---|---|
| `vaisfmt-check.sh` | 소스 공백 위생 |
| `test-vaisc-front.sh` | front 계약: 수용/거부 픽스처 340여 건 |
| `test-vaisc-direct.sh` | direct 엔진 기능·거부·트랩 |
| `test-fixpoint-full.sh` | full 엔진 end-to-end |
| `test.sh` | 예제 423개 값 잠금 (pass=418, 나머지는 패키지/특수) |
| `test-vaisc-parity.sh` | 두 엔진 결과 일치 418 |
| `test-fixpoint-full-self.sh` | 컴파일러가 자기 자신을 컴파일 |
| `test-vaisdb-workflow.sh` | 도구 17종 동작 350여 케이스 |
| `test-vaisdb-relevance.sh` | 검색 품질 바닥선(MRR@100 ≥ 70) |
| `test-release-gates.sh` | 위 전부 + 설치·웹사이트 빌드 |

규율: 컴파일러를 건드리면 전체 래더, 도구만 건드리면 워크플로 게이트부터. 게이트가 초록이 아니면 커밋하지 않습니다.

---

## 5. 설치형 도구 17종

모두 `scripts/vaisc package examples/<pkg> -o dist`로 설치하고, `vaisbox`가 busybox처럼 한 바이너리로 전부 디스패치합니다(`vaisbox list`, `vaisbox vaisjq ...`).

| 도구 | 한 줄 설명 | 예시 |
|---|---|---|
| **vaisdb** | 디스크 기반 전문 검색엔진 (64샤드, 원자적 ingest, 랭킹) | `vaisdb search idx "fixpoint gate" 5 -all` |
| **vaisgrep** | 패턴 검색 (`-c` 카운트, stdin `-`) | `vaisgrep cache file.txt` |
| **vaismake** | 의존 그래프 태스크 러너 (`gates.tasks`가 래더 정의) | `vaismake gates.tasks release` |
| **vaisfmt** | 소스 포매터/검사기 | `vaisfmt --check src/` |
| **vaisbench** | 반복 실행 벤치마크 | `vaisbench 5 ./cmd` |
| **vaisdiff** | 줄 단위 diff | `vaisdiff a.txt b.txt` |
| **vaiswc** | 줄·단어·바이트 카운트 | `vaiswc file.txt` |
| **vaissort** | 줄 정렬 (`-u` 중복 제거, `-r` 역순) | `vaissort -u -` |
| **vaisenv** | 환경 변수 출력 | `vaisenv HOME` |
| **vaistee** | stdin을 stdout+파일로 복제 (`-a` 추가) | `cmd \| vaistee log.txt` |
| **vaiscut** | 구분자 필드 추출 | `vaiscut -f 2 -d ,` |
| **vaisfind** | 이름 패턴 재귀 파일 찾기 | `vaisfind notes ./tree` |
| **vaisfreq** | 단어 빈도 (`-n` 상위 N) | `vaisfreq -n 10 -` |
| **vaislisp** | Lisp 인터프리터 (REPL/파일/셀프테스트) | `vaislisp prog.lisp` |
| **vaisjq** | JSON 파서 + jq 부분집합 쿼리 | `vaisjq '.users[] \| .name' data.json` |
| **vaiscalc** | 우선순위 수식 계산기 | `vaiscalc "2 * (3 + 4) * 3"` |
| **vaisbox** | 위 16종 멀티콜 디스패처 | `vaisbox vaiscalc "6 * 7"` |

공통 규약: 인자 없이 실행하면 결정적 셀프테스트(성공 = exit 42), 사용법 오류 exit 2, 입력·타입 오류는 stderr 메시지 + exit 3.

---

## 6. 프로그램형 도구 깊이 보기

### 6.1 vaisdb — 검색엔진

| 명령 | 뜻 |
|---|---|
| `ingest <idx> <id> <file>` / `ingest-stdin` / `ingest-dir <idx> <dir> [-r]` | 문서 추가 (디렉토리 재귀 `-r`) |
| `reindex <idx> <dir> [-r]` | 변경분만 갱신(추가/수정/삭제 감지) |
| `search <idx> <query> [k] [-all]` | 검색. `-all`은 모든 텀 보유 문서만 + 희귀 텀 가중 랭킹 |
| `rank` / `top [k] [-min bytes]` / `similar <id> [k]` / `phrase` | 랭킹 리포트 / 큰 문서 / 유사 문서 / 구절 |
| `why <idx> <query> <id>` | 왜 이 문서가 점수를 받았는지 설명 |
| `msearch <query> <k> <idx...>` | 여러 인덱스 동시 검색 |
| `docs` / `remove` / `stats` / `query` / `report` | 관리·조회 |

설계 키워드: 64개 텀-해시 샤드(4096 창 한계 제거), temp-then-rename 원자적 쓰기(크래시 안전), last-wins 중복 처리, MRR 하네스로 품질 바닥선 고정. 전체 레포 553문서 웜 검색 0.33초.

### 6.2 vaislisp — Lisp 인터프리터

값은 전부 태그드 정수 한 종류로 표현합니다: 숫자(±1e15), 문자열(풀 인덱스), cons 셀(병렬 힙), 함수(테이블), nil. 풀/힙은 8190 엔트리(2-way 파티션 + 내용 dedup — 핫루프에서도 고갈되지 않음).

| 분류 | 형태 |
|---|---|
| 정의·대입 | `(define x e)` `(set x e)` `(defun f (a b) body)` `(lambda (a) body)` — 일급 값, 동적 스코프 |
| 제어 | `(if c a [b])` `(cond (t e...) ... (else e...))` `(and ...)` `(or ...)` `(while c body...)` `(begin e...)` `(let ((n e) ...) body...)` |
| 산술·비교 | 가변 `(+ 1 2 3)` `(- x)` `(* ...)` `(/ ...)`, `(mod a b)`, 이항 `< > =`(문자열 `=`도) |
| 리스트 | `cons car cdr list null? nil list-ref length append reverse` |
| 문자열 | `"lit"` `str-len str-cat str-ref str-byte str->num num->str` |
| 데이터 | `(quote e)` / `'e` — 심볼은 문자열로, 스팬은 리스트로 |
| 출력 | `(print e)` |

```lisp
(defun fib (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))
(print (fib 20))                              ; 6765
(define names '("ann" "bo"))
(print (length (append names '("cy"))))       ; 3
```

모드: `vaislisp`(셀프테스트 42) · `vaislisp file.lisp`(exit = 마지막 값) · `vaislisp repl`(stdin 한 줄씩). `programs/`에 fizzbuzz·vowels·assoc·hof·sort·words 코퍼스가 게이트로 잠겨 있습니다.

### 6.3 vaisjq — JSON 쿼리

정수 JSON(소수·지수 없음, `\uXXXX` 없음)을 파싱해 jq 문법의 실용 부분집합을 실행합니다. 출력은 기본 pretty(2칸), `-c`로 compact. 결측 필드는 `null`로 이어지고(jq 관용), 타입 위반·파스 오류는 exit 3.

| 분류 | 필터 |
|---|---|
| 경로 | `.` `.name` `.["any key"]` `.[N]` `.[-N]` `.[a:b]` `.[]` — 체인 가능 |
| 파이프 | `f \| g` (경로끼리는 결합), 마지막에만 `\| length` `\| keys` `\| keys_unsorted` |
| 선택 | `select(<cond> [and\|or <cond>]...)` — `<path> op <리터럴 또는 경로>`(`== != < > <= >=`, 깊은 동등), bare 경로, `has("k")`/`has(N)` |
| 변환 | `map(<path>)` `{key: <path or "템플릿">, "quoted": .p, shorthand}` `"text \(.path)"` `tostring` `tonumber` `<path> op <path or 정수>`(`+ - * /`) |
| 집계 | `add` `min` `max` `any` `all` `not` `join("sep")` |
| 재구성 | `sort` `unique` `reverse` `group_by(<path>)` `to_entries` `from_entries` `first` `last` |
| 탐색·편집 | `recurse` `paths` `leaf_paths` `del(<path>)` |

```bash
vaisjq '.users[] | select(.age > 26 and has("name")) | {who: .name, line: "\(.name)/\(.age)"}' data.json
vaisjq '.items | map(.price) | add | . / 3'          # 평균
vaisjq -c '.logs | group_by(.lvl) | .[] | first'      # 레벨별 첫 항목
vaisjq '. | recurse | select(. == 9)'                 # 깊이 무관 탐색
vaisjq -c '. | del(.meta.tmp) | to_entries'           # 편집 후 엔트리화
```

내부 설계 한 줄: 배열과 객체가 `(key, val, next)` 공용 셀 힙을 쓰고, 모든 필터는 세그먼트 리스트로 컴파일되어 16-파라미터 평가기 하나가 실행합니다(self-test 170케이스).

### 6.4 vaiscalc — 수식 계산기

`+ - * / %`, 괄호, 단항 마이너스, 자유 공백. `vaiscalc "-(10 - 52) + 100 % 29 - 13"` → `42`. 0 나눗셈·불균형 괄호·트레일링 문자는 exit 3.

---

## 7. 할 수 있는 것 / 없는 것

**지금 바로 만들 수 있는 것** — CLI 텍스트 도구, 파서·인터프리터·DSL(이미 3종 실증), 디스크 기반 데이터 도구, 빌드·개발 도구, 정수 기반 시뮬레이션.

**막혀 있는 것** (수요가 생기면 호스트 API 승격으로 여는 것들)

| 없는 것 | 의미 |
|---|---|
| 부동소수점 | 과학 계산·그래픽 불가 (고정소수점으로 우회 가능) |
| 네트워크 | 서버·클라이언트 불가 — 가장 큰 단일 확장점 |
| 스레드 | 병렬은 `proc_run`으로 프로세스 단위만 |
| 호스트 클로저 | 고차 함수는 vaislisp 안에서만 |
| 가변 길이 컨테이너 | List 4095 / Map 4096 — 샤드 파티션 패턴으로 대규모 처리는 실증됨(26만 어휘) |
| raw 터미널 입력 | 대화형 게임은 턴제(stdin 줄 단위)만 |

---

## 8. 어떻게 믿는가

- **값 잠금**: 모든 예제 첫 줄 `# expect: 42` — 실행 결과가 곧 테스트.
- **두 엔진 일치**: 같은 소스가 full/direct에서 다른 값을 내면 parity 게이트가 잡습니다(단항 마이너스 silent-0 같은 발산이 이렇게 발견·수정됨).
- **LOUD 원칙**: 조용한 오동작 금지. 미검증 문법은 front 거부, 런타임 한계는 메시지와 함께 트랩.
- **근본 수정만**: 도구 개발 중 만난 컴파일러 버그는 우회하지 않고 컴파일러를 고칩니다(파라미터 10→16 확장, 단항 마이너스, 한줄 함수 본문 등 — 각각 예제로 잠금).
- **도그푸딩 환류**: 도구를 만들며 생긴 마찰이 다음 언어 표면이 됩니다(예: vaislisp 코퍼스의 중첩 if → `cond`/`and`/`or`).

---

## 9. 5분 투어

```bash
scripts/vaisc --version                                    # vaisc 1.5.0
scripts/vaisc run examples/e395_one_line_fn_bodies.vais; echo $?   # 42
bash scripts/test-vaisc-front.sh                           # 가장 빠른 게이트

scripts/vaisc package examples/e396_vaisjq_package -o /tmp/jq
printf '{"xs":[10,20,30]}' | /tmp/jq/bin/vaisjq '.xs | add | . / 3'   # 20

scripts/vaisc package examples/e393_vaislisp_package -o /tmp/lisp
printf '(+ 1 2 3 36)\n' | /tmp/lisp/bin/vaislisp repl                 # = 42

scripts/vaisc package examples/e355_vaisbox_package -o /tmp/box
/tmp/box/bin/vaisbox list | wc -l                                     # 16
```

---

## 10. 용어 사전

| 용어 | 뜻 |
|---|---|
| **full 엔진** | Vais로 쓴 자기호스팅 컴파일러(core)로 컴파일하는 기본 경로 |
| **direct 엔진** | 드라이버(C) 안의 독립 C 방출기 — full의 검산용 |
| **core** | `compiler/self/vaisc_core.ll` — 컴파일러 자신을 LLVM IR로 고정한 산물 |
| **front / vais-check** | 컴파일 전 계약 검사기. 검증되지 않은 형태를 거부 |
| **parity** | 두 엔진이 같은 예제에 같은 값을 내는지 (현재 418) |
| **게이트** | 자동 검증 스크립트. 초록이 아니면 머지 금지 |
| **LOUD** | 오류를 조용히 삼키지 않고 메시지+비정상 exit로 드러내는 원칙 |
| **태그드 Int** | 한 정수 안에 타입을 밴드로 인코딩하는 기법(vaislisp/vaisjq 값 모델) |
| **셀 힙** | 배열/객체를 `(key, val, next)` 병렬 리스트로 표현하는 vaisjq 저장소 |
| **수렴(gen2 == gen3)** | core로 자신을 컴파일한 결과가 다음 세대와 바이트 동일 — 자기호스팅 안정성 |

---

## 11. 더 읽을 문서

| 문서 | 내용 |
|---|---|
| `docs/reference/LANGUAGE.md` | 문법·타입·검증 상태 레퍼런스(정본) |
| `std/PRELUDE.md` | 호스트 API·설치형 도구 상태 표 |
| `examples/README.md` | 예제 423개 색인 |
| `CHANGELOG.md` | 릴리스별 변경 이력 |
| `ROADMAP.md` / `WORKLOG.md` | 진행 중 작업 / 사이클별 기록 |
| `docs/PERF-BASELINE.md`, `docs/RELEVANCE-BASELINE.md` | 성능·검색 품질 기준선 |
