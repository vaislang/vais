# 트러블슈팅 & FAQ

이 문서는 Vais 개발 중 자주 발생하는 문제와 해결 방법, 자주 묻는 질문을 다룹니다.

## 자주 발생하는 컴파일 에러

### E001: Type mismatch (타입 불일치)

**원인**: 함수나 연산자가 기대하는 타입과 전달된 값의 타입이 다릅니다.

**예시**:
```vais
F add(x: i64, y: i64) -> i64 {
  R x + y
}

F main() {
  result := add(5, "hello")  # E001: 기대 i64, 실제 str
}
```

**해결법**:
- 타입을 명시적으로 확인하고 올바른 타입의 값을 전달하세요.
- 필요시 타입 변환을 수행하세요.

```vais
F main() {
  result := add(5, 10)  # 올바름
}
```

**특수 케이스**:
- `()` (void) vs `i64`: `store_i64()`는 void 반환이므로 `I cond { store_i64(...); 0 } E { 0 }` 형태로 작성
- match arm에서 서로 다른 타입 반환 시 phi node 충돌 가능

### E002: Undefined variable (정의되지 않은 변수)

**원인**: 선언되지 않은 변수를 사용하려고 했습니다.

**예시**:
```vais
F main() {
  print(x)  # E002: x가 정의되지 않음
}
```

**해결법**:
- 변수를 사용하기 전에 `:=`로 선언하세요.

```vais
F main() {
  x := 42
  print(x)  # 올바름
}
```

### E003: Undefined type (정의되지 않은 타입)

**원인**: 선언되지 않은 타입을 참조했습니다.

**예시**:
```vais
F process(data: MyStruct) {  # E003: MyStruct가 정의되지 않음
  # ...
}
```

**해결법**:
- 타입을 사용하기 전에 `S` (struct) 또는 `E` (enum)으로 정의하세요.
- 표준 라이브러리 타입은 `U std/xxx`로 import하세요.

```vais
S MyStruct {
  value: i64
}

F process(data: MyStruct) {  # 올바름
  # ...
}
```

### E004: Undefined function (정의되지 않은 함수)

**원인**: 선언되지 않은 함수를 호출하려고 했습니다.

**예시**:
```vais
F main() {
  result := calculate(5)  # E004: calculate가 정의되지 않음
}
```

**해결법**:
- 함수를 호출하기 전에 `F`로 정의하거나 import하세요.

```vais
F calculate(x: i64) -> i64 {
  R x * 2
}

F main() {
  result := calculate(5)  # 올바름
}
```

### E005: Not callable (호출할 수 없는 값)

**원인**: 함수가 아닌 값에 `()` 호출 구문을 사용했습니다.

**예시**:
```vais
F main() {
  x := 42
  x()  # E005: i64는 호출할 수 없음
}
```

**해결법**:
- 함수만 호출할 수 있습니다. 변수가 함수 포인터인지 확인하세요.

### E006: Argument count mismatch (인수 개수 불일치)

**원인**: 함수 호출 시 전달한 인수의 개수가 함수 정의와 다릅니다.

**예시**:
```vais
F add(x: i64, y: i64) -> i64 {
  R x + y
}

F main() {
  result := add(5)  # E006: 2개 기대, 1개 전달
}
```

**해결법**:
- 함수 시그니처에 맞춰 정확한 개수의 인수를 전달하세요.

```vais
F main() {
  result := add(5, 10)  # 올바름
}
```

### E007: Cannot infer type (타입 추론 실패)

**원인**: 컴파일러가 표현식의 타입을 추론할 수 없습니다.

**예시**:
```vais
F main() {
  x := []  # E007: 빈 배열의 타입을 추론할 수 없음
}
```

**해결법**:
- 명시적으로 타입을 지정하세요.

```vais
F main() {
  x: [i64] = []  # 올바름
}
```

### E008: Duplicate definition (중복 정의)

**원인**: 같은 이름의 함수, 변수, 타입이 중복으로 정의되었습니다.

**예시**:
```vais
F foo() { }
F foo() { }  # E008: foo가 중복 정의됨
```

**해결법**:
- 각 심볼에 고유한 이름을 사용하세요.

### E009: Immutable assign (불변 변수에 대입 시도)

**원인**: `mut` 없이 선언된 변수에 새 값을 대입하려고 했습니다.

**예시**:
```vais
F main() {
  x := 5
  x = 10  # E009: x는 불변
}
```

**해결법**:
- 변수를 변경 가능하게 선언하려면 `:= mut`를 사용하세요.

```vais
F main() {
  x := mut 5
  x = 10  # 올바름
}
```

### E010: Non-exhaustive match (패턴 매칭 불완전)

**원인**: match 표현식이 모든 가능한 케이스를 처리하지 않습니다.

**예시**:
```vais
E Status {
  Ok,
  Error,
  Pending
}

F main() {
  status := Status.Ok
  M status {
    Status.Ok => print("ok"),
    Status.Error => print("error")
    # E010: Pending 케이스가 누락됨
  }
}
```

**해결법**:
- 모든 케이스를 처리하거나 `_` (와일드카드)를 추가하세요.

```vais
F main() {
  status := Status.Ok
  M status {
    Status.Ok => print("ok"),
    Status.Error => print("error"),
    _ => print("other")  # 올바름
  }
}
```

### E016: Move after use (이동 후 사용)

**원인**: 값이 이동된 후 다시 사용하려고 했습니다.

**예시**:
```vais
F take_ownership(s: str) {
  print(s)
}

F main() {
  x := "hello"
  take_ownership(x)
  print(x)  # E016: x는 이미 이동됨
}
```

**해결법**:
- 복사 가능한 타입을 사용하거나, 참조를 전달하거나, 값을 복제하세요.

### E022: Use after move (이동된 값 사용)

**원인**: 소유권이 이동된 값을 다시 사용하려고 했습니다.

**예시**:
```vais
F main() {
  s := "test"
  len := strlen(s)
  memcpy_str(dest, src, s)  # E022: s는 이미 strlen에서 move됨
}
```

**해결법**:
- `str_to_ptr(s)`로 포인터 변환 후 사용하거나, 순서를 조정하세요.

```vais
F main() {
  s := "test"
  ptr := str_to_ptr(s)
  len := calculate_length(ptr)
  # 포인터 기반 연산 사용
}
```

### E024: Assign while borrowed (차용 중 대입)

**원인**: 값이 차용(borrow)된 상태에서 대입을 시도했습니다.

**예시**:
```vais
F main() {
  x := mut 5
  r := &x
  x = 10  # E024: x가 r에 의해 차용됨
  print(r)
}
```

**해결법**:
- 차용이 끝난 후 대입하세요.

### E026: Borrow conflict (차용 충돌)

**원인**: 불변 차용과 가변 차용이 동시에 존재하거나, 가변 차용이 여러 개 존재합니다.

**해결법**:
- 차용 스코프를 조정하여 충돌을 피하세요.

### E030: No such field (존재하지 않는 필드)

**원인**: 구조체에 존재하지 않는 필드에 접근하려고 했습니다.

**예시**:
```vais
S Point {
  x: i64,
  y: i64
}

F main() {
  p := Point { x: 1, y: 2 }
  print(p.z)  # E030: Point에 z 필드가 없음
}
```

**해결법**:
- 올바른 필드 이름을 사용하거나, 필요하면 구조체 정의에 필드를 추가하세요.

```vais
F main() {
  p := Point { x: 1, y: 2 }
  print(p.x)  # 올바름
}
```

## 링크 에러

### undefined symbol

**원인**: LLVM IR에서 참조한 함수가 링크 단계에서 찾을 수 없습니다.

**예시**:
```
undefined symbol: _my_external_function
```

**해결법**:
- C 함수는 `extern F`로 선언하세요.
- 라이브러리 링크가 필요하면 `-l` 플래그를 추가하세요.

```vais
extern F puts(s: i64) -> i64

F main() {
  puts(str_to_ptr("hello"))
}
```

컴파일 시:
```bash
vaisc myfile.vais -o myfile
```

### LLVM not found

**원인**: LLVM 17이 시스템에 설치되지 않았거나 PATH에 없습니다.

**해결법**:

**macOS (Homebrew)**:
```bash
brew install llvm@17
export PATH="/opt/homebrew/opt/llvm@17/bin:$PATH"
export LDFLAGS="-L/opt/homebrew/opt/llvm@17/lib"
export CPPFLAGS="-I/opt/homebrew/opt/llvm@17/include"
```

**Ubuntu/Debian**:
```bash
sudo apt-get update
sudo apt-get install llvm-17 llvm-17-dev clang-17
```

**빌드 후 확인**:
```bash
llvm-config --version  # 17.x.x 출력 확인
cargo build
```

### clang linking errors

**원인**: clang이 생성된 LLVM IR을 실행 파일로 링크하지 못했습니다.

**해결법**:
- clang이 설치되어 있는지 확인하세요.
- 플랫폼별 링커 옵션이 필요할 수 있습니다.

```bash
which clang  # clang 경로 확인
```

## 런타임 에러

### Segmentation fault (Segfault)

**원인**:
- 널 포인터 역참조
- 배열 범위를 벗어난 접근
- 잘못된 메모리 주소 접근
- 해제된 메모리 접근

**디버깅**:
```bash
# LLDB (macOS)
lldb ./myprogram
run
bt  # 백트레이스

# GDB (Linux)
gdb ./myprogram
run
bt
```

**예방법**:
- 포인터 사용 전 널 체크
- 배열 인덱스 범위 검증
- 메모리 해제 후 포인터를 다시 사용하지 않기

```vais
F safe_access(arr: [i64], idx: i64) -> i64 {
  I idx < 0 || idx >= arr.len() {
    R 0  # 기본값 반환
  }
  R arr[idx]
}
```

### Stack overflow

**원인**: 무한 재귀 또는 너무 깊은 재귀 호출.

**예시**:
```vais
F infinite() {
  @()  # 자기 재귀, 종료 조건 없음
}
```

**해결법**:
- 재귀 함수에 기저 조건(base case)을 추가하세요.

```vais
F factorial(n: i64) -> i64 {
  I n <= 1 {
    R 1  # 기저 조건
  }
  R n * @(n - 1)
}
```

### Memory leaks

**원인**: `malloc`으로 할당한 메모리를 `free`하지 않았습니다.

**예방법**:
- RAII 패턴 사용
- 스코프가 끝날 때 리소스 해제
- GC 크레이트 활성화 고려 (`vais-gc`)

## 자주 묻는 질문 (FAQ)

### `:=` vs `=`의 차이는?

- `:=` (콜론-등호): 새 변수 선언 및 바인딩 (let binding)
- `=` (등호): 기존 변수에 새 값 대입 (reassignment)

```vais
F main() {
  x := 5      # 새 변수 선언
  x = 10      # E009: x는 불변이므로 에러

  y := mut 5  # 가변 변수 선언
  y = 10      # 올바름: 재할당 가능
}
```

### `mut` 없이 재할당하면?

**E009 에러**가 발생합니다. Vais는 기본적으로 불변(immutable)입니다.

```vais
F main() {
  counter := 0
  counter = counter + 1  # E009 에러
}
```

**해결법**: `:= mut`로 선언하세요.

```vais
F main() {
  counter := mut 0
  counter = counter + 1  # 올바름
}
```

### `E` 키워드가 else와 enum 둘 다?

네, 문맥에 따라 달라집니다.

**else로 사용**:
```vais
I x > 0 {
  print("positive")
} E {
  print("non-positive")
}
```

**enum으로 사용**:
```vais
E Status {
  Ok,
  Error
}
```

파서가 문맥을 통해 자동으로 구분합니다.

### `@` 자기 재귀란?

`@`는 현재 함수를 재귀 호출하는 연산자입니다. 함수 이름을 반복하지 않아도 됩니다.

```vais
F factorial(n: i64) -> i64 {
  I n <= 1 {
    R 1
  }
  R n * @(n - 1)  # factorial(n - 1)과 동일
}
```

**장점**:
- 함수 이름 변경 시 재귀 호출 수정 불필요
- 코드가 간결해짐

### Vais에 GC(가비지 컬렉션)가 있나요?

**기본적으로 없습니다**. Vais는 수동 메모리 관리(`malloc`/`free`)를 사용합니다.

**옵션**:
- `vais-gc` 크레이트를 활성화하면 선택적 GC 사용 가능
- 소유권(ownership) 시스템이 메모리 안전성을 제공

```vais
# 수동 관리
F main() {
  ptr := malloc(1024)
  # ... 사용 ...
  free(ptr)
}
```

### LLVM 버전 요구사항은?

**LLVM 17**이 필요합니다. Vais는 `inkwell 0.4` (LLVM 17 바인딩)를 사용합니다.

확인 방법:
```bash
llvm-config --version
# 17.x.x 출력되어야 함
```

다른 버전 사용 시 빌드 에러가 발생할 수 있습니다.

### Rust와 Vais의 차이는?

| 특징 | Vais | Rust |
|------|------|------|
| 키워드 | 단일 문자 (`F`, `S`, `E`, `I`) | 다중 문자 (`fn`, `struct`, `enum`, `if`) |
| 타입 추론 | 완전 추론 (100%) | 거의 완전 (일부 명시 필요) |
| 메모리 | 수동 + 소유권 | 소유권 + 빌림 검사기 |
| 백엔드 | LLVM IR 직접 생성 | LLVM IR via rustc |
| 재귀 | `@` 연산자 | 함수 이름 직접 호출 |
| 목표 | AI 친화적 시스템 언어 | 안전한 시스템 언어 |

### 반복문에서 `R`과 `B`의 차이는?

- `R` (return): 함수 전체에서 반환 (루프 포함)
- `B` (break): 현재 루프만 탈출

```vais
F find_value(arr: [i64], target: i64) -> i64 {
  i := mut 0
  L {
    I i >= arr.len() {
      B  # 루프 탈출
    }
    I arr[i] == target {
      R i  # 함수에서 반환
    }
    i = i + 1
  }
  R -1  # 루프 후 기본 반환값
}
```

**주의**: `I cond { R 0 }` 패턴은 루프를 벗어나는 게 아니라 함수를 종료합니다.

### 구조체 리터럴과 if 블록 구분은?

파서는 첫 문자가 대문자(A-Z)인지로 구분합니다.

```vais
# 구조체 리터럴 (첫 문자 대문자)
point := Point { x: 1, y: 2 }

# if 블록 (소문자 변수)
I condition {
  # ...
}
```

### 연산자 우선순위는?

**비교 (`==`, `!=`) > 비트 연산 (`&`, `|`)**

```vais
# 주의: 괄호 필요
I (val >> bit) & 1 == 1 {  # 잘못된 파싱
  # ...
}

# 올바른 방법:
masked := (val >> bit) & 1
I masked == 1 {
  # ...
}
```

### 표준 라이브러리 import가 E2E에서 안 되는데?

E2E 테스트에서는 `U std/xxx` import가 제한될 수 있습니다.

**해결법**: 필요한 로직을 테스트 파일에 인라인으로 작성하세요.

```vais
# import 대신
# U std/math

# 직접 구현
F abs(x: i64) -> i64 {
  I x < 0 { R -x }
  R x
}
```

### 더 많은 도움이 필요하면?

- **문서**: `docs-site/` 디렉토리의 다른 가이드 참조
- **예제**: `examples/` 디렉토리에 105+ 예제 파일
- **테스트**: `crates/vaisc/tests/` — 396개 E2E 테스트 참고
- **GitHub Issues**: vaislang/vais 저장소에 이슈 제출

## 추가 리소스

- **LLVM IR 가이드**: docs-site/src/llvm-ir-guide.md
- **타입 시스템**: docs-site/src/type-system.md
- **언어 스펙**: docs-site/src/language-spec.md
- **컴파일러 구조**: docs-site/src/compiler-architecture.md
