# 매크로 시스템

Vais는 선언적 매크로(declarative macros)를 지원합니다. 매크로는 코드 생성을 위한 패턴 매칭 기반 변환 시스템입니다.

## 기본 문법

```vais
macro name! {
    (pattern) => { template }
    (pattern) => { template }
}
```

매크로는 `macro` 키워드로 정의하며, 이름 뒤에 `!`를 붙입니다. 여러 규칙(rule)을 가질 수 있으며, 입력 토큰과 매칭되는 첫 번째 규칙이 적용됩니다.

## 매크로 정의

### 단순 매크로

```vais
macro hello! {
    () => { puts("Hello, Vais!") }
}

F main() -> i64 {
    hello!()
    0
}
```

### 파라미터가 있는 매크로

```vais
macro max! {
    ($a:expr, $b:expr) => {
        I $a > $b { $a } E { $b }
    }
}

F main() -> i64 {
    result := max!(10, 20)   # 20
    result
}
```

## 메타변수 (Fragment Specifiers)

메타변수는 `$name:kind` 형태로 선언합니다:

| 지정자 | 설명 | 예시 |
|--------|------|------|
| `expr` | 표현식 | `$x:expr` -- `1 + 2`, `foo()` |
| `ty` | 타입 | `$t:ty` -- `i64`, `Vec<i64>` |
| `ident` | 식별자 | `$name:ident` -- `foo`, `my_var` |
| `pat` | 패턴 | `$p:pat` -- `Some(x)`, `_` |
| `stmt` | 문장 | `$s:stmt` -- `x := 5` |
| `block` | 블록 | `$b:block` -- `{ expr }` |
| `item` | 아이템 | `$i:item` -- 함수, 구조체 정의 |
| `lit` | 리터럴 | `$l:lit` -- `42`, `"hello"` |
| `tt` | 토큰 트리 | `$t:tt` -- 임의의 단일 토큰 또는 그룹 |

## 반복 (Repetition)

매크로에서 가변 개수의 인자를 받으려면 반복 패턴을 사용합니다:

### `*` (0회 이상)

```vais
macro vec! {
    () => { Vec::new() }
    ($($item:expr),*) => {
        Vec::from([$($item),*])
    }
}

F main() -> i64 {
    empty := vec!()
    nums := vec!(1, 2, 3, 4, 5)
    0
}
```

### `+` (1회 이상)

```vais
macro sum! {
    ($first:expr $(, $rest:expr)+) => {
        $first $(+ $rest)+
    }
}

F main() -> i64 {
    result := sum!(1, 2, 3, 4, 5)   # 15
    result
}
```

### `?` (0회 또는 1회)

```vais
macro optional_return! {
    ($val:expr $(, $msg:expr)?) => {
        $($msg ;)?
        $val
    }
}
```

## 구분자 (Delimiters)

매크로 호출에는 세 가지 구분자를 사용할 수 있습니다:

```vais
macro_name!(...)     # 소괄호
macro_name![...]     # 대괄호
macro_name!{...}     # 중괄호
```

## 공개 매크로

`P` 키워드로 다른 모듈에서 사용 가능하게 합니다:

```vais
P macro assert_eq! {
    ($left:expr, $right:expr) => {
        I $left != $right {
            puts("Assertion failed!")
        }
    }
}
```

## 실전 예제

### Debug 출력 매크로

```vais
macro dbg! {
    ($val:expr) => {
        puts("dbg:")
        print_i64($val)
        $val
    }
}
```

### 조건부 실행

```vais
macro when! {
    ($cond:expr, $body:block) => {
        I $cond $body
    }
}

F main() -> i64 {
    x := 42
    when!(x > 0, {
        puts("positive")
    })
    0
}
```

## 주의사항

- 매크로는 컴파일 타임에 확장됩니다. 런타임 오버헤드가 없습니다.
- 매크로 확장 결과는 일반 코드와 동일하게 타입 검사됩니다.
- 재귀 매크로를 사용할 때 확장 깊이 제한에 주의하세요.
- 복잡한 로직에는 매크로 대신 제네릭 함수를 사용하는 것을 권장합니다.
