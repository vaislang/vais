# Defer 문 (D)

`D` 키워드는 현재 스코프가 종료될 때 실행할 표현식을 등록합니다. 리소스 정리, 파일 닫기, 잠금 해제 등에 유용합니다.

## 기본 문법

```vais
D expr
```

`D` 뒤에 오는 표현식은 현재 블록 스코프가 끝날 때 자동으로 실행됩니다.

## LIFO 실행 순서

여러 `D` 문이 있을 경우 **LIFO(Last In, First Out)** 순서로 실행됩니다. 나중에 등록된 defer가 먼저 실행됩니다:

```vais
F main() -> i64 {
    D puts("third")
    D puts("second")
    D puts("first")

    puts("main body")
    0
}
# 출력:
# main body
# first
# second
# third
```

## 리소스 정리 패턴

### 파일 핸들

```vais
F read_file(path: str) -> i64 {
    fd := open(path, 0)
    D close(fd)

    # fd를 사용한 작업...
    # 함수가 어떻게 종료되든 close(fd)가 호출됨
    0
}
```

### 메모리 해제

```vais
F process() -> i64 {
    buf := malloc(1024)
    D free(buf)

    # buf를 사용한 작업...
    # 스코프 종료 시 자동 해제
    0
}
```

## 에러 핸들링과의 연계

`D`는 조기 반환(`R`)이나 에러 상황에서도 실행이 보장됩니다:

```vais
F safe_operation() -> i64 {
    resource := acquire()
    D release(resource)

    I check_error() {
        R -1   # 에러로 조기 반환해도 release()가 실행됨
    }

    do_work(resource)
    0
}
```

## 블록 스코프

`D`는 선언된 블록 스코프에 바인딩됩니다:

```vais
F main() -> i64 {
    D puts("outer defer")

    {
        D puts("inner defer")
        puts("inner block")
    }
    # "inner defer" 출력됨 (내부 블록 종료)

    puts("outer block")
    0
}
# "outer defer"는 main 함수 종료 시 출력
```

## 주의사항

- `D`는 현재 스코프의 끝에서 실행됩니다 (함수 끝이 아닌 블록 끝).
- 여러 defer는 LIFO 순서로 실행됩니다.
- defer 내에서 에러가 발생해도 나머지 defer는 계속 실행됩니다.
- defer 표현식은 등록 시점의 변수 값을 캡처합니다.
