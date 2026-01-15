# Vais 2.0 Language Design

## 목표

```
1. 성능: C/Rust 동급 (LLVM 네이티브 컴파일)
2. 토큰: Rust 대비 50-70% 절약
3. AI 최적화: 모호성 제로, 파싱 용이
4. 실행: .vais → 네이티브 바이너리
```

## 설계 원칙

### 1. AI-First (사람 가독성 < 토큰 효율)

```
❌ 사람 친화적: fn fibonacci(number: i64) -> i64 { ... }
✅ AI 친화적:   F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)
```

### 2. 명시적 (모호성 제로)

```
❌ Python: 들여쓰기로 블록 구분 (모호함)
❌ Lisp: 괄호만으로 모든 것 표현 (파싱 어려움)
✅ Vais: 명확한 구분자, 최소 키워드
```

### 3. 정적 타입 + 추론

```
❌ 동적: x = 1 (런타임 타입 결정)
✅ 정적: x := 1 (컴파일 타임 i64 추론)
✅ 명시: x: i64 = 1
```

## 문법 스펙

### 타입 시스템

```
기본 타입:
  i8 i16 i32 i64 i128    # 정수
  u8 u16 u32 u64 u128    # 부호 없는 정수
  f32 f64                 # 부동소수점
  bool                    # 불리언
  str                     # 문자열 (UTF-8)
  ()                      # Unit (void)

복합 타입:
  [T]                     # 배열
  [K:V]                   # 맵
  (T1,T2,...)             # 튜플
  T?                      # Optional
  T!                      # Result
  *T                      # 포인터 (unsafe)
  &T                      # 참조 (RC)
```

### 변수 선언

```
x := 1          # 타입 추론 (불변)
x: i64 = 1      # 명시적 타입 (불변)
x := mut 1      # 가변
x: mut i64 = 1  # 명시적 + 가변
```

### 함수 정의

```
# 한 줄 함수
F add(a:i64,b:i64)->i64=a+b

# 재귀 호출: @ = 현재 함수
F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)

# 블록 함수
F process(data:[i64])->i64{
  sum:=0
  L i:data{sum+=i}  # Loop
  sum
}

# 제네릭
F swap<T>(a:&T,b:&T){t:=*a;*a=*b;*b=t}
```

### 제어 흐름

```
# 조건문 (삼항 연산자 확장)
x<0?-1:x>0?1:0

# If 블록
I x<0{-1}E x>0{1}E{0}

# Loop
L i:0..10{print(i)}     # range
L item:list{print(item)} # iterator
L{break?condition}       # while

# Match
M value{
  0=>zero
  1..10=>small
  _=>other
}
```

### 구조체 / 열거형

```
# Struct
S Point{x:f64,y:f64}

# Enum
E Option<T>{Some(T),None}
E Result<T,E>{Ok(T),Err(E)}

# 메서드
S Point{x:f64,y:f64}
  F len(&self)->f64=(self.x*self.x+self.y*self.y).sqrt()
```

### 메모리 관리: Reference Counting

```
# 자동 참조 카운팅
x:=Point{x:1.0,y:2.0}   # RC=1
y:=x                     # RC=2 (얕은 복사)
z:=x.clone()            # 새 객체, RC=1

# 순환 참조 방지: weak reference
w:=weak(x)              # weak ref
```

### 에러 처리

```
# Result 타입
F read(path:str)->str!{
  f:=open(path)?         # ? = 에러 전파
  f.read()
}

# 또는 panic
F must_read(path:str)->str{
  open(path)!            # ! = panic if error
}
```

### 동시성

```
# Async/Await
A fetch(url:str)->str!{
  resp:=http.get(url).await?
  resp.body()
}

# Spawn
task:=spawn{heavy_compute()}
result:=task.await
```

## 토큰 효율 비교

### Fibonacci

```
[Rust - 78 tokens]
fn fib(n: i64) -> i64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

[Vais - 32 tokens]
F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)

절약: 59%
```

### HTTP Server

```
[Rust + Axum - ~180 tokens]
use axum::{routing::get, Router};
async fn hello() -> &'static str { "Hello" }
#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service()).await.unwrap();
}

[Vais - ~50 tokens]
use net
A main(){
  serve(3000){
    G "/"=>"Hello"
  }.await
}

절약: 72%
```

## 키워드 목록 (최소화)

```
F    = function
S    = struct
E    = enum
I    = if
E    = else (context-dependent)
L    = loop
M    = match
A    = async
R    = return
B    = break
C    = continue
T    = type alias
U    = use/import
P    = pub (public)
```

## 연산자

```
산술: + - * / %
비교: < > <= >= == !=
논리: & | !
비트: << >> ^ ~
할당: = += -= *= /= :=
참조: & * @
제어: ? ! => -> ..
```

## 컴파일 파이프라인

```
.vais 소스
    ↓
[Lexer] → 토큰
    ↓
[Parser] → AST
    ↓
[TypeCheck] → Typed AST
    ↓
[IR Gen] → Vais IR
    ↓
[LLVM Gen] → LLVM IR
    ↓
[LLVM Opt] → 최적화된 IR
    ↓
[LLVM CodeGen] → 네이티브
    ↓
바이너리 (.exe / ELF / Mach-O)
```

## 파일 확장자

```
.vais    소스 코드
.vmod    모듈 정의
.vlib    컴파일된 라이브러리
```
